#!/usr/bin/env python3
"""Separate offline pre-turn compaction context attestation, never source repair.

The sole accounting predicate extension recognizes an exact, separately proved
future native context for a response explicitly identified as the pre-turn
compaction. Native rows, physical lines, prefix arithmetic and original failures
remain intact. The CLI exposes eligibility only; audit_run is for an independently
declared corrected accounting scope. No provider or configuration operation exists.

Public indexes are shared across native sources in one workflow. Qualifying compactions must
have distinct turns; their disjoint response-to-context spans are scanned once.
Repeated source compilation/hashing per native thread is O(T*D) for T threads and
fixed dependency bytes D, not an event-pair search. No private content is exported.
"""
import argparse
from collections import defaultdict
import copy
import hashlib
import json
from pathlib import Path
import re
import stat
import types

HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[1]
FROZEN = STUDY / 'phases/candidate-screen-01/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z'
ORIGINAL_PATH = FROZEN / 'accounting_untracked_reads.py'
ORIGINAL_SHA = 'c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a'
NATIVE_SHA = 'dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153'
PREDICATE = 'if not valid_identity(turn) or contexts.get(turn) != (model, effort):'
DERIVED_PREDICATE = ('if not valid_identity(turn) or (contexts.get(turn) != (model, effort) '
                     'and not _future_context_allowed(line, payload, model, effort)):')
NO_EXPECTED_DIGEST = object()


def require(value, message):
    if not value:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False, allow_nan=False).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def identity(value):
    require(type(value) is str and bool(value), 'identity must be a nonempty string')
    return value


def rpc(value):
    require(type(value) in (str, int) and (type(value) is int or value), 'RPC identity must retain its JSON type')
    return type(value).__name__, value


def read_bytes(path):
    path = Path(path)
    require(path.is_absolute() and path.resolve() == path and stat.S_ISREG(path.lstat().st_mode), 'source is not a canonical regular file')
    size = path.stat().st_size
    require(size <= 512 * 1024 * 1024, 'source exceeds bounded file size')
    data = path.read_bytes()
    require(len(data) == size, 'source changed while reading')
    return data


def compile_source(path, expected):
    data = read_bytes(path)
    require(sha(data) == expected, 'source digest differs')
    module = types.ModuleType('compaction_context_exact_' + Path(path).stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    module._executed_source_sha256 = sha(data)
    return module


def load_original():
    return compile_source(ORIGINAL_PATH, ORIGINAL_SHA)


def text_input(parts, native=False):
    require(isinstance(parts, list) and parts, 'missing complete text input')
    allowed = {'input_text', 'text'} if native else {'text'}
    require(all(isinstance(p, dict) and p.get('type') in allowed and type(p.get('text')) is str
                and ('text_elements' not in p or (type(p['text_elements']) is list and not p['text_elements']))
                for p in parts), 'unsupported input content or text metadata')
    if not native:
        require(len(parts) == 1, 'public/request input must be one text item')
    return ''.join(p['text'] for p in parts)


def explicit_turn(payload):
    metadata = payload.get('internal_chat_message_metadata_passthrough')
    require(isinstance(metadata, dict), 'native user lacks explicit passthrough metadata')
    turn = identity(metadata.get('turn_id'))
    direct = payload.get('turn_id')
    require(direct is None or (type(direct) is str and direct == turn), 'native direct/nested turn identities conflict')
    return turn, metadata


def public_index(captures):
    turns = {}; threads = {}; responses = defaultdict(list); response_turns = defaultdict(list)
    starts = defaultdict(list); ends = defaultdict(list); users = defaultdict(list)
    for capture in captures:
        path = capture['path']; replies = {}; forwarded = {}; requests = {}
        for line, frame in enumerate(capture['servers'], 1):
            if 'id' in frame and 'method' not in frame:
                key = rpc(frame['id']); require(key not in replies, 'duplicate public RPC reply'); replies[key] = line, frame
        for line, frame in enumerate(capture['forwarded'], 1):
            if frame.get('method') not in ('thread/start', 'thread/resume', 'turn/start'): continue
            key = rpc(frame.get('id')); require(key not in forwarded, 'duplicate forwarded RPC'); forwarded[key] = frame
        for line, frame in enumerate(capture['clients'], 1):
            method = frame.get('method')
            if method not in ('thread/start', 'thread/resume', 'turn/start'): continue
            key = rpc(frame.get('id')); require(key not in requests, 'duplicate original RPC'); requests[key] = frame
            expected = frame
            if method == 'thread/start' and capture.get('metadata_forwarding_verified') is True:
                # collect() derives this internal flag from the pinned complete
                # rewrite journal + endpoint hashes, never from a saved boolean.
                expected = copy.deepcopy(frame); expected['params']['experimentalRawEvents'] = True
            require(key in forwarded and canonical(forwarded.pop(key)) == canonical(expected), 'original/forwarded RPC bytes differ')
            require(key in replies, 'missing accepted RPC reply'); reply_line, reply = replies[key]
            require('error' not in reply, 'rejected RPC cannot prove context')
            if method != 'turn/start':
                thread = identity(reply.get('result', {}).get('thread', {}).get('id'))
                require(thread not in threads, 'ambiguous captured thread ownership')
                threads[thread] = dict(capture=path, rpc_id=frame['id'], client_line=line, reply_line=reply_line, params=frame['params'])
                continue
            params = frame['params']; thread = identity(params.get('threadId'))
            turn = identity(reply.get('result', {}).get('turn', {}).get('id')); scope = thread, turn
            require(scope not in turns, 'duplicate accepted turn')
            turns[scope] = dict(capture=path, rpc_id=frame['id'], client_line=line, reply_line=reply_line,
                                text=text_input(params.get('input')), params=params)
        require(not forwarded, 'extra forwarded RPC')
        for line, frame in enumerate(capture['servers'], 1):
            method = frame.get('method'); params = frame.get('params', {})
            if method == 'rawResponse/completed':
                response = identity(params.get('responseId'))
                responses[response].append(dict(capture=path, line=line, thread_id=identity(params.get('threadId')),
                    turn_id=identity(params.get('turnId')), usage=params.get('usage')))
                response_turns[params['threadId'], params['turnId']].append(responses[response][-1])
            if method not in ('item/started', 'item/completed'): continue
            item = params.get('item', {})
            if item.get('type') not in ('contextCompaction', 'userMessage'): continue
            scope = identity(params.get('threadId')), identity(params.get('turnId'))
            witness = dict(capture=path, line=line, item_id=identity(item.get('id')))
            if item['type'] == 'contextCompaction':
                (starts if method == 'item/started' else ends)[scope].append(witness)
            elif method == 'item/completed':
                witness['text'] = text_input(item.get('content')); users[scope].append(witness)
    return dict(turns=turns, threads=threads, responses=responses, response_turns=response_turns, starts=starts, ends=ends, users=users)


def prove_future_contexts(rows, metadata, captures, model, effort, source, *, public_cache=None):
    thread = identity(metadata.get('thread_id'))
    require((metadata.get('model'), metadata.get('effort')) == (model, effort), 'metadata model/effort differs')
    contexts = defaultdict(list); usages = defaultdict(list); native_users = defaultdict(list); candidates = []
    sessions = []; seen_contexts = {}
    for line, row in enumerate(rows, 1):
        kind = row.get('type'); payload = row.get('payload', {})
        if kind == 'session_meta': sessions.append(payload)
        if kind == 'turn_context':
            turn = identity(payload.get('turn_id')); contexts[turn].append((line, payload))
            seen_contexts[turn] = payload.get('model'), payload.get('effort')
        if kind == 'token_usage_record':
            rid = identity(payload.get('response_id')); turn = identity(payload.get('turn_id'))
            usages[rid].append((line, payload))
            if seen_contexts.get(turn) != (model, effort): candidates.append((line, payload))
        if kind == 'response_item' and payload.get('type') == 'message' and payload.get('role') == 'user':
            meta = payload.get('internal_chat_message_metadata_passthrough', {})
            if isinstance(meta, dict) and meta.get('content_item_kinds') == ['user.text']:
                turn, _ = explicit_turn(payload)
                native_users[turn].append((line, identity(payload.get('id')), text_input(payload.get('content'), native=True)))
    if not candidates:
        return []
    require(len(sessions) == 1 and sessions[0].get('id') == thread and sessions[0].get('cwd') == metadata.get('cwd'), 'native session/cwd scope differs')
    if public_cache is None: public_cache = {}
    if 'index' not in public_cache: public_cache['index'] = public_index(captures)
    public = public_cache['index']; original = load_original(); proofs = []; used_turns = set()
    rows_hash = sha(canonical(rows))
    for line, record in candidates:
        rid, turn = record['response_id'], record['turn_id']; scope = thread, turn
        require(record.get('thread_id') == thread and record.get('session_id') == thread, 'native response thread/session differs')
        require(len(usages[rid]) == 1 and turn not in used_turns, 'ambiguous native response/compaction turn')
        used_turns.add(turn)
        require(line < len(rows) and rows[line].get('type') == 'compacted', 'response lacks adjacent explicit compaction')
        marker = rows[line]['payload']
        require(marker.get('compaction_response_id') == rid and canonical(marker.get('latest_token_usage_record')) == canonical(record), 'explicit compaction identity/latest record differs')
        require(len(contexts[turn]) == 1, 'future turn context is missing or ambiguous')
        context_line, context = contexts[turn][0]
        require(context_line > line + 1 and (context.get('model'), context.get('effort')) == (model, effort)
                and context.get('cwd') == metadata.get('cwd'), 'future model/effort/cwd context differs or is not later')
        native_completion_ids = []
        for intervening in rows[line + 1:context_line - 1]:
            kind, payload = intervening.get('type'), intervening.get('payload', {})
            metadata_event = kind == 'event_msg' and payload.get('type') in ('thread_settings_applied', 'token_count')
            if kind == 'event_msg' and payload.get('type') == 'item_completed':
                item = payload.get('item', {})
                require(item.get('type') == 'ContextCompaction' and payload.get('thread_id') == thread
                        and payload.get('turn_id') == turn, 'intervening native completion is not the same compaction')
                native_completion_ids.append(identity(item.get('id'))); metadata_event = True
            require(metadata_event or kind == 'world_state' or (kind == 'response_item' and payload.get('type') == 'message'
                    and payload.get('role') in ('system', 'developer', 'user')
                    and payload.get('internal_chat_message_metadata_passthrough', {}).get('content_item_kinds') != ['user.text']),
                    'generation or ambiguous boundary intervenes before context')
        require(len(native_users[turn]) == 1, 'native request input is missing or ambiguous')
        user_line, native_id, native_text = native_users[turn][0]
        require(user_line > context_line, 'native user precedes future context')
        require(scope in public['turns'] and thread in public['threads'], 'native compaction lacks captured accepted thread/turn')
        request = public['turns'][scope]; owner = public['threads'][thread]
        require(owner['capture'] == request['capture'] and owner['reply_line'] < request['reply_line'], 'thread acceptance does not precede turn')
        for public_scope in (owner['params'], request['params']):
            for key, expected in (('model', model), ('effort', effort), ('reasoningEffort', effort), ('cwd', metadata['cwd'])):
                require(public_scope.get(key) is None or (type(public_scope[key]) is str and public_scope[key] == expected), 'explicit public ' + key + ' differs from native scope')
            require(public_scope.get('config') in (None, {}), 'unsupported explicit public configuration overrides')
        require(len(public['responses'][rid]) == len(public['starts'][scope]) == len(public['ends'][scope]) == len(public['users'][scope]) == 1,
                'compaction response/lifecycle/public user is missing or ambiguous')
        response = public['responses'][rid][0]; start = public['starts'][scope][0]; end = public['ends'][scope][0]; user = public['users'][scope][0]
        require((response['thread_id'], response['turn_id']) == scope and start['item_id'] == end['item_id'], 'public compaction identities differ')
        require(all(x['capture'] == request['capture'] for x in (response, start, end, user)), 'public witnesses cross captures')
        require(request['reply_line'] < start['line'] < response['line'] < end['line'] < user['line'], 'public compaction/request order differs')
        window = [r for r in public['response_turns'][scope] if r['capture'] == request['capture'] and start['line'] < r['line'] < end['line']]
        require(len(window) == 1, 'multiple response identities occur within the public compaction lifecycle')
        require(not native_completion_ids or native_completion_ids == [start['item_id']], 'native/public compaction completion identity differs')
        require(request['text'] == native_text == user['text'], 'complete accepted/public/native input bytes differ')
        require(original.checked_usage(record.get('usage'), camel=False) == original.checked_usage(response['usage']), 'raw/native exact response usage differs')
        # The model fallback is bound to the complete unchanged source, not only IDs.
        proofs.append(dict(source=source, thread_id=thread, turn_id=turn, response_id=rid,
            model=model, effort=effort, cwd=metadata['cwd'], native_response_line=line, native_marker_line=line+1,
            native_context_line=context_line, native_user_line=user_line, native_user_item_id=native_id,
            native_rows_sha256=rows_hash, native_metadata_sha256=sha(canonical(metadata)), native_response_payload_sha256=sha(canonical(record)),
            capture=request['capture'], rpc_id=request['rpc_id'], original_client_line=request['client_line'],
            accepted_reply_line=request['reply_line'], public_compaction_start_line=start['line'],
            raw_response_line=response['line'], public_compaction_complete_line=end['line'],
            public_compaction_item_id=start['item_id'], public_user_line=user['line'], public_user_item_id=user['item_id'],
            input_sha256=sha(native_text.encode()), basis='exact explicit pre-turn compaction and unique later native context; no elapsed-time inference'))
    return proofs


def derived_native(original, proofs):
    path = Path(original.NATIVE.__file__); data = read_bytes(path)
    require(sha(data) == NATIVE_SHA and data.decode().count(PREDICATE) == 1, 'native predicate source seam differs')
    derived = data.decode().replace(PREDICATE, DERIVED_PREDICATE, 1)
    module = types.ModuleType('compaction_context_derived_native'); module.__file__ = str(path)
    active = {}
    def allowed(line, payload, model, effort):
        proof = active.get((line, payload.get('response_id')))
        return bool(proof and (model, effort) == (proof['model'], proof['effort'])
                    and sha(canonical(payload)) == proof['native_response_payload_sha256'])
    module.__dict__['_future_context_allowed'] = allowed
    exec(compile(derived, str(path), 'exec'), module.__dict__)
    inner = module.audit_rollout
    def checked(rows, metadata, model, effort):
        active.clear()
        rows_hash = sha(canonical(rows))
        for proof in proofs:
            if (proof['native_rows_sha256'] == rows_hash and proof['thread_id'] == metadata.get('thread_id')
                    and proof['native_metadata_sha256'] == sha(canonical(metadata))):
                active[proof['native_response_line'], proof['response_id']] = proof
        return inner(rows, metadata, model, effort)
    module.audit_rollout = checked
    module._executed_derived_sha256 = sha(derived.encode())
    return module


def corrected_ledger(original, rows, metadata, model, effort, source, proofs):
    native = derived_native(original, proofs)
    fn = original.native_ledger
    bound = types.FunctionType(fn.__code__, {**fn.__globals__, 'NATIVE': native}, fn.__name__, fn.__defaults__)
    return bound(rows, metadata, model, effort, source)


class Sources:
    def __init__(self):
        self.hashes = {}

    def read(self, path, expected=NO_EXPECTED_DIGEST):
        require(expected is NO_EXPECTED_DIGEST or (type(expected) is str and re.fullmatch('[0-9a-f]{64}', expected)), 'invalid explicit source digest')
        path = str(Path(path)); data = read_bytes(path); digest = sha(data)
        require(expected is NO_EXPECTED_DIGEST or digest == expected, 'source digest differs: ' + path)
        require(path not in self.hashes or self.hashes[path] == digest, 'source identity drift')
        self.hashes[path] = digest
        return data

    def rows(self, path):
        data = self.read(path); require(not data or data.endswith(b'\n'), 'incomplete closed JSONL source')
        return [json.loads(line) if line.strip() else {} for line in data.splitlines()]

    def finish(self):
        for path, digest in self.hashes.items():
            require(sha(read_bytes(path)) == digest, 'source changed at endpoint: ' + path)


def collect(entry, frozen, sessions_root, original, sources):
    sources.read(ORIGINAL_PATH, ORIGINAL_SHA); sources.read(Path(__file__).resolve())
    pins = [x for x in frozen['files'] if x.get('path') == str(ORIGINAL_PATH)]
    require(len(pins) == 1 and pins[0].get('sha256') == ORIGINAL_SHA and pins[0].get('role') == 'frozen-evidence', 'original helper lacks unique frozen identity')
    for path, digest in original.PINS.items(): sources.read(path, digest)
    require(entry.get('started_at') and entry.get('artifact'), 'unlaunched outcome remains unavailable')
    require((frozen.get('model'), frozen.get('reasoning_effort')) == ('gpt-5.5', 'xhigh'), 'frozen model/effort differs')
    observation = Path(entry['artifact']) / 'observation'; captures = []
    for app in sorted((observation / 'app-server').iterdir()):
        if not app.is_dir(): continue
        provenance = original.BASE.capture_provenance(app)
        require(not provenance['errors'], 'capture provenance: ' + '; '.join(provenance['errors']))
        for path, digest in provenance['source_sha256'].items(): sources.read(path, digest)
        captures.append(dict(path=str(app), metadata_forwarding_verified=True, clients=sources.rows(app/'client-to-server.raw'),
            forwarded=sources.rows(app/'client-to-server.forwarded.raw'), servers=sources.rows(app/'server-to-client.raw')))
    require(captures, 'no closed captures')
    sessions_root = Path(sessions_root).resolve(); native = []; seen = set()
    for metadata in sources.rows(observation/'rollout-metadata.jsonl'):
        tid = identity(metadata.get('thread_id')); require(tid not in seen and not metadata.get('descendant'), 'duplicate/descendant native scope')
        seen.add(tid); relative = Path(metadata['source_relative_path']); path = sessions_root/relative
        require(not relative.is_absolute() and path.resolve() == path and path.is_relative_to(sessions_root), 'native source escapes declared root')
        sources.read(path, metadata['source_sha256']); rows = sources.rows(path)
        native.append((str(path), rows, metadata))
    require(native, 'no native inventory')
    return captures, native


def audit_eligibility(entry, frozen, sessions_root, sources=None):
    sources = sources or Sources(); original = load_original()
    result = dict(schema='work-leaf-compaction-context-eligibility-v1', run_id=entry.get('run_id'), status='unverifiable',
                  errors=[], proofs=[], native_sources=[], whole_workflow_totals_computed=False, token_values_exported=False,
                  outcome={key: entry.get(key) for key in ('launch_status', 'launcher_exit_code', 'outcome_classification', 'started_at', 'finished_at')},
                  scope='Native prefix eligibility only; no whole-workflow accounting or accounting.audit_run invocation.')
    try:
        captures, native = collect(entry, frozen, sessions_root, original, sources)
        identities = set(); public_cache = {}
        for path, rows, metadata in native:
            old = original.NATIVE.audit_rollout(rows, metadata, frozen['model'], frozen['reasoning_effort'])
            proofs = prove_future_contexts(rows, metadata, captures, frozen['model'], frozen['reasoning_effort'], path, public_cache=public_cache)
            module = derived_native(original, proofs)
            checked = module.audit_rollout(rows, metadata, frozen['model'], frozen['reasoning_effort'])
            result['native_sources'].append(dict(path=path, thread_id=metadata['thread_id'], original_errors=old['errors'], corrected_errors=checked['errors']))
            result['proofs'].extend(proofs)
            require(not checked['errors'], 'native prefix remains ineligible: ' + '; '.join(checked['errors']))
            require(not identities.intersection(checked['records']), 'response IDs cross native threads')
            identities.update(checked['records'])
            result['executed_derived_native_sha256'] = module._executed_derived_sha256
        result['status'] = 'eligible'
    except (ValueError, KeyError, TypeError, OSError, IndexError) as error:
        result['errors'].append(str(error))
    try: sources.finish()
    except (ValueError, OSError) as error: result['errors'].append(str(error))
    if result['errors']: result['status'] = 'unverifiable'
    result['source_sha256'] = sources.hashes
    return result


def audit_run(entry, frozen, sessions_root):
    """For a separately declared corrected all-outcome scope; not used by this CLI."""
    sources = Sources(); original = load_original(); proofs = []; old_errors = []
    try:
        captures, native = collect(entry, frozen, sessions_root, original, sources)
        by_source = {}; public_cache = {}
        for path, rows, metadata in native:
            proven = prove_future_contexts(rows, metadata, captures, frozen['model'], frozen['reasoning_effort'], path, public_cache=public_cache)
            by_source[path] = proven; proofs.extend(proven)
            old_errors.append(dict(path=path, errors=original.NATIVE.audit_rollout(rows, metadata, frozen['model'], frozen['reasoning_effort'])['errors']))
        base_ledger = original.native_ledger
        def bound(rows, metadata, model, effort, source):
            native_module = derived_native(original, by_source.get(source, []))
            fn = types.FunctionType(base_ledger.__code__, {**base_ledger.__globals__, 'NATIVE': native_module}, base_ledger.__name__, base_ledger.__defaults__)
            return fn(rows, metadata, model, effort, source)
        original.native_ledger = bound
        result = original.audit_run(entry, frozen, sessions_root)
        sources.finish()
    except (ValueError, KeyError, TypeError, OSError, IndexError) as error:
        result = dict(run_id=entry.get('run_id'), status='unknown', errors=[str(error)], measurement=dict(status='ineligible', bounds=None), source_sha256={})
    result['source_sha256'].update(sources.hashes)
    result['compaction_context_derivative'] = dict(helper_sha256=sha(read_bytes(Path(__file__).resolve())),
        original_helper_sha256=ORIGINAL_SHA, native_helper_sha256=NATIVE_SHA,
        original_native_errors=old_errors, proofs=proofs,
        scope='Only exact preproved pre-turn compaction context eligibility; original prefix arithmetic, raw scope, tails and outcomes remain unchanged.')
    return result


def main():
    parser = argparse.ArgumentParser(description='Eligibility only; no accounting totals')
    parser.add_argument('--input', required=True); parser.add_argument('--input-sha256', required=True); parser.add_argument('--output', required=True)
    args = parser.parse_args(); sources = Sources()
    value = json.loads(sources.read(args.input, args.input_sha256))
    require(value.get('schema') == 'work-leaf-compaction-context-input-v1', 'unknown correction input')
    sources.read(Path(__file__).resolve(), value['helper_sha256'])
    def read_ref(ref): return json.loads(sources.read(ref['path'], ref['sha256']))
    frozen = read_ref(value['phase_manifest']); score = read_ref(value['score_manifest'])
    rows = [x for x in score['runs'] if x['run_id'] == value['run_id']]; require(len(rows) == 1, 'ambiguous declared workflow')
    result = audit_eligibility(rows[0], frozen, value['sessions_root'], sources)
    output = Path(args.output); require(output.is_absolute() and output.parent.resolve() == output.parent, 'output parent is not canonical')
    with output.open('x') as stream: json.dump(result, stream, indent=2, sort_keys=True); stream.write('\n')
    return 0 if result['status'] == 'eligible' else 1


if __name__ == '__main__':
    raise SystemExit(main())
