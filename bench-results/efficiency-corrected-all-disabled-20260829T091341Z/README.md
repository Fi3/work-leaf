# Corrected All-Disabled Work Leaf Study

## Goal

This study collects three independent observations of concurrent Work Leaf with these three context
delivery mechanisms disabled:

1. changed-file rereads send the full current file instead of a diff;
2. unchanged-file rereads resend the full file instead of a digest; and
3. reviewers reconstruct the exact review target from Git instead of receiving that context inline.

The third control preserves the normal review route. A reviewer may use mediated Git commands while
gathering context, but its final response places `NO_FINDINGS` or `FINDINGS` before the standard
`@work-leaf done` directive. This matches the parser contract and prevents a clean review from being
routed to the patch agent as findings.

## Scope

The production Work Leaf checkout, original three-feature task, and frozen feature scorer are not
modified. The isolated source is commit `d217f3803ac0f417671e27cc8fb18064ff0f4ea9`, based on the
Points 8/9 instrumentation commit `4707ceb4903a09646857d1e316cb45acb15a3d07`.

Every provider thread uses GPT-5.5 with `xhigh` reasoning. The concurrent Work Leaf workflow keeps
its normal validation behavior and final formatting, Clippy, and test gate. Recursive provider
verification is blocked by the existing benchmark profile.

The three attempts run concurrently with separate temporary roots, result directories, observer
identities, and run IDs. They are independent observations, not pairs. A failure in one attempt does
not remove or invalidate another attempt.

## Accounting

Completed response usage is observed directly. Interrupted Work Leaf responses use the established
conservative raw-token ceiling based on the emitted effective context window, maximum output, and
captured new-turn prompt size. No exact Work Leaf token percentage is reported when interrupted
response usage is missing.

## Commands

`./test-study` validates the frozen source, binaries, schedule, and launcher contract without a
provider call. `./run-batch` launches the three declared attempts concurrently and waits for every
attempt to finish.
