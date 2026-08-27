# Three-Feature Smoke Bench

- result: pass
- workflow_result: pass
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-27T14:52:08+02:00
- finished_at: 2026-08-27T15:20:38+02:00
- duration_seconds: 1676
- benched_binary_commit: ad016cae62f037928e88cb81d762a294ca9bcebe
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.149.1
- profiled_codex_sha256: d5cbb8ee9971e0a77415b5ff5902f9a321da113f49eb7628c962c85f2ef78e3b
- agent_model: gpt-5.5
- agent_model_source: WORK_LEAF_BENCH_MODEL
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: WORK_LEAF_BENCH_REASONING_EFFORT
- requested_agent_model: gpt-5.5
- requested_agent_reasoning_effort: xhigh
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: unavailable
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-fair-normal-pilot-runtime-ad016ca/work-leaf/work-leaf-3feature-bench.HiRlMA
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1130
- changed_lines_deleted: 82
- changed_lines_total: 1212
- token_usage: total-workflow: input=7937586 cached_input=7254400 output=58567 reasoning_output=30116
- measurement_status: complete
- measurement_reason: all observer capture checks passed
- total_workflow_raw_tokens: 7996153
- total_workflow_uncached_tokens: 741753
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/candidate/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/patches/pass

## Measurement

Status: complete

- reason: all observer capture checks passed
- total-workflow input: 7937586
- total-workflow cached input: 7254400
- total-workflow uncached input: 683186
- total-workflow output: 58567
- total-workflow reasoning output: 30116
- total-workflow raw input plus output: 7996153
- total-workflow uncached input plus output: 741753

## Sessions

```
linearize	-	-	202	linearize reviewed patches
review-user-1	-	-	54	review user-agent
review-user-2	-	-	31	review user-agent
review-user-3	-	-	77	review user-agent
user-1	-	NeedsDecision	95	vim-visual-mode-selection
user-2	-	NeedsDecision	81	slash-command-backend-routing
user-3	-	NeedsDecision	126	review-done-feature-confirmation
```

## Recent Commits

```
84550b1 ADD terminal visual selection for pane text yanking
9459610 ADD reviewed-feature completion confirmation for intentional closure
c93847c ADD selected-agent slash prompt routing for backend commands
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
