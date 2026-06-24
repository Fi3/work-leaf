# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:21:21+02:00
- duration_seconds: 2607
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
- web_ui_url: http://127.0.0.1:36387
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.Jl3UKz
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1272
- changed_lines_deleted: 93
- changed_lines_total: 1365
- token_usage: linearize: input=3452774 cached_input=3282688 output=20837 reasoning_output=8083; review-user-1: input=482110 cached_input=387712 output=5285 reasoning_output=4329; review-user-2: input=966754 cached_input=894080 output=8398 reasoning_output=5232; review-user-3: input=1709285 cached_input=1542016 output=15782 reasoning_output=9322; reviewer-1: input=664351 cached_input=535424 output=10179 reasoning_output=6913; user-1: input=1395202 cached_input=1271296 output=7068 reasoning_output=4047; user-2: input=1441976 cached_input=1321728 output=15598 reasoning_output=9541; user-3: input=303939 cached_input=198400 output=6691 reasoning_output=5058
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6b retry run 6 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-6/parallel-current-6b-retry-20260624T053754+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-6/parallel-current-6b-retry-20260624T053754+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-6/parallel-current-6b-retry-20260624T053754+0200-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	197	linearize reviewed patches
review-user-1	-	-	23	review user-agent
review-user-2	-	-	48	review user-agent
review-user-3	-	-	98	review user-agent
reviewer-1	-	-	47	agent
user-1	-	NeedsDecision	105	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	167	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	69	review-done-patch-chat-prompt
```

## Recent Commits

```
13d06be ADD terminal visual selections so pane text can be yanked
8ec68d1 ADD selected-agent backend command routing so slash commands stay raw
a85d24a ADD review completion confirmation so terminal users close clean patch chats
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
