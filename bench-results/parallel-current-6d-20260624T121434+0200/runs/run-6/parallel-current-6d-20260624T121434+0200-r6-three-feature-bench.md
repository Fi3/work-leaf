# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:48:59+02:00
- duration_seconds: 2065
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
- web_ui_url: http://127.0.0.1:46167
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.hONyYk
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1481
- changed_lines_deleted: 137
- changed_lines_total: 1618
- token_usage: linearize: input=3164228 cached_input=2914304 output=18178 reasoning_output=6755; review-user-1: input=562319 cached_input=468352 output=8385 reasoning_output=5977; review-user-2: input=525355 cached_input=449024 output=6703 reasoning_output=2917; review-user-3: input=1601342 cached_input=1393280 output=11361 reasoning_output=8240; user-1: input=972335 cached_input=830720 output=5758 reasoning_output=3402; user-2: input=597602 cached_input=358656 output=8639 reasoning_output=5782; user-3: input=1326793 cached_input=1093248 output=20877 reasoning_output=16471
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 6
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-6/parallel-current-6d-20260624T121434+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-6/parallel-current-6d-20260624T121434+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-6/parallel-current-6d-20260624T121434+0200-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	218	linearize reviewed patches
review-user-1	-	-	43	review user-agent
review-user-2	-	-	78	review user-agent
review-user-3	-	-	55	review user-agent
user-1	-	NeedsDecision	107	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	107	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	178	review-done-feature-confirmation
```

## Recent Commits

```
07ba6f5 ADD review-completion confirmation so patch agents close only after yes/no input
d1968f8 ADD Vim visual selection and yanking for both terminal panes
6ba564d ADD strict selected-agent slash command routing to keep provider commands out of chat prompts
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
