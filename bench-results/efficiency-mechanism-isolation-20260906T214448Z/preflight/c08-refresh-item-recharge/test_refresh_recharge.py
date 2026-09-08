"""Synthetic only: no saved native/capture payload or accounting calls."""
import copy
import hashlib
import json
import unittest

import refresh_recharge as adapter


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def fixture():
    trace = {'runs': []}
    proofs, native, raw, hashes = {}, {}, {}, []
    lower = {'status': 'qualified_observed_lower', 'errors': [], 'candidates': []}
    assignments = ((3, 1), (3, 1, 1, 1, 3, 3), (3, 1, 3))
    for run_number, (run, lines) in enumerate(adapter.RUN_LINES.items()):
        cap = 'capture-' + str(run_number)
        raw_source = '/synthetic/' + run + '/' + cap + '/server-to-client.raw'
        proof = {'run_id': run, 'status': 'unverifiable',
                 'errors': ['trace-occurrence-not-joined'] * 2,
                 'native_membership': {'errors': [], 'threads': [], 'contexts': []},
                 'project_inventory': {'errors': []},
                 'invocation_streams': [{'errors': []}],
                 'frame_proofs': [{'capture_id': cap, 'turn_closure': {'errors': []}}],
                 'delivery': {'inputs': []}, 'source_sha256': {raw_source: 'a' * 64}}
        tr = {'run_id': run, 'records': []}
        ledger, previous, server = {}, {}, []
        for index, (line, owner) in enumerate(zip(lines, assignments[run_number])):
            thread = run + '-author-' + str(owner)
            source = '/synthetic/' + thread + '.jsonl'
            if source not in native:
                native[source] = []
                proof['native_membership']['threads'].append({'thread_id': thread, 'source': source})
                proof['source_sha256'][source] = 'b' * 64
            turn = thread + '-turn-' + str(index)
            native_line = 100 * (index + 1)
            item = thread + '-input-' + str(index)
            body = 'DO_NOT_EXPORT_BODY_é_🪴_' + str(index)
            text = 'refresh envelope\n' + body + '\nend'
            payload = {'type': 'message', 'role': 'user', 'id': item,
                       'content': [{'type': 'input_text', 'text': text}],
                       'internal_chat_message_metadata_passthrough':
                       {'turn_id': turn, 'content_item_kinds': ['user.text']}}
            native[source].append((native_line, {'type': 'response_item', 'payload': payload}))
            inp = {'status': 'joined', 'thread_id': thread, 'turn_id': turn,
                   'native_source': source, 'native_line': native_line, 'native_item_id': item,
                   'input_bytes': len(text.encode()), 'input_sha256': hashlib.sha256(text.encode()).hexdigest(),
                   'public_line': native_line, 'public_item_id': 'public-' + item,
                   'capture_id': cap, 'request_line': index + 1}
            proof['delivery']['inputs'].append(inp)
            proof['native_membership']['contexts'].append({'thread_id': thread, 'turn_id': turn})
            tr['records'].append({'eligible': True, 'event': 'automatic-refresh',
                                 'site': 'automatic-edit-refresh', 'trace_line': line,
                                 'direct_full_text_join': True, 'unowned_ranges_identical': True,
                                 'saved_trace_status': 'joined', 'input': copy.deepcopy(inp),
                                 'full_current_bodies': [{'path': 'file.rs',
                                  'body_start': len('refresh envelope\n'.encode()),
                                  'body_end': len(('refresh envelope\n' + body).encode()),
                                  'bytes': len(body.encode()),
                                  'sha256': hashlib.sha256(body.encode()).hexdigest()}]})
            hashes.append({'run_id': run, 'trace_line': line, 'payload_sha256': digest(payload)})
            previous.setdefault(thread, []).append(item)
            for suffix in (1, 2):
                rid, output_id = turn + '-response-' + str(suffix), turn + '-output-' + str(suffix)
                usage = {'input_tokens': 5 * len(previous[thread]), 'cached_input_tokens': 0,
                         'output_tokens': 1, 'reasoning_output_tokens': 0}
                native[source].append((native_line + suffix * 2 - 1,
                    {'type': 'response_item', 'payload': {'id': output_id, 'type': 'message',
                     'role': 'assistant', 'turn_id': turn,
                     'content': [{'type': 'output_text', 'text': 'DO_NOT_EXPORT_OUTPUT'}]}}))
                native[source].append((native_line + suffix * 2,
                    {'type': 'token_usage_record', 'payload': {'response_id': rid,
                     'thread_id': thread, 'turn_id': turn, 'usage': usage}}))
                ledger[rid] = {'thread_id': thread, 'turn_id': turn, 'usage': usage,
                               'sources': [{'path': source, 'native_line': native_line + suffix * 2}]}
                counts = lambda amount, output=0: {'input_tokens': amount, 'cached_tokens': 0,
                                                   'cache_write_tokens': 0, 'output_tokens': output}
                items = {key: counts(5) for key in previous[thread]}
                items[output_id] = counts(0, 1)
                params = {'threadId': thread, 'turnId': turn, 'responseId': rid,
                          'usage': {'inputTokens': usage['input_tokens'], 'cachedInputTokens': 0,
                                    'cacheWriteInputTokens': 0, 'outputTokens': 1,
                                    'reasoningOutputTokens': 0, 'totalTokens': usage['input_tokens'] + 1},
                          'usageMetadata': {'metadata': {'attribution':
                                           {'items': items, 'request_fields': {}}}}}
                server.append({'method': 'rawResponse/completed', 'params': params,
                               '_audit_source': raw_source, '_audit_line': native_line + suffix * 2})
        for source in native:
            native[source].sort()
        proofs[run], raw[run] = proof, server
        trace['runs'].append(tr)
        lower['candidates'].append({'run_id': run, 'status': 'qualified_observed_lower',
                                   'errors': [], 'response_count': len(ledger),
                                   'response_evidence': ledger, 'response_evidence_sha256': digest(ledger),
                                   'original_errors': ['retained original error'],
                                   'accepted_turns_without_recorded_response': []})
    return trace, proofs, lower, native, raw, hashes


class RefreshRechargeTests(unittest.TestCase):
    def test_complete_population_actual_frozen_extractor_and_safe_projection(self):
        prepared = adapter.prepare(*fixture())
        self.assertEqual([len(row['targets']) for row in prepared], [2, 6, 3])
        results = [adapter.extract(row) for row in prepared]
        self.assertTrue(all(r['status'] == 'exact_observed_attribution' for r in results))
        self.assertEqual(sum(len(r['evidence']['items']) for r in results), 11)
        self.assertTrue(all(item['charges'] for r in results for item in r['evidence']['items']))
        self.assertTrue(all(item['categories'] == ['automatic_refresh_input']
                            for r in results for item in r['evidence']['items']))
        self.assertNotIn('DO_NOT_EXPORT', json.dumps(results))
        self.assertTrue(all(r['full_attribution_sha256'] == r['evidence']['full_attribution_sha256']
                            for r in results))
        self.assertTrue(any(len(item['charges']) > 2 for r in results for item in r['evidence']['items']))

    def test_missing_target_rejects_before_extract(self):
        args = fixture(); args[0]['runs'][0]['records'].pop()
        with self.assertRaises(ValueError): adapter.prepare(*args)

    def test_duplicate_native_target_rejects(self):
        args = fixture(); rows = args[0]['runs'][1]['records']
        rows[2]['input'] = copy.deepcopy(rows[1]['input'])
        with self.assertRaises(ValueError): adapter.prepare(*args)

    def test_foreign_public_id_cannot_replace_native_id(self):
        args = fixture(); target = args[0]['runs'][0]['records'][0]
        target['input']['native_item_id'] = target['input']['public_item_id']
        with self.assertRaises(ValueError): adapter.prepare(*args)

    def test_full_payload_hash_and_full_text_hash_have_distinct_guards(self):
        for what in ('payload', 'text', 'body'):
            args = fixture()
            if what == 'payload': args[5][0]['payload_sha256'] = 'f' * 64
            elif what == 'text': args[0]['runs'][0]['records'][0]['input']['input_sha256'] = 'f' * 64
            else: args[0]['runs'][0]['records'][0]['full_current_bodies'][0]['sha256'] = 'f' * 64
            with self.subTest(what=what), self.assertRaises(ValueError): adapter.prepare(*args)

    def test_refresh_must_be_native_user_message(self):
        args = fixture(); source = args[0]['runs'][0]['records'][0]['input']['native_source']
        payload = args[3][source][0][1]['payload']; payload['role'] = 'assistant'
        args[5][0]['payload_sha256'] = digest(payload)
        with self.assertRaises(ValueError): adapter.prepare(*args)

    def test_explicit_turn_conflict_and_boolean_rejected(self):
        for turn in (True, 'foreign-turn'):
            args = fixture(); source = args[0]['runs'][0]['records'][0]['input']['native_source']
            args[3][source][0][1]['payload']['turn_id'] = turn
            with self.subTest(turn=turn), self.assertRaises(ValueError): adapter.prepare(*args)

    def test_failed_usage_less_accepted_turn_is_known(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES)); proof = args[1][run]
        first = proof['delivery']['inputs'][0]; inp = copy.deepcopy(first)
        inp.update(turn_id='failed-no-usage', native_item_id='failed-input', native_line=900,
                   public_line=900, input_sha256=hashlib.sha256(b'failed').hexdigest(), input_bytes=6)
        proof['delivery']['inputs'].append(inp)
        proof['native_membership']['contexts'].append({'thread_id': inp['thread_id'], 'turn_id': inp['turn_id']})
        args[3][inp['native_source']].append((900, {'type': 'response_item', 'payload': {
            'id': inp['native_item_id'], 'type': 'message', 'role': 'user', 'turn_id': inp['turn_id'],
            'content': [{'type': 'input_text', 'text': 'failed'}]}}))
        self.assertEqual(len(adapter.prepare(*args)), 3)

    def test_lower_map_digest_count_and_status_required(self):
        for kind in ('hash', 'count', 'status'):
            args = fixture(); row = args[2]['candidates'][0]
            if kind == 'hash': row['response_evidence_sha256'] = 'f' * 64
            elif kind == 'count': row['response_count'] = True
            else: row['status'] = 'unqualified'
            with self.subTest(kind=kind), self.assertRaises(ValueError): adapter.prepare(*args)

    def test_source_subproof_failure_rejects(self):
        args = fixture(); args[1][next(iter(adapter.RUN_LINES))]['native_membership']['errors'] = ['missing']
        with self.assertRaises(ValueError): adapter.prepare(*args)

    def test_missing_attribution_retains_every_target_unknown(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES))
        for row in args[4][run]: row['params'].pop('usageMetadata')
        result = adapter.extract(adapter.prepare(*args)[0])
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(len(result['evidence']['items']), 2)
        self.assertTrue(all(x['coverage'] == 'not_observed_in_completed_response_attribution'
                            for x in result['evidence']['items']))

    def test_identical_and_conflicting_duplicate_metadata_retained(self):
        for conflict in (False, True):
            args = fixture(); run = next(iter(adapter.RUN_LINES)); rows = args[4][run]
            duplicate = copy.deepcopy(rows[0]); duplicate['_audit_line'] = 999
            if conflict: duplicate['params']['usageMetadata']['metadata']['attribution']['items'] = {}
            rows.append(duplicate)
            result = adapter.extract(adapter.prepare(*args)[0])
            with self.subTest(conflict=conflict):
                self.assertEqual(result['status'], 'unknown' if conflict else 'exact_observed_attribution')
                self.assertEqual(result['evidence']['duplicate_responses'], 0 if conflict else 1)

    def test_foreign_response_id_is_unknown_not_dropped(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES))
        args[4][run][0]['params']['responseId'] = 'foreign-response'
        result = adapter.extract(adapter.prepare(*args)[0])
        self.assertEqual(result['status'], 'unknown')
        self.assertIn('foreign-response', result['raw_without_qualified_native'])

    def test_no_hit_response_stays_in_observed_window(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES))
        for row in args[4][run]: row['params']['usageMetadata']['metadata']['attribution']['items'].clear()
        prepared = adapter.prepare(*args)[0]
        self.assertEqual(len(prepared['servers']), 4)
        self.assertEqual(len(adapter.extract(prepared)['evidence']['responses']), 4)

    def test_unidentified_response_is_retained(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES))
        args[4][run][0]['params'].pop('responseId')
        result = adapter.extract(adapter.prepare(*args)[0])
        self.assertEqual(len(result['evidence']['unidentified_response_records']), 1)
        self.assertEqual(result['status'], 'unknown')

    def test_wrong_stream_or_reversed_physical_rows_rejected(self):
        for mutation in ('stream', 'order'):
            args = fixture(); rows = args[4][next(iter(adapter.RUN_LINES))]
            if mutation == 'stream': rows[0]['_audit_source'] = '/foreign/server-to-client.raw'
            else: rows.reverse()
            with self.subTest(mutation=mutation), self.assertRaises(ValueError): adapter.prepare(*args)

    def test_native_response_missing_raw_retains_coverage(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES)); removed = args[4][run].pop()
        result = adapter.extract(adapter.prepare(*args)[0])
        self.assertIn(removed['params']['responseId'], result['evidence']['native_response_ids_without_raw'])

    def test_limits_fail_without_clipping(self):
        prepared = adapter.prepare(*fixture())[0]
        for limits in ({'max_rows': 1}, {'max_bytes': 1}):
            result = adapter.extract(prepared, **limits)
            self.assertEqual(result['status'], 'unknown')
            self.assertIsNone(result['evidence'])
            self.assertEqual(len(result['full_attribution_sha256']), 64)
            self.assertGreater(result['full_attribution_bytes'], 1)
            self.assertEqual(result['extract_records_calls'], 1)

    def test_known_response_wrong_thread_cannot_leave_the_window_silently(self):
        args = fixture(); run = next(iter(adapter.RUN_LINES))
        args[4][run][0]['params']['threadId'] = 'foreign-thread'
        result = adapter.extract(adapter.prepare(*args)[0])
        self.assertEqual(len(result['evidence']['responses']), 4)
        self.assertEqual(result['status'], 'unknown')

    def test_charge_before_its_exact_target_delivery_is_unknown(self):
        args = fixture(); run = list(adapter.RUN_LINES)[1]
        first, later = args[0]['runs'][1]['records'][0], args[0]['runs'][1]['records'][4]
        items = args[4][run][0]['params']['usageMetadata']['metadata']['attribution']['items']
        counts = items.pop(first['input']['native_item_id'])
        items[later['input']['native_item_id']] = counts
        result = adapter.extract(adapter.prepare(*args)[1])
        self.assertEqual(result['status'], 'unknown')
        target = next(x for x in result['evidence']['items'] if x['item_id'] == later['input']['native_item_id'])
        self.assertFalse(target['charges'][0]['response_exact'])
        self.assertIn('target-charge-before-delivery', result['errors'])


if __name__ == '__main__':
    unittest.main()
