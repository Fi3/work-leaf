from pathlib import Path
import unittest

import batch_adapter as b


class BatchAdapterTests(unittest.TestCase):
    def test_exact_three_modified_identities_and_parallel_contract(self):
        runner = b.load_runner()
        rows = runner.schedule(Path('/tmp/batch'), Path('/tmp/source'), Path('/tmp/runtime'))
        self.assertEqual([r['run_id'] for r in rows], b.IDS)
        runner.validate_schedule(rows)
        with self.assertRaises(ValueError):
            runner.validate_schedule(rows[:2])
        self.assertIn('"maximum_concurrent_workflows": 3', runner.E.ENGINE_SOURCE)
        self.assertIn('"workflow_count": 3,', runner.E.ENGINE_SOURCE)
        self.assertIn('5400', runner.E.ENGINE_SOURCE)

    def test_environment_pins_only_benchmark_routes_and_subscription(self):
        runner = b.load_runner()
        manifest = {'study': b.PHASE, 'provider_dir':'/provider', 'bin_dir':'/bin', 'source_repo':'/source'}
        row = runner.schedule(Path('/tmp/batch'), None, Path('/tmp/runtime'))[0]
        env = runner.run_environment(manifest, row, {'PATH':'/bin','OPENAI_API_KEY':'must-not-pass', 'WORK_LEAF_OTHER':'must-not-pass'})
        self.assertNotIn('OPENAI_API_KEY', env)
        self.assertNotIn('WORK_LEAF_OTHER', env)
        self.assertEqual(env['WORK_LEAF_BENCH_FULL_INVERSE'], '1')
        self.assertEqual(env['WORK_LEAF_BENCH_FULL_HOST'], str(b.HERE/'host_custody.py'))
        self.assertTrue(env['WORK_LEAF_BENCH_INPUT_RECEIPTS'].endswith(row['run_id']+'/input-receipts'))
        self.assertEqual(env['WORK_LEAF_DIRECT_BENCH_MODEL'], 'gpt-5.5')

    def test_loaded_source_pins_remain_exact(self):
        b.load_runner()
        self.assertEqual(b.digest(b.ORIGINAL), b.ORIGINAL_SHA)


if __name__ == '__main__':
    unittest.main()
