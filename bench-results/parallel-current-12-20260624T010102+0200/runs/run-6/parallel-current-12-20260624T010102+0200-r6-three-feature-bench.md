# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:37:29+02:00
- duration_seconds: 2181
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
- web_ui_url: http://127.0.0.1:41075
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.JqxDHe
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1359
- changed_lines_deleted: 42
- changed_lines_total: 1401
- token_usage: linearize: input=6191600 cached_input=5925888 output=23043 reasoning_output=7389; review-user-1: input=1270132 cached_input=1188352 output=14150 reasoning_output=9959; review-user-2: input=602179 cached_input=477056 output=5915 reasoning_output=4008; review-user-3: input=1873990 cached_input=1809408 output=20804 reasoning_output=12143; user-1: input=978126 cached_input=891008 output=7091 reasoning_output=4269; user-2: input=890388 cached_input=790528 output=9398 reasoning_output=6759; user-3: input=2728301 cached_input=2465664 output=12534 reasoning_output=6824
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 12 run 6
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-6/parallel-current-12-20260624T010102+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-6/parallel-current-12-20260624T010102+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-6/parallel-current-12-20260624T010102+0200-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	328	linearize reviewed patches
review-user-1	-	-	52	review user-agent
review-user-2	-	-	34	review user-agent
review-user-3	-	-	119	review user-agent
user-1	-	NeedsDecision	105	vim-like-visual-mode-for-panes
user-2	-	NeedsDecision	130	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	151	review-done-feature-confirmation
```

## Recent Commits

```
917ca0a ADD feature-done confirmation so clean reviews close patch chats explicitly
7723789 ADD vim visual selection for terminal panes so users can copy focused text
48e56f2 ADD route selected-agent slash commands through backend command execution
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
