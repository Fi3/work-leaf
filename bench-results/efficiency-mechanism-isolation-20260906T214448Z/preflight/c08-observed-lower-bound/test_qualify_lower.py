"""Synthetic records only; no saved workflow, auditor or provider execution."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock

try:
    import qualify_lower as q
except ModuleNotFoundError:
    q = None

S = Path(__file__).resolve().parents[2]
ACCOUNTING = S / 'phases/automatic-refresh-01/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z/accounting_untracked_reads.py'
STRICT = ACCOUNTING.parent.parent / 'efficiency-measurement-gate-20260906/batch_analysis.py'
RUNS = [f'automatic-refresh-01-workflow-{n:03}' for n in (1, 2, 3)]
BASES = [f'work-units-01-workflow-{n:03}' for n in (2, 4, 6, 9, 10, 12)]


def canonical(v, unicode=False):
    return json.dumps(v, sort_keys=True, separators=(',', ':'), ensure_ascii=not unicode, allow_nan=False).encode()


def sha(v, unicode=False):
    return hashlib.sha256(canonical(v, unicode)).hexdigest()


def usage(n=3):
    return dict(input_tokens=n, cached_input_tokens=1, output_tokens=2,
                reasoning_output_tokens=1, uncached_input_tokens=n-1,
                raw_input_plus_output=n+2, uncached_input_plus_output=n+1)


def wire(u):
    return dict(zip(('inputTokens', 'cachedInputTokens', 'outputTokens', 'reasoningOutputTokens'),
                    (u['input_tokens'], u['cached_input_tokens'], u['output_tokens'], u['reasoning_output_tokens'])),
                totalTokens=u['raw_input_plus_output'])


def projection(original):
    return {'original_entry': {'id': original['run_id'], 'exit_code': 1}, 'accounting': {
        'exact_helper_result_sha256': sha(original), 'exact_response_evidence': {
            'count': len(original['exact_response_evidence']), 'sha256': sha(original['exact_response_evidence'])}}}


def original(run, rid):
    record = {'thread_id': run+'-thread', 'turn_id': 'turn', 'usage': usage(),
              'sources': [{'path': '/native/'+run, 'native_line': 3}, {'path': '/capture/'+run+'/server', 'server_line': 1}]}
    return dict(schema='work-leaf-read-identity-accounting-v1', run_id=run, status='unknown',
                errors=['retained original failure'], warnings=[], source_sha256={},
                exact_response_evidence={rid: record}, corrected_scope={'usage': usage()},
                measurement={'status': 'ineligible', 'bounds': None, 'recorded_usage': usage(),
                             'gap_inventory': {'errors': [], 'gaps': [{'response_count_upper': None}], 'raw_responses': {rid: {k:v for k,v in record.items() if k!='sources'}}}},
                whole_workflow_hidden_call_completeness_proven=False)


def fixture():
    runs=[]
    for i, run in enumerate(RUNS):
        tid=run+'-thread'; path='/native/'+run; capture='/capture/'+run
        full=original(run, 'response-'+str(i))
        source={'run_id':run,'errors':['trace-occurrence-not-joined']*2,
                'native_membership':{'errors':[],'threads':[{'thread_id':tid,'source':path,'accepted_turn_count':1}],
                                     'contexts':[{'thread_id':tid,'turn_id':'turn'}]},
                'delivery':{'inputs':[{'thread_id':tid,'turn_id':'turn','status':'joined','native_source':path,
                                      'capture_id':run,'capture_index':0}]},
                'project_inventory':{'errors':[]}, 'invocation_streams':[{'errors':[]}],
                'frame_proofs':[{'capture_id':run,'turn_closure':{'errors':[]}}]}
        item={'run_id':run,'expected_response_count':1,'original':full,'projection':projection(full),
              'source_result':source,'source_input':{'project_cwd':'/project','captures':[{'path':capture,'servers':{'path':capture+'/server'}}],
                                                   'native_sessions':[{'thread_id':tid,'source':{'path':path}}]},
              'observer_errors':['original observer gap'], 'original_outcome':{'exit_code':1 if i==0 else 0}}
        runs.append(item)
    item=runs[2]; tid=RUNS[2]+'-thread'; rid='response-2'; path='/native/'+RUNS[2]
    payload={'response_id':rid,'thread_id':tid,'session_id':tid,'turn_id':'turn',
             'usage':{**usage(),'total_tokens':5}}
    metadata={'thread_id':tid,'model':'gpt-5.5','effort':'xhigh','cwd':'/project'}
    rows=[{'type':'session_meta','payload':{'id':tid,'cwd':'/project','cli_version':'0.153.4'}},
          {'type':'token_usage_record','payload':payload},
          {'type':'compacted','payload':{'compaction_response_id':rid,'latest_token_usage_record':copy.deepcopy(payload)}},
          {'type':'turn_context','payload':{'turn_id':'turn','model':'gpt-5.5','effort':'xhigh','cwd':'/project'}}]
    raw={'method':'rawResponse/completed','params':{'responseId':rid,'threadId':tid,'turnId':'turn','usage':wire(usage())}}
    proof={'source':path,'thread_id':tid,'turn_id':'turn','response_id':rid,'model':'gpt-5.5','effort':'xhigh','cwd':'/project',
           'native_response_line':2,'native_context_line':4,'native_marker_line':3,
           'native_response_payload_sha256':sha(payload,True),'native_rows_sha256':sha(rows,True),
           'native_metadata_sha256':sha(metadata,True),'capture':'/capture/'+RUNS[2],'raw_response_line':2}
    item.update(sidecar={'schema_version':1,'records':[{'response_id':rid,'thread_id':tid,'turn_id':'turn',
                 'usage':usage(),'valid':True,'first_server_sequence':0}], 'errors':[], 'duplicate_events':0,
                 'threads':[{'thread_id':tid,'status':'cumulative_mismatch'}],
                 'changes_workflow_totals':False,'proves_interrupted_tail_coverage':False},
                captures=[{'path':'/capture/'+RUNS[2],'rows':[None,raw]}],
                native_sources=[{'source':path,'thread_id':tid,'rows':rows}], metadata=metadata,
                eligibility={'status':'source-proof-established','errors':[],'proofs':[proof],
                             'accounting_calls':0,'substantive_calls':1,'provider_calls':0},
                eligibility_input={'metadata_row_sha256':sha(metadata,True)})
    item['original'].update(exact_response_evidence={},corrected_scope=None,measurement={'status':'ineligible','bounds':None})
    item['projection']=projection(item['original'])
    baselines=[]
    for i, run in enumerate(BASES):
        full=original(run,'baseline-response-'+str(i)); p=projection(full)
        p['accounting'].update(status='validated', measurement={'status':'bounded','bounds':{'opaque':'copied without arithmetic'}})
        projected={'run_id':run,'exact_response_evidence':p['accounting']['exact_response_evidence']}
        baselines.append({'run_id':run,'expected_response_count':1,'receipt':p,
                          'supplement':{'schema':'work-leaf-admitted-provider-ledger-execution-v1',
                           'original_entry':{'id':run},'result':{'schema':'work-leaf-provider-ledger-qualification-v1','original':projected},
                           'response_evidence':full['exact_response_evidence']}})
    return {'runs':runs,'baselines':baselines}


class LowerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if q is not None: cls.arithmetic=q.load_arithmetic(ACCOUNTING.read_bytes(),STRICT.read_bytes())

    def setUp(self):
        self.assertIsNotNone(q,'new lower-only helper is absent')
        self.value=fixture()

    def run_case(self):
        return q.qualify(self.value,self.arithmetic)

    def reject(self):
        result=self.run_case()
        self.assertTrue(result['errors'] or any(r['errors'] for r in result['candidates']))
        return result

    def test_complete_population_lower_only_and_baselines_unchanged(self):
        result=self.run_case(); self.assertEqual(result['errors'],[])
        self.assertEqual(len(result['candidates']),3);self.assertEqual(len(result['baselines']),6)
        for row in result['candidates']:
            self.assertEqual(row['status'],'qualified_observed_lower')
            self.assertEqual(row['bounds']['raw_input_plus_output'],{'lower':5,'upper':None})
            self.assertEqual(row['original_errors'],['retained original failure'])
        self.assertEqual(result['baselines'],[r['receipt'] for r in self.value['baselines']])

    def test_failed_candidate_and_unknown_tail_preserved(self):
        row=self.run_case()['candidates'][0]
        self.assertEqual(row['original_outcome'],{'exit_code':1})
        self.assertIsNone(row['original_measurement']['gap_inventory']['gaps'][0]['response_count_upper'])

    def test_usage_less_owned_thread_is_not_zero_response(self):
        s=self.value['runs'][0]['source_result'];s['native_membership']['threads'].append({'thread_id':'usage-less','source':'/native/usage-less','accepted_turn_count':1})
        s['native_membership']['contexts'].append({'thread_id':'usage-less','turn_id':'failed'})
        s['delivery']['inputs'].append({'thread_id':'usage-less','turn_id':'failed','status':'joined','native_source':'/native/usage-less','capture_index':0,'capture_id':RUNS[0]})
        self.value['runs'][0]['source_input']['native_sessions'].append({'thread_id':'usage-less','source':{'path':'/native/usage-less'}})
        result=self.run_case();self.assertEqual(result['candidates'][0]['response_count'],1)
        self.assertIn({'thread_id':'usage-less','turn_id':'failed'},result['candidates'][0]['accepted_turns_without_recorded_response'])

    def test_projection_digest_mismatch(self):
        self.value['runs'][0]['projection']['accounting']['exact_helper_result_sha256']='0'*64;self.reject()

    def test_original_map_not_subset(self):
        self.value['runs'][0]['expected_response_count']=2;self.reject()

    def test_baseline_count_or_hash_mismatch(self):
        self.value['baselines'][0]['receipt']['accounting']['exact_response_evidence']['count']=2;self.reject()

    def test_all_nine_duplicate_response_identity_rejects(self):
        r=self.value['runs'][0];row=r['original']['exact_response_evidence'].pop('response-0')
        r['original']['exact_response_evidence']['baseline-response-0']=row
        raw=r['original']['measurement']['gap_inventory']['raw_responses'];raw['baseline-response-0']=raw.pop('response-0')
        r['projection']=projection(r['original'])
        result=self.reject();self.assertTrue(result['errors'])

    def test_sidecar_missing_extra_and_invalid_records(self):
        for mutation in ('missing','extra','false','null','bool','sequence'):
            with self.subTest(mutation=mutation):
                self.value=fixture();rows=self.value['runs'][2]['sidecar']['records']
                if mutation=='missing':rows.clear()
                elif mutation=='extra':rows.append({**rows[0],'response_id':'extra'})
                elif mutation=='false':rows[0]['valid']=False
                elif mutation=='null':rows[0]['usage']=None
                elif mutation=='bool':rows[0]['usage']['output_tokens']=True
                else:rows[0]['first_server_sequence']=1
                self.reject()

    def test_blank_physical_lines_not_message_sequence(self):
        self.assertEqual(self.run_case()['candidates'][2]['response_count'],1)

    def test_blank_and_literal_empty_object_have_distinct_ordinals(self):
        r=self.value['runs'][2];raw=r['captures'][0]['rows'][1]
        r['captures'][0]['rows']=[None,{},raw]
        r['sidecar']['records'][0]['first_server_sequence']=1
        r['eligibility']['proofs'][0]['raw_response_line']=3
        result=self.run_case();self.assertEqual(result['errors'],[])

    def test_extra_raw_or_native_response_rejects(self):
        for kind in ('raw','native'):
            with self.subTest(kind=kind):
                self.value=fixture();r=self.value['runs'][2]
                if kind=='raw':
                    row=copy.deepcopy(r['captures'][0]['rows'][1]);row['params']['responseId']='extra';r['captures'][0]['rows'].append(row)
                else:
                    row=copy.deepcopy(r['native_sources'][0]['rows'][1]);row['payload']['response_id']='extra';r['native_sources'][0]['rows'].append(row)
                self.reject()

    def test_raw_cache_write_and_native_total_reject(self):
        for native in (False,True):
            self.value=fixture();r=self.value['runs'][2]
            if native:r['native_sources'][0]['rows'][1]['payload']['usage']['total_tokens']=99
            else:r['captures'][0]['rows'][1]['params']['usage']['cacheWriteInputTokens']=1
            self.reject()

    def test_identical_raw_replay_consumed_once(self):
        r=self.value['runs'][2];r['captures'][0]['rows'].append(copy.deepcopy(r['captures'][0]['rows'][1]));r['sidecar']['duplicate_events']=1
        self.assertEqual(self.run_case()['candidates'][2]['response_count'],1)

    def test_conflicting_raw_replay_rejects(self):
        r=self.value['runs'][2];row=copy.deepcopy(r['captures'][0]['rows'][1]);row['params']['turnId']='other';r['captures'][0]['rows'].append(row);self.reject()

    def test_future_context_requires_exact_saved_proof(self):
        for field in ('native_rows_sha256','native_response_payload_sha256','native_metadata_sha256','response_id','native_response_line','raw_response_line'):
            with self.subTest(field=field):
                self.value=fixture();proof=self.value['runs'][2]['eligibility']['proofs'][0]
                proof[field]=99 if type(proof[field]) is int else 'wrong';self.reject()

    def test_proof_count_and_unknown_ownership_reject(self):
        self.value['runs'][2]['eligibility']['proofs']*=2;self.reject()
        self.value=fixture();self.value['runs'][2]['source_result']['delivery']['inputs'][0]['status']='ambiguous';self.reject()

    def test_wrong_context_or_foreign_native_thread_rejects(self):
        self.value['runs'][2]['native_sources'][0]['rows'][3]['payload']['model']='other';self.reject()
        self.value=fixture();self.value['runs'][2]['native_sources'][0]['thread_id']='foreign';self.reject()

    def test_missing_source_subproof_rejects(self):
        self.value['runs'][1]['source_result']['native_membership']['errors']=['bad'];self.reject()

    def test_unrelated_source_error_not_waived(self):
        self.value['runs'][0]['source_result']['errors'].append('source-endpoint-invalid');self.reject()

    def test_all_populations_pass_before_any_candidate_sum(self):
        self.value['runs'][2]['sidecar']['records'].clear()
        spy=Mock(wraps=self.arithmetic)
        result=q.qualify(self.value,spy)
        spy.add.assert_not_called()
        self.assertTrue(all(r['bounds'] is None and r['status']=='unqualified' for r in result['candidates']))

    def test_baseline_failure_keeps_all_six_receipts(self):
        self.value['baselines'][0]['expected_response_count']=99
        result=self.reject()
        self.assertEqual(result['baselines'],[r['receipt'] for r in self.value['baselines']])

    def test_prior_context_needs_no_numeric_compaction_addition(self):
        r=self.value['runs'][2];rows=r['native_sources'][0]['rows'];rows.insert(1,copy.deepcopy(rows[3]))
        proof=r['eligibility']['proofs'][0];proof.update(native_rows_sha256=sha(rows,True),native_response_line=3,native_marker_line=4,native_context_line=5)
        # Same completed response, not one ordinary plus an extra compaction charge.
        self.assertEqual(self.run_case()['candidates'][2]['bounds']['raw_input_plus_output']['lower'],5)

    def test_source_arithmetic_pin_and_ast_definition_identity(self):
        with self.assertRaises(ValueError):q.load_arithmetic(ACCOUNTING.read_bytes()+b'\n',STRICT.read_bytes())
        self.assertEqual(self.arithmetic.checked_usage(wire(usage())),usage())

    def test_disarmed_execute_before_attempt(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'scope.json';out=Path(d)/'result.json';p.write_text(json.dumps({'schema':'work-leaf-c08-observed-lower-scope-v1','execution_authorized':False}))
            with self.assertRaises(ValueError):q.execute(str(p),hashlib.sha256(p.read_bytes()).hexdigest(),str(out))
            self.assertFalse(out.exists());self.assertFalse(Path(str(out)+'.ATTEMPT.json').exists())

    def test_strict_source_reader_rejects_symlink_and_drift(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'input';p.write_bytes(b'{}');pins={str(p):hashlib.sha256(p.read_bytes()).hexdigest()};reader=q.Sources(pins)
            self.assertEqual(reader.read({'path':str(p),'sha256':pins[str(p)]}),b'{}')
            p.write_bytes(b'[]')
            with self.assertRaises(ValueError):reader.verify()
            link=Path(d)/'alias';link.symlink_to(p)
            with self.assertRaises(ValueError):q.Sources({str(link):hashlib.sha256(p.read_bytes()).hexdigest()}).verify()


if __name__=='__main__':unittest.main()
