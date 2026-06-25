# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-25T09:02:07+02:00
- finished_at: 2026-06-25T09:42:17+02:00
- duration_seconds: 2410
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
- web_ui_url: http://127.0.0.1:33403
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.JBIcDR
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1317
- changed_lines_deleted: 88
- changed_lines_total: 1405
- token_usage: linearize: input=4686597 cached_input=4437632 output=22641 reasoning_output=9114; review-user-1: input=1018681 cached_input=941312 output=8536 reasoning_output=6136; review-user-2: input=420386 cached_input=345856 output=6026 reasoning_output=4228; review-user-3: input=1282098 cached_input=1209984 output=12448 reasoning_output=8064; user-1: input=581224 cached_input=474624 output=4881 reasoning_output=2867; user-2: input=1410838 cached_input=1249536 output=16536 reasoning_output=11380; user-3: input=1674456 cached_input=1512576 output=9638 reasoning_output=6484
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
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-3/parallel-current-3-codex142-20260625T090150+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-3/parallel-current-3-codex142-20260625T090150+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-3-codex142-20260625T090150+0200/runs/run-3/parallel-current-3-codex142-20260625T090150+0200-r3-three-feature-bench-artifacts/patches/pass

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
linearize	-	-	242	linearize reviewed patches
review-user-1	-	-	39	review user-agent
review-user-2	-	-	40	review user-agent
review-user-3	-	-	69	review user-agent
user-1	-	NeedsDecision	82	vim-like-visual-mode-panes
user-2	-	NeedsDecision	146	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	152	patch-chat-done-confirmation
```

## Recent Commits

```
e4c1333 ADD terminal visual selections so panes support Vim-style yanks
9fdaf25 ADD selected-agent slash command routing so backend commands target the active chat
759ea8e ADD review-completion confirmation so patch chats close deliberately
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
