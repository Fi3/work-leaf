#!/usr/bin/env python3
"""R explicit bundle call/output/archive evidence; stdout only, no execution.

No saved shell command is run. Exact numeric sed selections are reconstructed in
memory. Other public output witnesses are exact archived lines after an explicit
known display prefix. A witness proves selected content, not entire output or
counterfactual savings. Every candidate and every read remains in the result.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re
import shlex
import types

HERE=Path(__file__).resolve().parent
data=(HERE/'replay_source_only.py').read_bytes()
M=types.ModuleType('r_source');M.__file__=str(HERE/'replay_source_only.py')
exec(compile(data,M.__file__,'exec'),M.__dict__)
C=M.C


def failure_kind(cmd,body,code):
    if code==2 and ('unexpected EOF while looking for matching' in body or 'rg: unrecognized flag' in body):
        lexer=shlex.shlex(cmd,posix=True,punctuation_chars=';&|');lexer.whitespace_split=True
        try:tokens=list(lexer)
        except ValueError:return 'failure_with_unresolved_prior_content'
        single_rg=(len(tokens) in (4,5) and tokens[0]=='rg' and tokens[1]=='-n'
                   and tokens[-1].startswith('/') and not any(t in ('&&','||',';','|','&') for t in tokens))
        only_error=body.startswith('rg: unrecognized flag') or body.startswith('/usr/bin/bash: -c: line 1: unexpected EOF')
        return 'failed_before_content_delivery' if single_rg and only_error else 'failure_with_unresolved_prior_content'
    return None


def read_disposition(read,calls):
    if not read.get('eligible'):return 'ineligible_preserved'
    if read.get('selected_candidate')=='inline':return 'complete_untracked_bodies_in_exact_native_input'
    bundle=read['bundle']['path']
    if any(bundle in r.get('complete_bundle_paths',[]) for r in calls):return 'complete_bundle_native_output'
    if any(any(w.get('bundle_path')==bundle for w in r.get('witnesses',[])) for r in calls):return 'selected_content_native_output'
    return 'unresolved_or_no_content' if calls else 'no_explicit_path_call_in_complete_captured_native_scope'


def line_values(text,bundle,multiple):
    if multiple:
        if not text.startswith(bundle+':'):return []
        match=re.fullmatch(r'(\d+):(.*)',text[len(bundle)+1:])
        return [(match[2],int(match[1]))] if match else []
    options={text}
    m=re.match(r'^\s*\d+(?:\t|:| )(.+)$',text)
    if m:options.add(m[1])
    m=re.match(r'^(?:[^:]+):\d+:(.*)$',text)
    if m:options.add(m[1])
    return [(value,None) for value in sorted(options)]


def run(number):
    index=C.SourceIndex();index.read(Path(__file__).resolve());index.read(HERE/'replay_source_only.py',M.sha(data))
    path=HERE/f'untracked-reads-01-workflow-{number:03d}'/'SOURCE-DELIVERY-ORIGINAL.json'
    saved=C.decode(index.read(path));rows={};public={}
    for source in saved['source_sha256']:index.read(source,saved['source_sha256'][source])
    for source in {c['source'] for c in saved['bundle_path_candidates']}:
        rows[source]={n:r for n,r in index.records(source) if r.get('type')=='response_item' and r.get('payload',{}).get('type') in C.CALLS|C.OUTPUTS}
    for capture in saved['capture_provenance']:
        source=Path(capture['capture'])/'server-to-client.raw'
        for line,row in index.records(source):
            p=row.get('params',{});item=p.get('item',{})
            if row.get('method')=='item/completed' and item.get('type')=='commandExecution':
                key=(p['threadId'],p['turnId'],item['id']);M.require(key not in public,'duplicate completed public command')
                public[key]=(str(source),line,item)
    archives={};line_maps={};bodies={}
    for read in saved['reads']:
        if 'archive' not in read:continue
        bundle=read['bundle']['path'];ar=read['archive'];body=index.read(ar['path'],ar['sha256']);archives[bundle]=body.splitlines(keepends=True);bodies[bundle]=body
        lm=defaultdict(list)
        for n,line in enumerate(body.decode().splitlines(),1):lm[line].append(n)
        line_maps[bundle]=lm
    results=[]
    for ordinal,c in enumerate(saved['bundle_path_candidates']):
        pl=rows[c['source']][c['line']]['payload'];args=json.loads(pl['arguments']);cmd=args['cmd'];key=(c['thread_id'],c['turn_id'],c['call_id'])
        M.require(key in public and c['output_link_status']=='linked','missing unique public/native command-output join')
        source,line,item=public[key];outer=shlex.split(item['command'])
        M.require(len(outer)==3 and outer[1] in ('-lc','-c') and outer[2]==cmd,'public/native exact command differs')
        output=c['outputs'][0];native_output=rows[output['source']][output['line']]['payload']['output']
        match=re.search(r'^Process exited with code (\d+)\n',native_output,re.M)
        M.require(match and int(match[1])==item['exitCode'] and 'Output:\n' in native_output,'public/native terminal output status')
        body=native_output.split('Output:\n',1)[1];truncated=body.startswith('Warning: truncated output (original token count: ')
        public_body=item.get('aggregatedOutput')
        M.require(public_body is None or isinstance(public_body,str),'unknown public command output shape')
        refs=list(dict.fromkeys(r['path'] for r in c['bundle_references']));verb=cmd.split()[0]
        M.require(len(refs)<=16,'bounded per-call archive witness scope')
        r={'candidate_index':ordinal,'thread_id':c['thread_id'],'turn_id':c['turn_id'],'native_source':c['source'],'native_call_line':c['line'],'native_call_item_id':c['item_id'],'call_id':c['call_id'],
           'native_output_line':output['line'],'native_output_item_id':output['item_id'],'native_output_payload_sha256':output['payload_sha256'],
           'public_source':source,'public_line':line,'public_item_payload_sha256':M.sha(C.canonical(item)),
           'exact_call_identity_and_command':True,'command_sha256':M.sha(cmd.encode()),'leading_program':verb,'exit_code':item['exitCode'],
           'native_display_truncation':truncated,'public_output_bytes':len(public_body.encode()) if public_body is not None else None,'public_output_sha256':M.sha(public_body.encode()) if public_body is not None else None,
           'native_display_body_bytes':len(body.encode()),'native_display_body_sha256':M.sha(body.encode()),'native_equals_public_output':body==public_body if public_body is not None else None,
           'references':c['bundle_references'],'classification':'unresolved','witnesses':[]}
        failed=failure_kind(cmd,body,item['exitCode'])
        if failed:
            r['classification']=failed;r['reason']='Actual shell/argument parser error; only an isolated single rg command with error-only output establishes failure before content delivery.'
        elif item['exitCode']==1 and verb=='rg' and body=='':
            r['classification']='executed_search_no_content';r['reason']='Exact successful tool execution reports rg no match, not content delivery or a full read.'
        elif item['exitCode']==0 and verb in ('sed','rg','grep','nl','awk'):
            selected=[]; selected_origins=[];supported=True
            # Only this closed numeric sed grammar receives full-output equality.
            for segment in re.split(r' && |; |\n',cmd):
                m=re.fullmatch(r"sed -n '([0-9,p;$]+)' (/[^\s;&]+|'[^']+')",segment)
                if not m:supported=False;break
                bundle=m[2].strip("'")
                if bundle not in archives:supported=False;break
                specifications=[]
                for expr in m[1].split(';'):
                    z=re.fullmatch(r'(\d+)(?:,(\d+|\$))?p',expr)
                    if not z:supported=False;break
                    lo=int(z[1]);hi=(len(archives[bundle]) if z[2]=='$' else int(z[2])) if z[2] else lo
                    specifications.append((lo,hi))
                if not supported:break
                M.require(len(specifications)<=16,'bounded numeric sed address inventory')
                # sed evaluates every address for each line (including overlaps).
                for i,text in enumerate(archives[bundle],1):
                    for lo,hi in specifications:
                        if lo<=i<=hi:selected.append(text);selected_origins.append((bundle,i,text))
            if supported:
                expected=b''.join(selected);r['numeric_sed_expected_bytes']=len(expected);r['numeric_sed_expected_sha256']=M.sha(expected)
                r['numeric_sed_public_exact']=expected.decode()==public_body if public_body is not None else None;r['numeric_sed_native_exact']=expected.decode()==body
                M.require(r['numeric_sed_public_exact'] or public_body is None and r['numeric_sed_native_exact'],'numeric sed execution output differs from archived selection')
            witnessed=set()
            if supported and r['numeric_sed_native_exact']:
                for n,(bundle,archive_line,text) in enumerate(selected_origins,1):
                    if bundle not in witnessed and len(text.strip())>=24:
                        value=text.decode().rstrip('\n')
                        r['witnesses'].append({'bundle_path':bundle,'native_body_line':n,'archive_line_candidates':[archive_line],'exact_line_sha256':M.sha(value.encode()),'origin':'fully_reconstructed_numeric_sed_segment'})
                        witnessed.add(bundle)
            for n,text in enumerate(body.splitlines(),1):
                for bundle in refs:
                    if bundle in witnessed:continue
                    for value,expected_line in line_values(text,bundle,len(refs)>1):
                        if len(value.strip())<24:continue
                        positions=line_maps[bundle].get(value,[])
                        if expected_line is not None:positions=[expected_line] if expected_line in positions else []
                        if positions:
                            r['witnesses'].append({'bundle_path':bundle,'native_body_line':n,'archive_line_candidates':positions,'exact_line_sha256':M.sha(value.encode())})
                            witnessed.add(bundle)
                            break
                if len(witnessed)==len(refs):break
            if r['witnesses']:
                r['classification']='selected_content_delivered';r['reason']='Read/search command names the issued archive; a nontrivial exact output line is present in that archive. Not a complete-output or byte-coverage claim.'
            elif body=='' and supported and r['numeric_sed_native_exact']:
                r['classification']='executed_selection_no_content';r['reason']='Exact numeric selection lies outside archived line range; native empty output is recorded, null public aggregatedOutput is not zero-filled.'
            r['complete_bundle_paths']=[b for b in refs if body.encode()==bodies[b]]
            if r['complete_bundle_paths']:r['classification']='complete_bundle_delivered'
        results.append(r)
    reads=[]; by_read=defaultdict(list)
    for r in results:
        for i in {z.get('read_index') for z in r['references']}:
            if i is not None:by_read[i].append(r)
    for i,x in enumerate(saved['reads']):
        associated=by_read[i]
        disposition=read_disposition(x,associated)
        reads.append({'read_index':i,'trace_line':x['trace_line'],'sequence':x['sequence'],'thread_id':x.get('thread_id'),'turn_id':x.get('turn_id'),'eligible':x.get('eligible'),'selected_candidate':x.get('selected_candidate'),'disposition':disposition,'candidate_indices':[a['candidate_index'] for a in associated]})
    index.verify()
    return {'schema':'retained-read-retrieval-source-only-v1','run_id':saved['run']['run_id'],'candidates':results,'reads':reads,
            'classification_counts':dict(Counter(r['classification'] for r in results)),
            'limitations':['Exact known numeric sed outputs are fully reconstructed; other commands retain selected exact line witnesses only.',
                'No inferred operating-system file-open count; no indirect retrieval absence, full union coverage, token accounting or causal savings.',
                'Native display truncation is distinct from command-produced public output. Failed commands and empty searches are retained.'],
            'source_sha256':index.hashes}


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--run',type=int,required=True);args=p.parse_args()
    M.require(1<=args.run<=6,'only original actual R runs')
    print(json.dumps(run(args.run),sort_keys=True,indent=2,allow_nan=False))
