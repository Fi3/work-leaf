# First-pair readiness

The experiment is one direct sequential workflow and one normal concurrent Work Leaf workflow,
using the existing ChatGPT subscription, GPT-5.5, `xhigh`, and Codex CLI 0.153.4. The governing
protocol is `../efficiency-measurement-gate-20260906/PROTOCOL.md`. Collection pauses after this pair.
Raw input plus output is primary; uncached input plus output is secondary. All admitted outcomes
remain in the report. No replacements or product-runtime ablations are part of this batch.

## Source and authentication

`source/` is a clean local clone at `41b6418ef420fbce6aab93706657bd77dba3ce51`. The drivers clone their
fixed candidate base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`. The user's dirty root configuration
and `.gitignore` are not copied into candidate checkouts. Product files in `src/` and production
benchmark drivers have no working-tree modifications. The external observer's source and binary
require their own file hashes because its metadata instrumentation is not in the source commit.

`subscription-codex login status` reports `Logged in using ChatGPT`. The wrapper forces ChatGPT
authentication and the OpenAI provider while removing API-key and endpoint overrides. No credential
material is copied into the batch or scratch directories. Official Codex authentication documentation
informs this route check; no API-key fallback is permitted.

## Verification

The following commands pass in the root workspace before admission:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo fmt --manifest-path bench-observer/Cargo.toml
cargo clippy --manifest-path bench-observer/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path bench-observer/Cargo.toml --all-targets --all-features
cargo build --release --bins
cargo build --manifest-path bench-observer/Cargo.toml
```

The main tests include 51 UI harness checks and four terminal PTY checks. The independent observer
suite has 107 passing tests. The approved report assertion passes its full 24-test mechanism study;
the predecessor report suite passes all four tests. The one-shot launcher and analysis helpers have
separate provider-free regression suites and review before their final manifest freeze.

## Real-agent verification and measurement limit

`../efficiency-measurement-gate-20260906/NORMAL-SMOKE-RESULT.md` records the explicitly enabled real
`observer_subscription_smoke::real_subscription_directive_interrupt_and_raw_follow_up` test.
The unchanged Work Leaf detector interrupts a real directive, and a raw same-session follow-up
completes on the subscription. The two-turn scenario passes in 11.53 seconds with no project writes.
The observer retains its 1,000 ms maximum pre-forward grace and release on resumed output.

The observer executable SHA-256 is
`b0a4f165bf19a994a3b5b01d13e4a19925a91019c73a8f83b25134ee05f62a7b`, matching that verified scenario.
The completed follow-up has exact usage; the interrupted response remains unresolved. This is
integration verification, not proof of exact whole-workflow accounting. The pilot retains missing
responses and uses finite bounds only when both tail cardinality and model limits are supported.
