# Read-representation isolation review

This is a prospective source-design review, not an experiment admission or a result. It uses the current renderer, private benchmark extension, architecture, operator policy, and the two frozen study protocols. It does not alter those sources or protocols, inspect additional outcome contrasts, or authorize provider calls.

## Recommendation and estimand

The narrowest feasible factor is **eager inline delivery of bundle-eligible, currently untracked project-file snapshots**, versus the normal compact bundle manifest. Retain the ordinary bundle creation in both arms. Do not change repeated-read handling in this experiment.

“Untracked” is the existing `FileReadTracker` state for the particular agent and normalized path, not “the first chronological read.” Successful own patches and edits clear tracked paths (`src/orchestrator.rs`, successful directive branches around lines 797 and 869), so a later read can qualify again. Adding once-ever state would be another mechanism.

This factor contrasts immediate full context with deferred bundle retrieval. It is not merely an alternative serialization of identical delivered information: normal agents can retrieve all, some, or none of a bundle, and can incur additional tool responses doing so. Those downstream choices are intended consequences, not variables to force equal. Inline delivery may eliminate retrieval turns while enlarging retained context; the net direction is not guaranteed. A causal result for this factor would not, by itself, assign any fraction of the historical WL-versus-sequential difference.

The parent reports that a separate exposure inventory found first-bundle manifests in all 12 work-unit workflows and 18 historical WL bundle manifests (not 18 historical workflows), while an inline-threshold-limited repeated-read factor would have no exposure in those observations. That is an exposure-feasibility observation, not a treatment-effect estimate; this review does not independently retotal it.

| Candidate | Actual source seam | Isolation assessment |
| --- | --- | --- |
| Successful untracked bundle manifest → full inline snapshot component | `render_file_read_response` → `send_file_read_response` | Recommended. Existing small untracked reads already use the inline formatter; launch/read policy can remain identical. |
| Repeated unchanged/diff response → full current text | `split_repeated_file_reads`, `render_file_read_response_with_repeats`, `render_changed_repeat_read_snapshot` | A different factor. Current launch and response guidance explicitly says repeated full text is not resent; leaving that guidance contradicts treatment, while changing it introduces a policy component. |
| Direct filesystem permission or `read --force` | `PromptPolicy::for_read_permission`; `split_repeated_file_reads(..., _force)` | Not a representation switch. Direct permission alters available read behavior; the accepted force flag does not bypass tracked-snapshot diff/unchanged handling. |

## Existing call chain and boundaries

`handle_agent_directives_streaming` coalesces consecutive read directives, including the force flag, into one `send_file_read_response` call (`src/orchestrator.rs:738`). That function:

1. Calls `read_requested_files` once. Owned bundle-file reads are handled separately; normalized project paths are deduplicated and read under existing locks (`:1556`).
2. Splits successful project snapshots into untracked, changed, and unchanged groups using existing tracker state (`:1068`).
3. Renders the untracked component, then appends repeated-read and failure sections (`:2051`, `:2076`). Bundling occurs when total untracked text exceeds 24 KiB or an individual untracked text exceeds 16 KiB, using UTF-8 byte lengths and strict greater-than comparisons (`:2172`).
4. If explicit owned bundle files were requested, prefixes their existing inline response and a separator before the project response (`:1032`).
5. Sends through `send_agent_streaming_interruptible`, then records successful project snapshots and existing follow-up/read events (`:1046`).

`ContextBundleStore::write` allocates its ordinary counter, creates the directory, formats the complete bundle, and attempts one write (`:271`). A failed write returns `None`; the renderer falls back to ordinary inline text. A threshold match therefore does not prove that the delivered baseline was a bundle manifest. Bundle lifetime and cleanup belong to this store. Automatic patch-conflict refreshes use a separate path (`patch_conflict_refresh_response`, `render_file_refresh_response`); they are outside this factor.

`src/bench_experiment.rs` is private and feature-gated by `bench-experiments` (`src/lib.rs:3`, `Cargo.toml:12`). Its admitted schemas are currently v1 and v2; `forward_continuation`, `forward_policy`, and `render_spans` handle static renderer-owned policy/ACK/command spans. There is no existing dynamic read-payload condition or v3 schema. A future v3 adapter must be explicit rather than masquerading as a v1/v2 cue replacement. The public provider/controller surfaces and ownership described by `docs/architecture.md` need no change.

## Smallest proposed v3 transformation

The implementation can remain private to the existing renderer and benchmark extension:

1. Execute the ordinary baseline untracked rendering exactly once in both active arms, including the ordinary threshold decision, bundle counter allocation, write attempt, successful manifest, or inline fallback. Record the exact untracked-component byte interval structurally while rendering, plus whether a bundle was actually written successfully. Do not rediscover the interval by searching prompt contents or bundle markers.
2. Build the inline alternative from the already-held `exact_snapshots`, using `render_file_read_response_inline(exact_snapshots, &[])`. No project or bundle reread is necessary. Both active arms perform this candidate construction.
3. Append the normal repeated-read and failure suffix exactly once. Preserve that same suffix in both candidates; do not invoke diff rendering twice. Diff rendering can create temporary files and run `git diff`, so repeated full-response rendering would not be neutral instrumentation.
4. At final composition, shift the owned interval by the exact existing explicit-bundle prefix length, if any. Replace only the baseline untracked component with the inline component. The resulting complete candidate must equal `prefix + inline_component + suffix` byte-for-byte. Explicit bundle content, changed/unchanged sections, unavailable-file diagnostics, their order, and separators remain identical.
5. A successful ordinary bundle write makes the boundary eligible. Control selects the baseline candidate; treatment selects the inline candidate. Ineligible reads, pure explicit-bundle reads, small untracked reads, and bundle-write failures retain the ordinary prompt, with explicit eligibility reasons. They are still recorded, not silently omitted.
6. Write evidence before sending. An evidence write failure is an explicit pre-send failure; do not send an unrecorded transformed prompt or emit extra model-visible messages. Continue through the same send function, interruption handling, tracker update point, follow-up collection, and read-event paths.

No changes are required to read coalescing, path normalization/order, locks, `_force`, thresholds, tracker clearing/recording, conflict refreshes, command execution, allocator/randomization, ACK cardinality, ownership/review routing, launch prompts, or completion policy. Default/inactive behavior and existing v1/v2 schema, evidence, and delivered bytes must remain unchanged. The v3 non-read policy/ACK/command sites should retain baseline bytes and auditable identity exposure records.

### Symmetric candidate evidence

Every active v3 read event should serialize **both complete candidates**, in the same field order for both arms: `baseline_prompt` and `inline_candidate_prompt`. Store `selected_candidate` and its UTF-8 length rather than serializing a third forwarded copy. The actual provider request must equal the selected saved candidate. For ineligible reads, both saved candidates can be identical and selection is baseline with the explicit reason. SHA-256 identities can be calculated offline from these exact saved strings; no runtime cryptographic dependency is necessary.

This keeps candidate construction and trace volume structurally symmetric at a given boundary; it avoids writing a second large body only in treatment. It does not guarantee equal wall time or total workflow evidence volume: selected payload size and subsequent agent behavior are the intervention. Preserve timing *policy*—no new waits, grace periods, forced retrievals, locks, or interruption rules—and do not claim identical execution timing.

## Minimal evidence contract and joins

Keep a single indexed evidence chain rather than another whole-workflow accounting framework. Proposed v3 read records need:

- The existing run/process/sequence/site/agent identity and a distinct read-response site; requested normalized paths and snapshot classes, including explicit bundle reads and failures. No provider messages carry this metadata.
- For each held project snapshot: normalized path, class, full UTF-8 byte length and existing FNV64 digest. Renderer-owned body ranges for the untracked snapshots actually present in the inline candidate let offline analysis hash their exact held text without parsing file contents for marker strings; tracked snapshots need no additional full-text projection. Record bundle-threshold eligibility separately from actual successful bundle creation, together with the successful bundle path.
- The structurally owned component interval, component lengths, both complete candidates, selection and eligibility reason. All offsets are UTF-8 byte offsets with boundary checks. Offline analysis computes SHA-256 from the exact retained candidate/component/snapshot bytes and source artifacts. The normal FNV64 label is not a cryptographic hash, and neither hash type is a token count.
- Source/manifest identities sufficient to pin the renderer, extension, condition, and observation configuration. Do not include credentials or unrelated environment contents.

Do not introduce a runtime SHA dependency, custom cryptographic implementation, or filesystem read-back to label the bundle. An archived bundle can be hashed offline when it exists. Reconstructing expected bundle bytes from source-pinned formatting and retained snapshots gives a reconstruction identity, not an independently archived-file identity or proof that a later external bundle read saw immutable bytes. Missing archive coverage stays explicit. Subsequent actual bundle retrievals require their own captured input/output evidence.

Evidence joins must distinguish a prepared prompt from an accepted delivery:

`renderer event → exact captured turn/start request → valid success reply / thread+turn → completed response ID → provider input-attribution items`.

Use typed, nonempty exact identities and physical source locators/hashes. Reject ambiguous duplicate/conflicting IDs and mixed success/error replies; preserve rejected or unverifiable requests. Require reverse coverage of actual supported read boundaries as well as trace-to-request coverage. A trace row alone is not an exposure. Reuse the current exact source/provenance rules, not a fake conversion of v3 records to a frozen older schema.

Native input enrichment is an exact `(thread_id, item_id)` join only. Some input-attribution identities may remain unlinked to native content, and an accepted turn need not give a uniquely attributable input item without that identity evidence. Keep unlinked charges explicit. Report exact eligible exposures, direct selected read-input charges where identifiable, actual bundle-retrieval actions/results, and repeated charges of the same identified input items across completed responses. Report missing response tails and native compaction coverage separately under the existing accounting rules. Do not substitute prompt bytes for tokens, infer item identity from text/prefixes, or assign all subsequent response changes to one read event.

## Resource failures and interpretation

The ordinary bundling path puts no new hard ceiling on full untracked file content. Eager inline delivery can encounter larger request/context limits or different compaction behavior. These are retained treatment outcomes, not reasons to clip content, fall back to a bundle, replace a workflow, or remove it from a denominator. Any operational resource/admission ceiling must be specified prospectively and applied consistently, with its restriction on the estimand explicit. A post-exposure failure remains observed even if accounting is only bounded or unknown.

The trace itself contains both full candidates and therefore needs ordinary source-artifact protection and an explicitly adequate prospective resource budget. Resource limits must fail visibly; they must not silently truncate prompt evidence. The implementation should concatenate/hash each candidate in a linear pass over its bytes and snapshot metadata, retaining existing path-set costs. Repeated whole-string rewrites or repeated event rescans would introduce avoidable O(n²) work and are review findings.

The next protocol must independently fix its sample, allocation, endpoint, and claim rules before launch. Exposure feasibility is not evidence of significance. Identified extra retained-input charges can establish concrete work/charge pathways without establishing a positive whole-workflow net token effect when other responses or missing tails counterbalance them. Neither this source review nor the previous failed work-unit primary test establishes the cause of the historical approximately 50% reduction.

## Required implementation verification

Use new tests with a demonstrated RED state before implementation; do not silently alter existing committed tests. Generic fixtures should cover:

1. Default/inactive/v1/v2 identity; active v3 control exact baseline bytes; both active arms retain the ordinary bundle creation, counter, path ownership and cleanup. Trace fields contain both candidates symmetrically.
2. Exact 16/24-KiB boundaries and one byte over, UTF-8 multibyte text, multiple/deduplicated/coalesced files, path ordering and one follow-up. Do not recognize benchmark filenames or model names.
3. A large untracked group plus changed, unchanged, missing and explicit-bundle content in one request. Only the owned untracked component changes; repeated/failure suffix and explicit-bundle prefix remain byte-identical. File contents containing renderer markers cannot redirect the span.
4. Unchanged/changed/force repeated reads retain baseline behavior; tracker absence is per agent; accepted own edits clear the same tracker paths and can make later reads eligible. Automatic conflict refreshes remain unchanged.
5. Bundle-write failure yields the same inline fallback and one ordinary allocation attempt; no added project/bundle reads or duplicate diff rendering. Send/trace failures retain failure outcomes and do not advance tracker state through a new path.
6. Exact candidate reconstruction and accepted typed request/reply evidence; omitted/truncated/unmatched events, bool/numeric IDs, duplicate/conflicting identities and unsupported request shapes cannot count as verified delivery. Large-request failure must not cause silent clipping or replacement.
7. A bounded real subscription-backed smoke in each active arm, through the actual WL read handoff, proving selected delivery, unchanged other boundaries and clean capture closure. Keep it separate from observations and do not turn forced smoke follow-ups into benchmark instructions. Run the repository's required format, all-target/all-feature clippy and test checks.

The source review itself changes no agent-facing behavior and requires no real provider verification. A future implementation does, and must update architecture/operator-facing documentation only where the resulting benchmark extension contract actually requires it.

## Review of the proposed implementation design

`DESIGN-UNTRACKED-READ-INLINE.md` is consistent with the recommended seam: actual successful bundle writes determine eligibility, the baseline executes once, both complete candidates are recorded symmetrically, the renderer owns the interval, legacy v1/v2 behavior remains intact, and large-request failures are retained. There is no blocking factor-isolation correction to that design. The evidence implementation should preserve the distinction between normal FNV64 labels and offline SHA-256 identities and explicitly retain unavailable native-input joins rather than assuming every accepted prompt can be attached to an exact charged item. Renderer-owned snapshot-body ranges are the small addition that makes per-file offline identity verification unambiguous without runtime hashing or marker parsing.

## Source identities

Inspected source identity: `e87c281a6a41def921b04412cfe304453a958de0`. The following file hashes were rechecked at the end of source inspection:

| Source | SHA-256 |
| --- | --- |
| `src/orchestrator.rs` | `11b1e131b8d6e6f50100575c4d43c5e245243fe501f9280b8f899f197f947ae0` |
| `src/agent.rs` | `a6bd797e780b1751026c141cfff508c036ea45af7f37b085336f8a34c8a85838` |
| `src/bench_experiment.rs` | `a06ee31112e7b4d8bb13f1a4a0e58063629c7a36caba6be9598e1ebc832a16eb` |
| `docs/architecture.md` | `22ff0deb0af1468da9374dc7750201357701f2d3d9bbf21730074c945746531d` |
| `docs/benchmark-operator-policy.md` | `77980b19ffca70cb880f6e9d871a25c7ef8253423a197c8373677e4ddf44e807` |
| Study `PROTOCOL.md` | `906762d9b036ccf196e87f91bc7800004b13f60d28144a789b3ccaba708999ae` |
| Study `PROTOCOL-WORK-UNITS.md` | `8d7f29516d37a1e050e539d7098249a2550d61b7eb3b9c30c1f18f82086b9964` |
| Reviewed `DESIGN-UNTRACKED-READ-INLINE.md` | `4cdc6472f7edffa6b750d56255aa97dadb7b1967c186c88bc12393cbc1dc8833` |
