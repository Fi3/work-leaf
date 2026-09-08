"""Temporary synthetic source/publication fixtures; no saved qualification."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import qualify_lower as q
from test_qualify_lower import ACCOUNTING, STRICT, fixture, RUNS, BASES


class MemoryIndex:
    """Synthetic materialization contract only; real endpoint guards have separate tests."""
    def __init__(self):self.documents={};self.pins={};self.raw={}
    def admit(self,pins):
        for path,digest in pins.items():
            if path in self.pins and self.pins[path]!=digest:raise ValueError('conflicting fixture pin')
            self.pins[path]=digest
    def json(self,ref):return copy.deepcopy(self.documents[ref['path']])
    def rows(self,ref,**_):return copy.deepcopy(self.documents[ref['path']])
    def read(self,ref):
        if ref['path'] in self.raw:return self.raw[ref['path']]
        value=self.documents[ref['path']]
        return b'\n'.join(q.canonical(r) for r in value)+b'\n' if isinstance(value,list) else q.canonical(value)
    def put(self,path,value):
        self.documents[path]=copy.deepcopy(value)
        return {'path':path,'sha256':q.sha(q.canonical(value))}


def input_fixture():
    pop=fixture();index=MemoryIndex();scope={'candidates':[],'source_sha256':{}}
    ar={'path':'/code/accounting.py','sha256':q.ACCOUNTING_SHA};scope['arithmetic']={'accounting':ar,'strict':{'path':'/code/strict.py','sha256':q.STRICT_SHA}}
    phase=index.put('/phase.json',{'files':[dict(ar,role='frozen-evidence')]})
    old={'schema':'work-leaf-c08-original-accounting-once-v1','new_run_ids':RUNS,'baseline_run_ids':BASES,
         'helper':ar,'phase_manifest':phase,'baselines':[],'source_sha256':{}}
    oldrows=[]
    for base in pop['baselines']:
        run=base['run_id'];receipt=index.put('/'+run+'/receipt.json',base['receipt']);supp=index.put('/'+run+'/supplement.json',base['supplement'])
        old['baselines'].append({'run_id':run,'expected_response_count':1,'receipt':receipt,'response_supplement':supp})
        oldrows.append({'run_id':run,'receipt':receipt,'response_supplement':supp,'accounting':base['receipt']['accounting']})
    scope['accounting_scope']=index.put('/old-scope.json',old)
    for case in pop['runs']:
        run=case['run_id'];inp=case['source_input'];sr=case['source_result'];capture=inp['captures'][0]['path']
        for name in ('clients','forwarded','servers','start','child','end','settings','journal','grace'):
            value=case['captures'][0]['rows'] if run==RUNS[2] and name=='servers' else {}
            inp['captures'][0][name]=index.put(capture+'/'+('server' if name=='servers' else name),value)
        for native in inp['native_sessions']:
            value=case['native_sources'][0]['rows'] if run==RUNS[2] else []
            native['source']=index.put(native['source']['path'],value)
        full=index.put('/'+run+'/full.json',case['original']);proj=copy.deepcopy(case['projection']);proj['scope_sha256']=scope['accounting_scope']['sha256']
        pref=index.put('/'+run+'/projection.json',proj)
        oldrows.append({'run_id':run,'full_result':full,'receipt':pref,'accounting':proj['accounting'],'original_entry':proj['original_entry']})
        input_path='/'+run+'/source-input.json';result_path='/'+run+'/source-result.json'
        ss=index.put('/'+run+'/source-scope.json',{'input_paths':{run:input_path},'output_paths':{run:result_path}})
        inp.update(schema='work-leaf-c08-natural-source-input-v1',run_id=run,helper_sha256=q.SOURCE_HELPER_SHA,scope=ss,phase_manifest=phase)
        inp_ref=index.put(input_path,inp)
        pins={inp_ref['path']:inp_ref['sha256'],ss['path']:ss['sha256'],'/code/source.py':q.SOURCE_HELPER_SHA}
        for cap in inp['captures']:
            pins.update({cap[k]['path']:cap[k]['sha256'] for k in ('clients','forwarded','servers','start','child','end','settings','journal','grace')})
        pins.update({n['source']['path']:n['source']['sha256'] for n in inp['native_sessions']})
        sr.update(schema='work-leaf-c08-natural-source-result-v1',source_sha256=pins)
        sr_ref=index.put(result_path,sr)
        candidate={'run_id':run,'expected_response_count':1,'original_full':full,'original_projection':pref,'source_input':inp_ref,'source_result':sr_ref}
        if run==RUNS[2]:
            metadata=index.put('/metadata.jsonl',[case['metadata']]);proof=case['eligibility']['proofs'][0]
            ei={'schema':'work-leaf-c08-compaction-eligibility-only-input-v1','run_id':run,
                'helper':{'path':'/code/context.py','sha256':q.CONTEXT_HELPER_SHA},
                'source_qualification':{'result':sr_ref,'input':inp_ref},'original_full_result':full,
                'metadata':metadata,'metadata_line':1,'metadata_row_sha256':case['eligibility_input']['metadata_row_sha256'],
                'capture':inp['captures'][0],'expected_proof':{k:proof[k] for k in ('response_id','thread_id','turn_id')}}
            ei_ref=index.put('/eligibility-input.json',ei);er=case['eligibility']
            er.update(schema='work-leaf-c08-compaction-eligibility-only-result-v1',run_id=run,input_sha256=ei_ref['sha256'],
                      source_sha256={metadata['path']:metadata['sha256'],'/code/context.py':q.CONTEXT_HELPER_SHA})
            candidate.update(eligibility_input=ei_ref,eligibility_result=index.put('/eligibility-result.json',er),
                             response_sidecar=index.put(capture+'/response-usage.json',case['sidecar']))
        scope['candidates'].append(candidate)
    consolidated={'schema':'work-leaf-c08-original-accounting-publication-v1','scope_sha256':scope['accounting_scope']['sha256'],
                  'runs':oldrows,'integrity_errors':[],'identity_errors':[],'execution_errors':[],'publication_errors':[],
                  'accounting_call_attempts':RUNS,'baseline_accounting_calls':0}
    scope['original_consolidation']=index.put('/original-consolidation.json',consolidated)
    scope['preparation']=index.put('/preparation.json',{})
    return scope,index


def disk_fixture(root):
    """Materialize the same finite synthetic DAG, never a saved study payload."""
    scope,index=input_fixture()
    def move(value):
        if isinstance(value,str) and value.startswith('/') and value!='/project':return str(root/value.lstrip('/'))
        if isinstance(value,list):return [move(v) for v in value]
        if isinstance(value,dict):return {move(k):move(v) for k,v in value.items()}
        return value
    scope=move(scope);docs={move(p):move(v) for p,v in index.documents.items()};raw={}
    def path(name):return str(root/name.lstrip('/'))
    raw[path('/code/accounting.py')]=ACCOUNTING.read_bytes()
    raw[path('/code/strict.py')]=STRICT.read_bytes()
    static=Path(q.__file__).resolve().parent.parent
    raw[path('/code/source.py')]=(static/'c08-natural-delivery/source-projection-correction/audit_automatic_refresh_sources_corrected.py').read_bytes()
    raw[path('/code/context.py')]=(static/'candidate-compaction-context-correction/accounting_compaction_context.py').read_bytes()
    for name,value in docs.items():
        if '/native/' in name or name.endswith('/server'):
            raw[name]=b'\n'.join(b'' if row is None else q.canonical(row) for row in value)+b'\n'
    metadata=docs[path('/metadata.jsonl')][0]
    raw[path('/metadata.jsonl')]=json.dumps(dict(reversed(list(metadata.items()))),separators=(', ', ': ')).encode()+b'\n'
    raw[scope['preparation']['path']]=b'# Prospective synthetic fixed population\n'
    oldpath=scope['accounting_scope']['path'];conpath=scope['original_consolidation']['path']
    by_id={r['run_id']:r for r in docs[conpath]['runs']}
    for candidate in scope['candidates']:
        full=docs[candidate['original_full']['path']];proj=docs[candidate['original_projection']['path']]
        proj['accounting']['exact_helper_result_sha256']=q.sha(q.canonical(full))
        proj['accounting']['exact_response_evidence']['sha256']=q.sha(q.canonical(full['exact_response_evidence']))
        by_id[candidate['run_id']]['accounting']=copy.deepcopy(proj['accounting'])
    for b in docs[oldpath]['baselines']:
        receipt=docs[b['receipt']['path']];supp=docs[b['response_supplement']['path']]
        digest=q.sha(q.canonical(supp['response_evidence']))
        receipt['accounting']['exact_response_evidence']['sha256']=digest
        supp['result']['original']['exact_response_evidence']['sha256']=digest
        by_id[b['run_id']]['accounting']=copy.deepcopy(receipt['accounting'])
    ep=scope['candidates'][2]['eligibility_input']['path'];er=scope['candidates'][2]['eligibility_result']['path']
    docs[ep]['metadata_row_sha256']=q.sha(raw[path('/metadata.jsonl')].splitlines()[0])
    memo={};active=set();projection_paths={c['original_projection']['path'] for c in scope['candidates']}
    def finalize(name):
        if name in raw:return raw[name]
        if name in memo:return memo[name]
        if name in active:raise AssertionError('synthetic reference cycle')
        active.add(name);value=copy.deepcopy(docs[name])
        if name in projection_paths or name==conpath:value['scope_sha256']=q.sha(finalize(oldpath))
        if name==er:value['input_sha256']=q.sha(finalize(ep))
        value=resolve(value)
        data=q.canonical(value);memo[name]=data;active.remove(name);return data
    def resolve(value):
        if isinstance(value,list):return [resolve(v) for v in value]
        if isinstance(value,dict):
            result={k:resolve(v) for k,v in value.items()}
            if 'path' in result and 'sha256' in result and result['path'] in docs.keys()|raw.keys():result['sha256']=q.sha(finalize(result['path']))
            if 'source_sha256' in result:
                result['source_sha256']={p:q.sha(finalize(p)) if p in docs or p in raw else h for p,h in result['source_sha256'].items()}
            return result
        return value
    for name in docs:finalize(name)
    for name,data in {**raw,**memo}.items():
        p=Path(name);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
    scope=resolve(scope);scope.update(schema='work-leaf-c08-observed-lower-scope-v1',execution_authorized=True,output_path=str(root/'result.json'))
    scope['arithmetic']['strict']={'path':path('/code/strict.py'),'sha256':q.STRICT_SHA}
    scope['source_sha256'].update({str(Path(q.__file__).resolve()):q.sha(Path(q.__file__).read_bytes()),
                                  str(Path('/usr/bin/python3').resolve()):q.sha(Path('/usr/bin/python3').resolve().read_bytes()),
                                  path('/code/strict.py'):q.STRICT_SHA})
    p=root/'scope.json';p.write_bytes(q.canonical(scope));return p,scope


class MaterializeTests(unittest.TestCase):
    def test_exact_nested_reference_contract(self):
        scope,index=input_fixture();pop=q.materialize(scope,index)
        arithmetic=q.load_arithmetic(ACCOUNTING.read_bytes(),STRICT.read_bytes())
        result=q.qualify(pop,arithmetic)
        self.assertEqual(result['errors'],[])

    def test_preparation_is_pinned_markdown_not_json(self):
        scope,index=input_fixture();scope['preparation']=index.put('/LOWER-BOUND-PREPARATION.md','# Prospective fixed population\n')
        index.raw[scope['preparation']['path']]=b'# Prospective fixed population\n'
        self.assertEqual(len(q.materialize(scope,index)['runs']),3)

    def test_raw_metadata_line_hash_is_not_canonical_object_hash(self):
        scope,index=input_fixture();metadata=index.documents['/metadata.jsonl'][0]
        raw=json.dumps(dict(reversed(list(metadata.items()))),separators=(', ', ': ')).encode()
        index.raw['/metadata.jsonl']=raw+b'\n'
        index.documents['/eligibility-input.json']['metadata_row_sha256']=q.sha(raw)
        self.assertNotEqual(q.sha(raw),q.sha(q.canonical(metadata,True)))
        pop=q.materialize(scope,index)
        result=q.qualify(pop,q.load_arithmetic(ACCOUNTING.read_bytes(),STRICT.read_bytes()))
        self.assertEqual(result['errors'],[])

    def test_wrong_original_ref_and_unbound_capture_fail(self):
        for change in ('original','capture','helper','eligibility','metadata','sidecar'):
            with self.subTest(change=change):
                scope,index=input_fixture();c=scope['candidates'][2]
                if change=='original':c['original_full']={'path':'/other','sha256':'1'*64}
                elif change=='capture':index.documents[c['source_result']['path']]['source_sha256'].pop('/capture/'+RUNS[2]+'/server')
                elif change=='helper':index.documents[c['source_input']['path']]['helper_sha256']='1'*64
                elif change=='eligibility':index.documents[c['eligibility_result']['path']]['input_sha256']='1'*64
                elif change=='metadata':index.documents[c['eligibility_input']['path']]['metadata_line']=99
                else:c['response_sidecar']={'path':'/other-sidecar','sha256':'1'*64}
                with self.assertRaises((ValueError,KeyError)):q.materialize(scope,index)

    def test_duplicate_original_disposition_is_not_collapsed(self):
        scope,index=input_fixture();rows=index.documents[scope['original_consolidation']['path']]['runs']
        rows.append(copy.deepcopy(rows[0]))
        with self.assertRaises(ValueError):q.materialize(scope,index)


class SourceTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name);self.output=self.root/'result.json';self.scope=self.root/'scope.json'
        self.value={'schema':'work-leaf-c08-observed-lower-scope-v1','execution_authorized':True,
                    'output_path':str(self.output),'source_sha256':{},'arithmetic':{}}
        for name,path in (('accounting',ACCOUNTING),('strict',STRICT)):
            ref={'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
            self.value['arithmetic'][name]=ref;self.value['source_sha256'][str(path)]=ref['sha256']
        for path in (Path(q.__file__).resolve(),Path('/usr/bin/python3').resolve()):
            self.value['source_sha256'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
        self.population=fixture()

    def write(self):
        self.scope.write_bytes(q.canonical(self.value));return hashlib.sha256(self.scope.read_bytes()).hexdigest()

    def invoke(self):
        with patch.object(q,'materialize',return_value=self.population):
            return q.execute(str(self.scope),self.write(),str(self.output))

    def test_create_new_once_with_finite_population_and_endpoint_checks(self):
        result=self.invoke();self.assertEqual(result['status'],'qualified_observed_lower')
        self.assertEqual(json.loads(self.output.read_bytes())['status'],result['status'])
        prior=self.output.read_bytes()
        with patch.object(q,'materialize',side_effect=AssertionError('must not retry')):
            with self.assertRaises(ValueError):q.execute(str(self.scope),self.write(),str(self.output))
        self.assertEqual(self.output.read_bytes(),prior)

    def test_existing_attempt_and_output_alias_prevent_population_call(self):
        Path(str(self.output)+'.ATTEMPT.json').write_bytes(b'old')
        with patch.object(q,'materialize',side_effect=AssertionError('must not start')):
            with self.assertRaises(ValueError):q.execute(str(self.scope),self.write(),str(self.output))
        self.assertFalse(self.output.exists())

    def test_symlink_output_prevents_work(self):
        self.output.symlink_to(self.root/'absent')
        with patch.object(q,'materialize',side_effect=AssertionError('must not start')):
            with self.assertRaises(ValueError):q.execute(str(self.scope),self.write(),str(self.output))

    def test_population_failure_preserves_attempt_and_terminal_error(self):
        with patch.object(q,'materialize',side_effect=ValueError('synthetic source binding failure')):
            result=q.execute(str(self.scope),self.write(),str(self.output))
        self.assertEqual(result['status'],'unqualified')
        self.assertTrue(Path(str(self.output)+'.ATTEMPT.json').exists())
        self.assertEqual(len(result['candidates']),3)

    def test_postread_drift_never_publishes_qualified_bounds(self):
        extra=self.root/'extra';extra.write_bytes(b'original')
        self.value['source_sha256'][str(extra)]=hashlib.sha256(extra.read_bytes()).hexdigest()
        def population(scope,index):
            extra.write_bytes(b'changed');return self.population
        with patch.object(q,'materialize',side_effect=population):
            result=q.execute(str(self.scope),self.write(),str(self.output))
        self.assertEqual(result['status'],'unqualified')
        self.assertTrue(all(row['bounds'] is None for row in result['candidates']))

    def test_source_union_conflict_fails(self):
        index=q.Sources({'/absolute':'1'*64})
        with self.assertRaises(ValueError):index.admit({'/absolute':'2'*64})

    def test_complete_actual_file_pipeline_without_materialize_mock(self):
        with tempfile.TemporaryDirectory() as d:
            p,scope=disk_fixture(Path(d));result=q.execute(str(p),q.sha(p.read_bytes()),scope['output_path'])
            self.assertEqual(result['errors'],[])
            self.assertEqual(result['status'],'qualified_observed_lower')
            self.assertTrue(all(r['bounds']['raw_input_plus_output']['upper'] is None for r in result['candidates']))

    def test_failed_result_write_keeps_attempt_and_forbids_retry(self):
        original=q.write_fd;calls=[]
        def write(fd,data):
            calls.append(fd)
            if len(calls)==2:raise OSError('synthetic publication failure')
            return original(fd,data)
        with patch.object(q,'materialize',return_value=self.population),patch.object(q,'write_fd',side_effect=write):
            with self.assertRaises(OSError):q.execute(str(self.scope),self.write(),str(self.output))
        self.assertTrue(Path(str(self.output)+'.ATTEMPT.json').exists())
        with patch.object(q,'materialize',side_effect=AssertionError('must not retry')):
            with self.assertRaises(ValueError):q.execute(str(self.scope),self.write(),str(self.output))

    def test_source_drift_after_arithmetic_removes_all_bounds(self):
        extra=self.root/'extra';extra.write_bytes(b'original')
        self.value['source_sha256'][str(extra)]=q.sha(extra.read_bytes());original=q.qualify
        def qualified(pop,arithmetic):
            result=original(pop,arithmetic);extra.write_bytes(b'changed');return result
        with patch.object(q,'materialize',return_value=self.population),patch.object(q,'qualify',side_effect=qualified):
            result=q.execute(str(self.scope),self.write(),str(self.output))
        self.assertIn('source-final-endpoint-failed',result['errors'])
        self.assertTrue(all(r['bounds'] is None and r['status']=='unqualified' for r in result['candidates']))


if __name__=='__main__':unittest.main()
