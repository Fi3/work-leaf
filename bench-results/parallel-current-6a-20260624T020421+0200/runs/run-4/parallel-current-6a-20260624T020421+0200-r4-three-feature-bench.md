# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T02:40:17+02:00
- duration_seconds: 2156
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
- web_ui_url: http://127.0.0.1:35405
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.6qBft1
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1477
- changed_lines_deleted: 169
- changed_lines_total: 1646
- token_usage: linearize: input=8761554 cached_input=8533120 output=25270 reasoning_output=6318; review-user-1: input=800213 cached_input=639104 output=7191 reasoning_output=4950; review-user-2: input=594508 cached_input=396288 output=7393 reasoning_output=4577; review-user-3: input=630930 cached_input=507520 output=8247 reasoning_output=5247; user-1: input=595693 cached_input=544768 output=5119 reasoning_output=1335; user-2: input=980620 cached_input=884992 output=6865 reasoning_output=4123; user-3: input=786534 cached_input=631296 output=5646 reasoning_output=1484
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6a run 4
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-4/parallel-current-6a-20260624T020421+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-4/parallel-current-6a-20260624T020421+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-4/parallel-current-6a-20260624T020421+0200-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	351	linearize reviewed patches
review-user-1	-	-	32	review user-agent
review-user-2	-	-	33	review user-agent
review-user-3	-	-	57	review user-agent
user-1	-	NeedsDecision	100	vim-visual-mode-for-panes
user-2	-	NeedsDecision	79	strict-slash-command-execution
user-3	-	NeedsDecision	134	review-done-agent-prompt
```

## Recent Commits

```
315df3e ADD terminal visual selection and clipboard yanking for pane text
2a6461d ADD prompt users to close cleanly reviewed patch-agent work
c6d2715 ADD route selected-agent slash commands through backend command hooks
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
