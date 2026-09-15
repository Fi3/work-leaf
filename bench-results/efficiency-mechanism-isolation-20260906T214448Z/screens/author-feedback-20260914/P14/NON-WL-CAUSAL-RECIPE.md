# Non-WL author-policy recipe and its verified causal reversal

This recipe has an actual complete non-WL execution and a qualified three-run
A+B policy reversal. Its measured all-on cost is18,545,304 raw versus a
24,297,147⅓ modified mean, net5,751,843⅓ lower after all model stages. That is
23.6729% of the modified mean, **not an attributable share of the historical50%**.
The historical combined explanation remains unfinished in P06–P08/P12/P13/P16.

## Executable components

Paths below are relative to the study directory
`bench-results/efficiency-mechanism-isolation-20260906T214448Z`.

| Component | Exact retained executable/source | Ownership |
| --- | --- | --- |
| Author host | `phases/standalone-global-hunk-pilot-01/infrastructure/drivers/host_custody.py` | `Host.apply`, `Host.command`, `Host.consume`, `invoke`, `run_stage`: actual file/commit/check feedback to an ordinary native session. SHA256`5e5be14ae50574e1d3ee81991cb4fee2d33e923ca825655644a1c33aac18b537`. |
| Full workflow driver | `phases/standalone-global-hunk-pilot-01/infrastructure/drivers/bench-three-features-serialized-host-custody` | `host_author_policy`, `run_host_author`, `run_feature_cycle`, `run_sequential_bench`: author → independent review → necessary same-author repair → integration. SHA256`a621b4ee2ae74dbef03d6c3b29b8dbaaf5842d27e186f61089ce16d658d1526f`. |
| Subscription provider | `phases/standalone-global-hunk-pilot-01/infrastructure/provider/codex` | Pinned CLI0.153.4, GPT-5.5/xhigh, ChatGPT login; unsets API-key/base/token overrides. Wrapper SHA256`8ad1d261979029fec24cf4e143aaf6afc6c688056c6bc702c824b3538b92ff32`. |
| Admission/outer supervision | `preflight/standalone-global-hunk-pilot-20260913/runner_global_hunk.py` and its retained `MONITOR-LOOP.js` | Source/input binding, one-shot identity, independent wall and recorded-usage tripwire, cancellation and retained outcomes. |

The host is Python calling the native CLI and Git, not the WL library/runtime.
The driver launches authors through `run_host_author`, reviewers/linearizer
through native `run_direct_agent`/`run_direct_agent_resume`. Its references to
the packaged work-leaf binaries at lines422–424 copy benchmark artifacts; those
binaries are not the author orchestrator. Building/testing the target Work Leaf
application is distinct from using it to orchestrate the coding workflow.

## Procedure and the upstream decision mechanism

1. Keep the complete task, required tests and repair obligations. Let the author
   inspect with ordinary read-only native tools; the host owns edits and checks.
2. **A: publish implementation and its tests as a cohesive unit when both are
   known.** Design tests first, but do not require a separate test-only publication
   and deliberately failing run solely for procedural timing. This is the explicit
   benchmark policy in `host_author_policy`, not permission to waive the current
   repository's normal test-first rules outside the experiment.
3. Accept one complete typed operation per owned final reply. One edit can contain
   multiple files/hunks. Validate paths and exact old blocks before publication;
   apply the real change and commit it. Return its factual acceptance/commit result
   to the same native thread before it chooses another operation.
4. **B: give result-bound next-action guidance.** After accepted work, request one
   relevant focused validation, then DONE when that work is complete. The same
   guidance explicitly permits genuinely unresolved requirements and real repairs;
   it is not a mandatory first-GREEN quota. Do not restate or resend accepted edits.
   After commands or an explicit discard, guide the next required action or handoff.
5. Execute checks for real. Return command, exit status and actual stdout/stderr.
   Keep full raw streams and declared rendering limits. Pending command-produced
   tracked changes require an ordinary committed edit or explicit discard; DONE
   cannot conceal them.
6. Run independent reviews, resume the original author for necessary repairs and
   repeat review until clean. Keep final cross-feature formatting, Clippy, complete
   tests and history integration. Preserve all failed checks/rejections and costs.

The experimentally observed chain is **joint publication/guidance policy → chosen
publication/check/handoff sequence → actual validation/repair/review work → later
charged context**, including the integration offset. It is not the statement
“fewer responses save tokens.” Separate A/B percentages and an independent effect
of host custody C are not identified by this reversal; C remains held fixed.

## Failure and recovery behavior

`Host.consume` rejects a grouped/malformed final before executing an operation.
`plan_edit` rejects invalid paths, duplicate identities, absent/ambiguous exact
blocks and stale input. Rejections report the actual failure; prior accepted
work is not presented as rejected or replayed as a new success. The corrected
bare-hunk matcher searches the evolving whole file for one unique old block;
explicit contextual anchoring remains intact.

`Host.unchanged` checks expected source/index/HEAD before and after native turns
and host work. Unexpected source effects are fatal and retained, not silently
restored and retried. A command's allowed tracked changes are captured with real
before/after diffs and restored pending explicit acceptance/discard. Errors,
timeouts, nonregular sources and unauthorized effects remain failures. A completed
stage cannot consume more replies. Native generation failure is not a replacement
slot; complete diagnostic-bearing turns use only the separately qualified parser
rules recorded in the modified-arm evidence.

These paths matter to the actual recipe: the all-on execution contains five
rejected proposals, six accepted edits, eight checks including three failures,
and five completed author/fix stages. No cost or required recovery is removed.

## Running boundary, not a new run authorization

The standalone host's executable interface is:

```sh
python3 "$recipe_host" \
  --repo "$fresh_checkout" --feature "$task_label" --stage initial-author \
  --prompt-file "$frozen_prompt" --artifact-dir "$fresh_artifacts" \
  --codex-bin "$qualified_subscription_wrapper" --model gpt-5.5 \
  --serialized-feedback --deadline "$admitted_monotonic_deadline"
```

The variables are explicit admission inputs, not defaults or instructions to run
this unchecked. Use an independently bounded supervisor, unchanged effective
benchmark AGENTS, clean exact basec92a0b7060a36eac6db2d869b85e589a7a9480f9 for
the historical task, frozen complete prompt/provider/tool settings and new
exclusive artifact identities. The full driver calls that interface and owns
the review/fix/integration sequence; the source table identifies each entry point.

The original admitted command and its complete environment are retained in
`phases/standalone-global-hunk-pilot-01/ROOT-ADMISSION.json`, `SCHEDULE.json` and
`PHASE-MANIFEST.json`. That one-shot run is consumed: **do not rerun or overwrite
its batch path/identity**. A new scientific reproduction needs its own authorized
admission and actual current startup/resume input qualification. Stored subscription
auth remains in its normal location; never substitute API credits or copy credentials.
Private source-pinned input restoration belongs outside the measured policy, and
unexpected actual tool/catalog/permission drift invalidates attribution.

This document verifies the retained executable recipe and its actual causal
reversal. It does not certify a fresh current launch, freeze remote account state,
promise the same stochastic token total, or authorize another all-on workflow.

## Retained execution and causal verification

[Source/execution verification](SOURCE-EXECUTION-VERIFICATION-20260915.json)
matches all61 original manifest file hashes, both exact reference prompt/host
hashes and all10 input-evidence pins of the connected cost report. It joins the
five actual author/fix result files and their event logs without re-executing
the native usage core. Thirteen full-adapter ownership/source tests and the
observer-path inverse test pass; no new model turn belongs to this verification.

| Actual retained observation | Evidence and interpretation |
| --- | --- |
| All-on non-WL execution | `../../../phases/standalone-global-hunk-pilot-01/postcapture/MECHANISM-SCREEN.md` and `NATIVE-USAGE-SUPPLEMENT.json`: all three initial authors publish one implementation/tests unit, execute a focused check and hand off. Five author/fix stages retain24 invocations, six accepted edits, five rejections and eight actual checks. Whole workflow18,545,304 raw. |
| Qualified reversal | `../../../phases/author-joint-confirmation-qualified-20260915/PROTOCOL.md` and its linked parent protocol; `../P11/PREFIX-WORK-CHAIN-20260915.json`: A-off restores actual separate test publication/RED; B-off removes only the owned guidance group. All nine initial authors execute RED; seven perform further host work after first GREEN, versus immediate handoff in the three all-on initial authors. |
| Whole-stage effect | [P11 connected result](../P11/CONNECTED-RESULT-20260915.md): modified totals27,049,174 /23,754,840 /22,087,428. Mean24,297,147⅓; earlier-stage excess7,582,710 minus1,830,866⅔ cheaper integration gives net5,751,843⅓. |
| Exact reverse operation | `../P02/EXACT-SPANS.json`, `../full-workflow-009/full_workflow.py::driver_replacements/load_host`, and the qualified phase's frozen source: two A spans and the complete B launch/result guidance group. Real commit/check facts, matcher, required work, inspection tools and stage flow stay fixed. The recorded parser/input/path repairs are setup, not additional saving factors. |

The actual all-on first author recovers from two rejected proposals, publishes
a five-file unit, passes its focused test and returns DONE. Subsequent real
review findings lead to a two-edit repair. The completion fix retains three
failing formatting checks before its passing check and handoff. Thus the recipe
does not suppress every failure, disallow repairs or impose a one-edit limit.

Both quality/outcome limitations remain: the all-on specimen passes two of the
three frozen feature checks and fails the completion check; the user's working
no-quality-loss premise does not turn that into a pass. In the modified contrast,
004's integration completes model work but fails the instruction-restoration
guard; its exact saved code separately passes the original local final gates.
Its failed report remains failed and its cost stays in the mean. Older failed/
unqualified integrations and their unknown tails remain in the separately
reported all-attempt cost, not silently removed from the research record.

The causal verification resolves the **A+B group in this connected comparison**.
It does not identify one stopping sentence, an independent C share, the joint
C08/C25 effect, a population confidence interval or the amount transferable to
the original six-versus-six gap. P14's executable-recipe obligation is distinct
from those still-required attribution and final-acceptance obligations.
