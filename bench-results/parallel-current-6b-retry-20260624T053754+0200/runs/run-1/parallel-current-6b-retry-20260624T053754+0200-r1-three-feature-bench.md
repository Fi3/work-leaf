# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:13:16+02:00
- duration_seconds: 2122
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
- web_ui_url: http://127.0.0.1:43739
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.30mi2Y
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1485
- changed_lines_deleted: 118
- changed_lines_total: 1603
- token_usage: linearize: input=7556616 cached_input=7372800 output=27350 reasoning_output=9693; review-user-1: input=981377 cached_input=929664 output=11363 reasoning_output=8281; review-user-2: input=1665033 cached_input=1471872 output=11501 reasoning_output=7686; review-user-3: input=691770 cached_input=562688 output=11559 reasoning_output=8506; user-1: input=1446053 cached_input=1306752 output=4012 reasoning_output=1425; user-2: input=1962231 cached_input=1784960 output=20460 reasoning_output=13151; user-3: input=807558 cached_input=669824 output=10494 reasoning_output=6624
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6b retry run 1 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-1/parallel-current-6b-retry-20260624T053754+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-1/parallel-current-6b-retry-20260624T053754+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-1/parallel-current-6b-retry-20260624T053754+0200-r1-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	400	linearize reviewed patches
review-user-1	-	-	50	review user-agent
review-user-2	-	-	47	review user-agent
review-user-3	-	-	63	review user-agent
user-1	-	NeedsDecision	125	vim-visual-mode-copying
user-2	-	NeedsDecision	193	selected-agent-slash-commands
user-3	-	NeedsDecision	115	review-done-feature-confirmation
```

## Recent Commits

```
90daa65 ADD strict selected-agent slash commands through backend execution
a77aa7c ADD terminal visual copy selection for both panes
08914f3 ADD clean-review completion confirmation so users close accepted patches
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
