# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:06:09+02:00
- duration_seconds: 1695
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
- web_ui_url: http://127.0.0.1:36619
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.EzIB4a
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1297
- changed_lines_deleted: 50
- changed_lines_total: 1347
- token_usage: linearize: input=4748670 cached_input=4481920 output=22570 reasoning_output=8638; review-user-1: input=751595 cached_input=638464 output=10712 reasoning_output=7883; review-user-2: input=242859 cached_input=220672 output=5366 reasoning_output=3695; review-user-3: input=610399 cached_input=516608 output=9474 reasoning_output=6635; user-1: input=1753175 cached_input=1622144 output=8775 reasoning_output=5637; user-2: input=839566 cached_input=712064 output=11576 reasoning_output=8613; user-3: input=927530 cached_input=782336 output=15606 reasoning_output=10451
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6b retry run 2 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-2/parallel-current-6b-retry-20260624T053754+0200-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-2/parallel-current-6b-retry-20260624T053754+0200-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-2/parallel-current-6b-retry-20260624T053754+0200-r2-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	268	linearize reviewed patches
review-user-1	-	-	59	review user-agent
review-user-2	-	-	39	review user-agent
review-user-3	-	-	32	review user-agent
user-1	-	NeedsDecision	127	vim-visual-mode-for-panes
user-2	-	NeedsDecision	84	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	153	patch-agent-review-done-confirmation
```

## Recent Commits

```
b0a1d92 ADD terminal visual selection and clipboard yanks
328b51b ADD clean-review feature completion confirmation
d0b15e0 ADD route selected-agent slash commands through backend hooks
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
