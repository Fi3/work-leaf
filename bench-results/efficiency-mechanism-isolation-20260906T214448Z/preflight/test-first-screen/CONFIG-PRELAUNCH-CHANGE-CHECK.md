# Prelaunch configuration discrepancy: finite check

**Unresolved substantive-or-unknown change.** The exact current global raw SHA is
`d36b9caec082759492578037173fe3bca099aad63f8e2553314601d2506e366d`,
not the 09:06 receipt's `21bcb482…`. It stayed byte-identical throughout every
read-only check. No configuration contents, credentials or arbitrary setting values
were printed/copied; the original [reconstruction receipt](CONFIG-RECONSTRUCTION.json)
is preserved.

The [separate JSON](CONFIG-PRELAUNCH-CHANGE-CHECK.json) records source hashes, finite
domains and tool receipts. All 16 existing proof rows still match exactly
`{trust_level:trusted}`; all four proof-source hashes match. Removing only those
exact project keys produces parsed SHA `d2bdf854…` and behavioral SHA `01c01798…`,
neither equal to saved W's `e91e4449…` / `78cd609b…` (tool `d5016f`).

The frozen safe-enum summary remains empty because it does not include the currently
observed installed enums `gpt-6-astra` / `max`. Those two values were reported only
through the explicit installed enum domain (`b0d66c`), not by exporting arbitrary
configuration values. Their prior values are not inferred.

## Exhausted finite equality tests

| Read-only in-memory test after the exact 16 removals | Candidates | Exact matches |
| --- | ---: | ---: |
| Remove one of six current top-level keys; compare full parsed hash (`fd71cd`) | 6 | 0 |
| Substitute only the explicit five-model × six-effort domain; compare full parsed hash (`97736f`) | 30 | 0 |
| Same domain under the existing four-notice behavioral rule (`ce06bf`) | 30 | 0 |
| Remove one remaining exact-trusted project entry; compare full parsed and behavioral hashes (`ce06bf`) | 293 | 0 |

The top-level names are `model`, `model_reasoning_effort`, `notice`, `projects`,
`tui` and `personality`. No nested arbitrary values were exported. The exact four
notice keys are the existing `runner_work_units.py::config_snapshot` rule, not a
new blanket exclusion. No unknown field is ignored.

Shallow configuration-filename discovery (`23ee5f`) found the current global file
and old run-local config/observer/study filenames, but no root global-backup filename.
Run-local configuration contents were not opened or substituted for an old global
source. No further search or wider value domain belongs to this check.

## Admission consequence

The original source-pinned subscription wrapper overrides model/effort to
`gpt-5.5`/`xhigh`; that does **not** prove the unidentified configuration
difference irrelevant. The finite tests establish no exact sole-added key,
sole-added trust entry, or model/effort-plus-known-notice explanation.

Parent reports raw-identity guard `112c40` stopped before phase/provider creation;
only the fixed plan was subsequently written (`5ea050`). This note does not
relabel the previous compatibility cutoff, edit global settings, permit generation,
or authorize a new comparison scope. Exact-change evidence or explicit scope
direction remains necessary.
