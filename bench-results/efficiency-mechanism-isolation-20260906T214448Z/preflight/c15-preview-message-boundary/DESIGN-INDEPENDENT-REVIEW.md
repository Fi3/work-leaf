# Independent typed-message boundary design review

Scope: complete [DESIGN.md](DESIGN.md), SHA-256
`597f6f137957de8e924ace7642a3f3d8219b4ff9e27a4397722a11967281cee2`, the retained finite
PHASE-EXPOSURE report, and the affected existing provider/launch/send/handler/registry interfaces.
This is a prospective source review, not implementation qualification or provider admission.
No code, Cargo, provider, benchmark, extraction or accounting operation was performed.

The design is feasible without a public API change. The smallest supported route is a private
call-local completion receipt, followed by a private result wrapper and `AgentFollowUp` payload.
No global pending-text registry or mutable public transcript is necessary. `AgentStreamEvent`
must remain unchanged: its generic `AgentMessage` is not completion proof, and Claude's current
callback supplies deltas. Codex's actual `item/completed` branch can supply a private typed source
before interruption; unsupported sources cannot manufacture proof from a standalone-looking return.

## Ownership obligations before source qualification

1. **Atomic generation and call consumption.** Current `Registry::reserve` derives the generation
   from the current owner under its mutex. A separate earlier `owner()`/generation check followed
   by ordinary `reserve()` permits a completed relaunch between checks. The new reservation must
   compare the expected registry, agent, launch generation and call capability in the same critical
   section that reserves the preview and checks cancellation. Provisional launch proof must refer
   to `LaunchGuard`'s ticket generation and cannot dispatch until that exact launch commits.
2. **By-value is not automatically consume-once.** Existing `AgentFollowUp` and `DirectiveRun`
   derive `Clone`. A cloneable proof needs one shared, sealed consumed capability, or the private
   carrier must be non-cloneable. Checking consumption alongside reservation avoids accepting a
   copied call receipt. This does not remove existing explicit proposal/result replay semantics;
   a newly delivered repeated proposal still needs its own completed-call proof.
3. **Proof scope is the consumed call, not unseen provider output.** The current Codex branch
   requests interrupt, unregisters the turn and returns its consumed message join. Additional raw
   notifications may remain observer-only. Reject later consumed callbacks before return, including
   a provider ignoring the stop request; do not claim that this proves absence of all later raw
   output. Preserve the existing timing rather than introducing a drain. The future observer must
   bind the same selected terminal boundary and retain subsequent tails separately.

The implementer explicitly accepted obligations 1 and 2 before implementation: expected generation
and a unique shared call capability are checked in the reservation mutex, without a global text
store. These are review constraints, not claims that unimplemented code already satisfies them.

## Non-target and source boundaries

The selector must distinguish ordinary no-preview replies from a selected preview whose proof was
invalidated. Only the latter fails proof finalization; unrelated ordinary replies retain the existing
handler. Under explicit opt-in, missing/unknown/contradictory phase, delta-only callbacks or missing
completion proof cannot fall back through the old aggregate-preview shortcut. A prior ordinary stop
must prevent preview selection without changing ordinary parsing or its precedence. A malformed
whole final message remains malformed; no substring, fence removal or selected-fragment joining is
authorized.

Exact complete-return equality and UTF-8 ranges bind dispatch to the unchanged public reply.
The session message, displayed transcript, review-source history, prompt bytes, ACK, shared apply
gate and private executor inputs remain unchanged. Every private send path, including correction,
read/refresh, ACK, command feedback, routed sends and review-fix, must carry the **target** agent's
registry/generation rather than the initiating author's identity. Other roles remain un-enrolled.
Scoped TLS is only an acquisition bridge to a live synchronous call; it cannot be the final proof
store, survive guard drop, or recover a late/crossed call by matching text or agent name.

The default/legacy path must perform no recorder allocation or counter advancement when the new
object is absent. Parser bytes remain the exact `1583e715…` slice. The selector has one separately
frozen pure source, shared with the future observer; the strict-aggregate observer `e72a21b` cannot
be represented as supporting this expanded boundary. Any final whole-source hash changes require
explicit new source/build pins, not merely manifest assertions.

Incremental completed-item metadata and one final byte comparison can be linear in item count and
bytes. Do not rescan all prior text for every callback or scan registries by content. The old
cumulative ordinary detector is separately retained; no linearity claim covers that existing scan.

Current inspected source hashes match the design's agent-runtime, Codex, Claude, CLI and private
parser identities (`e9dc43`). The current C08 orchestrator is
`e19e477a4d9eccbdf479c75589da35dda623a6236c7fdd6194748342841015fe`, not the historical frozen
`e0ef8167…`; the relevant call boundaries were read directly. Current architecture is
`ee443aeead88d6b70125eccdc3c3a7ce9268f3e33dd59876eaa242a692862670`.
Implementation is confined to `/home/user/.codex/tmp/c15-message-boundary.jKHKEJ`; resulting
architecture documentation, RED-first actual completion-source tests, legacy identity coverage,
independent code review and separately authorized real-agent verification remain required. No new
diagnostic or three-workflow batch is admitted by this review.
