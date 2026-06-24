# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T02:55:10+02:00
- duration_seconds: 3049
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
- web_ui_url: http://127.0.0.1:44795
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.LzeN1q
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1363
- changed_lines_deleted: 99
- changed_lines_total: 1462
- token_usage: linearize: input=3087499 cached_input=2860928 output=22785 reasoning_output=9992; review-user-1: input=1144986 cached_input=1096448 output=11033 reasoning_output=7909; review-user-2: input=2223560 cached_input=1955840 output=15826 reasoning_output=9671; review-user-3: input=1144883 cached_input=984576 output=17596 reasoning_output=13760; user-1: input=693968 cached_input=581248 output=13004 reasoning_output=10789; user-2: input=2143203 cached_input=1939328 output=7752 reasoning_output=4154; user-3: input=2530540 cached_input=2382208 output=14162 reasoning_output=9900
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6a run 2
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-2/parallel-current-6a-20260624T020421+0200-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-2/parallel-current-6a-20260624T020421+0200-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-2/parallel-current-6a-20260624T020421+0200-r2-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	255	linearize reviewed patches
review-user-1	-	-	51	review user-agent
review-user-2	-	-	99	review user-agent
review-user-3	-	-	58	review user-agent
user-1	-	NeedsDecision	86	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	156	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	215	patch-agent-review-done-confirm
```

## Recent Commits

```
4b6cf2e ADD review completion closure flow so accepted patch chats can be reopened deliberately
006c351 ADD vim visual selection for terminal panes so users can yank precise ranges
432a41d ADD strict selected-agent slash routing so chat slash prompts stay with the selected backend
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
