# Three-Feature Smoke Bench

- result: pass
- workflow_result: pass
- bench_mode: work-leaf
- feature_schedule: concurrent
- started_at: 2026-08-29T13:06:38+02:00
- finished_at: 2026-08-29T14:08:27+02:00
- duration_seconds: 3672
- benched_binary_commit: d217f3803ac0f417671e27cc8fb18064ff0f4ea9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.150.1
- profiled_codex_sha256: 6fcecd4b2562c1ab4c781b98b3dc002e5c8468742b1cedf6be33701e811977b6
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
- temp_checkout: /home/user/.codex/work-leaf-corrected-control-runtime-20260829T091341Z/step4-control-001/work-leaf-3feature-bench.qiFmYm
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 9
- changed_lines_added: 1248
- changed_lines_deleted: 50
- changed_lines_total: 1298
- token_usage: total-workflow: input=10046818 cached_input=9353344 output=71949 reasoning_output=35366
- measurement_status: incomplete
- measurement_reason: controller usage row review-user-1 has no visible provider thread; controller usage row review-user-1 has no replayable streamed usage; controller usage for user-1 does not match replayed pre-interrupt usage; invocation 00002652116694817336-30656 has no end metadata; invocation 00002653630485487518-58166 has no end metadata; rollout: 1 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=88
- total_workflow_raw_tokens: 10118767
- total_workflow_uncached_tokens: 765423
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts
- observation: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts/observation
- machine_report: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts/report.json
- binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts/candidate/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- runner_binaries: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts/runner-bin
- patch_artifacts: /home/user/src/work-leaf/bench-results/efficiency-corrected-all-disabled-20260829T091341Z/runs/step4-control-001/corrected-study-step4-control-001-three-feature-bench-artifacts/patches/pass

## Measurement

Status: incomplete

- reason: controller usage row review-user-1 has no visible provider thread; controller usage row review-user-1 has no replayable streamed usage; controller usage for user-1 does not match replayed pre-interrupt usage; invocation 00002652116694817336-30656 has no end metadata; invocation 00002653630485487518-58166 has no end metadata; rollout: 1 rollout thread(s) share an observed cwd but are absent from process capture; interrupted provider turn has no complete usage: count=88
- total-workflow input: 10046818
- total-workflow cached input: 9353344
- total-workflow uncached input: 693474
- total-workflow output: 71949
- total-workflow reasoning output: 35366
- total-workflow raw input plus output: 10118767
- total-workflow uncached input plus output: 765423

## Sessions

```
linearize	-	-	188	linearize reviewed patches
review-user-1	-	-	94	review user-agent
review-user-2	-	-	50	review user-agent
review-user-3	-	-	111	review user-agent
user-1	-	NeedsDecision	269	vim-like-visual-pane-selection
user-2	-	NeedsDecision	66	slash-command-backend-response
user-3	-	NeedsDecision	120	feature-done-chat-confirmation
```

## Recent Commits

```
102f8e7 ADD terminal visual selections so pane text can be yanked
7e9747d ADD reviewed-chat completion confirmation so accepted feature work closes cleanly
b3f2905 ADD route selected-agent slash commands so backend replies stay in chat
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
