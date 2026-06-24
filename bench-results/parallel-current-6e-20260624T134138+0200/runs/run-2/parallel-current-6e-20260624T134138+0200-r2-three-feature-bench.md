# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T13:41:38+02:00
- finished_at: 2026-06-24T14:18:25+02:00
- duration_seconds: 2207
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
- web_ui_url: http://127.0.0.1:46439
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.9dogrP
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 7
- changed_files: 10
- changed_lines_added: 1551
- changed_lines_deleted: 124
- changed_lines_total: 1675
- token_usage: review-user-1: input=716591 cached_input=585728 output=10682 reasoning_output=8776; review-user-2: input=407280 cached_input=339584 output=9042 reasoning_output=6762; review-user-3: input=1470965 cached_input=1398144 output=11715 reasoning_output=7024; user-1: input=776916 cached_input=673152 output=15996 reasoning_output=12927; user-2: input=1048676 cached_input=969856 output=18532 reasoning_output=13675; user-3: input=1991938 cached_input=1799168 output=11403 reasoning_output=8207
- code_quality: not run
- comment: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=3 done_users=3 ready_users=2 patch_agents_with_commits=3
- operator_notes: parallel baseline batch parallel-current-6e-20260624T134138+0200 run 2
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-2/parallel-current-6e-20260624T134138+0200-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-2/parallel-current-6e-20260624T134138+0200-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-2/parallel-current-6e-20260624T134138+0200-r2-three-feature-bench-artifacts/patches/fail

## Sessions

```
review-user-1	-	-	37	review user-agent
review-user-2	-	-	31	review user-agent
review-user-3	-	-	97	review user-agent
user-1	-	NeedsDecision	131	vim-visual-mode-for-panes
user-2	-	NeedsDecision	148	strict-slash-command-execution
user-3	-	-	121	patch-agent-done-confirmation
```

## Recent Commits

```
48bff56 UPDATE apply when-review-process-done-patch-agent patch from user-3
17cf4b7 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
cd821a4 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
2572b5b UPDATE apply user-agent patch from user-3
da0deb9 UPDATE apply user-agent patch from user-1
924a97a UPDATE apply implement-strict-selected-agent-slash patch from user-2
43e94cf UPDATE apply user-agent patch from user-2
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
