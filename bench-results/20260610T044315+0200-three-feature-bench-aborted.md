# Three Feature Bench Aborted

- result: aborted by operator after inefficient progress
- benched_binary_commit: 9a03ee1a365f95b1fafc507f1d3e9431bd923a8c
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- duration_seconds: ~3785
- agent_backend: Codex SDK sidecar
- agent_model: observed from Codex runtime, not captured in this aborted report
- read_permission: orchestrator-mediated reads
- review_completed: no
- linearize_completed: no
- temp_checkout_cleaned: yes

## Patch Progress

The run produced multiple provisional patch commits before it was stopped:

- 8521252 UPDATE apply user-agent patch from user-2
- 2676417 UPDATE apply user-agent patch from user-1
- d30a6f7 UPDATE apply user-agent patch from user-3
- 6da26e5 UPDATE apply add-vim-like-visual-mode-for-both-panes-when-i-do-v-i-can-select-the-text-in-foc patch from user-1
- 1320636 UPDATE apply user-agent patch from user-1
- d1a8750 UPDATE apply implement-strict-selected-agent-slash-command-execution-when-a-selected-agent-ch patch from user-2
- 366b491 UPDATE apply when-review-process-is-done-the-patch-agent-chat-must-be-highlighted-and-ask-is patch from user-3
- 5678c85 UPDATE apply user-agent patch from user-2
- 5783b4c UPDATE apply when-review-process-is-done-the-patch-agent-chat-must-be-highlighted-and-ask-is patch from user-3

## Token Snapshot

Last observed per-agent usage from the controller state:

- user-1: input ~5.64M, cached input ~5.31M, output ~71K, reasoning output ~37.9K
- user-2: input ~6.03M, cached input ~5.51M, output ~49K, reasoning output ~29.9K
- user-3: input ~8.99M, cached input ~8.47M, output ~54K, reasoning output ~36K

## Operator Notes

This run improved space behavior compared with the previous baseline: launch rows moved out of
startup as soon as provider streams arrived, large file and command-output lines stayed compact, and
the visible transcripts did not grow from repeated full-file snapshots.

The run still failed the performance target. Patch agents spent too long reconciling broad checks
and shared test files owned by other agents, especially terminal and UI harness tests. A routed
follow-up stream also made a secondary session do real work while the visible loading state was idle.
Those two generic issues are addressed by the next patch: project instructions are interpreted for
concurrent shared-worktree operation, and secondary follow-up streams mark the target session busy
until the owning worker finishes.
