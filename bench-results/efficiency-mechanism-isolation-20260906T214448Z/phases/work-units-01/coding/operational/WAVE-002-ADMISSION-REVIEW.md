# Wave 002 admission review

## Result and admission locations

The completed bounded read-only review finds no deviation at the second-wave
boundary. The frozen and recorded launch order is exactly **006 → 005 → 004**.
The complete before-launch journal sequence at that review equals the first two
frozen waves, with no extra or duplicate admission snapshots.

Phase root: `/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.

The admitted `PHASE-MANIFEST.json` SHA256 remains
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.

Exact locators in phase-local `config-history.jsonl` follow. Line numbers are
one-based; snapshot indices are zero-based.

| Workflow | Before-launch timestamp, UTC | JSONL line | Snapshot index |
| --- | --- | ---: | ---: |
| work-units-01-workflow-006 | 2026-09-07T01:56:38.250357+00:00 | 497 | 496 |
| work-units-01-workflow-005 | 2026-09-07T01:56:38.267084+00:00 | 498 | 497 |
| work-units-01-workflow-004 | 2026-09-07T01:56:38.274925+00:00 | 499 | 498 |

Each admission snapshot records `allows_admission: true`,
`pending_transition: false`, empty pending IDs and errors, and the three verified
prior owned-workflow proofs below. Its normalized parsed hash matches the recorded
baseline. The original `behavioral_or_unknown` legacy label remains unchanged;
the distinct verified verdict does not clear that legacy fact.

| Prior workflow | `trust-evidence.jsonl` line | Verified proof ID |
| --- | ---: | --- |
| work-units-01-workflow-001 | 1 | `0e15fdeba140bbd5615b0d2a7ae3a37c93afad2034c5410ddbc06e895d1882c7` |
| work-units-01-workflow-002 | 2 | `e09a1bb76e87708c578f7c6f738e6c9bd3db0b0238a78b89a6db132ed60c99fc` |
| work-units-01-workflow-003 | 3 | `8a564d11c4635669faac163692ad69c862667f69277fdb6c299960232663cf08` |

## Actual startup identities

Each new workflow has exactly one primary AppServer capture with the correct
observer run/study identity. Its working directory is the exact canonical checkout
under `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-NNN/`,
using the checkout suffix below.

| Workflow suffix | Primary invocation | Child PID | Checkout suffix |
| --- | --- | ---: | --- |
| 006 | `00000446260578466031-3719533` | 3719581 | `work-leaf-3feature-bench.hDNVyV/repo` |
| 005 | `00000446260581883146-3719549` | 3719600 | `work-leaf-3feature-bench.Oo4sTk/repo` |
| 004 | `00000446260578462469-3719532` | 3719587 | `work-leaf-3feature-bench.60oQns/repo` |

Captured start-metadata locators, relative to the phase root:

- Workflow 006: `runs/work-units-01-workflow-006/.bench-artifact-publish.7c4d82fa85f13e929b65c73e30039e05/observation/invocations/00000446260578466031-3719533/start.json`; sibling `child.json` supplies the child-start clock. Inventory records are at the same observation root's `project-layer-inventory/manifest.jsonl`.
- Workflow 005: `runs/work-units-01-workflow-005/.bench-artifact-publish.88844278eeee5b86a86c9d5028ef850b/observation/invocations/00000446260581883146-3719549/start.json`; sibling `child.json` supplies the child-start clock. Inventory records are at the same observation root's `project-layer-inventory/manifest.jsonl`.
- Workflow 004: `runs/work-units-01-workflow-004/.bench-artifact-publish.8ed22ad55d0b04f84a97c71f27f16f28/observation/invocations/00000446260578462469-3719532/start.json`; sibling `child.json` supplies the child-start clock. Inventory records are at the same observation root's `project-layer-inventory/manifest.jsonl`.

These are the capture-time staging locators from the completed review. Eventual
publication can rename the staging artifact directory; invocation identity and
retained metadata remain the source keys.

## Settings and inventory checks

All three captures record GPT-5.5, `xhigh`, Codex CLI `0.153.4`, normal
`app-server --listen stdio://`, and the frozen release observer SHA256:

`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.

Each actual profile wrapper hash matches both its observer configuration metadata
and invocation start metadata, and delegates to the frozen subscription wrapper
at `infrastructure/provider/codex`, SHA256:

`0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`.

The metadata records 1,000 ms usage grace, resumed-output policy `forward`,
raw-response metadata enabled, and project inventory required. The runtime and
observer locations reference the admitted `infrastructure/bin` executables;
this check does not unnecessarily rehash the complete earlier source inventory.

Each workflow has one valid `checkpoint:base` and one valid `pre-spawn`
inventory for its exact actual root, with empty errors. Independent entry-digest
replay matches the same baseline and first-wave SHA256:

`08b328252e86541a1d1343ad84f1e599eb6a2bb9fa9334a0564ed04e3eb29370`.

For all three workflows, pre-spawn completion on the monotonic clock precedes or
equals the captured child-start clock. Snapshot equality is not a claim of
continuous filesystem immutability.

## Scope

This report preserves the completed admission audit; preparing it only retrieves
safe journal line indices and proof identifiers. It does not repeat the audit.
No global configuration contents, active provider messages, token totals, ACK
counts, benchmark outcomes, semantic classifications, or condition contrasts are
inspected. No provider, active artifact, helper, protocol, or global setting is
changed. No provider is launched or interrupted, and no commit is made.
