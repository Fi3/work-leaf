# Local C08 guard contract

Five automatic guards pass (`7599c5`) under the qualified dependency graph:
owned whole initial read, exact local recovery chain, foreign/duplicate/forged
current inputs, failed/timed-out/missing or unprocessed completion rejection, and
repeated ACK texts joined to distinct ordered call occurrences.
These are local evidence guards, not the runtime's general directive parser and
not native or semantic qualification. The conservative line guard rejects
ambiguous quoted/fenced/indented/mixed contexts rather than granting eligibility.

The initial offline RED command (`97a195`, terminal `dd5cf2`) fails because
`guards.rs` is absent. Without a private-crate lockfile it also selected newer
cached dependencies. That lock is retained as `Cargo.lock.initial-unqualified`,
SHA `081f5c992d4e7c0ec26021383daad8a29a3876aa85f23568281db4743b53c685`.
It is not an admitted or source-identical qualification build.

The active lock uses the entire existing D3 dependency graph plus this diagnostic
package's dependency entry, SHA
`524d78fb7d362ca19a29eb23a2507e43d010177060e4bd6d9b2d78fcf0a77277`.
All subsequent commands use `--locked --offline`. The first pinned build
(`ed3556` / `59bca8`) also exposes a syntax typo in the new, uncommitted test;
after correcting it, the four tests pass. No old test or dependency source was
changed and no provider was invoked by these gates.

Independent review identifies a false rejection when two ordinary repairs yield
identical ACK text. The new two-ACK regression fails before correction (`9b5586`)
and passes after ordered consume-once input matching (`7599c5`). Unmatched repeated
inputs still fail. This fixes only the new local qualification guard, not runtime
ACK wording, evidence or accounting. Final private gates (`9b874c`) pass all seven
tests, with the real-provider test ignored and its synthetic child invoked by the
automatic parents; private Clippy (`07e2b5`) and formatting (`2f002b`) pass.

The local join checks at most eight calls. It verifies every call's owner,
ordinal, actual return, launch status and complete prompt/reply; exactly one
owned read and eligible automatic-refresh input; ordered UTF-8 component ranges,
the complete current body and unchanged non-owned intervals; and later ordinary
ACK, successful focused command and sole final DONE. Actual CommandChat processing,
Git semantics, original/forwarded/native delivery and real-agent completion remain
separate required witnesses.
