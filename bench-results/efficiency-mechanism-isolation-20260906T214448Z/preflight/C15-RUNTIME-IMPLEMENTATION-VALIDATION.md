# C15 private test-first runtime qualification

Scope: benchmark-only `work-leaf-bench-experiment-v6` / `private-test-first` under
`AUTHORITY-C15-C21-20260907T1806Z.md`. This is the approved private workflow with
extra feedback/build/environment costs, not a pure timing-only intervention.
The ordinary shared patch, ACK, focused validation and review remain authoritative.
Only delivered factual test execution opens the first shared-apply gate. No live
private implementation/GREEN operation, provider call, control or benchmark was run
by this implementation owner. Real-agent qualification is **pending root admission**.

## Resulting ownership and source cut

`CommandChat` owns exact prepared author roles/revisions and shares its registry
across clones. A provisional synchronous injection ticket precedes provider launch;
process-unique generations, persistent explicit-author identity, rollback and
generation-bound results prevent cross-thread/deferred/missing-session enrollment.
Cancellation covers pending/provisional/committed/pre-execution/executing states.
Shutdown rejects fresh preparations; an undelivered cancelled preview stays blocked,
with no automatic replay/relaunch recovery. `WorkLeafController` revisions preserve
the prepared owner; rejected title/dependency revisions remain visible.

`orchestrator::run_private_test_preview` admits only standalone typed envelopes,
uses existing command/path/other-agent-test/failure-mask predicates, and retains
proposal identity. Actual `FileLockTable::with_read_locks(["."])` contains capture
only. Materialization/overlays/private apply/build/test/send are outside that lock.
The exact-source Python bootstrap has cleared environment and endpoint pins; it
calls only `validate`, `capture`, `test`. Qualified predecessor helper files are
unchanged. The private root is disjoint from shared source and ordinary bundles.
Cancellation publication/watchdog failures reap the direct bridge child and retain
`closed:false`, not an invented namespace-closure proof. No uncertain tree is deleted.

Live metadata is purpose plus normalized `test_paths`, not byte-offset/hash work
for the model. Host receipts retain exact afterimages/length/SHA/mode and held body;
whole-file identity does not prove semantic test-only or same-test equivalence.
Feedback has proposal, exact command/status/output/stop reason and private-not-ACK
context, but no host launch-generation, absolute receipt path or receipt SHA fields.
All those identities remain in host trace/bridge receipts.

SHA-256, relative to repository root:

```text
5850fc7dd63a97249888dc8eeff7c525f749e4f2ae6e35c372cb7fc8d97ee800  src/agent.rs
658766748beca26424d23a3d95201ba9d1948d8fbcd7910dfc0f2f0e4f52bd94  src/bench_experiment.rs
98b08ef350616c2f9d33ca0dd6ca6916cf31a0a985e694370b0097acd4e867ad  src/cli.rs
e0ef8167bd1f51cd99d56ae58835ba5d6951549c9dc9a2ccdabf9b2e568ff0f7  src/orchestrator.rs
9b05ae5bdae5f334fdceca1b81d638eff230c68edf2d3009d1c2e3bdf9fe07cc  src/workspace.rs
3d5b7a82aaa21fc0e2b8fb087ba0ce562d0fae1ca878046e9cdbbb0ebe153eb6  src/ui_harness.rs
1700f26d5ff99271ca12acbfa783ff0d5525d3720c9ba47ff088c1aaa9d40d2f  src/bench_private_test_first.rs
923c23aad1d6af7ec9b1f168a2fa33f88967464dc40648fb228669e056a3e291  src/bench_private_test_first_bridge.rs
fdd186754e73c528e8b0d196b86db63f23303f94c4c5343a62e841922013d6ac  src/bench_private_test_first_bridge_tests.rs
3705e6c148f5362bcb2e6cd9d2c25e388e64a4ec512c409aecbbc666324fb3c1  src/bench_private_test_first_controller_tests.rs
95a8943cb64c47c9cfe7b0f1dcee5e6671181422c2a3359cece9e78148d27ab2  src/bench_private_test_first_integration_tests.rs
82bfc571df12402c25e6a152a0fea0e478a8bc3dd3918199f8c4f32c91fd6e37  src/bench_private_test_first_tests.rs
cee954c70597828cb6bb9cd9d641866a6647249f9f66fe79661771368e7f2abb  src/bench_private_test_first_ui_tests.rs
20c1f2bd4cdec7f0ba4fd613eecc2fd4156554f4b4d46d53bdfbb4f6773df921  docs/architecture.md
```

`ui_harness.rs` contains only a new private cfg(test) module include; the new tests
render actual controller notice text and drive raw input through `UiHarness`.
No committed test was removed or modified. No public export/API changed.

## Test-first and final gates

The development record contains observed RED before each implementation slice.
Main command: `cargo test --lib --all-features private_test_first -- --nocapture`.
Tool-output witnesses distinguish actual assertion failures from missing new seams:

- Initial absent module/API: `b015ad`, `754f85`, `99f59f`, `91e564`, `1cd2ec`.
- Active-preview displacement, relaunch generation, reverse preparation race and
  replay settling: `c47d01`, `1cc34b`, `354a4d`, `1d3eb7`.
- Actual CommandChat result reinjection: `51f35b`.
- Independent-registry generation collision and ordinary-bundle overlap: `563874`.
- Supervisor/receipt API tests initially absent: `56f569`.
- Cross-thread injection and cancelled result publication: `d56d8a`.
- Prepared/provisional/committed cancellation and actual shutdown-before-commit:
  `bd4d76` (the old path executed private feedback/shared apply after stop).
- Actual controller/UI seam initially absent: `c2f563`. Its new fixture's buffered
  Escape assertion failed (`8a030c`); using the existing complete-byte input API
  fixed the fixture, not UI behavior. Retained failed fixture roots are untouched.
- `cargo test --lib --all-features task_facing_feedback -- --nocapture`:
  `1773aa`, visible host generation field before removal.
- `cargo test --lib --all-features unified_context_lines -- --nocapture`:
  `70d25e`, valid unified context mistaken for nested protocol before correction.
- A retained failed fixture collided with reused process ID (`a79f83`); new test
  roots use time plus a per-process counter. No saved root was overwritten.

Final exact-source gates:

- `cargo fmt`: exit 0; `git diff --check`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0, no warnings.
- `cargo test --all-targets --all-features`: exit 0, **529 passed / 0 failed /
  20 ignored across 44 targets**. New private target: **33 passed / 2 ignored**.
- `cargo test --lib --no-default-features`: exit 0, **50 passed / 0 failed**.
- `cargo check --no-default-features`: exit 0.

The full suite includes unchanged v1–v5 experiment/default tests, provider-interface,
normal patch/read/command, controller, HTTP and UI/PTY tests. These establish their
tested identities and behavior, not a newly generated benchmark comparison.
Earlier Clippy development errors (length comparison and a test-only needless
borrow) are fixed in the source cut above; no warning is deferred.

## Actual provider-free bridge replay

Two distinct retained qualification roots exist. `/tmp/c15-rust-bridge-qualified-001`
passed an earlier guard/feedback cut; it retains its original host fields. The final
cut uses **`/tmp/c15-rust-bridge-qualified-002`**, never overwriting the first.
Exact command (environment inputs are test-only, not runtime activation extensions):

```sh
WORK_LEAF_PRIVATE_QUALIFIED_ROOT=/tmp/c15-rust-bridge-qualified-002 WORK_LEAF_PRIVATE_QUALIFIED_CONFIG=/tmp/c15-bridge-project.TaPaRh/CONFIG.json WORK_LEAF_PRIVATE_QUALIFIED_BRIDGE=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-runtime-bridge/bridge.py cargo test --lib --all-features bench_experiment::private_test_first::integration_tests::qualified_bridge_runs_held_test_then_ordinary_shared_apply_and_check -- --exact --ignored --nocapture
```

Exit 0, one test passed in 1.39s. Executed test binary before/after SHA:
`b358b9cc15e993abad3bd67ff0ee758c26a3dbe127472089b9d29c36851313b4`
(`target/debug/deps/work_leaf-a5ff396da63d12c5`). It uses a generic owned clean Git
fixture, explicit empty overlays, real pinned namespace/patch/materialization/cache
bridge, and a synthetic backend only for deterministic proposal/feedback transport.
Actual held `sh check.sh` exits 1 privately; shared files remain old until the same
session receives private feedback, then explicit ordinary combined patch/ACK and
the same ordinary `sh check.sh` exits 0. No private implementation operation runs.
The 30-byte held/final test-only file is byte-identical at SHA
`715af249a390e3369e31690619dcd3569143d9b403f1112c40a10912b345b65c`.
The 65 bridge dependency pins independently rehash; source endpoints match.
The actual shared lock receipt records wait 0.000007124s and capture-held 0.312013309s;
it explicitly disclaims all-live atomicity.

Final qualification endpoint pins, relative to `/tmp/c15-rust-bridge-qualified-002`:

```text
91070148bcb2b768a023ac8f0e77cfd9bbdb160caf8097533e3b9343ccde23ad  manifest.json
251bd6b7253639f013412f96a512ea9496eff68fa465f9b45927cf40cb2fedf3  config.json
181d733ebbe9edcf8a08635f6360e707c44a783682fddd6f69fce2e7736f5930  trace.jsonl
1f426b2c86d85717b03df8e0904647c4605e591a530d4f069c4241214cf241d3  result.json
4f64c34447ea4aa178bb406e7886b80c3ddd6d95c516247e5ba314cf2962d24a  private/preview-0000000000000001/root-lock.json
48d13206d5092c0c1f3d43e93a5cb56661139748dbf9c21944c58b274c4fc17f  private/preview-0000000000000001/CAPTURE-RESULT.json
b26739bcba9b1d7d79c42ce7a79d6f9ee9abe4314f6adcb7745e4979a4b46db5  private/preview-0000000000000001/TEST-RESULT.json
9493c1e86be3fabb7863163d0a6ffd2b1eea80ac730614bdc904beaac249fdaa  private/preview-0000000000000001/TEST-EXECUTION.json
08f7580fdf8e913afc145a9b7387712dfe09c5f2899f26bd2798fdf6cd681515  private/validation/VALIDATE-RESULT.json
```

Trace sequence 1–7 is owned policy → held proposal → factual result → delivery
attempt → delivery returned → ordinary ACK → ordinary command result. This synthetic
provider transport is not an accepted native thread/turn/input join.
The separate actual frozen-project Cargo feasibility is retained in
`preflight/c15-runtime-bridge/PROJECT-QUALIFICATION.json`, SHA
`32644011b5d64f70b883ee641063337f06944de28e648b15ad6a0a9421381d92`;
its bridge `VALIDATION.md` SHA is
`f5147f59eee63cd28b37ade1ec1744cc2d5c79758280727cbdc3ff3b245115b5`.
That is separate evidence, not a claim this generic shell fixture compiled Cargo.

## Limits and remaining gate

Hostile external races, arbitrary dependency equivalence, continuous live atomicity,
unbounded history storage and semantic test equivalence are not established.
Body/state work is linear per proposal; keyed registry work is logarithmic. Repeated
completed-message parsing can rescan growing text, O(K×B), worst-case quadratic;
the eight-MiB envelope bound is not a detector-memory bound. Existing bounded mount
O(M²) and ancestor-path O(D²) checks, Git/history and repeated source census/hash work
remain explicit. This is a benchmark qualification, not a public sandbox guarantee.

The documentation catalog was checked: architecture has the resulting private v6
boundary; ordinary Readme/file-workflow/API docs retain their ordinary contract.
Operator authority/protocol prose belongs to root. No unrelated documentation churn,
observer/runtime frozen-copy mutation, native accounting change, provider launch,
cost total or causal-effect claim is part of this receipt. Independent source review
and a separately admitted actual-agent workflow are required before readiness.
