"""Closed, pinned six-row item replay driver; admission belongs to the operator."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import types

FIELDS=('input_tokens','cached_input_tokens','output_tokens','reasoning_output_tokens')
MAX_FILE=512*1024*1024


def sha(data): return hashlib.sha256(data).hexdigest()


def canonical(value): return json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()


def require(value,message):
    if not value: raise ValueError(message)


def ident(value):
    require(isinstance(value,str) and bool(value),'invalid identity')
    return value


def decode(data):
    def pairs(rows):
        result={}
        for key,value in rows:
            require(key not in result,'duplicate JSON key');result[key]=value
        return result
    def invalid(_):raise ValueError('nonfinite JSON value')
    return json.loads(data,object_pairs_hook=pairs,parse_constant=invalid)


def jsonl(data):
    require(not data or data.endswith(b'\n'),'incomplete closed JSONL tail')
    for line,body in enumerate(data.splitlines(),1):
        if body.strip():
            row=decode(body);require(isinstance(row,dict),'non-object JSONL row');yield line,row


class Reader:
    def __init__(self,pins):self.pins=pins

    def path(self,name):
        require(name in self.pins,'source absent from scope');p=Path(name)
        require(p.is_absolute() and p.resolve()==p and not p.is_symlink() and p.is_file(),'unsupported source path')
        require(p.stat().st_size<=MAX_FILE,'source exceeds bounded read');return p

    def read(self,name):
        with self.path(name).open('rb') as source:data=source.read(MAX_FILE+1)
        require(len(data)<=MAX_FILE and sha(data)==self.pins[name],'source size/SHA mismatch');return data

    def verify(self,names):
        for name in names:
            h=hashlib.sha256();size=0
            with self.path(name).open('rb') as source:
                for chunk in iter(lambda:source.read(1024*1024),b''):
                    size+=len(chunk);require(size<=MAX_FILE,'source exceeds bounded hash');h.update(chunk)
            require(h.hexdigest()==self.pins[name],'source endpoint SHA mismatch')


def owned_turns(delivery):
    result={ident(r['thread_id']):set() for r in delivery['native_contexts']}
    for r in delivery['accepted_inputs']:
        thread,turn=ident(r['thread_id']),ident(r['turn_id'])
        require(thread in result and turn not in result[thread],'unowned/reused accepted turn')
        result[thread].add(turn)
    return result


def ledger(value,expected,known):
    require(len(value)==expected['count'] and sha(canonical(value))==expected['canonical_sha256'],'saved response ledger identity differs')
    for rid,row in value.items():
        ident(rid);thread,turn=ident(row['thread_id']),ident(row['turn_id'])
        require(thread in known and turn in known[thread],'response outside accepted thread/turn inventory')
        for field in FIELDS:require(type(row['usage'].get(field)) is int and row['usage'][field]>=0,'invalid saved response counter')
    return value


def native_counter_replay(records,native_rows):
    for rid,record in records.items():
        references=[r for r in record['sources'] if 'native_line' in r]
        require(bool(references),'response has no saved native counter witness')
        for ref in references:
            row=native_rows[ref['path']][ref['native_line']];p=row['payload']
            require(row['type']=='token_usage_record' and p['response_id']==rid
                    and p['thread_id']==record['thread_id'] and p['turn_id']==record['turn_id'],'native counter identity differs')
            for field in FIELDS:
                require(type(p['usage'].get(field)) is int and p['usage'][field]==record['usage'][field],'native response counter differs')
    return len(records)


def one_capture_per_thread(rows):
    captures={}
    for row in rows:
        thread=ident(row['params']['threadId']);source=ident(row['_audit_source'])
        require(captures.setdefault(thread,source)==source,'thread response ordering crosses captures')


def metadata_rows(rows):
    count=len(rows)
    for row in rows:
        node=row
        for key in ('params','usageMetadata','metadata','attribution','items'):
            node=node.get(key) if isinstance(node,dict) else None
        if isinstance(node,dict):count+=len(node)
    return count


def execute(scope_path,scope_sha,run_id):
    data=Path(scope_path).read_bytes();require(sha(data)==scope_sha,'scope SHA mismatch');scope=decode(data)
    require(len(scope['rows'])==6 and scope['run_id_order']==[r['run_id'] for r in scope['rows']],'scope must retain exact six ordered rows')
    selected=[r for r in scope['rows'] if r['run_id']==run_id];require(len(selected)==1,'run outside fixed scope');row=selected[0]
    require(not Path(row['output']).exists(),'declared result already exists')
    attempt=decode(Path(row['attempt']).read_bytes())
    require(attempt['scope_sha256']==scope_sha and attempt['run_id']==run_id,'missing/mismatched once-only attempt marker')
    reader=Reader(scope['source_sha256']);common={x['path'] for x in scope['common_inputs']}
    this=str(Path(__file__).resolve());require(this in common,'executing driver not pinned');reader.verify(common)
    module=types.ModuleType('pinned_recharge');module.__file__=str(Path(__file__).with_name('recharge.py').resolve())
    exec(compile(reader.read(module.__file__),module.__file__,'exec'),module.__dict__)
    names=set(common);result={'schema':'work-leaf-retained-read-item-recharge-execution-v1','run_id':run_id,
        'scope':{'path':str(Path(scope_path).resolve()),'sha256':scope_sha},'status':'unknown',
        'errors':[],'extract_records_calls':0,'provider_calls':0,'whole_workflow_audit_calls':0,
        'preserved_status':row['preserved_status'],'original_run':row['original_run'],'evidence':None}
    try:
        values={}
        for key,ref in row['inputs'].items():
            require(scope['source_sha256'][ref['path']]==ref['sha256'],'input pin differs');names.add(ref['path']);values[key]=decode(reader.read(ref['path']))
        delivery,retrieval,q=values['source_delivery'],values['retrieval'],values['qualification'];required={}
        for report in (delivery,retrieval):
            for path,h in report['source_sha256'].items():
                require(path not in required or required[path]==h,'source reports conflict');required[path]=h
                require(scope['source_sha256'].get(path)==h,'source report differs from scope')
        require(len(required)==row['required_sources']['count'] and sha(canonical(required))==row['required_sources']['canonical_sha256'],'required source inventory differs')
        names.update(required);reader.verify(names)
        require(q['original_entry']['run_id']==delivery['run']['run_id']==retrieval['run_id']==run_id,'source run identity mismatch')
        known=owned_turns(delivery);native={};native_rows={}
        for thread in delivery['native_contexts']:
            source=thread['source'];records=dict(jsonl(reader.read(source)));native_rows[source]=records
            native.update(module.native_inventory(thread['thread_id'],source,records.items(),known[thread['thread_id']]))
        records=ledger(q['response_evidence'],row['saved_response_ledger'],known)
        result['native_response_counter_matches']=native_counter_replay(records,native_rows)
        target=module.targets(delivery,retrieval);module.validate_targets(target,native)
        require(len(target)==row['target_population']['unique_target_items'],'target population differs')
        require(len(delivery['reads'])==row['target_population']['read_inputs'] and len(retrieval['candidates'])==row['target_population']['retrieval_candidates'],'source population differs')
        servers=[];raw_sources=[]
        for capture in sorted({r['capture'] for r in delivery['capture_provenance']}):
            path=str(Path(capture)/'server-to-client.raw');raw_sources.append(path)
            for line,event in jsonl(reader.read(path)):
                if event.get('method')=='rawResponse/completed':servers.append({**event,'_audit_source':path,'_audit_line':line})
        one_capture_per_thread(servers)
        require(len(native)+metadata_rows(servers)<=module.MAX_ROWS,'input metadata exceeds row ceiling')
        helper=str(Path(module.__file__).parents[2]/'audit_input_attribution.py');require(helper in common,'exact attribution helper path not pinned')
        extractor=module.load_extractor(helper);result['extract_records_calls']=1
        full=extractor(servers,native,records)
        result['full_attribution_sha256']=sha(canonical(full))
        result['full_attribution_bytes']=len(canonical(full))
        result['extract_return']={k:full[k] for k in ('status','errors','unidentified_response_records','native_response_ids_without_raw','duplicate_responses')}
        result['response_coverage']={'qualified_response_ids':sorted(records),
            'completed_raw_response_ids':[r['response_id'] for r in full['responses']],
            'native_without_raw':full['native_response_ids_without_raw'],
            'raw_without_qualified_native':sorted({r['response_id'] for r in full['responses']}-set(records)),
            'raw_sources':raw_sources,'complete_tail_item_coverage_proven':False}
        result['status']=full['status'];result['errors'].extend(full['errors'])
        result['evidence']=module.project(full,target)
    except (OSError,ValueError,TypeError,KeyError,StopIteration) as error:
        result['status']='unknown';result['errors'].append(str(error))
    try:
        reader.verify(names);require(sha(Path(scope_path).read_bytes())==scope_sha,'scope changed during replay')
        result['source_endpoints_match']=True
    except (OSError,ValueError) as error:
        result['status']='unknown';result['errors'].append(str(error));result['source_endpoints_match']=False
    result['verified_source_inventory_sha256']=sha(canonical({p:scope['source_sha256'][p] for p in sorted(names)}))
    result['verified_source_count']=len(names)
    require(len(json.dumps(result,sort_keys=True,indent=2,allow_nan=False).encode())+1<=module.MAX_BYTES,'complete output exceeds byte ceiling; no output published')
    return row['output'],result


def publish(writer,path,result):
    require(not Path(path).exists(),'publication target exists')
    text=json.dumps(result,sort_keys=True,indent=2,allow_nan=False)+'\n'
    require(len(text.encode())<=32*1024*1024,'publication exceeds output byte ceiling')
    patch='*** Begin Patch\n*** Add File: '+str(path)+'\n'+''.join('+'+line+'\n' for line in text.splitlines())+'*** End Patch\n'
    written=subprocess.run([writer],input=patch,text=True,capture_output=True,check=False)
    require(written.returncode==0,'apply_patch publication failed')
    require(Path(path).read_bytes()==text.encode(),'published result differs')
    return sha(text.encode())


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--scope',required=True,type=Path);p.add_argument('--scope-sha256',required=True);p.add_argument('--run-id',required=True);args=p.parse_args()
    data=args.scope.read_bytes();require(sha(data)==args.scope_sha256,'scope SHA mismatch');scope=decode(data)
    selected=[r for r in scope['rows'] if r['run_id']==args.run_id];require(len(selected)==1,'unknown scoped row');row=selected[0]
    writer=shutil.which('apply_patch');require(bool(writer),'apply_patch unavailable')
    for name in ('output','attempt'):
        path=Path(row[name]);require(path.is_absolute() and path.parent.resolve()==path.parent and path.parent.is_dir() and not path.exists() and not path.is_symlink(),'publication path unavailable')
    require(row['output']!=row['attempt'],'attempt and result paths alias')
    # One trusted operator, no concurrent calls of the same row. The durable marker
    # prevents transparent retries after publication failure; it is not crash-proof storage.
    attempt_sha=publish(writer,row['attempt'],{'run_id':args.run_id,'scope_sha256':args.scope_sha256,'state':'attempt-started','retry_authorized':False})
    path,result=execute(args.scope,args.scope_sha256,args.run_id)
    require(path==row['output'],'output path changed');result['attempt']={'path':row['attempt'],'sha256':attempt_sha}
    output_sha=publish(writer,path,result)
    print(json.dumps({'run_id':args.run_id,'output':path,'sha256':output_sha,'status':result['status'],
                      'extract_records_calls':result['extract_records_calls'],'errors':result['errors']}))


if __name__=='__main__':main()
