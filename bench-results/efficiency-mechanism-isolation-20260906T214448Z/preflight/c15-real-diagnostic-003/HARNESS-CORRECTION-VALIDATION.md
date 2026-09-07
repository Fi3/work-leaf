# Complete-author diagnostic qualification

Diagnostic003 uses the normal `CommandChat` round limit and requires exact successful ordinary post-patch validation plus actual author completion before entering review. The private crate alone owns this qualification: Work Leaf runtime, public API, model-visible task/policy, bridge and original001/002 artifacts are unchanged.

## Actual boundary

`guards.rs::diagnostic_chat` is the shared factory used by the real harness and the new automatic runtime test. It calls `CommandChat::new` and retains the 30-second locked-command timeout, without overriding the normal round count. The independent outer eight-call budget, two intended roles, 240-second cancellation watchdog, 300-second outer timeout plus five-second final kill grace, and 30/180-second private limits remain authoritative.

`harness.rs` checks `author_capture` after the author call returns and **before** `Budget::begin_review`. It records the resulting structured `author_qualification` alongside the author transcript. An `Ok` launch transcript, generated check request or subsequent clean review cannot replace this guard.

`guards.rs::author_completion` binds the owned author policy's selected full input to the first accepted thread. Every accepted input has an exact original/forwarded typed request and matching complete public user message with empty text metadata. String IDs, unique request/turn/item identities, ordered occurrence queues and renderer-owned trace sites distinguish delivery from copied text. Before review there must be only that author thread. The latest actual ACK must precede an ordinary `command-result` continuation for the exact focused command, status zero and no timed-out header. A private result, missing trace, stale earlier GREEN, altered command, mixed result/error reply or later ACK cannot qualify.

The successful check's final author response must contain one real unindented top-level DONE, with no other operational directive. Benign prose in the same response or preceding commentary-only message items is allowed. Quoted, fenced, indented-code and patch-contained markers do not qualify. Completed public item IDs are retained separately from native-rollout IDs; this pre-review check does not claim native membership, complete tool accounting or semantic test equivalence.

Capture publication can lag the returned directive. `await_author_completion` polls only existing capture/trace files every 25 ms for at most 15 seconds, bounded by the remaining 240-second diagnostic budget. Complete JSONL prefixes are read; missing/partial required messages remain pending until the deadline. No provider send, usage-grace change or prompt rewrite occurs. Watchdog cancellation is checked before review. Each inspection uses indexed linear passes through frames and bytes; readiness polling repeats those passes within its fixed bound (O(P×B) for P polls of B bytes), not nested per-record scans. This is local observer-readiness overhead, not a hidden model continuation.

## Test-first evidence

`HARNESS-CORRECTION-GATES.json` retains exact local RED/GREEN outputs and final source hashes. The initial author-gate test baseline was an explicit development no-op representing the inherited missing pre-review requirement; it was never admitted or used with a provider.

| Regression | Observed RED | Final scope |
| --- | --- | --- |
| Saved002 exit-zero false positive and missing completion | `d9631b`, 1 pass / 2 failures | Actual saved002 is rejected; valid complete typed/public chain passes |
| Delayed publication of required public DONE | `6169a6`, 4 pass / 1 failure | One bounded source-only readiness wait, no extra provider turn |
| Benign commentary item then final DONE item | `60d450`, failure | Multiple valid generated items retain exact final completion |
| Indented code marker | `bba091`, failure | No false top-level completion |
| Timed-out renderer header with zero exit status | `7a2713`, failure | Timeout cannot masquerade as validation success |
| Actual CommandChat preview→patch→check→DONE flow | `925dc7`, failure with the old cap | `81b87f` passes after factory cap removal |

The automatic runtime test uses the public Codex adapter connected only to an embedded synthetic app-server and a pinned synthetic private bridge. It checks private feedback before shared mutation, then performs a real ordinary Git patch and `sh check.sh`, checks the resulting on-disk marker and command-result trace, and verifies the actual orchestrator processed its Done event. Its retained cap-two RED fixture is `/tmp/c15-diagnostic-sequence-100-1788817629544023443`. This is real local control-flow/shell qualification, **not** another provider attempt or confined private-test result. The real diagnostic retains the unchanged natural task and `cargo test --offline --locked`; no fixture test body is supplied to its model.

The capture guard additionally covers mismatched forwarding/ownership/IDs, duplicate deliveries/items, wrong commands, nonzero outcomes, missing records, stale GREEN followed by a later ACK, quoted/fenced/patch/mixed DONE and a zero readiness budget. The nine inherited guard tests remain byte-identical (`0735f66c…`). All new tests are confined to003; committed old tests are not edited.

## Final private-crate gates

- `cargo fmt --manifest-path .../c15-real-diagnostic-003/Cargo.toml`: `994f79`, exit 0.
- `cargo clippy --offline --locked --manifest-path .../c15-real-diagnostic-003/Cargo.toml --all-targets --all-features -- -D warnings`: `205f27`, exit 0, no warnings.
- `cargo test --offline --locked --manifest-path .../c15-real-diagnostic-003/Cargo.toml --all-targets --all-features`: `2c9c19`, exit 0; 18 passed, 0 failed, 2 ignored across five targets. The two ignored entries are the real provider scenario and the synthetic subprocess entry automatically called by its parent test.

A developmental JSON-macro syntax failure and a collapsed-if Clippy finding are retained in the gate receipt; both were resolved before this final cut. The normal repository source and documentation have no diff. Its prior required runtime gates remain a separate root-owned receipt, not rerun or relabeled by this private-crate task. The documentation catalog/architecture were reviewed; no product documentation/API change is needed for this harness-only qualification. Root owns003 scope, source freeze, executable admission and the single subsequent real run. Automated success does not predeclare that run's outcome or authorize a replacement.

Final key source hashes:

```text
92263d07347251f50b53fcd17c0d58bab0855062ab9acf8537a610c8d0f245f7  guards.rs
651a984518102c3af76aff09d93955d9c44c57c189f5a0aecf0bfdb25ba358ba  harness.rs
4a86008589f8f9fce73caa540f18a42bc46fcbe8c8671a5f2d49651373528754  completion_tests.rs
dafabac998e4976c2e94c9d86e1a00d6459807ea90f0423b59955a9009d2e23c  runtime_tests.rs
```
