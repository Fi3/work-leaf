# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:34:15+02:00
- duration_seconds: 1987
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
- web_ui_url: http://127.0.0.1:43993
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.rxPNc2
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1554
- changed_lines_deleted: 83
- changed_lines_total: 1637
- token_usage: linearize: input=3051116 cached_input=2884096 output=19695 reasoning_output=6698; review-user-1: input=1856916 cached_input=1800320 output=20957 reasoning_output=16782; review-user-2: input=557130 cached_input=475008 output=7691 reasoning_output=5207; review-user-3: input=434902 cached_input=358784 output=4071 reasoning_output=2129; user-1: input=921663 cached_input=769280 output=6031 reasoning_output=1808; user-2: input=959837 cached_input=864000 output=9821 reasoning_output=5667; user-3: input=1527840 cached_input=1345792 output=4903 reasoning_output=1411
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: current worktree parallel batch 12 run 4
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-4/parallel-current-12-20260624T010102+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-4/parallel-current-12-20260624T010102+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-4/parallel-current-12-20260624T010102+0200-r4-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	237	linearize reviewed patches
review-user-1	-	-	67	review user-agent
review-user-2	-	-	52	review user-agent
review-user-3	-	-	30	review user-agent
user-1	-	NeedsDecision	116	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	139	strict-agent-slash-commands
user-3	-	NeedsDecision	109	patch-agent-done-prompt
```

## Recent Commits

```
0ce3930 ADD vim visual selection in both terminal panes so users can copy displayed text
42c480b ADD reviewed-feature confirmation so completed patch chats close after user approval
ae820eb ADD strict selected-agent slash command execution so backend commands bypass ordinary prompts
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
