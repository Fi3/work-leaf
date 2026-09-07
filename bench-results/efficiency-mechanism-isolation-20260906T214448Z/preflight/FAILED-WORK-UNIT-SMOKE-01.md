# Work-unit diagnostic round limit

The two diagnostics in `WORK-UNIT-SMOKE-ADMISSION.json` are failed and retained, not benchmark
observations. Control generated four responses in 12.75 seconds; incremental generated four in
14.65 seconds. Both applied two fixture edits and executed the first check. They generated the
second validation directive but the diagnostic stopped before executing it. The observer has final
invocation metadata and a valid pre-spawn project-layer inventory for each attempt.

The diagnostic configured `CommandChat::with_max_review_rounds(3)`. Despite its name,
`src/cli.rs::process_agent_reply_streaming_result` uses this limit for all pending orchestrator
directive rounds. It processed edit/check/edit, received response four, and reached its limit before
processing the second check. The five-response assertion failed with actual 4 versus expected 5.

Normal `CommandChat::new` retains its 80,000,000-round default. The benchmark daemon constructors do
not override it, and neither `src/cli.rs` nor the frozen benchmark driver is modified for this
diagnostic. This is a fixture error, not a WL runtime regression or an effect of the treatment.

The new automatic regression `diagnostic_round_budget_executes_both_checks_and_processes_completion`
uses the actual `CommandChat` with a scripted backend and both real shell checks. It fails at the
three-round cap with `did not converge`, then passes with the fixture's five-round cap. The real
fixture shares that constant. Reply/call budgets still permit only the declared five responses;
the termination receipt and 300-second watchdog remain bounded.

Original diagnostic test binary SHA-256:
`d1a94d43cbaa6baa0046ce552777277e08bbfab06565c98881a92d4e67b244b3`.
Original diagnostic source SHA-256:
`7a941a251f51f9940ac2ebb8661e6151a57ad9d639090bc5daadf9bd5a87861f`.
Its admitted observer hash and original v2 manifests are retained in the admission file and
`work-units-control` / `work-units-incremental`. A corrected diagnostic requires new identities,
separate project/artifact directories and a separate prelaunch admission; neither failed attempt
is overwritten or relabeled as successful.
