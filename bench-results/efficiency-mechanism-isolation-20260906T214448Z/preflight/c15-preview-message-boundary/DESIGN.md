# Private preview ownership at genuine message boundaries

Status: source design only. No implementation, provider call, test execution, runtime mutation or
admission belongs to this design. The observer-only commit `e72a21b565f807c4547b972bea1b323303d8e355`
in `/tmp/c15-observer-preview.0Wb07l` remains a separate, strict-aggregate opt-in; it does not fix
the runtime prefix problem.

## Evidence and constraint

The complete finite source is `phases/test-first-01/PHASE-EXPOSURE.md` (SHA-256
`5b056b8d176a7beb6796eb8ed58cf553681ed127921d903fb0ac87b6dc6dadd3`) and its JSON
`215eb22a8daaf7fe7743e6e9ca133a593efa3a3b71bde10d13e388cf54148b09`. All nine first preview
items follow separately completed commentary in the same turn. Their physical preview lines are
001: S6282/S4954/S5014; 002: S4112/S2287/S6199; 003: S6955/S2749/S6354. The existing finite
index retains exact preceding items, accepted inputs and subsequent corrections; it is reused,
not re-extracted. Its 24 preview items include ten prefixed aggregates, eight newline headers,
five standalone valid envelopes and one empty-lock declaration. These are different failure classes.

A bounded additional projection of only the nine indexed first previews and their 38 indexed
preceding items finds an explicit public phase boundary: every preview has `phase=final_answer`,
every preceding item has `phase=commentary`. Tool `2111ac` verifies each exact indexed item ID and
full-text SHA before reading its phase; prefix counts by author are 001: 1/4/12, 002: 2/4/4,
003: 1/4/6. No text, reasoning, cost, usage or wider population is exported. This supports a smaller
typed-phase design than guessing whether arbitrary preceding prose is benign.

Frozen `codex.rs` SHA `334eb3008f3b0a8db0d98bed2d3191f5cbf37052fb342dcf26a91ccd4ee502d8`
emits each completed native `agentMessage` at 275–289 and returns their two-newline join.
Frozen `orchestrator.rs` SHA `e0ef8167bd1f51cd99d56ae58835ba5d6951549c9dc9a2ccdabf9b2e568ff0f7`
also joins messages in `DirectiveStreamInterruptDetector::observe` (641), while the final handler
(696) parses the returned aggregate again. `cli.rs` SHA
`98b08ef350616c2f9d33ca0dd6ca6916cf31a0a985e694370b0097acd4e867ad` passes that aggregate through
launch/send and its pending-reply loop (1201). Fixing the detector alone cannot fix final dispatch.

The exact private parser remains unchanged: source SHA
`1700f26d5ff99271ca12acbfa783ff0d5525d3720c9ba47ff088c1aaa9d40d2f`, `parse_preview` at 320.
The frozen negative test at `bench_private_test_first_tests.rs:232` (source SHA
`82bfc571df12402c25e6a152a0fea0e478a8bc3dd3918199f8c4f32c91fd6e37`) rejects copied/fenced,
mixed and duplicate envelopes. Existing private integration fixtures return single complete
`ChatMessage` values; they do not qualify a commentary-item → preview-item boundary.

**A generic stream callback is not a completed-message receipt.** `agent_runtime.rs:239` has
`AgentMessage(String)` but no completion identity. `codex.rs:275` uses completed messages;
`claude.rs::handle_stream_event:643` uses text deltas. Architecture's provider contract explicitly
permits both. Current inspected identities are agent_runtime `cae5903d6b34271613fe2ef3a8033eeda3c294a602f2b4b9206b3649881babb0`
and claude `c5047a3815b438149a096e927b8b7fb218aff0f708fea44f1b096eea3ff3b22b`.
Therefore treating every `AgentMessage` callback as completed would be an unsupported shortcut.

## Smallest safe implementation boundary

Use a benchmark-private, call-local completed-message receipt, not an aggregate substring search,
global pending-text registry, provider-name heuristic or rewritten public session. The core receipt
contract is provider-neutral; a provider must explicitly supply an actual completed-message boundary.
Use private `Commentary` / `Final` / `Unknown` categories derived from explicit provider metadata,
not from wording, filenames, provider names or substrings in the aggregate.

1. The existing private launch helper in `cli.rs:40` and send helper in `orchestrator.rs:661`
   establish an owned synchronous call scope: exact AgentId, existing launch generation and opaque
   call identity. A private scoped recorder can reuse the existing benchmark-only synchronous
   ownership pattern. It must reject crossed/nested/late ownership, not borrow a stale global entry.
   Pass `LaunchGuard`'s generation explicitly for launch; continuation ownership needs the caller's
   existing `Registry` through CLI/`DirectiveServices`. `resume_scope` only serializes AgentId and
   is not itself a generation receipt. Revalidate the captured generation before final dispatch.
2. At the provider's actual completion seam, record the exact complete text and occurrence into that
   scope before testing interruption. For Codex this is the existing `item/completed` branch with
   `turn.agent_id`, native turn/item identity and explicit public phase. Raw/native identities remain
   evidence, not required public DTO fields. Delta callbacks do not create this receipt. Unsupported
   providers retain the old whole-turn path when not opted in. Under the new explicit option,
   absent capability cannot authorize any preview merely from text shape, even a standalone return.
3. The new prefix selector requires exactly one source-attested completed `Final` message after
   only source-attested `Commentary` messages and parses that entire final message with unchanged
   `parse_preview`. Unknown/missing phase cannot authorize prefix acceptance. Preserve ordinary
   directive precedence: if an earlier message already triggered the ordinary stop detector, the
   later preview cannot replace it. A fenced/copied/mixed final message still fails the exact parser;
   final text cannot be reconstructed by trimming quote/fence markers or joining selected fragments.
   Another final/preview message or an operational prelude invalidates the new selection. Explicit
   phases define separate complete messages, rather than a guessed Markdown continuation across
   commentary and final phases. Freeze this precise private receipt contract before implementation.
4. The stop detector may request interruption when that owned complete message qualifies. It must
   continue tracking callbacks for finalization: a backend may ignore the stop request because the
   public trait's default interruptible methods do not enforce stopping. Any later completed agent
   message invalidates the selected preview. Status/usage events do not become message boundaries.
   The new option disables the old aggregate-preview shortcut, which would otherwise let a
   non-final or unknown-phase callback bypass ownership. Ordinary non-preview directive detection
   stays unchanged. Without the option the old v6 shortcut stays unchanged too.
5. On provider return, require exact equality between the retained completed-message aggregate and
   the returned final reply, with one selected complete message and no later/mixed output. Mismatch,
   missing completion proof or ownership conflict cannot be repaired by searching the return text.
   Failed selected-proof finalization must fail closed rather than fall back to an unrelated
   standalone-looking suffix. Standalone non-streaming previews retain their existing path only
   without the new option; the opted-in route requires a completed `Final` receipt too.
6. Carry this proof by value through new crate-private launch/reply wrappers, `AgentFollowUp`
   (`orchestrator.rs:622`, conversion at 1816) and the CLI pending queue to the private handler.
   Keep the public `ChatMessage`, stored `AgentSession`, displayed transcript, source-review history
   and returned full text unchanged. The handler consumes the exact selected range only for private
   preview dispatch; existing registry reservation/revision/cancellation/execution/delivery gates
   remain authoritative. Host offsets/hashes must never become extra model bookkeeping.

No `AgentStreamEvent` variant or public trait/DTO change is necessary for an internal benchmark-only
completion recorder. Adding an enum variant would break downstream exhaustive matches and needs
human API authorization; it is not this proposal. A supported provider's private adapter hook is
necessary—there is no honest completed-message guarantee available from the existing generic event
alone. External providers without that private capability are explicitly unsupported for the new
prefix path. This is narrower than designing a public general-purpose message API.

## Explicit future activation and source binding

Keep schema v6's condition `private-test-first`. A new optional **top-level** private manifest object
`preview_message_boundary` has exactly `mode`, `parser_sha256`, and `selector_sha256` fields;
`mode` must be `completed-final-v1`. Absent means the entire old v6 path, including strict aggregate
parsing, unchanged activation serialization and no completion-recorder allocation. Explicit null,
unknown fields/mode, invalid digests or mismatching source identities fail closed before launch.
Default builds and v1–v5/v7 behavior do not acquire this option. The v6 `Descriptor` and qualified
Python operation/config shapes need not change: the Rust manifest parser owns the optional sibling
field, so no completion metadata leaks into model prompts or executor inputs.

`parser_sha256` names the unchanged exact parser slice `1583e715…`; `selector_sha256` names the
separate private selector source. The complete value must be calculated from final reviewed bytes,
never invented here. The activation source carries the corresponding expected digest outside that
selector file, with an exact-source automatic guard; frozen admission independently checks the
named full source endpoints and executable/build attestation. A manifest assertion alone is not
proof of executing bytes, and mutable source-file reads cannot stand in for compiled provenance.
Activation and per-call host boundary receipts retain this object plus the manifest identity.

The future observer uses a separately admitted value of its existing explicit grammar-SHA opt-in.
Its build extracts the same private typed selector and unchanged parser, bound to exact complete
source/slice digests; live recognition and offline replay both consume typed completed phases and
the same selector. Saved per-invocation settings bind the mode, parser, selector, runtime manifest
and closure streams. The reviewed `e72a21b` mode remains available only for its original strict
aggregate contract. No old invocation is enrolled by the analyzer's environment.

The separately reviewed future derived-output bridge v2 is an independent selected dependency.
Its existing `bridge_path`/`bridge_sha256` and config pins can be chosen prospectively under the
ordinary v6 descriptor; neither this selector nor its tests rewrite original bridge/helper/config
or archived preview artifacts. No silent substitution into an already admitted run is permissible.

## Required RED-first coverage and gates

The first regression must exercise an actual typed completion source, both launch and send:
separate commentary → complete preview, exact returned aggregate, one private dispatch, unchanged
session/transcript and no shared mutation before delivered feedback. A fake callback stream alone
cannot prove provider completion; include the existing local synthetic app-server seam without a
provider call. Then retain negative tests for copied/fenced/indented/quoted/current-message mixed
text, fragments that would need quote/fence stripping, prelude operational directives, two previews,
later messages after ignored stop, wrong agent/generation/call, missing completion receipt,
absent/unknown/contradictory phases, delta-only events, final aggregate mismatch,
replay/cancellation and body bytes containing literal directives in unified-diff context. All default
and v1–v5/v7 plus non-opted-in v6 text/session/interrupt/activation identities stay unchanged;
old frozen v6 artifacts remain immutable.

The future observer must consume the **same newly frozen selector semantics and completion source**
under a separate source-bound choice. Its reviewed strict-aggregate commit cannot provide grace for
this expanded boundary. Do not hand-maintain a second selector or retroactively enroll old captures.
Actual configured-agent qualification must then show the owned multi-item preview, source-exact
grace ordering, one closed private result, normal shared application/check/DONE and review. No run
is authorized here. Incremental prelude inspection plus one final byte comparison can be linear in
completed-message bytes; avoid repeated whole-prefix scans or registry text matching. The unrelated
legacy detector's documented cumulative rescans remain separately qualified.
