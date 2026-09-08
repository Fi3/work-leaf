# C08 public recovery coverage

This receipt addresses the two automatic-coverage gaps in
`INDEPENDENT-REVIEW.md`. It does not establish actual-provider qualification or
replace final full repository gates. Only the new, uncommitted public test file
is edited; reviewed runtime sources and committed predecessor tests are unchanged.

## Exact cut and execution

`tests/bench_automatic_refresh.rs` SHA-256:
`1fa6f3a37e8480ce0c30beb1df85c4354f1b44fbbd4104a04648cfb04c519f85`.

Runtime source endpoints remain:

| Source | SHA-256 |
| --- | --- |
| `src/bench_experiment.rs` | `cb7e1326d1f598a81df6f1544804c6b6285df185aee3021b15a01b6cdf7a3b28` |
| `src/orchestrator.rs` | `e19e477a4d9eccbdf479c75589da35dda623a6236c7fdd6194748342841015fe` |
| `src/bench_automatic_refresh.rs` | `dc09943eb17ed9d02bbed4d938908fe5dbbd4fe74aaa8ee8204844f9ae406ec2` |

The new assertions were staged before fixture support. Under the root-coordinated
Cargo slot, `cargo test --locked --offline --test bench_automatic_refresh
--features bench-experiments <name> -- --exact --nocapture` produced:

- Tool `ccf1a1`: the valid-v6 test failed because the original fixture manifest
  lacked `private_preview`.
- Tool `343630`: the evidence-write test failed because the required public-path
  failure invariant was absent from the fixture result.

After fixture implementation, the complete public target passed in `fedc4a`.
The owned test file was formatted with rustfmt (`ca9337`); the final repeat of
that same narrow target (`47e465`) passed **5 tests, 0 failed, 1 subprocess test
ignored**, 0.21 seconds. The ignored child is invoked by the automatic parent
tests. Final `git diff --check` passed (`99e5bc`). No Cargo process remained at
handoff. No provider or private command executor was invoked.

## What the tests establish

`valid_v6_bootstrap_preserves_actual_public_recovery_bytes` owns a canonical
project, empty private namespace, exact executable/source/config hashes and a
complete v6 descriptor. The actual Rust bootstrap compiles and invokes the pinned
Python validation-only sentinel and publishes the expected completed validation
output. Both real public `AgentOrchestrator` edit and unified-patch conflict
paths then produce exactly the baseline read, rejection refresh, subsequent read,
final source, ACK and bundle-population results. No v7 automatic-refresh event
appears in the v6 trace.

The sentinel is explicitly synthetic. AgentOrchestrator has no private author
registry, so this test does not claim private-preview enrollment, confinement,
test execution or real-provider behavior. Those are distinct C15 qualifications.
This is the valid-v6 automatic-recovery identity boundary.

`failed_evidence_write_blocks_actual_recovery_send_but_keeps_snapshot_advance`
uses a fresh Linux subprocess for each ordinary edit/patch path. After the actual
initial read and genuine fixture source advancement, the test locates the sole
runtime trace descriptor and temporarily replaces only that descriptor with a
read-only file description. Git filesystem writes are otherwise unaffected.
The real automatic-recovery call returns an I/O error with zero backend
deliveries and unchanged current project source. No automatic-refresh trace row
is published. RAII restores the original file description and descriptor flags,
including close-on-exec, even when unwinding.

The next same-author ordinary read reports the current snapshot unchanged; the
failed evidence write did not roll back or defer automatic before-send tracking.
A subsequent real ordinary repair produces PatchApplied/ACK with the restored
writer. This directly covers `record_snapshots → evidence failure → no send`,
separately from the earlier failed-provider-send test. Existing pure renderer
tests and source review retain the one-diff/no-extra-read claims; this fixture
does not pretend to measure hidden filesystem operations itself.

The additional fixtures are bounded subprocess/source checks, not a measured
workflow, hidden runtime injection seam or new provider architecture. Independent
delta review, final default/all-feature gates and the separately admitted C08
actual-agent recovery scenario remain required before readiness.
