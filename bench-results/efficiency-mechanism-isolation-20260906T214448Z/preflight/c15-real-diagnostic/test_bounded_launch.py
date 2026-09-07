import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

import bounded_launch as L


class LaunchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-launch-guard-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = self.root / 'environment.json'
        self.env.write_text(json.dumps({'variables': {}, 'unset': []}))
        self.admission = self.root / 'admission.json'
        self.value = {'schema': 'work-leaf-single-diagnostic-admission-v1',
            'run_id': 'fixture', 'condition': 'private-test-first', 'observer_condition': 'work-leaf',
            'cwd': str(self.root), 'artifact_root': str(self.root),
            'argv': [sys.executable, '-I', '-c', 'print("original output"); raise SystemExit(7)'],
            'environment_path': str(self.env),
            'source_sha256': {str(self.env): L.sha(self.env)}}

    def save(self):
        self.admission.write_text(json.dumps(self.value))
        return hashlib.sha256(self.admission.read_bytes()).hexdigest()

    def test_original_failure_has_exact_terminal_contract_and_cannot_repeat(self):
        digest = self.save()
        result = L.run(self.admission, digest)
        self.assertEqual(result['launcher_exit_code'], 7)
        self.assertEqual(result['id'], result['run_id'])
        self.assertEqual(result['condition'], 'private-test-first')
        self.assertEqual(result['observer_condition'], 'work-leaf')
        self.assertEqual(result['launch_status'], 'completed')
        self.assertLessEqual(result['started_at'], result['finished_at'])
        self.assertTrue(result['finished_at'].endswith('+00:00'))
        self.assertEqual((self.root / 'PROCESS.stdout').read_bytes(), b'original output\n')
        original = (self.root / 'TERMINAL.json').read_bytes()
        with self.assertRaises((ValueError, FileExistsError)):
            L.run(self.admission, digest)
        self.assertEqual((self.root / 'TERMINAL.json').read_bytes(), original)

    def test_source_drift_rejects_before_attempt(self):
        digest = self.save()
        self.env.write_text('{}')
        with self.assertRaises(ValueError): L.run(self.admission, digest)
        self.assertFalse((self.root / 'ATTEMPT.json').exists())

    def test_wrong_admission_digest_rejects_before_attempt(self):
        self.save()
        with self.assertRaises(ValueError): L.run(self.admission, '0' * 64)
        self.assertFalse((self.root / 'ATTEMPT.json').exists())

    def test_executed_admission_is_the_exact_bytes_that_were_hashed(self):
        digest = self.save(); original = self.admission.read_bytes()
        changed = dict(self.value); changed['argv'] = [sys.executable, '-I', '-c', 'raise SystemExit(0)']
        changed = json.dumps(changed).encode(); reads = 0; read_bytes = Path.read_bytes
        def switched(path):
            nonlocal reads
            if path == self.admission:
                reads += 1
                return original if reads == 1 else changed
            return read_bytes(path)
        with mock.patch.object(Path, 'read_bytes', switched):
            result = L.run(self.admission, digest)
        self.assertEqual(result['launcher_exit_code'], 7)

    def test_spawn_error_is_retained_not_a_fabricated_process_exit(self):
        self.value['argv'] = [str(self.root / 'absent-executable')]
        result = L.run(self.admission, self.save())
        self.assertEqual(result['launch_status'], 'spawn_failed')
        self.assertIsNone(result['launcher_exit_code'])
        self.assertIn('FileNotFoundError', result['error'])
        self.assertTrue((self.root / 'ATTEMPT.json').exists())

    def test_environment_is_parsed_from_its_verified_bytes(self):
        self.value['argv'] = [sys.executable, '-I', '-c',
            'import os; raise SystemExit(9 if "C15_TEST_ONLY" in os.environ else 7)']
        digest = self.save(); original = self.env.read_bytes()
        changed = json.dumps({'variables': {'C15_TEST_ONLY': 'unexpected'}, 'unset': []}).encode()
        reads = 0; read_bytes = Path.read_bytes
        def switched(path):
            nonlocal reads
            if path == self.env:
                reads += 1
                return original if reads == 1 else changed
            return read_bytes(path)
        with mock.patch.object(Path, 'read_bytes', switched):
            result = L.run(self.admission, digest)
        self.assertEqual(result['launcher_exit_code'], 7)


if __name__ == '__main__': unittest.main()
