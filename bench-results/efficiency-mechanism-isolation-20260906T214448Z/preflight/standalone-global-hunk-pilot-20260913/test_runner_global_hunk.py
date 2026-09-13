"""Fail-first fresh global-hunk identity qualification; no provider is executed."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import types
import unittest
from unittest.mock import Mock, patch

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / 'standalone-bounded-package-20260912/runner_bounded.py'
BASE_SHA = '8811139a870bdd12586560461e3526e6434cc622af7736a40250d5a04716f91b'
WRAPPER = HERE.parent.parent / 'screens/native-production-cohesion-20260912/provider/codex'
WRAPPER_SHA = '8ad1d261979029fec24cf4e143aaf6afc6c688056c6bc702c824b3538b92ff32'
SOURCE = Path(os.environ.get('GLOBAL_HUNK_RUNNER_SOURCE', str(HERE / 'runner_global_hunk.py'))).resolve()
ORIGINAL = HERE.parent / 'standalone-completion-pilot-20260912/runner_completion.py'
ORIGINAL_SHA = '9fa3462016f4be95de1d22479bb85b01f1d9fdb7c7a396e312d4f44a024157c0'
assert SOURCE in (ORIGINAL.resolve(), (HERE / 'runner_global_hunk.py').resolve())
assert hashlib.sha256(ORIGINAL.read_bytes()).hexdigest() == ORIGINAL_SHA
assert hashlib.sha256(BASE.read_bytes()).hexdigest() == BASE_SHA

# Exact reviewed adapter deltas, inverted against the retained V4 adapter bytes.
ADAPTER_SUBSTITUTIONS = [
  [
    "\"\"\"One fixed wave of three standalone serialized host-custody workflows; no replacements.\"\"\"",
    "\"\"\"One disarmed full-workflow completion pilot; no replacements or retries.\"\"\""
  ],
  [
    "IDS = [f'standalone-bounded-{n:03d}' for n in range(1, 4)]",
    "IDS = ['standalone-global-hunk-pilot-001']"
  ],
  [
    "CONDITION = 'standalone-bounded'",
    "CONDITION = 'standalone-global-hunk-pilot'\nPHASE = 'standalone-global-hunk-pilot-01'"
  ],
  [
    "'\"supervisor_wall_timeout_seconds\": 3600'",
    "'\"supervisor_wall_timeout_seconds\": 5400'"
  ],
  [
    "'manifest.get(\"supervisor_wall_timeout_seconds\") != 3600'",
    "'manifest.get(\"supervisor_wall_timeout_seconds\") != 5400'"
  ],
  [
    "'\"maximum_concurrent_workflows\": 3'",
    "'\"maximum_concurrent_workflows\": 1'"
  ],
  [
    "'manifest.get(\"maximum_concurrent_workflows\") != 3'",
    "'manifest.get(\"maximum_concurrent_workflows\") != 1'"
  ],
  [
    "'\"workflow_count\": 3,'",
    "'\"workflow_count\": 1,'"
  ],
  [
    "    substitutions = [\n",
    "    substitutions = [\n        ('        freeze(source / name, infrastructure / \"drivers\" / name, \"frozen-driver\")',\n         '        if name == \"bench-candidate-common\":\\n'\n         '            freeze(source / name, infrastructure / \"original-drivers\" / name, \"original-artifact-common\")\\n'\n         '            record(args.artifact_common_source, \"artifact-common-source\")\\n'\n         '            freeze(args.artifact_common_source, infrastructure / \"drivers\" / name, \"private-artifact-common\")\\n'\n         '        else:\\n'\n         '            freeze(source / name, infrastructure / \"drivers\" / name, \"frozen-driver\")', 1),\n"
  ],
  [
    "         '                    host_source=str(args.host_source.resolve()),\\n'\n",
    "         '                    host_source=str(args.host_source.resolve()),\\n'\n         '                    artifact_common_source=str(args.artifact_common_source.resolve()),\\n'\n"
  ],
  [
    "                           SELF=SELF, ENGINE_PATH=ENGINE_PATH, configuration=configuration)",
    "                           SELF=SELF, ENGINE_PATH=ENGINE_PATH, configuration=configuration,\n                           ENGINE_SUBSTITUTIONS=substitutions, ENGINE_SOURCE=source)"
  ],
  [
    "not isinstance(rows, list) or len(rows) != 3",
    "not isinstance(rows, list) or len(rows) != 1"
  ],
  [
    "exactly the three declared standalone serialized host-custody rows are required",
    "exactly the one declared standalone completion-pilot row is required"
  ],
  [
    "    args = copy.copy(args)\n",
    "    args = copy.copy(args)\n    if args.batch_root.resolve().name != PHASE:\n        raise ValueError('fresh completion-pilot phase identity is required')\n    if args.artifact_common_source.is_symlink() or not args.artifact_common_source.is_file():\n        raise ValueError('artifact common source must be an ordinary regular file')\n    args.artifact_common_source = args.artifact_common_source.resolve(strict=True)\n"
  ],
  [
    "    manifest = _verify(Path(batch).resolve())\n",
    "    manifest = _verify(Path(batch).resolve())\n    if Path(batch).resolve().name != PHASE or manifest.get('study') != PHASE:\n        raise ValueError('fresh completion-pilot phase identity is required')\n"
  ],
  [
    "                manifest['host_source'], str(Path(batch).resolve()/'infrastructure/drivers/host_custody.py')}",
    "                manifest['host_source'], str(Path(batch).resolve()/'infrastructure/drivers/host_custody.py'),\n                manifest['artifact_common_source'],\n                str(Path(batch).resolve()/'infrastructure/drivers/bench-candidate-common'),\n                str(Path(batch).resolve()/'infrastructure/original-drivers/bench-candidate-common')}"
  ],
  [
    "    # The adapter evidence is copied by the inherited preparation routine.\n",
    "    private_common = str(Path(batch).resolve()/'infrastructure/drivers/bench-candidate-common')\n    original_common = str(Path(batch).resolve()/'infrastructure/original-drivers/bench-candidate-common')\n    source_common = str(Path(manifest['source_repo'])/'bench-candidate-common')\n    if (index[private_common]['sha256'] != index[manifest['artifact_common_source']]['sha256']\n            or index[original_common]['sha256'] != index[source_common]['sha256']):\n        raise ValueError('private or original artifact common copy differs from its source')\n    # The adapter evidence is copied by the inherited preparation routine.\n"
  ],
  [
    "'runtime-root','generated-driver','global-config','host-source'):",
    "'runtime-root','generated-driver','global-config','host-source','artifact-common-source'):"
  ],
  [
    "standalone serialized host-custody runner:",
    "standalone completion-pilot runner:"
  ],
  [
    "    if any(str(Path(p).resolve()) == config['path'] for p in args.evidence + args.identity_file):",
    "    if any(str(Path(p).resolve()) == config['path']\n           for p in args.evidence + args.identity_file + [args.artifact_common_source]):"
  ]
]


def module_at(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


m = module_at('completion_runner_under_test', SOURCE)
legacy_path = HERE.parent / 'direct-author-contract-20260912/test_runner_direct.py'
assert hashlib.sha256(legacy_path.read_bytes()).hexdigest() == '5373382c652031611aa29f0076ff7f4a2c48d4dfe2a3b6ae71e7dd4c53a59486'
legacy = module_at('pinned_completion_fixture', legacy_path)


class CompletionTests(unittest.TestCase):
    m = m
    git = legacy.RunnerTests.git

    def fixture(self, armed=False):
        original = self.m.prepare
        def private_inputs(args):
            args.batch_root = args.batch_root.parent / 'standalone-global-hunk-pilot-01'
            args.subscription_wrapper = WRAPPER
            args.host_source = args.source_repo.parent / 'host_custody.py'
            args.host_source.write_text('# synthetic host, never executed\n')
            args.artifact_common_source = args.source_repo.parent / 'private-common'
            args.artifact_common_source.write_text('# synthetic retention helper, never executed\n')
            return original(args)
        with patch.object(self.m, 'prepare', side_effect=private_inputs):
            return legacy.RunnerTests.fixture(self, armed)

    def test_single_disarmed_full_workflow_with_exact_budgets(self):
        args, manifest = self.fixture()
        self.assertEqual(self.m.IDS, ['standalone-global-hunk-pilot-001'])
        self.assertEqual(len(manifest['schedule']), 1)
        row = manifest['schedule'][0]
        self.assertEqual(row['condition'], 'standalone-global-hunk-pilot')
        self.assertEqual(row['argv'], [row['driver'], 'sequential'])
        self.assertEqual(manifest['maximum_concurrent_workflows'], 1)
        self.assertEqual(manifest['supervisor_wall_timeout_seconds'], 5400)
        self.assertFalse(manifest['execution_authorized'])
        self.assertFalse(manifest['replacements_allowed'])
        self.assertTrue(manifest['pause_after_wave'])
        self.assertEqual(manifest['study'], 'standalone-global-hunk-pilot-01')
        self.assertEqual(hashlib.sha256(WRAPPER.read_bytes()).hexdigest(), WRAPPER_SHA)
        for function in (self.m._prepare, self.m._verify):
            self.assertIn(5400, function.__code__.co_consts)
            self.assertNotIn(3600, function.__code__.co_consts)
            self.assertNotIn(86400, function.__code__.co_consts)
        self.assertIn(WRAPPER_SHA, self.m._prepare.__code__.co_consts)
        with self.git():
            self.m.verify_manifest(args.batch_root)

    def test_common_source_original_and_private_copy_all_frozen(self):
        args, manifest = self.fixture()
        private = args.batch_root / 'infrastructure/drivers/bench-candidate-common'
        original = args.batch_root / 'infrastructure/original-drivers/bench-candidate-common'
        self.assertEqual(manifest['artifact_common_source'], str(args.artifact_common_source))
        self.assertEqual(private.read_bytes(), args.artifact_common_source.read_bytes())
        self.assertEqual(original.read_bytes(), (args.source_repo / 'bench-candidate-common').read_bytes())
        self.assertNotEqual(private.read_bytes(), original.read_bytes())
        index = {row['path']: row for row in manifest['files']}
        for path in (private, original, args.artifact_common_source, args.source_repo / 'bench-candidate-common',
                     args.host_source, args.generated_driver, self.m.ENGINE_PATH, self.m.SELF):
            self.assertEqual(index[str(path)]['sha256'], self.m.digest(path))
        for name in self.m.E.DRIVERS:
            if name != 'bench-candidate-common':
                self.assertEqual((args.source_repo / name).read_bytes(),
                                 (args.batch_root / 'infrastructure/drivers' / name).read_bytes())

    def test_private_common_drift_or_missing_identity_rejected(self):
        for endpoint in ('source', 'copy', 'original', 'record'):
            args, manifest = self.fixture(armed=True)
            if endpoint == 'record':
                manifest['files'] = [row for row in manifest['files'] if row['path'] != manifest['artifact_common_source']]
                (args.batch_root / 'PHASE-MANIFEST.json').write_text(json.dumps(manifest))
            else:
                target = {'source': args.artifact_common_source,
                          'copy': args.batch_root / 'infrastructure/drivers/bench-candidate-common',
                          'original': args.batch_root / 'infrastructure/original-drivers/bench-candidate-common'}[endpoint]
                target.chmod(0o600)
                target.write_text('drift\n')
            launch = Mock()
            with self.git(), self.assertRaises(ValueError):
                self.m.run_batch(args.batch_root, popen=launch)
            launch.assert_not_called()

    def test_no_other_schedule_or_phase_or_wall_is_admitted(self):
        args, manifest = self.fixture()
        row = manifest['schedule'][0]
        for rows in ([], [row, row], [dict(row, run_id='standalone-bounded-001')],
                     [dict(row, condition='direct')]):
            with self.assertRaises(ValueError):
                self.m.validate_schedule(rows)
        for field, value in (('supervisor_wall_timeout_seconds', 3600),
                             ('supervisor_wall_timeout_seconds', 5401),
                             ('maximum_concurrent_workflows', 3), ('study', 'replacement-01')):
            changed = dict(manifest, **{field: value})
            (args.batch_root / 'PHASE-MANIFEST.json').write_text(json.dumps(changed))
            with self.git(), self.assertRaises(ValueError):
                self.m.verify_manifest(args.batch_root)
        args.batch_root = args.batch_root.parent / 'wrong-phase'
        with self.git(), self.assertRaises(ValueError):
            self.m.prepare(args)
        self.assertFalse(args.batch_root.exists())

    def test_missing_or_symlinked_private_common_rejected(self):
        args, _ = self.fixture()
        args.batch_root = args.batch_root.parent / 'fresh' / 'standalone-global-hunk-pilot-01'
        args.artifact_common_source = args.source_repo.parent / 'missing'
        with self.git(), self.assertRaises(ValueError):
            self.m.prepare(args)
        args.artifact_common_source.symlink_to(args.source_repo / 'bench-candidate-common')
        with self.git(), self.assertRaises(ValueError):
            self.m.prepare(args)
        self.assertFalse(args.batch_root.exists())

    def test_private_common_cannot_copy_global_configuration(self):
        args, _ = self.fixture()
        args.batch_root = args.batch_root.parent / 'fresh' / 'standalone-global-hunk-pilot-01'
        args.artifact_common_source = args.global_config
        with self.git(), self.assertRaisesRegex(ValueError, 'global configuration'):
            self.m.prepare(args)
        self.assertFalse(args.batch_root.exists())

    def test_common_copy_hash_cannot_be_rebased_away_from_source(self):
        args, manifest = self.fixture(armed=True)
        private = args.batch_root / 'infrastructure/drivers/bench-candidate-common'
        private.chmod(0o600)
        private.write_text('different copy with individually updated manifest digest\n')
        for row in manifest['files']:
            if row['path'] == str(private):
                row['sha256'] = self.m.digest(private)
        (args.batch_root / 'PHASE-MANIFEST.json').write_text(json.dumps(manifest))
        launch = Mock()
        with self.git(), self.assertRaisesRegex(ValueError, 'copy differs from its source'):
            self.m.run_batch(args.batch_root, popen=launch)
        launch.assert_not_called()

    def test_one_launch_one_admission_and_no_retry_after_failure(self):
        for failure in (False, True):
            args, _ = self.fixture(armed=True)
            launch = Mock(side_effect=OSError('synthetic spawn failure')) if failure else Mock(
                return_value=Mock(pid=999999, poll=Mock(return_value=101), wait=Mock(return_value=101)))
            with self.git():
                result = self.m.run_batch(args.batch_root, popen=launch)
            self.assertEqual(launch.call_count, 1)
            self.assertEqual(len(result['runs']), 1)
            self.assertEqual(json.loads((args.batch_root / 'RUN-ONCE/admission.json').read_text())['workflow_count'], 1)
            for name in ('PHASE-RESULT.json', 'score-manifest.json', 'POST-RUN-IDENTITY.json'):
                self.assertTrue((args.batch_root / name).is_file())
            with self.git(), self.assertRaises((ValueError, FileExistsError)):
                self.m.run_batch(args.batch_root, popen=launch)
            self.assertEqual(launch.call_count, 1)

    def test_real_nonprovider_child_wall_stop_reaps_after_unchanged_grace(self):
        args, _ = self.fixture(armed=True)
        children = []
        def launch(_argv, **kw):
            child = subprocess.Popen(['/usr/bin/python3', '-c',
                'import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); print("ready",flush=True); time.sleep(60)'],
                stdin=kw['stdin'], stdout=kw['stdout'], stderr=kw['stderr'], start_new_session=True)
            children.append(child)
            for _ in range(200):
                if 'ready' in Path(kw['stdout'].name).read_text():
                    return child
                time.sleep(.005)
            self.fail('synthetic child did not install handler')
        ticks = iter([0, 5401, 5401, 5422])
        try:
            with self.git(), patch.object(self.m.E, 'time', types.SimpleNamespace(
                    monotonic=lambda: next(ticks, 5422), sleep=time.sleep)):
                result = self.m.run_batch(args.batch_root, popen=launch)
            self.assertEqual(len(children), 1)
            self.assertTrue(result['supervisor_wall_timeout'])
            self.assertEqual(children[0].returncode, -signal.SIGKILL)
        finally:
            for child in children:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait(timeout=5)

    def test_engine_supervisor_unchanged_except_single_admission_count(self):
        old = module_at('pinned_bounded_reference', BASE)
        def supervisor(source):
            return source.split('def run_batch(', 1)[1].split('\n\ndef main():', 1)[0]
        expected = supervisor(self.m.ENGINE_PATH.read_text())
        for before, after, count in (
                ('FIRST-BATCH-MANIFEST.json', 'PHASE-MANIFEST.json', 2),
                ('FIRST-BATCH-RESULT.json', 'PHASE-RESULT.json', 1),
                ('"workflow_count": 2,', '"workflow_count": 1,', 1),
                ('popen([row["driver"]],', 'popen(row["argv"],', 1)):
            self.assertEqual(expected.count(before), count)
            expected = expected.replace(before, after)
        self.assertEqual(supervisor(self.m.E.ENGINE_SOURCE), expected)
        self.assertEqual(self.m.digest(self.m.ENGINE_PATH), self.m.ENGINE_SHA)
        self.assertIn('time.monotonic() - stopping_at > 20', expected)
        self.assertEqual(self.m._environment.__code__.co_code, old._environment.__code__.co_code)
        self.assertEqual(self.m._environment.__code__.co_consts, old._environment.__code__.co_consts)

    def test_every_engine_override_has_exact_inverse(self):
        restored = self.m.E.ENGINE_SOURCE
        for before, after, count in reversed(self.m.E.ENGINE_SUBSTITUTIONS):
            self.assertEqual(restored.count(after), count, after)
            restored = restored.replace(after, before)
        self.assertEqual(restored.encode(), self.m.ENGINE_PATH.read_bytes())

    def test_every_adapter_change_has_exact_inverse(self):
        restored = SOURCE.read_text()
        for before, after in reversed(ADAPTER_SUBSTITUTIONS):
            self.assertEqual(restored.count(after), 1, after)
            restored = restored.replace(after, before, 1)
        self.assertEqual(restored.encode(), BASE.read_bytes())

    test_auth_strip_homes_preserved_and_no_wl_factor = legacy.RunnerTests.test_auth_strip_homes_preserved_and_no_wl_factor
    test_disarmed_rejected_before_marker_or_call = legacy.RunnerTests.test_disarmed_rejected_before_marker_or_call
    test_source_config_and_adapter_drift_rejected_before_launch = legacy.RunnerTests.test_source_config_and_adapter_drift_rejected_before_launch
    test_existing_output_or_attempt_rejected = legacy.RunnerTests.test_existing_output_or_attempt_rejected
    test_loaded_engine_drift_and_dirty_source_are_rejected = legacy.RunnerTests.test_loaded_engine_drift_and_dirty_source_are_rejected
    test_live_engine_is_in_postrun_source_population = legacy.RunnerTests.test_live_engine_is_in_postrun_source_population
    test_postrun_manifest_change_is_retained = legacy.RunnerTests.test_postrun_manifest_change_is_retained



class GlobalIdentityTests(unittest.TestCase):
    def test_fresh_global_identity_and_single_schedule(self):
        self.assertEqual(m.IDS, ['standalone-global-hunk-pilot-001'])
        self.assertEqual(m.CONDITION, 'standalone-global-hunk-pilot')
        self.assertEqual(m.PHASE, 'standalone-global-hunk-pilot-01')
        rows = m.schedule(Path('/synthetic/standalone-global-hunk-pilot-01'),
                          Path('/unused'), Path('/synthetic/runtime'))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['run_id'], 'standalone-global-hunk-pilot-001')
        self.assertEqual(rows[0]['argv'], [rows[0]['driver'], 'sequential'])
        self.assertEqual(rows[0]['workflow'], 'direct-sequential-serialized-host-custody')

    def test_previous_pilot_schedule_cannot_be_reused(self):
        rows = m.schedule(Path('/synthetic/standalone-global-hunk-pilot-01'),
                          Path('/unused'), Path('/synthetic/runtime'))
        rows[0]['run_id'] = 'standalone-completion-pilot-001'
        rows[0]['condition'] = 'standalone-completion-pilot'
        with self.assertRaises(ValueError):
            m.validate_schedule(rows)

    def test_identity_only_inverse_restores_entire_original_runner(self):
        restored = SOURCE.read_bytes()
        replacements = [
            (b"IDS = ['standalone-completion-pilot-001']", b"IDS = ['standalone-global-hunk-pilot-001']"),
            (b"CONDITION = 'standalone-completion-pilot'", b"CONDITION = 'standalone-global-hunk-pilot'"),
            (b"PHASE = 'standalone-completion-pilot-01'", b"PHASE = 'standalone-global-hunk-pilot-01'"),
        ]
        for before, after in replacements:
            self.assertEqual(restored.count(after), 1)
            restored = restored.replace(after, before)
        self.assertEqual(restored, ORIGINAL.read_bytes())
        self.assertEqual(hashlib.sha256(restored).hexdigest(), ORIGINAL_SHA)


if __name__ == '__main__':
    unittest.main()
