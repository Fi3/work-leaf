"""Pure fixed-population C08 adapter; actual source loading/admission belongs to its caller."""
import hashlib
import json
from pathlib import Path
import types

RUN_LINES = {
    'automatic-refresh-01-workflow-001': (4, 9),
    'automatic-refresh-01-workflow-002': (4, 13, 19, 23, 28, 31),
    'automatic-refresh-01-workflow-003': (4, 9, 14),
}
RECHARGE_SHA = '4d6ac35d6bb857e6c34564020f035f6fdaf4a7f43fbe8d4e8d682fb78990ad0c'
HERE = Path(__file__).resolve().parent
RECHARGE_PATH = HERE.parent / 'read-item-recharge/recharge.py'
ATTRIBUTION_PATH = HERE.parents[1] / 'audit_input_attribution.py'
FIELDS = ('input_tokens', 'cached_input_tokens', 'output_tokens', 'reasoning_output_tokens')


def require(value, reason):
    if not value:
        raise ValueError(reason)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def identity(value):
    require(type(value) is str and bool(value), 'invalid identity')
    return value


def positive(value):
    require(type(value) is int and value > 0, 'invalid physical line/count')
    return value


def load_recharge():
    data = RECHARGE_PATH.read_bytes()
    require(hashlib.sha256(data).hexdigest() == RECHARGE_SHA, 'recharge dependency differs')
    module = types.ModuleType('frozen_c08_recharge')
    module.__file__ = str(RECHARGE_PATH)
    exec(compile(data, str(RECHARGE_PATH), 'exec'), module.__dict__)
    return module


def source_membership(proof, run):
    require(proof['run_id'] == run and proof['status'] == 'unverifiable'
            and proof['errors'] == ['trace-occurrence-not-joined'] * 2, 'source qualification differs')
    require(proof['native_membership']['errors'] == [] and proof['project_inventory']['errors'] == [],
            'source subproof failed')
    require(proof['invocation_streams'] and all(x['errors'] == [] for x in proof['invocation_streams']),
            'stream subproof failed')
    require(proof['frame_proofs'] and all(x['turn_closure']['errors'] == [] for x in proof['frame_proofs']),
            'frame subproof failed')
    sources = {}
    for row in proof['native_membership']['threads']:
        thread, source = identity(row['thread_id']), identity(row['source'])
        require(thread not in sources, 'duplicate thread')
        sources[thread] = source
    require(len(set(sources.values())) == len(sources), 'native source aliases threads')
    accepted, contexts = {}, set()
    for row in proof['native_membership']['contexts']:
        key = identity(row['thread_id']), identity(row['turn_id'])
        require(key[0] in sources and key not in contexts, 'duplicate/foreign native context')
        contexts.add(key)
    for row in proof['delivery']['inputs']:
        key = identity(row['thread_id']), identity(row['turn_id'])
        require(row['status'] == 'joined' and key not in accepted and key[0] in sources
                and row['native_source'] == sources[key[0]], 'accepted input unavailable')
        accepted[key] = row
    require(set(accepted) == contexts, 'accepted/context population differs')
    return sources, accepted


def prepare(trace_report, source_results, lower_result, native_rows, raw_rows, target_payload_hashes):
    """Materialized inputs only. No saved source collector, arithmetic or extraction."""
    require(tuple(row['run_id'] for row in trace_report['runs']) == tuple(RUN_LINES)
            and set(source_results) == set(raw_rows) == set(RUN_LINES), 'fixed run population differs')
    require(lower_result['status'] == 'qualified_observed_lower' and lower_result['errors'] == []
            and tuple(row['run_id'] for row in lower_result['candidates']) == tuple(RUN_LINES),
            'lower response qualification unavailable')
    payload_hashes = {}
    for row in target_payload_hashes:
        key = row['run_id'], positive(row['trace_line'])
        require(key not in payload_hashes, 'duplicate target payload hash')
        payload_hashes[key] = identity(row['payload_sha256'])
    require(set(payload_hashes) == {(run, line) for run, lines in RUN_LINES.items() for line in lines},
            'target payload population differs')
    recharge = load_recharge()
    result, global_responses = [], set()
    for trace, lower in zip(trace_report['runs'], lower_result['candidates']):
        run = trace['run_id']; proof = source_results[run]
        sources, accepted = source_membership(proof, run)
        require(all(type(row['eligible']) is bool for row in trace['records']), 'invalid eligibility')
        selected = [row for row in trace['records'] if row['eligible']]
        require(tuple(row['trace_line'] for row in selected) == RUN_LINES[run], 'fixed target population differs')
        owner_threads = {identity(row['input']['thread_id']) for row in selected}
        require(len(owner_threads) == 2 and owner_threads <= set(sources), 'target author population differs')
        known = {thread: set() for thread in owner_threads}
        for thread, turn in accepted:
            if thread in known:
                known[thread].add(turn)
        inventory, physical = {}, {}
        for thread in sorted(owner_threads):
            source = sources[thread]; rows = native_rows[source]
            require(source in proof['source_sha256'], 'native source absent from source proof')
            previous = 0; byline = {}
            for line, row in rows:
                require(positive(line) > previous and type(row) is dict, 'native physical rows differ')
                previous = line; byline[line] = row
            physical[source] = byline
            inventory.update(recharge.native_inventory(thread, source, rows, known[thread]))
        targets, first_public, first_native = {}, {}, {}
        for row in selected:
            inp = row['input']; thread, turn = inp['thread_id'], inp['turn_id']
            require(inp == accepted.get((thread, turn)) and row['event'] == 'automatic-refresh'
                    and row['site'] in ('automatic-edit-refresh', 'automatic-patch-refresh')
                    and row['direct_full_text_join'] is True and row['unowned_ranges_identical'] is True
                    and row['saved_trace_status'] == 'joined', 'target lacks exact accepted refresh proof')
            key = thread, identity(inp['native_item_id'])
            require(key not in targets, 'duplicate native target')
            payload = physical[inp['native_source']][positive(inp['native_line'])]['payload']
            require(payload['type'] == 'message' and payload['role'] == 'user' and payload['id'] == key[1],
                    'automatic refresh is not the selected native user')
            content = payload['content']
            require(type(content) is list and content and all(type(x) is dict
                    and x.get('type') in ('input_text', 'text') and type(x.get('text')) is str
                    and x.get('text_elements', []) == [] for x in content), 'unsupported complete user text')
            text = ''.join(x['text'] for x in content).encode()
            require(type(inp['input_bytes']) is int and len(text) == inp['input_bytes']
                    and hashlib.sha256(text).hexdigest() == inp['input_sha256'], 'full input text differs')
            payload_hash = digest(payload)
            require(payload_hash == payload_hashes[run, row['trace_line']], 'full native payload differs')
            end = 0
            require(row['full_current_bodies'], 'missing owned full-current body')
            for component in row['full_current_bodies']:
                start, stop = component['body_start'], component['body_end']
                require(type(start) is int and type(stop) is int and end <= start < stop <= len(text),
                        'invalid or overlapping owned body range')
                body = text[start:stop]; body.decode('utf-8')
                require(type(component['bytes']) is int and len(body) == component['bytes']
                        and hashlib.sha256(body).hexdigest() == component['sha256'], 'owned body differs')
                end = stop
            native = {'source': inp['native_source'], 'line': inp['native_line'],
                      'turn_id': turn, 'payload_sha256': payload_hash}
            targets[key] = {'thread_id': thread, 'item_id': key[1], 'native': native,
                            'categories': ['automatic_refresh_input'],
                            'associations': [{'trace_line': row['trace_line'],
                             'input_sha256': inp['input_sha256'], 'input_bytes': len(text),
                             'public_line': inp['public_line'],
                             'current_body_bytes': [c['bytes'] for c in row['full_current_bodies']]}]}
            public_line = positive(inp['public_line'])
            first_public[thread] = min(first_public.get(thread, public_line), public_line)
            first_native[thread] = min(first_native.get(thread, inp['native_line']), inp['native_line'])
        recharge.validate_targets(targets, inventory)
        records = lower['response_evidence']
        require(lower['status'] == 'qualified_observed_lower' and lower['errors'] == []
                and type(lower['response_count']) is int and lower['response_count'] == len(records)
                and lower['response_evidence_sha256'] == digest(records), 'lower map identity differs')
        require(not global_responses.intersection(records), 'response identity crosses runs')
        global_responses.update(records)
        ledger = {}
        for rid, record in records.items():
            identity(rid); thread, turn = identity(record['thread_id']), identity(record['turn_id'])
            require((thread, turn) in accepted and all(type(record['usage'].get(f)) is int
                    and record['usage'][f] >= 0 for f in FIELDS), 'invalid qualified response scope/counter')
            if thread not in owner_threads:
                continue
            refs = [ref for ref in record['sources'] if 'native_line' in ref]
            require(refs, 'response lacks native source locator')
            later = False
            for ref in refs:
                require(ref['path'] == sources[thread], 'response source crosses thread')
                line = positive(ref['native_line']); actual = physical[ref['path']][line]
                payload = actual['payload']
                require(actual['type'] == 'token_usage_record' and payload['response_id'] == rid
                        and payload['thread_id'] == thread and payload['turn_id'] == turn
                        and all(type(payload['usage'].get(f)) is int
                                and payload['usage'][f] == record['usage'][f] for f in FIELDS),
                        'native response witness differs')
                later |= line > first_native[thread]
            if later:
                ledger[rid] = record
        servers = []; prior_line = 0; stream = None
        capture_ids = {accepted[thread, next(iter(known[thread]))]['capture_id'] for thread in owner_threads}
        require(len(capture_ids) == 1, 'owner response windows cross captures')
        for event in raw_rows[run]:
            line = positive(event['_audit_line']); path = identity(event['_audit_source'])
            require(line > prior_line and (stream is None or stream == path)
                    and path in proof['source_sha256'] and Path(path).parent.name in capture_ids,
                    'raw stream/physical order differs')
            prior_line = line; stream = path
            require(event.get('method') == 'rawResponse/completed', 'expected complete raw response rows')
            params = event.get('params'); thread = params.get('threadId') if type(params) is dict else None
            if type(thread) is not str or not thread:
                if line > min(first_public.values()):
                    servers.append(event)  # Scope cannot be assigned: retain the unresolved record.
            elif thread in owner_threads and line > first_public[thread]:
                servers.append(event)
            elif type(params) is dict and params.get('responseId') in ledger:
                servers.append(event)  # A contradictory scope for an owned ID is not a filter.
        result.append({'run_id': run, 'targets': targets, 'native': inventory,
                       'ledger': ledger, 'servers': servers,
                       'lower_response_map_sha256': lower['response_evidence_sha256'],
                       'retained_original_source_errors': list(proof['errors']),
                       'retained_original_errors': list(lower.get('original_errors') or []),
                       'accepted_turns_without_recorded_response':
                           lower.get('accepted_turns_without_recorded_response', [])})
    return result


def extract(prepared, *, max_rows=100_000, max_bytes=32 * 1024 * 1024):
    """One frozen extraction on an already admitted, materialized run; no caller CLI."""
    require(type(max_rows) is int and 0 < max_rows <= 100_000
            and type(max_bytes) is int and 0 < max_bytes <= 32 * 1024 * 1024, 'unsupported output ceilings')
    recharge = load_recharge()
    extractor = recharge.load_extractor(ATTRIBUTION_PATH)
    full = extractor(prepared['servers'], prepared['native'], prepared['ledger'])
    full_bytes = recharge.canonical(full)
    errors = list(full['errors'])
    evidence = None
    try:
        evidence = recharge.project(full, prepared['targets'], max_rows=max_rows, max_bytes=max_bytes)
        premature = set()
        for item in evidence['items']:
            introduced = item['associations'][0]['public_line']
            premature.update(charge['response_id'] for charge in item['charges'] if charge['line'] <= introduced)
        if premature:
            errors.append('target-charge-before-delivery')
            evidence['status'] = 'unknown'
            evidence['errors'] = [*evidence['errors'], 'target-charge-before-delivery']
            for response in evidence['responses']:
                if response['response_id'] in premature:
                    response['status'] = 'unknown'
                    response['errors'] = [*response['errors'], 'target-charge-before-delivery']
            for item in evidence['items']:
                for charge in item['charges']:
                    if charge['response_id'] in premature:
                        charge['response_exact'] = False
                        item['all_charge_responses_exact'] = False
    except (ValueError, KeyError, TypeError) as error:
        errors.append('projection-rejected: ' + str(error))
    missing = sorted({r['response_id'] for r in full['responses']} - set(prepared['ledger']))
    return {'schema': 'work-leaf-c08-refresh-item-recharge-v1', 'run_id': prepared['run_id'],
            'status': 'unknown' if errors else full['status'], 'errors': errors,
            'full_attribution_sha256': hashlib.sha256(full_bytes).hexdigest(),
            'full_attribution_bytes': len(full_bytes), 'extract_records_calls': 1,
            'provider_calls': 0, 'workflow_accounting_calls': 0, 'evidence': evidence,
            'raw_without_qualified_native': missing,
            'lower_response_map_sha256': prepared['lower_response_map_sha256'],
            'retained_original_source_errors': prepared['retained_original_source_errors'],
            'retained_original_errors': prepared['retained_original_errors'],
            'accepted_turns_without_recorded_response': prepared['accepted_turns_without_recorded_response'],
            'scope': 'Whole automatic-refresh input items; observed charges only, no body-substring price or net effect.'}
