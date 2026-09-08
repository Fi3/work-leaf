#!/usr/bin/env python3
"""Fixed three-workflow automatic-refresh screen over the unchanged supervisor.

Only launch order is randomized. There is no control allocation or replacement.
The pinned engine retains its environment, timing, trust and outcome handling.
Its frozen-input checking remains O(W*E); this screen fixes W=1 and N=3.
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
ADAPTER_PATH = Path(__file__).resolve()
ADAPTER_SHA256 = hashlib.sha256(ADAPTER_PATH.read_bytes()).hexdigest()
ENGINE_PATH = HERE / 'runner_work_units.py'
ENGINE_SHA256 = '2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e'
EXPERIMENT_SCHEMA = 'work-leaf-bench-experiment-v7'
CONDITIONS = ('automatic-changed-refresh-full',)
METHOD = 'fixed three workflows of one candidate; only launch order randomized; saved baseline reused'

_data = ENGINE_PATH.read_bytes()
if hashlib.sha256(_data).hexdigest() != ENGINE_SHA256:
    raise ValueError('pinned automatic-refresh supervisor source changed')
_engine = types.ModuleType('automatic_refresh_supervisor')
_engine.__file__ = str(ENGINE_PATH)
exec(compile(_data, str(ENGINE_PATH), 'exec'), _engine.__dict__)
_base_validate = _engine.validate_plan
_base_verify = _engine.verify_manifest
DRIVERS = _engine.DRIVERS
CANONICAL_WRAPPER = _engine.CANONICAL_WRAPPER


def make_plan(phase):
    if not _engine.identifier(phase) or len(phase) > 76:
        raise ValueError('phase must leave room for explicit run and block identifiers')
    block = phase + '-block-01'
    return {'phase': phase, 'phase_kind': 'screening',
            'runs': [{'run_id': f'{phase}-workflow-{index:03d}', 'condition': CONDITIONS[0],
                      'wave': 1, 'block_id': block} for index in range(1, 4)],
            'randomization': {'method': METHOD, 'unit': 'workflow', 'scheme': 'within_block',
                              'mixed_waves': False,
                              'block_condition_counts': {block: {CONDITIONS[0]: 3}}}}


def validate_plan(plan):
    _base_validate(plan)
    if plan != make_plan(plan['phase']):
        raise ValueError('automatic-refresh screen requires exactly three declared workflows; no controls or replacements')


def dependency_pins():
    return {ADAPTER_PATH: ADAPTER_SHA256, ENGINE_PATH: ENGINE_SHA256}


def verify_current_dependencies():
    for path, expected in dependency_pins().items():
        if _engine.sha256(path) != expected:
            raise ValueError('executing automatic-refresh source changed: ' + str(path))


def prepare(args):
    verify_current_dependencies()
    if args.evidence_root.resolve() != REPO:
        raise ValueError('automatic-refresh evidence root must preserve the actual dependency tree')
    prepared = copy.copy(args)
    prepared.evidence = list(args.evidence) + list(dependency_pins())
    return _engine.prepare(prepared)


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
            raise ValueError('automatic-refresh adapter/engine is absent from frozen identity: ' + str(frozen))
    return manifest


_engine.CONDITIONS = CONDITIONS
_engine.EXPERIMENT_SCHEMA = EXPERIMENT_SCHEMA
_engine.validate_plan = validate_plan
_engine.verify_manifest = verify_manifest
run_environment = _engine.run_environment
run_phase = _engine.run_phase


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest='mode', required=True)
    plan = modes.add_parser('plan', help='write fixed allocation without provider admission')
    plan.add_argument('--phase', required=True)
    plan.add_argument('--output', type=Path, required=True)
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
        if args.mode == 'plan':
            _engine.write_new(args.output, make_plan(args.phase))
        elif args.mode == 'prepare':
            prepare(args)
        elif args.mode == 'verify':
            verify_manifest(args.phase_root)
        else:
            result = run_phase(args.phase_root)
            print(json.dumps(result, sort_keys=True))
            return 0 if (all(row['launcher_exit_code'] == 0 for row in result['runs'])
                         and not result['unexplained_config_drift_detected']
                         and not result['pending_config_attestation_at_finish']
                         and not result['frozen_input_integrity_errors']) else 1
        print(json.dumps({'mode': args.mode, 'provider_work_started': False}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'automatic-refresh supervisor: {type(error).__name__}: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
