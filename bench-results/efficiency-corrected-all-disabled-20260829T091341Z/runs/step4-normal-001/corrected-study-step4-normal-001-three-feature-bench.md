# Three-Feature Smoke Bench

- result: fail
- workflow_result: fail
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-29T13:06:38+02:00
- finished_at: 2026-08-29T13:12:08+02:00
- duration_seconds: 330
- benched_binary_commit: d217f3803ac0f417671e27cc8fb18064ff0f4ea9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.150.1
- profiled_codex_sha256: 5f04d45dc720575a4d45aa32654af7a62fae38d5a5ffde26db099ee878be74ae
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
- temp_checkout: /home/user/.codex/work-leaf-corrected-control-runtime-20260829T091341Z/step4-normal-001/work-leaf-3feature-bench.vES2uo
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 0
- changed_files: 0
- changed_lines_added: 0
- changed_lines_deleted: 0
- changed_lines_total: 0
- token_usage: total-workflow: input=0 cached_input=0 output=0 reasoning_output=0
- measurement_status: incomplete
- measurement_reason: rollout: 3 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=3
- total_workflow_raw_tokens: 0
- total_workflow_uncached_tokens: 0
- code_quality: not run
- comment: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=0 done_users=0 ready_users=0 patch_agents_with_commits=0
- operator_notes: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=0 done_users=0 ready_users=0 patch_agents_with_commits=0
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts/candidate/bin
- binaries_produced: none
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-normal-001/corrected-study-step4-normal-001-three-feature-bench-artifacts/patches/fail

## Measurement

Status: incomplete

- reason: rollout: 3 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=3
- total-workflow input: 0
- total-workflow cached input: 0
- total-workflow uncached input: 0
- total-workflow output: 0
- total-workflow reasoning output: 0
- total-workflow raw input plus output: 0
- total-workflow uncached input plus output: 0

## Sessions

```
user-1	-	-	4	add-vim-visual-mode-both-panes
user-2	-	-	4	when-user-prompt-start-followed
user-3	-	-	4	when-review-process-done-patch-agent
```

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
