# Read-mechanism census validation

The census is a provider-free, complete descriptive derivation. It preserves every frozen workflow
and physical read boundary, exact accepted request/native-item/response identities, unknown accounting,
and public tool metadata. It computes no condition contrast, cost ranking, primary endpoint or causal
share. Runtime, frozen predecessor helpers and earlier reports are outside this patch.

## Source identities

| Source | SHA-256 |
| --- | --- |
| `audit_read_mechanism.py` | `2ad81ed5dc27ccba621f21f087312f48ac26e7799073baa5da6130eb2f7119e9` |
| `test_audit_read_mechanism.py` | `61bd3c59b7ea8d09de5d7e78a92ba557984e4e0ac815c2b6ff341a1725c42127` |
| `DESIGN-READ-MECHANISM-CENSUS.md` | `c7c99b96afdd5fb3de613c62be742df5d9fe860c6bf7a3852b79fe668d70940e` |
| `preflight/replay_read_mechanism_diagnostics.py` | `2b74fde7960562591750fabca36ce930797f6638055f270632f2bd7e50f40969` |

## Automatic checks

From the study directory, both commands pass:

```sh
python -m unittest test_audit_read_mechanism test_analyze_untracked_reads test_audit_input_attribution test_accounting_untracked_reads -q
python -m py_compile audit_read_mechanism.py test_audit_read_mechanism.py preflight/replay_read_mechanism_diagnostics.py
```

The combined suite has 107 tests, including 31 new census tests. Initial feature tests failed before
implementation. Subsequent observed RED regressions cover body leakage through unknown nested fields,
read loss on global trace rejection, explicit native-turn identity, one-item ownership ambiguity,
shared repeated-user/issued-path groups, invalid recorded/frozen SHA values, and the real launched-row
receipt interface. In particular, per-run exit receipts are independently hashed at their canonical
phase-local path: the primary report does not inventory them. Run ID, exact status, start/finish times
and integer exit code must agree with the retained row; failed workflows are not zero-filled.

A RED delegation test requires the phase wrapper to use `replay_launched_sources`. A separate RED
test constructs timestamp-compatible stale bytecode; the diagnostic reproducer executes the captured
source bytes and binds their hash before replay. The earlier replay report is retained. Independent
inspection found its cached code equal to current source, not an actual stale-code execution.

## Actual closed-source replay

`READ-MECHANISM-DIAGNOSTIC-REPLAY-v2.json`, SHA-256
`faa20da012c3f63c6587d0a621d3db4ccd472f131ad86a4a8819fe9ad849ec65`, preserves 47 endpoint-hashed
sources and both admitted corrected smoke captures. The same launched-source branch as the phase
wrapper performs real file loading, native-metadata hashing, frozen capture-provenance replay,
future-only accounting replay and input-attribution joins; none of these functions is mocked.
Both captures pass, with four exact native input identities, six complete response records, all
signed attribution residuals zero, and no tool calls. Full whole-item charge references are retained.

Diagnostic closure is proved by the retained admission, successful bounded test logs and actual
invocation start/end receipts. These two cases are not a fabricated twelve-workflow phase. The full
phase admission/terminal-row wrapper is covered separately by realistic synthetic tests, including
the launched-source path; its immutable dependency functions are mocked only in those unit fixtures.
The real fixture prohibits bundle opening, so this replay proves delivery/accounting plumbing, not
natural retrieval behavior. The original diagnostic projection's limitations remain applicable.

Reproduction uses the retained script with `--admission preflight/READ-INLINE-PRIMARY-DIAGNOSTIC-ADMISSION.json`,
`--review preflight/READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.json`, `--sessions-root /home/user/.codex/sessions`
and a new absolute `--output` filename in `preflight`. Admission and review arguments must resolve to
the absolute paths recorded in their source inventory. Existing outputs are never overwritten.

## Phase freeze and invocation

The phase's frozen evidence tree must contain the census helper, its tests and design, plus the
existing analyzer/accounting dependency tree. In particular `audit_input_attribution.py` must be
included with SHA-256 `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740`;
`analyze_untracked_reads.py` remains pinned at
`30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e`. The helper itself and these two
direct dependencies require exact executing-path `frozen-evidence` entries. The unchanged accounting
helper and its own pinned predecessors remain part of the existing primary evidence tree.

Execute the frozen copy of `audit_read_mechanism.py`, not the working-tree copy, after the complete
phase result and final source-pinned primary report are published:

```text
python <phase>/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z/audit_read_mechanism.py --manifest <phase>/score-manifest.json --analysis <absolute-final-primary-report> --sessions-root /home/user/.codex/sessions --output <phase>/READ-MECHANISM-CENSUS.json
```

Every path above is absolute. This invocation reads the report's source inventory/accounting rows,
not its primary result. The output must be canonical, phase-local and create-new.

## Scope and complexity

No public API, agent-facing workflow or architectural ownership changes. The docs catalog
(`architecture.md`, `benchmark-operator-policy.md`, `orchestrator-file-workflow.md`) requires no
additional runtime documentation for this private offline helper; root owns prospective protocol
and operator references. No new real-agent generation is required or performed by this patch.

Dictionary indexes and shared ambiguity tables avoid per-read whole-stream scans or repeated owner
lists. Path scanning is linear in argument bytes plus emitted matches, with cached UTF-8 lengths;
serializing matches also costs the bytes actually emitted. Source verification performs bounded
whole-file passes, and stable output ordering uses sorting. Explicit path substrings are retrieval
candidates only: they may occur inside longer paths or non-reading commands, and indirect retrieval
may have no explicit match. Whole-item token charges are never assigned to individual file spans.
