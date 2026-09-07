import copy
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

import qualify as Q


def fixture():
    thread, turn, agent = 'thread-a', 'turn-a', 'author-a'
    prompt = 'You are running under the work-leaf orchestrator.\n\nAgent-ID: author-a\nFeature: demo\n\nUser prompt:\nwork'
    usage = dict(input_tokens=10, cached_input_tokens=2, output_tokens=4, reasoning_output_tokens=1,
                 uncached_input_tokens=8, raw_input_plus_output=14, uncached_input_plus_output=12)
    response = dict(thread_id=thread, turn_id=turn, usage=usage)
    error = 'visible provider agent author-a has no controller usage row'
    observed = dict(threads=[dict(thread_id=thread, agent_id=agent, primary=True, visible=True)],
                    errors=[error], capture_complete=False, interrupted_provider_turns=0,
                    invocation_count=1, complete_invocation_count=1, session_only_threads=[],
                    controller_usage_reconciliation=[dict(agent_id=agent, provider_thread_ids=[thread],
                        controller_streamed_usage=None, replayed_streamed_usage=None,
                        provider_largest_cumulative_usage=usage, controller_matches_replay=False)])
    policy = dict(model='gpt-5.5', reasoning_effort='xhigh', per_response=dict(input_upper=1050000, output_upper=128000))
    original = dict(status='unknown', errors=[error, 'capture status is inconsistent'],
        corrected_scope=dict(usage=usage), exact_response_evidence={'response-a':response},
        measurement=dict(status='ineligible', bounds=None, reasons=[error, 'capture status is inconsistent'],
            recorded_usage=usage, bound_policy=policy,
            gap_inventory=dict(errors=[], gaps=[], raw_responses={'response-a':response})))
    client = {'id':1,'method':'thread/start','params':{'cwd':'/project','model':'gpt-5.5'}}
    request = {'id':2,'method':'turn/start','params':{'threadId':thread,'input':[{'type':'text','text':prompt}]}}
    camel = dict(inputTokens=10,cachedInputTokens=2,outputTokens=4,reasoningOutputTokens=1,totalTokens=14)
    servers = [
        {'id':1,'result':{'thread':{'id':thread}}},
        {'id':2,'result':{'turn':{'id':turn}}},
        {'method':'item/completed','params':{'threadId':thread,'turnId':turn,
            'item':{'type':'userMessage','id':'public-a','content':[{'type':'text','text':prompt}]}}},
        {'method':'item/completed','params':{'threadId':thread,'turnId':turn,
            'item':{'type':'agentMessage','id':'message-a','text':'@work-leaf done'}}},
        {'method':'rawResponse/completed','params':{'threadId':thread,'turnId':turn,'responseId':'response-a','usage':camel}},
        {'method':'thread/tokenUsage/updated','params':{'threadId':thread,'turnId':turn,'tokenUsage':{'last':camel,'total':camel}}},
        {'method':'turn/completed','params':{'threadId':thread,'turn':{'id':turn,'status':'interrupted'}}}]
    trace = [dict(schema='work-leaf-bench-experiment-v3',event='activation',run_id='run-a',condition='control',process_id=7),
        dict(schema='work-leaf-bench-experiment-v3',event='prompt',site='policy-injection',run_id='run-a',
            condition='control',process_id=7,sequence=1,agent_id=agent,original_prompt=prompt,forwarded_prompt=prompt,
            original_bytes=len(prompt.encode()),forwarded_bytes=len(prompt.encode()),changed=False,byte_delta=0)]
    native = [dict(source='/sessions/thread-a.jsonl', thread_id=thread, rows=[
        {'type':'session_meta','payload':{'id':thread,'cwd':'/project','cli_version':'0.153.4'}},
        {'type':'turn_context','payload':{'turn_id':turn,'cwd':'/project','model':'gpt-5.5','effort':'xhigh'}},
        {'type':'response_item','payload':{'type':'message','role':'user','id':'native-a',
            'content':[{'type':'input_text','text':prompt}],
            'internal_chat_message_metadata_passthrough':{'turn_id':turn,'content_item_kinds':['user.text']}}},
        {'type':'token_usage_record','payload':{'thread_id':thread,'turn_id':turn,'response_id':'response-a','usage':usage}}])]
    return dict(original=original,observed=observed,controller=[],state={'snapshot':{'sessions':[{'id':agent,'token_usage':None}]}},
        trace=trace,captures=[dict(path='/capture',clients=[client,request],forwarded=[copy.deepcopy(client),copy.deepcopy(request)],servers=servers)],
        natives=native,run_id='run-a',condition='control')


class QualifierTests(unittest.TestCase):
    def test_controller_unavailable_is_not_zero_and_original_is_unchanged(self):
        args=fixture(); before=copy.deepcopy(args)
        got=Q.qualify(**args)
        self.assertEqual(got['status'],'qualified_provider_ledger')
        self.assertEqual(got['measurement']['status'],'exact')
        self.assertEqual(got['measurement']['bounds']['raw_input_plus_output'],{'lower':14,'upper':14})
        self.assertEqual(got['controller_unavailable_agents'],['author-a'])
        self.assertEqual(got['evidence']['unavailable_turns'][0]['directive_line'],4)
        self.assertEqual(args,before)

    def test_v2_actual_schema_not_converted(self):
        args=fixture()
        for row in args['trace']:row['schema']='work-leaf-bench-experiment-v2'
        self.assertEqual(Q.qualify(**args)['status'],'qualified_provider_ledger')

    def test_actual_pre_directive_usage_rejects_null_controller(self):
        args=fixture(); s=args['captures'][0]['servers'];s.insert(3,s.pop(5))
        self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_eventless_accepted_turn_cannot_escape_missing_controller_proof(self):
        args=fixture();cap=args['captures'][0]
        request=copy.deepcopy(cap['clients'][1]);request['id']=3
        cap['clients'].append(request);cap['forwarded'].append(copy.deepcopy(request))
        cap['servers'].append({'id':3,'result':{'turn':{'id':'failed-turn'}}})
        public=copy.deepcopy(cap['servers'][2]);public['params']['turnId']='failed-turn';public['params']['item']['id']='public-b'
        cap['servers'].append(public)
        cap['servers'].append({'method':'turn/completed','params':{'threadId':'thread-a','turn':{'id':'failed-turn','status':'failed'}}})
        native=args['natives'][0]['rows'];context=copy.deepcopy(native[1]);context['payload']['turn_id']='failed-turn'
        user=copy.deepcopy(native[2]);user['payload']['id']='native-b';user['payload']['internal_chat_message_metadata_passthrough']['turn_id']='failed-turn'
        native.extend([context,user])
        got=Q.qualify(**args)
        self.assertEqual(got['status'],'unknown')
        self.assertIn('every accepted turn',got['errors'][0])

    def test_controller_or_state_or_replay_disagreement_is_not_waived(self):
        for field in ('controller','state','replayed'):
            args=fixture();u=args['original']['measurement']['recorded_usage']
            if field=='controller':args['controller']=[{'agent_id':'author-a','usage':u}]
            if field=='state':args['state']['snapshot']['sessions'][0]['token_usage']=u
            if field=='replayed':args['observed']['controller_usage_reconciliation'][0]['replayed_streamed_usage']=u
            with self.subTest(field=field):self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_every_unrelated_error_and_false_original_completeness_is_retained(self):
        for change in ('transport','capture','invocation','scope','gap','reason'):
            args=fixture()
            if change=='transport':args['observed']['errors'].append('unrelated transport error')
            if change=='capture':args['observed']['capture_complete']=True
            if change=='invocation':args['observed']['complete_invocation_count']=0
            if change=='scope':args['observed']['session_only_threads']=['unknown']
            if change=='gap':args['original']['measurement']['gap_inventory']['errors']=['unsupported tail']
            if change=='reason':args['original']['errors'].append('actual raw mismatch')
            with self.subTest(change=change):self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_owned_policy_and_all_native_threads_are_required(self):
        for change in ('policy','agent','native','title','extra'):
            args=fixture()
            if change=='policy':args['trace']=args['trace'][:1]
            if change=='agent':args['trace'][1]['agent_id']='someone-else'
            if change=='native':args['natives']=[]
            if change=='title':args['observed']['threads'].append({'thread_id':'hidden','agent_id':'title-agent','primary':True,'visible':False})
            if change=='extra':args['natives'].append(copy.deepcopy(args['natives'][0]))
            with self.subTest(change=change):self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_typed_one_to_one_input_and_terminal_boundaries(self):
        for change in ('mixed','bool','forward','input','native','terminal','duplicate'):
            args=fixture();c=args['captures'][0]
            if change=='mixed':c['servers'][1]['error']={'code':1,'message':'failure'}
            if change=='bool':c['servers'][1]['result']['turn']['id']=True
            if change=='forward':c['forwarded'][1]['params']['input'][0]['text']='different'
            if change=='input':c['clients'][1]['params']['input'].append({'type':'image','url':'x'})
            if change=='native':args['natives'][0]['rows'][2]['payload']['internal_chat_message_metadata_passthrough']['turn_id']='wrong'
            if change=='terminal':c['servers'].pop()
            if change=='duplicate':c['servers'].append(copy.deepcopy(c['servers'][1]))
            with self.subTest(change=change):self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_native_and_raw_exact_response_fields_are_not_optional(self):
        for change in ('usage','identity','extra','missing'):
            args=fixture();p=args['natives'][0]['rows'][3]['payload']
            if change=='usage':p['usage']['input_tokens']=True
            if change=='identity':p['response_id']=True
            if change=='extra':p['response_id']='different'
            if change=='missing':args['natives'][0]['rows'].pop()
            with self.subTest(change=change):self.assertEqual(Q.qualify(**args)['status'],'unknown')

    def test_finite_and_unknown_tails_remain_distinct(self):
        for finite in (True,False):
            args=fixture();o=args['observed'];m=args['original']['measurement']
            o['interrupted_provider_turns']=1;o['errors'].append('interrupted provider turn has no complete usage: count=1')
            args['original']['errors'].remove('capture status is inconsistent');m['reasons'].remove('capture status is inconsistent')
            m['gap_inventory']['gaps']=[dict(thread_id='thread-a',turn_id='turn-a',response_count_upper=1 if finite else None,
                proof='isolated_normal_response_tail' if finite else None)]
            got=Q.qualify(**args)
            self.assertEqual(got['status'],'qualified_provider_ledger')
            self.assertEqual(got['measurement']['bounds']['raw_input_plus_output']['upper'],1178014 if finite else None)

    def test_complete_directive_parser_matches_supported_observer_boundary(self):
        for text in ('@work-leaf read file','@work-leaf read','@work-leaf read\tfile','@work-leaf done','@work-leaf edit title\n@work-leaf end','@work-leaf locks run true'):
            self.assertTrue(Q.complete_directive(text))
        for text in ('@work-leaf edit title','@work-leafish done','ordinary text','@work-leaf end'):
            self.assertFalse(Q.complete_directive(text))

    def test_dependency_admission_precedes_original_module_execution(self):
        study=Path(__file__).resolve().parents[2]
        helper=study/'phases/untracked-reads-01/infrastructure/evidence/bench-results'/study.name/'accounting_untracked_reads.py'
        index=Q.SourceIndex({str(helper):Q.ACCOUNTING_SHA})
        with patch('builtins.exec') as execute:
            with self.assertRaisesRegex(ValueError,'admitted'):
                Q.load_original(helper,index)
        execute.assert_not_called()

    def test_source_wrapper_rejects_before_accounting_on_changed_source(self):
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'source.json';path.write_text('{}')
            admission={'source_sha256':{str(path):'0'*64},'original_source_qualification':{'original_membership_flag':'FAILED'}}
            with patch.object(Q,'load_original') as load:
                got=Q.audit_run({}, {}, root, admission)
            load.assert_not_called()
            self.assertEqual(got['qualification']['status'],'unknown')
            self.assertEqual(got['original_source_qualification'],admission['original_source_qualification'])

    def test_closed_source_json_and_path_guards(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root);path=root/'rows.jsonl';path.write_bytes(b'{"id":1}\n{"id":2}')
            index=Q.SourceIndex({str(path):hashlib.sha256(path.read_bytes()).hexdigest()})
            with self.assertRaisesRegex(ValueError,'closed JSONL'):index.rows(path)
            path.write_text('{"id":1,"id":2}\n')
            index=Q.SourceIndex({str(path):hashlib.sha256(path.read_bytes()).hexdigest()})
            with self.assertRaisesRegex(ValueError,'duplicate JSON'):index.rows(path)
            alias=root/'alias';alias.symlink_to(path)
            with self.assertRaisesRegex(ValueError,'canonical'):Q.SourceIndex({str(alias):'0'*64}).read(alias)

    def test_source_wrapper_delegates_once_and_preserves_original_exceptions(self):
        args=fixture();args['original']['source_sha256']={}
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);artifact=root/'artifact';obs=artifact/'observation';app=obs/'app-server'/'capture-a'
            app.mkdir(parents=True);sessions=root/'sessions';sessions.mkdir();paths=[]
            def save(path,value,rows=False):
                path.parent.mkdir(parents=True,exist_ok=True)
                path.write_text(''.join(json.dumps(v)+'\n' for v in value) if rows else json.dumps(value))
                paths.append(path)
            save(obs/'analysis.json',args['observed']);save(obs/'controller-usage.json',args['controller'])
            save(artifact/'final-state.json',args['state']);save(artifact/'trace.jsonl',args['trace'],True)
            save(obs/'process-invocations.jsonl',[{'capture_kind':'app-server','invocation_id':'capture-a'}],True)
            for name,key in [('client-to-server.raw','clients'),('client-to-server.forwarded.raw','forwarded'),('server-to-client.raw','servers')]:save(app/name,args['captures'][0][key],True)
            native=sessions/'native.jsonl';save(native,args['natives'][0]['rows'],True)
            save(obs/'rollout-metadata.jsonl',[{'thread_id':'thread-a','source_relative_path':'native.jsonl','source_sha256':hashlib.sha256(native.read_bytes()).hexdigest()}],True)
            helper=root/'accounting.py';save(helper,{})
            observer=root/'observer.rs';save(observer,{})
            paths.append(Path(Q.__file__).resolve())
            admission={'accounting_helper':str(helper),'observer_source':str(observer),
                'original_result_sha256':Q.digest(Q.canonical(args['original'])),
                'source_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                'original_source_qualification':{'original_membership_flag':'FAILED','execution_exception':'retained'}}
            entry={'artifact':str(artifact),'prompt_trace':str(artifact/'trace.jsonl'),'run_id':'run-a','condition':'control'}
            original=copy.deepcopy(args['original'])
            with patch.object(Q,'OBSERVER_SHA',admission['source_sha256'][str(observer)]),patch.object(Q,'load_original') as load:
                mock=unittest.mock.Mock(return_value=copy.deepcopy(original));load.return_value=types.SimpleNamespace(audit_run=mock)
                got=Q.audit_run(entry,{},str(sessions),admission)
            mock.assert_called_once_with(entry,{},str(sessions))
            self.assertEqual(got['qualification']['status'],'qualified_provider_ledger',got['qualification']['errors'])
            self.assertEqual(got['original'],original)
            self.assertEqual(got['original_source_qualification'],admission['original_source_qualification'])

    def test_identical_already_qualified_original_passes_without_scope_rewrite(self):
        args=fixture();u=args['original']['measurement']['recorded_usage'];o=args['observed'];r=o['controller_usage_reconciliation'][0]
        events=args['captures'][0]['servers'];events.insert(5,events.pop(3))
        args['controller']=[{'agent_id':'author-a','usage':u}];args['state']['snapshot']['sessions'][0]['token_usage']=u
        r.update(controller_streamed_usage=u,replayed_streamed_usage=u,controller_matches_replay=True)
        o.update(errors=[],capture_complete=True)
        original=args['original'];original.update(status='validated',errors=[])
        original['measurement'].update(status='exact',reasons=[],bounds={'raw_input_plus_output':{'lower':14,'upper':14},'uncached_input_plus_output':{'lower':12,'upper':12}})
        got=Q.qualify(**args)
        self.assertEqual(got['status'],'qualified_provider_ledger')
        self.assertEqual(got['measurement'],original['measurement'])
        self.assertEqual(got['controller_unavailable_agents'],[])


if __name__=='__main__':unittest.main()
