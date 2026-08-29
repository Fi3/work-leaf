# Three-Feature Smoke Bench

- result: pass
- workflow_result: pass
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-29T11:24:18+02:00
- finished_at: 2026-08-29T12:08:15+02:00
- duration_seconds: 2602
- benched_binary_commit: d217f3803ac0f417671e27cc8fb18064ff0f4ea9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.150.1
- profiled_codex_sha256: 6a5c60cd4e228750556e6e242b831d1f57b6798d730ddee19353cefb20bd7042
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
- temp_checkout: /home/user/.codex/work-leaf-corrected-control-runtime-20260829T091341Z/corrected-all-disabled-001/work-leaf-3feature-bench.O9diig
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 1252
- changed_lines_deleted: 87
- changed_lines_total: 1339
- token_usage: total-workflow: input=8843162 cached_input=8143360 output=78015 reasoning_output=47024
- measurement_status: incomplete
- measurement_reason: invocation 00002645284719394449-24372 has no end metadata; invocation 00002645914618370128-45126 has no end metadata; rollout: 2 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=103
- total_workflow_raw_tokens: 8921177
- total_workflow_uncached_tokens: 777817
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts/candidate/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-001/corrected-control-corrected-all-disabled-001-three-feature-bench-artifacts/patches/pass

## Measurement

Status: incomplete

- reason: invocation 00002645284719394449-24372 has no end metadata; invocation 00002645914618370128-45126 has no end metadata; rollout: 2 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=103
- total-workflow input: 8843162
- total-workflow cached input: 8143360
- total-workflow uncached input: 699802
- total-workflow output: 78015
- total-workflow reasoning output: 47024
- total-workflow raw input plus output: 8921177
- total-workflow uncached input plus output: 777817

## Sessions

```
linearize	-	-	209	linearize reviewed patches
review-user-1	-	-	103	review user-agent
review-user-2	-	-	117	review user-agent
review-user-3	-	-	121	review user-agent
user-1	-	NeedsDecision	103	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	133	slash-command-backend-routing
user-3	-	NeedsDecision	108	patch-agent-done-confirmation
```

## Recent Commits

```
4d3ebdd ADD selected-agent slash routing so chat commands reach the active backend session
2ef7ca1 UPGRADE terminal selection with visual yank support for pane text copying
83900ec ADD reviewed-feature completion prompt so clean reviews can close patch chats
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
