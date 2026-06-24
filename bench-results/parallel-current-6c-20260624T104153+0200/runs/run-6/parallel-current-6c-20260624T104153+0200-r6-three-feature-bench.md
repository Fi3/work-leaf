# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T10:41:53+02:00
- finished_at: 2026-06-24T11:37:01+02:00
- duration_seconds: 3308
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
- web_ui_url: http://127.0.0.1:37429
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.PA1LS5
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1528
- changed_lines_deleted: 89
- changed_lines_total: 1617
- token_usage: linearize: input=4776943 cached_input=4525824 output=23395 reasoning_output=10292; review-user-1: input=1673816 cached_input=1530368 output=11862 reasoning_output=9191; review-user-2: input=784124 cached_input=698752 output=7208 reasoning_output=4648; review-user-3: input=1110862 cached_input=995456 output=18710 reasoning_output=13370; user-1: input=4118316 cached_input=3804672 output=25302 reasoning_output=17981; user-2: input=1606403 cached_input=1482368 output=11491 reasoning_output=8048; user-3: input=3610144 cached_input=3335424 output=24041 reasoning_output=16727
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree post-commit parallel batch 6c run 6
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-6/parallel-current-6c-20260624T104153+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-6/parallel-current-6c-20260624T104153+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-6/parallel-current-6c-20260624T104153+0200-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	284	linearize reviewed patches
review-user-1	-	-	42	review user-agent
review-user-2	-	-	35	review user-agent
review-user-3	-	-	107	review user-agent
user-1	-	NeedsDecision	180	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	134	selected-agent-slash-execution
user-3	-	NeedsDecision	146	patch-chat-done-confirmation
```

## Recent Commits

```
403d644 ADD Vim visual selection in terminal panes so focused text can be copied
0dcde3d ADD review-completion confirmations so reviewed patch chats require a user decision
a97d254 ADD selected-agent backend slash commands so control input bypasses chat prompts
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
