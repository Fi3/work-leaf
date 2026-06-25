# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-25T09:02:07+02:00
- finished_at: 2026-06-25T09:31:53+02:00
- duration_seconds: 1786
- benched_binary_commit: 01932712a913d15840ce27eeb67bae7d66b00b7b
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.142.1
- agent_model: gpt-5.5
- agent_model_source: codex config /home/user/.codex/config.toml
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: codex config /home/user/.codex/config.toml
- requested_agent_model: default
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: http://127.0.0.1:35583
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.kiBZCR
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1338
- changed_lines_deleted: 51
- changed_lines_total: 1389
- token_usage: linearize: input=2673242 cached_input=2537728 output=14673 reasoning_output=5157; review-user-1: input=1641566 cached_input=1559680 output=16872 reasoning_output=11927; review-user-2: input=868990 cached_input=805248 output=10011 reasoning_output=6625; review-user-3: input=895772 cached_input=801280 output=11780 reasoning_output=8164; user-1: input=1205356 cached_input=966016 output=7222 reasoning_output=4627; user-2: input=1112876 cached_input=927872 output=8854 reasoning_output=4932; user-3: input=2453187 cached_input=2177664 output=20276 reasoning_output=16279
- token_model_status: unavailable
- token_model_label: unavailable
- token_model_total: 0
- token_model_baseline_count: 0
- token_model_mean: 0
- token_model_stddev: 0
- token_model_delta_tokens: 0
- token_model_z: 0.00
- token_model_percentile: 0.00
- token_model_central95_low: 0
- token_model_central95_high: 0
- token_model_rerun: unknown
- token_model_comment: baseline-manifest.json not found
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: 3-run parallel bench batch; local codex --version reports codex-cli 0.142.1; user noted Codex 1,42.
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-2/parallel-current-3-codex142-20260625T090150+0200-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-2/parallel-current-3-codex142-20260625T090150+0200-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-2/parallel-current-3-codex142-20260625T090150+0200-r2-three-feature-bench-artifacts/patches/pass

## Token Model Fit

Status: unavailable

- total input+output: 0
- fitted baseline runs: 0
- fitted mean: 0
- fitted stddev: 0
- delta from mean: 0
- z-score: 0.00
- fitted percentile: 0.00%
- central 95% interval: 0 to 0
- rerun recommendation: unknown
- interpretation: baseline-manifest.json not found

## Sessions

```
linearize	-	-	193	linearize reviewed patches
review-user-1	-	-	58	review user-agent
review-user-2	-	-	70	review user-agent
review-user-3	-	-	64	review user-agent
user-1	-	NeedsDecision	103	vim-visual-mode-selection
user-2	-	NeedsDecision	179	strict-slash-command-execution
user-3	-	NeedsDecision	157	review-done-feature-confirmation
```

## Recent Commits

```
1c35bef ADD terminal visual selection so pane text can be yanked
2edf75a ADD reviewed-feature acknowledgement so completed patch chats close cleanly
f7fdc0b ADD selected-agent slash commands through backend command routing
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
