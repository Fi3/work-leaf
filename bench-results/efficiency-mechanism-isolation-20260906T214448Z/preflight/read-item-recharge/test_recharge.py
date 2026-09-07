import importlib.util
import json
import copy
from pathlib import Path
import unittest

P = Path(__file__).with_name('recharge.py')
spec = importlib.util.spec_from_file_location('read_item_recharge', P)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


def native(item='read', turn='turn', kind='message', role='user'):
    return {'type': 'response_item', 'payload': {
        'type': kind, 'id': item, 'role': role,
        'internal_chat_message_metadata_passthrough': {'turn_id': turn},
        'content': [{'type': 'input_text', 'text': 'PRIVATE_BODY_NOT_EXPORTED'}]}}


def sources():
    user = {'thread_id': 'thread', 'turn_id': 'turn', 'item_id': 'read',
            'source': '/native', 'line': 2, 'payload_sha256': 'a' * 64,
            'kind': 'message', 'role': 'user'}
    return ({'reads': [{'native_user': user, 'thread_id': 'thread', 'turn_id': 'turn',
                       'trace_line': 1, 'sequence': 4, 'selected_candidate': 'baseline',
                       'selected_sha256': 'b' * 64, 'eligible': True,
                       'bundle':{'threshold_eligible':True,'write_succeeded':True,'path':'/bundle'},
                       'snapshot_metadata':[{'class':'untracked'}]}],
             'accepted_inputs': [{'native_user': user}]},
            {'candidates': [{'thread_id': 'thread', 'turn_id': 'turn',
                             'native_output_item_id': 'out', 'native_output_line': 9,
                             'native_source': '/native', 'native_output_payload_sha256': 'c' * 64,
                             'candidate_index': 0, 'call_id': 'call',
                             'classification': 'selected_content_delivered',
                             'references': [{'read_index': 0, 'path': '/A'},
                                            {'read_index': 1, 'path': '/B'}],
                             'witnesses': [{'bundle_path': '/A'}],
                             'complete_bundle_paths': []}]})


def attribution():
    return {'status': 'exact_observed_attribution', 'errors': [],
            'unidentified_response_records': [], 'native_response_ids_without_raw': [],
            'duplicate_responses': 0, 'repeated_input_items': [],
            'responses': [
                {'response_id': 'r1', 'thread_id': 'thread', 'turn_id': 'turn',
                 'sequence': 0, 'source': '/raw', 'line': 1,
                 'status': 'exact_observed_attribution', 'errors': [], 'residual': {'input_tokens': 0},
                 'request_fields': {}, 'input_items': [
                     {'item_id': 'out', 'input_tokens': 11, 'cached_input_tokens': 0,
                      'cache_write_input_tokens': 0, 'output_tokens': 0}], 'output_items': []},
                {'response_id': 'r2', 'thread_id': 'thread', 'turn_id': 'later',
                 'sequence': 1, 'source': '/raw', 'line': 2,
                 'status': 'unknown', 'errors': ['residual'], 'residual': {'input_tokens': -1},
                 'request_fields': {}, 'input_items': [
                     {'item_id': 'out', 'input_tokens': 13, 'cached_input_tokens': 7,
                      'cache_write_input_tokens': 0, 'output_tokens': 0}], 'output_items': []}]}


class RechargeTests(unittest.TestCase):
    def test_delivery_class_binds_bundle_and_eligibility_not_candidate_tag(self):
        a,b=sources();row=a['reads'][0]
        self.assertEqual(m.targets(a,b)[('thread','read')]['associations'][0]['delivered_class'],'eligible_bundle_manifest')
        row['eligible']=False;row['bundle']['write_succeeded']=False
        self.assertEqual(m.targets(a,b)[('thread','read')]['associations'][0]['delivered_class'],'other_ineligible_delivery')

    def test_target_locators_must_match_exact_native_inventory(self):
        a,b=sources(); target=m.targets(a,b)
        items={key:{**row['native'],'kind':'message' if key[1]=='read' else 'function_call_output',
                    'role':'user' if key[1]=='read' else 'tool'} for key,row in target.items()}
        m.validate_targets(target,items)
        items[('thread','out')]['payload_sha256']='d'*64
        with self.assertRaises(ValueError): m.validate_targets(target,items)
        del items[('thread','out')]
        with self.assertRaises(ValueError): m.validate_targets(target,items)

    def test_actual_pinned_extractor_duplicate_and_partial_metadata(self):
        extractor=m.load_extractor(P.parents[2]/'audit_input_attribution.py')
        def counts(i=0,c=0,o=0):return {'input_tokens':i,'cached_tokens':c,'output_tokens':o,'cache_write_tokens':0}
        row={'method':'rawResponse/completed','params':{
            'threadId':'thread','turnId':'turn','responseId':'r1',
            'usage':{'inputTokens':15,'cachedInputTokens':10,'outputTokens':5,
                     'reasoningOutputTokens':3,'cacheWriteInputTokens':0,'totalTokens':20},
            'usageMetadata':{'metadata':{'attribution':{'items':{
                'read':{**counts(10,10),'content':[counts(10,10)]},
                'reason':counts(o=3),'action':counts(o=2)},'request_fields':{'tools':counts(5)}}}}}}
        native_items={('thread','read'):{'kind':'message','role':'user','turn_id':'turn'},
                      ('thread','reason'):{'kind':'reasoning','turn_id':'turn'},
                      ('thread','action'):{'kind':'message','role':'assistant','turn_id':'turn'}}
        ledger={'r1':{'thread_id':'thread','turn_id':'turn','usage':{
            'input_tokens':15,'cached_input_tokens':10,'output_tokens':5,'reasoning_output_tokens':3}}}
        out=extractor([row,copy.deepcopy(row)],native_items,ledger)
        self.assertEqual(out['status'],'exact_observed_attribution')
        self.assertEqual(out['duplicate_responses'],1)
        self.assertEqual(out['responses'][0]['input_items'][0]['input_tokens'],10)
        broken=copy.deepcopy(row);del broken['params']['usageMetadata']['metadata']['attribution']['request_fields']
        self.assertEqual(extractor([broken],native_items,ledger)['status'],'unknown')
        conflicting=copy.deepcopy(row);conflicting['params']['usage']['inputTokens']=16
        self.assertEqual(extractor([row,conflicting],native_items,ledger)['status'],'unknown')

    def test_explicit_turn_precedes_context_without_fallback(self):
        rows = [(1, native()), (2, {'type': 'turn_context', 'payload': {'turn_id': 'other'}})]
        items = m.native_inventory('thread', '/native', rows, {'turn', 'other'})
        self.assertEqual(items[('thread', 'read')]['turn_id'], 'turn')
        self.assertNotIn('PRIVATE_BODY', json.dumps(list(items.values())))

    def test_missing_bool_or_disagreeing_explicit_turn_rejected(self):
        for turn in [None, True, 1, 'unknown']:
            with self.subTest(turn=turn), self.assertRaises(ValueError):
                m.native_inventory('thread', '/native', [(1, native(turn=turn))], {'turn'})
        row = native(); row['payload']['turn_id'] = 'other'
        with self.assertRaises(ValueError):
            m.native_inventory('thread', '/native', [(1, row)], {'turn', 'other'})

    def test_conflicting_item_rejected_exact_duplicate_retained_once(self):
        rows = [(1, native()), (2, native())]
        self.assertEqual(len(m.native_inventory('thread', '/native', rows, {'turn'})), 1)
        rows[1][1]['payload']['role'] = 'assistant'
        with self.assertRaises(ValueError): m.native_inventory('thread', '/native', rows, {'turn'})

    def test_all_read_and_output_items_retained_once_not_per_bundle(self):
        a,b=sources(); targets=m.targets(a,b)
        self.assertEqual(len(targets),2)
        out=targets[('thread','out')]
        self.assertEqual(len(out['associations']),1)
        self.assertEqual(out['associations'][0]['witnessed_bundle_paths'],['/A'])
        self.assertEqual(out['associations'][0]['referenced_bundle_paths'],['/A','/B'])
        self.assertNotIn('content',out)

    def test_same_output_multiple_candidates_dedup_but_retain_associations(self):
        a,b=sources();b['candidates'].append({**b['candidates'][0],'candidate_index':1})
        out=m.targets(a,b)[('thread','out')]
        self.assertEqual(len(out['associations']),2)

    def test_failed_candidate_optional_complete_paths_remains_failed(self):
        a,b=sources(); row=b['candidates'][0]
        row['classification']='failed_before_content_delivery';row['witnesses']=[]
        del row['complete_bundle_paths']
        out=m.targets(a,b)[('thread','out')]['associations'][0]
        self.assertEqual(out['classification'],'failed_before_content_delivery')
        self.assertEqual(out['complete_bundle_paths'],[])

    def test_unlinked_read_reference_retains_candidate_without_guess(self):
        a,b=sources();b['candidates'][0]['references'][1]['read_index']=None
        out=m.targets(a,b)[('thread','out')]['associations'][0]
        self.assertEqual(out['read_indices'],[0])
        self.assertEqual(out['unlinked_read_references'],1)

    def test_read_accepted_identity_required_and_bool_rejected(self):
        a,b=sources();a['accepted_inputs']=[]
        with self.assertRaises(ValueError):m.targets(a,b)
        a,b=sources();b['candidates'][0]['native_output_item_id']=True
        with self.assertRaises(ValueError):m.targets(a,b)

    def test_first_subsequent_whole_item_charges_unknown_retained(self):
        a,b=sources();out=m.project(attribution(),m.targets(a,b))
        items={r['item_id']:r for r in out['items']}
        self.assertEqual([r['charge_position'] for r in items['out']['charges']],[1,2])
        self.assertEqual([r['input_tokens'] for r in items['out']['charges']],[11,13])
        self.assertFalse(items['out']['all_charge_responses_exact'])
        self.assertEqual(items['read']['coverage'],'not_observed_in_completed_response_attribution')
        self.assertEqual(out['responses'][1]['residual']['input_tokens'],-1)
        self.assertNotIn('whole_workflow_total',out)

    def test_wrong_thread_or_prefix_never_joined(self):
        a,b=sources();r=attribution();r['responses'][0]['thread_id']='other';r['responses'][1]['input_items'][0]['item_id']='out-prefix'
        out=m.project(r,m.targets(a,b))
        self.assertTrue(all(not x['charges'] for x in out['items']))

    def test_output_charge_does_not_become_input_recharge(self):
        a,b=sources();r=attribution();r['responses'][0]['input_items']=[]
        r['responses'][0]['output_items']=[{'item_id':'out','output_tokens':20}]
        out=m.project(r,m.targets(a,b));item=next(x for x in out['items'] if x['item_id']=='out')
        self.assertEqual(len(item['charges']),1)

    def test_row_and_byte_ceiling_fail_not_clip(self):
        a,b=sources()
        with self.assertRaises(ValueError):m.project(attribution(),m.targets(a,b),max_rows=1)
        with self.assertRaises(ValueError):m.project(attribution(),m.targets(a,b),max_bytes=20)

    def test_projection_no_unrecognized_bodies_or_arguments(self):
        a,b=sources();r=attribution();r['responses'][0]['secret']='PRIVATE_BODY';r['responses'][0]['input_items'][0]['arguments']='PRIVATE_ARGS'
        result=m.project(r,m.targets(a,b))
        self.assertNotIn('PRIVATE',json.dumps(result))
        self.assertEqual(len(result['full_attribution_sha256']),64)


if __name__ == '__main__': unittest.main()
