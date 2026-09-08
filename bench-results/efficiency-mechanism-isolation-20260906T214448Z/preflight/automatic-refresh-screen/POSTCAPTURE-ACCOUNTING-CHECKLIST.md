# Prospective C08 postcapture checklist

This is source-only preparation, not execution authorization or a phase result.
All three workflows must close before payload inspection or accounting. No live
artifact was hashed or analyzed for this checklist; no accounting, observer,
native extraction, provider or Cargo command was invoked.

## Fixed inputs and population

Paths below use `S` for this study directory and `P=S/phases/automatic-refresh-01`.
`F=P/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z`
is the frozen helper/evidence directory, not the moving study source.

Authority: [protocol](../../PROTOCOL-AUTOMATIC-REFRESH-SCREEN.md), SHA256
`889fc49bdc40224cc9616a32cc743a351040c82e02210f700d2920d022f8fa2d`, and
[phase manifest](../../phases/automatic-refresh-01/PHASE-MANIFEST.json), SHA256
`cfdfd1342c3fc31fe3286a366b743dc37ecbbf4fd4605f64a6ab404a07e9d6c8`.

The three new accounting entries are exactly
`automatic-refresh-01-workflow-001`, `automatic-refresh-01-workflow-002`, and
`automatic-refresh-01-workflow-003`, all condition
`automatic-changed-refresh-full`, wave1, block `automatic-refresh-01-block-01`.
For each complete ID `r`, the frozen schedule names:

- artifact: `P/runs/r/r-three-feature-bench-artifacts`;
- report: that artifact's `report.json`;
- prompt trace: `P/prompt-events/r.jsonl`;
- experiment manifest: `P/experiments/r.json`.

Use the eventual terminal score entry, reconciled to this schedule and the
actual process receipt; do not invent `started_at`, closure or success. The
runner fixes three rows/one wave, `mixed_waves=False`, and only randomized launch
order. It does not allocate controls or replacements.

The six unchanged measurement receipts are
`F/phases/candidate-screen-01/postcapture/retained-baselines/<W-ID>/COMMON-ACCOUNTING-ORIGINAL.json`:

| W-ID suffix (`work-units-01-workflow-`) | Frozen receipt SHA256 |
| --- | --- |
| 002 | `202e3dab2c68f08dcc845d26ea086a2738639c97f6a78dc32f30fd7f4958a0e9` |
| 004 | `a058ed0b140850a3cdf7a36143f840ec273d0741b99ff052a037dd7c3985b460` |
| 006 | `7493fbcbbc8a2ca45d6cd1574583a57f3baeb701ef10ebdfba3cdb2335f68eea` |
| 009 | `cde7df45117b967bb1d1990bdb6f7624f1f4970f3bee48e749d052d3eb5b188c` |
| 010 | `75ae137ca71d540f0b6d27e487e2d43eb2b8c8d7e584ee1c178d59363ecb3600` |
| 012 | `747c8faefd03ffefce91d2e42ab9c6d99f44e107e899afb8fb8fca2d203318f6` |

Failed W010 remains included. Historical reuse is descriptive, not a fresh
randomized arm comparison. Do not substitute later accounting supplements.

## Closed-source and delivery gates

1. After all three close, reconcile complete invocation directories to the
   invocation inventory, original start/child/end records, raw stream digests,
   typed accepted starts, interrupts and terminal outcomes. Preserve original
   observer analyze/extract exits and diagnostics before any separate derivative;
   do not silently rerun or rewrite them. Pin all consumed closed artifacts,
   including `report.json`, before the fixed accounting execution and recheck
   them afterward. Preserve trust/configuration, timeout and capture failures.
2. Recheck the immutable phase/admission dependency closure: actual runtime and
   observer binaries, executable/provider chain, five frozen drivers, scorer,
   protocol, configuration and experiment manifests, source/build attestation,
   helper tree and baseline receipts. Initial mutable project files are separate
   launch inventories, not immutable postlaunch endpoints. Compare every recorded
   project boundary and child-before-launch evidence; a boundary snapshot is not
   continuous filesystem visibility. The frozen compatibility record remains
   [BASELINE-COMPATIBILITY.md](BASELINE-COMPATIBILITY.md), not a drift waiver.
3. Establish every captured accepted thread/turn and exact original → forwarded
   → public user → native explicit-turn input, with typed RPC identity and
   consume-once, occurrence-ordered joins. Include title and usage-less threads;
   native/public item IDs are distinct. Native context must establish actual
   model/effort/cwd/CLI and tool/action inventory, not infer these from prompts or
   a usage-derived thread list. Retain unmatched, rejected, failed and incomplete
   records. Check actual raw/grace1000/forward/project-inventory settings and
   metadata-only request rewrites; a 1000-ms maximum is not a one-second wait on
   every interruption. No future C15 observer mode belongs to this phase.
4. Validate every v7 automatic-refresh occurrence, including identity-only
   reasons, both rejection sites, exact owned UTF-8 spans, held snapshot
   bytes/digests and untouched diagnostics/non-owned bytes. Verify eligible
   changed diff → full-current replacement without a new current-text cap, and
   unchanged oversized48-KiB diff/untracked8-KiB/empty/error branches. Preserve
   snapshot-before-evidence/send semantics and undelivered occurrences. Link
   subsequent actions separately; zero exposure is not a reason to discard a
   workflow or manufacture a conflict.

The v4 [audit_candidate_delivery.py](../../audit_candidate_delivery.py), SHA256
`ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1`,
is not a v7 validator. Its indexed typed-RPC/public-item/occurrence primitives
are reusable only within a separately reviewed adapter; its v4 schema, runtime
pins, policy/repeat/resupply validators and title ownership rules cannot be
relabeled v7. Its public-item join is not a native-rollout join. The source-only
`preflight/c08-natural-delivery` design/tests are pending, not a qualified helper.

## Exact accounting interface and limits

Declare one source-bound scope for three new `audit_run(entry, frozen,
Path('/home/user/.codex/sessions'))` calls only. `frozen` is the parsed JSON object
from the exact verified `P/PHASE-MANIFEST.json`, not its path string. Compile the
verified actual `F/accounting_untracked_reads.py` bytes with its
real `__file__`; it must occur exactly once as matching `frozen-evidence`.
Its relative dependency tree must remain intact:

| Frozen dependency | SHA256 |
| --- | --- |
| `accounting_untracked_reads.py` | `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a` |
| `analyze.py` (selected pure definitions only) | `2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78` |
| `audit_compaction.py` | `dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153` |
| sibling measurement-gate `batch_analysis.py` | `dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3` |
| `runner_automatic_refresh.py` | `eeba247844dfa073754b01060d50de7898728d422aab27ff69af42d97b8b15fe` |
| `runner_work_units.py` | `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e` |

`audit_run` requires actual report/`observation/analysis.json` ledger agreement,
matching observer/native metadata scope, native source SHA, explicit raw/native
response identities and four-counter agreement, captured compaction lifecycle,
capture provenance and strict tail/error predicates. Its `status=validated` can
coexist with unbounded measurement; it explicitly does not prove all hidden
provider calls or all accepted-input membership. Usage-less scope or unrelated
observer errors can fail the original helper even if separate delivery proof
passes. Preserve that original result before considering any fixed, reviewed
derivative; do not fabricate controller rows, suppress flags or mutate inputs.

Retain the full canonical result/hash before projection, complete response-ID
ownership and source references, warnings, gaps and exact/bounded/unbounded/
unknown status. Preflight create-new output paths and publication readiness;
do not turn a publication failure into an unrecorded retry. No W accounting
calls are needed. Any eventual comparison includes all three outcomes and all
six saved baselines, supported null endpoints intact: no missing-as-zero,
midpoint, percentages, additive shares or failure-as-equivalent-work savings.

## Available saved W identity supplement

The frozen W receipts contain response-evidence count/hash projections, not
the complete maps. Existing full maps are already retained at
`S/phases/untracked-reads-01/postcapture/retained-baselines/<W-ID>/PROVIDER-LEDGER-QUALIFICATION.json`.
Metadata-only check `bebb1b` matched every map's canonical count/SHA to the exact
frozen receipt above, and its ownership to the saved original ownership map.
No counters were totaled, helper rerun or native log reopened.

| W suffix | Exact ID count | Qualification receipt SHA256 |
| --- | ---: | --- |
| 002 | 172 | `1b9e5aa7190c50f73aefdd7d9159f489e040f547e3ae80403d42e03e99ed3a29` |
| 004 | 189 | `87d03b7fb83087a4971033ff0fb667ce1d0a226b1c8ff4edc60277df13c5e540` |
| 006 | 196 | `460df4f2790ce6cf663c83fc14f1e5eb762b284da2d6799b6f1eb923380d3b99` |
| 009 | 201 | `4ef71a48c84bab0f1b7876ccbfcb069ed9e01a875b6887f4836c0a3bad14d67e` |
| 010 | 191 | `8812bc40b1607d7d2c08e004ee1c34a317b6614e0eb9de25974a5eb892f6c258` |
| 012 | 215 | `e6e809c4974e56e78f610f72d84b07406116f83581954e1daf352ad0ab8a1dc2` |

The six maps contain 1,164 distinct IDs. Receipt hashes match
`RETAINED-READ-PROVIDER-LEDGER-RESULT.json`
(`bca1387ce702c98e41d1b24487a062886e955b845974058aa9991ffce780022a`),
under saved scope `bc92a08e47037198e1d0cbc3ca9fe213cfe49b17de6b397bf1d6dcba8d685f31`.
Original ownership file `phases/untracked-reads-01/postcapture/COMMON-ACCOUNTING-RESPONSE-OWNERSHIP.json`
has SHA256 `f5e6be5993f801193edf6e2110eabdd3f013eac640881c33b4e2e8775d83e984`.
Pin these supplemental identity files prospectively, retaining the original six
measurement receipts unchanged. Compare their ID sets against all three future
returned maps; an aggregate digest alone cannot establish cross-set disjointness.
