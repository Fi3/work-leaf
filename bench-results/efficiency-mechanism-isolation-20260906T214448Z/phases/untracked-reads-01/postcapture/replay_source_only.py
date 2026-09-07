#!/usr/bin/env python3
"""Read-only R source/delivery/native/path census; stdout only, never accounting.

Uses unchanged frozen pure functions. Usage records and private reasoning bodies
are omitted before native inventory. Retrieval semantics remain a separate review.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import sys
import types

PHASE = Path(__file__).resolve().parent.parent
STUDY = PHASE.parent.parent
FROZEN = PHASE/'infrastructure/evidence/bench-results'/STUDY.name
MANIFEST_SHA = '706c0b13d62376b3ea10c4604bb63957c271a1a7224d39b27db20fa24e2da263'
PINS = {'analyze_untracked_reads.py': '30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e',
        'audit_read_mechanism.py': '2ad81ed5dc27ccba621f21f087312f48ac26e7799073baa5da6130eb2f7119e9'}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require(value, why):
    if not value:
        raise ValueError(why)


def load(name):
    path = FROZEN/name
    data = path.read_bytes()
    require(sha(data) == PINS[name], 'frozen helper pin: '+name)
    module = types.ModuleType('retained_source_'+path.stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


D = load('analyze_untracked_reads.py')
C = load('audit_read_mechanism.py')


def json_source(index, path, expected=C.NO_EXPECTED_HASH):
    return C.decode(index.read(path, expected))


def safe(value, names):
    return {k: value.get(k) for k in names}


def ref(source, line, payload=None):
    row = {'path': str(source), 'line': line}
    if payload is not None:
        row['payload_sha256'] = sha(C.canonical(payload))
    return row


def initial():
    index = C.SourceIndex()
    index.read(Path(__file__).resolve())
    frozen = json_source(index, PHASE/'PHASE-MANIFEST.json', MANIFEST_SHA)
    require(len(frozen['files']) == 114, 'unexpected frozen population')
    for row in frozen['files']:
        index.read(row['path'], row['sha256'])
    for name in PINS:
        D.require_helper(frozen, FROZEN/name)
    score = json_source(index, PHASE/'score-manifest.json')
    result = json_source(index, PHASE/'PHASE-RESULT.json')
    require(score['phase_manifest_sha256'] == result['manifest_sha256'] == MANIFEST_SHA, 'phase parent pin')
    require([r['run_id'] for r in score['runs']] == [r['run_id'] for r in frozen['schedule']], 'score order')
    require(score['runs'] == result['runs'] and len(score['runs']) == 12, 'all terminal rows')
    for row, plan in zip(score['runs'], frozen['schedule']):
        for key in ('run_id', 'condition', 'artifact', 'report', 'prompt_trace', 'experiment_manifest', 'wave', 'block_id'):
            require(row[key] == plan[key], 'frozen row identity '+key)
    for row in score['runs'][:6]:
        receipt = json_source(index, PHASE/'logs'/(row['run_id']+'.exit.json'))
        require(receipt == row and row['launch_status'] == 'completed', 'exact terminal receipt')
    require(all(r['launch_status'] == 'not_launched_after_signal' and r['started_at'] is None and r['launcher_exit_code'] is None for r in score['runs'][6:]), 'unlaunched identities')
    return index, frozen, score, result


def phase_sources():
    index, frozen, score, result = initial()
    receipts = {}
    for name in ('OPERATOR-ADMISSION-GATES.json', 'OPERATOR-CONTROL-STOP.json', 'OPERATOR-HOLD-005-TRUST.json',
                 'OPERATOR-DRAIN-FINALIZATION.json', 'STOPPED-DESCRIPTIVE-SCOPE.json', 'trust-final.json'):
        value = json_source(index, PHASE/name)
        receipts[name] = {'path': str(PHASE/name), 'sha256': index.hashes[str(PHASE/name)]}
        if name == 'trust-final.json':
            receipts[name]['final_verdict'] = value['final_global_snapshot'].get('trust_classification')
            receipts[name]['legacy_flags_preserved'] = value['legacy_flags_preserved']
    for name in ('config-history.jsonl', 'config-history.json', 'trust-evidence.jsonl', 'trust-pending.jsonl'):
        index.read(PHASE/name)
    out = {'schema': 'retained-read-source-only-v1', 'scope': 'All twelve original identities, six actual workflows; no accounting or randomization',
        'manifest_sha256': MANIFEST_SHA, 'frozen_file_count': len(frozen['files']),
        'original_phase_flags': {k:v for k,v in result.items() if k != 'runs'},
        'runs': score['runs'], 'receipts': receipts,
        'limitations': ['Original paused-monitoring interval and final unexplained config drift are not waived by source hashes.',
            'All original observer gaps remain; source delivery proof is not whole-workflow accounting.',
            'Initial schema inspection incidentally printed one existing thread/native usage row; no totals, ranking or comparison was calculated.']}
    index.verify(); out['source_sha256'] = index.hashes
    return out


def run_sources(number):
    index, frozen, score, phase_result = initial()
    require(1 <= number <= 6, 'only six actually launched workflows')
    row = score['runs'][number-1]; rid = row['run_id']; obs = Path(row['artifact'])/'observation'
    analysis = json_source(index, obs/'analysis.json')
    threads = [safe(v, ('thread_id', 'agent_id', 'invocation_id', 'primary', 'visible', 'role')) for v in analysis['threads']]
    ownership = {r['thread_id']: r['agent_id'] for r in threads}
    require(len(ownership) == len(threads), 'duplicate observer thread')
    invocations = index.records(obs/'process-invocations.jsonl')
    apps = {v['invocation_id']:v for _,v in invocations if v['capture_kind'] == 'app-server'}
    actual = {p.name for p in (obs/'app-server').iterdir() if p.is_dir()}
    require(actual == set(apps), 'complete app capture census')
    captures = []; proof = []; accepted = []; captured_threads = {}; public_users = {}; terminals = {}
    for app_id in sorted(apps):
        app = obs/'app-server'/app_id; iv = obs/'invocations'/app_id
        start = json_source(index, iv/'start.json'); end = json_source(index, iv/'end.json'); child = json_source(index, iv/'child.json')
        require(all(end.get(k) == v for k,v in apps[app_id]['end'].items()), 'invocation end index differs')
        require(start['primary'] is True and start['raw_response_usage'] is True and start['provider_usage_grace_ms'] == 1000 and start['provider_usage_grace_output_resume'] == 'forward', 'original primary/raw/grace settings')
        for name in ('raw-response-usage.json', 'raw-response-rewrites.jsonl', 'provider-usage-grace.jsonl'):
            index.read(app/name)
        clients = index.records(app/'client-to-server.raw'); servers = index.records(app/'server-to-client.raw'); forwarded = index.records(app/'client-to-server.forwarded.raw')
        p = D.capture_provenance(app)
        for path, expected in p['source_sha256'].items(): index.read(path, expected)
        proof.append({'capture':str(app), 'original_frozen_provenance':p, 'settings':safe(start, ('primary','raw_response_usage','provider_usage_grace_ms','provider_usage_grace_output_resume')), 'child':child})
        captures.append({'path':str(app), 'clients':[v for _,v in clients], 'client_lines':[n for n,_ in clients], 'servers':[v for _,v in servers], 'server_lines':[n for n,_ in servers]})
        replies = {}; forwarded_rpc = {}
        for n,v in servers:
            if 'method' not in v and 'id' in v:
                key = D.rpc(v); require(key not in replies, 'duplicate RPC reply'); replies[key] = (n,v)
            if v.get('method') == 'item/completed' and v['params']['item'].get('type') == 'userMessage':
                q = v['params']; key = (q['threadId'],q['turnId']); require(key not in public_users, 'duplicate public user item'); public_users[key] = (app,n,q['item'])
            if v.get('method') == 'turn/completed':
                q = v['params']; terminals[(q['threadId'],q['turn']['id'])] = {'source':str(app/'server-to-client.raw'),'line':n,'status':q['turn'].get('status')}
        for n,v in forwarded:
            if 'id' in v:
                key=D.rpc(v); require(key not in forwarded_rpc, 'duplicate forwarded RPC'); forwarded_rpc[key]=(n,v)
        for n,v in clients:
            if v.get('method') not in ('thread/start','turn/start'): continue
            key=D.rpc(v); reply_line, reply = replies.get(key,(None,{}))
            require(not ('result' in reply and 'error' in reply), 'mixed typed reply')
            if 'result' not in reply: continue
            require(key in forwarded_rpc, 'missing forwarded accepted request')
            if v['method'] == 'thread/start':
                tid=reply['result']['thread']['id']; require(D.identity(tid) and tid not in captured_threads, 'duplicate accepted thread')
                captured_threads[tid]={'capture':str(app),'client_line':n,'reply_line':reply_line,'rpc_id':v['id'],'cwd':v['params']['cwd'],
                    'model':v['params'].get('model'),'effort':(v['params'].get('config') or {}).get('model_reasoning_effort'),'sandbox':v['params'].get('sandbox'),'approval_policy':v['params'].get('approvalPolicy')}
                continue
            require(forwarded_rpc[key][1] == v, 'changed accepted turn bytes')
            tid=v['params']['threadId']; turn=reply['result']['turn']['id']; inputs=v['params']['input']
            require(D.identity(tid) and D.identity(turn) and len(inputs)==1 and inputs[0]['type']=='text' and isinstance(inputs[0]['text'],str), 'unsupported accepted input')
            pub=public_users.get((tid,turn)); require(pub is not None,'missing actual public user item')
            content=pub[2]['content']; require(len(content)==1 and content[0].get('type')=='text' and content[0].get('text')==inputs[0]['text'], 'public full text differs')
            require(all(not x for k,x in content[0].items() if k not in ('type','text')), 'unsupported extra public input metadata')
            accepted.append({'capture':str(app),'client_line':n,'forwarded_line':forwarded_rpc[key][0],'reply_line':reply_line,'rpc_id':v['id'],'thread_id':tid,'turn_id':turn,
                'text_sha256':sha(inputs[0]['text'].encode()),'text_bytes':len(inputs[0]['text'].encode()),'public_item_id':pub[2]['id'],'public_line':pub[1], '_text':inputs[0]['text']})
    require(set(captured_threads)==set(ownership),'all captured threads versus observer ownership')
    require(len({(x['thread_id'],x['turn_id']) for x in accepted})==len(accepted)==len(public_users),'bidirectional accepted/public census')
    trace_pairs=index.records(row['prompt_trace']); trace=[v for _,v in trace_pairs]; trace_lines=[n for n,_ in trace_pairs]
    inventory=D.prompt_inventory(trace, threads, captures, rid, row['condition'], trace_lines)
    metadata=index.records(obs/'rollout-metadata.jsonl'); natives=[]; native_contexts=[]; usage_presence={}; native_by_source={}
    require(len({v['thread_id'] for _,v in metadata})==len(metadata) and {v['thread_id'] for _,v in metadata}==set(captured_threads),'all native threads incl usage-less')
    for meta_line,m in metadata:
        tid=m['thread_id']; path=Path('/home/user/.codex/sessions')/m['source_relative_path']; rows=index.records(path,m['source_sha256'])
        session=[(n,v['payload']) for n,v in rows if v.get('type')=='session_meta']; require(len(session)==1,'native session metadata cardinality'); n,sm=session[0]
        require(sm['id']==tid and sm['cwd']==captured_threads[tid]['cwd'] and sm['cli_version']=='0.153.4','native session model scope')
        contexts=[]; public=[]; usage_presence[tid]=False
        for n,v in rows:
            kind=v.get('type'); payload=v.get('payload',{})
            if kind=='token_usage_record': usage_presence[tid]=True; continue
            if kind=='turn_context':
                require(payload.get('model')=='gpt-5.5' and payload.get('effort')=='xhigh' and payload.get('cwd')==sm['cwd'],'native effective model/effort/cwd')
                contexts.append({**ref(path,n),'turn_id':payload.get('turn_id'),'model':payload.get('model'),'effort':payload.get('effort'),'cwd':payload.get('cwd'),'approval_policy':payload.get('approval_policy'),'sandbox_policy':payload.get('sandbox_policy')})
                public.append((n,v))
            elif kind=='response_item' and payload.get('type')!='reasoning': public.append((n,v))
        natives.append({'source':str(path),'thread_id':tid,'rows':public}); native_by_source[str(path)]={n:v for n,v in public}
        native_contexts.append({'thread_id':tid,'source':str(path),'metadata_line':meta_line,'session_meta':{**ref(path,session[0][0]),**safe(sm,('id','cwd','cli_version','model_provider'))},'contexts':contexts})
    native=C.native_inventory(natives); require(not native['ledger'],'usage records must not enter source-only native inventory')
    used=set()
    for a in accepted:
        key=(a['thread_id'],a['turn_id'],a['text_sha256']); group=native['users'].get(key)
        require(group is not None,'exact explicit native input absent'); choices=native['user_groups'][group]['candidates']
        require(len(choices)==1 and native['user_texts'][group]==a.pop('_text'),'exact native fulltext/cardinality')
        item=choices[0]; unique=(item['thread_id'],item['item_id']);require(unique not in used,'native input reuse');used.add(unique)
        payload=native_by_source[item['source']][item['line']]['payload']; require(payload['internal_chat_message_metadata_passthrough'].get('content_item_kinds')==['user.text'],'native authored input kind')
        a['native_user']=item; a['terminal']=terminals.get((a['thread_id'],a['turn_id']))
    authored={(g['thread_id'],v['item_id']) for g in native['user_groups'] for v in g['candidates'] if native_by_source[v['source']][v['line']]['payload'].get('internal_chat_message_metadata_passthrough',{}).get('content_item_kinds')==['user.text']}
    require(authored==used,'bidirectional native user.text coverage')
    by_turn={(a['thread_id'],a['turn_id']):a for a in accepted}; exposures={e['trace_line']:e for e in inventory['exposures']}
    archives={}; archive_pairs=index.records(obs/'context-bundles/manifest.jsonl')
    for n,v in archive_pairs:
        require(v.get('parse_error') is None and v['source'] not in archives,'invalid/duplicate archived bundle')
        p=obs/'context-bundles'/v['archived_path']; body=index.read(p,v['sha256']);require(len(body)==v['bytes'],'archive byte count'); archives[v['source']]={'record':v,'path':str(p),'line':n,'body':body}
    reads=[]; issued={}; bundle_paths=set()
    for n,e in trace_pairs:
        if e.get('event')!='read-response':continue
        x=exposures.get(n,{}); read=C.safe_read(e,x,n); read.pop('charge_refs',None);read.pop('same_turn_response_indices',None)
        if x.get('status')=='delivered': read['native_user']=by_turn[(x['thread_id'],x['turn_id'])]['native_user']
        bundle=e['bundle'].get('path')
        if bundle:
            bundle_paths.add(bundle); require(bundle in archives,'issued trace bundle missing from archive'); ar=archives[bundle]
            candidate=e['inline_candidate_prompt'].encode()
            pieces=[b'# Work Leaf Context Bundle\n\nThis file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.\n']
            for snapshot in e['snapshots']:
                if snapshot['class']!='untracked':continue
                body=candidate[snapshot['inline_body_start']:snapshot['inline_body_end']]
                pieces.extend([('\n----- BEGIN FILE '+snapshot['path']+' -----\ndigest: '+snapshot['digest']+'\n\n').encode(),body,b'' if body.endswith(b'\n') else b'\n',('----- END FILE '+snapshot['path']+' -----\n').encode()])
            require(b''.join(pieces)==ar['body'],'archived bundle differs from source-owned formatter and held snapshot bytes')
            read['archive']={'path':ar['path'],'manifest_line':ar['line'],'bytes':len(ar['body']),'sha256':sha(ar['body']),'exact_held_snapshot_body_and_formatter':True}
            if x.get('status')=='delivered' and e['selected_candidate']=='baseline' and e['eligible']:
                key=(x['thread_id'],bundle); require(key not in issued,'bundle issued multiple times to same thread');issued[key]=len(reads)
        reads.append(read)
    calls=native['calls']; outputs=native['outputs']; outmap=defaultdict(list);callmap=defaultdict(list)
    for i,o in enumerate(outputs):outmap[(o['thread_id'],o.get('call_id'))].append(i)
    for i,c in enumerate(calls):callmap[(c['thread_id'],c.get('call_id'))].append(i)
    matcher=C.PatternIndex(bundle_paths); candidates=[]
    for i,c in enumerate(calls):
        matches=list(matcher.matches(native['call_inputs'].get(i,b'')))
        if not matches:continue
        key=(c['thread_id'],c.get('call_id')); outs=outmap.get(key,[]); c=dict(c);c['output_indices']=outs;c['output_link_status']='linked' if len(outs)==1 and len(callmap[key])==1 else 'unresolved'
        references=[]
        for begin,end,bundle in matches:
            read_index=issued.get((c['thread_id'],bundle)); association='not_issued_to_this_thread'
            if read_index is not None:
                u=reads[read_index]['native_user']; association='after_unique_issued_input' if u.get('source')==c['source'] and u.get('line',10**30)<c['line'] else 'not_after_issued_input'
            references.append({'path':bundle,'argument_byte_start':begin,'argument_byte_end':end,'read_index':read_index,'association':association,'actual_open':'not_proven_by_substring'})
        c['bundle_references']=references;c['outputs']=[outputs[j] for j in outs];c['retrieval_semantics']='pending_source_review';candidates.append(c)
    last={}
    for a in accepted:last[a['thread_id']]=a
    out={'schema':'retained-read-run-source-only-v1','run':row,'original_observer_flags':safe(analysis,('capture_complete','errors','interrupted_provider_turns')),
        'original_delivery_inventory':inventory,'capture_provenance':proof,'threads':threads,'captured_thread_starts':captured_threads,'native_contexts':native_contexts,
        'native_usage_record_present':usage_presence,'native_usage_fields_read_or_exported':False,'accepted_inputs':accepted,
        'last_accepted_turns':[safe(a,('thread_id','turn_id','terminal')) for a in last.values()], 'reads':reads,'bundle_path_candidates':candidates,
        'native_tool_call_count':len(calls),'native_tool_output_count':len(outputs),'native_unknown_public_items':native['unknown'],
        'native_tool_inventory_sha256':sha(C.canonical({'calls':calls,'outputs':outputs})),
        'checks':{'all_captured_threads_native_matched':True,'all_accepted_inputs_native_exact_explicit_turn_and_fulltext':True,'all_user_text_items_consumed_once':True,
            'archive_count':len(archives),'trace_read_count':len(reads),'candidate_call_count':len(candidates),'all_frozen_input_pins':len(frozen['files'])},
        'not_claimed':['Continuous config integrity','Complete token accounting','Full-phase randomized inference','Retrieval from path substring alone','Absence of indirect or obfuscated retrieval']}
    index.verify();out['source_sha256']=index.hashes
    return out


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--run',type=int);args=parser.parse_args()
    value=phase_sources() if args.run is None else run_sources(args.run)
    print(json.dumps(value,sort_keys=True,indent=2,allow_nan=False))
