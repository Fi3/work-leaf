# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T10:41:53+02:00
- finished_at: 2026-06-24T11:22:40+02:00
- duration_seconds: 2447
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
- web_ui_url: http://127.0.0.1:41635
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.SNeSd8
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1484
- changed_lines_deleted: 48
- changed_lines_total: 1532
- token_usage: linearize: input=7821595 cached_input=7454208 output=29304 reasoning_output=10091; review-user-1: input=1396321 cached_input=1191552 output=9002 reasoning_output=6061; review-user-2: input=622528 cached_input=513920 output=9301 reasoning_output=6630; review-user-3: input=1189807 cached_input=1084672 output=11300 reasoning_output=7908; user-1: input=654412 cached_input=542464 output=7920 reasoning_output=5562; user-2: input=879295 cached_input=790784 output=9334 reasoning_output=6281; user-3: input=1184831 cached_input=1026560 output=9651 reasoning_output=7283
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree post-commit parallel batch 6c run 1
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-1/parallel-current-6c-20260624T104153+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-1/parallel-current-6c-20260624T104153+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-1/parallel-current-6c-20260624T104153+0200-r1-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	364	linearize reviewed patches
review-user-1	-	-	43	review user-agent
review-user-2	-	-	53	review user-agent
review-user-3	-	-	62	review user-agent
user-1	-	NeedsDecision	108	vim-visual-pane-selection
user-2	-	NeedsDecision	95	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	111	review-done-chat-confirmation
```

## Recent Commits

```
51a3e5e ADD visual-mode pane selection so terminal users can yank focused-pane text
e8a1940 ADD selected-agent slash commands so provider commands bypass normal chat prompts
228b72b ADD clean-review confirmation prompts so patch-agent chats close only after user consent
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
