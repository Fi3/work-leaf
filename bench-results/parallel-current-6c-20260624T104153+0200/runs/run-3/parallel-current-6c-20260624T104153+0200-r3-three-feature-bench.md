# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T10:41:53+02:00
- finished_at: 2026-06-24T11:20:56+02:00
- duration_seconds: 2343
- benched_binary_commit: 0d72ac4fbabbdd5729f38ca556ed8e923e11f1e7
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
- web_ui_url: http://127.0.0.1:37765
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.rGSObY
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1324
- changed_lines_deleted: 42
- changed_lines_total: 1366
- token_usage: linearize: input=3821982 cached_input=3585792 output=22724 reasoning_output=9582; review-user-1: input=1431459 cached_input=1365120 output=12775 reasoning_output=7806; review-user-2: input=1517917 cached_input=1322624 output=12348 reasoning_output=7051; review-user-3: input=560309 cached_input=463360 output=7643 reasoning_output=4948; user-1: input=1279569 cached_input=1140096 output=20861 reasoning_output=17448; user-2: input=867568 cached_input=758656 output=9526 reasoning_output=5559; user-3: input=1176808 cached_input=1033728 output=3322 reasoning_output=1046
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: current worktree post-commit parallel batch 6c run 3
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-3/parallel-current-6c-20260624T104153+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-3/parallel-current-6c-20260624T104153+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-3/parallel-current-6c-20260624T104153+0200-r3-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	216	linearize reviewed patches
review-user-1	-	-	58	review user-agent
review-user-2	-	-	70	review user-agent
review-user-3	-	-	57	review user-agent
user-1	-	NeedsDecision	180	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	105	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	101	review-done-close-reopen-chat
```

## Recent Commits

```
9bf36e9 ADD Vim visual selection and copy so both terminal panes can be yanked
353891e ADD selected-agent slash command dispatch so slash input avoids ordinary prompts
6395c6a ADD reviewed patch-agent completion confirmation so finished chats can close and reopen
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
cb349f9 UPDATE apply user-agent patch from user-1
bba96a6 ADD locked command execution so agents can run required checks safely
df67f96 UPDATE apply user-agent patch from user-1
82facd9 UPDATE keep repo checks and chat titles in backend agents
```

## Final Status

```

```
