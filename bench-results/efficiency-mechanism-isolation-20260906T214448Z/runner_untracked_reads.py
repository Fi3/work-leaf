#!/usr/bin/env python3
"""V3 control adapter over the unchanged, hash-pinned three-workflow supervisor.

The manifest's runner_sha256 identifies that actual supervisor engine. Its frozen
files additionally bind this control adapter and the finite-allocation adapter.
Private module-instance parameters select only v3 conditions and exact fixed-N
validation; source files, providers, timing, environments and trust rules are unchanged.
Frozen-input checking retains the engine's O(W*E) cost (potentially quadratic as
per-run evidence grows); this protocol fixes W=4 waves and N=12 workflows.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import types

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
CONTROL_PATH = Path(__file__).resolve()
CONTROL_LOADED_SHA256 = hashlib.sha256(CONTROL_PATH.read_bytes()).hexdigest()
ENGINE_PATH = HERE / 'runner_work_units.py'
ENGINE_SHA256 = '2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e'
ALLOCATION_PATH = HERE / 'allocate_untracked_reads.py'
EXPERIMENT_SCHEMA = 'work-leaf-bench-experiment-v3'
CONDITIONS = ('control', 'untracked-read-inline')


def checked_instance(path, name, expected=None):
    data = path.read_bytes(); actual = hashlib.sha256(data).hexdigest()
    if expected is not None and actual != expected:
        raise ValueError('pinned control dependency changed: ' + str(path))
    instance = types.ModuleType(name); instance.__file__ = str(path)
    exec(compile(data, str(path), 'exec'), instance.__dict__)
    return instance, actual


_engine, _engine_loaded_sha = checked_instance(ENGINE_PATH, 'untracked_read_supervisor', ENGINE_SHA256)
_allocator, ALLOCATION_LOADED_SHA256 = checked_instance(ALLOCATION_PATH, 'untracked_read_allocation')
_base_validate = _engine.validate_plan
_base_verify = _engine.verify_manifest
DRIVERS = _engine.DRIVERS
CANONICAL_WRAPPER = _engine.CANONICAL_WRAPPER


def validate_plan(plan):
    _base_validate(plan)
    if set(plan) - {'phase', 'phase_kind', 'runs', 'randomization', 'allocation_provenance'}:
        raise ValueError('unexpected plan fields')
    if plan['phase_kind'] != 'confirmation' or len(plan['runs']) != 12:
        raise ValueError('v3 requires the complete fixed twelve-workflow confirmation')
    phase = plan['phase']; first = plan['runs'][0]['run_id']
    if not first.startswith(phase + '-') or not first.endswith('-001'):
        raise ValueError('numbered allocation identities must be phase-qualified')
    prefix = first[len(phase) + 1:-4]
    slots = plan['randomization'].get('actual_variant_slots_by_block')
    block_ids = [f'{phase}-block-{i:02d}' for i in (1, 2)]
    if not isinstance(slots, dict) or set(slots) != set(block_ids):
        raise ValueError('exactly two canonical block allocations are required')
    draws = []
    for block_id in block_ids:
        values = slots[block_id]
        if (not isinstance(values, list) or any(type(v) is not int for v in values)
                or tuple(values) not in _allocator.block_population()):
            raise ValueError('block allocation is outside the declared finite population')
        draws.append(tuple(values))
    choices = iter(draws)
    expected = _allocator.make_plan(phase, prefix, lambda _: next(choices))
    if {key: value for key, value in plan.items() if key != 'allocation_provenance'} != expected:
        raise ValueError('schedule/order/randomization differs from its declared exact allocation')


def dependency_pins():
    return {CONTROL_PATH: CONTROL_LOADED_SHA256, ENGINE_PATH: ENGINE_SHA256,
            ALLOCATION_PATH: ALLOCATION_LOADED_SHA256,
            _allocator.LEGACY_PATH: _allocator.LEGACY_SHA256}


def verify_current_dependencies():
    for path, expected in dependency_pins().items():
        if _engine.sha256(path) != expected:
            raise ValueError('executing control source changed: ' + str(path))


def prepare(args):
    verify_current_dependencies()
    if args.evidence_root.resolve() != REPO:
        raise ValueError('v3 evidence-root must preserve the actual control dependency tree')
    prepared_args = copy.copy(args)
    prepared_args.evidence = list(args.evidence) + list(dependency_pins())
    return _engine.prepare(prepared_args)


def verify_manifest(phase):
    verify_current_dependencies()
    phase = Path(phase).resolve()
    manifest = _base_verify(phase)
    entries = {entry['path']: entry for entry in manifest['files']}
    for path, expected in dependency_pins().items():
        frozen = phase / 'infrastructure' / 'evidence' / path.relative_to(REPO)
        entry = entries.get(str(frozen))
        if (entry is None or entry.get('role') != 'frozen-evidence'
                or entry.get('sha256') != expected or _engine.sha256(frozen) != expected):
            raise ValueError('control adapter or engine is missing from frozen identity: ' + str(frozen))
    return manifest


# Only this newly compiled private instance is parameterized. Existing imports and
# files keep their v2 schema. The actual engine __file__ and its digest are untouched.
_engine.CONDITIONS = CONDITIONS
_engine.EXPERIMENT_SCHEMA = EXPERIMENT_SCHEMA
_engine.validate_plan = validate_plan
_engine.verify_manifest = verify_manifest
run_environment = _engine.run_environment
run_phase = _engine.run_phase


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest='mode', required=True)
    prep = modes.add_parser('prepare', help='freeze inputs without provider admission')
    for name in ('phase-root', 'source-repo', 'bin-dir', 'observer-bin', 'subscription-wrapper',
                 'runtime-root', 'schedule', 'protocol', 'scorer-config', 'global-config'):
        prep.add_argument('--' + name, type=Path, required=True)
    prep.add_argument('--task-list-sha256', required=True)
    prep.add_argument('--evidence-root', type=Path, default=REPO)
    prep.add_argument('--evidence', type=Path, action='append', default=[])
    prep.add_argument('--identity-file', type=Path, action='append', default=[])
    for mode in ('verify', 'run'):
        modes.add_parser(mode).add_argument('--phase-root', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.mode == 'prepare': prepare(args)
        elif args.mode == 'verify': verify_manifest(args.phase_root)
        else:
            result = run_phase(args.phase_root)
            print(json.dumps(result, sort_keys=True))
            return 0 if (all(row['launcher_exit_code'] == 0 for row in result['runs'])
                         and not result['unexplained_config_drift_detected']
                         and not result['pending_config_attestation_at_finish']
                         and not result['frozen_input_integrity_errors']) else 1
        print(json.dumps({'phase_root': str(args.phase_root), 'mode': args.mode, 'provider_work_started': False}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'untracked-read control: {type(error).__name__}: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
