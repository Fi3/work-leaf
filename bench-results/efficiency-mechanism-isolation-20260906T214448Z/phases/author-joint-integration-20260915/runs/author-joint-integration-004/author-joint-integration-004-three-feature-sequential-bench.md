# Three-Feature Direct Agent Bench

- result: fail
- workflow_result: fail
- bench_mode: sequential
- feature_schedule: sequential
- started_at: 2026-09-15T06:19:17+02:00
- finished_at: 2026-09-15T06:29:36+02:00
- duration_seconds: 619
- benched_binary_commit: n/a-direct-agent-baseline
- bench_driver_commit: 3c907f7266b63efa1558213f3e49587e999b899d
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-sessions
- agent_cli_version: agent_bin=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/infrastructure/provider/codex
codex-cli 0.153.4
- profiled_codex_sha256: 72d0880a05f410fe38b8390d65b2c50cbee7b8d97eb2e51e9fa6e3d3902deffc
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
- temp_checkout: /home/user/.codex/wl-author-integration.VrZvTe/author-joint-integration-004/work-leaf-3feature-sequential-bench.b8EKZW
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- review_round_limit: 0
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1360
- changed_lines_deleted: 81
- changed_lines_total: 1441
- token_usage: total-workflow: input=5327769 cached_input=5187712 output=25796 reasoning_output=8797
- measurement_status: complete
- measurement_reason: all observer capture checks passed
- total_workflow_raw_tokens: 5353565
- total_workflow_uncached_tokens: 165853
- code_quality: not run
- comment: failed to restore benchmark repository instructions
- operator_notes: failed to restore benchmark repository instructions
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts/candidate/bin
- binaries_produced: none
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/author-joint-integration-20260915/runs/author-joint-integration-004/author-joint-integration-004-three-feature-sequential-bench-artifacts/patches/fail

## Measurement

Status: complete

- reason: all observer capture checks passed
- total-workflow input: 5327769
- total-workflow cached input: 5187712
- total-workflow uncached input: 140057
- total-workflow output: 25796
- total-workflow reasoning output: 8797
- total-workflow raw input plus output: 5353565
- total-workflow uncached input plus output: 165853

## Recent Commits

```
3ad407b ADD review completion confirmation so reviewed patch chats close only after user approval
2fda4a6 UPDATE command-surface slash prompts so selected-agent commands reach the active chat
f26b6fa ADD terminal visual yank modes so pane text can be copied from the UI
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
