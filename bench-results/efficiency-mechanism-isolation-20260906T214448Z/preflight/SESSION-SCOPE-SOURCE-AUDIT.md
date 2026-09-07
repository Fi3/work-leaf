# Session-scope source audit

## Reference and scope

The frozen driver/reference checkout is
[`infrastructure/driver-source`](../infrastructure/driver-source), at Git HEAD
`3c907f7266b63efa1558213f3e49587e999b899d`. File and symbol citations below are
relative to that checkout; line numbers refer to these frozen source bytes.

This audit establishes configured session and context boundaries from source and
the retained historical workflow contract. It does not inspect run outcomes,
compute costs or contrasts, launch providers, or establish a causal contribution
to token savings. The role description is not a census of auxiliary provider
sessions, such as title generation.

## Direct sequential workflow

Each of the three features receives a **fresh implementation conversation**, not
one coding conversation carried across all three requests.

- `bench-three-features-sequential:5` delegates to the direct driver in sequential
  mode. `bench-three-features-direct-common:1131::run_sequential_bench` loops over
  features 1, 2 and 3, recording the checkout HEAD as each feature's review base.
- `bench-three-features-direct-common:1057::run_feature_cycle` invokes
  `:801::run_direct_agent` for that feature's implementation. Its command uses
  `exec --color never --json -`, without `resume`, and records the new thread ID.
- The cycle retains that implementation thread ID for review fixes.
  `:852::run_direct_agent_resume` invokes `exec ... resume --json <thread-id> -`;
  `:945::fix_prompt` supplies the same request and the actual reviewer findings.

Each feature also receives a **separate reviewer conversation**. The local
`reviewer_thread_id` is reset inside `run_feature_cycle`, the first review launches
a new session, and later review rounds resume that reviewer. `:968::review_prompt`
scopes the review to the request's `feature_base..HEAD` diff and includes the
previous fix response/evidence when present. The driver does not inject the whole
implementation conversation into the initial reviewer prompt.

The three feature cycles share one evolving checkout. Later features can inspect
earlier features' files and commits, but do not inherit their conversation history
through a cross-feature `resume` call.

After all feature reviews, `run_sequential_bench` launches `linearize-plan` as a
**new final conversation**, then resumes its saved thread for `linearize-accept`.
The prompts at `:1002::linearize_plan_prompt_sequential` and
`:1026::linearize_accept_prompt_sequential` cover all three reviewed requests since
the benchmark base, directing inspection of repository history and resulting diff.
They do not merge the implementation or reviewer conversations into that thread.

## Normal Work Leaf workflow

`bench-three-features:1011::start_benchmark_features` submits three distinct `new`
requests for concurrent execution in a shared checkout. In `src/cli.rs`,
`:938::prepare_agent_launch` increments the user-agent number and
`:1399::build_user_agent_launch` constructs distinct `user-N` identities.

`src/codex.rs:1573::launch_streaming_interruptible` starts with no thread ID and
stores the resulting ID under the agent identity. `:1676::send_streaming_interruptible`
retrieves that identity's saved thread; ordinary known-session follow-ups are sent
without reinjecting launch policy. `request_turn_streaming` starts a thread only
when none is supplied and otherwise loads/reuses it. Thus a shared app-server
process is not a shared feature conversation.

Review remains scoped by patch-agent identity. `src/workspace.rs:487` creates
`review-<patch-agent>` and `:498` determines reviewer reuse.
`src/cli.rs:1185::review_commit_streaming_with_ids` launches or resumes that reviewer,
routes fixes back to the original patch agent, and resumes reviewer rechecks.
`src/review.rs:177::render_review_source_context` explicitly supplies the target
patch agent's commit metadata, review scope, commit log and recorded `AgentSession`
messages. This is a host-assembled context handoff, not all feature histories or
the provider's entire native tool/reasoning history.

`src/cli.rs:945::prepare_linearize_launch` creates a **separate linearizer** from
reviewed targets selected by `:1364::linearize_commits`.
`src/linearize.rs:168::build_interactive_linearize_prompt` lists compact reviewed
targets across patch agents and limits rewriting to those targets. The benchmark's
`bench-three-features:1227` sends plan acceptance to that same linearizer identity.

## Interpretation boundary

The retained [historical fairness contract](../infrastructure/driver-source/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/FAIRNESS-CONTRACT.md)
(lines 28–34) describes concurrent WL and per-request Direct implementation/review
conversations with resumed fixes and a final separate linearizer.

Both workflows therefore separate feature conversations and reuse conversations
within a feature/review cycle. An explanation premised on Direct retaining one
ever-growing conversation across all three features is inconsistent with this
driver. The sources establish explicit differences in review-context preparation
and linearization handoffs, but neither their actual token cost nor their causal
share of any observed saving follows from session topology alone.
