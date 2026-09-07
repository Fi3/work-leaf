#!/usr/bin/env python3
"""Separate provider-ledger qualification; original controller flags stay intact.

The pure function is not an admission or source verifier. The source-bound wrapper
executes the unchanged pinned accounting helper before supplying its result.
No original analysis boolean, controller row, response or usage value is replaced.
"""
from collections import Counter, defaultdict
import copy
import hashlib
import json
from pathlib import Path
import stat
import types

FIELDS = ('input_tokens','cached_input_tokens','output_tokens','reasoning_output_tokens')
CAMEL = ('inputTokens','cachedInputTokens','outputTokens','reasoningOutputTokens')
METRICS = ('raw_input_plus_output','uncached_input_plus_output')


def require(value, message):
    if not value: raise ValueError(message)


def identity(value):
    require(type(value) is str and bool(value), 'identity must be a nonempty string')
    return value


def integer(value):
    require(type(value) is int and value >= 0, 'counter must be a nonnegative integer')
    return value


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()


def counters(value, camel=False):
    require(isinstance(value,dict),'usage object required')
    counts=tuple(integer(value.get(k)) for k in (CAMEL if camel else FIELDS))
    i,c,o,r=counts
    require(c<=i and r<=o,'usage subset exceeds parent')
    return counts


def rpc(value):
    key=value.get('id')
    require(type(key) is int or type(key) is str and bool(key),'typed RPC identity required')
    return type(key).__name__,key


def complete_directive(text):
    """Supported exact observer grammar; strings are never exported as evidence."""
    return advance_directives(text,False)[1]


def advance_directives(text,in_patch):
    for line in text.splitlines():
        line=line.lstrip()
        if not line.startswith('@work-leaf'):continue
        rest=line[len('@work-leaf'):]
        if not rest or not rest[0].isspace():continue
        body=rest.strip()
        if in_patch:
            if body=='end':return in_patch,True
            continue
        if body=='done' or any(directive_rest(body,prefix) for prefix in ('read','locks run','locks classify','send')):return in_patch,True
        if any(directive_rest(body,prefix) for prefix in ('patch','edit')):in_patch=True
    return in_patch,False


def directive_rest(body,prefix):
    return body.startswith(prefix) and (len(body)==len(prefix) or body[len(prefix)].isspace())


def single_text(content, native=False):
    require(isinstance(content,list) and len(content)==1 and isinstance(content[0],dict),'single text input required')
    item=content[0]
    require(item.get('type') in (('input_text','text') if native else ('text',)) and isinstance(item.get('text'),str),'unsupported text input')
    require(all(not v for k,v in item.items() if k not in ('type','text')),'unsupported additional input metadata')
    return item['text']


def evidence(observed,controller,state,trace,captures,natives,run_id,condition,expected):
    """Indexed exact delivery/ownership and pre-directive controller replay."""
    identity(run_id);identity(condition)
    threads={}
    for row in observed['threads']:
        tid=identity(row.get('thread_id'));agent=identity(row.get('agent_id'))
        require(tid not in threads and row.get('primary') is True and type(row.get('visible')) is bool,'unsupported observed thread scope')
        threads[tid]=row
    require(threads and not observed.get('session_only_threads'),'incomplete observed thread scope')
    require(trace and isinstance(trace[0],dict),'trace activation required')
    schema=trace[0].get('schema');pid=trace[0].get('process_id')
    require(schema in ('work-leaf-bench-experiment-v2','work-leaf-bench-experiment-v3') and integer(pid)>0,'unsupported ownership trace')
    policies={}
    for n,row in enumerate(trace):
        require(row.get('schema')==schema and row.get('run_id')==run_id and row.get('condition')==condition and type(row.get('process_id')) is int and row['process_id']==pid,'trace identity differs')
        if n==0:
            require(row.get('event')=='activation','initial activation required');continue
        require(row.get('event')!='activation' and type(row.get('sequence')) is int and row['sequence']==n,'noncontiguous trace')
        if row.get('site')!='policy-injection':continue
        agent=identity(row.get('agent_id'));text=row.get('forwarded_prompt')
        require(row.get('event')=='prompt' and isinstance(text,str) and row.get('original_prompt')==text and row.get('changed') is False and type(row.get('byte_delta')) is int and row['byte_delta']==0,'nonidentity launch policy')
        require(row.get('original_bytes')==row.get('forwarded_bytes')==len(text.encode()),'policy byte count differs')
        require(agent not in policies and '\n\nAgent-ID: '+agent+'\nFeature: ' in text,'ambiguous owned agent policy')
        policies[agent]=(text,n+1)
    accepted={};starts={};raw={};turns=defaultdict(lambda:{'messages':[],'usage':[],'responses':[]})
    public={};terminals={};seen_captures=set()
    for cap in captures:
        path=identity(cap['path']);require(path not in seen_captures,'duplicate capture');seen_captures.add(path)
        replies={};forwarded={};seen_requests=set()
        for line,v in enumerate(cap['forwarded'],1):
            if 'id' not in v:continue
            key=rpc(v);require(key not in forwarded,'duplicate forwarded RPC');forwarded[key]=(line,v)
        for line,v in enumerate(cap['servers'],1):
            if 'id' in v and 'method' not in v:
                key=rpc(v);require(key not in replies,'duplicate typed reply');replies[key]=(line,v)
            p=v.get('params') or {};m=v.get('method')
            if m=='item/completed' and (p.get('item') or {}).get('type')=='userMessage':
                k=identity(p.get('threadId')),identity(p.get('turnId'));item=p['item']
                require(k not in public,'duplicate public user input');public[k]=(identity(item.get('id')),single_text(item.get('content')),line)
            if m=='turn/completed':
                k=identity(p.get('threadId')),identity((p.get('turn') or {}).get('id'))
                require(k not in terminals and p['turn'].get('status') in ('completed','interrupted','failed'),'unsupported terminal')
                terminals[k]=(line,p['turn']['status'])
            if m not in ('item/completed','thread/tokenUsage/updated','rawResponse/completed'):continue
            if not p.get('threadId') or not p.get('turnId'):continue
            k=identity(p['threadId']),identity(p['turnId'])
            if m=='item/completed' and (p.get('item') or {}).get('type')=='agentMessage':
                text=p['item'].get('text');require(isinstance(text,str),'unknown public assistant text')
                turns[k]['messages'].append((line,text))
            elif m=='thread/tokenUsage/updated':
                turns[k]['usage'].append((line,counters(p['tokenUsage']['last'],True)))
            elif m=='rawResponse/completed':
                rid=identity(p.get('responseId'));entry=(k,counters(p.get('usage'),True))
                require(rid not in raw or raw[rid][0]==entry,'conflicting raw response')
                raw[rid]=(entry,line);turns[k]['responses'].append((rid,line))
        for line,v in enumerate(cap['clients'],1):
            if v.get('method') not in ('thread/start','turn/start'):continue
            key=rpc(v);require(key not in seen_requests,'duplicate request');seen_requests.add(key)
            reply_line,reply=replies.get(key,(None,{}))
            require('result' in reply and 'error' not in reply,'missing/rejected/mixed typed acceptance')
            require(key in forwarded,'accepted request not forwarded')
            if v['method']=='thread/start':
                tid=identity(reply['result']['thread']['id']);require(tid not in starts,'duplicate accepted thread')
                starts[tid]={'capture':path,'client_line':line,'reply_line':reply_line,'cwd':identity(v['params'].get('cwd'))}
                continue
            require(forwarded[key][1]==v,'forwarded turn differs')
            k=identity(v['params'].get('threadId')),identity(reply['result']['turn'].get('id'))
            require(k not in accepted and k[0] in threads,'duplicate/unknown accepted turn')
            text=single_text(v['params'].get('input'));pub=public.get(k)
            require(pub is not None and pub[1]==text,'public input differs')
            accepted[k]={'capture':path,'client_line':line,'reply_line':reply_line,'rpc_id':v['id'],'text':text,
                         'public_item_id':pub[0],'public_line':pub[2]}
    require(set(starts)==set(threads),'accepted thread census differs')
    require(set(accepted)==set(public)==set(terminals),'accepted/public/terminal census differs')
    require(all(k in accepted for k in turns),'event outside accepted turn scope')
    accepted_by_thread=defaultdict(set)
    for k in accepted:accepted_by_thread[k[0]].add(k)
    native_users={};native_responses={};native_threads=set();native_item_ids=set()
    for source in natives:
        tid=identity(source['thread_id']);path=identity(source['source'])
        require(tid not in native_threads and tid in starts,'native thread census differs');native_threads.add(tid)
        sessions=[];contexts=set()
        for line,v in enumerate(source['rows'],1):
            kind=v.get('type');p=v.get('payload') or {}
            if kind=='session_meta':sessions.append(p)
            if kind=='turn_context':
                turn=identity(p.get('turn_id'));contexts.add((tid,turn))
                require(p.get('model')=='gpt-5.5' and p.get('effort')=='xhigh' and p.get('cwd')==starts[tid]['cwd'],'native effective context differs')
            if kind=='token_usage_record':
                rid=identity(p.get('response_id'));k=identity(p.get('thread_id')),identity(p.get('turn_id'))
                require(k[0]==tid and k in accepted,'native response scope differs');value=(k,counters(p.get('usage')))
                require(rid not in native_responses or native_responses[rid][0]==value,'conflicting native response')
                native_responses[rid]=(value,path,line)
            if kind!='response_item' or p.get('type')!='message' or p.get('role')!='user':continue
            meta=p.get('internal_chat_message_metadata_passthrough') or {}
            require(isinstance(meta,dict),'invalid native metadata')
            if meta.get('content_item_kinds')!=['user.text']:continue
            direct=p.get('turn_id');nested=meta.get('turn_id')
            require(direct is None or type(direct) is str and direct,'invalid direct native turn')
            require(nested is None or type(nested) is str and nested,'invalid nested native turn')
            require(not(direct and nested and direct!=nested),'contradictory native turn')
            k=tid,identity(direct or nested);item_id=identity(p.get('id'))
            require(k not in native_users and (tid,item_id) not in native_item_ids,'duplicate native user identity')
            native_item_ids.add((tid,item_id));native_users[k]=(single_text(p.get('content'),True),path,line,item_id)
        require(len(sessions)==1 and sessions[0].get('id')==tid and sessions[0].get('cwd')==starts[tid]['cwd'] and sessions[0].get('cli_version')=='0.153.4','native session identity differs')
        require(contexts==accepted_by_thread[tid],'native turn context census differs')
    require(native_threads==set(threads) and set(native_users)==set(accepted),'native delivery census differs')
    require(set(raw)==set(native_responses)==set(expected),'raw/native/original response ID census differs')
    for rid,(value,line) in raw.items():
        require(native_responses[rid][0]==value,'raw/native four-field identity differs')
        other=expected[rid];require(value==((identity(other.get('thread_id')),identity(other.get('turn_id'))),counters(other.get('usage'))),'original response record differs')
    owners={};inputs=[];first={}
    for k,a in accepted.items():
        n=native_users[k];require(n[0]==a['text'],'native full input differs')
        first.setdefault(k[0],a)
        inputs.append({k2:v for k2,v in a.items() if k2!='text'}|{'thread_id':k[0],'turn_id':k[1],
            'text_sha256':digest(a['text'].encode()),'native_source':n[1],'native_line':n[2],'native_item_id':n[3],
            'terminal_line':terminals[k][0],'terminal_status':terminals[k][1]})
    require({r['agent_id'] for r in threads.values()}==set(policies),'policy agent census differs')
    for tid,row in threads.items():
        agent=row['agent_id'];text,line=policies[agent]
        require(first[tid]['text']==text,'owned policy is not first accepted input')
        owners[tid]={'agent_id':agent,'policy_trace_line':line,**starts[tid]}
    sessions={};control={};reconciliation={}
    for row in state['snapshot']['sessions']:
        agent=identity(row.get('id'));require(agent not in sessions,'duplicate controller state identity');sessions[agent]=row.get('token_usage')
    for row in controller:
        agent=identity(row.get('agent_id'));require(agent not in control,'duplicate controller usage row');control[agent]=counters(row.get('usage'))
    for row in observed['controller_usage_reconciliation']:
        agent=identity(row.get('agent_id'));require(agent not in reconciliation,'duplicate controller reconciliation');reconciliation[agent]=row
    visible=defaultdict(set)
    for tid,row in threads.items():
        if row['visible']:visible[row['agent_id']].add(tid)
    require(set(sessions)==set(visible) and set(reconciliation)==set(visible),'controller visible scope differs')
    require(set(control)<=set(visible),'controller usage has unknown agent')
    missing_set=set(visible)-set(control)
    missing=sorted(missing_set);unavailable=[];unavailable_keys=set();preusage=defaultdict(list)
    for k,t in turns.items():
        in_patch=False;boundary=None
        for line,message in t['messages']:
            in_patch,complete=advance_directives(message,in_patch)
            if complete:boundary=line;break
        agent=threads[k[0]]['agent_id']
        before=[u for line,u in t['usage'] if boundary is None or line<boundary]
        preusage[agent].extend(before)
        if agent in missing_set:
            require(boundary is not None and not before and t['usage'] and t['responses'],'missing controller is not explained by directive-before-usage')
            unavailable_keys.add(k)
            unavailable.append({'agent_id':agent,'thread_id':k[0],'turn_id':k[1],'directive_line':boundary,
                'usage_lines':[line for line,u in t['usage']],
                'responses':[{'response_id':rid,'server_line':line,'native_source':native_responses[rid][1],'native_line':native_responses[rid][2]} for rid,line in t['responses']]})
    require(unavailable_keys=={k for k in accepted if threads[k[0]]['agent_id'] in missing_set},
            'every accepted turn of an unavailable controller needs a directive-before-usage witness')
    for agent,tids in visible.items():
        row=reconciliation[agent]
        require(set(row['provider_thread_ids'])==tids,'controller provider thread scope differs')
        if agent in missing_set:
            require(sessions[agent] is None and row.get('controller_streamed_usage') is None and row.get('replayed_streamed_usage') is None and row.get('controller_matches_replay') is False,'controller absence not independently consistent')
            require(not preusage[agent],'absent controller has actual streamed usage')
        else:
            require(preusage[agent],'controller has no replayable usage')
            replay=tuple(sum(v[i] for v in preusage[agent]) for i in range(4))
            require(control[agent]==counters(sessions[agent])==counters(row.get('controller_streamed_usage'))==counters(row.get('replayed_streamed_usage'))==replay and row.get('controller_matches_replay') is True,'controller replay differs')
    return {'ownership':owners,'accepted_inputs':inputs,'unavailable_turns':unavailable,'controller_unavailable_agents':missing,
            'exact_completed_response_ids':sorted(raw),'native_thread_ids':sorted(native_threads)}


def qualify(original,observed,controller,state,trace,captures,natives,run_id,condition):
    out={'status':'unknown','errors':[],'measurement':{'status':'ineligible','bounds':None},
         'original_result_sha256':digest(canonical(original)),'original_errors':copy.deepcopy(original.get('errors')),
         'original_capture_complete':observed.get('capture_complete'),'controller_unavailable_agents':[],
         'controller_unavailability_is_zero_usage':False,'whole_workflow_hidden_call_completeness_proven':False}
    try:
        require(original.get('corrected_scope') is not None,'original response reconciliation did not finish')
        m=original['measurement'];inv=m['gap_inventory'];require(not inv['errors'],'original gap inventory errors')
        require(set(inv['raw_responses'])==set(original['exact_response_evidence']),'original response inventories disagree')
        proof=evidence(observed,controller,state,trace,captures,natives,run_id,condition,original['exact_response_evidence'])
        out['evidence']=proof;missing=proof['controller_unavailable_agents'];out['controller_unavailable_agents']=missing
        missing_count=integer(observed.get('interrupted_provider_turns'));gap_error=f'interrupted provider turn has no complete usage: count={missing_count}'
        allowed=[f'visible provider agent {agent} has no controller usage row' for agent in missing]
        require(Counter(observed.get('errors',[]))==Counter(allowed+([gap_error] if missing_count else [])),'unrelated/missing observer diagnostics')
        require(observed.get('capture_complete') is (not observed['errors']),'original observer completeness semantics differ')
        require(len(inv['gaps'])==missing_count,'strict tail inventory count differs')
        require(observed.get('invocation_count')==observed.get('complete_invocation_count') and integer(observed['invocation_count'])>0,'unfinished invocation inventory')
        policy=m['bound_policy'];require(policy.get('model')=='gpt-5.5' and policy.get('reasoning_effort')=='xhigh','accounting model policy differs')
        require(policy.get('per_response',{}).get('input_upper')==1050000 and policy['per_response'].get('output_upper')==128000,'declared response ceiling differs')
        measured=m['recorded_usage'];i,c,o,r=counters(measured)
        require(counters(original['corrected_scope']['usage'])==(i,c,o,r) and measured.get(METRICS[0])==i+o and measured.get(METRICS[1])==i-c+o,'original measured scope differs')
        if missing:
            expected=allowed+(['capture status is inconsistent'] if missing_count==0 else [])
            require(original.get('status')=='unknown' and Counter(original.get('errors',[]))==Counter(expected) and m.get('status')=='ineligible' and Counter(m.get('reasons',[]))==Counter(expected),'unrelated original accounting rejection')
            finite=all(g.get('response_count_upper')==1 and type(g.get('response_count_upper')) is int and g.get('proof')=='isolated_normal_response_tail' for g in inv['gaps'])
            result=copy.deepcopy(m)
            result.update(status='exact' if not missing_count else 'bounded' if finite else 'unbounded_accounting_gap',
                bounds={name:{'lower':measured[name],'upper':measured[name]+missing_count*1178000 if finite else None} for name in METRICS},
                reasons=[],controller_crosscheck='unavailable for precisely evidenced agents; original diagnostic retained')
        else:
            require(original.get('status')=='validated' and not original.get('errors') and m.get('status') in ('exact','bounded','unbounded_accounting_gap') and not m.get('reasons'),'original accounting is not qualified')
            result=copy.deepcopy(m)
        out.update(status='qualified_provider_ledger',measurement=result)
    except (ValueError,KeyError,TypeError,IndexError,AttributeError) as error:out['errors'].append(str(error))
    return out


ACCOUNTING_SHA='c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a'
OBSERVER_SHA='ab7ede87ca9162d87b4e00d2f42d8e2798a2a10d8543d5aaeec69d032b814af8'
MAX_FILE=1024**3
MAX_TOTAL=32*1024**3


def decode(data):
    def pairs(values):
        result={}
        for key,value in values:
            require(key not in result,'duplicate JSON object key');result[key]=value
        return result
    def nonfinite(value):raise ValueError('nonfinite JSON value')
    return json.loads(data,object_pairs_hook=pairs,parse_constant=nonfinite)


class SourceIndex:
    def __init__(self,pins):
        require(isinstance(pins,dict) and 0<len(pins)<=20000,'invalid admitted source census')
        self.pins=pins;self.hashes={};self.bytes=0

    def read(self,value):
        path=Path(value)
        require(path.is_absolute() and str(path)==str(value) and path.resolve()==path and stat.S_ISREG(path.lstat().st_mode),'canonical regular source required')
        expected=self.pins.get(str(path))
        require(type(expected) is str and len(expected)==64 and all(x in '0123456789abcdef' for x in expected),'source lacks admitted SHA-256')
        size=path.stat().st_size
        require(size<=MAX_FILE and self.bytes+size<=MAX_TOTAL,'source byte bound exceeded')
        self.bytes+=size;data=path.read_bytes()
        require(len(data)==size and digest(data)==expected,'admitted source hash differs')
        self.hashes[str(path)]=expected;return data

    def json(self,path):return decode(self.read(path))

    def rows(self,path):
        data=self.read(path)
        require(not data or data.endswith(b'\n'),'closed JSONL has incomplete tail')
        values=[decode(line) if line.strip() else {} for line in data.splitlines()]
        require(all(isinstance(v,dict) for v in values),'JSONL object required')
        return values

    def verify(self):
        for path in self.pins:self.read(path)


def load_original(path,index):
    path=Path(path);data=index.read(path)
    require(digest(data)==ACCOUNTING_SHA,'only unchanged accounting helper is supported')
    here=path.parent
    dependencies={here.parents[1]/'bench-results/efficiency-measurement-gate-20260906/batch_analysis.py':'dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3',
        here/'analyze.py':'2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78',
        here/'audit_compaction.py':'dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153'}
    for dependency,expected in dependencies.items():
        require(digest(index.read(dependency))==expected,'admitted dependency identity differs')
    # Its own exact-byte dependency loader performs all three pinned imports.
    module=types.ModuleType('provider_ledger_original_accounting');module.__file__=str(path)
    exec(compile(data,str(path),'exec'),module.__dict__)
    return module


def audit_run(entry,frozen,sessions_root,admission):
    """Execute only after a separately pinned fixed-population supplement admits it.

    admission supplies accounting_helper, observer_source, original_result_sha256,
    source_sha256, and original_source_qualification. The last field retains the
    original source-membership/execution exceptions; it does not clear them.
    Return both the complete unchanged original and a separate qualification.
    """
    output={'schema':'work-leaf-provider-ledger-qualification-v1','original':None,
        'qualification':{'status':'unknown','errors':[],'measurement':{'status':'ineligible','bounds':None}},
        'original_source_qualification':copy.deepcopy(admission.get('original_source_qualification')),
        'source_sha256':{},'provider_calls':0,'phase_integrity_flags_cleared':False}
    index=None
    try:
        index=SourceIndex(admission['source_sha256']);index.verify()
        require(isinstance(output['original_source_qualification'],dict),'original source qualifications must be retained')
        index.read(Path(__file__).resolve())
        require(digest(index.read(admission['observer_source']))==OBSERVER_SHA,'controller grammar source identity differs')
        helper=Path(admission['accounting_helper'])
        original_module=load_original(helper,index)
        original=original_module.audit_run(entry,frozen,sessions_root);output['original']=original
        require(digest(canonical(original))==admission['original_result_sha256'],'fresh original differs from declared original result')
        for path,expected in original['source_sha256'].items():
            require(index.pins.get(path)==expected,'original consumed source lacks admitted pin');index.read(path)
        artifact=Path(entry['artifact']);observation=artifact/'observation'
        observed=index.json(observation/'analysis.json');controller=index.json(observation/'controller-usage.json')
        state=index.json(artifact/'final-state.json');trace=index.rows(entry['prompt_trace'])
        invocations=index.rows(observation/'process-invocations.jsonl')
        apps={}
        for row in invocations:
            if row.get('capture_kind')!='app-server':continue
            app_id=identity(row.get('invocation_id'));require(app_id not in apps,'duplicate app-server invocation')
            apps[app_id]=row
        actual={p.name for p in (observation/'app-server').iterdir() if p.is_dir()}
        require(actual==set(apps),'process/app-server census differs')
        captures=[]
        for app_id in sorted(apps):
            app=observation/'app-server'/app_id
            captures.append({'path':str(app),'clients':index.rows(app/'client-to-server.raw'),
                'forwarded':index.rows(app/'client-to-server.forwarded.raw'),'servers':index.rows(app/'server-to-client.raw')})
        natives=[];root=Path(sessions_root).resolve()
        for meta in index.rows(observation/'rollout-metadata.jsonl'):
            relative=Path(meta['source_relative_path']);path=(root/relative).resolve()
            require(not relative.is_absolute() and path.is_relative_to(root),'native source escapes sessions root')
            require(index.pins.get(str(path))==meta.get('source_sha256'),'native retained/admitted source differs')
            natives.append({'source':str(path),'thread_id':meta['thread_id'],'rows':index.rows(path)})
        output['qualification']=qualify(original,observed,controller,state,trace,captures,natives,
            entry.get('run_id',entry.get('id')),entry['condition'])
    except (ValueError,KeyError,TypeError,OSError,IndexError,AttributeError) as error:
        output['qualification']['errors'].append(str(error))
    if index is not None:
        try:index.verify()
        except (ValueError,OSError,KeyError,TypeError) as error:output['qualification']['errors'].append(str(error))
        output['source_sha256']=dict(index.hashes)
    if output['qualification']['errors']:
        output['qualification']['status']='unknown';output['qualification']['measurement']={'status':'ineligible','bounds':None}
    return output
