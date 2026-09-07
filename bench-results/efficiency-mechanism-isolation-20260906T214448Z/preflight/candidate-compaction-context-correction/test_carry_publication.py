"""Publication ownership only: synthetic fixtures, no saved-workflow audit."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
CARRY_SHA = '47d28ef42c3243344cb28d6db680a531b04255cdc310653e45fc3ee81b8418a1'
TEST_SHA = 'efffcc4c03bb1b3d738bc83500dc4ca1c4ffdf9bf3b6e180ed6900286ca144b5'


def load(path, expected=None):
    data = path.read_bytes()
    if expected is not None:
        assert hashlib.sha256(data).hexdigest() == expected
    module = types.ModuleType(path.stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


FIXTURE = load(HERE / 'test_compaction_carry.py', TEST_SHA)
BASE = load(HERE / 'accounting_compaction_carry.py', CARRY_SHA)
M = load(HERE / 'accounting_carry_publication.py')


class PublicationTests(unittest.TestCase):
    def scope(self, root, before=0, after=0, auxiliary_used=False, partial=False,
              unused_before_primary=0):
        ctx, entry = FIXTURE.CarryTests().fake_scope(root, extra_capture=partial)
        args = FIXTURE.repetitions(1)
        sources = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in (root / 'observation').rglob('*') if p.is_file()}
        def execute(entry, frozen, sessions):
            for _ in range(unused_before_primary):
                ctx.load_original()
            primary = ctx.load_original()
            auxiliary = [ctx.load_original() for _ in range(before)]
            inv = primary.reconcile_stream(*args)
            if auxiliary_used:
                auxiliary[0].reconcile_stream(*args)
            for _ in range(after):
                ctx.load_original()
            return dict(run_id=entry['run_id'], status='unknown' if partial else 'validated',
                errors=['retained stopped capture'] if partial else [],
                measurement=dict(status='synthetic_fixture', bounds=None),
                warnings=copy.deepcopy(inv['warnings']), preserved_gaps=copy.deepcopy(inv['gaps']),
                source_sha256=copy.deepcopy(sources))
        ctx.audit_run = execute
        return ctx, entry

    def invoke(self, module, root, **kwargs):
        ctx, entry = self.scope(root, **kwargs)
        with patch.object(module, 'load_context', return_value=ctx):
            return module.audit_run(entry, {}, '/unused')

    def test_auxiliary_loads_before_and_after_primary_preserve_actual_diagnostics(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(M, Path(tmp), before=2, after=2)
        diagnostic = result['repeated_metadata_derivative']
        self.assertEqual(len(diagnostic['capture_diagnostics']), 1)
        self.assertEqual(diagnostic['capture_diagnostics'][0]['original_inner_errors'],
                         ['nonadditive last lacks unchanged explicit compaction boundary'])
        self.assertEqual(diagnostic['original_outer_scope_diagnostic']['status'], 'complete_capture_census')
        self.assertEqual(result['errors'], [])
        self.assertIsNone(result['preserved_gaps'][0]['response_count_upper'])
        self.assertIsNone(result['preserved_gaps'][0]['proof'])

    def test_two_actually_used_modules_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(M, Path(tmp), before=1, auxiliary_used=True)
        self.assertEqual(result['status'], 'unknown')
        self.assertIn('multiple accounting modules reconciled captures', result['errors'])
        self.assertIsNone(result['measurement']['bounds'])

    def test_partial_census_failure_stays_retained_with_auxiliary_loads(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(M, Path(tmp), before=1, after=1, partial=True)
        self.assertEqual(result['errors'], ['retained stopped capture'])
        diagnostic = result['repeated_metadata_derivative']['original_outer_scope_diagnostic']
        self.assertEqual(diagnostic['status'], 'unavailable_partial_capture_census')
        self.assertIsNone(diagnostic['original_outer_scope_error'])

    def test_actual_owner_need_not_be_first_loaded_module(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(M, Path(tmp), before=1, after=1, unused_before_primary=2)
        self.assertEqual(result['errors'], [])
        self.assertEqual(len(result['repeated_metadata_derivative']['capture_diagnostics']), 1)

    def test_predecessor_missing_publication_remains_reproducible(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(BASE, Path(tmp), before=2, after=2)
        self.assertEqual(result['repeated_metadata_derivative']['capture_diagnostics'], [])
        self.assertIsNone(result['repeated_metadata_derivative']['original_outer_scope_diagnostic'])

    def test_no_auxiliary_result_is_exact_except_added_source_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            ctx, entry = self.scope(Path(tmp))
            loader = ctx.load_original
            with patch.object(BASE, 'load_context', return_value=ctx):
                old = BASE.audit_run(entry, {}, '/unused')
            ctx.load_original = loader
            with patch.object(M, 'load_context', return_value=ctx):
                new = M.audit_run(entry, {}, '/unused')
        self.assertFalse(new.pop('diagnostic_publication_derivative')['numerical_predicates_changed'])
        new['source_sha256'].pop(str(HERE / 'accounting_carry_publication.py'))
        self.assertEqual(old, new)

    def test_loaded_source_drift_fails_before_invocation(self):
        read = Path.read_bytes
        def altered(path):
            content = read(path)
            return content + b'\n' if path == M.CARRY_PATH else content
        with patch.object(Path, 'read_bytes', altered):
            with self.assertRaisesRegex(ValueError, 'loaded carry dependency'):
                M.audit_run({}, {}, '/unused')

    def test_unlaunched_unknown_is_not_zero_filled(self):
        result = M.audit_run(dict(run_id='unlaunched'), dict(files=[]), '/unused')
        self.assertEqual(result['status'], 'unknown')
        self.assertIsNone(result['measurement']['bounds'])
        self.assertEqual(result['repeated_metadata_derivative']['capture_diagnostics'], [])


if __name__ == '__main__':
    unittest.main()
