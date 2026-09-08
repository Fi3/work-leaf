# C08 saved-baseline source compatibility

The release cut is `e497ff5df5bf84e169aa4460953383639df956f8`, published as
`d0a22e78326fccd60e1639fef34da75cfca8947c`. The complete50-input
[build attestation](BUILD-ATTESTATION.json) binds the actual private build source
to identical published source and both release binaries. Build receipts
`cc6924`/`3890ff` record a successful locked offline release build with
`bench-experiments`. This source check is not phase admission or token evidence.

Of the47 existing inputs in the qualified C15 preflight, only
`src/bench_experiment.rs` and `src/orchestrator.rs` differ. Their C08 change is
the separately selected v7 identity and automatic-refresh construction/delivery
hook. The new production helper is `src/bench_automatic_refresh.rs`; the other
two new inputs are cfg(test) modules. The source, guards and actual public patch/
edit recovery-path coverage are in the committed C08 implementation and
independent reviews. Full repository checks pass546tests; default checks pass53,
and the private diagnostic checks pass7, with clean formatting and Clippy.

`patch_conflict_refresh_response` retains the actual held snapshots and original
single diff construction. The private renderer records owned UTF-8 ranges and
only selects full current text for eligible v7 automatic changed diffs.
`automatic_refresh::Capture::forward` runs after the original snapshot-recording point and
before the existing send. Requested reads, default/inactive and prior schemas
retain ordinary bytes, tested through real Git patch/edit conflicts; evidence
write failure prevents delivery without undoing snapshot advancement. There is
no provider-facing diagnostic fixture in the natural workflow runner.

The prior [C15 source comparison](../test-first-screen/BASELINE-COMPATIBILITY.md)
and [saved W comparison](../../PREFLIGHT-CANDIDATE-BASELINE.md) retain the earlier
benchmark-only hooks' non-target scopes. Under v7, v6 private-preview enrollment,
test-first author policy and apply gate are inactive; v4 requested-read/write-
format/resupply and v5 reviewer archive selection are inactive. The C08
default/legacy identity coverage supplements those source checks. Ordinary
provider transports, GitPatcher, locks, command execution, review/linearize,
Cargo dependency graph, observer source and web assets remain byte-identical to
the preceding qualified cut. No prospective observer or C15 derived-output
correction is included in these executables.

Current global configuration retains exact rawSHA
`d36b9caec082759492578037173fe3bca099aad63f8e2553314601d2506e366d`.
The unchanged [tooltip/trust qualification](../test-first-screen/CONFIG-TOOLTIP-QUALIFICATION.md)
therefore remains directly applicable: only16 individually proven unrelated
project-trust entries and the one identified startup-tooltip counter separate
the complete parsed configuration from saved W. No config bytes are copied or
written, no blanket field exclusion is introduced, and any later drift remains
subject to the original supervisor. Actual fresh checkout inventories and
ancestor eligibility remain per-launch gates.

The clean five-driver snapshot, task, subscription wrapper, native CLI,
unchanged observer and six retained W accounting receipts must match the final
phase freeze. No new baseline or accounting replay belongs to this check.
Whole-workflow descriptive compatibility does not assert an unobserved
provider-side temporal identity. C08's real-agent source/public/native/semantic
qualification and independent final prelaunch review remain separate gates.
