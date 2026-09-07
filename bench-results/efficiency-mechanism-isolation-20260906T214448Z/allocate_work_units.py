#!/usr/bin/env python3
"""Claim and draw one fixed twelve-workflow work-unit allocation, without generation.

The hash-pinned v1 helper supplies only its reviewed finite-population construction.
This module has its own condition scope, create-new claim, provenance and CLI; it does
not select a winner from the original screen or modify that screen's helper.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import random
import sys


VARIANT = "buildable-work-unit-incremental"
LEGACY_PATH = Path(__file__).with_name("allocate_confirmation.py")
LEGACY_SHA256 = "57c2188bf3f641a12eaa8942ace1d8bc5ce3aa5cbe7a8fc8e0fd1573af121ff6"
if hashlib.sha256(LEGACY_PATH.read_bytes()).hexdigest() != LEGACY_SHA256:
    raise ValueError("frozen finite-allocation helper changed")
_spec = importlib.util.spec_from_file_location("work_units_finite_population", LEGACY_PATH)
_population = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_population)
# Parameterize this private module instance, never the source or an admitted manifest.
_population.VARIANTS = (VARIANT,)
block_population = _population.block_population


def make_plan(phase, run_prefix, choose):
    plan = _population.make_plan(phase, run_prefix, VARIANT, choose)
    plan["randomization"]["candidate_selection"] = "new prospective work-unit hypothesis; no screen winner"
    return plan


def allocate(phase_root, phase, run_prefix):
    _population.validate_parameters(phase, run_prefix, VARIANT)
    phase_root = Path(phase_root)
    if not phase_root.is_absolute() or phase_root.name != phase:
        raise ValueError("phase-root must be absolute and its directory name must match phase")
    phase_root.mkdir(parents=True, exist_ok=False)
    provenance = {"schema": "work-leaf-work-unit-allocation-v2", "phase": phase,
                  "phase_root": str(phase_root.resolve()), "run_prefix": run_prefix,
                  "selected_variant": VARIANT, "provider_work_started": False,
                  "created_at": datetime.now(timezone.utc).isoformat(), "redraws_allowed": False,
                  "helper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  "finite_population_helper_sha256": LEGACY_SHA256,
                  "random_source": "random.SystemRandom.choice"}
    _population.write_new(phase_root / "ALLOCATION-CLAIM.json", provenance)
    plan = make_plan(phase, run_prefix, random.SystemRandom().choice)
    plan["allocation_provenance"] = provenance
    _population.write_new(phase_root / "ALLOCATION.json", plan)
    return plan


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase-root", type=Path, required=True)
    parser.add_argument("--phase", required=True)
    parser.add_argument("--run-prefix", required=True)
    args = parser.parse_args()
    try:
        plan = allocate(args.phase_root, args.phase, args.run_prefix)
        print(json.dumps({"phase": plan["phase"], "schedule": str(args.phase_root / "ALLOCATION.json"),
                          "workflow_count": len(plan["runs"]), "provider_work_started": False}, sort_keys=True))
        return 0
    except (OSError, ValueError) as error:
        print(f"work-unit allocation: {type(error).__name__}: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
