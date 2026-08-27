# Run Analysis

## Result

This point-7 gate is not green and provides no valid Work Leaf versus direct Codex token comparison.
Both admitted workflows and all their evidence are retained. No retry, step 8, or step 9 was
launched.

The intended measurement repairs did work:

- every observed provider thread used GPT-5.5 with xhigh reasoning;
- neither workflow launched a recursive Codex session;
- both `recursive-codex-attempts.log` files are empty;
- the direct observer capture is complete and internally reconciled; and
- the original task, fixed base, normal validation rules, and quality scorer were unchanged.

Two separate infrastructure failures prevented the workflow comparison.

## Direct Codex Failure

The direct workflow reached one GPT-5.5/xhigh implementation conversation and used 1,931,994 raw
tokens and 137,562 uncached tokens. Those token totals are usable for that failed attempt, but not for
a Work Leaf comparison.

The agent could inspect and reason about feature 1, but every write-capable tool failed before editing
the repository. The nested Codex sandbox tried to open:

`/tmp/codex-bwrap-synthetic-mount-targets-1000/lock`

The outer environment exposes that path as a read-only mount. The agent therefore produced no commit
and the direct workflow stopped during feature 1. The exact errors are preserved in
`runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/runs/sequential-feature-1-implement.stderr`.

The scorer reports direct Codex as 1/3 because the unchanged fixed base already passes the `/status`
fixture. It does not mean this failed direct attempt implemented one feature. The saved direct
candidate is exactly the fixed base commit.

## Work Leaf Failure

Work Leaf completed and passed review for the visual-selection and `/status` requests. The third
agent was still actively working on the reviewed-feature close/reopen request when the launcher
stopped it.

The launcher considered only visible session-state changes when applying its 30-minute busy-stall
guard. Feature 3 produced no new visible line between 18:10:20 and the stop, but its raw app-server
stream continued growing until 18:40:22. The launcher therefore terminated an active provider turn
and wrote its report at 18:41:50.

The interrupted app-server invocation has no end record, so Work Leaf's partial totals of 947,745 raw
and 198,049 uncached tokens are not a complete workflow measurement. They must not be used to claim a
token increase or reduction.

The scorer reports the retained partial candidate as 2/3: visual selection and `/status` pass, while
reviewed-feature close/reopen fails. This is useful partial-quality evidence, not an equal-output
token comparison.

## Accounting Status

This direct attempt had only one CLI invocation, so it did not itself exercise launch-plus-resume
addition. The repaired observer was separately checked against the complete first-pilot direct
capture before this run: it summed 42 CLI invocations into 15 conversations, matched all 15 rollout
files, and reported 35,947,089 raw and 1,353,041 uncached tokens with no accounting error. Those
outputs remain under `preflight/`.

## Required Next Gate

Another paid pair is not ready yet. The next infrastructure patch should:

1. Give Work Leaf and direct Codex separate writable provider temporary directories while retaining
   the same direct `workspace-write` sandbox.
2. Verify the direct path with one bounded real workspace-write call that actually performs a tiny
   edit in an isolated fixture. The earlier read-only reply smoke could not expose this write failure.
3. Count active app-server capture growth as busy progress, so an xhigh turn that is still streaming
   cannot be killed only because no new UI line appeared.
4. Add deterministic regressions for both failures and rerun all local checks.
5. Run a new one-pair point-7 gate with the same fixed inputs and no automatic retry.

Only a green replacement gate can authorize discussion of steps 8 and 9. The Work Leaf
implementation, original task text, and quality scorer do not need to change.
