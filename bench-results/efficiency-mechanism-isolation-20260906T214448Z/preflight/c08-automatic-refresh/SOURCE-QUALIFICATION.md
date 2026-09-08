# Automatic changed-refresh source qualification

The v7 representation changes only eligible automatic changed-file diff bodies to
the exact held current full text, plus two owned response-local coherence spans.
Default and earlier schemas retain ordinary behavior. The source ownership,
before-send snapshot advancement, failure behavior and linear construction are
documented in `docs/architecture.md` and the implementation/review receipts here.

Full repository gates pass: **546 tests, zero failures, 23 ignores across45
targets** (`740d96` / `4ac1e9`), including automatic invocation of the synthetic
child fixtures. All-feature Clippy (`70ed6e` / `05e500`), default all-target Clippy
(`6c9ae6` / `d01b7d`) and formatting (`a991f3`) are clean. Default library/automatic
recovery checks pass53 tests with one automatic child ignore (`c0d34d` / `7eea31`).
The root-gate receipt retains the complete all-feature tool output.

The private diagnostic gates pass seven tests (`9b874c`), formatting (`2f002b`)
and all-target/all-feature Clippy (`07e2b5`). Actual CommandChat synthetic coverage
includes a real distinct-host GitPatcher mutation, unchanged old-read forwarding,
genuine stale rejection/current refresh, ordinary repair ACK, executed locked
Cargo check and processed DONE in five calls. An artificial two-round limit
returns incomplete after three calls and cannot pass the local success guard.

These are source and automatic checks only. No actual provider has yet run for
C08, so the agent-facing intervention is **not marked ready**. A separately frozen
one-author subscription scenario, complete source/public/native delivery and
semantic review are required before a natural modified-only benchmark admission.
No C15 rerun, replacement, control or percentage estimate is part of these gates.
