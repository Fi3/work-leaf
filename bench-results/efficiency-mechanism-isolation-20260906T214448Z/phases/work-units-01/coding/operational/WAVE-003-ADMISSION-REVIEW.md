# Wave 003 admission review

Result: **PASS; no deviations** in the bounded read-only audit at
`2026-09-07T02:43:04.379224+00:00`. The third-wave launch order is exactly **007 → 009 → 008**;
the before-launch journal sequence matches all first-three-wave frozen slots,
with no extra or duplicate admission snapshots.

Phase root: `/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.
Admitted manifest SHA256:
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.

## Admission snapshots and trust references

Locators refer to phase-local `config-history.jsonl`; lines are one-based and
snapshot indices are zero-based.

| Workflow suffix | Before-launch timestamp, UTC | Line | Snapshot index |
| --- | --- | ---: | ---: |
| 007 | 2026-09-07T02:41:02.566228+00:00 | 769 | 768 |
| 009 | 2026-09-07T02:41:02.583093+00:00 | 770 | 769 |
| 008 | 2026-09-07T02:41:02.590952+00:00 | 771 | 770 |

All three snapshots record `allows_admission: true`, no pending transition,
empty pending IDs and errors, and all six prior owned-workflow proofs below.
Their normalized parsed hashes equal the recorded baseline. Each original legacy
label remains `behavioral_or_unknown`; no historical flag is cleared.

| Prior workflow suffix | `trust-evidence.jsonl` line | Verified proof ID |
| --- | ---: | --- |
| 001 | 1 | `0e15fdeba140bbd5615b0d2a7ae3a37c93afad2034c5410ddbc06e895d1882c7` |
| 002 | 2 | `e09a1bb76e87708c578f7c6f738e6c9bd3db0b0238a78b89a6db132ed60c99fc` |
| 003 | 3 | `8a564d11c4635669faac163692ad69c862667f69277fdb6c299960232663cf08` |
| 004 | 6 | `3d9b96f518e13d0d9681c0e0d9b0491093b43cb805d7bfd24c3cbc76a4fc1cc8` |
| 005 | 5 | `50ef771978c9a079840913817de2bc7108c0528eeaf09d320cf600df1b182a5e` |
| 006 | 4 | `6fd87e7241401a7dbe6b30b4ef652ba62c1a77dc20b19bc960059af35dcc9780` |

This boundary check verifies the recorded admission references and safe hash
metadata. It does not repeat whole-phase external-global reconstruction or inspect
global configuration contents.

## Actual primary startup

Each row has exactly one primary AppServer invocation, the correct observer
run/study identity, and an actual canonical cwd owned by its exact scheduled row.
Every start uses `app-server --listen stdio://`, GPT-5.5, `xhigh`, and CLI
`0.153.4`. Each profile wrapper matches its recorded start/config hash and
delegates to the frozen subscription wrapper.

| Workflow | Primary invocation | Child PID | Pre-spawn completed monotonic ns | Child started monotonic ns |
| --- | --- | ---: | ---: | ---: |
| 007 | `00000448924886790377-3844443` | 3844475 | 448924891678508 | 448924897628088 |
| 009 | `00000448924886506612-3844440` | 3844472 | 448924891456482 | 448924897068867 |
| 008 | `00000448924902156465-3844489` | 3844535 | 448924906909242 | 448924911958846 |

Startup paths below are relative to the phase root; cwd paths are absolute.
Staging publication may rename the observation parent without changing invocation
identity.

- **007:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-007/work-leaf-3feature-bench.XhY7T1/repo`; start metadata `runs/work-units-01-workflow-007/.bench-artifact-publish.ee7c24a46f361c94d9d18594d1ef0756/observation/invocations/00000448924886790377-3844443/start.json`; sibling `child.json`. Actual profile wrapper SHA256: `469dd3ae8849d4d26db92cfe2888d698dfc7345f87c7e4902bf975f272b933b7`.
- **009:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-009/work-leaf-3feature-bench.uwRKlu/repo`; start metadata `runs/work-units-01-workflow-009/.bench-artifact-publish.376bb69651ee4f4a90dd0161d5d9ba43/observation/invocations/00000448924886506612-3844440/start.json`; sibling `child.json`. Actual profile wrapper SHA256: `70764ca50b484434b7f432042e207a92a0c2d3b315b52539c496caf74598f53a`.
- **008:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-008/work-leaf-3feature-bench.CnN1ui/repo`; start metadata `runs/work-units-01-workflow-008/.bench-artifact-publish.0575717a56a70e372164cf865a6a170e/observation/invocations/00000448924902156465-3844489/start.json`; sibling `child.json`. Actual profile wrapper SHA256: `aa5d9bb3faad9be779fde99c55daf4939ecb9f3419d6a62842bf0f7e08c6ca52`.

The observer metadata matches frozen release SHA256
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.
The subscription wrapper reference is `infrastructure/provider/codex`, frozen
SHA256 `0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`.
Observed startup settings are 1,000 ms usage grace, `forward` on resumed output,
raw-response metadata enabled, and project inventory required.

For every observation root, `project-layer-inventory/manifest.jsonl` line 1 is
`checkpoint:base` and line 2 is `pre-spawn`. Both are valid, have empty errors,
identify the exact actual cwd, and independently replay to the same entry digest
as earlier waves:

`08b328252e86541a1d1343ad84f1e599eb6a2bb9fa9334a0564ed04e3eb29370`.

Every pre-spawn completion precedes its actual child start, as shown above.
Inventory equality is a boundary snapshot fact, not continuous immutability.

## Scope

Only admission journals, safe proof identities, startup metadata, profile-wrapper
identity and project inventory hashes are examined. The complete old source/file
inventory is not redundantly rehashed. No active provider message bodies, global
configuration contents, credentials, token totals, ACK counts, benchmark outcome
comparisons, semantic labels or condition contrasts are inspected.

No provider is launched or interrupted. Active artifacts, frozen helpers,
protocols and settings are unchanged. The only write is this phase-local evidence
report; no commit is made.
