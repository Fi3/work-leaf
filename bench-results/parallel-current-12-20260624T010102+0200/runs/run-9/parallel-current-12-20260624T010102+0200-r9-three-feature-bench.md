# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:27:24+02:00
- duration_seconds: 1576
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
- web_ui_url: http://127.0.0.1:42633
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.1TaCDF
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1283
- changed_lines_deleted: 34
- changed_lines_total: 1317
- token_usage: linearize: input=2744682 cached_input=2495744 output=16861 reasoning_output=6259; review-user-1: input=946152 cached_input=830848 output=11732 reasoning_output=8168; review-user-2: input=1031825 cached_input=918528 output=11444 reasoning_output=8383; review-user-3: input=872202 cached_input=832384 output=10034 reasoning_output=7844; user-1: input=864316 cached_input=721408 output=5412 reasoning_output=2715; user-2: input=1951311 cached_input=1792128 output=7894 reasoning_output=2675; user-3: input=1000777 cached_input=876800 output=3216 reasoning_output=1098
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 12 run 9
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-9/parallel-current-12-20260624T010102+0200-r9-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-9/parallel-current-12-20260624T010102+0200-r9-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-9/parallel-current-12-20260624T010102+0200-r9-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	195	linearize reviewed patches
review-user-1	-	-	65	review user-agent
review-user-2	-	-	50	review user-agent
review-user-3	-	-	41	review user-agent
user-1	-	NeedsDecision	94	vim-visual-mode-text-selection
user-2	-	NeedsDecision	122	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	79	patch-agent-done-confirmation
```

## Recent Commits

```
44f2dc3 ADD strict selected-agent slash commands so slash input targets backend commands
a983496 ADD vim visual copy for terminal panes so focused text can be selected
e444194 ADD review completion prompts so reviewed patch-agent chats close explicitly
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
