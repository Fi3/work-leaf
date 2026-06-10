# Three-Feature Bench Interim Baseline

- captured_at: 2026-06-10T04:20:15+0200
- benched_orchestrator_commit: 657d66d2c9e880920fa307766bcb9d92e564a042
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.R4n6sQ/repo
- daemon_url: http://127.0.0.1:36331
- elapsed_at_capture: about 45 minutes
- result_at_capture: still running
- read_permission_mode: default mediated reads

## State

```text
busy=true
review-user-1 loading=WaitingForReply completion=- lines=15 done=0 max_line=756
review-user-2 loading=- completion=- lines=12 done=0 max_line=1039
user-1 loading=- completion=- lines=37 done=1 max_line=82834
user-2 loading=- completion=NeedsDecision lines=60 done=1 max_line=109867
user-3 loading=Launching completion=- lines=55 done=0 max_line=25368
```

## Commits After Base

```text
e4e3a07 UPDATE apply implement-strict-selected-agent-slash-command-execution-when-a-selected-agent-ch patch from user-2
25f4873 UPDATE apply implement-strict-selected-agent-slash-command-execution-when-a-selected-agent-ch patch from user-2
9cf78ea UPDATE apply when-review-process-is-done-the-patch-agent-chat-must-be-highlighted-and-ask-is patch from user-3
c54c950 UPDATE apply add-vim-like-visual-mode-for-both-panes-when-i-do-v-i-can-select-the-text-in-foc patch from user-1
423fa8a UPDATE apply implement-strict-selected-agent-slash-command-execution-when-a-selected-agent-ch patch from user-2
5a9de13 UPDATE apply when-review-process-is-done-the-patch-agent-chat-must-be-highlighted-and-ask-is patch from user-3
1974a44 UPDATE apply user-agent patch from user-3
a0df45d UPDATE apply user-agent patch from user-1
6951e69 UPDATE apply user-agent patch from user-2
```

## Observations

- The review transition works for at least user-1 and user-2: both emitted `@work-leaf done`, review chats were created, and user-2 reached `NeedsDecision`.
- Review is catching real issues: `review-user-1` found a left-pane visual selection bug and user-1 produced follow-up commit `c54c950`.
- The run is functionally progressing but not performance-clean. At capture time it already exceeded a practical baseline target for a 3-feature run.
- Very large single chat lines are present: user-1 max line about 82 KB, user-2 max line about 110 KB, user-3 max line about 25 KB. Command/test output and terminal snapshots need compaction before appending to chat transcripts.
- user-3 repeatedly ran masked validation commands with `|| true`. Locked command execution should reject obvious failure-masking shell constructs and ask the agent to rerun checks normally.
- Live loading labels are misleading. Patch agents can still show `Launching` long after patches, checks, and follow-up work have happened.
- Shared-worktree interference is reduced but not gone. Agents avoided some other-agent focused tests and used build-output locks, but overlapping terminal/workspace changes still caused integration churn.

## Candidate Orchestrator Fixes Before Rerun

- Reject masked locked commands such as unquoted `|| true` or `|| :`, including inside `sh -c` and `bash -c` script arguments.
- Compact large locked-command outputs before adding them to agent transcripts, especially whitespace-heavy terminal snapshots.
- Improve live status/introspection labels so long first-turn work is not reported as only `Launching`.
- Add an optional, generic shared context/scout path only if it does not serialize patch-agent startup.
