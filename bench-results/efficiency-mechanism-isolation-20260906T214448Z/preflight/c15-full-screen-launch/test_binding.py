"""Host-only fixtures; no provider, capture_selection, or private executor."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent


def load():
    path = HERE / 'bind_daemon.py'
    source = path.read_bytes()
    module = types.ModuleType('binding_test')
    module.__file__ = str(path)
    module.__compiled_sha256__ = hashlib.sha256(source).hexdigest()
    exec(compile(source, str(path), 'exec'), module.__dict__)
    return module


class BindingTests(unittest.TestCase):
    def setUp(self):
        self.b = load()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.runtime = self.root / 'runtime'
        self.project = self.runtime / 'allocated' / 'repo'
        self.project.mkdir(parents=True)
        self.private = self.root / 'private'
        self.private.mkdir()
        self.out = self.root / 'binding'
        self.config = self.root / 'config.json'
        self.config.write_text('{}\n')
        self.bridge = self.root / 'bridge.py'
        self.bridge.write_text('# synthetic source, never executes\n')
        self.daemon = Path('/usr/bin/true').resolve()
        self.wrapper = self.root / 'work-leaf-orchestrator'
        self.wrapper.write_text('# fake executable marker\n')
        self.template_path = self.root / 'template.json'
        self.template = dict(schema=self.b.SCHEMA, run_id='fixture-001', condition='private-test-first',
            evidence_path=str(self.root / 'events.jsonl'), runtime_root=str(self.runtime),
            private_root=str(self.private), output_root=str(self.out), base_commit='a' * 40,
            bridge=self.ref(self.bridge), python=self.ref(Path(sys.executable).resolve()),
            config=self.ref(self.config), daemon=self.ref(self.daemon),
            source_sha256={str(self.config): self.ref(self.config)['sha256']},
            required_environment={'WORK_LEAF_BENCH_EXPERIMENT': '1',
                'WORK_LEAF_BENCH_RUN_ID': 'fixture-001',
                'WORK_LEAF_BENCH_EXPERIMENT_MANIFEST': str(self.template_path),
                'WORK_LEAF_BENCH_TMPDIR': str(self.runtime)})
        self.env = {**self.template['required_environment'], 'PATH': '/synthetic:/usr/bin',
            'CODEX_HOME': '/existing/subscription', 'UNRELATED': 'keep exactly',
            'WORK_LEAF_CONTEXT_BUNDLE_DIR': str(self.project.parent / 'context-bundles'),
            'WORK_LEAF_COMMAND_TMPDIR': str(self.project.parent / 'tmp'),
            'TMPDIR': str(self.project.parent / 'tmp')}
        (self.project.parent / 'tmp').mkdir()
        self.project_calls = []
        def census(root, base, overlays):
            self.project_calls.append((root, base, overlays))
            if (root / 'WRONG-OVERLAY').exists():
                raise ValueError('declared effective overlay differs')
            return {'root': str(root), 'commit': base, 'tree': 'b' * 40,
                    'files': {}, 'administrative_sha256': {}, 'object_format': 'sha1'}
        self.fake = types.SimpleNamespace(validate_config=lambda request, qualification:
            ({'overlays': [], 'timeout_seconds': 120, 'operation_timeout_seconds': 300,
              'qualification_implementation': False},
             {'live_selection': types.SimpleNamespace(census=census)}, {},
             copy.deepcopy(self.template['source_sha256'])))
        self.patcher = patch.object(self.b, 'load_bridge', return_value=self.fake)
        self.patcher.start()
        self.addCleanup(self.patcher.stop)
        self.publish_template()

    def ref(self, path):
        return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}

    def publish_template(self):
        self.template_path.write_bytes(self.b.encoded(self.template))
        self.index = {self.template['run_id']: {'template': self.ref(self.template_path),
                                              'output_root': str(self.out)}}

    def invoke(self, exec_fn=None):
        with patch.object(self.b.Path, 'cwd', return_value=self.project):
            return self.b.main([str(self.wrapper), '--listen', '127.0.0.1:0', '--model', 'gpt-5.5'],
                self.index, self.ref(Path(sys.executable).resolve()), dict(self.env), exec_fn)

    def test_preserves_arguments_environment_and_static_template_before_exec(self):
        original = self.template_path.read_bytes()
        calls = []
        def execute(path, args, env):
            self.assertTrue((self.out / 'BINDING.json').is_file())
            self.assertTrue((self.out / 'EXPERIMENT.json').is_file())
            calls.append((path, args, env))
        self.assertEqual(self.invoke(execute), 0)
        self.assertEqual(self.template_path.read_bytes(), original)
        self.assertEqual(calls[0][0], str(self.daemon))
        self.assertEqual(calls[0][1], [str(self.wrapper), '--listen', '127.0.0.1:0', '--model', 'gpt-5.5'])
        expected = dict(self.env, WORK_LEAF_BENCH_EXPERIMENT_MANIFEST=str(self.out / 'EXPERIMENT.json'))
        self.assertEqual(calls[0][2], expected)
        receipt = json.loads((self.out / 'BINDING.json').read_text())
        self.assertEqual(receipt['environment_sha256'], self.b.value_sha(self.env))
        self.assertEqual(receipt['forwarded_environment_sha256'], self.b.value_sha(expected))
        self.assertNotIn('/existing/subscription', (self.out / 'BINDING.json').read_text())
        self.assertEqual(receipt['environment_changed_keys'], ['WORK_LEAF_BENCH_EXPERIMENT_MANIFEST'])
        self.assertEqual(len(self.project_calls), 2)
        manifest = json.loads((self.out / 'EXPERIMENT.json').read_text())
        self.assertEqual(manifest['private_preview']['project_root'], str(self.project))
        self.assertEqual(manifest['private_preview']['root'], str(self.private))
        self.assertEqual(manifest['schema'], 'work-leaf-bench-experiment-v6')

    def test_wrong_cwd_retains_failure_and_never_executes(self):
        self.project = self.root
        self.assertEqual(self.invoke(lambda *_: self.fail('must not execute')), 2)
        self.assertTrue((self.out / 'ATTEMPT.json').exists())
        self.assertTrue((self.out / 'ERROR.json').exists())

    def test_overlay_mismatch_and_nonempty_private_root_fail_closed(self):
        (self.project / 'WRONG-OVERLAY').touch()
        self.assertEqual(self.invoke(lambda *_: self.fail('must not execute')), 2)
        self.assertIn('overlay', (self.out / 'ERROR.json').read_text())

    def test_private_root_must_be_empty_and_outside_all_live_roots(self):
        (self.private / 'prior').touch()
        self.assertEqual(self.invoke(lambda *_: self.fail('must not execute')), 2)

    def test_replay_does_not_change_existing_receipts(self):
        self.assertEqual(self.invoke(lambda *_: None), 0)
        saved = {x.name: x.read_bytes() for x in self.out.iterdir()}
        self.assertEqual(self.invoke(lambda *_: self.fail('replay')), 2)
        self.assertEqual(saved, {x.name: x.read_bytes() for x in self.out.iterdir()})

    def test_modified_template_rejected_against_bootstrap_pin(self):
        self.template_path.write_text('{}\n')
        self.assertEqual(self.invoke(lambda *_: self.fail('mutable template')), 2)
        self.assertFalse((self.out / 'EXPERIMENT.json').exists())

    def test_mutated_dependency_at_endpoint_fails_before_exec(self):
        original = self.fake.validate_config
        def mutate(*args):
            result = original(*args)
            self.config.write_text('{"changed":true}\n')
            return result
        self.fake.validate_config = mutate
        self.assertEqual(self.invoke(lambda *_: self.fail('mutable source')), 2)

    def test_symlinked_source_or_output_is_rejected(self):
        real = self.root / 'real-config'
        self.config.rename(real)
        self.config.symlink_to(real)
        self.assertEqual(self.invoke(lambda *_: self.fail('source alias')), 2)

    def test_symlink_output_rejected_without_writing_target(self):
        target = self.root / 'elsewhere'
        target.mkdir()
        self.out.symlink_to(target, target_is_directory=True)
        self.assertEqual(self.invoke(lambda *_: self.fail('output alias')), 2)
        self.assertEqual(list(target.iterdir()), [])

    def test_changed_required_environment_is_not_silently_corrected(self):
        self.env['WORK_LEAF_BENCH_EXPERIMENT'] = '0'
        self.assertEqual(self.invoke(lambda *_: self.fail('bad env')), 2)

    def test_exec_failure_retains_ready_receipt_and_separate_error(self):
        def execute(*_): raise OSError('synthetic exec failure')
        self.assertEqual(self.invoke(execute), 2)
        self.assertTrue((self.out / 'BINDING.json').exists())
        self.assertTrue((self.out / 'ERROR.json').exists())

    def real_census_fixture(self):
        source = HERE.parent / 'c15-live-selection/live_selection.py'
        module = types.ModuleType('real_read_only_census')
        module.__file__ = str(source)
        exec(compile(source.read_bytes(), str(source), 'exec'), module.__dict__)
        def git(*args):
            return subprocess.run(['/usr/bin/git', '-C', str(self.project), *args], check=True,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, env={**os.environ,
                'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}).stdout.decode().strip()
        git('init', '-q'); git('config', 'user.name', 'Synthetic fixture'); git('config', 'user.email', 'fixture@example.invalid')
        (self.project / 'instructions').write_text('ordinary\n')
        git('add', 'instructions'); git('commit', '-qm', 'initial')
        base, blob = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD:instructions')
        (self.project / 'instructions').write_text('ordinary\noverlay\n')
        git('update-index', '--skip-worktree', '--', 'instructions')
        overlay = {'path': 'instructions', 'base_blob': blob, 'base_sha256': self.b.sha(b'ordinary\n'),
            'base_mode': '100644', 'effective_mode': '100644', 'effective_text': 'ordinary\noverlay\n',
            'effective_sha256': self.b.sha(b'ordinary\noverlay\n'),
            'index_flags': {'skip_worktree': True, 'assume_unchanged': False}}
        self.template['base_commit'] = base
        self.fake.validate_config = lambda *_: ({'overlays': [overlay], 'timeout_seconds': 120,
            'operation_timeout_seconds': 300, 'qualification_implementation': False},
            {'live_selection': module}, {}, copy.deepcopy(self.template['source_sha256']))
        self.publish_template()
        return git

    def test_actual_read_only_git_overlay_census_matches_and_preserves_shared_state(self):
        git = self.real_census_fixture()
        before = (git('rev-parse', 'HEAD'), (self.project / '.git/index').read_bytes(),
                  (self.project / 'instructions').read_bytes())
        self.assertEqual(self.invoke(lambda *_: None), 0)
        self.assertEqual(before, (git('rev-parse', 'HEAD'), (self.project / '.git/index').read_bytes(),
                                  (self.project / 'instructions').read_bytes()))
        receipt = json.loads((self.out / 'BINDING.json').read_text())
        self.assertEqual(receipt['project_census']['files']['instructions']['index_flags'],
                         {'skip_worktree': True, 'assume_unchanged': False})

    def test_actual_overlay_or_base_mismatch_never_executes(self):
        self.real_census_fixture()
        (self.project / 'instructions').write_text('wrong\n')
        self.assertEqual(self.invoke(lambda *_: self.fail('invalid actual overlay')), 2)

    def test_actual_noninitial_head_never_executes(self):
        self.real_census_fixture()
        self.template['base_commit'] = '0' * 40
        self.publish_template()
        self.assertEqual(self.invoke(lambda *_: self.fail('noninitial head')), 2)

    def test_generated_bootstrap_executes_true_after_real_read_only_validation(self):
        # Real exact-byte bootstrap/config/census; /usr/bin/true is the ONLY
        # daemon substitute. No bridge.execute/capture_selection/executor call.
        self.real_census_fixture()
        overlays = self.fake.validate_config(None, False)[0]['overlays']
        self.patcher.stop()
        self.bridge = HERE.parent / 'c15-runtime-bridge/bridge.py'
        bridge_ref = self.ref(self.bridge)
        qualified = self.b.load_bridge(bridge_ref)
        toolchain = Path('/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu')
        config = dict(schema='c15-runtime-bridge-config-v1', helpers=qualified.DEFAULT_HELPERS,
            executables={'git': self.ref(Path('/usr/bin/git')), 'bwrap': self.ref(Path('/usr/bin/bwrap')),
                'patch_driver': self.ref(HERE.parent / 'c15-private-materialization/target/debug/c15-patch-driver'),
                'toolchain_cargo': self.ref(toolchain / 'bin/cargo'),
                'toolchain_rustc': self.ref(toolchain / 'bin/rustc')},
            toolchain_root=str(toolchain), cargo_capsule=self.ref(HERE.parent /
                'c15-project-qualification/PROJECT-001/public-cargo/CAPSULE.json'), overlays=overlays,
            timeout_seconds=120, operation_timeout_seconds=300, max_output_bytes=1048576,
            qualification_implementation=False)
        self.config.write_bytes(self.b.encoded(config))
        validation = self.root / 'validation'; validation.mkdir()
        _, _, _, pins = self.b.inspect_config(self.ref(self.config), bridge_ref, self.project, validation)
        self.template.update(config=self.ref(self.config), bridge=bridge_ref, source_sha256=pins)
        self.publish_template()
        source = HERE.parent.parent / 'runner_test_first.py'
        runner = types.ModuleType('bootstrap_fixture'); runner.__file__ = str(source)
        exec(compile(source.read_bytes(), str(source), 'exec'), runner.__dict__)
        self.wrapper.write_bytes(runner.bootstrap(self.index, self.template['python'], self.ref(HERE / 'bind_daemon.py')))
        self.wrapper.chmod(0o700)
        result = subprocess.run([str(self.wrapper), '--listen', '127.0.0.1:0', '--model', 'gpt-5.5'],
            cwd=self.project, env={**self.env, 'PYTHONPATH': '/intentionally-invalid-fixture'},
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(result.stdout, b'')
        receipt = json.loads((self.out / 'BINDING.json').read_text())
        self.assertEqual(receipt['actual_daemon'], self.ref(Path('/usr/bin/true').resolve()))
        self.assertEqual(list(self.private.iterdir()), [])
        self.assertEqual(list(validation.iterdir()), [])


if __name__ == '__main__':
    unittest.main()
