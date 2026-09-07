"""Provider-free qualification of the private Linux preview execution boundary."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time
import unittest
from unittest import mock
import uuid

import executor


BWRAP = Path('/usr/bin/bwrap')
BWRAP_SHA = 'eabbccb0f7f755b96d30834026a9b5d941c606400d097d87c1ff16622edaf68c'


def live_tag(tag):
    found = []
    for item in Path('/proc').iterdir():
        if not item.name.isdigit():
            continue
        try:
            if tag.encode() in (item / 'cmdline').read_bytes():
                found.append(int(item.name))
        except OSError:
            pass
    return found


class ExecutorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-executor-')
        self.root = Path(self.temp.name)
        for name in ('source', 'build', 'scratch', 'shared', 'retained', 'config'):
            (self.root / name).mkdir()
        for name in ('shared', 'retained', 'config'):
            (self.root / name / 'sentinel').write_text('UNCHANGED')
        self.spec = {
            'bwrap': str(BWRAP), 'bwrap_sha256': BWRAP_SHA,
            'source': str(self.root / 'source'), 'build': str(self.root / 'build'),
            'scratch': str(self.root / 'scratch'),
            'view': str(self.root / 'shared'),
            'readonly': [
                {'source': str(self.root / name), 'target': '/audit/' + name}
                for name in ('shared', 'retained', 'config')
            ],
        }

    def tearDown(self):
        self.temp.cleanup()

    def run_code(self, code, *args, **kw):
        return executor.run_preview(self.spec, ['/usr/bin/python3', '-c', code, *args], **kw)

    def assert_shared(self):
        for name in ('shared', 'retained', 'config'):
            self.assertEqual((self.root / name / 'sentinel').read_text(), 'UNCHANGED')

    def test_private_writes_same_path_and_empty_environment(self):
        with mock.patch.dict(os.environ, {'C15_SECRET_SURROGATE': 'DO_NOT_INHERIT'}):
            result = self.run_code(
                'import os,pathlib,json; '
                'pathlib.Path("own").write_text("source"); '
                'pathlib.Path("/build/own").write_text("build"); '
                'pathlib.Path("/tmp/own").write_text("scratch"); '
                'print(json.dumps({"cwd":os.getcwd(),"leak":"C15_SECRET_SURROGATE" in os.environ,'
                '"home":os.environ["HOME"],"real_auth":pathlib.Path("/home/user/.codex/auth.json").exists()}))')
        self.assertEqual(result['exit_code'], 0, result)
        facts = json.loads(result['stdout'])
        self.assertEqual(facts['cwd'], self.spec['view'])
        self.assertFalse(facts['leak'])
        self.assertFalse(facts['real_auth'])
        self.assertEqual(facts['home'], '/tmp/home')
        self.assertEqual((self.root / 'source/own').read_text(), 'source')
        self.assertEqual((self.root / 'build/own').read_text(), 'build')
        self.assertEqual((self.root / 'scratch/own').read_text(), 'scratch')
        self.assertFalse((self.root / 'shared/own').exists())
        self.assert_shared()

    def test_absolute_symlink_and_child_writes_blocked(self):
        (self.root / 'source/link').symlink_to('/audit/shared/sentinel')
        result = self.run_code('''import pathlib,subprocess,json
targets=['/audit/shared/sentinel','/audit/retained/sentinel','/audit/config/sentinel','link','/usr/c15-forbidden','/dev/c15-forbidden']
blocked=[]
for target in targets:
    try: pathlib.Path(target).write_text('CORRUPTED')
    except OSError: blocked.append(target)
child=subprocess.run(['/usr/bin/python3','-c',"import pathlib; pathlib.Path('/audit/shared/sentinel').write_text('CHILD')"],capture_output=True)
print(json.dumps({'blocked':blocked,'child_exit':child.returncode}))
''')
        self.assertEqual(result['exit_code'], 0, result)
        facts = json.loads(result['stdout'])
        self.assertEqual(len(facts['blocked']), 6)
        self.assertNotEqual(facts['child_exit'], 0)
        self.assert_shared()

    def test_network_and_new_user_namespace_not_available(self):
        result = self.run_code('''import socket,subprocess,json
s=socket.socket(); s.settimeout(.2)
try: s.connect(('192.0.2.1',9)); connected=True
except OSError: connected=False
r=subprocess.run(['/usr/bin/unshare','--user','--map-root-user','/usr/bin/true'],capture_output=True)
print(json.dumps({'connected':connected,'nested_namespace_exit':r.returncode}))
''')
        self.assertEqual(result['exit_code'], 0, result)
        self.assertFalse(json.loads(result['stdout'])['connected'])
        self.assertNotEqual(json.loads(result['stdout'])['nested_namespace_exit'], 0)

    def test_timeout_kills_detached_child_and_closes_pipes(self):
        tag = 'C15_CHILD_' + uuid.uuid4().hex
        result = self.run_code(
            'import subprocess,time; subprocess.Popen(["/usr/bin/python3","-c",'
            '"import time; time.sleep(60)","' + tag + '"],start_new_session=True); time.sleep(60)',
            timeout=0.5)
        self.assertTrue(result['timed_out'])
        self.assertTrue(result['closed'])
        self.assertLess(result['elapsed_seconds'], 4)
        self.assertEqual(live_tag(tag), [])
        self.assert_shared()

    def test_cancellation_closes_child(self):
        tag = 'C15_CANCEL_' + uuid.uuid4().hex
        cancel = threading.Event()
        timer = threading.Timer(.5, cancel.set)
        timer.start()
        try:
            result = self.run_code(
                'import subprocess,time; subprocess.Popen(["/usr/bin/python3","-c",'
                '"import time; time.sleep(60)","' + tag + '"],start_new_session=True); time.sleep(60)',
                cancel=cancel, timeout=5)
        finally:
            timer.cancel()
        self.assertTrue(result['cancelled'])
        self.assertTrue(result['closed'])
        self.assertEqual(live_tag(tag), [])

    def test_normal_exit_does_not_leave_detached_child(self):
        tag = 'C15_RETURN_' + uuid.uuid4().hex
        result = self.run_code(
            'import subprocess; subprocess.Popen(["/usr/bin/python3","-c",'
            '"import time; time.sleep(60)","' + tag + '"],start_new_session=True,'
            'stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)', timeout=3)
        self.assertEqual(result['exit_code'], 0, result)
        self.assertTrue(result['closed'])
        self.assertEqual(live_tag(tag), [])

    def test_pin_alias_and_overlap_fail_before_spawn(self):
        for mutation in ('pin', 'alias', 'overlap', 'missing'):
            with self.subTest(mutation=mutation):
                spec = dict(self.spec)
                if mutation == 'pin': spec['bwrap_sha256'] = '0' * 64
                if mutation == 'alias':
                    (self.root / 'alias').symlink_to(self.root / 'source')
                    spec['source'] = str(self.root / 'alias')
                if mutation == 'overlap': spec['scratch'] = spec['source']
                if mutation == 'missing': spec['bwrap'] = str(self.root / 'missing')
                with mock.patch.object(executor.subprocess, 'Popen', side_effect=AssertionError('spawned')):
                    with self.assertRaises((ValueError, OSError)):
                        executor.run_preview(spec, ['/usr/bin/true'])

    def test_output_bound_is_explicit_not_silent_success(self):
        result = self.run_code('import sys; sys.stdout.write("x"*2000000)', max_output_bytes=4096)
        self.assertEqual(result['stop_reason'], 'output_limit')
        self.assertTrue(result['closed'])
        self.assertEqual(len(result['stdout'].encode()) + len(result['stderr'].encode()), 4096)

    def test_existing_hardlink_and_special_file_reject_before_spawn(self):
        link = self.root / 'source/alias'
        os.link(self.root / 'shared/sentinel', link)
        with mock.patch.object(executor.subprocess, 'Popen', side_effect=AssertionError('spawned')):
            with self.assertRaisesRegex(ValueError, 'hardlink'):
                executor.run_preview(self.spec, ['/usr/bin/true'])
        link.unlink()
        os.mkfifo(link)
        with mock.patch.object(executor.subprocess, 'Popen', side_effect=AssertionError('spawned')):
            with self.assertRaisesRegex(ValueError, 'special'):
                executor.run_preview(self.spec, ['/usr/bin/true'])

    def test_mount_descriptors_are_not_inherited(self):
        result = self.run_code('''import os,json
links=[]
for fd in os.listdir('/proc/self/fd'):
    if int(fd)>2:
        try: links.append(os.readlink('/proc/self/fd/'+fd))
        except FileNotFoundError: pass
print(json.dumps(links))
''')
        self.assertEqual(result['exit_code'], 0, result)
        self.assertEqual(json.loads(result['stdout']), [])

    def test_offline_cargo_same_test_red_green_and_readonly_cache(self):
        sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip()).resolve()
        self.spec['readonly'].append({'source': str(sysroot), 'target': '/toolchain'})
        cache = self.root / 'cache'
        cache.mkdir()
        (cache / 'public-marker').write_text('PUBLIC-CACHE-FIXTURE')
        cache_sha = hashlib.sha256((cache / 'public-marker').read_bytes()).hexdigest()
        self.spec['readonly'].append({'source': str(cache), 'target': '/cache'})
        source = self.root / 'source'
        (source / 'src').mkdir()
        (source / 'Cargo.toml').write_text('[package]\nname="preview-fixture"\nversion="0.0.0"\nedition="2021"\n')
        test = '#[cfg(test)] mod tests { #[test] fn required_behavior() { assert_eq!(super::value(), 2); } }\n'
        (source / 'src/lib.rs').write_text('pub fn value() -> u8 { 1 }\n' + test)
        argv = ['/usr/bin/env', 'RUSTC=/toolchain/bin/rustc', '/toolchain/bin/cargo',
                'test', '--offline', '--lib', 'required_behavior', '--', '--exact', 'tests::required_behavior']
        red = executor.run_preview(self.spec, argv, timeout=20)
        self.assertEqual(red['exit_code'], 101, red)
        self.assertIn('tests::required_behavior ... FAILED', red['stdout'])
        self.assertIn('left: 1', red['stdout'])
        self.assertEqual((source / 'src/lib.rs').read_text().split('\n', 1)[1], test)
        (source / 'src/lib.rs').write_text('pub fn value() -> u8 { 2 }\n' + test)
        green = executor.run_preview(self.spec, argv, timeout=20)
        self.assertEqual(green['exit_code'], 0, green)
        self.assertIn('tests::required_behavior ... ok', green['stdout'])
        self.assertEqual((source / 'src/lib.rs').read_text().split('\n', 1)[1], test)
        check = self.run_code('''import pathlib,json
before=pathlib.Path('/cache/public-marker').read_text()
try: pathlib.Path('/cache/public-marker').write_text('BAD'); blocked=False
except OSError: blocked=True
print(json.dumps({'before':before,'blocked':blocked}))
''')
        self.assertEqual(check['exit_code'], 0, check)
        self.assertEqual(json.loads(check['stdout']), {'before': 'PUBLIC-CACHE-FIXTURE', 'blocked': True})
        self.assertEqual(hashlib.sha256((cache / 'public-marker').read_bytes()).hexdigest(), cache_sha)
        self.assert_shared()


if __name__ == '__main__':
    unittest.main()
