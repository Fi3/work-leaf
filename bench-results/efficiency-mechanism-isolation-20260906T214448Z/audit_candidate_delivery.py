#!/usr/bin/env python3
"""Provider-free v4 delivered-prompt census, not a token or causal estimator.

The source-pinned v3 occurrence inventory is reused in a private module with v4
boundary validators. No frozen helper, primary endpoint or provider is invoked.
Policy literal correctness belongs to the attested Rust renderer; this audit
checks owned structure, factor selection, delivery and public item identities.
"""
from collections import Counter, defaultdict
from functools import lru_cache
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
import types

HERE = Path(__file__).resolve().parent
SCHEMA = 'work-leaf-bench-experiment-v4'
CONDITIONS = ('requested-repeat-full', 'unified-diff-preferred', 'review-fix-request-resupply')
PINS = {'analyze_untracked_reads.py': '30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e',
        'analyze_work_units.py': 'e59e6a22bf1ab09e0aabec86791cefb54cd3f3d38167a4c9822cdce1d09b50d3'}
RUNTIME_PINS = {
    'src/agent.rs': 'a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd',
    'src/bench_experiment.rs': '8c9db45eda3a10acabe441259e901617d44bf3d92f800231099ba84036de9a89',
    'src/bench_candidate_experiment.rs': 'e327d28fb05ceaa8f9c9465a78fc61069eddb56faa767b4909f29f83684d8b94',
    'src/orchestrator.rs': '027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b',
    'src/cli.rs': 'e68988f004ac9a61d25c0565b41dd46a371ebcf62a2415c0aabc4d6d83a96763',
    'src/chat_title.rs': 'ba67ff00c43231791f42ffa964c94162e415b2e4b4ee4c71ac1c29ac3d81057f'}
POLICY_FACTORS = {name: CONDITIONS[1] for name in ('policy-write-format', 'policy-edit-request',
    'policy-edit-preference', 'policy-manual-write-format', 'instruction-commit-format')}
POLICY_FACTORS.update({name: CONDITIONS[0] for name in ('policy-requested-repeat-contract', 'policy-repeat-authority')})
NORMAL = 'You are running under the work-leaf orchestrator.\n'
LINEARIZER = 'You are running as the work-leaf linearize agent.\n'
REPEAT = '\nRepeated file reads: current full text\n'
RESUPPLY = '\n\nOriginal feature request (unchanged from launch):\n'
TITLE = ("Name this Work Leaf chat from the user's first prompt.\nRules:\n"
         '- Return only the chat name, no prose and no quotes.\n- Derive the name from the first prompt only.\n'
         '- Use at most 40 characters.\n- Use lowercase words separated by hyphens.\n- Do not use spaces.\n\nFirst prompt:\n')


def load_pinned(name):
    path = HERE/name; data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != PINS[name]:
        raise ValueError('predecessor source differs: '+name)
    module = types.ModuleType('candidate_delivery_'+path.stem)
    module.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


CORE = load_pinned('analyze_work_units.py')
BASE = load_pinned('analyze_untracked_reads.py')
require, identity, integer = BASE.require, BASE.identity, BASE.integer
digest, fnv, byte_range, rpc = BASE.digest, BASE.fnv, BASE.byte_range, BASE.rpc


def site_of(text):
    if not isinstance(text, str): return None
    if text.startswith((NORMAL, LINEARIZER)): return 'candidate-policy'
    if text.startswith(BASE.HEADER): return 'requested-repeat-read'
    if text.startswith('The reviewer found issues in your patch for commit '): return 'review-fix-request'
    return CORE.site_of(text)


def owned_components(event):
    old = event['original_prompt'].encode(); new = event['candidate_prompt'].encode()
    spans = event['components']; require(isinstance(spans, list), 'components must be a list')
    previous_old = previous_new = 0
    for span in spans:
        require(isinstance(span, dict) and set(span) == {'baseline_start', 'baseline_end', 'inline_start', 'inline_end'}, 'unknown component shape')
        a, b, c, d = (span[key] for key in ('baseline_start', 'baseline_end', 'inline_start', 'inline_end'))
        byte_range(old, a, b); byte_range(new, c, d)
        require(a >= previous_old and c >= previous_new and old[previous_old:a] == new[previous_new:c], 'unordered or changed unowned bytes')
        previous_old, previous_new = b, d
    require(old[previous_old:] == new[previous_new:], 'changed unowned suffix')
    return old, new, spans


def validate_policy(event, old, new, components):
    agent = event['agent_id']; linearizer = agent == 'linearize' or agent.startswith('linearize-')
    require(old.startswith((LINEARIZER if linearizer else NORMAL).encode()), 'policy role/header differs')
    ends = [offset for marker in (b'\n\n--- ', b'\n\nAgent-ID: ') if (offset := old.find(marker)) >= 0]
    require(ends, 'missing policy owned-section boundary')
    limit = min(ends); rows = event['metadata']['spans']
    require(isinstance(rows, list) and len(rows) == len(components), 'policy span inventory differs')
    counts = Counter()
    for row, span in zip(rows, components):
        require(isinstance(row, dict) and set(row) == {'id', 'condition', 'original', 'replacement'}, 'unknown policy metadata')
        name = row['id']; require(name in POLICY_FACTORS and row['condition'] == POLICY_FACTORS[name], 'unknown policy factor')
        require(span['baseline_end'] <= limit, 'policy span enters copied instructions/user text')
        require(old[span['baseline_start']:span['baseline_end']] == row['original'].encode(), 'policy original span differs')
        expected = row['replacement'] if row['condition'] == event['condition'] else row['original']
        require(new[span['inline_start']:span['inline_end']] == expected.encode(), 'policy changed wrong factor')
        counts[name] += 1
    if linearizer:
        require(not rows and old == new, 'linearizer policy must be identity')
    else:
        for name in ('policy-write-format', 'policy-edit-request', 'policy-edit-preference', 'policy-manual-write-format'):
            require(counts[name] == 1, 'missing/duplicate ordinary policy span: '+name)
        restricted = b'You are not allowed to read files directly;' in old[:limit]
        require(counts['policy-requested-repeat-contract'] == int(restricted), 'repeat policy permission scope differs')
        instructions = b'\n\nRepository instructions from the launch project:' in old[:limit]
        require(counts['policy-repeat-authority'] == int(instructions), 'instruction authority scope differs')
        require(counts['instruction-commit-format'] == old[:limit].count(b'\n- Commit-message rules remain mandatory.'), 'commit translation inventory differs')


def inline_prefix(data, offset, snapshots):
    require(data[offset:offset+len(BASE.HEADER)] == BASE.HEADER.encode(), 'inline header differs')
    offset += len(BASE.HEADER)
    for row in snapshots:
        header = ('\n--- '+row['path']+' ---\n').encode()
        require(data[offset:offset+len(header)] == header, 'inline snapshot heading differs')
        offset += len(header); body = byte_range(data, offset, offset+row['bytes'])
        require(fnv(body) == row['digest'], 'inline nonfactor snapshot digest differs')
        offset += len(body)
        if not body.endswith(b'\n'):
            require(data[offset:offset+1] == b'\n', 'inline terminal newline differs'); offset += 1
    return offset


def validate_repeat(event, old, new, components):
    meta = event['metadata']
    require(set(meta) == {'eligible', 'requested_paths', 'bundle', 'snapshots', 'failures'}, 'unknown read metadata')
    snapshots = meta['snapshots']; failures = meta['failures']; paths = meta['requested_paths']
    require(isinstance(snapshots, list) and isinstance(failures, list) and isinstance(paths, list) and all(identity(p) for p in paths), 'invalid read inventory')
    buckets = {key: [] for key in ('untracked', 'changed', 'unchanged', 'explicit-bundle')}
    last = -1; seen = set(); bodies = []
    for row in snapshots:
        require(isinstance(row, dict) and set(row) == {'path', 'class', 'bytes', 'digest', 'inline_body_start', 'inline_body_end'}, 'unknown snapshot metadata')
        kind = row['class']; require(kind in buckets and identity(row['path']) and integer(row['bytes']), 'invalid snapshot class/path/bytes')
        index = list(buckets).index(kind); require(index >= last, 'snapshot class order differs'); last = index
        require((kind == 'explicit-bundle' or row['path'] not in seen), 'duplicate project snapshot'); seen.add(row['path'])
        require(isinstance(row['digest'], str) and re.fullmatch(r'fnv64:[0-9a-f]{16}; bytes:'+str(row['bytes']), row['digest']), 'invalid snapshot digest')
        buckets[kind].append(row)
        if kind in ('changed', 'unchanged'):
            body = byte_range(new, row['inline_body_start'], row['inline_body_end'])
            require(len(body) == row['bytes'] and fnv(body) == row['digest'], 'tracked body length/digest differs')
            bodies.append((row, body))
        else:
            require(row['inline_body_start'] is None and row['inline_body_end'] is None, 'nonfactor snapshot acquired body range')
    for bucket in buckets.values():
        require([r['path'] for r in bucket] == sorted((r['path'] for r in bucket), key=lambda p: PurePosixPath(p).parts), 'snapshot path order differs')
    for failure in failures:
        require(isinstance(failure, dict) and set(failure) == {'path', 'diagnostic'} and identity(failure['path']) and isinstance(failure['diagnostic'], str), 'unknown failure metadata')
    eligible = bool(bodies)
    require(type(meta['eligible']) is bool and meta['eligible'] == eligible and len(components) == int(eligible), 'repeat eligibility differs')
    bundle = meta['bundle']; require(isinstance(bundle, dict) and set(bundle) == {'threshold_eligible', 'write_succeeded', 'path'}, 'unknown bundle metadata')
    untracked = buckets['untracked']; explicit = buckets['explicit-bundle']
    threshold = sum(r['bytes'] for r in untracked) > 24576 or any(r['bytes'] > 16384 for r in untracked)
    require(type(bundle['threshold_eligible']) is bool and bundle['threshold_eligible'] == threshold and type(bundle['write_succeeded']) is bool, 'bundle threshold differs')
    success = bundle['write_succeeded']; require(not success or threshold, 'impossible bundle success')
    require(identity(bundle['path']) if success else bundle['path'] is None, 'bundle path/success differs')
    offset = inline_prefix(old, 0, explicit) if explicit else 0
    if explicit and not untracked and not bodies and not failures:
        require(offset == len(old) and old == new, 'pure bundle read differs'); return []
    if explicit:
        require(old[offset:offset+1] == b'\n', 'explicit/project separator differs'); offset += 1
    if success:
        prefix = BASE.HEADER+BASE.BUNDLE_INTRO+bundle['path']+BASE.BUNDLE_PERMISSION
        prefix += ''.join('- '+r['path']+' ('+r['digest']+')\n' for r in untracked)
        require(old[offset:offset+len(prefix.encode())] == prefix.encode(), 'untracked bundle component differs')
        offset += len(prefix.encode())
    else:
        offset = inline_prefix(old, offset, untracked)
    if eligible:
        span = components[0]
        require(span['baseline_start'] == offset == span['inline_start'], 'repeat component is not after owned first-read prefix')
        require(old[offset:span['baseline_end']].startswith(b'\nRepeated file reads '+(b'with changes\n' if buckets['changed'] else b'unchanged\n')), 'repeat baseline heading differs')
        cursor = offset+len(REPEAT.encode()); pieces = [REPEAT.encode()]
        for row, body in bodies:
            header = ('\n--- '+row['path']+' ---\n').encode()
            require(row['inline_body_start'] == cursor+len(header), 'tracked body range not renderer-owned')
            tail = b'' if body.endswith(b'\n') else b'\n'
            pieces.extend((header, body, tail)); cursor += len(header)+len(body)+len(tail)
        require(span['inline_end'] == cursor and new[offset:cursor] == b''.join(pieces), 'repeat candidate format differs')
        offset = span['baseline_end']
    suffix = '' if not failures else '\nUnavailable file text\n'+''.join('- '+r['path']+': '+r['diagnostic']+'\n' for r in failures)
    require(old[offset:] == suffix.encode(), 'failure suffix is outside repeat component')
    return [{'path': row['path'], 'class': row['class'], 'bytes': row['bytes'], 'sha256': digest(body),
             'inline_body_start': row['inline_body_start'], 'inline_body_end': row['inline_body_end']} for row, body in bodies]


def validate_event(event, condition):
    try:
        require(event.get('schema') == SCHEMA and event.get('condition') == condition and condition in CONDITIONS, 'wrong schema/condition')
        site = event.get('site'); require(site_of(event.get('original_prompt')) == site, 'not an owned renderer boundary')
        if event.get('event') == 'prompt':
            require(site in ('patch-applied', 'command-result'), 'unsupported identity event')
            require(not CORE.validate_transform(event, 'control'), 'nonfactor ACK/command transformation differs')
            return {'errors': [], 'eligible': False, 'selected_prompt': event['forwarded_prompt'], 'selected_sha256': digest(event['forwarded_prompt'].encode())}
        require(event.get('event') == 'candidate-prompt' and isinstance(event.get('metadata'), dict), 'unknown candidate event')
        old, new, spans = owned_components(event)
        selected_candidate = site == 'candidate-policy' or condition == {'requested-repeat-read': CONDITIONS[0], 'review-fix-request': CONDITIONS[2]}.get(site)
        selected = new if selected_candidate else old
        require(event['selected_candidate'] == ('candidate' if selected_candidate else 'baseline'), 'wrong candidate selected')
        for key, wanted in (('original_bytes', len(old)), ('candidate_bytes', len(new)), ('selected_bytes', len(selected)), ('byte_delta', len(selected)-len(old))):
            require(type(event.get(key)) is int and event[key] == wanted, 'invalid prompt byte arithmetic')
        require(type(event.get('changed')) is bool and event['changed'] == (selected != old), 'invalid changed flag')
        result = {'errors': [], 'eligible': False, 'changed': event['changed'], 'byte_delta': event['byte_delta'],
                  'selected_prompt': selected.decode(), 'selected_sha256': digest(selected), 'original_sha256': digest(old)}
        if site == 'candidate-policy': validate_policy(event, old, new, spans)
        elif site == 'requested-repeat-read':
            result['snapshot_evidence'] = validate_repeat(event, old, new, spans)
            result['eligible'] = event['metadata']['eligible']
        elif site == 'review-fix-request':
            meta = event['metadata']
            require(meta['original_request_source'] == 'prepared-agent-launch.prompt' and meta['source_agent_id'] == event['agent_id'] and isinstance(meta['source_feature'], str), 'invalid original-request provenance')
            require(len(spans) == 1 and spans[0] == {'baseline_start': len(old), 'baseline_end': len(old), 'inline_start': len(old), 'inline_end': len(new)}, 'request must be one terminal insertion')
            request = byte_range(new, meta['candidate_request_start'], meta['candidate_request_end'])
            require(integer(meta['original_request_bytes']) and len(request) == meta['original_request_bytes'] and new == old+RESUPPLY.encode()+request and meta['candidate_request_start'] == len(old)+len(RESUPPLY.encode()) and meta['candidate_request_end'] == len(new), 'original request framing/span differs')
            result.update(eligible=True, original_request_sha256=digest(request))
        else: raise ValueError('unsupported candidate site')
        return result
    except (ValueError, KeyError, TypeError, AttributeError, IndexError) as error:
        return {'errors': [str(error)]}


@lru_cache(maxsize=3)
def inventory_module(condition):
    require(condition in CONDITIONS, 'unsupported v4 condition')
    module = load_pinned('analyze_untracked_reads.py')
    module.SCHEMA = SCHEMA; module.VARIANT = condition
    module.site_of = site_of
    module.validate_read = module.validate_nonread = lambda event: validate_event(event, condition)
    return module


def public_items(captures):
    users = {}; actions = []; errors = []; seen = set()
    for capture in captures:
        physical = capture.get('server_lines', list(range(1, len(capture['servers'])+1)))
        for line, frame in zip(physical, capture['servers']):
            try:
                require(isinstance(frame, dict), 'unknown public server frame')
                if frame.get('method') != 'item/completed': continue
                params = frame['params']; item = params['item']
                if item.get('type') not in ('userMessage', 'agentMessage'): continue
                thread, turn, item_id = params.get('threadId'), params.get('turnId'), item.get('id')
                require(all(identity(v) for v in (thread, turn, item_id)), 'unknown public item identity')
                require((thread, item_id) not in seen, 'duplicate public item identity'); seen.add((thread, item_id))
                locator = {'capture': capture['path'], 'server_line': line, 'thread_id': thread, 'turn_id': turn, 'item_id': item_id}
                if item['type'] == 'userMessage':
                    content = item.get('content')
                    require(isinstance(content, list) and len(content) == 1 and isinstance(content[0], dict) and content[0].get('type') == 'text' and isinstance(content[0].get('text'), str), 'unknown public user input')
                    require((thread, turn) not in users, 'duplicate public user turn identity')
                    users[thread, turn] = (content[0]['text'], locator)
                else:
                    require(isinstance(item.get('text'), str), 'unknown public agent text')
                    # This is a public directive-format witness, not an accepted patch/commit join.
                    active = None; message_hash = digest(item['text'].encode())
                    for number, text in enumerate(item['text'].splitlines(), 1):
                        body = text.lstrip()
                        if active is not None:
                            if body == '@work-leaf end': active = None
                            continue
                        match = re.match(r'^@work-leaf\s+(edit|patch)\s+\S', body)
                        if match:
                            active = match[1]
                            actions.append({**locator, 'message_line': number, 'format': active,
                                            'public_message_sha256': message_hash})
            except (ValueError, KeyError, TypeError, AttributeError) as error:
                errors.append(f"public {capture['path']}:{line}: {error}")
    return users, actions, errors


def audit_delivery(trace, captures, run_id, condition, trace_lines=None):
    errors = []; threads = []; owner_by_text = defaultdict(set); policy_bytes = {}; policies = {}
    request_order = {}; auxiliary_titles = 0
    native_users = {}; actions = []; inventory = {'exposures': [], 'errors': []}
    try:
        require(condition in CONDITIONS and identity(run_id), 'invalid expected run/condition')
        require(isinstance(trace, list) and trace and all(isinstance(e, dict) for e in trace), 'unknown trace inventory')
        require(isinstance(captures, list), 'unknown capture inventory')
        for index, event in enumerate(trace[1:], 1):
            if event.get('site') != 'candidate-policy': continue
            checked = validate_event(event, condition); require(not checked['errors'], '; '.join(checked['errors']))
            text = checked['selected_prompt']; key = digest(text.encode())
            owner_by_text[key].add(event['agent_id']); policy_bytes[key] = text
        owner = {}
        for capture in captures:
            require(isinstance(capture, dict) and all(isinstance(capture.get(k), list) for k in ('clients', 'forwarded', 'servers')), 'unknown capture shape')
            forwarded = {}
            for frame in capture['forwarded']:
                require(isinstance(frame, dict), 'unknown forwarded frame')
                if frame.get('method') != 'turn/start': continue
                key = rpc(frame); require(key not in forwarded, 'duplicate forwarded request'); forwarded[key] = frame
            original_keys = set()
            for ordinal, frame in enumerate(capture['clients']):
                require(isinstance(frame, dict), 'unknown original frame')
                if frame.get('method') != 'turn/start': continue
                key = rpc(frame); require(key not in original_keys, 'duplicate original request'); original_keys.add(key)
                require(forwarded.pop(key, None) == frame, 'original/forwarded request differs')
                request_order[capture['path'], *key] = ordinal
                params = frame['params']; inputs = params['input']; thread = params.get('threadId')
                require(identity(thread) and isinstance(inputs, list) and len(inputs) == 1 and isinstance(inputs[0], dict) and isinstance(inputs[0].get('text'), str), 'unknown request text/thread')
                if thread not in owner:
                    text = inputs[0]['text']; owners = owner_by_text[digest(text.encode())]
                    if text.startswith(TITLE):
                        require(not owners, 'title/policy ownership conflict'); owner[thread] = 'title-agent'
                    else:
                        require(len(owners) == 1 and policy_bytes.get(digest(text.encode())) == text, 'first thread input lacks unambiguous owned policy identity')
                        owner[thread] = next(iter(owners))
                    threads.append({'thread_id': thread, 'agent_id': owner[thread]})
                if owner[thread] == 'title-agent':
                    require(inputs[0]['text'].startswith(TITLE), 'auxiliary title thread contains unsupported request')
                    auxiliary_titles += 1
            require(not forwarded, 'extra forwarded request')
        inventory = inventory_module(condition).prompt_inventory(trace, threads, captures, run_id, condition, trace_lines)
        errors.extend(inventory['errors'])
        native_users, actions, public_errors = public_items(captures); errors.extend(public_errors)
        recognized_turns = set()
        for capture in captures:
            replies = {rpc(frame): frame for frame in capture['servers'] if isinstance(frame, dict) and 'method' not in frame and 'id' in frame}
            for frame in capture['clients']:
                if frame.get('method') != 'turn/start': continue
                reply = replies.get(rpc(frame), {}); turn = reply.get('result', {}).get('turn', {}).get('id')
                if identity(turn) and 'error' not in reply:
                    turn_key = frame['params']['threadId'], turn
                    require(turn_key not in recognized_turns, 'duplicate accepted turn identity')
                    recognized_turns.add(turn_key)
                    require(turn_key in native_users and native_users[turn_key][0] == frame['params']['input'][0]['text'], 'accepted input/public user bytes differ')
        require(set(native_users) == recognized_turns, 'accepted turns and public user items lack bidirectional coverage')
        by_line = dict(zip(trace_lines or range(1, len(trace)+1), trace))
        last = {}
        for row in inventory['exposures']:
            if row.get('rpc_id') is not None:
                scope = row['capture'], row['thread_id']; position = request_order[row['capture'], *rpc({'id': row['rpc_id']})]
                require(position > last.get(scope, -1), 'trace followups reorder captured requests')
                last[scope] = position
            if row.get('status') != 'delivered': continue
            event = by_line[row['trace_line']]; selected = validate_event(event, condition)['selected_prompt']
            key = row['thread_id'], row['turn_id']; user_text, witness = native_users.pop(key)
            require(user_text == selected, 'public user body differs from selected prompt')
            row.update(native_user_item_id=witness['item_id'], native_user_line=witness['server_line'])
            if row['site'] == 'candidate-policy': policies[event['agent_id']] = event
            if row['site'] == 'review-fix-request':
                meta = event['metadata']; request = byte_range(event['candidate_prompt'].encode(), meta['candidate_request_start'], meta['candidate_request_end'])
                source = policies.get(event['agent_id'])
                footer = f"\n\nAgent-ID: {event['agent_id']}\nFeature: {meta['source_feature']}\n\nUser prompt:\n".encode()+request
                require(source is not None and source['original_prompt'].encode().endswith(footer), 'resupplied request differs from latest owned launch footer')
                row['original_request_policy_sequence'] = source['sequence']
        for action in actions:
            require((action['thread_id'], action['turn_id']) in recognized_turns, 'public format item has no accepted turn')
    except (ValueError, KeyError, TypeError, AttributeError, IndexError) as error:
        errors.append(str(error))
    # Invalid input remains an explicit physical trace row, never a zero exposure assertion.
    physical = trace_lines or list(range(1, len(trace)+1)) if isinstance(trace, list) else []
    indexed = {row['trace_line']: row for row in inventory['exposures']}
    exposures = [indexed.get(line, {'trace_line': line, 'site': event.get('site') if isinstance(event, dict) else None,
                    'status': 'unverified', 'errors': ['trace could not be linked']})
                 for line, event in list(zip(physical, trace))[1:]] if isinstance(trace, list) else []
    delivered = [r for r in exposures if r.get('status') == 'delivered' and r.get('native_user_item_id')]
    counts = {'candidate_policy': sum(r['site'] == 'candidate-policy' for r in delivered),
              'requested_reads': sum(r['site'] == 'requested-repeat-read' for r in delivered),
              'eligible_repeat_reads': sum(r['site'] == 'requested-repeat-read' and r.get('eligible') is True for r in delivered),
              'original_request_resupplies': sum(r['site'] == 'review-fix-request' and r.get('changed') is True for r in delivered),
              'successful_acks': sum(r['site'] == 'patch-applied' for r in delivered)}
    return {'status': 'available' if not errors else 'unverifiable', 'errors': errors, 'run_id': run_id, 'condition': condition,
            'exposures': exposures, 'delivered': counts,
            'public_formats': {kind: sum(a['format'] == kind for a in actions) for kind in ('edit', 'patch')},
            'public_format_items': actions,
            'auxiliary_title_turns': auxiliary_titles,
            'scope': 'Source-attested prompt/public-item delivery only, not a native rollout join. Original-request footer/registry semantics rely on attested renderer source, not parsing copied metadata. Format witnesses are not accepted patch groups; no response inventory, usage, percentages or causal inference.'}


class Sources:
    """Expected bytes before reads; endpoint hashes after replay. No body export."""
    def __init__(self):
        self.hashes = {}; self.total = 0

    def read(self, ref):
        require(isinstance(ref, dict) and set(ref) == {'path', 'sha256'} and isinstance(ref['path'], str)
                and Path(ref['path']).is_absolute() and isinstance(ref['sha256'], str)
                and re.fullmatch('[0-9a-f]{64}', ref['sha256']), 'invalid expected source identity')
        path = BASE.regular(ref['path']); size = path.stat().st_size
        require(size <= 512*1024*1024 and self.total+size <= 2*1024*1024*1024, 'source byte budget exceeded; no clipping')
        with path.open('rb') as stream: data = stream.read(size+1)
        require(len(data) == size and digest(data) == ref['sha256'], 'source bytes differ: '+str(path))
        require(str(path) not in self.hashes or self.hashes[str(path)] == ref['sha256'], 'conflicting source identity')
        self.hashes[str(path)] = ref['sha256']; self.total += size
        return data

    def json(self, ref): return BASE.decode(self.read(ref).decode())

    def records(self, ref):
        data = self.read(ref); require(not data or data.endswith(b'\n'), 'unterminated physical JSONL source')
        rows = [(number, BASE.decode(line)) for number, line in enumerate(data.decode().splitlines(), 1) if line.strip()]
        return [row for _, row in rows], [line for line, _ in rows]

    def finish(self):
        for path, expected in self.hashes.items(): require(BASE.sha(path) == expected, 'source changed during audit: '+path)


def terminal_projection(value, run_id, condition):
    require(isinstance(value, dict) and value.get('run_id') == run_id and value.get('condition') == condition, 'terminal run identity differs')
    if value.get('schema') == 'work-leaf-candidate-diagnostic-terminal-v4':
        start, end, code = value.get('launch_clock'), value.get('receipt_clock'), value.get('exit_code')
        before = datetime.strptime(start, '%Y-%m-%d %H:%M:%S UTC'); after = datetime.strptime(end, '%Y-%m-%d %H:%M:%S UTC')
        kind = 'retained-diagnostic-attempt'
    else:
        require(value.get('id') == run_id and value.get('launch_status') == 'completed', 'workflow is not terminally published')
        start, end, code = value.get('started_at'), value.get('finished_at'), value.get('launcher_exit_code')
        before = datetime.fromisoformat(start); after = datetime.fromisoformat(end); kind = 'terminal-workflow'
        require(before.tzinfo is not None and after.tzinfo is not None, 'terminal timestamps lack timezone')
    require(after >= before and type(code) is int, 'unknown terminal timing/exit')
    return {'kind': kind, 'started_at': start, 'finished_at': end, 'exit_code': code}


def audit_sources(manifest, sources=None):
    """One source-bound closed workflow/diagnostic; never calls a phase analyzer."""
    sources = sources or Sources()
    require(isinstance(manifest, dict) and manifest.get('schema') == 'work-leaf-candidate-delivery-input-v1', 'unsupported delivery input')
    run_id, condition = manifest.get('run_id'), manifest.get('condition')
    require(identity(run_id) and condition in CONDITIONS, 'unsupported run/condition')
    sources.read({'path': str(Path(__file__).resolve()), 'sha256': manifest['helper_sha256']})
    for name, expected in PINS.items(): sources.read({'path': str(HERE/name), 'sha256': expected})
    runtime = manifest['runtime_sources']; require(isinstance(runtime, dict) and set(runtime) == set(RUNTIME_PINS), 'runtime source inventory differs')
    for name, expected in RUNTIME_PINS.items():
        require(runtime[name].get('sha256') == expected, 'unsupported runtime source bytes: '+name); sources.read(runtime[name])
    terminal = terminal_projection(sources.json(manifest['terminal']), run_id, condition)
    trace, trace_lines = sources.records(manifest['trace'])
    invocations, _ = sources.records(manifest['invocations']); expected_invocations = {}
    for row in invocations:
        require(isinstance(row, dict) and identity(row.get('invocation_id')), 'invalid invocation inventory')
        if row.get('capture_kind') != 'app-server': continue
        key = row['invocation_id']; require(key not in expected_invocations, 'duplicate app-server invocation'); expected_invocations[key] = row
    captures = []; seen = set(); observation = Path(manifest['invocations']['path']).parent
    require(Path(manifest['invocations']['path']).name == 'process-invocations.jsonl', 'unknown observer inventory path')
    for cap in manifest['captures']:
        path = Path(cap['path']); key = path.name
        require(path == observation/'app-server'/key and key in expected_invocations and key not in seen, 'capture inventory/path differs'); seen.add(key)
        required = {'clients': path/'client-to-server.raw', 'forwarded': path/'client-to-server.forwarded.raw',
                    'servers': path/'server-to-client.raw', 'start': observation/'invocations'/key/'start.json',
                    'end': observation/'invocations'/key/'end.json'}
        for name, target in required.items(): require(cap[name]['path'] == str(target), 'capture source is not its canonical observer artifact')
        start, end = sources.json(cap['start']), sources.json(cap['end']); logged = expected_invocations[key]
        require(start.get('invocation_id') == end.get('invocation_id') == key and start.get('capture_kind') == 'app-server'
                and start.get('primary') is True and integer(start.get('start_unix_ns')) and integer(end.get('end_unix_ns'))
                and end['end_unix_ns'] >= start['start_unix_ns'] and (type(end.get('exit_code')) is int or type(end.get('terminating_signal')) is int), 'capture is not a closed primary invocation')
        for name in ('invocation_id', 'capture_kind', 'primary', 'start_unix_ns'): require(logged.get(name) == start.get(name), 'observer start inventory differs')
        for name in ('invocation_id', 'end_unix_ns', 'exit_code', 'terminating_signal', 'stdin_sha256', 'stdout_sha256'):
            require(isinstance(logged.get('end'), dict) and logged['end'].get(name) == end.get(name), 'observer terminal inventory differs')
        require(end.get('raw_response_usage_start_sha256') == cap['start']['sha256'] and end.get('stdin_sha256') == cap['clients']['sha256']
                and end.get('stdout_sha256') == cap['servers']['sha256'] and end.get('raw_response_usage_sha256', {}).get('client-to-server.forwarded.raw') == cap['forwarded']['sha256'], 'capture terminal digests differ')
        clients, client_lines = sources.records(cap['clients']); servers, server_lines = sources.records(cap['servers']); forwarded, _ = sources.records(cap['forwarded'])
        captures.append({'path': str(path), 'clients': clients, 'servers': servers, 'forwarded': forwarded, 'client_lines': client_lines, 'server_lines': server_lines})
    require(seen == set(expected_invocations) and seen, 'missing app-server capture')
    result = audit_delivery(trace, captures, run_id, condition, trace_lines)
    sources.finish()
    result.update(terminal=terminal, source_sha256=sources.hashes, provenance_scope='Closed observer invocation census and exact stream digests; source-pinned runtime semantics. Native rollout/accounting/config-trust and binary admission remain separate required audits.')
    return result


def write_new(path, result):
    with Path(path).open('x', encoding='utf-8') as stream: json.dump(result, stream, indent=2, sort_keys=True); stream.write('\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--input-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        require(not args.output.exists(), 'derived output already exists')
        sources = Sources(); manifest = sources.json({'path': str(args.input.absolute()), 'sha256': args.input_sha256})
        result = audit_sources(manifest, sources); write_new(args.output, result)
        print(json.dumps({'status': result['status'], 'output': str(args.output), 'provider_work_started': False}))
        return 0 if result['status'] == 'available' else 1
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print('candidate delivery: '+str(error), file=sys.stderr); return 2


if __name__ == '__main__': raise SystemExit(main())
