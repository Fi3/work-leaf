# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T02:38:32+02:00
- duration_seconds: 2051
- benched_binary_commit: d4cb33d9cae99387831c690ca3b5201450558634
- benched_binary_dirty: yes
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.141.0
- agent_model: gpt-5.5
- agent_model_source: codex config /home/user/.codex/config.toml
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: codex config /home/user/.codex/config.toml
- requested_agent_model: default
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: http://127.0.0.1:41785
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.3Taxmq
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1293
- changed_lines_deleted: 82
- changed_lines_total: 1375
- token_usage: linearize: input=5113128 cached_input=4924544 output=26483 reasoning_output=10347; review-user-1: input=1071362 cached_input=908032 output=13544 reasoning_output=10390; review-user-2: input=485295 cached_input=347392 output=8097 reasoning_output=5464; review-user-3: input=517226 cached_input=428672 output=7550 reasoning_output=5341; user-1: input=878272 cached_input=723584 output=10785 reasoning_output=6837; user-2: input=1657512 cached_input=1358080 output=12306 reasoning_output=7570; user-3: input=741082 cached_input=601216 output=14786 reasoning_output=10812
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6a run 5
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-5/parallel-current-6a-20260624T020421+0200-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-5/parallel-current-6a-20260624T020421+0200-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-5/parallel-current-6a-20260624T020421+0200-r5-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	319	linearize reviewed patches
review-user-1	-	-	45	review user-agent
review-user-2	-	-	50	review user-agent
review-user-3	-	-	36	review user-agent
user-1	-	NeedsDecision	138	vim-like-visual-mode-for-panes
user-2	-	NeedsDecision	107	strict-agent-slash-command-execution
user-3	-	NeedsDecision	153	patch-chat-done-confirmation
```

## Recent Commits

```
653d808 ADD terminal visual selection so focused panes can copy text
625dc1b ADD selected-agent slash command routing so backend commands run raw
e90f639 ADD clean-review feature closure prompts so users confirm completed patch work
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
