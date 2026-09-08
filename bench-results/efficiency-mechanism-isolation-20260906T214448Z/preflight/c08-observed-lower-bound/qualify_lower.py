"""Finite saved-response lower bounds. No auditor, provider or baseline arithmetic."""
import argparse
import ast
import copy
import hashlib
import json
import os
from pathlib import Path
import stat
import sys
import types

RUNS = tuple(f'automatic-refresh-01-workflow-{n:03}' for n in (1, 2, 3))
BASES = tuple(f'work-units-01-workflow-{n:03}' for n in (2, 4, 6, 9, 10, 12))
FIELDS = ('input_tokens', 'cached_input_tokens', 'output_tokens', 'reasoning_output_tokens')
METRICS = ('raw_input_plus_output', 'uncached_input_plus_output')
ACCOUNTING_SHA = 'c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a'
STRICT_SHA = 'dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3'
SOURCE_HELPER_SHA = '7ea311907b975f25b02215c3e750eb8d69b49311c30247db6967862fe44ed092'
CONTEXT_HELPER_SHA = '513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3'


def require(value, reason):
    if not value:
        raise ValueError(reason)


def canonical(value, unicode=False):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=not unicode, allow_nan=False).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def same(left, right):
    return canonical(left) == canonical(right)


def identity(value):
    require(type(value) is str and bool(value), 'invalid identity')
    return value


def positive(value):
    require(type(value) is int and value > 0, 'invalid physical line or count')
    return value


def digest_value(value):
    require(type(value) is str and len(value) == 64 and all(c in '0123456789abcdef' for c in value), 'invalid source digest')
    return value


def decode(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON value')))


def load_arithmetic(accounting_bytes, strict_bytes):
    """Compile only unchanged named pure definitions from exact complete sources."""
    require(sha(accounting_bytes) == ACCOUNTING_SHA and sha(strict_bytes) == STRICT_SHA, 'arithmetic source pin differs')
    def selected(data, functions, constants, namespace, filename):
        nodes = []
        for node in ast.parse(data, filename).body:
            if isinstance(node, ast.FunctionDef) and node.name in functions:
                nodes.append(node)
            elif isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name) and node.targets[0].id in constants:
                nodes.append(node)
        require(len(nodes) == len(functions) + len(constants), 'arithmetic definitions differ')
        exec(compile(ast.Module(body=nodes, type_ignores=[]), filename, 'exec'), namespace)
        return types.SimpleNamespace(**namespace)
    strict = selected(strict_bytes, {'integer', 'usage'}, {'FIELDS', 'CAMEL'}, {}, 'pinned/batch_analysis.py')
    return selected(accounting_bytes, {'checked_usage', 'add'}, {'FIELDS', 'CAMEL'},
                    {'STRICT': strict}, 'pinned/accounting_untracked_reads.py')


def checked_map(value, arithmetic):
    require(type(value) is dict, 'response map missing')
    for rid, row in value.items():
        identity(rid); require(type(row) is dict, 'response row missing')
        identity(row.get('thread_id')); identity(row.get('turn_id'))
        usage = arithmetic.checked_usage(row.get('usage'), camel=False)
        for key, number in usage.items():
            require(key not in row['usage'] or type(row['usage'][key]) is int and row['usage'][key] == number, 'typed derived usage differs')
        require(type(row.get('sources')) is list and bool(row['sources']), 'response locators missing')
    return value


def projection_identity(original, projection):
    require(projection['original_entry'].get('id', projection['original_entry'].get('run_id')) == original['run_id'], 'projection run differs')
    p = projection['accounting']
    require(p.get('exact_helper_result_sha256') == sha(canonical(original)), 'full original digest differs')
    expected = p['exact_response_evidence']; records = original['exact_response_evidence']
    require(type(expected.get('count')) is int and expected['count'] == len(records) and
            expected.get('sha256') == sha(canonical(records)), 'original response projection differs')


def source_population(case):
    result = case['source_result']; manifest = case['source_input']
    require(result.get('run_id') == case['run_id'], 'source result run differs')
    require(result.get('errors') == ['trace-occurrence-not-joined'] * 2, 'unqualified source error')
    require(result['native_membership'].get('errors') == [] and result['project_inventory'].get('errors') == [], 'source subproof failed')
    require(result['invocation_streams'] and all(r.get('errors') == [] for r in result['invocation_streams']), 'stream proof failed')
    require(result['frame_proofs'] and all(r.get('turn_closure', {}).get('errors') == [] for r in result['frame_proofs']), 'frame closure proof missing')
    sources = {}; source_names = set(); accepted = {}; contexts = set()
    for row in result['native_membership']['threads']:
        tid = identity(row.get('thread_id')); path = identity(row.get('source'))
        require(tid not in sources and path not in source_names, 'duplicate native membership')
        sources[tid] = path; source_names.add(path)
    declared = {row['thread_id']: row['source']['path'] for row in manifest['native_sessions']}
    require(len(declared) == len(manifest['native_sessions']) and same(declared, sources), 'declared native source population differs')
    for row in result['native_membership']['contexts']:
        key = identity(row.get('thread_id')), identity(row.get('turn_id'))
        require(key not in contexts and key[0] in sources, 'duplicate or foreign context'); contexts.add(key)
    for row in result['delivery']['inputs']:
        key = identity(row.get('thread_id')), identity(row.get('turn_id'))
        require(row.get('status') == 'joined' and key not in accepted and key[0] in sources and
                row.get('native_source') == sources[key[0]], 'input membership unavailable')
        accepted[key] = row
    require(set(accepted) == contexts, 'accepted input/context populations differ')
    return sources, accepted


def retained_records(case, arithmetic, sources, accepted):
    original = case['original']; records = checked_map(original['exact_response_evidence'], arithmetic)
    require(original.get('corrected_scope') is not None, 'original reconciliation missing')
    raw = original['measurement']['gap_inventory']
    require(raw.get('errors') == [] and set(raw['raw_responses']) == set(records), 'original response populations differ')
    server_paths = {row['servers']['path'] for row in case['source_input']['captures']}
    for rid, row in records.items():
        key = row['thread_id'], row['turn_id']
        require(key in accepted, 'record outside accepted population')
        require(same(raw['raw_responses'][rid], {k:v for k,v in row.items() if k != 'sources'}), 'original raw record differs')
        native = []; public = []
        for loc in row['sources']:
            if 'native_line' in loc:
                require(loc['path'] == sources[key[0]], 'record native source differs'); positive(loc['native_line']); native.append(loc)
            elif 'server_line' in loc:
                require(loc['path'] in server_paths, 'record capture source differs'); positive(loc['server_line']); public.append(loc)
            else: raise ValueError('unsupported record locator')
        require(native and public, 'incomplete response locators')
    return records


def new_records(case, arithmetic, sources, accepted):
    sidecar = case['sidecar']; eligibility = case['eligibility']
    require(sidecar.get('schema_version') == 1 and type(sidecar.get('schema_version')) is int and sidecar.get('errors') == [] and
            sidecar.get('changes_workflow_totals') is False and sidecar.get('proves_interrupted_tail_coverage') is False, 'sidecar schema or errors')
    require(eligibility.get('status') == 'source-proof-established' and eligibility.get('errors') == [] and
            type(eligibility.get('accounting_calls')) is int and eligibility['accounting_calls'] == 0 and
            type(eligibility.get('provider_calls')) is int and eligibility['provider_calls'] == 0 and
            type(eligibility.get('proofs')) is list and len(eligibility['proofs']) == 1, 'singleton eligibility proof missing')
    proof = eligibility['proofs'][0]; proof_rid = identity(proof['response_id'])
    require(proof['native_metadata_sha256'] == sha(canonical(case['metadata'], True)), 'proof canonical metadata differs')
    expected = ('gpt-5.5', 'xhigh', case['source_input']['project_cwd'])
    require(tuple(proof.get(k) for k in ('model','effort','cwd')) == expected, 'proof context differs')
    natives = {}; native_payloads = {}; native_set = set(); proof_bound = False
    for source in case['native_sources']:
        tid = identity(source['thread_id']); path = identity(source['source']); rows = source['rows']
        require(tid not in native_set and sources.get(tid) == path, 'native source population differs'); native_set.add(tid)
        contexts = {}; sessions = 0
        if path == proof['source']:
            require(proof['thread_id'] == tid and proof['native_rows_sha256'] == sha(canonical(rows, True)), 'proof full native source differs')
            line = positive(proof['native_response_line']); context_line = positive(proof['native_context_line']); marker_line = positive(proof['native_marker_line'])
            require(line < marker_line < context_line <= len(rows), 'proof line order differs')
            payload = rows[line-1].get('payload')
            require(rows[line-1].get('type') == 'token_usage_record' and payload.get('response_id') == proof_rid and
                    sha(canonical(payload, True)) == proof['native_response_payload_sha256'], 'proof response differs')
            context = rows[context_line-1]
            require(context.get('type') == 'turn_context' and context['payload'].get('turn_id') == proof['turn_id'] and
                    tuple(context['payload'].get(k) for k in ('model','effort','cwd')) == expected, 'proof future context differs')
            marker = rows[marker_line-1]
            require(marker.get('type') == 'compacted' and marker['payload'].get('compaction_response_id') == proof_rid and
                    same(marker['payload'].get('latest_token_usage_record'), payload), 'proof marker differs')
            proof_bound = True
        for physical, row in enumerate(rows, 1):
            require(type(row) is dict, 'native row shape')
            if not row: continue
            kind = row.get('type'); p = row.get('payload')
            require(type(p) is dict, 'native payload shape')
            if kind == 'session_meta':
                sessions += 1
                require(p.get('id') == tid and p.get('cwd') == expected[2] and p.get('cli_version') == '0.153.4', 'native session differs')
            elif kind == 'turn_context':
                turn = identity(p.get('turn_id'))
                require((tid,turn) in accepted and tuple(p.get(k) for k in ('model','effort','cwd')) == expected, 'native context differs')
                require(turn not in contexts or same(contexts[turn], p), 'conflicting native context'); contexts[turn] = p
            elif kind == 'token_usage_record':
                rid = identity(p.get('response_id')); turn = identity(p.get('turn_id'))
                require(p.get('thread_id') == p.get('session_id') == tid and (tid,turn) in accepted, 'native response ownership differs')
                fallback = proof_bound and path == proof['source'] and physical == proof['native_response_line'] and rid == proof_rid and turn == proof['turn_id']
                require(turn in contexts or fallback, 'response lacks preceding or proved context')
                require(type(p.get('usage')) is dict and 'total_tokens' in p['usage'], 'native additive total missing')
                u = arithmetic.checked_usage(p['usage'], camel=False)
                encoded = canonical(p)
                require(rid not in native_payloads or native_payloads[rid] == encoded, 'conflicting native response replay')
                if rid in natives:
                    require(natives[rid]['sources'][0]['path'] == path, 'native response repeated across sources')
                else:
                    natives[rid] = {'thread_id':tid,'turn_id':turn,'usage':u,'sources':[{'path':path,'native_line':physical}]}
                    native_payloads[rid] = encoded
        require(sessions == 1, 'native session census differs')
    require(native_set == set(sources) and proof_bound and proof_rid in natives, 'native or proof population incomplete')
    declared = {row['path']:row for row in case['source_input']['captures']}
    seen_caps = set(); raw = {}; first = {}; duplicates = 0; proof_raw = False
    for cap in case['captures']:
        path = identity(cap['path']); require(path in declared and path not in seen_caps, 'capture population differs'); seen_caps.add(path)
        ordinal = 0
        for physical, row in enumerate(cap['rows'],1):
            if row is None: continue
            require(type(row) is dict, 'server row shape')
            sequence = ordinal; ordinal += 1
            if row.get('method') != 'rawResponse/completed': continue
            p = row.get('params'); require(type(p) is dict, 'raw response payload missing')
            rid = identity(p.get('responseId')); key = identity(p.get('threadId')),identity(p.get('turnId'))
            require(key in accepted, 'raw response outside accepted population')
            u = arithmetic.checked_usage(p.get('usage')); value = {'thread_id':key[0],'turn_id':key[1],'usage':u}
            require(rid in natives and same(value,{k:v for k,v in natives[rid].items() if k!='sources'}), 'raw/native response differs')
            if rid in raw:
                require(first[rid][0] == path and same(raw[rid],value), 'conflicting or cross-capture response replay'); duplicates += 1
            else: raw[rid] = value; first[rid] = (path,sequence)
            natives[rid]['sources'].append({'path':declared[path]['servers']['path'],'server_line':physical})
            if rid == proof_rid and path == proof['capture'] and physical == proof['raw_response_line']:
                require(key == (proof['thread_id'],proof['turn_id']), 'proof raw scope differs'); proof_raw = True
    require(seen_caps == set(declared) and set(raw) == set(natives) and proof_raw, 'complete raw/native/proof response population differs')
    side = {}
    for row in sidecar['records']:
        rid = identity(row.get('response_id')); require(rid not in side and row.get('valid') is True, 'sidecar identity or validity differs')
        u = arithmetic.checked_usage(row.get('usage'),camel=False)
        value = {'thread_id':identity(row.get('thread_id')),'turn_id':identity(row.get('turn_id')),'usage':u}
        require(rid in raw and same(value,raw[rid]), 'sidecar response differs')
        require(type(row.get('first_server_sequence')) is int and row['first_server_sequence'] == first[rid][1], 'sidecar parsed-message ordinal differs')
        side[rid] = value
    require(set(side) == set(raw) and type(sidecar.get('duplicate_events')) is int and sidecar['duplicate_events'] == duplicates, 'sidecar population or duplicate count differs')
    return natives


def qualify(population, arithmetic):
    result = {'schema':'work-leaf-c08-observed-lower-result-v1','errors':[], 'candidates':[], 'baselines':[],
              'accounting_calls':0,'provider_calls':0,'baseline_ledger_calls':0,'hidden_call_completeness_proven':False}
    maps = {}; prior = set()
    result['baselines'] = [copy.deepcopy(r.get('receipt')) for r in population.get('baselines',[])]
    try:
        require(tuple(r['run_id'] for r in population['runs']) == RUNS and tuple(r['run_id'] for r in population['baselines']) == BASES, 'fixed nine-run population differs')
        for item in population['baselines']:
            receipt = item['receipt']; supplement = item['supplement']; run = item['run_id']
            require(receipt['original_entry'].get('id',receipt['original_entry'].get('run_id')) == run and supplement['original_entry'].get('id',supplement['original_entry'].get('run_id')) == run, 'baseline ownership differs')
            require(supplement.get('schema') == 'work-leaf-admitted-provider-ledger-execution-v1' and supplement['result'].get('schema') == 'work-leaf-provider-ledger-qualification-v1', 'baseline supplement schema differs')
            original = supplement['result']['original']; require(original.get('run_id') == run, 'baseline original identity differs')
            records = checked_map(supplement['response_evidence'], arithmetic); count = item['expected_response_count']
            require(type(count) is int and count == len(records), 'baseline count differs')
            for p in (receipt['accounting']['exact_response_evidence'],original['exact_response_evidence']):
                require(type(p.get('count')) is int and p['count'] == count and p.get('sha256') == sha(canonical(records)), 'baseline map projection differs')
            require(not prior.intersection(records), 'baseline response identity overlaps'); prior.update(records)
            maps[run] = records
    except (ValueError, KeyError, TypeError, IndexError, AttributeError):
        result['errors'].append('baseline-or-fixed-population-invalid')
    for case in population.get('runs',[]):
        row = {'run_id':case.get('run_id'),'status':'unqualified','errors':[], 'bounds':None,
               'original_outcome':copy.deepcopy(case.get('original_outcome')), 'original_errors':copy.deepcopy(case.get('original',{}).get('errors')),
               'original_warnings':copy.deepcopy(case.get('original',{}).get('warnings')),
               'original_measurement':copy.deepcopy(case.get('original',{}).get('measurement')),
               'original_status':case.get('original',{}).get('status'),
               'original_observer_ledger':copy.deepcopy(case.get('original',{}).get('original_observer_ledger')),
               'original_phase_flags':copy.deepcopy(case.get('source_result',{}).get('original_phase_flags')),
               'original_source_errors':copy.deepcopy(case.get('source_result',{}).get('errors')),
               'original_observer_errors':copy.deepcopy(case.get('observer_errors'))}
        result['candidates'].append(row)
        if result['errors']: continue
        try:
            original = case['original']; require(original['run_id'] == case['run_id'], 'original run differs')
            projection_identity(original, case['projection']); sources, accepted = source_population(case)
            records = new_records(case, arithmetic, sources, accepted) if case['run_id'] == RUNS[2] else retained_records(case, arithmetic, sources, accepted)
            require(type(case['expected_response_count']) is int and len(records) == case['expected_response_count'], 'candidate response count differs')
            require(not prior.intersection(records), 'candidate response identity overlaps'); prior.update(records)
            covered = {(r['thread_id'],r['turn_id']) for r in records.values()}
            row.update(response_count=len(records),
                       response_evidence=records, response_evidence_sha256=sha(canonical(records)),
                       accepted_turns_without_recorded_response=[{'thread_id':t,'turn_id':u} for t,u in sorted(set(accepted)-covered)])
            if 'sidecar' in case: row['original_response_thread_statuses'] = copy.deepcopy(case['sidecar']['threads'])
            if 'eligibility' in case:
                row['context_eligibility_proof'] = copy.deepcopy(case['eligibility']['proofs'][0])
                row['metadata_raw_line_sha256'] = case.get('metadata_raw_line_sha256')
            maps[case['run_id']] = records
        except (ValueError, KeyError, TypeError, IndexError, AttributeError) as error:
            row['errors'].append(str(error) if type(error) is ValueError else 'candidate-shape-invalid')
            if isinstance(error,ValueError) and str(error) == 'candidate response identity overlaps': result['errors'].append(str(error))
    if any(row['errors'] for row in result['candidates']):
        result['errors'].append('candidate-population-unqualified')
    if not result['errors']:
        # No charge aggregation until every declared map and the global ID set pass.
        for case,row in zip(population['runs'],result['candidates']):
            try:
                original=case['original']; records=maps[case['run_id']]
                measured=arithmetic.add(*(r['usage'] for r in records.values()))
                if case['run_id'] != RUNS[2]:
                    require(same(measured,original['corrected_scope']['usage']) and same(measured,original['measurement']['recorded_usage']), 'original recorded lower differs')
                row.update(status='qualified_observed_lower',recorded_usage=measured,
                           bounds={name:{'lower':measured[name],'upper':None} for name in METRICS})
            except (ValueError,KeyError,TypeError):
                row['errors'].append('recorded-lower-consistency-failed');result['errors'].append('arithmetic-unqualified')
    if result['errors']:
        for row in result['candidates']: row.update(status='unqualified',bounds=None)
    result['status'] = 'qualified_observed_lower' if not result['errors'] and all(r['status']=='qualified_observed_lower' for r in result['candidates']) else 'unqualified'
    return result


class Sources:
    def __init__(self, pins):
        self.pins = {}; self.total = 0
        self.admit(pins)

    def admit(self, pins):
        require(type(pins) is dict, 'source map missing')
        for path, digest in pins.items():
            identity(path); digest_value(digest)
            require(path not in self.pins or self.pins[path] == digest, 'conflicting source identities')
            self.pins[path] = digest
        require(len(self.pins) <= 20000, 'source-count limit')

    def read(self, ref):
        path = Path(ref['path']); expected = digest_value(ref['sha256'])
        require(path.is_absolute() and str(path) == ref['path'] and path.resolve() == path and stat.S_ISREG(path.lstat().st_mode), 'source must be canonical regular file')
        require(self.pins.get(str(path)) == expected, 'source is not declared')
        size = path.stat().st_size; require(size <= 1024**3, 'source-file limit')
        data = path.read_bytes(); require(len(data) == size and sha(data) == expected, 'source endpoint differs')
        return data

    def json(self, ref): return decode(self.read(ref))

    def rows(self, ref, *, server=False):
        data = self.read(ref); require(not data or data.endswith(b'\n'), 'incomplete closed JSONL')
        result = [decode(line) if line.strip() else None if server else {} for line in data.splitlines()]
        require(all(type(row) is dict or server and row is None for row in result), 'JSONL object required')
        return result

    def verify(self):
        self.total = 0
        for path,digest in self.pins.items():
            self.total += len(self.read({'path':path,'sha256':digest}))
            require(self.total <= 32*1024**3, 'total source-byte limit')


def materialize(scope, index):
    """Read only explicit refs and admitted nested source maps; never run old code."""
    def document(ref):
        index.admit({ref['path']:ref['sha256']})
        value=index.json(ref)
        require(type(value) is dict, 'referenced JSON object required')
        if 'source_sha256' in value: index.admit(value['source_sha256'])
        return value
    ref=scope['preparation'];index.admit({ref['path']:ref['sha256']});index.read(ref)
    old=document(scope['accounting_scope']); consolidated=document(scope['original_consolidation'])
    require(old.get('schema') == 'work-leaf-c08-original-accounting-once-v1' and
            tuple(old.get('new_run_ids',[])) == RUNS and tuple(old.get('baseline_run_ids',[])) == BASES, 'original scope population differs')
    require(consolidated.get('schema') == 'work-leaf-c08-original-accounting-publication-v1' and
            consolidated.get('scope_sha256') == scope['accounting_scope']['sha256'], 'original consolidation scope differs')
    for key in ('integrity_errors','identity_errors','execution_errors','publication_errors'):
        require(consolidated.get(key) == [], 'original publication integrity failed')
    require(consolidated.get('accounting_call_attempts') == list(RUNS) and consolidated.get('baseline_accounting_calls') == 0, 'original call inventory differs')
    require(same(scope['arithmetic']['accounting'],old['helper']), 'arithmetic helper differs from original scope')
    phase=document(old['phase_manifest'])
    helper=scope['arithmetic']['accounting']
    matches=[r for r in phase['files'] if r.get('path') == helper['path']]
    require(len(matches)==1 and matches[0].get('role')=='frozen-evidence' and matches[0].get('sha256')==ACCOUNTING_SHA==helper['sha256'], 'arithmetic manifest identity differs')
    require(set(scope['arithmetic'])=={'accounting','strict'} and scope['arithmetic']['strict']['sha256']==STRICT_SHA, 'arithmetic references differ')
    for ref in scope['arithmetic'].values():index.admit({ref['path']:ref['sha256']})
    original_rows={r['run_id']:r for r in consolidated['runs']}
    require(len(consolidated['runs'])==len(original_rows)==9 and set(original_rows)==set(RUNS+BASES), 'original result population differs')
    require(tuple(r['run_id'] for r in scope['candidates'])==RUNS, 'candidate input population differs')
    population={'runs':[],'baselines':[]}
    for candidate in scope['candidates']:
        run=candidate['run_id']; oldrow=original_rows[run]
        require(same(candidate['original_full'],oldrow['full_result']) and same(candidate['original_projection'],oldrow['receipt']), 'original sidecar reference differs')
        full=document(candidate['original_full']); projection=document(candidate['original_projection'])
        require(projection.get('scope_sha256')==scope['accounting_scope']['sha256'] and
                same(projection['accounting'],oldrow['accounting']) and same(projection['original_entry'],oldrow['original_entry']), 'original projection relation differs')
        inp=document(candidate['source_input']); result=document(candidate['source_result']); source_scope=document(inp['scope'])
        require(inp.get('schema')=='work-leaf-c08-natural-source-input-v1' and result.get('schema')=='work-leaf-c08-natural-source-result-v1' and
                inp.get('run_id')==result.get('run_id')==run and inp.get('helper_sha256')==SOURCE_HELPER_SHA, 'source result implementation differs')
        require(same(inp['phase_manifest'],old['phase_manifest']) and
                source_scope['input_paths'][run]==candidate['source_input']['path'] and
                source_scope['output_paths'][run]==candidate['source_result']['path'], 'source receipt input/output relation differs')
        require(result['source_sha256'].get(candidate['source_input']['path'])==candidate['source_input']['sha256'] and
                result['source_sha256'].get(inp['scope']['path'])==inp['scope']['sha256'] and
                SOURCE_HELPER_SHA in result['source_sha256'].values(), 'source receipt dependency closure differs')
        require({r['capture_id'] for r in result['frame_proofs']} == {Path(r['path']).name for r in inp['captures']}, 'source frame population differs')
        # The existing source receipt, not this helper, owns whole-frame/source checks.
        for cap in inp['captures']:
            for name in ('clients','forwarded','servers','start','child','end','settings','journal','grace'):
                ref=cap[name];require(result['source_sha256'].get(ref['path'])==ref['sha256'], 'capture absent from closed source proof')
        for native in inp['native_sessions']:
            ref=native['source'];require(result['source_sha256'].get(ref['path'])==ref['sha256'], 'native absent from closed source proof')
        row={'run_id':run,'expected_response_count':candidate['expected_response_count'],'original':full,'projection':projection,
             'source_input':inp,'source_result':result,'original_outcome':copy.deepcopy(projection['original_entry']),
             'observer_errors':copy.deepcopy((full.get('original_observer_ledger') or {}).get('errors'))}
        if run==RUNS[2]:
            e_input=document(candidate['eligibility_input']); eligibility=document(candidate['eligibility_result'])
            require(e_input.get('schema')=='work-leaf-c08-compaction-eligibility-only-input-v1' and
                    eligibility.get('schema')=='work-leaf-c08-compaction-eligibility-only-result-v1' and
                    eligibility.get('run_id')==e_input.get('run_id')==run and
                    eligibility.get('input_sha256')==candidate['eligibility_input']['sha256'], 'eligibility input identity differs')
            require(e_input['helper']['sha256']==CONTEXT_HELPER_SHA and
                    eligibility['source_sha256'].get(e_input['helper']['path'])==CONTEXT_HELPER_SHA, 'eligibility implementation differs')
            require(same(e_input['source_qualification']['result'],candidate['source_result']) and
                    same(e_input['source_qualification']['input'],candidate['source_input']) and
                    same(e_input['original_full_result'],candidate['original_full']), 'eligibility source relations differ')
            ref=e_input['metadata'];require(eligibility['source_sha256'].get(ref['path'])==ref['sha256'], 'eligibility metadata source differs')
            metadata=index.rows(ref);line=positive(e_input['metadata_line']);metadata_bytes=index.read(ref).splitlines()
            require(line<=len(metadata) and line<=len(metadata_bytes), 'metadata physical row missing')
            require(sha(metadata_bytes[line-1])==e_input['metadata_row_sha256'], 'raw metadata physical-line hash differs')
            proof=eligibility['proofs'];require(type(proof) is list and len(proof)==1, 'singleton proof missing')
            require(all(same(proof[0].get(k),v) for k,v in e_input['expected_proof'].items()), 'saved expected proof differs')
            side=candidate['response_sidecar'];capture=e_input['capture']
            require(side['path']==str(Path(capture['path'])/'response-usage.json') and
                    any(same(capture,cap) for cap in inp['captures']), 'sidecar capture binding differs')
            row.update(sidecar=document(side),eligibility=eligibility,eligibility_input=e_input,metadata=metadata[line-1],
                       metadata_raw_line_sha256=e_input['metadata_row_sha256'],
                       captures=[{'path':cap['path'],'rows':index.rows(cap['servers'],server=True)} for cap in inp['captures']],
                       native_sources=[{'thread_id':n['thread_id'],'source':n['source']['path'],'rows':index.rows(n['source'])} for n in inp['native_sessions']])
        population['runs'].append(row)
    require(tuple(b['run_id'] for b in old['baselines'])==BASES, 'baseline references differ')
    for baseline in old['baselines']:
        run=baseline['run_id'];row=original_rows[run]
        require(same(row['receipt'],baseline['receipt']) and same(row['response_supplement'],baseline['response_supplement']), 'baseline consolidation references differ')
        receipt=document(baseline['receipt']);supplement=document(baseline['response_supplement'])
        require(same(receipt['accounting'],row['accounting']), 'retained baseline projection differs')
        population['baselines'].append({'run_id':run,'expected_response_count':baseline['expected_response_count'],
                                         'receipt':receipt,'supplement':supplement})
    return population


def write_fd(fd, data):
    view=memoryview(data)
    while view:
        count=os.write(fd,view);require(count>0,'short output write');view=view[count:]
    os.fsync(fd)


def execute(scope_path, expected_scope_sha256, output_path):
    scope_ref={'path':scope_path,'sha256':expected_scope_sha256};index=Sources({scope_path:expected_scope_sha256})
    scope=index.json(scope_ref)
    require(scope.get('schema') == 'work-leaf-c08-observed-lower-scope-v1' and scope.get('execution_authorized') is True, 'scope is not execution-authorized')
    require(scope.get('output_path')==output_path, 'output differs from declared path')
    output=Path(output_path);attempt=Path(output_path+'.ATTEMPT.json')
    require(output.is_absolute() and str(output)==output_path and output.parent.resolve()==output.parent and output.parent.is_dir(), 'output parent is not canonical')
    index.admit(scope['source_sha256'])
    require(str(Path(__file__).resolve()) in index.pins and str(Path(sys.executable).resolve()) in index.pins, 'helper or interpreter source not pinned')
    for path in (output,attempt):
        require(str(path) not in index.pins and not os.path.lexists(path), 'output or attempt already exists or aliases source')
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW
    marker=os.open(attempt,flags,0o600)
    try:write_fd(marker,canonical({'schema':'work-leaf-c08-observed-lower-attempt-v1','scope':scope_ref,'output':output_path,'retry_allowed':False})+b'\n')
    finally:os.close(marker)
    stream=os.open(output,flags,0o600)
    result={'schema':'work-leaf-c08-observed-lower-result-v1','status':'unqualified','errors':[],
            'candidates':[{'run_id':run,'status':'unqualified','bounds':None,'errors':[]} for run in RUNS],
            'baselines':[{'run_id':run,'status':'reused-source-not-yet-qualified'} for run in BASES],
            'accounting_calls':0,'baseline_ledger_calls':0,'provider_calls':0,'hidden_call_completeness_proven':False}
    try:
        try:
            population=materialize(scope,index)
            for path in (output,attempt): require(str(path) not in index.pins, 'nested source aliases publication')
            index.verify()
            arithmetic=load_arithmetic(index.read(scope['arithmetic']['accounting']),index.read(scope['arithmetic']['strict']))
            result=qualify(population,arithmetic)
        except (ValueError,KeyError,TypeError,IndexError,AttributeError,OSError,UnicodeError):
            result['errors'].append('source-population-qualification-failed')
        try:index.verify()
        except (ValueError,OSError,KeyError,TypeError):result['errors'].append('source-final-endpoint-failed')
        if result['errors']:
            result['status']='unqualified'
            for row in result['candidates']:row.update(status='unqualified',bounds=None)
        result.update(scope_sha256=expected_scope_sha256,source_sha256=dict(index.pins))
        encoded=canonical(result)+b'\n';require(len(encoded)<=32*1024**2,'result exceeds publication bound')
        write_fd(stream,encoded)
        return result
    finally:os.close(stream)


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input',required=True);parser.add_argument('--input-sha256',required=True);parser.add_argument('--output',required=True)
    args=parser.parse_args(argv)
    try:result=execute(args.input,args.input_sha256,args.output)
    except (ValueError,OSError,KeyError,TypeError):
        print(json.dumps({'status':'not-qualified','execution_or_publication_rejected':True}));return 2
    print(json.dumps({'status':result['status'],'candidate_rows':len(result['candidates']),'baseline_rows':len(result['baselines']),'accounting_calls':0,'provider_calls':0}))
    return 0 if result['status']=='qualified_observed_lower' else 1


if __name__=='__main__':raise SystemExit(main())
