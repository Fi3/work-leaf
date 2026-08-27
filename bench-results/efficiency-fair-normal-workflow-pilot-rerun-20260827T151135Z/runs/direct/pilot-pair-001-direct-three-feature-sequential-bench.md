# Three-Feature Direct Agent Bench

- result: fail
- workflow_result: fail
- bench_mode: sequential
- feature_schedule: sequential
- started_at: 2026-08-27T17:25:57+02:00
- finished_at: 2026-08-27T17:32:00+02:00
- duration_seconds: 362
- benched_binary_commit: n/a-direct-agent-baseline
- bench_driver_commit: ef9528def88dc65b6a2dde81ef853b8e84b88525
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-sessions
- agent_cli_version: agent_bin=/usr/bin/codex
codex-cli 0.149.1
- profiled_codex_sha256: 645348cdade04d0e019719d4fef4fbd4d42ed4639f3346b98a7fe004768685ed
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
- temp_checkout: /tmp/work-leaf-fair-normal-pilot-rerun-ef9528d/direct/work-leaf-3feature-sequential-bench.QZhJEs
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- review_round_limit: 0
- commits_after_base: 0
- changed_files: 0
- changed_lines_added: 0
- changed_lines_deleted: 0
- changed_lines_total: 0
- token_usage: total-workflow: input=1914451 cached_input=1794432 output=17543 reasoning_output=11657
- measurement_status: complete
- measurement_reason: all observer capture checks passed
- total_workflow_raw_tokens: 1931994
- total_workflow_uncached_tokens: 137562
- code_quality: not run
- comment: sequential feature 1 did not reach a clean review
- operator_notes: sequential feature 1 did not reach a clean review
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/candidate/bin
- binaries_produced: none
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/patches/fail

## Measurement

Status: complete

- reason: all observer capture checks passed
- total-workflow input: 1914451
- total-workflow cached input: 1794432
- total-workflow uncached input: 120019
- total-workflow output: 17543
- total-workflow reasoning output: 11657
- total-workflow raw input plus output: 1931994
- total-workflow uncached input plus output: 137562

## Recent Commits

```
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
2964f73 UPDATE agent validation rules for Rust changes
5463c6b FIX process reviewer directives before findings to prevent patch-agent misrouting
3d46bdc UPDATE apply when focus is on patch from user-1
```

## Final Status

```

```
