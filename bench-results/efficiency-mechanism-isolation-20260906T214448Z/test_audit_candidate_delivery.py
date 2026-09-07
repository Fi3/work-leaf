"""Synthetic v4 delivery fixtures only; no provider or outcome analysis."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import audit_candidate_delivery as subject

RUN = 'synthetic-run'
CONDITION = 'requested-repeat-full'


def event(site, baseline, candidate, components, metadata, agent='a', condition=CONDITION):
    chosen = (site == 'candidate-policy' or
              condition == {'requested-repeat-read': CONDITION,
                            'review-fix-request': 'review-fix-request-resupply'}.get(site))
    selected = candidate if chosen else baseline
    return {'schema': subject.SCHEMA, 'event': 'candidate-prompt', 'run_id': RUN,
            'condition': condition, 'process_id': 7, 'sequence': 1, 'unix_time_ns': '123',
            'agent_id': agent, 'site': site, 'original_prompt': baseline,
            'candidate_prompt': candidate, 'components': components, 'metadata': metadata,
            'selected_candidate': 'candidate' if chosen else 'baseline',
            'original_bytes': len(baseline.encode()), 'candidate_bytes': len(candidate.encode()),
            'selected_bytes': len(selected.encode()), 'changed': selected != baseline,
            'byte_delta': len(selected.encode())-len(baseline.encode())}


def component(bs, be, cs, ce):
    return dict(baseline_start=bs, baseline_end=be, inline_start=cs, inline_end=ce)


def policy(agent='a', condition=CONDITION):
    baseline = 'You are running under the work-leaf orchestrator.\n'
    candidate = baseline; spans = []; components = []
    for name in ('policy-write-format', 'policy-edit-request', 'policy-edit-preference', 'policy-manual-write-format'):
        old, new = name+' original', name+' replacement'
        bs, cs = len(baseline.encode()), len(candidate.encode())
        baseline += old
        candidate += new if condition == 'unified-diff-preferred' else old
        components.append(component(bs, len(baseline.encode()), cs, len(candidate.encode())))
        spans.append({'id': name, 'condition': 'unified-diff-preferred', 'original': old, 'replacement': new})
        baseline += '\n'; candidate += '\n'
    footer = f'\n\nAgent-ID: {agent}\nFeature: fixture\n\nUser prompt:\nLiteral request λ'
    return event('candidate-policy', baseline+footer, candidate+footer, components, {'spans': spans}, agent, condition)


def read(agent='a', condition=CONDITION):
    prefix = 'work-leaf file text\n'
    old = '\nRepeated file reads unchanged\nWork Leaf already sent this agent the exact text for these files, and the current digests still match. Full text is not resent; use the existing snapshot.\n'
    body = 'λ\n--- adversarial ---'
    digest = subject.fnv(body.encode())
    old += '- source.rs ('+digest+')\n'
    opening = '\nRepeated file reads: current full text\n\n--- source.rs ---\n'
    new = opening+body+'\n'
    start = len((prefix+opening).encode()); end = start+len(body.encode())
    return event('requested-repeat-read', prefix+old, prefix+new,
                 [component(len(prefix.encode()), len((prefix+old).encode()), len(prefix.encode()), len((prefix+new).encode()))],
                 {'eligible': True, 'requested_paths': ['source.rs'],
                  'bundle': {'threshold_eligible': False, 'write_succeeded': False, 'path': None},
                  'snapshots': [{'path': 'source.rs', 'class': 'unchanged', 'bytes': len(body.encode()),
                                 'digest': digest, 'inline_body_start': start, 'inline_body_end': end}],
                  'failures': []}, agent, condition)


def resupply(request='Literal request λ'):
    old = 'The reviewer found issues in your patch for commit abc.\nFINDINGS: fix behavior'
    new = old+subject.RESUPPLY+request
    return event('review-fix-request', old, new,
        [component(len(old.encode()), len(old.encode()), len(old.encode()), len(new.encode()))],
        {'original_request_source': 'prepared-agent-launch.prompt', 'source_agent_id': 'a',
         'source_feature': 'fixture', 'original_request_bytes': len(request.encode()),
         'candidate_request_start': len((old+subject.RESUPPLY).encode()),
         'candidate_request_end': len(new.encode())}, condition='review-fix-request-resupply')


def extra_turn(cap, text, thread='thread-a', number=100):
    frame = {'id': str(number), 'method': 'turn/start', 'params': {'threadId': thread, 'input': [{'type': 'text', 'text': text}]}}
    cap['clients'].append(frame); cap['forwarded'].append(copy.deepcopy(frame))
    cap['servers'] += [{'id': str(number), 'result': {'turn': {'id': 'turn-'+str(number)}}},
        {'method': 'item/completed', 'params': {'threadId': thread, 'turnId': 'turn-'+str(number),
         'item': {'type': 'userMessage', 'id': 'user-'+str(number), 'content': [{'type': 'text', 'text': text}]}}}]


def fixture(events=None, condition=CONDITION):
    events = copy.deepcopy(events if events is not None else [policy(condition=condition), read(condition=condition)])
    trace = [{'schema': subject.SCHEMA, 'event': 'activation', 'run_id': RUN,
              'condition': condition, 'process_id': 7}]
    clients, servers = [], []
    for index, item in enumerate(events, 1):
        item['sequence'] = index; trace.append(item)
        selected = item.get('forwarded_prompt', item.get('candidate_prompt') if item.get('selected_candidate') == 'candidate' else item.get('original_prompt'))
        thread, turn = 'thread-'+item['agent_id'], f'turn-{index}'
        clients.append({'id': str(index), 'method': 'turn/start', 'params': {'threadId': thread, 'input': [{'type': 'text', 'text': selected}]}})
        servers.append({'id': str(index), 'result': {'turn': {'id': turn}}})
        servers.append({'method': 'item/completed', 'params': {'threadId': thread, 'turnId': turn,
            'item': {'type': 'userMessage', 'id': f'user-{index}', 'content': [{'type': 'text', 'text': selected}]}}})
    return trace, [{'path': 'capture', 'clients': clients, 'forwarded': copy.deepcopy(clients), 'servers': servers}]


class DeliveryTests(unittest.TestCase):
    def audit(self, trace, captures, condition=CONDITION):
        return subject.audit_delivery(trace, captures, RUN, condition)

    def test_read_exact_body_and_public_identity(self):
        result = self.audit(*fixture())
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['delivered']['eligible_repeat_reads'], 1)
        self.assertEqual(result['exposures'][1]['native_user_item_id'], 'user-2')
        self.assertNotIn('λ\n--- adversarial ---', str(result))

    def test_identical_prompts_are_consumed_in_thread_order(self):
        result = self.audit(*fixture([policy(), read(), read()]))
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual([row['rpc_id'] for row in result['exposures']], ['1', '2', '3'])
        self.assertEqual(result['delivered']['eligible_repeat_reads'], 2)

    def test_identical_prompts_across_agents_use_owned_thread_identity(self):
        result = self.audit(*fixture([policy('a'), policy('b'), read('a'), read('b')]))
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual([row['thread_id'] for row in result['exposures'][-2:]], ['thread-a', 'thread-b'])

    def test_missing_extra_duplicate_or_unforwarded_request_is_not_available(self):
        for mutation in ('missing', 'extra', 'duplicate', 'unforwarded'):
            trace, captures = fixture(); cap = captures[0]
            if mutation == 'missing': trace.pop()
            elif mutation == 'extra': trace.append(copy.deepcopy(trace[-1])); trace[-1]['sequence'] = 3
            elif mutation == 'duplicate': cap['servers'].append(copy.deepcopy(cap['servers'][0]))
            else: cap['forwarded'].pop()
            self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable', mutation)

    def test_malformed_metadata_wrong_condition_and_nonfactor_mutation(self):
        for mutation in ('body', 'class', 'eligible', 'condition', 'selection', 'outside'):
            trace, captures = fixture()
            row = trace[-1]
            if mutation == 'body': row['metadata']['snapshots'][0]['inline_body_end'] = True
            elif mutation == 'class': row['metadata']['snapshots'][0]['class'] = 'unknown'
            elif mutation == 'eligible': row['metadata']['eligible'] = 1
            elif mutation == 'condition': row['condition'] = 'unified-diff-preferred'
            elif mutation == 'selection': row['selected_candidate'] = 'baseline'
            else: row['candidate_prompt'] += 'unowned suffix'
            self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable', mutation)
        trace, captures = fixture(condition='unified-diff-preferred')
        trace[-1]['selected_candidate'] = 'candidate'
        self.assertEqual(self.audit(trace, captures, 'unified-diff-preferred')['status'], 'unverifiable')

    def test_policy_spans_cannot_be_in_copied_user_text(self):
        trace, captures = fixture(); row = trace[1]
        text = row['original_prompt']; start = len(text.encode())
        row['original_prompt'] += '\npolicy-write-format original'
        row['components'][0] = component(start+1, len(row['original_prompt'].encode()), start+1, len(row['original_prompt'].encode()))
        self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable')

    def test_duplicate_turn_or_user_item_is_not_reused(self):
        for kind in ('turn', 'user', 'text'):
            trace, captures = fixture(); server = captures[0]['servers']
            if kind == 'turn': server[2]['result']['turn']['id'] = 'turn-1'
            elif kind == 'user': server[3]['params']['item']['id'] = 'user-1'
            else: server[3]['params']['item']['content'][0]['text'] = 'wrong'
            self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable', kind)

    def test_literal_private_reasoning_never_exported_and_formats_are_public_only(self):
        trace, captures = fixture(); server = captures[0]['servers']
        server += [
            {'method': 'item/completed', 'params': {'threadId': 'thread-a', 'turnId': 'turn-2',
                'item': {'type': 'reasoning', 'id': 'private', 'text': 'PRIVATE BODY @work-leaf patch x'}}},
            {'method': 'item/completed', 'params': {'threadId': 'thread-a', 'turnId': 'turn-2',
                'item': {'type': 'agentMessage', 'id': 'public', 'text': '@work-leaf patch reason\nbody\n@work-leaf end'}}},
        ]
        result = self.audit(trace, captures)
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['public_formats']['patch'], 1)
        self.assertEqual(result['public_formats']['edit'], 0)
        self.assertNotIn('PRIVATE BODY', str(result))

    def test_resupply_counts_and_latest_owned_launch_provenance(self):
        condition = 'review-fix-request-resupply'
        first = policy(condition=condition)
        result = self.audit(*fixture([first, resupply()], condition), condition)
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['delivered']['original_request_resupplies'], 1)
        second = copy.deepcopy(first)
        for key in ('original_prompt', 'candidate_prompt'):
            second[key] = second[key].replace('Literal request λ', 'Another request λ')
        second.update(original_bytes=len(second['original_prompt'].encode()), candidate_bytes=len(second['candidate_prompt'].encode()), selected_bytes=len(second['candidate_prompt'].encode()))
        result = self.audit(*fixture([first, second, resupply()], condition), condition)
        self.assertEqual(result['status'], 'unverifiable')

    def test_reviewer_copied_metadata_does_not_define_owner(self):
        row = policy('review-a')
        tail = '\n\nGit metadata:\n\nAgent-ID: a\nFeature: literal\n\nUser prompt:\ncopied text'
        row['original_prompt'] += tail; row['candidate_prompt'] += tail
        for key in ('original_bytes', 'candidate_bytes', 'selected_bytes'):
            row[key] += len(tail.encode())
        result = self.audit(*fixture([row]))
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['exposures'][0]['agent_id'], 'review-a')

    def test_unique_followups_cannot_reorder_and_uninstrumented_body_must_match(self):
        other = read()
        for key in ('original_prompt', 'candidate_prompt'): other[key] = other[key].replace('source.rs', 'second.rs')
        other['metadata']['requested_paths'] = ['second.rs']; other['metadata']['snapshots'][0]['path'] = 'second.rs'
        trace, captures = fixture([policy(), read(), other]); cap = captures[0]
        cap['clients'][1:3] = reversed(cap['clients'][1:3]); cap['forwarded'][1:3] = reversed(cap['forwarded'][1:3])
        self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable')
        trace, captures = fixture(); extra_turn(captures[0], 'Ordinary reviewer recheck')
        captures[0]['servers'][-1]['params']['item']['content'][0]['text'] = 'different'
        self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable')

    def test_title_thread_and_linearizer_are_retained(self):
        text = subject.LINEARIZER+'Normal integration instructions\n\nAgent-ID: linearize\nFeature: integration\n\nUser prompt:\nIntegrate work'
        linearizer = event('candidate-policy', text, text, [], {'spans': []}, 'linearize')
        trace, captures = fixture([policy(), read(), linearizer])
        title = ('Name this Work Leaf chat from the user\'s first prompt.\nRules:\n'
                 '- Return only the chat name, no prose and no quotes.\n- Derive the name from the first prompt only.\n'
                 '- Use at most 40 characters.\n- Use lowercase words separated by hyphens.\n- Do not use spaces.\n\nFirst prompt:\n')
        for index in range(3): extra_turn(captures[0], title+f'feature {index}', 'thread-title', 100+index)
        result = self.audit(trace, captures)
        self.assertEqual(result['status'], 'available', result['errors'])
        self.assertEqual(result['auxiliary_title_turns'], 3)
        captures[0]['clients'][-1]['params']['input'][0]['text'] = 'arbitrary hidden role'
        captures[0]['forwarded'][-1] = copy.deepcopy(captures[0]['clients'][-1])
        self.assertEqual(self.audit(trace, captures)['status'], 'unverifiable')

    def test_public_message_hash_is_computed_once_for_many_blocks(self):
        text = ('@work-leaf patch reason\nbody\n@work-leaf end\n')*20
        cap = {'path': 'capture', 'servers': [{'method': 'item/completed', 'params': {'threadId': 'thread', 'turnId': 'turn',
            'item': {'type': 'agentMessage', 'id': 'item', 'text': text}}}]}
        with patch.object(subject, 'digest', wraps=subject.digest) as hashed:
            _, actions, errors = subject.public_items([cap])
        self.assertFalse(errors); self.assertEqual(len(actions), 20)
        self.assertEqual(hashed.call_count, 1)


class SourceTests(unittest.TestCase):
    def manifest(self, root):
        def save(path, value, lines=False):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(''.join(json.dumps(v)+'\n' for v in value) if lines else json.dumps(value))
            return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
        trace, caps = fixture(); cap = caps[0]; obs = root/'observation'; raw = obs/'app-server'/'inv'
        refs = {key: save(raw/name, cap[key], True) for key, name in (
            ('clients', 'client-to-server.raw'), ('forwarded', 'client-to-server.forwarded.raw'), ('servers', 'server-to-client.raw'))}
        start = {'invocation_id': 'inv', 'capture_kind': 'app-server', 'primary': True, 'start_unix_ns': 1}
        refs['start'] = save(obs/'invocations/inv/start.json', start)
        end = {'invocation_id': 'inv', 'exit_code': 0, 'end_unix_ns': 2,
               'stdin_sha256': refs['clients']['sha256'], 'stdout_sha256': refs['servers']['sha256'],
               'raw_response_usage_start_sha256': refs['start']['sha256'],
               'raw_response_usage_sha256': {'client-to-server.forwarded.raw': refs['forwarded']['sha256']}}
        refs['end'] = save(obs/'invocations/inv/end.json', end)
        runtime = {name: {'path': str(subject.HERE.parents[1]/name), 'sha256': expected} for name, expected in subject.RUNTIME_PINS.items()}
        terminal = {'run_id': RUN, 'id': RUN, 'condition': CONDITION, 'launch_status': 'completed',
                    'started_at': '2026-09-07T00:00:00+00:00', 'finished_at': '2026-09-07T00:01:00+00:00', 'launcher_exit_code': 1}
        return {'schema': 'work-leaf-candidate-delivery-input-v1', 'run_id': RUN, 'condition': CONDITION,
                'helper_sha256': hashlib.sha256(Path(subject.__file__).read_bytes()).hexdigest(), 'runtime_sources': runtime,
                'trace': save(root/'trace.jsonl', trace, True), 'terminal': save(root/'exit.json', terminal),
                'invocations': save(obs/'process-invocations.jsonl', [{**start, 'end': end}], True),
                'captures': [{'path': str(raw), **refs}]}

    def test_complete_closed_source_branch_retains_failure_and_create_new(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); manifest = self.manifest(root)
            result = subject.audit_sources(manifest)
            self.assertEqual(result['status'], 'available', result['errors'])
            self.assertEqual(result['terminal']['exit_code'], 1)
            self.assertGreater(len(result['source_sha256']), 10)
            output = root/'report.json'; subject.write_new(output, result)
            with self.assertRaises(FileExistsError): subject.write_new(output, result)

    def test_source_hashes_closure_and_capture_census_fail_closed(self):
        for kind in ('null-hash', 'bad-hash', 'runtime', 'live', 'missing-capture', 'changed-stream', 'symlink'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp); manifest = self.manifest(root)
                if kind == 'null-hash': manifest['trace']['sha256'] = None
                elif kind == 'bad-hash': manifest['trace']['sha256'] = 'F'*64
                elif kind == 'runtime': next(iter(manifest['runtime_sources'].values()))['sha256'] = '0'*64
                elif kind == 'missing-capture': manifest['captures'] = []
                elif kind == 'changed-stream': Path(manifest['captures'][0]['clients']['path']).write_text('{}\n')
                elif kind == 'live':
                    path = Path(manifest['terminal']['path']); value = json.loads(path.read_text()); value['launch_status'] = 'running'
                    path.write_text(json.dumps(value)); manifest['terminal']['sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
                else:
                    link = root/'linked.jsonl'; link.symlink_to(manifest['trace']['path']); manifest['trace']['path'] = str(link)
                with self.assertRaises((ValueError, OSError)): subject.audit_sources(manifest)


if __name__ == '__main__':
    unittest.main()
