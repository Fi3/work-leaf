# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:28:46+02:00
- duration_seconds: 1658
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
- web_ui_url: http://127.0.0.1:40791
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.ZwN5LG
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1341
- changed_lines_deleted: 136
- changed_lines_total: 1477
- token_usage: linearize: input=4794514 cached_input=4481920 output=23681 reasoning_output=8869; review-user-1: input=927674 cached_input=869632 output=12854 reasoning_output=9890; review-user-2: input=489809 cached_input=399744 output=7234 reasoning_output=4420; review-user-3: input=612290 cached_input=521856 output=10546 reasoning_output=5349; user-1: input=1136036 cached_input=945152 output=14393 reasoning_output=11232; user-2: input=611969 cached_input=530816 output=5996 reasoning_output=3160; user-3: input=745911 cached_input=531456 output=11265 reasoning_output=7443
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 12 run 8
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-8/parallel-current-12-20260624T010102+0200-r8-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-8/parallel-current-12-20260624T010102+0200-r8-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-8/parallel-current-12-20260624T010102+0200-r8-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	265	linearize reviewed patches
review-user-1	-	-	44	review user-agent
review-user-2	-	-	39	review user-agent
review-user-3	-	-	87	review user-agent
user-1	-	NeedsDecision	103	vim-visual-mode-for-panes
user-2	-	NeedsDecision	110	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	116	patch-chat-feature-done-prompt
```

## Recent Commits

```
463cf3e ADD vim visual selections so terminal users can copy pane text
400aacd ADD review-done confirmation flow so completed patch chats close deliberately
f2b5fed ADD selected-agent slash command routing to keep provider commands off prompt sends
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
