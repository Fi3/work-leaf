# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T10:41:53+02:00
- finished_at: 2026-06-24T11:28:31+02:00
- duration_seconds: 2798
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
- web_ui_url: http://127.0.0.1:39133
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.k7hajK
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 15
- changed_lines_added: 1415
- changed_lines_deleted: 118
- changed_lines_total: 1533
- token_usage: linearize: input=5422644 cached_input=5194368 output=22115 reasoning_output=7391; review-user-1: input=1239689 cached_input=1115392 output=9862 reasoning_output=6484; review-user-2: input=306780 cached_input=247936 output=4917 reasoning_output=3393; review-user-3: input=765153 cached_input=643712 output=14749 reasoning_output=10937; user-1: input=1083122 cached_input=977792 output=6322 reasoning_output=2764; user-2: input=806192 cached_input=696832 output=3214 reasoning_output=1261; user-3: input=2488857 cached_input=2160512 output=19887 reasoning_output=15511
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree post-commit parallel batch 6c run 4
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-4/parallel-current-6c-20260624T104153+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-4/parallel-current-6c-20260624T104153+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6c-20260624T104153+0200/runs/run-4/parallel-current-6c-20260624T104153+0200-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	231	linearize reviewed patches
review-user-1	-	-	49	review user-agent
review-user-2	-	-	41	review user-agent
review-user-3	-	-	66	review user-agent
user-1	-	NeedsDecision	104	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	80	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	220	patch-chat-review-done-confirm
```

## Recent Commits

```
d00ecda ADD resolved-review feature completion prompts with robust review markers
55f3477 ADD visual yank selections for both terminal panes
02e2941 ADD selected-agent slash commands through backend command execution
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
