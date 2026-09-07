#!/usr/bin/env python3
"""Claim one v3 twelve-workflow allocation without launching provider work.

Only the pinned predecessor's finite population/construction is reused, in a private
module instance. The original helper and every admitted allocation stay immutable.
"""
import argparse
from datetime import datetime, timezone
import hashlib
from pathlib import Path
import random
import sys
import types

VARIANT = 'untracked-read-inline'
LEGACY_PATH = Path(__file__).with_name('allocate_confirmation.py')
LEGACY_SHA256 = '57c2188bf3f641a12eaa8942ace1d8bc5ce3aa5cbe7a8fc8e0fd1573af121ff6'
_source = LEGACY_PATH.read_bytes()
if hashlib.sha256(_source).hexdigest() != LEGACY_SHA256:
    raise ValueError('finite-allocation engine differs from its pin')
_population = types.ModuleType('untracked_read_finite_population')
_population.__file__ = str(LEGACY_PATH)
exec(compile(_source, str(LEGACY_PATH), 'exec'), _population.__dict__)
_population.VARIANTS = (VARIANT,)
block_population = _population.block_population


def make_plan(phase, run_prefix, choose):
    plan = _population.make_plan(phase, run_prefix, VARIANT, choose)
    plan['randomization']['candidate_selection'] = 'distinct prospective source-representation hypothesis; no selected screen winner'
    return plan


def allocate(phase_root, phase, run_prefix):
    _population.validate_parameters(phase, run_prefix, VARIANT)
    if hashlib.sha256(LEGACY_PATH.read_bytes()).hexdigest() != LEGACY_SHA256:
        raise ValueError('finite-allocation source changed after loading')
    phase_root = Path(phase_root)
    if not phase_root.is_absolute() or phase_root.name != phase:
        raise ValueError('phase-root must be absolute and its basename must match phase')
    phase_root.mkdir(parents=True, exist_ok=False)
    provenance = {
        'schema': 'work-leaf-untracked-read-allocation-v3', 'phase': phase,
        'phase_root': str(phase_root.resolve()), 'run_prefix': run_prefix,
        'selected_variant': VARIANT, 'provider_work_started': False,
        'created_at': datetime.now(timezone.utc).isoformat(), 'redraws_allowed': False,
        'helper_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'finite_population_helper_sha256': LEGACY_SHA256,
        'random_source': 'random.SystemRandom.choice',
    }
    _population.write_new(phase_root / 'ALLOCATION-CLAIM.json', provenance)
    plan = make_plan(phase, run_prefix, random.SystemRandom().choice)
    plan['allocation_provenance'] = provenance
    _population.write_new(phase_root / 'ALLOCATION.json', plan)
    return plan


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--phase-root', type=Path, required=True)
    parser.add_argument('--phase', required=True)
    parser.add_argument('--run-prefix', required=True)
    args = parser.parse_args()
    try:
        plan = allocate(args.phase_root, args.phase, args.run_prefix)
        print(f"Allocation saved: {args.phase_root / 'ALLOCATION.json'}; {len(plan['runs'])} workflows; no provider work")
        return 0
    except (OSError, ValueError) as error:
        print(f'untracked-read allocation: {type(error).__name__}: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
