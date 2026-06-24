# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T02:04:21+02:00
- finished_at: 2026-06-24T02:39:55+02:00
- duration_seconds: 2134
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
- web_ui_url: http://127.0.0.1:36601
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.L9C9io
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1620
- changed_lines_deleted: 123
- changed_lines_total: 1743
- token_usage: linearize: input=6332754 cached_input=5985408 output=27305 reasoning_output=10113; review-user-1: input=1305286 cached_input=1203840 output=18798 reasoning_output=15090; review-user-2: input=1099651 cached_input=964480 output=13792 reasoning_output=10241; review-user-3: input=762151 cached_input=560768 output=17772 reasoning_output=14435; user-1: input=2050397 cached_input=1787008 output=18890 reasoning_output=14879; user-2: input=827517 cached_input=693632 output=3317 reasoning_output=1208; user-3: input=2766797 cached_input=2573824 output=15339 reasoning_output=9624
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: current worktree parallel batch 6a run 3
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-3/parallel-current-6a-20260624T020421+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-3/parallel-current-6a-20260624T020421+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6a-20260624T020421+0200/runs/run-3/parallel-current-6a-20260624T020421+0200-r3-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	299	linearize reviewed patches
review-user-1	-	-	54	review user-agent
review-user-2	-	-	65	review user-agent
review-user-3	-	-	44	review user-agent
user-1	-	NeedsDecision	134	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	91	strict-agent-slash-commands
user-3	-	NeedsDecision	179	review-complete-feature-confirm
```

## Recent Commits

```
dd711e6 ADD reviewed-feature completion prompts to close or reopen patch-agent chats
0a83bd3 ADD selected-agent slash command dispatch to keep backend commands out of prompts
a991eb1 ADD terminal visual selection copy for focused panes
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
