import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

import continuation


class ContinuationTests(unittest.TestCase):
    def test_exact_three_seam_inverse_preserves_retained_driver(self):
        original = continuation.repair.driver_source()
        changed = continuation.driver_source()
        for before, after in reversed(continuation.replacements(original)):
            self.assertEqual(changed.count(after), 1)
            changed = changed.replace(after, before)
        self.assertEqual(changed, original)

    def test_runtime_cycle_has_no_author_or_reviewer_execution(self):
        source = continuation.driver_source()
        body = source.split('run_sequential_bench() {\n', 1)[1].split('\n}\n', 1)[0]
        self.assertNotIn('run_feature_cycle', body)
        self.assertNotIn('run_host_author', body)
        self.assertIn('run_direct_agent "linearize-plan"', body)
        self.assertIn('run_direct_agent_resume "linearize-accept"', body)
        subprocess.run(['bash', '-n'], input=source.encode(), check=True)

    def test_trust_wrap_preserves_every_original_argument(self):
        repo = '/tmp/check.out/repo with space'
        args = ['--cd', repo, '--sandbox', 'danger-full-access',
                '--ask-for-approval', 'never', '--model', 'gpt-5.5',
                'exec', '--color', 'never', 'resume', '--json',
                '01a0a311-f047-7a41-966d-426485f13972', '-']
        actual = continuation.provider_args(args, 'linearize-accept')
        self.assertEqual(actual[2:], args)
        self.assertEqual(actual[:2], ['-c', 'projects={'+json.dumps(repo)+'={trust_level="trusted"}}'])

    def test_foreign_or_ambiguous_provider_arguments_fail(self):
        args = ['--cd', '/tmp/exact/repo', 'exec', '--json', '-']
        for changed, stage in ((args, 'sequential-feature-1-implement'),
                               (args + ['--cd', '/tmp/other/repo'], 'linearize-plan'),
                               (['-c', 'arbitrary=true'] + args, 'linearize-plan'),
                               (['--cd', '/tmp', 'exec'], 'linearize-plan')):
            with self.subTest(stage=stage, args=changed), self.assertRaises(ValueError):
                continuation.provider_args(changed, stage)

    def test_seed_is_exact_pinned_and_not_reassigned(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'seed-input'
            path.write_bytes(b'retained evidence')
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            seed = dict(run_id='continuation-a', original_run_id='original-a',
                        base='a'*40, head='b'*40, tree='c'*40,
                        pins={str(path):digest})
            manifest = dict(schema=1, rows=[seed])
            self.assertEqual(continuation.select_seed(manifest, 'continuation-a'), seed)
            with self.assertRaises(ValueError):
                continuation.select_seed(manifest, 'foreign')
            with self.assertRaises(ValueError):
                continuation.select_seed(dict(schema=1, rows=[seed, seed]), 'continuation-a')
            path.write_bytes(b'stale evidence')
            with self.assertRaises(ValueError):
                continuation.select_seed(manifest, 'continuation-a')

    def test_restore_rejects_foreign_head_before_import(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary) / 'repo'
            subprocess.run(['git', 'init', '-q', str(repo)], check=True)
            subprocess.run(['git', '-C', str(repo), '-c', 'user.name=Fixture',
                            '-c', 'user.email=fixture@example.com', 'commit',
                            '--allow-empty', '-qm', 'fixture'], check=True)
            head = subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'])
            with self.assertRaises(ValueError):
                continuation.restore(repo, dict(base='a'*40))
            self.assertEqual(head, subprocess.check_output(
                ['git', '-C', str(repo), 'rev-parse', 'HEAD']))

    def test_cache_preparation_only_compiles_and_has_no_agent_or_test_execution(self):
        self.assertEqual(continuation.warm_commands(), [
            ['cargo', 'clippy', '--all-targets', '--all-features', '--', '-D', 'warnings'],
            ['cargo', 'test', '--all-targets', '--all-features', '--no-run'],
        ])

    def test_cache_preparation_does_not_filter_reviewed_source_with_lint_failures(self):
        outcomes = iter([dict(exit_code=101, timed_out=False, cancelled_signal=None),
                         dict(exit_code=0, timed_out=False, cancelled_signal=None)])
        host = SimpleNamespace(snapshot=lambda repo: {'fixed': True},
                               execute_child=lambda *args: next(outcomes),
                               save_json=lambda *args: None)
        with mock.patch.object(continuation.repair.original, 'load_host', return_value=host):
            result = continuation.warm_cache(Path('/tmp/retained/repo'), Path('/tmp/retained'))
        self.assertEqual([r['exit_code'] for r in result['commands']], [101, 0])
        self.assertTrue(result['source_unchanged'])


if __name__ == '__main__':
    unittest.main()
