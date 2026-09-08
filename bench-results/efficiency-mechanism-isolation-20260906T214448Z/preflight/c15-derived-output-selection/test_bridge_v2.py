"""Private bridge fixtures use actual Git/patch/bwrap, never a provider."""
import copy
import hashlib
import json
import os
from pathlib import Path
import tempfile
import threading
import time
import unittest
from unittest import mock
import shutil
import subprocess
import sys

import bridge_v2 as B


HERE = Path(__file__).resolve().parent
PRE = HERE.parent


def ref(path):
    return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


class BridgeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='c15-bridge-test-')
        self.root = Path(self.tmp.name)
        self.live = self.root / 'live'; self.live.mkdir()
        self.owned = self.root / 'preview-operation'; self.owned.mkdir()
        self.m = B.load_helper('materializer', B.DEFAULT_HELPERS['materializer'])
        self.m.git(self.live, ['init', '-q'])
        self.m.git(self.live, ['config', 'user.name', 'Fixture'])
        self.m.git(self.live, ['config', 'user.email', 'fixture@example.invalid'])
        (self.live / 'value.txt').write_text('old\n')
        (self.live / 'guidance.txt').write_text('base\n')
        self.m.git(self.live, ['add', '.']); self.m.git(self.live, ['commit', '-qm', 'fixture'])
        self.toolchain = Path('/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu')
        self.config = {
            'schema': 'c15-runtime-bridge-config-v2', 'helpers': copy.deepcopy(B.DEFAULT_HELPERS),
            'derived_output_roots': [],
            'executables': {
                'git': ref(Path('/usr/bin/git')), 'bwrap': ref(Path('/usr/bin/bwrap')),
                'patch_driver': ref(PRE / 'c15-private-materialization/target/debug/c15-patch-driver'),
                'toolchain_cargo': ref(self.toolchain / 'bin/cargo'),
                'toolchain_rustc': ref(self.toolchain / 'bin/rustc')},
            'toolchain_root': str(self.toolchain),
            'cargo_capsule': ref(PRE / 'c15-project-qualification/PROJECT-001/public-cargo/CAPSULE.json'),
            'overlays': [], 'timeout_seconds': 3, 'max_output_bytes': 65536,
            'operation_timeout_seconds': 60, 'qualification_implementation': False}
        self.config_path = self.root / 'CONFIG.json'; self.save_config()
        test = "test \"$(cat value.txt)\" = new\n"
        self.request = {
            'schema': 'work-leaf-private-preview-bridge-v1', 'operation': 'capture',
            'config_path': str(self.config_path), 'config_sha256': ref(self.config_path)['sha256'],
            'project_root': str(self.live), 'operation_root': str(self.owned),
            'run_id': 'fixture-run', 'agent_id': 'author', 'launch_generation': 1,
            'proposal_id': 'proposal-one', 'revision_of': None,
            'cancel_path': str(self.owned / 'CANCEL'),
            'proposal': {'format': 'edit', 'reason': 'held behavior test',
                         'body': '*** Begin Patch\n*** Add File: check.sh\n+' + test.rstrip() + '\n*** End Patch\n',
                         'test_purpose': 'expect the requested value',
                         'test_paths': ['check.sh'],
                         'command': '/bin/sh check.sh', 'lock_paths': ['.']}}

    def tearDown(self):
        self.tmp.cleanup()

    def save_config(self):
        self.config_path.write_text(json.dumps(self.config))
        if hasattr(self, 'request'): self.request['config_sha256'] = ref(self.config_path)['sha256']

    def capture(self, qualification=False):
        return B.execute(self.request, qualification=qualification)

    def request_test(self, captured):
        value = copy.deepcopy(self.request); value['operation'] = 'test'; value['selection'] = captured['selection']
        return value

    def test_missing_or_changed_dependency_fails_before_selection(self):
        self.config['executables']['git']['sha256'] = '0' * 64; self.save_config()
        with self.assertRaises(ValueError): self.capture()
        self.assertFalse((self.owned / 'capture').exists())

    def test_validate_has_no_selection_or_execution_and_create_new(self):
        request = {k: self.request[k] for k in ('schema', 'config_path', 'config_sha256', 'project_root', 'operation_root')}
        request['operation'] = 'validate'
        result = B.execute(request)
        self.assertTrue(result['closed']); self.assertIsNone(result['exit_code'])
        self.assertFalse((self.owned / 'capture').exists())
        with self.assertRaises(ValueError): B.execute(request)

    def test_actual_private_red_preserves_live_and_source_fields(self):
        before = self.m.source_identity(self.live)
        capture = self.capture(); result = B.execute(self.request_test(capture))
        self.assertEqual(result['exit_code'], 1); self.assertTrue(result['closed'])
        saved = json.loads(Path(result['result_path']).read_bytes())
        self.assertEqual(saved['snapshot']['selected_origin']['live_root'], str(self.live))
        self.assertNotEqual(saved['snapshot']['shared']['root'], str(self.live))
        self.assertEqual(saved['command_execution']['cwd'], str(self.live))
        self.assertTrue(saved['patch']['applied_privately']); self.assertFalse(saved['patch']['shared_accepted'])
        self.assertEqual(self.m.source_identity(self.live), before)
        self.assertFalse((self.live / 'check.sh').exists())
        with self.assertRaises(ValueError): B.execute(self.request_test(capture))

    def test_selected_old_tree_survives_later_live_commit(self):
        capture = self.capture()
        (self.live / 'value.txt').write_text('new\n'); self.m.git(self.live, ['add', '.']); self.m.git(self.live, ['commit', '-qm', 'later'])
        result = B.execute(self.request_test(capture))
        self.assertEqual(result['exit_code'], 1)
        self.assertEqual((self.live / 'value.txt').read_text(), 'new\n')

    def test_both_overlay_flags_retained_and_installed_privately(self):
        head = self.m.git(self.live, ['rev-parse', 'HEAD']).decode().strip()
        row = {'path': 'guidance.txt', 'base_blob': self.m.git(self.live, ['rev-parse', head + ':guidance.txt']).decode().strip(),
               'base_sha256': hashlib.sha256(b'base\n').hexdigest(), 'base_mode': '100644',
               'effective_text': 'effective\n', 'effective_sha256': hashlib.sha256(b'effective\n').hexdigest(),
               'effective_mode': '100644', 'index_flags': {'skip_worktree': True, 'assume_unchanged': True}}
        (self.live / 'guidance.txt').write_text(row['effective_text'])
        self.m.git(self.live, ['update-index', '--skip-worktree', 'guidance.txt'])
        self.m.git(self.live, ['update-index', '--assume-unchanged', 'guidance.txt'])
        self.config['overlays'] = [row]; self.save_config()
        capture = self.capture(); result = B.execute(self.request_test(capture))
        saved = json.loads(Path(result['result_path']).read_bytes())
        self.assertEqual(saved['overlay_adapter']['original_declarations'], [row])
        self.assertEqual(Path(saved['snapshot']['repo'], 'guidance.txt').read_text(), row['effective_text'])
        self.assertEqual((self.live / 'guidance.txt').read_text(), row['effective_text'])

    def test_changed_selection_identity_or_owner_rejected(self):
        capture = self.capture(); request = self.request_test(capture); request['agent_id'] = 'other'
        with self.assertRaises(ValueError): B.execute(request)
        self.assertFalse((self.owned / 'test').exists())

    def test_host_test_paths_reject_boolean_and_missing_afterimage(self):
        bad = copy.deepcopy(self.request); bad['proposal']['test_paths'][0] = False
        with self.assertRaises(ValueError): B.execute(bad)
        self.request['proposal']['test_paths'][0] = 'missing.sh'
        capture = self.capture(); result = B.execute(self.request_test(capture))
        self.assertIsNone(result['exit_code']); self.assertEqual(result['stop_reason'], 'preparation_failed')
        self.assertFalse(json.loads(Path(result['result_path']).read_bytes()).get('command_execution'))

    def test_cancelled_command_returns_closed_not_success(self):
        self.request['proposal']['command'] = '/bin/sleep 10'
        capture = self.capture(); request = self.request_test(capture)
        timer = threading.Timer(.5, lambda: Path(request['cancel_path']).touch()); timer.start()
        try: result = B.execute(request)
        finally: timer.join()
        self.assertTrue(result['closed']); self.assertEqual(result['stop_reason'], 'cancelled')

    def test_runtime_rejects_qualification_and_implementation(self):
        self.config['qualification_implementation'] = True; self.save_config()
        with self.assertRaises(ValueError): self.capture()
        bad = copy.deepcopy(self.request); bad['operation'] = 'implementation'
        with self.assertRaises(ValueError): B.execute(bad)

    def test_qualification_red_green_keeps_same_command_and_test(self):
        self.config['qualification_implementation'] = True; self.save_config()
        captured = self.capture(qualification=True); red = B.execute(self.request_test(captured), qualification=True)
        request = copy.deepcopy(self.request); request['operation'] = 'implementation'
        request['prior_test'] = {'path': red['result_path'], 'sha256': red['result_sha256']}
        request['implementation'] = {'format': 'patch', 'reason': 'implement value',
                                     'body': '--- a/value.txt\n+++ b/value.txt\n@@ -1 +1 @@\n-old\n+new\n'}
        green = B.execute(request, qualification=True)
        self.assertEqual((red['exit_code'], green['exit_code']), (1, 0))
        saved = json.loads(Path(green['result_path']).read_bytes())
        self.assertEqual(saved['request']['proposal']['command'], self.request['proposal']['command'])
        self.assertEqual(saved['declared_test_afterimages'], json.loads(Path(red['result_path']).read_bytes())['declared_test_afterimages'])
        self.assertEqual((self.live / 'value.txt').read_text(), 'old\n')

    def test_main_create_new_io_and_source_loader_avoids_pyc(self):
        request = {k: self.request[k] for k in ('schema', 'config_path', 'config_sha256', 'project_root', 'operation_root')}
        request['operation'] = 'validate'; inp = self.owned / 'input.json'; out = self.owned / 'output.json'
        inp.write_text(json.dumps(request)); self.assertEqual(B.main([str(inp), str(out)]), 0)
        first = out.read_bytes(); self.assertNotEqual(B.main([str(inp), str(out)]), 0); self.assertEqual(out.read_bytes(), first)
        module = B.load_helper('executor', B.DEFAULT_HELPERS['executor'])
        self.assertEqual(module.__file__, B.DEFAULT_HELPERS['executor']['path'])

    def test_loaded_source_identity_mismatch_fails_before_work(self):
        with mock.patch.object(B, '__compiled_sha256__', '0' * 64, create=True):
            with self.assertRaises(ValueError): self.capture()
        self.assertFalse((self.owned / 'CAPTURE-ATTEMPT.json').exists())

    def test_undeclared_empty_capsule_directory_rejected(self):
        source = Path(self.config['cargo_capsule']['path']).parent
        target = self.root / 'capsule'; shutil.copytree(source, target)
        saved = json.loads((target / 'CAPSULE.json').read_bytes()); saved['root'] = str(target)
        (target / 'CAPSULE.json').write_text(json.dumps(saved)); (target / 'unlisted').mkdir()
        self.config['cargo_capsule'] = ref(target / 'CAPSULE.json'); self.save_config()
        with self.assertRaises(ValueError): self.capture()

    def test_git_timeout_publishes_failure_without_execution(self):
        original = B.bind_modules
        def timeout(*args):
            m, p, l, run = original(*args)
            m.git = mock.Mock(side_effect=subprocess.TimeoutExpired('git', 30))
            return m, p, l, run
        with mock.patch.object(B, 'bind_modules', side_effect=timeout):
            result = self.capture()
        self.assertEqual(result['status'], 'failed'); self.assertIsNone(result['exit_code'])
        self.assertTrue(Path(result['result_path']).exists())

    def test_failure_still_verifies_source_endpoints(self):
        original = B.bind_modules
        def failure(*args):
            m, p, l, run = original(*args)
            m.git = mock.Mock(side_effect=ValueError('retained source failure'))
            return m, p, l, run
        with mock.patch.object(B, 'bind_modules', side_effect=failure): result = self.capture()
        saved = json.loads(Path(result['result_path']).read_bytes())
        self.assertTrue(saved['source_endpoints_match'])
        self.assertEqual(saved['error']['message'], 'retained source failure')

    def test_executor_exception_never_claims_closed(self):
        captured = self.capture(); original = B.validate_config
        def inject(*args):
            config, modules, capsule, pins = original(*args)
            modules['executor'].run_preview = mock.Mock(side_effect=subprocess.TimeoutExpired('uncertain-child', 2))
            return config, modules, capsule, pins
        with mock.patch.object(B, 'validate_config', side_effect=inject): result = B.execute(self.request_test(captured))
        self.assertFalse(result['closed']); self.assertEqual(result['status'], 'failed')

    def test_protocol_identity_alphabet_matches_owned_ids(self):
        self.request.update(run_id='run.01', proposal_id='proposal.01', revision_of='previous.01', agent_id='author:λ')
        result = self.capture(); self.assertEqual(result['status'], 'completed')

    def test_source_drift_on_failed_operation_retains_both_errors(self):
        original = B.bind_modules
        def failure(*args):
            m, p, l, run = original(*args)
            def mutate(*_args, **_kwargs):
                self.config_path.write_text(self.config_path.read_text() + '\n')
                raise ValueError('initial failure')
            m.git = mutate
            return m, p, l, run
        with mock.patch.object(B, 'bind_modules', side_effect=failure): result = self.capture()
        saved = json.loads(Path(result['result_path']).read_bytes())
        self.assertFalse(saved['source_endpoints_match'])
        self.assertTrue(saved['source_endpoint_errors'])
        self.assertEqual(saved['error']['message'], 'initial failure')

    def test_isolated_exact_source_bootstrap_cli(self):
        request = {k: self.request[k] for k in ('schema', 'config_path', 'config_sha256', 'project_root', 'operation_root')}
        request['operation'] = 'validate'
        inp = self.owned / 'validate-input.json'; out = self.owned / 'validate-output.json'
        inp.write_text(json.dumps(request))
        script = ('import hashlib,pathlib,sys,types; '
                  'path=sys.argv[1]; body=pathlib.Path(path).read_bytes(); '
                  'assert hashlib.sha256(body).hexdigest()==sys.argv[2]; '
                  'm=types.ModuleType("verified_bridge"); m.__file__=path; '
                  'm.__compiled_sha256__=sys.argv[2]; exec(compile(body,path,"exec"),m.__dict__); '
                  'sys.exit(m.main(sys.argv[3:]))')
        result = subprocess.run([str(Path(sys.executable).resolve()), '-I', '-B', '-c', script,
            str(HERE / 'bridge_v2.py'), ref(HERE / 'bridge_v2.py')['sha256'], str(inp), str(out)],
            env={}, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        response = json.loads(out.read_bytes()); self.assertEqual(response['status'], 'completed')
        saved = json.loads(Path(response['result_path']).read_bytes())
        self.assertEqual(saved['source_sha256'][str(HERE / 'bridge_v2.py')], ref(HERE / 'bridge_v2.py')['sha256'])

    def test_duplicate_input_json_key_rejected_before_attempt(self):
        inp = self.owned / 'bad-input.json'; out = self.owned / 'bad-output.json'
        inp.write_text('{"operation_root":' + json.dumps(str(self.owned)) + ',"schema":"x","schema":"y"}')
        self.assertEqual(B.main([str(inp), str(out)]), 2)
        self.assertFalse(out.exists()); self.assertFalse((self.owned / 'VALIDATE-ATTEMPT.json').exists())

    def test_dangling_result_path_rejected_before_private_work(self):
        captured = self.capture()
        (self.owned / 'TEST-RESULT.json').symlink_to(self.owned / 'absent-target')
        with self.assertRaises(ValueError): B.execute(self.request_test(captured))
        self.assertFalse((self.owned / 'TEST-ATTEMPT.json').exists())
        self.assertFalse((self.owned / 'test').exists())


    def derived(self):
        (self.live / '.gitignore').write_text('build-cache/\nunknown-cache/\n')
        self.m.git(self.live, ['add', '.gitignore'])
        self.m.git(self.live, ['commit', '-qm', 'declare derived fixture output'])
        (self.live / 'build-cache').mkdir()
        (self.live / 'build-cache/cache.bin').write_bytes(b'fixture generated bytes')
        self.config['derived_output_roots'] = ['build-cache']; self.save_config()

    def test_explicit_output_survives_actual_private_command_and_is_not_copied(self):
        self.derived()
        captured = self.capture(); self.assertEqual(captured['status'], 'completed', captured)
        selected = json.loads(Path(captured['result_path']).read_bytes())['selected']
        self.assertEqual(selected['schema'], 'c15-caller-owned-selected-source-v2')
        self.assertEqual(selected['derived_output_roots'], ['build-cache'])
        self.assertEqual(selected['selector_source_sha256'], B.HELPER_FILES['live_selection'][1])
        result = B.execute(self.request_test(captured))
        self.assertEqual((result['status'], result['closed'], result['exit_code']), ('completed', True, 1), result)
        saved = json.loads(Path(result['result_path']).read_bytes())
        self.assertFalse(Path(saved['snapshot']['repo'], 'build-cache').exists())
        self.assertEqual((self.live / 'build-cache/cache.bin').read_bytes(), b'fixture generated bytes')
        self.assertTrue(saved['source_endpoints_match'])

    def test_unlisted_ignored_output_is_still_a_retained_capture_failure(self):
        self.derived()
        (self.live / 'unknown-cache').mkdir()
        (self.live / 'unknown-cache/source').write_bytes(b'not declared')
        result = self.capture()
        self.assertEqual(result['status'], 'failed')
        self.assertIsNone(result['exit_code'])
        self.assertIn('undeclared ignored', result['stderr'])
        self.assertFalse((self.owned / 'test').exists())

    def test_bad_derived_declaration_rejected_before_attempt(self):
        for roots in [None, 'build-cache', [False], ['.'], ['.git'], ['a','a'], ['a','a/b']]:
            with self.subTest(roots=roots):
                self.config['derived_output_roots'] = roots; self.save_config()
                with self.assertRaises(ValueError): self.capture()
                self.assertFalse((self.owned / 'CAPTURE-ATTEMPT.json').exists())

    def test_v1_configuration_and_selector_cannot_masquerade_as_v2(self):
        self.config['schema'] = 'c15-runtime-bridge-config-v1'; self.save_config()
        with self.assertRaises(ValueError): self.capture()
        self.config['schema'] = 'c15-runtime-bridge-config-v2'
        self.config['helpers']['live_selection'] = ref(PRE/'c15-live-selection/live_selection.py')
        self.save_config()
        with self.assertRaises(ValueError): self.capture()
        self.assertFalse((self.owned / 'CAPTURE-ATTEMPT.json').exists())

    def test_prior_selected_declaration_and_source_pin_must_match_config(self):
        self.derived()
        captured = self.capture()
        path = Path(captured['result_path']); saved = json.loads(path.read_bytes())
        saved['selected']['derived_output_roots'] = []
        path.write_text(json.dumps(saved))
        captured['selection'] = ref(path)
        with self.assertRaises(ValueError): B.execute(self.request_test(captured))
        self.assertFalse((self.owned / 'test').exists())


if __name__ == '__main__': unittest.main()
