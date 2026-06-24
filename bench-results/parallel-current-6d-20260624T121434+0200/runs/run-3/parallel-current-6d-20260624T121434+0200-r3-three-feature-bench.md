# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:55:44+02:00
- duration_seconds: 2470
- benched_binary_commit: 65a71fe8bd1a9a2adc4173d0775526514c01a76e
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
- web_ui_url: http://127.0.0.1:42585
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.D46lcS
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1430
- changed_lines_deleted: 24
- changed_lines_total: 1454
- token_usage: linearize: input=4107274 cached_input=3871872 output=24795 reasoning_output=8517; review-user-1: input=1187548 cached_input=1144064 output=8682 reasoning_output=5630; review-user-2: input=352877 cached_input=287232 output=6739 reasoning_output=5619; review-user-3: input=450226 cached_input=423168 output=7270 reasoning_output=5546; user-1: input=959874 cached_input=860672 output=3572 reasoning_output=1374; user-2: input=669283 cached_input=582144 output=9613 reasoning_output=6138; user-3: input=1771576 cached_input=1453312 output=9794 reasoning_output=6142
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 3
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-3/parallel-current-6d-20260624T121434+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-3/parallel-current-6d-20260624T121434+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-3/parallel-current-6d-20260624T121434+0200-r3-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	261	linearize reviewed patches
review-user-1	-	-	42	review user-agent
review-user-2	-	-	20	review user-agent
review-user-3	-	-	33	review user-agent
user-1	-	NeedsDecision	104	vim-visual-mode-panes
user-2	-	NeedsDecision	138	strict-agent-slash-commands
user-3	-	NeedsDecision	110	review-done-chat-confirmation
```

## Recent Commits

```
ff5c883 ADD visual-mode clipboard selection so terminal panes support vim-style yanks
cafc3a9 ADD selected-agent backend slash commands so provider commands bypass chat prompts
208ffad ADD review-completion prompts so reviewed patch-agent chats close cleanly
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
