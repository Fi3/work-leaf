# Three-Feature Direct Agent Bench

- result: pass
- workflow_result: pass
- bench_mode: sequential
- feature_schedule: sequential
- started_at: 2026-08-29T13:06:38+02:00
- finished_at: 2026-08-29T14:19:41+02:00
- duration_seconds: 4376
- benched_binary_commit: n/a-direct-agent-baseline
- bench_driver_commit: d217f3803ac0f417671e27cc8fb18064ff0f4ea9
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-sessions
- agent_cli_version: agent_bin=/usr/bin/codex
codex-cli 0.150.1
- profiled_codex_sha256: 230ada04d4c91260b82e0d82f3f119faaa1adfd1f93065422bf92aca10b60611
- agent_model: gpt-5.5
- agent_model_source: WORK_LEAF_DIRECT_BENCH_MODEL
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT
- requested_agent_model: gpt-5.5
- requested_agent_reasoning_effort: xhigh
- no_read_permission: n/a-direct-agent
- web_ui_url: n/a-direct-agent
- read_permission_mode: direct agent filesystem access
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /home/user/.codex/work-leaf-corrected-control-runtime-20260829T091341Z/step4-direct-001/work-leaf-3feature-sequential-bench.6Ki9ZL
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- review_round_limit: 0
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1421
- changed_lines_deleted: 138
- changed_lines_total: 1559
- token_usage: total-workflow: input=35255005 cached_input=34059520 output=199601 reasoning_output=90293
- measurement_status: complete
- measurement_reason: all observer capture checks passed
- total_workflow_raw_tokens: 35454606
- total_workflow_uncached_tokens: 1395086
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: direct codex sequential benchmark completed review, linearize, and final checks
- operator_notes: direct codex sequential benchmark completed review, linearize, and final checks
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/candidate/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-direct-001/corrected-study-step4-direct-001-three-feature-sequential-bench-artifacts/patches/pass

## Measurement

Status: complete

- reason: all observer capture checks passed
- total-workflow input: 35255005
- total-workflow cached input: 34059520
- total-workflow uncached input: 1195485
- total-workflow output: 199601
- total-workflow reasoning output: 90293
- total-workflow raw input plus output: 35454606
- total-workflow uncached input plus output: 1395086

## Recent Commits

```
2039185 ADD review completion decisions so clean patches can close or reopen chats
43df5d4 ADD selected-agent slash routing so command prompts can send agent commands
075c4fc7 ADD terminal visual selection so users can copy pane text
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
