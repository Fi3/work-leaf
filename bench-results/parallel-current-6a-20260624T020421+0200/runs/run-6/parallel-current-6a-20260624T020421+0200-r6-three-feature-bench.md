# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T02:46:09+02:00
- duration_seconds: 2508
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
- web_ui_url: http://127.0.0.1:32925
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.8QKC5k
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 15
- changed_lines_added: 1460
- changed_lines_deleted: 92
- changed_lines_total: 1552
- token_usage: linearize: input=4430434 cached_input=4238208 output=19206 reasoning_output=6566; review-user-1: input=1394332 cached_input=1182720 output=9253 reasoning_output=6197; review-user-2: input=733562 cached_input=585088 output=9172 reasoning_output=6255; review-user-3: input=711828 cached_input=628224 output=6879 reasoning_output=4356; user-1: input=3527936 cached_input=3290368 output=12871 reasoning_output=8143; user-2: input=1235086 cached_input=1074176 output=4629 reasoning_output=1441; user-3: input=590234 cached_input=494080 output=13801 reasoning_output=11386
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6a run 6
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-6/parallel-current-6a-20260624T020421+0200-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-6/parallel-current-6a-20260624T020421+0200-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-6/parallel-current-6a-20260624T020421+0200-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	202	linearize reviewed patches
review-user-1	-	-	44	review user-agent
review-user-2	-	-	48	review user-agent
review-user-3	-	-	47	review user-agent
user-1	-	NeedsDecision	193	vim-visual-mode-panes
user-2	-	NeedsDecision	114	strict-agent-slash-commands
user-3	-	NeedsDecision	86	review-done-feature-prompt
```

## Recent Commits

```
d062b3f ADD visual selection and yank support so terminal panes offer Vim-style copying
90ffe6b ADD review completion confirmation so patch chats close only after approval
8b9f212 ADD selected-agent slash execution so backend commands bypass prompt routing
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
