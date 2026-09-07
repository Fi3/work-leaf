# Wave 004 admission review

Result: **PASS; no deviations** in the bounded read-only audit completed at
`2026-09-07T03:39:29.547769+00:00`. The fourth-wave admission order is exactly
**012 → 010 → 011**. The complete before-launch journal sequence matches the
12 frozen launch slots, without extra or duplicate admission snapshots.

Phase root: `/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01`.
Manifest SHA256, independently rechecked against its admission receipt:
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
Schedule SHA256, independently rechecked:
`38e3de56326ddff9868601bc0c67cad0aa948d1709a6c6f76d6c7954d2543aec`.

## Admission snapshots and trust references

Locators refer to phase-local `config-history.jsonl`; lines are one-based and
snapshot indices are zero-based.

| Workflow suffix | Before-launch timestamp, UTC | Line | Snapshot index |
| --- | --- | ---: | ---: |
| 012 | 2026-09-07T03:35:17.436436+00:00 | 1099 | 1098 |
| 010 | 2026-09-07T03:35:17.454839+00:00 | 1100 | 1099 |
| 011 | 2026-09-07T03:35:17.463830+00:00 | 1101 | 1100 |

Every snapshot records `allows_admission: true`, `pending_transition: false`,
empty pending IDs and errors, and exactly the nine prior proof IDs below. Every
proof's classification timestamp precedes these admissions. The distinct verdict
is `verified_own_workflow_trust_transition`; both original legacy labels remain
`behavioral_or_unknown`. No historical flag is cleared.

All three snapshots retain raw global SHA256
`a99fe41e37b68eee5bf0235d2d19281ba1664640f2247afde29aab14c6091d2c`
and parsed SHA256
`d27a275af6755c0a6e815cf0672222efb3fb0f60df66f9447a6d84154afccbbf`.
Their normalized parsed SHA256 equals the frozen baseline:
`e91e44495343edc3d4ced57caa5bb296c01f0447688d27a1577f8d1243a3a389`.

| Prior workflow suffix | `trust-evidence.jsonl` line | Verified proof ID |
| --- | ---: | --- |
| 001 | 1 | `0e15fdeba140bbd5615b0d2a7ae3a37c93afad2034c5410ddbc06e895d1882c7` |
| 002 | 2 | `e09a1bb76e87708c578f7c6f738e6c9bd3db0b0238a78b89a6db132ed60c99fc` |
| 003 | 3 | `8a564d11c4635669faac163692ad69c862667f69277fdb6c299960232663cf08` |
| 004 | 6 | `3d9b96f518e13d0d9681c0e0d9b0491093b43cb805d7bfd24c3cbc76a4fc1cc8` |
| 005 | 5 | `50ef771978c9a079840913817de2bc7108c0528eeaf09d320cf600df1b182a5e` |
| 006 | 4 | `6fd87e7241401a7dbe6b30b4ef652ba62c1a77dc20b19bc960059af35dcc9780` |
| 007 | 7 | `cb624971c592d2e43586e0d28aa1f052f494b7e2c7c7ae21aa28f30ec2810004` |
| 008 | 8 | `cce004974c65c2317dc36117f8faeafd44d144a1608cfd2bccbd213ace707dc9` |
| 009 | 9 | `e936cd1f777cb680df741f838234fad7b789418533c19603c4e4bcc200664198` |

This boundary review checks recorded proof references and safe hash metadata, not
whole-phase external-global reconstruction. Global configuration contents are not
read; final external replay remains a separate closure gate.

## Actual primary startup

Each new row has exactly one primary AppServer invocation, correct observer
run/study/block identities, and an actual canonical cwd owned by that exact
scheduled runtime directory. Each startup uses `app-server --listen stdio://`,
GPT-5.5, `xhigh`, CLI `0.153.4`, the frozen base/driver commits, and the declared
existing ChatGPT subscription route. Actual profile-wrapper bytes match both
start and observer-config hashes. Shell-parsed delegation is exactly the frozen
subscription wrapper plus the prescribed model/effort overrides and original
arguments.

| Workflow | Primary invocation | Child PID | Pre-spawn completed monotonic ns | Child started monotonic ns |
| --- | --- | ---: | ---: | ---: |
| 012 | `00000452179765332046-4005151` | 4005199 | 452179770680879 | 452179776885762 |
| 010 | `00000452179764996037-4005148` | 4005200 | 452179770844728 | 452179777523348 |
| 011 | `00000452179770293816-4005169` | 4005214 | 452179775295954 | 452179781063907 |

Startup paths are relative to the phase root; cwd paths are absolute. Artifact
publication may rename the observation parent without changing invocation identity.

- **012:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-012/work-leaf-3feature-bench.XgvriO/repo`; start metadata `runs/work-units-01-workflow-012/.bench-artifact-publish.9592b962fb8e14f67e572d93515ef8b8/observation/invocations/00000452179765332046-4005151/start.json`; sibling `child.json`. Profile SHA256: `e393a870a72d00b56ab8ad8c52d5ed121daa99abb73f81f361bc9ceb4a10d96e`.
- **010:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-010/work-leaf-3feature-bench.d6U8cB/repo`; start metadata `runs/work-units-01-workflow-010/.bench-artifact-publish.1dce579e778eb28e48164a2eb0a46ef0/observation/invocations/00000452179764996037-4005148/start.json`; sibling `child.json`. Profile SHA256: `920a46f2e28b47f78a152ba83b3004c130da21eb266df89eb8070da7bbe70f6d`.
- **011:** cwd `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-011/work-leaf-3feature-bench.2mUGCF/repo`; start metadata `runs/work-units-01-workflow-011/.bench-artifact-publish.f6a622810b113dcf4f2e0131e63022e2/observation/invocations/00000452179770293816-4005169/start.json`; sibling `child.json`. Profile SHA256: `994c25df6b4ba1fd09e98ba785cf305a49b3bdf38994ec284754716288d6b1c6`.

Observed observer metadata matches frozen release SHA256
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.
The subscription wrapper reference is `infrastructure/provider/codex`, frozen
SHA256 `0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`.
Startup settings are 1,000 ms usage grace, `forward` on resumed output,
raw-response metadata enabled, and project inventory required.

The admitted metadata retains the previously verified executable and source
identities: runtime work-leaf SHA256
`07c403c5290371a7e1700cb9d03127b9b9a8d384c55a98692115b2fbf01fe1db`,
orchestrator SHA256
`9f9a6c88a45bd39a99b6c7eeae82e0037575592ed75d8a41a013d3792ade95fa`,
driver source commit `3c907f7266b63efa1558213f3e49587e999b899d`,
runner SHA256 `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e`,
and trust helper SHA256
`37122d613c96b21fc4cd078ad8dfd7784e0fa60c1f430bb2ff78cf6d82ea88b8`.
The frozen source-build attestation reference is
`infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/BUILD-ATTESTATION-WORK-UNITS.json`,
SHA256 `aefdc03b08740ee1d6ec93a5a9f17251ac6c2d9f565d8e5ac52fb35f8e86534e`.
The complete old input inventory is not redundantly rehashed in this boundary audit.

For each observation root, `project-layer-inventory/manifest.jsonl` line 1 is
`checkpoint:base` and line 2 is `pre-spawn`. Both are valid, have empty errors,
identify the exact actual cwd/run/study, and independently replay their entry
digest to the shared baseline:

`08b328252e86541a1d1343ad84f1e599eb6a2bb9fa9334a0564ed04e3eb29370`.

Each base completion precedes primary proxy startup, and every pre-spawn
completion precedes its actual child start as shown above. These are boundary
snapshot facts, not a claim of continuous immutability.

## Scope

The audit examines admission journals, safe proof identities, startup metadata,
profile-wrapper identity, and project inventory hashes. It does not inspect active
provider message bodies, global configuration contents, credentials, token totals,
ACK counts, benchmark outcome comparisons, semantic labels or condition contrasts.

No provider is launched or interrupted. Active artifacts, frozen helpers,
protocols and settings are unchanged. The audit writes only this phase-local
report; separately authorized closed-workflow 009 packet preparation does not
authorize reading its semantic contents. No commit is made.
