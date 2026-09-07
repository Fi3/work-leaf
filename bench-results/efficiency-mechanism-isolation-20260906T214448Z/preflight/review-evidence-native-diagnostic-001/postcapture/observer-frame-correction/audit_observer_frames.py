#!/usr/bin/env python3
"""Separate source-bound observer-frame predicate; no runtime or accounting work."""
import argparse
import copy
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import types

HERE = Path(__file__).resolve().parent
DIAGNOSTIC = HERE.parents[1]
FROZEN = DIAGNOSTIC / 'infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z'
COLLECTOR = FROZEN / 'audit_review_evidence_sources.py'
PRIMITIVE = FROZEN / 'audit_review_evidence.py'
COLLECTOR_SHA = 'd3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee'
PRIMITIVE_SHA = '34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257'
OBSERVER_SHA = '238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386'
OLD = 'require(typed_equal(capture["clients"], capture["forwarded"]), "original/forwarded frames differ")'
NEW = 'require(_bound_forwarding(capture), "original/forwarded frames differ")'
SETTINGS = {'enabled': True, 'environment_variable': 'WORK_LEAF_OBSERVER_RAW_RESPONSE_USAGE',
            'forwarded_client_stream': 'client-to-server.forwarded.raw', 'interrupt_policy_unchanged': True,
            'original_client_stream': 'client-to-server.raw', 'request_metadata_overrides': {
                'initialize.params.capabilities.experimentalApi': True,
                'initialize.params.capabilities.optOutNotificationMethods.append': 'rawResponseItem/completed',
                'thread/start.params.experimentalRawEvents': True},
            'rewrite_decisions': 'raw-response-rewrites.jsonl', 'schema_version': 1}


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(value):
    return hashlib.sha256(value).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode()


def compile_exact(path, expected):
    require(path.is_absolute() and path.resolve() == path and path.is_file(), 'noncanonical executable source')
    data = path.read_bytes(); require(sha(data) == expected, 'executable source SHA differs')
    module = types.ModuleType('exact_' + path.stem); module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


@lru_cache(maxsize=1)
def original_modules():
    return compile_exact(COLLECTOR, COLLECTOR_SHA), compile_exact(PRIMITIVE, PRIMITIVE_SHA)


def prove_frames(original_raw, forwarded_raw, decisions, settings):
    collector, primitive = original_modules()
    require(primitive.typed_equal(settings, SETTINGS), 'raw metadata settings differ')
    require(original_raw.endswith(b'\n') and forwarded_raw.endswith(b'\n'), 'incomplete raw frame tail')
    before = original_raw.splitlines(keepends=True); after = forwarded_raw.splitlines(keepends=True)
    require(len(before) == len(after), 'frame inventory differs')
    original = []; forwarded = []; changed = []; journal_index = 0; seen = set()
    require(isinstance(decisions, list), 'rewrite journal must be a list')
    for line, (old, new) in enumerate(zip(before, after), 1):
        value = collector.decode(old); actual = collector.decode(new)
        require(isinstance(value, dict) and isinstance(actual, dict), 'nonobject frame')
        expected = copy.deepcopy(value); method = value.get('method')
        if 'id' in value:
            ident = value['id']; require(type(ident) in (str, int) and (type(ident) is int or ident), 'invalid typed RPC identity')
            key = type(ident).__name__, ident; require(key not in seen, 'duplicate RPC identity'); seen.add(key)
        metadata = method in ('initialize', 'thread/start')
        if metadata:
            require('id' in value and isinstance(value.get('params'), dict), 'metadata RPC lacks exact parameters')
            if method == 'initialize':
                caps = expected['params'].get('capabilities')
                require(isinstance(caps, dict) and caps.get('experimentalApi') is True, 'experimentalApi must already be true')
                notifications = caps.get('optOutNotificationMethods')
                if notifications is None: notifications = []; caps['optOutNotificationMethods'] = notifications
                require(type(notifications) is list and all(type(x) is str for x in notifications), 'unsupported notifications')
                if 'rawResponseItem/completed' not in notifications: notifications.append('rawResponseItem/completed')
            else:
                expected['params']['experimentalRawEvents'] = True
            require(journal_index < len(decisions), 'missing rewrite journal entry')
            decision = decisions[journal_index]; journal_index += 1
            require(isinstance(decision, dict), 'nonobject rewrite decision')
            time = decision.get('observed_monotonic_ns'); require(type(time) is int and time >= 0, 'invalid rewrite timestamp')
            wanted = dict(method=method, id=value['id'], changed=not primitive.typed_equal(value, expected),
                          original_sha256=sha(old), forwarded_sha256=sha(new), original_bytes=len(old), forwarded_bytes=len(new),
                          observed_monotonic_ns=time)
            require(primitive.typed_equal(decision, wanted), 'journal identity/hash/shape differs')
        require(primitive.typed_equal(actual, expected), 'nonpermitted forwarded field/type mutation')
        same = primitive.typed_equal(value, expected)
        require(not same or old == new, 'unchanged frame bytes differ')
        if not same: changed.append(line)
        original.append(value); forwarded.append(actual)
    require(journal_index == len(decisions), 'extra rewrite journal entry')
    return dict(frame_count=len(before), changed_frame_lines=changed,
                original_rows_sha256=sha(canonical(original)), forwarded_rows_sha256=sha(canonical(forwarded)),
                original_raw_sha256=sha(original_raw), forwarded_raw_sha256=sha(forwarded_raw))


def matches_proof(proof, original, forwarded):
    return (proof.get('original_rows_sha256') == sha(canonical(original))
            and proof.get('forwarded_rows_sha256') == sha(canonical(forwarded)))


def audit_sources(value, sources):
    result = dict(schema='work-leaf-review-observer-frame-derived-v1', status='unverifiable', errors=[], provider_work_started=False)
    try:
        require(value.get('schema') == 'work-leaf-review-observer-frame-input-v1', 'unsupported derivative input')
        sources.read({'path': str(Path(__file__).resolve()), 'sha256': value['helper_sha256']})
        sources.read({'path': str(COLLECTOR), 'sha256': COLLECTOR_SHA})
        primitive_data = sources.read({'path': str(PRIMITIVE), 'sha256': PRIMITIVE_SHA})
        original = sources.json(value['source_input']); config = sources.json(value['observer_config'])
        require(original['helper_sha256'] == COLLECTOR_SHA, 'original collector input pin differs')
        result.update(run_id=original['run_id'], condition=original['condition'])
        result['terminal'] = original_modules()[0].terminal(sources.json(original['terminal']), original['run_id'], original['condition'])
        observation = Path(original['invocations']['path']).parent
        require(value['observer_config']['path'] == str(observation/'observer-config.json')
                and config.get('root') == str(observation) and config.get('run_id') == original['run_id']
                and config.get('condition') == 'work-leaf' and config.get('observer_sha256') == OBSERVER_SHA,
                'observer config/root/run/executable scope differs')
        sources.read({'path': config['observer_executable'], 'sha256': OBSERVER_SHA})
        refs = {x['path']: x for x in value['captures']}
        require(len(refs) == len(value['captures']) and set(refs) == {x['path'] for x in original['captures']}, 'capture proof census differs')
        proofs = {}
        for capture in original['captures']:
            path = Path(capture['path']); extra = refs[str(path)]
            require(path.parent == observation/'app-server', 'capture root differs')
            for name, filename in [('settings', 'raw-response-usage.json'), ('journal', 'raw-response-rewrites.jsonl')]:
                require(extra[name]['path'] == str(path/filename), 'proof artifact path differs')
            settings = sources.json(extra['settings']); journal = [row for _, row in sources.records(extra['journal'])]
            start = sources.json(capture['start']); end = sources.json(capture['end'])
            require(start.get('raw_response_usage') is True and start.get('primary') is True
                    and start.get('capture_kind') == 'app-server' and start.get('invocation_id') == end.get('invocation_id') == path.name,
                    'raw metadata route is not enabled for this primary capture')
            require(end.get('raw_response_usage_start_sha256') == capture['start']['sha256'], 'start digest differs')
            for name, filename in [('settings', 'raw-response-usage.json'), ('journal', 'raw-response-rewrites.jsonl')]:
                require(end.get('raw_response_usage_sha256', {}).get(filename) == extra[name]['sha256'], 'proof artifact terminal digest differs')
            raw = sources.read(capture['clients']); forwarded = sources.read(capture['forwarded'])
            require(end.get('stdin_sha256') == sha(raw)
                    and end.get('raw_response_usage_sha256', {}).get('client-to-server.forwarded.raw') == sha(forwarded), 'terminal raw digest differs')
            proofs[str(path)] = prove_frames(raw, forwarded, journal, settings)
        require(primitive_data.decode().count(OLD) == 1, 'original predicate seam differs')
        derived_source = primitive_data.decode().replace(OLD, NEW, 1)
        derived = types.ModuleType('derived_review_primitive'); derived.__file__ = str(PRIMITIVE)
        derived.__dict__['_bound_forwarding'] = lambda cap: cap.get('path') in proofs and matches_proof(proofs[cap['path']], cap['clients'], cap['forwarded'])
        exec(compile(derived_source, str(PRIMITIVE), 'exec'), derived.__dict__)
        collector = compile_exact(COLLECTOR, COLLECTOR_SHA)
        collector.primitives = lambda: derived
        result = collector.audit_sources(original, sources)
        result['schema'] = 'work-leaf-review-observer-frame-derived-v1'
        result['observer_frame_derivative'] = dict(helper_sha256=value['helper_sha256'], original_collector_sha256=COLLECTOR_SHA,
            original_primitive_sha256=PRIMITIVE_SHA, executed_derived_primitive_sha256=sha(derived_source.encode()), proofs=proofs,
            scope='Only exact journalled observer metadata fields qualify; original sources/frames/input joins/retrieval arithmetic unchanged. Original failed collector and diagnostic outcome remain retained.')
    except (ValueError, KeyError, TypeError, OSError, IndexError) as error:
        result['errors'].append(str(error))
    try: sources.finish()
    except (ValueError, OSError) as error: result['errors'].append(str(error))
    if result['errors']: result['status'] = 'unverifiable'
    result['source_sha256'] = sources.hashes
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', required=True); parser.add_argument('--input-sha256', required=True); parser.add_argument('--output', required=True)
    args = parser.parse_args(); collector, _ = original_modules(); sources = collector.Sources()
    output = Path(args.output)
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists() and not output.is_symlink(), 'output must be canonical and create-new')
    value = sources.json({'path': str(Path(args.input).absolute()), 'sha256': args.input_sha256})
    result = audit_sources(value, sources)
    with output.open('x') as stream: json.dump(result, stream, indent=2, sort_keys=True); stream.write('\n')
    print(json.dumps({'status': result['status'], 'output': str(output), 'provider_work_started': False}))
    return 0 if result['status'] == 'available' else 1


if __name__ == '__main__':
    raise SystemExit(main())
