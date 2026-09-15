from pathlib import Path
import unittest

import batch_continuation


class IntegrationBatchTests(unittest.TestCase):
    def test_exact_three_continuations_and_no_unadmitted_execution(self):
        runner = batch_continuation.load_runner()
        rows = runner.schedule(Path('/tmp/phase'), None, Path('/tmp/runtime'))
        self.assertEqual([r['run_id'] for r in rows],
                         ['author-joint-integration-004', 'author-joint-integration-005',
                          'author-joint-integration-006'])
        self.assertEqual(runner.PHASE, 'author-joint-integration-20260915')
        self.assertIn('"supervisor_wall_timeout_seconds": 1800', runner.E.ENGINE_SOURCE)
        self.assertIn('manifest.get("supervisor_wall_timeout_seconds") != 1800',
                      runner.E.ENGINE_SOURCE)
        runner.validate_schedule(rows)
        with self.assertRaises(ValueError):
            runner.validate_schedule(rows[:2])
        with self.assertRaises(ValueError):
            runner.run_batch(Path('/tmp/unadmitted'))

    def test_resource_monitor_has_lower_declared_threshold(self):
        monitor, source = batch_continuation.load_monitor()
        self.assertIn('>= 12_000_000', source)
        self.assertNotIn('>= 45_000_000', source)
        self.assertEqual(monitor.next_sample, 0)

    def test_effective_environment_uses_the_qualified_immutable_provider(self):
        runner = batch_continuation.load_runner()
        rows = runner.schedule(Path('/tmp/phase'), None, Path('/tmp/runtime'))
        manifest = dict(study='fixture', source_repo='/tmp/source',
                        provider_dir='/tmp/provider', bin_dir='/tmp/bin')
        env = runner.run_environment(manifest, rows[0], {'PATH':'/usr/bin'})
        self.assertEqual(env.get('WORK_LEAF_BENCH_CATALOG_IMMUTABLE'), '1')
        self.assertEqual(env['WORK_LEAF_BENCH_FULL_PROVIDER'], str(
            Path(batch_continuation.__file__).resolve().parent.parent / 'catalog-immutability-016/input_provider'))


if __name__ == '__main__':
    unittest.main()
