# Three-Feature Direct Agent Bench

- result: fail
- workflow_result: fail
- bench_mode: sequential
- feature_schedule: sequential
- started_at: 2026-08-27T14:52:08+02:00
- finished_at: 2026-08-27T16:09:24+02:00
- duration_seconds: 4635
- benched_binary_commit: n/a-direct-agent-baseline
- bench_driver_commit: ad016cae62f037928e88cb81d762a294ca9bcebe
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-sessions
- agent_cli_version: agent_bin=/usr/bin/codex
codex-cli 0.149.1
- profiled_codex_sha256: d5cbb8ee9971e0a77415b5ff5902f9a321da113f49eb7628c962c85f2ef78e3b
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
- temp_checkout: /tmp/work-leaf-fair-normal-pilot-runtime-ad016ca/direct/work-leaf-3feature-sequential-bench.Uzex9B
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- review_round_limit: 0
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 1548
- changed_lines_deleted: 87
- changed_lines_total: 1635
- token_usage: total-workflow: input=22916620 cached_input=22186240 output=132361 reasoning_output=66595
- measurement_status: incomplete
- measurement_reason: rollout: thread 01a04347-1d78-7983-b668-330863a12a7a rollout usage does not match captured provider usage; rollout: thread 01a0435e-f293-7eb3-8e87-45583fec6864 rollout usage does not match captured provider usage; rollout: thread 01a04368-5811-7de2-84f9-f089563b738c rollout usage does not match captured provider usage; rollout: thread 01a04371-1671-75a2-879b-60aa26c2f697 rollout usage does not match captured provider usage; rollout: thread 01a0437a-2758-7eb3-a663-9ed64c096582 rollout usage does not match captured provider usage
- total_workflow_raw_tokens: 23048981
- total_workflow_uncached_tokens: 862741
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: final repository checks failed
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/candidate/bin
- binaries_produced: none
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/runs/direct/pilot-pair-001-direct-three-feature-sequential-bench-artifacts/patches/fail

## Measurement

Status: incomplete

- reason: rollout: thread 01a04347-1d78-7983-b668-330863a12a7a rollout usage does not match captured provider usage; rollout: thread 01a0435e-f293-7eb3-8e87-45583fec6864 rollout usage does not match captured provider usage; rollout: thread 01a04368-5811-7de2-84f9-f089563b738c rollout usage does not match captured provider usage; rollout: thread 01a04371-1671-75a2-879b-60aa26c2f697 rollout usage does not match captured provider usage; rollout: thread 01a0437a-2758-7eb3-a663-9ed64c096582 rollout usage does not match captured provider usage
- total-workflow input: 22916620
- total-workflow cached input: 22186240
- total-workflow uncached input: 730380
- total-workflow output: 132361
- total-workflow reasoning output: 66595
- total-workflow raw input plus output: 23048981
- total-workflow uncached input plus output: 862741

## Recent Commits

```
0c1472a ADD review completion prompts so users can close finished patch chats
307da34 ADD prompt slash routing so selected agents receive commands
662e114 ADD terminal visual yanks so users can copy pane text
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
