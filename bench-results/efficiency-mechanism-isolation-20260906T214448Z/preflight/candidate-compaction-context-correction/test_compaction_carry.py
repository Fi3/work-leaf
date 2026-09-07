"""Offline repeated-metadata invariants; never invoke a whole-run audit/provider."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
FROZEN = HERE.parents[1] / 'phases/candidate-screen-01/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z'


def load(path, expected=None):
    data = path.read_bytes()
    if expected:
        assert hashlib.sha256(data).hexdigest() == expected
    module = types.ModuleType(path.stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


BASE = load(FROZEN / 'accounting_untracked_reads.py', 'c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a')
M = load(HERE / 'accounting_compaction_carry.py')


def usage(i=10, c=2, o=3, r=1):
    return dict(inputTokens=i, cachedInputTokens=c, outputTokens=o,
                reasoningOutputTokens=r, totalTokens=i+o, cacheWriteInputTokens=0)


def frame(method, turn_id='turn-a', thread='thread-a', **extra):
    return dict(method=method, params=dict(threadId=thread, turnId=turn_id, **extra))


def scenario(compaction=True):
    clients = [dict(id='start-a', method='turn/start', params=dict(threadId='thread-a'))]
    servers = [dict(id='start-a', result=dict(turn=dict(id='turn-a'))),
               frame('turn/started', turn=dict(id='turn-a')),
               frame('rawResponse/completed', responseId='ordinary', usage=usage()),
               frame('thread/tokenUsage/updated', tokenUsage=dict(total=usage(), last=usage()))]
    native = dict(records={'ordinary': dict(thread_id='thread-a', turn_id='turn-a', usage=BASE.checked_usage(usage()), sources=[])}, compactions={})
    if compaction:
        compact_usage = usage(20, 4, 5, 0)
        stale = usage(0, 0, 0, 0); stale['totalTokens'] = 7
        native['records']['compact'] = dict(thread_id='thread-a', turn_id='turn-a', usage=BASE.checked_usage(compact_usage), sources=[])
        native['compactions']['compact'] = dict(thread_id='thread-a', turn_id='turn-a', sources=[])
        servers += [frame('item/started', item=dict(type='contextCompaction', id='compact-item')),
                    frame('rawResponse/completed', responseId='compact', usage=compact_usage),
                    frame('thread/tokenUsage/updated', tokenUsage=dict(total=usage(), last=stale)),
                    frame('item/completed', item=dict(type='contextCompaction', id='compact-item'))]
    servers += [frame('turn/completed', turn=dict(id='turn-a', status='completed'))]
    return clients, servers, [], native


def repetitions(count=5):
    clients, servers, grace, native = scenario()
    stale = copy.deepcopy(servers[-3]['params']['tokenUsage'])
    servers.pop()
    for n in range(count):
        turn = 'turn-a' if n == 0 else f'turn-{n}'
        if n:
            clients.append(dict(id=f'start-{n}', method='turn/start', params=dict(threadId='thread-a')))
            servers += [dict(id=f'start-{n}', result=dict(turn=dict(id=turn))),
                        frame('turn/started', turn_id=turn, turn=dict(id=turn))]
        servers += [frame('item/completed', turn_id=turn, item=dict(type='agentMessage', id=f'directive-{n}', text='@work-leaf read source.rs')),
                    frame('item/started', turn_id=turn, item=dict(type='reasoning', id=f'next-item-{n}')),
                    frame('thread/tokenUsage/updated', turn_id=turn, tokenUsage=copy.deepcopy(stale)),
                    frame('turn/completed', turn_id=turn, turn=dict(id=turn, status='interrupted'))]
        clients.append(dict(id=f'interrupt-{n}', **frame('turn/interrupt', turn_id=turn)))
        grace.append(dict(thread_id='thread-a', turn_id=turn, outcome='forwarded-after-output-resumed'))
    return clients, servers, grace, native


class CarryTests(unittest.TestCase):
    def run_case(self, args):
        return M.derive_reconcile(BASE)(*args)

    def test_original_valid_results_are_identical(self):
        for compaction in (False, True):
            args = scenario(compaction)
            self.assertEqual(self.run_case(args), BASE.reconcile_stream(*args))

    def test_original_rejects_repetition_and_derivative_retains_warning(self):
        args = repetitions(1)
        self.assertEqual(BASE.reconcile_stream(*args)['errors'], ['nonadditive last lacks unchanged explicit compaction boundary'])
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['omitted_compaction_ids'], ['compact'])
        self.assertEqual(len(result['raw_responses']), 2)
        self.assertEqual(len(result['warnings']), 2)
        carry = result['warnings'][1]
        self.assertEqual(carry['classification'], 'unusable_compaction_last_metadata_carry_forward')
        self.assertEqual(carry['origin_server_line'], 7)
        self.assertFalse(carry['used_for_fresh_usage_or_tail_recovery'])

    def test_five_generated_unmeasured_tails_stay_unbounded(self):
        args = repetitions()
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertEqual(len(result['warnings']), 6)
        self.assertEqual(len(result['gaps']), 5)
        self.assertEqual({g['turn_id'] for g in result['gaps']}, {'turn-a', 'turn-1', 'turn-2', 'turn-3', 'turn-4'})
        self.assertTrue(all(g['response_count_upper'] is None and g['proof'] is None for g in result['gaps']))
        self.assertEqual(result['late_terminal_usage_recoveries'], [])
        self.assertEqual(result['thread_ledger']['thread-a']['corrected']['raw_input_plus_output'], 38)

    def test_origin_must_be_explicit_native_compaction(self):
        args = repetitions(1); args[3]['compactions'].clear()
        self.assertTrue(self.run_case(args)['errors'])

    def test_repetition_before_lifecycle_completion_is_rejected(self):
        args = list(repetitions(1)); repeated = args[1].pop(-2); args[1].insert(7, repeated)
        self.assertTrue(self.run_case(args)['errors'])

    def test_changed_last_or_cumulative_invalidates(self):
        for field, key, value in [('last', 'totalTokens', 8), ('last', 'extra', False), ('total', 'extra', False), ('last', 'inputTokens', True)]:
            with self.subTest(field=field, key=key):
                args = repetitions(1); args[1][-2]['params']['tokenUsage'][field][key] = value
                self.assertTrue(self.run_case(args)['errors'])

    def test_unknown_turn_and_typed_reply_collision_are_rejected(self):
        args = repetitions(2); args[1][-2]['params']['turnId'] = 'not-accepted'
        self.assertTrue(self.run_case(args)['errors'])
        args = repetitions(2)
        for client in args[0]:
            if client.get('id') == 'start-1': client['id'] = 1
        for server in args[1]:
            if server.get('id') == 'start-1': server['id'] = True
        self.assertTrue(self.run_case(args)['errors'])

    def test_no_cross_thread_origin_transfer(self):
        args = repetitions(1); args[0].append(dict(id='other-start', method='turn/start', params=dict(threadId='other-thread')))
        args[1].insert(-2, dict(id='other-start', result=dict(turn=dict(id='other-turn'))))
        args[1][-2]['params'].update(threadId='other-thread', turnId='other-turn')
        self.assertTrue(self.run_case(args)['errors'])

    def test_new_zero_usage_response_invalidates_origin(self):
        args = repetitions(1)
        args[1].insert(-2, frame('rawResponse/completed', responseId='new-zero', usage=usage(0, 0, 0, 0)))
        args[3]['records']['new-zero'] = dict(thread_id='thread-a', turn_id='turn-a', usage=BASE.checked_usage(usage(0, 0, 0, 0)), sources=[])
        self.assertTrue(self.run_case(args)['errors'])

    def test_changed_additive_metadata_cannot_restore_old_origin(self):
        args = repetitions(1)
        args[1].insert(-2, frame('thread/tokenUsage/updated', tokenUsage=dict(total=usage(), last=usage())))
        self.assertTrue(self.run_case(args)['errors'])

    def test_cumulative_regression_never_becomes_a_carry_warning(self):
        args = repetitions(1)
        args[1][-2]['params']['tokenUsage']['total'] = usage(1, 0, 1, 0)
        result = self.run_case(args)
        self.assertEqual(result['errors'], ['cumulative provider usage regressed'])
        self.assertEqual(len(result['warnings']), 1)

    def test_duplicate_exact_raw_identity_does_not_double_charge(self):
        args = repetitions(1); args[1].insert(-2, copy.deepcopy(args[1][5]))
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['omitted_compaction_ids'], ['compact'])
        self.assertEqual(len(result['raw_responses']), 2)

    def test_a_second_compaction_requires_and_owns_its_new_origin(self):
        args = repetitions(1)
        second = copy.deepcopy(args[1][4:8])
        for value in second:
            q = value['params']
            if 'item' in q: q['item']['id'] = 'second-item'
            if 'responseId' in q: q['responseId'] = 'second-compact'
        args[1][-2:-2] = second
        args[3]['records']['second-compact'] = copy.deepcopy(args[3]['records']['compact'])
        args[3]['compactions']['second-compact'] = copy.deepcopy(args[3]['compactions']['compact'])
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['omitted_compaction_ids'], ['compact', 'second-compact'])
        self.assertEqual(result['warnings'][-1]['response_id'], 'second-compact')
        self.assertEqual(result['warnings'][-1]['item_id'], 'second-item')

    def test_malformed_or_incomplete_second_compaction_is_not_ignored(self):
        args = repetitions(1)
        args[1].insert(-2, frame('item/started', item=dict(type='contextCompaction', id='unclosed')))
        self.assertTrue(self.run_case(args)['errors'])

    def test_other_thread_interleaving_does_not_change_ownership(self):
        args = repetitions(1)
        args[0].append(dict(id='other-start', method='turn/start', params=dict(threadId='other-thread')))
        args[1][-2:-2] = [dict(id='other-start', result=dict(turn=dict(id='other-turn'))),
            frame('turn/started', thread='other-thread', turn_id='other-turn', turn=dict(id='other-turn')),
            frame('rawResponse/completed', thread='other-thread', turn_id='other-turn', responseId='other', usage=usage()),
            frame('thread/tokenUsage/updated', thread='other-thread', turn_id='other-turn', tokenUsage=dict(total=usage(), last=usage())),
            frame('turn/completed', thread='other-thread', turn_id='other-turn', turn=dict(id='other-turn', status='completed'))]
        args[3]['records']['other'] = dict(thread_id='other-thread', turn_id='other-turn', usage=BASE.checked_usage(usage()), sources=[])
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['warnings'][-1]['thread_id'], 'thread-a')

    def test_input_frames_and_original_functions_are_unchanged(self):
        args = repetitions(); saved = copy.deepcopy(args)
        original_code = BASE.reconcile_stream.__code__
        strict_code = BASE.STRICT.accounting.__code__
        self.run_case(args)
        self.assertEqual(args, saved)
        self.assertIs(BASE.reconcile_stream.__code__, original_code)
        self.assertIs(BASE.STRICT.accounting.__code__, strict_code)

    def test_strict_accounting_retains_unbounded_upper_endpoint(self):
        inv = self.run_case(repetitions())
        observed = dict(usage_scopes=dict(total_workflow=BASE.checked_usage(usage(30, 6, 8, 1))),
            interrupted_provider_turns=5, errors=['interrupted provider turn has no complete usage: count=5'],
            capture_complete=False, invocation_count=1, complete_invocation_count=1,
            model_strata=[dict(model=BASE.BASE.POLICY['model'], effort=BASE.BASE.POLICY['reasoning_effort'])])
        measured = BASE.STRICT.accounting(observed, inv, BASE.BASE.POLICY)
        self.assertEqual(measured['status'], 'unbounded_accounting_gap')
        self.assertTrue(all(b['upper'] is None for b in measured['bounds'].values()))

    def test_source_derivation_rejects_changed_pinned_bytes(self):
        with patch.object(Path, 'read_bytes', return_value=b'different source'):
            with self.assertRaisesRegex(ValueError, 'source identity'):
                M.derive_reconcile(BASE)

    def test_unknown_same_thread_lifecycle_invalidates_carry(self):
        for method in ('thread/contextReplaced', 'turn/restarted'):
            args = repetitions(1); args[1].insert(-2, frame(method))
            self.assertTrue(self.run_case(args)['errors'])

    def test_unknown_lifecycle_cannot_hide_ownership(self):
        args = repetitions(1); args[1].insert(-2, dict(method='thread/contextReplaced', params={}))
        self.assertTrue(self.run_case(args)['errors'])

    def test_unknown_other_thread_lifecycle_does_not_claim_current_origin(self):
        args = repetitions(1); args[1].insert(-2, frame('thread/contextReplaced', thread='other'))
        result = self.run_case(args)
        self.assertEqual(result['errors'], [])
        self.assertIsNone(result['gaps'][0]['response_count_upper'])

    def test_original_failure_diagnostics_are_separate_from_derived_result(self):
        wrapper, diagnostics = M.instrument_original(BASE)
        original_function = BASE.reconcile_stream
        result = wrapper(*repetitions())
        self.assertEqual(result['errors'], [])
        self.assertEqual(diagnostics[0]['original_inner_errors'], ['nonadditive last lacks unchanged explicit compaction boundary'])
        self.assertEqual(diagnostics[0]['original_thread_ledger_ids'], [])
        self.assertEqual(diagnostics[0]['derived_thread_ledger_ids'], ['thread-a'])
        self.assertIs(BASE.reconcile_stream, original_function)

    def test_unlaunched_scope_remains_unknown_without_zero_fill(self):
        result = M.audit_run(dict(run_id='not-launched'), dict(files=[]), '/unused')
        self.assertEqual(result['status'], 'unknown')
        self.assertIsNone(result['measurement']['bounds'])
        self.assertEqual(result['repeated_metadata_derivative']['capture_diagnostics'], [])

    def fake_scope(self, root, extra_capture=False, mismatch=False):
        args = repetitions(1)
        app = root / 'observation/app-server/capture-a'
        app.mkdir(parents=True)
        paths = []
        for name, rows in [('client-to-server.raw', args[0]), ('server-to-client.raw', args[1]), ('provider-usage-grace.jsonl', args[2])]:
            path = app / name
            path.write_text(''.join(json.dumps(row)+'\n' for row in rows)); paths.append(path)
        if extra_capture:
            (app.parent / 'capture-b').mkdir()
        analysis = root / 'observation/analysis.json'
        analysis.write_text(json.dumps(dict(threads=[dict(thread_id='thread-a')])))
        paths.append(analysis)
        sources = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
        ctx = types.SimpleNamespace(read_bytes=lambda p:Path(p).read_bytes(),
            load_original=lambda:load(FROZEN / 'accounting_untracked_reads.py', M.ORIGINAL_SHA))
        def execute(entry, frozen, sessions):
            original = ctx.load_original()
            actual = copy.deepcopy(args)
            if mismatch:
                actual[1].insert(1, {})
            original.reconcile_stream(*actual)
            return dict(run_id=entry['run_id'], status='unknown' if extra_capture else 'validated',
                errors=['retained interrupted collector'] if extra_capture else [],
                measurement=dict(status='synthetic_fixture', bounds=None), source_sha256=copy.deepcopy(sources))
        ctx.audit_run = execute
        return ctx, dict(run_id='synthetic-owned-run', artifact=str(root))

    def test_partial_capture_census_does_not_claim_whole_outer_scope(self):
        with tempfile.TemporaryDirectory() as tmp:
            ctx, entry = self.fake_scope(Path(tmp), extra_capture=True)
            with patch.object(M, 'load_context', return_value=ctx):
                result = M.audit_run(entry, {}, '/unused')
            diagnostic = result['repeated_metadata_derivative']['original_outer_scope_diagnostic']
            self.assertEqual(diagnostic['status'], 'unavailable_partial_capture_census')
            self.assertIsNone(diagnostic['original_outer_scope_error'])
            self.assertEqual(result['errors'], ['retained interrupted collector'])

    def test_capture_source_must_match_executed_rows_not_only_sorted_position(self):
        with tempfile.TemporaryDirectory() as tmp:
            ctx, entry = self.fake_scope(Path(tmp), mismatch=True)
            with patch.object(M, 'load_context', return_value=ctx):
                result = M.audit_run(entry, {}, '/unused')
            self.assertEqual(result['status'], 'unknown')
            self.assertIn('capture source rows differ from executed reconciliation', result['errors'])

    def test_complete_capture_scope_records_exact_original_outer_guard(self):
        with tempfile.TemporaryDirectory() as tmp:
            ctx, entry = self.fake_scope(Path(tmp))
            with patch.object(M, 'load_context', return_value=ctx):
                result = M.audit_run(entry, {}, '/unused')
            self.assertEqual(result['errors'], [])
            diagnostic = result['repeated_metadata_derivative']['original_outer_scope_diagnostic']
            self.assertEqual(diagnostic['status'], 'complete_capture_census')
            self.assertEqual(diagnostic['original_outer_scope_error'], 'raw cumulative thread scope differs from observer')
            self.assertFalse(diagnostic['second_original_audit_run_executed'])


if __name__ == '__main__':
    unittest.main()
