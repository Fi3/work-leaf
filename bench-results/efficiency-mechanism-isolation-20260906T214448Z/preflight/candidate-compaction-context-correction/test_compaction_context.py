import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('context_correction', HERE / 'accounting_compaction_context.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def fixture():
    usage = dict(input_tokens=11, cached_input_tokens=2, output_tokens=3,
                 reasoning_output_tokens=1, total_tokens=14)
    record = dict(response_id='response-a', thread_id='thread-a', session_id='thread-a',
                  turn_id='turn-a', usage=copy.deepcopy(usage), thread_token_usage=copy.deepcopy(usage), turn_token_usage=copy.deepcopy(usage))
    context = dict(turn_id='turn-a', model='gpt-5.5', effort='xhigh', cwd='/owned/repo')
    user = dict(type='message', role='user', id='native-user', content=[dict(type='input_text', text='exact request')],
                internal_chat_message_metadata_passthrough=dict(turn_id='turn-a', content_item_kinds=['user.text']))
    rows = [dict(type='session_meta', payload=dict(id='thread-a', cwd='/owned/repo')),
            dict(type='token_usage_record', payload=record),
            dict(type='compacted', payload=dict(compaction_response_id='response-a', latest_token_usage_record=copy.deepcopy(record))),
            dict(type='turn_context', payload=context), dict(type='response_item', payload=user)]
    meta = dict(thread_id='thread-a', model='gpt-5.5', effort='xhigh', cwd='/owned/repo')
    thread = dict(id='thread-rpc', method='thread/start', params=dict(cwd='/owned/repo'))
    request = dict(id='turn-rpc', method='turn/start', params=dict(threadId='thread-a', input=[dict(type='text', text='exact request')]))
    servers = [dict(id='thread-rpc', result=dict(thread=dict(id='thread-a'))),
               dict(id='turn-rpc', result=dict(turn=dict(id='turn-a'))),
               dict(method='item/started', params=dict(threadId='thread-a', turnId='turn-a', item=dict(type='contextCompaction', id='compact-a'))),
               dict(method='rawResponse/completed', params=dict(threadId='thread-a', turnId='turn-a', responseId='response-a', usage=dict(inputTokens=11, cachedInputTokens=2, outputTokens=3, reasoningOutputTokens=1, totalTokens=14))),
               dict(method='item/completed', params=dict(threadId='thread-a', turnId='turn-a', item=dict(type='contextCompaction', id='compact-a'))),
               dict(method='item/completed', params=dict(threadId='thread-a', turnId='turn-a', item=dict(type='userMessage', id='public-user', content=[dict(type='text', text='exact request')])))]
    capture = dict(path='/capture', clients=[thread, request], forwarded=copy.deepcopy([thread, request]), servers=servers)
    return rows, meta, [capture]


class ProofTests(unittest.TestCase):
    def prove(self, rows, meta, captures):
        return audit.prove_future_contexts(rows, meta, captures, 'gpt-5.5', 'xhigh', '/native')

    def test_exact_future_context_proof_preserves_rows_and_native_line_numbers(self):
        rows, meta, captures = fixture(); before = copy.deepcopy(rows)
        proofs = self.prove(rows, meta, captures)
        self.assertEqual(len(proofs), 1)
        self.assertEqual((proofs[0]['native_response_line'], proofs[0]['native_marker_line'], proofs[0]['native_context_line'], proofs[0]['native_user_line']), (2, 3, 4, 5))
        self.assertEqual(rows, before)
        original = audit.load_original()
        self.assertTrue(original.NATIVE.audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')['errors'])
        corrected = audit.derived_native(original, proofs)
        result = corrected.audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')
        self.assertEqual(result['errors'], [])
        self.assertEqual(result['records']['response-a']['line'], 2)
        self.assertTrue(original.NATIVE.audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')['errors'])

    def test_already_valid_response_is_unchanged(self):
        rows, meta, captures = fixture(); rows.insert(1, rows.pop(3))
        original = audit.load_original(); proofs = self.prove(rows, meta, captures)
        self.assertEqual(proofs, [])
        self.assertEqual(original.NATIVE.audit_rollout(rows, meta, 'gpt-5.5', 'xhigh'), audit.derived_native(original, proofs).audit_rollout(rows, meta, 'gpt-5.5', 'xhigh'))

    def test_missing_or_conflicting_context_is_rejected(self):
        for mutation in ('missing', 'model', 'effort', 'cwd', 'duplicate', 'conflicting'):
            with self.subTest(mutation=mutation):
                rows, meta, cap = fixture()
                if mutation == 'missing': rows.pop(3)
                elif mutation == 'duplicate': rows.append(copy.deepcopy(rows[3]))
                elif mutation == 'conflicting':
                    other = copy.deepcopy(rows[3]); other['payload']['model'] = 'other'; rows.append(other)
                else: rows[3]['payload'][mutation] = 'other'
                with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_only_explicit_adjacent_compaction_can_qualify(self):
        for mutation in ('absent', 'wrong-response', 'wrong-latest', 'not-adjacent'):
            with self.subTest(mutation=mutation):
                rows, meta, cap = fixture()
                if mutation == 'absent': rows.pop(2)
                elif mutation == 'wrong-response': rows[2]['payload']['compaction_response_id'] = 'other'
                elif mutation == 'wrong-latest': rows[2]['payload']['latest_token_usage_record']['turn_id'] = 'other'
                else: rows.insert(2, dict(type='event_msg', payload={}))
                with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_cross_native_identity_and_boolean_ids_fail(self):
        for field in ('thread_id', 'session_id', 'turn_id', 'response_id'):
            for value in (True, '', 'other'):
                with self.subTest(field=field, value=value):
                    rows, meta, cap = fixture(); rows[1]['payload'][field] = value
                    rows[2]['payload']['latest_token_usage_record'] = copy.deepcopy(rows[1]['payload'])
                    with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_missing_rejected_or_typed_alias_rpc_fails(self):
        for mutation in ('missing', 'rejected', 'alias', 'forwarded'):
            with self.subTest(mutation=mutation):
                rows, meta, cap = fixture()
                if mutation == 'missing': cap[0]['servers'].pop(1)
                elif mutation == 'rejected': cap[0]['servers'][1]['error'] = {}
                elif mutation == 'alias':
                    cap[0]['clients'][1]['id'] = 1; cap[0]['forwarded'][1]['id'] = 1; cap[0]['servers'][1]['id'] = True
                else: cap[0]['forwarded'][1]['params']['input'][0]['text'] = 'other'
                with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_public_lifecycle_must_be_unique_ordered_and_same_turn(self):
        for mutation in ('start', 'complete', 'id', 'turn', 'duplicate', 'order'):
            with self.subTest(mutation=mutation):
                rows, meta, cap = fixture(); s = cap[0]['servers']
                if mutation == 'start': s.pop(2)
                elif mutation == 'complete': s.pop(4)
                elif mutation == 'id': s[4]['params']['item']['id'] = 'other'
                elif mutation == 'turn': s[3]['params']['turnId'] = 'other'
                elif mutation == 'duplicate': s.insert(4, copy.deepcopy(s[3]))
                else: s[3], s[4] = s[4], s[3]
                with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_exact_native_public_request_text_and_unique_users_required(self):
        for mutation in ('native', 'public', 'duplicate-native', 'duplicate-public', 'user-before-context'):
            with self.subTest(mutation=mutation):
                rows, meta, cap = fixture()
                if mutation == 'native': rows[4]['payload']['content'][0]['text'] = 'other'
                elif mutation == 'public': cap[0]['servers'][5]['params']['item']['content'][0]['text'] = 'other'
                elif mutation == 'duplicate-native': rows.append(copy.deepcopy(rows[4]))
                elif mutation == 'duplicate-public': cap[0]['servers'].append(copy.deepcopy(cap[0]['servers'][5]))
                else: rows[3], rows[4] = rows[4], rows[3]
                with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_wrong_raw_usage_and_intervening_generation_fail(self):
        rows, meta, cap = fixture(); cap[0]['servers'][3]['params']['usage']['inputTokens'] = 12
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)
        rows, meta, cap = fixture(); rows.insert(3, dict(type='response_item', payload=dict(type='reasoning')))
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_correction_cannot_waive_unrelated_prefix_failure(self):
        rows, meta, cap = fixture(); rows[1]['payload']['thread_token_usage']['total_tokens'] = 99
        rows[2]['payload']['latest_token_usage_record'] = copy.deepcopy(rows[1]['payload'])
        proofs = self.prove(rows, meta, cap)
        result = audit.derived_native(audit.load_original(), proofs).audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')
        self.assertTrue(result['errors'])

    def test_proof_is_bound_to_exact_response_payload(self):
        rows, meta, cap = fixture(); proofs = self.prove(rows, meta, cap)
        rows[1]['payload']['response_id'] = 'other'
        result = audit.derived_native(audit.load_original(), proofs).audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')
        self.assertTrue(result['errors'])

    def test_exact_source_loader_rejects_modified_bytes(self):
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'module.py'; p.write_bytes(b'VALUE = 1\n'); digest = hashlib.sha256(p.read_bytes()).hexdigest()
            self.assertEqual(audit.compile_source(p, digest).VALUE, 1)
            p.write_bytes(b'VALUE = 2\n')
            with self.assertRaises(ValueError): audit.compile_source(p, digest)

    def test_generated_event_cannot_hide_between_marker_and_context(self):
        rows, meta, cap = fixture()
        rows.insert(3, dict(type='event_msg', payload=dict(type='agent_message')))
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_explicit_public_scope_must_match_future_context(self):
        for method_index in (0, 1):
            for key in ('model', 'effort', 'reasoningEffort', 'cwd'):
                with self.subTest(method_index=method_index, key=key):
                    rows, meta, cap = fixture()
                    for channel in ('clients', 'forwarded'): cap[0][channel][method_index]['params'][key] = 'other'
                    with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_proof_activation_is_bound_to_full_native_metadata(self):
        rows, meta, cap = fixture(); proofs = self.prove(rows, meta, cap)
        meta['cwd'] = '/other'
        self.assertTrue(audit.derived_native(audit.load_original(), proofs).audit_rollout(rows, meta, 'gpt-5.5', 'xhigh')['errors'])

    def test_text_metadata_must_be_absent_or_exact_empty_list(self):
        for value in (False, 0, None, {}, ''):
            rows, meta, cap = fixture()
            cap[0]['servers'][5]['params']['item']['content'][0]['text_elements'] = value
            with self.subTest(value=value), self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_other_response_inside_compaction_window_is_ambiguous(self):
        rows, meta, cap = fixture(); other = copy.deepcopy(cap[0]['servers'][3]); other['params']['responseId'] = 'other'
        cap[0]['servers'].insert(4, other)
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_proven_observer_thread_metadata_rewrite_only(self):
        rows, meta, cap = fixture()
        cap[0]['forwarded'][0]['params']['experimentalRawEvents'] = True
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)
        cap[0]['metadata_forwarding_verified'] = True
        self.assertEqual(len(self.prove(rows, meta, cap)), 1)
        cap[0]['forwarded'][0]['params']['model'] = 'other'
        with self.assertRaises(ValueError): self.prove(rows, meta, cap)

    def test_corrected_native_ledger_preserves_physical_identity(self):
        rows, meta, cap = fixture(); original = audit.load_original(); proofs = self.prove(rows, meta, cap)
        with self.assertRaises(ValueError): original.native_ledger(rows, meta, 'gpt-5.5', 'xhigh', '/native')
        result = audit.corrected_ledger(original, rows, meta, 'gpt-5.5', 'xhigh', '/native', proofs)
        self.assertEqual(result['records']['response-a']['sources'], [{'path': '/native', 'native_line': 2}])
        self.assertEqual(result['compactions']['response-a']['sources'], [{'path': '/native', 'native_line': 3}])

    def test_eligibility_cli_preserves_failure_and_refuses_overwrite(self):
        import subprocess
        import sys
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            root = Path(d); phase = root/'phase.json'; score = root/'score.json'; request = root/'input.json'; output = root/'output.json'
            phase.write_text(json.dumps(dict(model='gpt-5.5', reasoning_effort='xhigh', files=[dict(path=str(audit.ORIGINAL_PATH), sha256=audit.ORIGINAL_SHA, role='frozen-evidence')])))
            score.write_text(json.dumps(dict(runs=[dict(run_id='retained-failure', started_at='2026-01-01T00:00:00Z', artifact=None, launcher_exit_code=1, launch_status='completed')])))
            ref = lambda p: dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest())
            request.write_text(json.dumps(dict(schema='work-leaf-compaction-context-input-v1', run_id='retained-failure', helper_sha256=hashlib.sha256(Path(audit.__file__).read_bytes()).hexdigest(), phase_manifest=ref(phase), score_manifest=ref(score), sessions_root=str(root))))
            command = [sys.executable, '-B', audit.__file__, '--input', str(request), '--input-sha256', ref(request)['sha256'], '--output', str(output)]
            first = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(first.returncode, 1, first.stderr)
            result = json.loads(output.read_bytes()); self.assertEqual(result['status'], 'unverifiable')
            self.assertEqual(result['outcome']['launcher_exit_code'], 1)
            self.assertFalse(result['whole_workflow_totals_computed']); self.assertFalse(result['token_values_exported'])
            self.assertNotIn('usage', result); self.assertNotIn('measurement', result)
            before = output.read_bytes(); second = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertNotEqual(second.returncode, 0); self.assertEqual(output.read_bytes(), before)

    def test_explicit_source_digest_cannot_be_null_or_malformed(self):
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)/'source'; path.write_bytes(b'{}')
            for invalid in (None, '', True, 'x'*64, 'A'*64):
                with self.subTest(invalid=invalid), self.assertRaises(ValueError): audit.Sources().read(path, invalid)

    def test_public_index_is_shared_across_qualifying_native_threads(self):
        from unittest.mock import patch
        rows, meta, captures = fixture()
        second_rows, second_meta, second_caps = json.loads(json.dumps((rows, meta, captures)).replace('thread-a', 'thread-b').replace('response-a', 'response-b'))
        second_caps[0]['path'] = '/capture-b'; all_captures = captures + second_caps; cache = {}
        with patch.object(audit, 'public_index', wraps=audit.public_index) as index:
            self.assertEqual(len(audit.prove_future_contexts(rows, meta, all_captures, 'gpt-5.5', 'xhigh', '/a', public_cache=cache)), 1)
            self.assertEqual(len(audit.prove_future_contexts(second_rows, second_meta, all_captures, 'gpt-5.5', 'xhigh', '/b', public_cache=cache)), 1)
            self.assertEqual(index.call_count, 1)


if __name__ == '__main__':
    unittest.main()
