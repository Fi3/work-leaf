#!/usr/bin/env python3
"""Draw an offline, fixed twelve-workflow confirmation allocation once.

The caller supplies an already selected variant; this helper never reads outcomes or
selects a candidate. Each block independently uses SystemRandom.choice over all eighteen
balanced six-slot allocations whose two three-workflow waves both contain both labels.
Rows use phase-qualified numbered identities, independent of their assigned condition.

A new phase directory is the allocation claim. Existing directories are never reused,
including a directory left by a failed draw. ALLOCATION.json is input to runner.py's
prepare --schedule; allocation is not provider admission and does not launch a process.
The runner separately randomizes execution order within each declared wave.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import re
import sys


VARIANTS = ("ack-validation-unlimited", "command-guidance-neutral")


def block_population():
    """Return variant positions (one-based) in canonical lexicographic order."""
    return tuple(slots for slots in itertools.combinations(range(1, 7), 3)
                 if slots not in {(1, 2, 3), (4, 5, 6)})


def validate_parameters(phase, run_prefix, variant):
    if variant not in VARIANTS:
        raise ValueError("a permitted experimental variant must be supplied; no candidate is selected here")
    names = (phase, run_prefix, f"{phase}-{run_prefix}-012", f"{phase}-block-02")
    if any(not isinstance(name, str) or not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,95}", name)
           for name in names):
        raise ValueError("phase, run prefix, and resulting identifiers must be safe and at most 96 characters")


def json_digest(value):
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
    return hashlib.sha256(data).hexdigest()


def make_plan(phase, run_prefix, variant, choose):
    """Construct a plan; production passes SystemRandom.choice, tests inject finite draws."""
    validate_parameters(phase, run_prefix, variant)
    population = block_population()
    rows = []
    counts = {}
    actual = {}
    draw_indexes = {}
    slots = []
    for block_number in range(1, 3):
        chosen = choose(population)
        if chosen not in population:
            raise ValueError("random source returned an allocation outside the declared population")
        block_id = f"{phase}-block-{block_number:02d}"
        counts[block_id] = {"control": 3, variant: 3}
        actual[block_id] = list(chosen)
        draw_indexes[block_id] = population.index(chosen) + 1
        for position in range(1, 7):
            ordinal = (block_number - 1) * 6 + position
            wave = (block_number - 1) * 2 + (position - 1) // 3 + 1
            row = {"run_id": f"{phase}-{run_prefix}-{ordinal:03d}", "block_id": block_id,
                   "condition": variant if position in chosen else "control", "wave": wave}
            rows.append(row)
            slots.append({**row, "block_allocation_slot": position, "wave_allocation_slot": (position - 1) % 3 + 1})
    population_json = [list(item) for item in population]
    return {"phase": phase, "phase_kind": "confirmation", "runs": rows,
            "randomization": {
                "unit": "workflow", "scheme": "within_block", "mixed_waves": True,
                "method": "independent uniform random.SystemRandom.choice over the full eighteen-allocation population per block",
                "selected_variant": variant, "candidate_selection": "supplied by caller; no outcome inspection",
                "block_condition_counts": counts, "workflow_count": 12, "block_count": 2,
                "slots_per_block": 6, "workflows_per_wave": 3, "control_count": 6, "variant_count": 6,
                "independent_blocks": True, "per_block_population": population_json,
                "per_block_population_sha256": json_digest(population_json), "per_block_population_size": 18,
                "joint_population_size": 324, "per_block_allocation_probability": "1/18",
                "joint_allocation_probability": "1/324", "actual_variant_slots_by_block": actual,
                "actual_population_index_by_block_one_based": draw_indexes,
                "allocation_slots": slots,
                "slot_semantics": "allocation positions; runner freezes a separate execution-order shuffle within each wave",
            }}


def write_new(path, value):
    with path.open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True, allow_nan=False)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())


def allocate(phase_root, phase, run_prefix, variant):
    validate_parameters(phase, run_prefix, variant)
    phase_root = Path(phase_root)
    if not phase_root.is_absolute() or phase_root.name != phase:
        raise ValueError("phase-root must be absolute and its directory name must match phase")
    # Claim before invoking the random source: a failed draw cannot silently become a redraw.
    phase_root.mkdir(parents=True, exist_ok=False)
    provenance = {"schema": "work-leaf-confirmation-allocation-v1", "phase": phase,
                  "phase_root": str(phase_root.resolve()), "run_prefix": run_prefix, "selected_variant": variant,
                  "created_at": datetime.now(timezone.utc).isoformat(), "provider_work_started": False,
                  "helper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  "random_source": "random.SystemRandom.choice", "redraws_allowed": False}
    write_new(phase_root / "ALLOCATION-CLAIM.json", provenance)
    plan = make_plan(phase, run_prefix, variant, random.SystemRandom().choice)
    plan["allocation_provenance"] = provenance
    write_new(phase_root / "ALLOCATION.json", plan)
    return plan


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--phase-root", type=Path, required=True, help="new absolute phase directory; existing paths are refused")
    parser.add_argument("--phase", required=True, help="fresh phase identifier, matching phase-root basename")
    parser.add_argument("--run-prefix", required=True, help="condition-blind prefix for phase-qualified numbered run IDs")
    parser.add_argument("--variant", choices=VARIANTS, required=True, help="candidate already selected outside this helper")
    args = parser.parse_args()
    try:
        plan = allocate(args.phase_root, args.phase, args.run_prefix, args.variant)
        print(json.dumps({"phase": plan["phase"], "schedule": str(args.phase_root / "ALLOCATION.json"),
                          "workflow_count": len(plan["runs"]), "provider_work_started": False}, sort_keys=True))
        return 0
    except (OSError, ValueError) as error:
        print(f"confirmation allocation: {type(error).__name__}: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
