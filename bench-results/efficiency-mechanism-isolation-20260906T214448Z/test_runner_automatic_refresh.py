"""Provider-free checks for the fixed three-workflow automatic-refresh, no-control screen."""
import copy
import hashlib
import importlib.util
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location('candidate_test_' + name, HERE / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AutomaticRefreshRunnerTests(unittest.TestCase):
    def setUp(self):
        self.runner = load('runner_automatic_refresh')

    def test_fixed_three_variants_and_no_control(self):
        plan = self.runner.make_plan('automatic-refresh-01')
        self.runner.validate_plan(plan)
        self.assertEqual(len(plan['runs']), 3)
        self.assertEqual({row['wave'] for row in plan['runs']}, {1})
        self.assertEqual({row['condition'] for row in plan['runs']}, set(self.runner.CONDITIONS))
        self.assertNotIn('control', self.runner.CONDITIONS)
        self.assertEqual(self.runner.CONDITIONS, ('automatic-changed-refresh-full',))
        self.assertFalse(plan['randomization']['mixed_waves'])
        self.assertEqual(plan['randomization']['block_condition_counts'],
                         {'automatic-refresh-01-block-01': {'automatic-changed-refresh-full': 3}})
        self.assertEqual(self.runner.EXPERIMENT_SCHEMA, 'work-leaf-bench-experiment-v7')

    def test_rejects_expansion_replacement_control_and_fake_randomization(self):
        original = self.runner.make_plan('automatic-refresh-01')
        for mutate in (
            lambda p: p['runs'].pop(),
            lambda p: p['runs'].append(copy.deepcopy(p['runs'][0])),
            lambda p: p['runs'][0].update(condition='control'),
            lambda p: p['runs'][0].update(condition='direct'),
            lambda p: p.update(phase_kind='confirmation'),
            lambda p: p['randomization'].update(method='randomized treatment versus control'),
            lambda p: p['runs'].reverse(),
            lambda p: p.update(replacements=True),
        ):
            plan = copy.deepcopy(original)
            mutate(plan)
            with self.assertRaises(ValueError):
                self.runner.validate_plan(plan)

    def test_environment_and_old_engine_remain_identical(self):
        old = load('runner_work_units')
        manifest = {'provider_dir': '/fixture/provider', 'bin_dir': '/fixture/bin', 'study': 'fixture'}
        row = {'run_id': 'fixture', 'block_id': 'fixture-block', 'runtime_dir': '/fixture/runtime',
               'results_dir': '/fixture/results', 'experiment_manifest': '/fixture/manifest.json'}
        inherited = {'PATH': '/usr/bin', 'CODEX_HOME': '/existing/subscription',
                     'OPENAI_API_KEY': 'fake-fixture-not-a-credential', 'WORK_LEAF_OVERRIDE': 'discard'}
        expected = old.run_environment(manifest, {**row, 'condition': 'control'}, inherited)
        for condition in self.runner.CONDITIONS:
            self.assertEqual(self.runner.run_environment(manifest, {**row, 'condition': condition}, inherited), expected)
        self.assertEqual(expected['CODEX_HOME'], inherited['CODEX_HOME'])
        self.assertNotIn('OPENAI_API_KEY', expected)
        self.assertEqual(old.CONDITIONS, ('control', 'buildable-work-unit-incremental'))
        self.assertEqual(old.EXPERIMENT_SCHEMA, 'work-leaf-bench-experiment-v2')
        self.assertEqual(hashlib.sha256((HERE / 'runner_work_units.py').read_bytes()).hexdigest(), self.runner.ENGINE_SHA256)


if __name__ == '__main__':
    unittest.main()
