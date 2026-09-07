"""Pure retained-item identity adapter; no provider or whole-workflow audit entrypoint."""
import hashlib
import json
from pathlib import Path

ATTRIBUTION_SHA256 = 'ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740'
MAX_ROWS = 100_000
MAX_BYTES = 32 * 1024 * 1024
COUNTS = ('input_tokens', 'cached_input_tokens', 'cache_write_input_tokens')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def identity(value):
    if not isinstance(value, str) or not value:
        raise ValueError('identity must be a nonempty string')
    return value


def number(value):
    if type(value) is not int or value < 0:
        raise ValueError('counter/index must be a nonnegative integer')
    return value


def native_inventory(thread, source, rows, known_turns):
    """Only explicit native turn identities; never the latest turn_context."""
    identity(thread); identity(source)
    if any(not isinstance(t, str) or not t for t in known_turns):
        raise ValueError('unsupported known turn identity')
    items = {}
    for line, row in rows:
        if row.get('type') != 'response_item':
            continue
        payload = row.get('payload')
        if not isinstance(payload, dict):
            raise ValueError('unsupported native response item')
        item_id = payload.get('id')
        if item_id is None:
            continue  # No invented identity for anonymous native items.
        identity(item_id)
        nested = payload.get('internal_chat_message_metadata_passthrough')
        if nested is not None and not isinstance(nested, dict):
            raise ValueError('unsupported explicit native metadata')
        turns = [identity(p['turn_id']) for p in (payload, nested or {}) if 'turn_id' in p]
        if not turns or len(set(turns)) != 1 or turns[0] not in known_turns:
            raise ValueError('missing/conflicting/unowned explicit native turn')
        item = {'kind': identity(payload.get('type')), 'turn_id': turns[0],
                'source': source, 'line': number(line), 'payload_sha256': digest(canonical(payload))}
        for key in ('name', 'call_id', 'role'):
            if key in payload:
                item[key] = identity(payload[key])
        key = thread, item_id
        if key in items and items[key]['payload_sha256'] != item['payload_sha256']:
            raise ValueError('conflicting native item identity')
        items.setdefault(key, item)
    return items


def item_identity(row):
    return identity(row.get('thread_id')), identity(row.get('item_id'))


def locator(row):
    return {k: row[k] for k in ('source', 'line', 'payload_sha256', 'turn_id')}


def targets(delivery, retrieval):
    """Each native item is charged once; provenance associations are many-to-one."""
    accepted = {}
    for row in delivery['accepted_inputs']:
        native = row['native_user']; key = item_identity(native)
        if key in accepted:
            raise ValueError('reused accepted native input identity')
        accepted[key] = native
    result = {}

    def add(native, category, association):
        key = item_identity(native); loc = locator(native)
        identity(loc['source']); identity(loc['payload_sha256']); identity(loc['turn_id']); number(loc['line'])
        if key in result and result[key]['native'] != loc:
            raise ValueError('conflicting target source identity')
        item = result.setdefault(key, {'thread_id': key[0], 'item_id': key[1],
                                      'native': loc, 'categories': [], 'associations': []})
        if category not in item['categories']:
            item['categories'].append(category)
        item['associations'].append(association)

    for index, row in enumerate(delivery['reads']):
        native = row['native_user']; key = item_identity(native)
        if accepted.get(key) != native or row['thread_id'] != key[0] or row['turn_id'] != native['turn_id']:
            raise ValueError('read lacks exact accepted native input identity')
        bundle = {k: row['bundle'][k] for k in ('threshold_eligible', 'write_succeeded', 'path')}
        if type(row['eligible']) is not bool or any(type(bundle[k]) is not bool for k in ('threshold_eligible', 'write_succeeded')):
            raise ValueError('unsupported read eligibility/bundle shape')
        if bundle['path'] is not None:
            identity(bundle['path'])
        delivered = 'other_ineligible_delivery'
        if row['eligible']:
            if not bundle['write_succeeded'] or not bundle['threshold_eligible'] or row['selected_candidate'] not in {'baseline', 'inline'}:
                raise ValueError('eligible delivery lacks exact bundle/candidate evidence')
            delivered = 'eligible_bundle_manifest' if row['selected_candidate'] == 'baseline' else 'eligible_inline_candidate'
        add(native, 'direct_read_input', {'read_index': index, 'trace_line': number(row['trace_line']),
            'sequence': number(row['sequence']), 'selected_candidate': identity(row['selected_candidate']),
            'selected_sha256': identity(row['selected_sha256']), 'eligible': row['eligible'],
            'bundle': bundle, 'delivered_class': delivered,
            'snapshot_classes': [identity(s['class']) for s in row['snapshot_metadata']]})
    for row in retrieval['candidates']:
        native = {'thread_id': row['thread_id'], 'turn_id': row['turn_id'],
                  'item_id': row['native_output_item_id'], 'source': row['native_source'],
                  'line': row['native_output_line'], 'payload_sha256': row['native_output_payload_sha256']}
        add(native, 'retrieval_candidate_output', {
            'candidate_index': number(row['candidate_index']), 'call_id': identity(row['call_id']),
            'classification': identity(row['classification']),
            'read_indices': sorted({number(r['read_index']) for r in row['references'] if r['read_index'] is not None}),
            'unlinked_read_references': sum(r['read_index'] is None for r in row['references']),
            'referenced_bundle_paths': sorted({identity(r['path']) for r in row['references']}),
            'witnessed_bundle_paths': sorted({identity(r['bundle_path']) for r in row['witnesses']}),
            'complete_bundle_paths': sorted({identity(p) for p in row.get('complete_bundle_paths', [])})})
    return result


def load_extractor(path):
    """Compile captured, pinned bytes; no importlib cache/dependency audit calls."""
    path = Path(path).resolve(); data = path.read_bytes()
    if digest(data) != ATTRIBUTION_SHA256:
        raise ValueError('frozen attribution source mismatch')
    namespace = {'__file__': str(path), '__name__': 'frozen_read_item_attribution'}
    exec(compile(data, str(path), 'exec'), namespace)
    return namespace['extract_records']


def validate_targets(target_items, native_items):
    for key, target in target_items.items():
        actual = native_items.get(key)
        if not actual or any(actual.get(k) != value for k, value in target['native'].items()):
            raise ValueError('target lacks exact native source/line/payload/turn identity')
        if 'direct_read_input' in target['categories']:
            if actual.get('kind') != 'message' or actual.get('role') != 'user':
                raise ValueError('read input is not a native user message')
        if 'retrieval_candidate_output' in target['categories']:
            if actual.get('kind') not in {'function_call_output', 'custom_tool_call_output', 'tool_search_output'}:
                raise ValueError('retrieval output is not a native tool output')


def project(attribution, target_items, *, max_rows=MAX_ROWS, max_bytes=MAX_BYTES):
    """Metadata-only whole-item charges, complete targets, no per-bundle price."""
    full = canonical(attribution)
    if len(full) > max_bytes:
        raise ValueError('full attribution exceeds byte ceiling')
    full_hash = digest(full)
    rows = len(attribution['responses']) + len(target_items)
    for response in attribution['responses']:
        rows += len(response['input_items']) + len(response['output_items'])
    if rows > max_rows:
        raise ValueError('full attribution exceeds row ceiling')
    items = {key: {**item, 'charges': [], 'all_charge_responses_exact': True}
             for key, item in target_items.items()}
    responses = []; observed = set(); previous_sequence = -1
    for response in attribution['responses']:
        rid = identity(response['response_id']); thread = identity(response['thread_id'])
        identity(response['turn_id']); sequence = number(response['sequence'])
        if rid in observed or sequence <= previous_sequence:
            raise ValueError('duplicate or unordered extracted response identity')
        observed.add(rid); previous_sequence = sequence
        responses.append({key: response.get(key) for key in (
            'response_id', 'thread_id', 'turn_id', 'sequence', 'source', 'line',
            'status', 'errors', 'attribution_sha256', 'residual', 'request_fields')})
        seen_items = set()
        for charge in response['input_items']:
            item_id = identity(charge['item_id'])
            if item_id in seen_items:
                raise ValueError('duplicate extracted input item')
            seen_items.add(item_id)
            item = items.get((thread, item_id))
            if item is None:
                continue
            counts = {key: number(charge[key]) for key in COUNTS}
            if counts['cached_input_tokens'] + counts['cache_write_input_tokens'] > counts['input_tokens']:
                raise ValueError('target cached/write fields exceed input')
            exact = response['status'] == 'exact_observed_attribution'
            item['charges'].append({'response_id': rid, 'turn_id': response['turn_id'],
                'source': response.get('source'), 'line': response.get('line'),
                'charge_position': len(item['charges']) + 1, 'response_exact': exact, **counts})
            item['all_charge_responses_exact'] &= exact
    for item in items.values():
        item['coverage'] = ('observed_completed_response_charges' if item['charges']
                            else 'not_observed_in_completed_response_attribution')
        if not item['charges']:
            item['all_charge_responses_exact'] = None
    result = {'schema': 'work-leaf-retained-read-item-recharge-v1',
              'status': attribution['status'], 'errors': attribution['errors'],
              'full_attribution_sha256': full_hash, 'full_attribution_bytes': len(full),
              'unidentified_response_records': attribution['unidentified_response_records'],
              'native_response_ids_without_raw': attribution['native_response_ids_without_raw'],
              'duplicate_responses': attribution['duplicate_responses'],
              'items': list(items.values()), 'responses': responses,
              'scope': 'Whole-item observed charges, never per-bundle bytes, net effects or exhaustive tail coverage.'}
    projected_rows = len(responses) + len(items) + len(result['unidentified_response_records'])
    projected_rows += sum(len(item['charges']) + len(item['associations']) for item in items.values())
    if projected_rows > max_rows:
        raise ValueError('projected evidence exceeds row ceiling')
    if len(canonical(result)) > max_bytes:
        raise ValueError('projected evidence exceeds byte ceiling')
    return result
