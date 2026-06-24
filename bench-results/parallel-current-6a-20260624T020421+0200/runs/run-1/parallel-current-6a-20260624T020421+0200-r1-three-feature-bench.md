# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T03:00:27+02:00
- duration_seconds: 3366
- benched_binary_commit: d4cb33d9cae99387831c690ca3b5201450558634
- benched_binary_dirty: yes
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
- web_ui_url: http://127.0.0.1:46363
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.IssFI2
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 7
- changed_files: 11
- changed_lines_added: 1302
- changed_lines_deleted: 155
- changed_lines_total: 1457
- token_usage: review-user-1: input=2378009 cached_input=2214272 output=14967 reasoning_output=7990; review-user-2: input=911472 cached_input=784896 output=10342 reasoning_output=7018; review-user-3: input=827896 cached_input=747264 output=8824 reasoning_output=5356; user-1: input=1382853 cached_input=1200128 output=12537 reasoning_output=7356; user-2: input=986921 cached_input=769664 output=11715 reasoning_output=7017; user-3: input=2529101 cached_input=2321280 output=21824 reasoning_output=17518
- code_quality: not run
- comment: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=3 done_users=2 ready_users=2 patch_agents_with_commits=3
- operator_notes: current worktree parallel batch 6a run 1
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-1/parallel-current-6a-20260624T020421+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-1/parallel-current-6a-20260624T020421+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-1/parallel-current-6a-20260624T020421+0200-r1-three-feature-bench-artifacts/patches/fail

## Sessions

```
review-user-1	-	-	90	review user-agent
review-user-2	-	-	48	review user-agent
review-user-3	-	-	42	review user-agent
user-1	-	NeedsDecision	141	vim-visual-selection-panes
user-2	-	NeedsDecision	142	strict-slash-command-execution
user-3	-	-	205	patch-chat-done-confirmation
```

## Recent Commits

```
910b1b5 UPDATE apply when-review-process-done-patch-agent patch from user-3
081cc13 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
471df47 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
178065f UPDATE apply implement-strict-selected-agent-slash patch from user-2
7f7f626 UPDATE apply user-agent patch from user-1
9483b4b UPDATE apply user-agent patch from user-3
b46fc63 UPDATE apply user-agent patch from user-2
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
c37e302 UPDATE apply mouse-scrollable-chat-pane patch from user-1
```

## Final Status

```

```
