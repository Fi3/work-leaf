# Three-Feature Smoke Bench

- result: fail
- workflow_result: fail
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-27T17:25:57+02:00
- finished_at: 2026-08-27T18:41:50+02:00
- duration_seconds: 4552
- benched_binary_commit: ef9528def88dc65b6a2dde81ef853b8e84b88525
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.149.1
- profiled_codex_sha256: 12e3d4a23b3260136b6c71ae1962d77cac0f8ae4853b4e0b1133a74e32224101
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
- temp_checkout: /tmp/work-leaf-fair-normal-pilot-rerun-ef9528d/work-leaf/work-leaf-3feature-bench.56izx2
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 3
- changed_files: 6
- changed_lines_added: 984
- changed_lines_deleted: 74
- changed_lines_total: 1058
- token_usage: total-workflow: input=851171 cached_input=749696 output=96574 reasoning_output=17507
- measurement_status: incomplete
- measurement_reason: invocation 00002493756997977008-4306 has no end metadata
- total_workflow_raw_tokens: 947745
- total_workflow_uncached_tokens: 198049
- code_quality: not run
- comment: busy stalled for more than 1800s without session state changes; user_count=3 terminal_users=2 done_users=2 ready_users=2 patch_agents_with_commits=2
- operator_notes: busy stalled for more than 1800s without session state changes; user_count=3 terminal_users=2 done_users=2 ready_users=2 patch_agents_with_commits=2
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/candidate/bin
- binaries_produced: none
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-rerun-20260827T151135Z/runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/patches/fail

## Measurement

Status: incomplete

- reason: invocation 00002493756997977008-4306 has no end metadata
- total-workflow input: 851171
- total-workflow cached input: 749696
- total-workflow uncached input: 101475
- total-workflow output: 96574
- total-workflow reasoning output: 17507
- total-workflow raw input plus output: 947745
- total-workflow uncached input plus output: 198049

## Sessions

```
review-user-1	-	-	10	review user-agent
review-user-2	-	-	8	review user-agent
user-1	-	NeedsDecision	49	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	38	slash-command-backend-response
user-3	WaitingForReply	-	203	when-review-process-done-patch-agent
```

## Recent Commits

```
2f35b04 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
4f1ac88 UPDATE apply user-agent patch from user-1
8f57c2e UPDATE apply user-agent patch from user-2
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
