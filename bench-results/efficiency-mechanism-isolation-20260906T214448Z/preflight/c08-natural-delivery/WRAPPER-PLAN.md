# Finite C08 closed-source wrapper

Prospective source-only plan. No workflow payload, native log or live artifact
was inspected to prepare it. Implementation/tests, source assembly and actual
execution require their separate reviewed scopes. This is not admission or a
phase result; no accounting function belongs to this wrapper.

## One source-bound operation per closed row

Use one small `audit_automatic_refresh_sources.py` entrypoint:

```text
--input <exact absolute RUN-INPUT.json> --input-sha256 <sha> --output <new result>
```

The fixed postcapture scope contains only
`automatic-refresh-01-workflow-001`, `002`, `003`, all
`automatic-changed-refresh-full`. All three must be terminal before assembly or
execution. The input binds a unique exact phase schedule entry and actual final
score/terminal/process receipt, not a synthesized successful diagnostic. Each
row receives an outcome even when source or delivery gates fail. No baseline
replay, replacement, provider call or automatic repair/retry is part of the unit.

`RUN-INPUT.json` contains these exact references, each `{path,sha256}`:

- fixed scope, phase manifest, final score/terminal and process outcome;
- v7 experiment manifest, trace, actual observer configuration and unchanged
  observer executable; original analyze/extract reports and execution receipts;
- reviewed wrapper/join/event/primitive/source/frame helper identities and the
  frozen C08 source/build/admission closure;
- `process-invocations.jsonl`, plus all invocation start/child/end and applicable
  raw captures; every app-server entry additionally has original/forwarded/server
  raw files, rewrite settings/journal, grace journal and their terminal digests;
- exact native source list keyed by thread ID, explicit mapping provenance and
  any missing/ambiguous thread-source candidates;
- every recorded project-layer inventory boundary and its parent/child timing
  sources, plus original trust/configuration qualifications.

Keep original outputs and flags immutable. A derivative result states its own
source/delivery status without changing original observer, workflow or accounting
status. Preflight unique output/parent/writer readiness before consuming the
fixed scope; preserve an attempt marker and failures if publication fails.

## Reuse without relabeling an old collector

Exact reusable code:

| Source | SHA256 | Allowed use |
| --- | --- | --- |
| `audit_review_evidence_sources.py` | `d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee` | `Sources`, `decode`, natural-workflow `terminal`, `collect_captures` |
| `audit_review_evidence.py` | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` | Supplied exact bytes consumed by reviewed pure join |
| `validate_automatic_refresh.py` | `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f` | Pure event/census validation through reviewed join |
| `preflight/review-evidence-native-diagnostic-001/postcapture/observer-frame-correction/audit_observer_frames.py` | `ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f` | `prove_frames` only, with its exact two dependencies |

Compile captured verified code bytes under their recorded filenames; no cached
bytecode or unpinned normal import. `prove_frames.original_modules()` still uses
the exact retained collector/primitive tree; pin its actual dependency paths as
well as the wrapper's supplied code. Do not relabel v5 `audit_sources`, its build
schema, archive selection, policy ownership or `join_review_inputs` as v7.
The new pure join's final source/test hashes remain a prerequisite, not invented
here. It must include the three pending duplicate-anchor/native-metadata/mixed-
reply regressions from design review.

## Ordered implementation checks

1. **Source admission.** Verify the new input/scope and immutable dependency
   union; bind C08 runtime/source/build identity to the actual phase. Reuse
   `Sources` exact path/type/inode/read/SHA guards and strict duplicate-key,
   nonfinite-JSON and full JSONL-tail checks. Inherited limits are 1GiB/file,
   32GiB accumulated unique source bytes and20,000 sources: explicitly retain or
   prospectively tighten them before execution, never silently clip. These are
   source admission limits, not proof of bounded peak memory.
2. **Terminal population.** Reconcile all actual invocation and app-server
   directory names against the pinned inventory/input before and after reads.
   Preserve every non-app invocation's closure as well as primary app servers.
   `collect_captures` can check its declared app-server rows and raw end digests
   independent of usage, but does not itself census actual directories or native
   source membership. Supplement those finite checks rather than claim it does.
3. **Actual transport proof.** For every app server call unchanged `prove_frames`
   with original/forwarded raw bytes and exact settings/journal; match its decoded
   frame digests to the same complete framed populations supplied to the pure
   join. Require the configured raw/primary/grace1000/forward route, end-bound
   settings/journal/forwarded/grace hashes, and preserve each interrupt's actual
   terminal/grace outcome. No method-name-only normalization, saved PASS flag or
   filtered turn-only stream substitutes for this proof. `prove_frames` proves
   forwarding only; server endings and settings/terminal membership stay wrapper
   checks.
4. **Native membership independent of usage.** Index all typed accepted thread
   starts and accepted turn starts across the closed captures. Include title,
   linearizer, failed/no-usage and otherwise auxiliary threads, even with no v7
   owner anchor. Use existing extraction references first. Missing references
   require a separately recorded exact-thread-ID mapping: bounded filename/header
   resolution under the real sessions root, exact native session identity/cwd,
   unique candidate or explicit ambiguity. Do not select by latest timestamp or
   fabricate an empty native source. Native sources belonging to original
   unexpected-thread diagnostics remain separately unresolved, not erased.
   Freeze this completed source mapping before the wrapper execution; the wrapper
   itself does not recursively discover or analyze other sessions.
5. **Native context and input joins.** Verify exact source SHA and physical
   records, one session identity, recorded CLI/cwd and every explicit turn-context
   model/effort/cwd against the admitted run. All accepted turns need their exact
   native user evidence regardless of usage. Invoke the reviewed pure v7 join on
   the complete supplied populations; restore source-bound physical trace lines
   by row index. Require the pure census itself, not supplied validity flags.
   Preserve failed/unmatched/orphan/order-conflict/unknown-owner records. Native
   tool semantics, successful repair and zero-hidden-work are not input-join
   conclusions and are not silently delegated to the old collector's tool parser.
6. **Closure/publication.** Recheck every consumed source and directory census;
   retain original flags and full safe result before any bounded presentation.
   Freeze an explicit output row/byte limit in the execution scope; oversize is
   an error, not truncation. Publish canonical full-result SHA, sources, terminal
   status, all input/trace associations and separate source/frame/native gates.
   A valid trace with no refresh events is retained without claiming no natural
   conflicts. Numeric accounting remains the separate three-new-row scope in the
   [prospective checklist](../automatic-refresh-screen/POSTCAPTURE-ACCOUNTING-CHECKLIST.md).

## Why original extraction is not the missing-source resolver

`bench-observer/src/lib.rs::extract_rollout_metadata` (source SHA256
`188a1b4fd9913556c51024dcc0068be353813c44c688d6d22929da139f392b5f`)
initializes app-server expectations from `summary.threads` and emits a metadata
row only with final native usage. Its separate native-exec fallback does not turn
every accepted app-server thread into an expectation. Thus its original output
cannot prove coverage of usage-less accepted app-server threads. The v5 source
collector can read such threads when explicitly supplied, but neither discovers
them nor supplies a v7 ownership proof. Preserve original extraction flags;
complete native input membership is a distinct source-bound derivative.

## Finite test gate before real source execution

Add synthetic temporary-source tests only: (a) closed success and nonzero-exit
workflow retained; (b) omitted/extra invocation or directory; (c) broken end/raw/
journal/source hash, truncated final line, duplicate JSON key and source drift;
(d) legitimate metadata rewrite proved from exact raw bytes versus forged saved
proof or non-target mutation; (e) two accepted threads where one has no usage or
extractor metadata, requiring its explicit native source; (f) missing/duplicate/
foreign native source and model/cwd/turn mismatch; (g) malformed trace retains
its physical row; (h) output/attempt preflight prevents reexecution. Use the
reviewed pure synthetic join cases, not a five-call diagnostic-shaped workflow.

This is one wrapper and one fixed three-row scope, not a generic collector or a
new observation. Constant source passes and indexed joins are sufficient; no
per-event whole-capture scan or archive/tool Cartesian product is needed.
