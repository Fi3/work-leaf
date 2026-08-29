# Three-Feature Smoke Bench

- result: pass
- workflow_result: pass
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-29T11:24:18+02:00
- finished_at: 2026-08-29T12:13:22+02:00
- duration_seconds: 2908
- benched_binary_commit: d217f3803ac0f417671e27cc8fb18064ff0f4ea9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.150.1
- profiled_codex_sha256: 1d4fc27d1cb8b57cc14adb87c0264bb7cbf79e1af6fbf7c85e46cd827af89d61
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
- temp_checkout: /home/user/.codex/work-leaf-corrected-control-runtime-20260829T091341Z/corrected-all-disabled-002/work-leaf-3feature-bench.XUO5i2
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1192
- changed_lines_deleted: 87
- changed_lines_total: 1279
- token_usage: total-workflow: input=7000409 cached_input=6331776 output=51014 reasoning_output=21299
- measurement_status: incomplete
- measurement_reason: controller usage row review-user-2 has no visible provider thread; controller usage row review-user-2 has no replayable streamed usage; controller usage for user-2 does not match replayed pre-interrupt usage; invocation 00002645862464471882-41615 has no end metadata; interrupted provider turn has no complete usage: count=63
- total_workflow_raw_tokens: 7051423
- total_workflow_uncached_tokens: 719647
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts/candidate/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/corrected-all-disabled-002/corrected-control-corrected-all-disabled-002-three-feature-bench-artifacts/patches/pass

## Measurement

Status: incomplete

- reason: controller usage row review-user-2 has no visible provider thread; controller usage row review-user-2 has no replayable streamed usage; controller usage for user-2 does not match replayed pre-interrupt usage; invocation 00002645862464471882-41615 has no end metadata; interrupted provider turn has no complete usage: count=63
- total-workflow input: 7000409
- total-workflow cached input: 6331776
- total-workflow uncached input: 668633
- total-workflow output: 51014
- total-workflow reasoning output: 21299
- total-workflow raw input plus output: 7051423
- total-workflow uncached input plus output: 719647

## Sessions

```
linearize	-	-	212	linearize reviewed patches
review-user-1	-	-	74	review user-agent
review-user-2	-	-	40	review user-agent
review-user-3	-	-	131	review user-agent
user-1	-	NeedsDecision	101	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	69	slash-command-backend-routing
user-3	-	NeedsDecision	93	review-done-feature-confirmation
```

## Recent Commits

```
c759e54 ADD terminal visual selections so users can yank focused pane text
82a764e ADD reviewed-feature confirmation so clean patch chats can close or continue
5267660 ADD selected-agent slash command routing so backend providers handle prompt commands
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
