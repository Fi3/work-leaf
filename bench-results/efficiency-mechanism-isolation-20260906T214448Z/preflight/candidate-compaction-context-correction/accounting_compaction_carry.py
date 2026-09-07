#!/usr/bin/env python3
"""Offline, separately scoped compaction-metadata carry-forward attestation.

An exact originating omission remains the authority; matching later metadata is
only a warning. Physical frames, raw/native usage, cumulative arithmetic and the
strict tail accountant remain intact. No provider/configuration operation exists.
The added state uses one forward pass and one entry per thread: O(E + T), with
bounded-field serialization per metadata event; no scan of earlier event pairs.
"""
import hashlib
import json
from pathlib import Path
import textwrap
import types

HERE = Path(__file__).resolve().parent
CONTEXT_PATH = HERE / 'accounting_compaction_context.py'
CONTEXT_SHA = '513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3'
ORIGINAL_SHA = 'c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def load_context():
    data = CONTEXT_PATH.read_bytes()
    if CONTEXT_PATH.resolve() != CONTEXT_PATH or CONTEXT_PATH.is_symlink() or sha(data) != CONTEXT_SHA:
        raise ValueError('context dependency identity differs')
    module = types.ModuleType('compaction_carry_pinned_context')
    module.__file__ = str(CONTEXT_PATH)
    exec(compile(data, str(CONTEXT_PATH), 'exec'), module.__dict__)
    return module


class _CarryState:
    """Capture-local ownership; only the original proved branch creates origins."""
    def __init__(self):
        self.origins = {}

    def observe(self, method, params, seen):
        thread = params.get('threadId')
        lifecycle = isinstance(method, str) and method.partition('/')[0] in {'thread', 'turn'}
        supported = {'thread/tokenUsage/updated', 'thread/status/changed', 'turn/started', 'turn/completed'}
        if lifecycle and method not in supported:
            nested = params.get('thread')
            nested_id = nested.get('id') if isinstance(nested, dict) else None
            if thread is None:
                thread = nested_id
            if not isinstance(thread, str) or (nested_id is not None and nested_id != thread):
                self.origins.clear()
            else:
                self.origins.pop(thread, None)
            return
        if thread not in self.origins:
            return
        item = params.get('item') or {}
        if ((method == 'rawResponse/completed' and params.get('responseId') not in seen)
                or (method == 'item/started' and item.get('type') == 'contextCompaction')):
            self.origins.pop(thread)
        elif method == 'thread/tokenUsage/updated':
            data = params.get('tokenUsage') or {}
            origin = self.origins[thread]
            if (canonical(data.get('last')) != origin['last']
                    or canonical(data.get('total')) != origin['total']):
                self.origins.pop(thread)

    def establish(self, thread, turn, sequence, data, response_id, window):
        self.origins[thread] = dict(thread_id=thread, turn_id=turn,
            origin_server_line=sequence+1, response_id=response_id, window=window,
            last=canonical(data['last']), total=canonical(data['total']))

    def match(self, thread, sequence, data, total, prior, expected):
        origin = self.origins.get(thread)
        if (origin is None or sequence <= origin['window'][1]
                or total != prior or total != expected
                or canonical(data['last']) != origin['last']
                or canonical(data['total']) != origin['total']):
            return None
        return origin

    @staticmethod
    def warning(origin, thread, turn, sequence, data, total):
        begin, end, key = origin['window']
        return dict(classification='unusable_compaction_last_metadata_carry_forward',
            server_line=sequence+1, thread_id=thread, turn_id=turn,
            origin_server_line=origin['origin_server_line'], origin_turn_id=origin['turn_id'],
            response_id=origin['response_id'], item_id=key[2],
            lifecycle_start_line=begin+1, lifecycle_end_line=end+1,
            original_last=data['last'], unchanged_cumulative=total,
            exact_last_sha256=sha(origin['last']), exact_cumulative_sha256=sha(origin['total']),
            establishes_completed_response_usage=False, establishes_context_size_estimate=False,
            used_for_fresh_usage_or_tail_recovery=False,
            basis='same capture/thread originating compaction; exact typed values and unchanged raw-prefix expectation; no intervening new same-thread response or compaction')


def _replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError('frozen reconciliation derivation anchor differs')
    return source.replace(before, after, 1)


def derive_reconcile(original):
    """Derive only warning provenance; the original result remains available."""
    path = Path(original.__file__).resolve()
    data = path.read_bytes()
    if sha(data) != ORIGINAL_SHA:
        raise ValueError('original reconciliation source identity differs')
    text = data.decode()
    begin = text.index('def reconcile_stream(')
    end = text.index('\ndef audit_run(', begin)
    source = text[begin:end]
    anchor = ('        fresh = {}; latest_raw = {}\n'
              '        for sequence, value in enumerate(servers):\n'
              '            method = value.get("method"); p = value.get("params") or {}\n')
    source = _replace_once(source, anchor,
        anchor.replace('latest_raw = {}', 'latest_raw = {}; carry = _CarryState()')
        + '            carry.observe(method, p, seen)\n')
    block_begin = source.index('                if nonadditive:\n')
    block_end = source.index('                else:\n', block_begin)
    old_block = source[block_begin+len('                if nonadditive:\n'):block_end]
    replacement = (
        '                if nonadditive:\n'
        '                    carried = carry.match(thread, sequence, data, total, prior, expected) if newly_omitted is None else None\n'
        '                    if carried is None:\n'
        + textwrap.indent(old_block, '    ')
        + '                        carry.establish(thread, turn, sequence, data, newly_omitted, windows[newly_omitted])\n'
        '                    else:\n'
        '                        warning_sequences.add(sequence); affected_turns.add((thread, turn))\n'
        '                        result["warnings"].append(carry.warning(carried, thread, turn, sequence, data, total))\n')
    source = source[:block_begin] + replacement + source[block_end:]
    namespace = {**original.__dict__, '_CarryState': _CarryState}
    exec(compile(source, str(path), 'exec'), namespace)
    result = namespace['reconcile_stream']
    result._executed_derived_sha256 = sha(source.encode())
    result._original_source_sha256 = ORIGINAL_SHA
    return result


def instrument_original(original):
    """Retain original pure diagnostics, without a second whole-workflow audit."""
    baseline = original.reconcile_stream
    derived = derive_reconcile(original)
    diagnostics = []

    def wrapped(clients, servers, grace, native):
        if wrapped._native_object is None:
            wrapped._native_object = native
            wrapped._native_ids = set(native['records'])
            wrapped._native_sha256 = sha(canonical(native))
        elif wrapped._native_object is not native:
            raise ValueError('native inventory ownership changed across capture reconciliation')
        old = baseline(clients, servers, grace, native)
        result = derived(clients, servers, grace, native)
        duplicates = wrapped._raw_ids.intersection(old['raw_responses'])
        wrapped._raw_ids.update(old['raw_responses'])
        wrapped._thread_ids.update(old['thread_ledger'])
        wrapped._duplicate_raw_ids.update(duplicates)
        diagnostics.append(dict(capture_index=len(diagnostics),
            client_rows_sha256=sha(canonical(clients)), server_rows_sha256=sha(canonical(servers)),
            grace_rows_sha256=sha(canonical(grace)), native_inventory_sha256=wrapped._native_sha256,
            original_inner_errors=list(old['errors']),
            original_thread_ledger_ids=sorted(old['thread_ledger']),
            original_raw_response_count=len(old['raw_responses']),
            original_warning_lines=[w['server_line'] for w in old['warnings']],
            derived_inner_errors=list(result['errors']),
            derived_thread_ledger_ids=sorted(result['thread_ledger']),
            derived_warning_lines=[w['server_line'] for w in result['warnings']],
            executed_derived_reconciliation_sha256=derived._executed_derived_sha256))
        return result

    wrapped._native_object = None
    wrapped._native_ids = set()
    wrapped._native_sha256 = None
    wrapped._raw_ids = set()
    wrapped._thread_ids = set()
    wrapped._duplicate_raw_ids = set()
    return wrapped, diagnostics


def _original_scope_consequence(wrapped, observed):
    """Only the frozen pre-cumulative outer scope guards, not an old total."""
    threads = [row['thread_id'] for row in observed['threads']]
    if len(set(threads)) != len(threads):
        raise ValueError('duplicate observer thread in source-bound outer diagnostic')
    error = None
    if wrapped._duplicate_raw_ids:
        error = 'response repeated across primary captures'
    elif wrapped._native_object is None or wrapped._raw_ids != wrapped._native_ids:
        error = 'raw/native response inventory coverage differs'
    elif wrapped._thread_ids != set(threads):
        error = 'raw cumulative thread scope differs from observer'
    return dict(status='complete_capture_census', original_outer_scope_error=error,
        observed_thread_ids=sorted(threads), original_reconciliation_thread_ids=sorted(wrapped._thread_ids),
        original_raw_native_response_coverage_equal=wrapped._raw_ids == wrapped._native_ids,
        original_duplicate_response_ids=sorted(wrapped._duplicate_raw_ids),
        second_original_audit_run_executed=False,
        basis='Only the exact frozen audit_run pre-cumulative scope guards; retained original reports remain authority for their complete original outcomes.')


def audit_run(entry, frozen, sessions_root):
    """Callable only under a separately declared numerical scope; no CLI admission."""
    context = load_context()
    self_path = Path(__file__).resolve()
    self_sha = sha(context.read_bytes(self_path))
    loader = context.load_original
    state = {}

    def loaded():
        original = loader()
        wrapped, diagnostics = instrument_original(original)
        original.reconcile_stream = wrapped
        state.update(wrapped=wrapped, diagnostics=diagnostics)
        return original

    context.load_original = loaded
    result = context.audit_run(entry, frozen, sessions_root)
    provenance = dict(helper_sha256=self_sha, context_helper_sha256=CONTEXT_SHA,
        original_helper_sha256=ORIGINAL_SHA, capture_diagnostics=state.get('diagnostics', []),
        original_outer_scope_diagnostic=None,
        scope='Exact originating-compaction metadata carry-forward warnings only; all original arithmetic, raw/native identities and unsupported-tail rules remain unchanged.')
    try:
        if provenance['capture_diagnostics']:
            observation = Path(entry['artifact']) / 'observation'
            apps = sorted(path for path in (observation / 'app-server').iterdir() if path.is_dir())
            if len(provenance['capture_diagnostics']) > len(apps):
                raise ValueError('capture diagnostic ownership differs')
            for row, path in zip(provenance['capture_diagnostics'], apps):
                for filename, key in [('client-to-server.raw', 'client_rows_sha256'),
                                      ('server-to-client.raw', 'server_rows_sha256'),
                                      ('provider-usage-grace.jsonl', 'grace_rows_sha256')]:
                    source = (path / filename).resolve()
                    if filename == 'provider-usage-grace.jsonl' and not source.exists():
                        source_rows = None
                    else:
                        content = context.read_bytes(source)
                        if result['source_sha256'].get(str(source)) != sha(content):
                            raise ValueError('capture source identity differs for reconciliation diagnostic')
                        if content and not content.endswith(b'\n'):
                            raise ValueError('capture source has incomplete closed JSONL')
                        source_rows = [json.loads(line) if line.strip() else {} for line in content.splitlines()]
                    if sha(canonical(source_rows)) != row[key]:
                        raise ValueError('capture source rows differ from executed reconciliation')
                row['capture'] = str(path)
            analysis_path = (observation / 'analysis.json').resolve()
            analysis = context.read_bytes(analysis_path)
            if result['source_sha256'].get(str(analysis_path)) != sha(analysis):
                raise ValueError('observer source identity differs for outer diagnostic')
            if len(provenance['capture_diagnostics']) == len(apps):
                provenance['original_outer_scope_diagnostic'] = _original_scope_consequence(state['wrapped'], json.loads(analysis))
            else:
                provenance['original_outer_scope_diagnostic'] = dict(
                    status='unavailable_partial_capture_census', original_outer_scope_error=None,
                    reconciled_capture_count=len(provenance['capture_diagnostics']), declared_capture_count=len(apps),
                    second_original_audit_run_executed=False,
                    basis='A retained failure stopped reconciliation before the complete capture census; no whole-scope consequence is claimed.')
        result['source_sha256'][str(self_path)] = self_sha
        result['source_sha256'][str(CONTEXT_PATH)] = CONTEXT_SHA
        for path, expected in result['source_sha256'].items():
            if sha(context.read_bytes(path)) != expected:
                raise ValueError('source changed before repeated-metadata closure: ' + path)
    except (ValueError, KeyError, TypeError, OSError) as error:
        result['errors'].append(str(error))
        result['status'] = 'unknown'
        result['measurement'] = dict(status='ineligible', bounds=None)
    result['repeated_metadata_derivative'] = provenance
    return result
