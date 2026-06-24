# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T13:41:38+02:00
- finished_at: 2026-06-24T14:17:54+02:00
- duration_seconds: 2176
- benched_binary_commit: 65157b102e1fdebca6264286fdcb6bf38b1d92c9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.141.0
- agent_model: gpt-5.5
- agent_model_source: codex config /home/user/.codex/config.toml
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: codex config /home/user/.codex/config.toml
- requested_agent_model: default
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: http://127.0.0.1:40883
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.36bkDL
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 8
- changed_files: 11
- changed_lines_added: 1334
- changed_lines_deleted: 75
- changed_lines_total: 1409
- token_usage: review-user-1: input=540531 cached_input=464640 output=7648 reasoning_output=5204; review-user-2: input=464974 cached_input=354560 output=8374 reasoning_output=5466; review-user-3: input=418198 cached_input=356992 output=10790 reasoning_output=8299; user-1: input=227016 cached_input=128640 output=3308 reasoning_output=1306; user-2: input=599165 cached_input=444160 output=8639 reasoning_output=4970; user-3: input=2089455 cached_input=1774336 output=18639 reasoning_output=10155
- code_quality: not run
- comment: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=3 done_users=3 ready_users=2 patch_agents_with_commits=3
- operator_notes: parallel baseline batch parallel-current-6e-20260624T134138+0200 run 6
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-6/parallel-current-6e-20260624T134138+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-6/parallel-current-6e-20260624T134138+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-6/parallel-current-6e-20260624T134138+0200-r6-three-feature-bench-artifacts/patches/fail

## Sessions

```
review-user-1	-	-	49	review user-agent
review-user-2	-	-	42	review user-agent
review-user-3	-	-	56	review user-agent
user-1	-	NeedsDecision	75	vim-visual-mode-for-panes
user-2	-	NeedsDecision	127	strict-slash-command-execution
user-3	-	-	213	patch-agent-done-confirmation
```

## Recent Commits

```
c3cee74 UPDATE apply when-review-process-done-patch-agent patch from user-3
717eadc UPDATE apply when-review-process-done-patch-agent patch from user-3
423801e UPDATE apply when-review-process-done-patch-agent patch from user-3
e76887b UPDATE apply when-review-process-done-patch-agent patch from user-3
cca59f5 UPDATE apply implement-strict-selected-agent-slash patch from user-2
aebf92a UPDATE apply user-agent patch from user-1
5a1ca71 UPDATE apply user-agent patch from user-2
90923f2 UPDATE apply user-agent patch from user-3
c92a0b7 FIX compact orchestrator and UI traffic for concurrent agents
cb5c388 FIX keep Codex resume prompts compact to avoid context blowups
2673db7 ADD localhost orchestrator daemon for CLI isolation
b831ebf UPDATE command-mode typing hints to ignore pure navigation bursts
d731958 UPDATE apply user-agent patch from user-1
9a2e3a6 UPDATE apply user-agent patch from user-1
358999c UPDATE apply user-agent patch from user-1
114c939 FIX review full patch-agent scopes before acceptance
41b4167 UPDATE document Codex slash-command resume policy exception
d9a1176 UPDATE format slash-command regression test so cargo fmt stays clean
db00ed5 UPDATE apply user-agent patch from user-1
50db6e2 UPDATE apply user-agent patch from user-1
cdf31a5 agent
e97dc14 FIX preserve exact reviewed commits for linearize scope
0ae881e FIX preserve new session snapshots before worker polling
bbef6e1 UPDATE apply user-agent patch from user-1
81634c9 UPDATE apply user-agent patch from user-1
cb4e212 UPDATE apply user-agent patch from user-1
0ccfe09 UPDATE apply user-agent patch from user-1
427a5c6 FIX block dirty command output before review and scope linearize
d504abf UPDATE document terminal ready notifications
a5f8a15 FIX require patch-agent readiness before review and cap locked commands
```

## Final Status

```

```
