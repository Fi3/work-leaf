# Archive population accounting qualification

Completed 2026-09-15T01:42:00+00:00. Supplemental accounting compares both archive
populations using the same string ordering. Actual membership and content changes
remain errors. The unchanged original template and original failed audit results
are preserved; `population_audit.py` publishes separate supplements exclusively.

The fail-first regression reproduces the erroneous population flag on unchanged
`unit/result` and `unit-log` paths. Eleven tests pass after the correction, including
real population/content changes and once-only native cache ownership. The previous
eleven accounting tests, all 83 counter guards and required Cargo checks pass.

All three actual supplements under
`phases/author-joint-confirmation-qualified-20260915/postcapture/<run-id>/`
reuse seven cached threads each. Every native response, turn, thread, stage total
and whole total equals its original audit exactly. Native-core executions: zero.

| Run suffix | Recorded raw | Accounting qualification |
| --- | ---: | --- |
| 004 | 21,730,404 | Incomplete integration turn; unknown tail retained |
| 005 | 25,087,991 | Incomplete integration turn; unknown tail retained |
| 006 | 23,252,285 | Complete observed ledger; no accounting errors |

These are recorded outcome costs, not a qualified full-workflow causal contrast.
BUG009's extra integration-resume developer instruction is not an accounting error
and is not cleared by this repair. No historical attributable share is asserted.

No real-agent workflow is affected: this code only reads closed artifacts and
cached accounting results, then writes a separate analysis file. It cannot call
the native audit core on a new thread or launch a provider. No normal WL, public
API, architecture or benchmark-agent policy changes; no additional real call is
needed to verify this offline correction. Existing system documentation remains
accurate. Population sorting is O(n log n), with linear content/record checks;
no quadratic pass is introduced.
