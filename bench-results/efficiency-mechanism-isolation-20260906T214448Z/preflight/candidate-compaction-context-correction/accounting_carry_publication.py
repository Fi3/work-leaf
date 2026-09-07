#!/usr/bin/env python3
"""Publication-only sibling of the immutable carry accountant.

The numerically active module owns its diagnostics even when context attestation
loads unused auxiliary modules. All reconciliation and strict-accounting code is
the exact predecessor code. No provider or configuration operation exists.
Selection scans L loaded module states once: O(L) time and references. Retaining
those wrappers holds O(L * D) compiled dependency state for dependency size D;
it introduces no pairwise event scan. Fresh numerical use needs its own scope.
"""
import hashlib
from pathlib import Path
import types

HERE = Path(__file__).resolve().parent
CARRY_PATH = HERE / 'accounting_compaction_carry.py'
CARRY_SHA = '47d28ef42c3243344cb28d6db680a531b04255cdc310653e45fc3ee81b8418a1'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load_carry():
    data = CARRY_PATH.read_bytes()
    if CARRY_PATH.resolve() != CARRY_PATH or CARRY_PATH.is_symlink() or sha(data) != CARRY_SHA:
        raise ValueError('carry publication dependency identity differs')
    module = types.ModuleType('publication_pinned_carry')
    module.__file__ = str(CARRY_PATH)
    exec(compile(data, str(CARRY_PATH), 'exec'), module.__dict__)
    return module, data


_CARRY, _CARRY_BYTES = load_carry()
load_context = _CARRY.load_context


def select_used_state(states):
    # instrument_original sets this exact object before attempting reconciliation.
    active = [state for state in states if state['wrapped']._native_object is not None]
    if len(active) > 1:
        raise ValueError('multiple accounting modules reconciled captures')
    return active[0] if active else {}


def derive_audit():
    """Only replace diagnostic-state ownership; not any accounting predicate."""
    text = _CARRY_BYTES.decode()
    source = text[text.index('\ndef audit_run(') + 1:]
    replacements = [
        ('    state = {}\n', '    states = []\n'),
        ('        state.update(wrapped=wrapped, diagnostics=diagnostics)\n',
         '        states.append(dict(wrapped=wrapped, diagnostics=diagnostics))\n'),
        ('    result = context.audit_run(entry, frozen, sessions_root)\n',
         '    result = context.audit_run(entry, frozen, sessions_root)\n'
         '    try:\n'
         '        state = _select_used_state(states)\n'
         '    except ValueError as error:\n'
         '        state = {}\n'
         '        result["errors"].append(str(error))\n'
         '        result["status"] = "unknown"\n'
         '        result["measurement"] = dict(status="ineligible", bounds=None)\n'),
    ]
    for before, after in replacements:
        source = _CARRY._replace_once(source, before, after)
    namespace = {**_CARRY.__dict__, 'load_context': load_context,
                 '_select_used_state': select_used_state}
    exec(compile(source, str(CARRY_PATH), 'exec'), namespace)
    return namespace['audit_run'], sha(source.encode())


def audit_run(entry, frozen, sessions_root):
    """No CLI; a separately declared future numerical scope is mandatory."""
    self_path = Path(__file__).resolve()
    self_bytes = self_path.read_bytes()
    if CARRY_PATH.read_bytes() != _CARRY_BYTES:
        raise ValueError('loaded carry dependency differs from current source')
    execute, derived_sha = derive_audit()
    result = execute(entry, frozen, sessions_root)
    result['diagnostic_publication_derivative'] = dict(
        helper_sha256=sha(self_bytes), predecessor_sha256=CARRY_SHA,
        executed_publication_wrapper_sha256=derived_sha,
        selection='Unique module whose instrumented reconciliation was actually invoked; no last-load inference.',
        numerical_predicates_changed=False, original_receipts_replaced=False)
    result['source_sha256'][str(self_path)] = sha(self_bytes)
    if self_path.read_bytes() != self_bytes or CARRY_PATH.read_bytes() != _CARRY_BYTES:
        result['errors'].append('publication source changed before closure')
        result['status'] = 'unknown'
        result['measurement'] = dict(status='ineligible', bounds=None)
    return result
