# C15 diagnostic 003: independent harness review

Recorded 2026-09-07 21:54:50 UTC. **No remaining introduced source blocker found.** Scope is only the new diagnostic crate's cap removal and pre-review success qualification. No WL runtime, existing test, admitted artifact, provider call or admission was changed/performed by this review. The real diagnostic remains ignored until root's separate fixed admission; this is not a successful real-agent result.

## Exact source and independent test result

| File | SHA-256 |
| --- | --- |
| `guards.rs` | `92263d07347251f50b53fcd17c0d58bab0855062ab9acf8537a610c8d0f245f7` |
| `harness.rs` | `651a984518102c3af76aff09d93955d9c44c57c189f5a0aecf0bfdb25ba358ba` |
| `completion_tests.rs` | `4a86008589f8f9fce73caa540f18a42bc46fcbe8c8671a5f2d49651373528754` |
| `runtime_tests.rs` | `dafabac998e4976c2e94c9d86e1a00d6459807ea90f0423b59955a9009d2e23c` |
| `guard_tests.rs` (unchanged predecessor bytes) | `0735f66c0100b1090a251d69902ad9fe36b391093f53fa695770826f4953f011` |
| `Cargo.toml` | `974ccf2dc6b9df6968ec32c9773d1fd5c020178739c3f89e047bf3be96700aaf` |
| `Cargo.lock` (unchanged predecessor bytes) | `f702c77dee349e5ffdf48f8da8bd719bcc7cb48b2e4df93391f89695c1230279` |

Independent command (`c30cba`, exit 0):

```text
cargo test --offline --locked --manifest-path bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic-003/Cargo.toml --all-targets --all-features
```

Result: **18 passed, 0 failed, 2 ignored, five targets**. One ignored entry is the real subscription diagnostic; the other is a synthetic child invoked automatically by its parent test. Final hashes match again after inspection (`8b4874`). Complete source/tests, the harness diff, [SCOPE.md](SCOPE.md) (`1c79a71f…`), and the applicable architecture/operator boundaries were reviewed.

The implementer reports test-first REDs for saved002 false success (`d9631b`), publication lag (`6169a6`), multiple message items (`60d450`), indented marker (`bba091`) and timed-out status0 (`7a2713`). The actual CommandChat path's retained two-round RED is `925dc7`; its uncapped GREEN is `81b87f`. Final private fmt `994f79` and clippy `205f27` are owner-reported green; the independent test execution above is distinct. [ROOT-GATES.json](ROOT-GATES.json), SHA `08b4bb414bb1509f1874ce3d7004442d5d69d1b351371989dc71fa22d1f0dd28`, separately records the unchanged runtime's required repository gates. This review does not relabel those runtime gates as the standalone diagnostic crate's tests.

## Reviewed behavior

`diagnostic_chat` preserves `CommandChat::new` and the 30-second locked-command timeout, without the fixture's `with_max_review_rounds(2)`. The ordinary default is **80,000,000**, shared by author orchestration and review; there is no WL runtime setting change. Existing `Budget`, observer/credential guards, natural request, initial source constants, and terminal-settling logic are unchanged. The eight-call ceiling, two intended roles, 240-second cancellation watchdog and external 300-second/five-second bound remain authoritative.

Before `Budget::begin_review` or `handle_line("review")`, `author_capture` supplies exact source trace and public frames to `author_completion`. A mixed `CommandChatResult` transcript or clean-review Boolean cannot authorize this transition. The guard establishes one author thread through the explicitly owned policy and its accepted first input, consumes typed request/public-user identities once, checks exact forwarded input equality, and maps repeated prompt texts by ordered occurrence queues. It requires the private-delivered event, an actual source-owned ordinary ACK, and then a later source-owned `command-result` selected input for exactly `cargo test --offline --locked` with status0 and no renderer timeout header. A later ACK or failed/wrong command invalidates the earlier success. Source `orchestrator.rs::run_command_for_agent` renders/emits that result only after the ordinary execution path returns; the guard is not accepting a model-authored result assertion.

The qualifying check is the final author input. Its final completed public agent-message item must contain a genuine unindented DONE directive without conflicting directives or fenced/patch text; prior benign commentary items are permitted, while action-bearing predecessors are rejected. Missing, duplicate, wrong-owner, forged/quoted/indented, stale and failed evidence are covered by the new regressions. The saved002 exit0/clean-review artifact specifically remains rejected.

Publication readiness polls only saved source frames every 25 ms for at most 15 seconds, further bounded by the remaining 240-second watchdog interval. It neither sends another prompt nor changes usage grace; zero/expired readiness fails. Watchdog cancellation is checked before inspection and again before entering review. The ordinary closed-capture and outcome publication path remains in place, so a failed pre-review guard remains an actual failed scenario rather than a retry request.

## Coverage limits and complexity

The actual control-flow regression uses public `CodexBackend`/`CommandChat` with an explicitly synthetic app-server and pinned synthetic bridge results. It exercises private feedback → actual normal shared patch → actual ordinary shell check → actual scripted DONE, and verifies the check's created marker, four model-bound inputs, and trace events. It does **not** qualify confinement, authentic provider behavior, or behavioral RED: its private result is deliberately synthetic. Real native/public/source equality, full ordinary shell receipts, held/final test semantics, and complete review remain root's closed-source gates after the sole admitted observation.

One inspection uses indexed maps/sets/queues and a constant number of frame/text passes: linear in inspected records/bytes, with no new quadratic join. Readiness repeatedly reads the complete current prefix under a fixed time bound; it is not a streaming memory bound or incremental collector. Existing live-reader incomplete-tail tolerance is used only during pre-review readiness; unchanged closed-capture verification and the separate postcapture source audit retain strict closure responsibility.

No public API or product UI changes occur, so product architecture/UI-harness changes are unnecessary. The diagnostic scope accurately describes its additional success gate and remaining real-agent requirement. Originals001/002, their failed/partial qualifications and accounting flags remain immutable; no savings, causal effect, automatic replacement or broader-study completion is claimed.
