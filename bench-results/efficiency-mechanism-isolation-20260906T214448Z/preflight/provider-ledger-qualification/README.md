# Separate provider-ledger qualification

This private offline helper distinguishes an unavailable controller cross-check
from unqualified provider-response accounting. It does not modify Work Leaf,
controller state, observer errors, capture-completeness flags, original accounting
results, reports or their source-membership exceptions. It admits no provider,
replacement, benchmark or contrast. Actual saved-workflow execution requires a
separately frozen common-population supplement and independent helper review.

## Source-bound diagnosis

[DIAGNOSIS.json](DIAGNOSIS.json) retains R005–006's original errors, tails,
source/report qualifications, all 16 captured launch-owned threads, and all 33
turns of the three agents missing a controller row. Each of those turns has an
actual completed public directive before its usage notification, plus one exact
raw/native response identity and matching four counters. Counts are identities,
not token totals. Both complete recorded response inventories were independently
matched without summation; no accounting helper was rerun for that diagnosis.

The retained driver `bench-three-features::capture_observation_controller_state`
selects its saved controller state. Observer `record_controller_usage` omits
sessions whose usage is null. `analyze_app_server` stops its controller replay
at a completed directive and includes only turns with at least one earlier usage
event. The provider backend returns at that same handoff; subsequent provider
usage remains in raw/native evidence, not the controller's streamed aggregate.

Observer `capture_complete` means its entire error list is empty. The frozen
measurement gate separately requires completeness to equal absence of interrupted
tails. Therefore R006's extra gate reason is a consequence of its controller
errors, not an independently demonstrated transport loss. The original UNKNOWN
classifications remain authoritative original results. R005's four conditional
tail gaps are unrelated reviewer turns and remain present; R006 has no such gap.

The original common-twelve result, source-report supplement and their execution
limitations remain in [the accounting record](../../EVIDENCE-RETAINED-READ-ACCOUNTING.md).
In particular, later complete source checks do not retroactively satisfy an
omitted report prehash or intermediate recheck.

## Interface and qualification rule

`audit_run(entry, frozen, sessions_root, admission)` is the only source-bound
entrypoint. It returns the complete untouched original helper result alongside
`qualification`, `original_source_qualification` and source hashes. There is no
whole-phase CLI, comparison, randomization or alternate endpoint.

The caller must execute the exact reviewed source bytes, retain their digest,
and supply a separately pinned admission object with these fields:

| Field | Meaning |
| --- | --- |
| `accounting_helper` | Actual unchanged frozen `accounting_untracked_reads.py` path; SHA `c365aa86…` is fixed in code. |
| `observer_source` | Retained controller-grammar source with fixed SHA `ab7ede87…`. |
| `original_result_sha256` | Previously declared full canonical original result identity, including source paths; no numeric-field projection. |
| `source_sha256` | Complete predeclared canonical regular-file inventory, including this helper, its original dependencies, all original consumed sources, final state, controller usage, prompt trace, process inventory, raw/forwarded captures and native metadata/files. |
| `original_source_qualification` | Original source-membership/report/execution exceptions copied intact; not a replacement verdict. |

All admitted sources are verified before calling the unchanged accounting helper
and at closure. Its helper/dependency bytes are checked before exact compilation;
cached bytecode is not used for that dependency chain. Its fresh full result must
match the separately declared original digest. Thus the qualifier cannot accept
a fabricated saved success boolean or quietly change original accounting.

Additional qualification joins every typed accepted thread/turn, forwarded input,
public user item and explicit native `user.text` item. Native session/turn metadata
must match the declared model, effort, cwd and CLI scope. All captured threads,
including title generation, require exact owned first-policy delivery; actual
v2/v3 traces retain their own schemas. No v3 record is synthesized for a v2 run.
Every recorded response must match raw/native/original identity and four usage
counters. Native item and public item IDs remain separate.

For a missing controller row, final-state usage and replayed controller usage
must both be absent, every accepted turn must have a complete directive before
all usage events, and actual post-directive response evidence must exist. The
absence is reported as unavailable, never zero. Present controller rows must
match final state and the independently reconstructed pre-directive usage.

Only those exact missing-controller reasons and their mechanically explained
completeness consequence can be separated. Every unrelated source, response,
scope, invocation, replay, model or tail error rejects qualification. Original
valid results keep their measurement unchanged. For a qualified missing-row case,
the original recorded ledger and frozen 1,178,000-per-proved-tail rule supply a
separate measurement; unsupported tails keep null upper bounds. Hidden provider
call completeness is not asserted. Phase configuration, outcome and source
exceptions are never cleared by this layer.

## Verification and limits

`python -B -m unittest -v test_qualify.py` passes 16 tests. The initial tests failed
before the module existed. Source-wrapper tests failed before their API existed;
additional RED checks reproduced whitespace/empty-rest grammar differences and
original-module execution before dependency admission. The final tests cover
those cases, all missing-controller negative rules, unchanged original results,
finite versus unknown tails, typed identities, duplicate/cross-source scope,
public/native full inputs, strict closed JSONL and source confinement. The
accepted-turn coverage regression also reproduced an eventless failed turn
escaping the missing-controller proof; exact accepted/witness set equality rejects
that case. Missing-agent membership uses a set, not a per-turn linear list scan.
The positive wrapper fixture mocks only the original accounting execution; it is not
a saved-workflow qualification or provider observation.

No actual saved workflow has been passed through this new `audit_run` during
development. The separately retained diagnostic uses existing source evidence,
not new model output. No real-agent workflow is affected by this offline helper;
no provider verification or production architecture/API change belongs to it.
Architecture and benchmark operator documentation were inspected; this local
interface description contains the relevant private operating constraints.

All joins are keyed and source/directive scans are linear in bytes/events. Fixed
protocol alternatives have constant cardinality. Sorting identity lists costs
O(N log N); no new O(N²) conversation or source join exists. Source input is capped
at 20,000 files, 1 GiB/file and 32 GiB of reads. Endpoint checks assume retained
closed artifacts and are not protection against a hostile concurrent host.
Bodies, tool arguments and private reasoning are not copied into evidence output.

Reviewed handoff identities (independent review pending):

- `qualify.py`: `ef4c28ed51030b775c6d75e956c6e1019f4fef655b48f968c4073d9cc6812e2e`.
- `test_qualify.py`: `9752aab1762697133a91fc6913ad89825e924a55591e600f24199033970bed94`.
- `DIAGNOSIS.json`: `1b4d55f44c04bab3ccff88b723e225a53dfca3307db525f37fbc1e2f367a5243`.
