# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-25T09:02:07+02:00
- finished_at: 2026-06-25T09:32:03+02:00
- duration_seconds: 1796
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
- web_ui_url: http://127.0.0.1:42371
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.HehuFv
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1496
- changed_lines_deleted: 55
- changed_lines_total: 1551
- token_usage: linearize: input=4923976 cached_input=4717312 output=22682 reasoning_output=8345; review-user-1: input=1436477 cached_input=1330560 output=8413 reasoning_output=5154; review-user-2: input=495509 cached_input=410624 output=10543 reasoning_output=7868; review-user-3: input=993471 cached_input=886144 output=13633 reasoning_output=10831; user-1: input=366667 cached_input=267520 output=1709 reasoning_output=614; user-2: input=1767661 cached_input=1638016 output=13839 reasoning_output=9063; user-3: input=1898665 cached_input=1734912 output=19785 reasoning_output=15181
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
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-1/parallel-current-3-codex142-20260625T090150+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-1/parallel-current-3-codex142-20260625T090150+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-1/parallel-current-3-codex142-20260625T090150+0200-r1-three-feature-bench-artifacts/patches/pass

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
linearize	-	-	238	linearize reviewed patches
review-user-1	-	-	46	review user-agent
review-user-2	-	-	64	review user-agent
review-user-3	-	-	57	review user-agent
user-1	-	NeedsDecision	59	vim-visual-mode-for-panes
user-2	-	NeedsDecision	119	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	150	patch-chat-done-confirmation
```

## Recent Commits

```
17337d5 ADD terminal visual yanking so focused panes copy selections with Vim keys
05eb5dd ADD review-completion confirmation so clean patch agents close only after user approval
dca72be ADD selected-agent slash command dispatch so provider commands run as raw resumes
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
