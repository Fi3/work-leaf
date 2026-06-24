# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:47:21+02:00
- duration_seconds: 1967
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
- web_ui_url: http://127.0.0.1:34347
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.OHSc94
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 15
- changed_lines_added: 1169
- changed_lines_deleted: 54
- changed_lines_total: 1223
- token_usage: linearize: input=5758581 cached_input=5479168 output=24160 reasoning_output=9142; review-user-1: input=748528 cached_input=701824 output=10596 reasoning_output=8050; review-user-2: input=703946 cached_input=600448 output=7793 reasoning_output=6137; review-user-3: input=1395732 cached_input=1238272 output=12293 reasoning_output=7545; user-1: input=272325 cached_input=192128 output=1676 reasoning_output=407; user-2: input=764289 cached_input=634496 output=11311 reasoning_output=6803; user-3: input=464521 cached_input=357376 output=8246 reasoning_output=5856
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 4
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-4/parallel-current-6d-20260624T121434+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-4/parallel-current-6d-20260624T121434+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-4/parallel-current-6d-20260624T121434+0200-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	258	linearize reviewed patches
review-user-1	-	-	43	review user-agent
review-user-2	-	-	36	review user-agent
review-user-3	-	-	88	review user-agent
user-1	-	NeedsDecision	81	vim-visual-mode-pane-copy
user-2	-	NeedsDecision	161	strict-agent-slash-command-exec
user-3	-	NeedsDecision	106	patch-chat-review-done-confirm
```

## Recent Commits

```
330a851 ADD vim visual selection so terminal panes can copy focused text
df0b3de ADD selected-agent slash command hook so provider commands bypass prompts
379d727 ADD reviewed-feature confirmation so accepted patch chats can close
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
