# V7 automatic-refresh source and delivery evidence

Status: pure in-memory source-consistency validator with synthetic tests; independent review remains a separate gate. This directory is not an admitted phase input. No live capture access, provider, executor or accounting call is part of this implementation. Closed-workflow source assembly and delivery joins remain a later separately scoped step; neither a new accounting framework nor a frozen helper rewrite is provided.

## Smallest pure boundary

`validate_automatic_refresh.py` exposes this private-directory interface:

```text
validate_event(event, expected_run_id)
    -> {errors, eligible, original_sha256, selected_sha256, snapshots, ...safe facts}
census_trace(rows, expected_run_id, physical_lines=None)
    -> {errors, events:[one safe record per nonactivation physical row],
        complete_delivery_proven:false}
```

The first function validates one v7 automatic-refresh event, not delivery or Git history. The second checks the activation/run/process/sequence census and retains invalid/unknown rows without dropping them or counting them as zero exposure. Neither receives filesystem paths or reads files. It exports no prompt, source, diagnostic or reasoning bodies: only bounded identifiers, physical locators, hashes, byte lengths, recognized classes/statuses and fixed error codes. Bound resource use explicitly before any later external source read; refusal is not clipping or row truncation.

The synthetic literal renderer fixture is independent of the implementation; no production formatter is called to produce expected output. `VALIDATION.md` records the fixture-source correction, initial missing-module RED and bounded synthetic GREEN checks. The implementation covers only the tested pure boundary, not hidden postcapture execution or broad workflow analysis.

## Exact source-owned event contract

`src/orchestrator.rs::handle_agent_directives_streaming` produces an event only from the actual patch/edit conflict branch that obtains a `FileReadResponse`. It retains ordinary `record_snapshots` before `RefreshPrompt::finish` and backend send. `Capture::candidate` consumes construction-owned replacement ranges; `forward_with` publishes the candidate evidence first. No validator claim may move these points or treat trace publication as proof of delivery.

Strictly require schema `work-leaf-bench-experiment-v7`, the sole `automatic-changed-refresh-full` condition, exact run/process identity, positive typed integer sequence/process ID, decimal timestamp string, typed byte counts/booleans, and matching `metadata.kind` with `automatic-edit-refresh` or `automatic-patch-refresh`. Reject boolean-as-integer, floats, missing/unknown component fields and unsupported event shapes.

For eligible events, exactly one `refresh-guidance` and one `refresh-intro` component precede exactly one `current-full-text` component for each eligible snapshot, in the renderer's actual order. Snapshot references must be unique and in range. Body-less components have null body/snapshot coordinates. Validate byte offsets on UTF-8 scalar boundaries; body offsets exclude the formatter's optional terminal newline. One ordered walk compares every unowned interval and the terminal suffix. No search for copied headings, directive markers or diagnostic substrings owns a boundary.

Reconstruct the complete prefix from `metadata.kind/files/diagnostic` and the exact pinned patch/edit format-guidance literals. Validate both coherence-span literals and exact source-owned offsets, then every metadata-owned snapshot section and failure suffix. Diagnostics and file names may contain marker-like text; their known byte lengths and metadata ranges, not a global `find`, determine boundaries. Reconstructing the wrapper is a source-attested consistency check, not independent proof that the conflict occurred; genuine patch rejection/commit semantics require the separately retained workflow evidence.

`metadata.files` retains the request's recorded order. `read_requested_files` normalizes project paths with `FileLockTable::normalize_path`, stores them in a `BTreeSet<PathBuf>`, then records successful snapshots and read failures. Reject duplicate successful normalized snapshot paths and noncanonical successful paths; preserve the actual snapshot order without substituting naive string sorting. Parent/root components are not successful relative paths; CurDir and repeated separators are normalized by the runtime. The input/request list and failure list must not receive invented deduplication or sorting: normalization failures preserve their original path and diagnostics, including repeated failures. Source/canonical-path ambiguity remains an error, not a recognized current-repository filename rule.

## Branches and evidence strength

| Snapshot branch | Required representation and selection | Independently recomputable current body |
| --- | --- | --- |
| changed / available | Nonempty diff length1..49152 inclusive; exact raw diff interval plus only its optional formatter newline is replaced by `current full text:` and the complete held text. Exactly one body component. | Yes: exact candidate body bytes, FNV64+byte count, offline SHA-256. No full-text8192-byte cap. |
| changed / empty | Raw diff length0, empty metadata body interval, renderer newline retained; no body component. | No. |
| changed / omitted | Diff length greater than49152, exact omission notice and count; no metadata body interval/component. | No. |
| changed / unavailable | Null diff byte count/body intervals, exact unavailable notice; no body component. | No. |
| unchanged | Previous/current digest and byte metadata agree; exact unchanged notice; null diff fields/body intervals. | No body is carried. |
| untracked, at most8192 bytes | Null previous metadata, exact ordinary full-file wrapper and terminal-newline handling; no v7 body replacement. | Yes: exact original/current identity section body. |
| untracked, greater than8192 bytes | Null previous metadata and exact ordinary omission/request notice; no replacement. | No. |
| read failure | Exact ordered `Unavailable file text` suffix from recorded path/diagnostic pairs. | No successful snapshot is invented. |

If no snapshot is eligible, `components=[]`, `selected_candidate=baseline`, `eligible=false`, `changed=false` and the entire original/candidate prompt must be identical. Even the two available coherence replacements are discarded by `Capture::candidate` when no body is eligible. A mixed event changes only the eligible sections and two coherence spans; all identity sections and failures survive byte-for-byte.

Validate the exact FNV string grammar, embedded length and companion byte count on every snapshot. Recompute FNV only where carried body bytes exist. A changed branch need not have different FNV values: hashes are not collision-free proof of text inequality. In available events the diff is source-attested, length/range-bound data; do not rerun Git diff or claim the diff alone proves the full previous digest.

Every snapshot reports separate `current_body` and `previous_body` evidence. Carried bodies use `status=verified` with length/SHA; unavailable bodies use `status=not-carried`. Previous full text is not present in these event rows. No current checkout text, guessed historical preimage, empty string or source length substitutes for omitted body bytes. A future exact historical source witness may supplement this result under a separately declared source scope; it is not required to truthfully report the limited event evidence.

## Trace census is not natural conflict completeness

The v7 stream includes activation, automatic-refresh events, and ordinary identity `prompt` events for `patch-applied`/`command-result`. These continuation events retain their existing v2-compatible typed spans and original/forwarded equality. `forward_policy` does **not** emit v7 policy trace rows: it selects only v2/v3/v5; v4/v6 have separate hooks. A policy event must not be fabricated to satisfy a predecessor auditor.

Identity validation requires strict span keys/types, exact source literals, byte coordinates and renderer framing as well as equality. The ACK has a complete fixed suffix whose length determines both owned spans, without searching copied file-list text. Command feedback has a source-consistent header, owned guidance and stdout/stderr framing. Arbitrary multiline command/path/output strings prevent those framing checks from uniquely recovering command, status or output fields; that separate actual-command proof requires retained execution evidence. Fixed separator scanning avoids ambiguous wildcard backtracking. Timeout, cross-agent guard and pending-command text is retained, not interpreted as a validation verdict.

There is exactly one first activation. Every later row remains in the census, even if it is a duplicate activation, malformed row or unsupported event. Run/process and contiguous event-sequence checks do not mistake legitimate blank physical lines for missing events. An invalid physical-line map gives all retained records null physical locators instead of truncating the population. An invalid event has unknown eligibility, not a zero-exposure classification.

An activation-only or zero-automatic-row trace is a trace fact, not proof of zero generated conflicts or complete delivery. Already-applied/no-file/same-snapshot branches can produce ordinary untraced recovery replies; evidence publication or provider send can fail. All prepared-without-request, rejected request, missing reply, malformed event and unowned/ambiguous delivery outcomes remain distinct. A later whole-workflow wrapper must preserve every frozen run, including failed and unexposed rows, with no whole-scope PASS inferred from an empty exposure list.

## Reuse for later closed-source delivery joins

The frozen v4 `audit_candidate_delivery.py` cannot validate v7: its schema, conditions, runtime pins, event set, policy-owner lookup and title assumptions differ. Its `native_user_*` output fields actually denote public app-server user items, not native rollout identities. Reuse is limited to source-pinned primitives/patterns, never relabeling a v7 run as v4:

- `analyze_untracked_reads.py::byte_range/fnv/rpc`: constant-time UTF-8 boundary checks, exact runtime FNV format, typed RPC identity. Its `prompt_inventory` supplies indexed per-owner/hash occurrence queues, exact text collision checks and explicit nondelivery states; its schema/policy-dependent whole function is not an off-the-shelf v7 audit.
- `audit_candidate_delivery.py::public_items` and its capture/terminal source checks: exact physical public user/turn identity, complete original/forwarded/accepted input census, canonical closed primary invocation paths and terminal stream digests. Add strict content metadata types rather than assuming arbitrary public text metadata is valid. Do not import its usage-derived owner scope.
- `audit_review_evidence.py::typed_equal/native_user_index`: exact native session/thread IDs, explicit direct/passthrough turn IDs and full user bytes, including usage-less threads. Its v5-specific `join_review_inputs` cannot simply accept v7. Preserve native context items separately and retain ambiguous/malformed identities; no latest-context turn inference.
- The separately qualified observer-frame `prove_frames` checks only exact journalled initialize/thread-start metadata rewriting. All turn/start bytes still match. It may be reused by exact verified source/dependency identity, not by deleting those frames or trusting a saved `valid` boolean.
- The title correction documents the actual first owned policy launch followed by exact raw title requests. Preserve title/linearizer and every accepted thread, including usage-less threads. Neither a missing public tool item nor absent usage means absent native activity.

V7 ownership has two possible narrow witnesses. A complete trace-selected prompt may uniquely match a typed accepted request across the entire closed capture population, binding its source-attested agent ID without policy parsing. Repeated identical text needs ordered occurrence consumption and exactly one admissible agent/thread assignment; ambiguity remains unresolved. Otherwise require exact complete launch/session/task reconstruction from source-bound saved records. `src/codex.rs::request_turn_streaming` emits the thread status before turn/start, and `WorkLeafSession` preserves id/feature/lines through the frozen driver's saved state files, but those rendered lines are not typed events: arbitrary agent text can imitate `codex: Codex session ...`. A standalone status regex or first copied `Agent-ID:` marker is not ownership proof. `src/agent.rs` appends a known launch footer; only a separately known exact complete footer/input or full unambiguous session/public join can authenticate it.

Build accepted-request and native-user indexes once for **all** app-server captures, not per event or only reported usage threads. Retain exact `(capture, typed RPC, thread, turn, public item, native source/line/item)` links and consume each occurrence once. Verify ordered/complete selected prompt bytes before counting an exposure. Public inputs, native membership, source/config/terminal scope and arithmetic remain separate facts. Safe output holds hashes/locators, never hidden reasoning or duplicated large bodies. No diagnostic five-call/one-agent sequence, forced check/DONE chain, source-path special case or percentage estimator belongs to this validator.

## Source identities and pending gates

The source read cut is `Capture` `dc09943eb17ed9d02bbed4d938908fe5dbbd4fe74aaa8ee8204844f9ae406ec2`, orchestrator `e19e477a4d9eccbdf479c75589da35dda623a6236c7fdd6194748342841015fe`, experiment activation `cb7e1326d1f598a81df6f1544804c6b6285df185aee3021b15a01b6cdf7a3b28`, patch guidance `aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`, and locks `0590b3409c5f8ad9cd44b0320d24b153d4485a26a8c065a877f8742f0b029697`.

Reusable exact sources: v4 auditor `ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1`; v3 primitives `30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e`; review primitives `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257`; frame proof `ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f`; title correction `e8b71f024e8035b5c034e8cf16f092b61fef3944bef93a0f7eddc252c84a6e84`. Their predecessors and original `__file__` dependency layouts must also be pinned before any future compilation; listing a helper hash here is not admission.

The closed C08 diagnostic guard and source/public/native witness were inspected only as prior evidence. Its 57-byte body and five-call chain establish that declared fixture, not all natural branches. Synthetic tests separately cover Unicode/no-final-newline, large eligible bodies, exact49152/8192 limits, mixed empty/unavailable/omitted/unchanged/untracked branches, marker-bearing diagnostics, per-snapshot component ownership, duplicate/noncanonical paths and complete safe invalid-row census.

Pure validation is linear in prompt/metadata bytes plus event counts under ordinary Python hash-table assumptions. Range/type/order checks precede carried-body hashing, so invalid overlapping components cannot repeatedly hash the same large body. Prompts are encoded once; range prechecks do not copy the referenced bytes. Valid body sections are disjoint, and no snapshot loop rehashes a preceding prompt or rescans event history. Working/output space is linear in supplied prompt/metadata sizes; no external-input size or peak-memory admission guarantee is claimed. A full source wrapper, ownership/delivery tests, closed capture scope and independent review are pending and outside this in-memory implementation. Accounting readiness is owned separately; no usage totals, response charges, model prices, contrasts or causal shares appear in this design.
