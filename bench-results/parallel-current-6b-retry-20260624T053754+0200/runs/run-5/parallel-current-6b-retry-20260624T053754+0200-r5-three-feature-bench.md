# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:12:29+02:00
- duration_seconds: 2075
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
- web_ui_url: http://127.0.0.1:37555
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.tXPLJF
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1413
- changed_lines_deleted: 84
- changed_lines_total: 1497
- token_usage: linearize: input=6307681 cached_input=6122880 output=25754 reasoning_output=8896; review-user-1: input=1305722 cached_input=1158912 output=15870 reasoning_output=11610; review-user-2: input=1385307 cached_input=1304448 output=9884 reasoning_output=5063; review-user-3: input=781148 cached_input=670592 output=9667 reasoning_output=7051; user-1: input=2405054 cached_input=2250368 output=11344 reasoning_output=6719; user-2: input=1316393 cached_input=1165952 output=11140 reasoning_output=7737; user-3: input=882650 cached_input=697600 output=8352 reasoning_output=6422
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 6b retry run 5 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-5/parallel-current-6b-retry-20260624T053754+0200-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-5/parallel-current-6b-retry-20260624T053754+0200-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-5/parallel-current-6b-retry-20260624T053754+0200-r5-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	362	linearize reviewed patches
review-user-1	-	-	53	review user-agent
review-user-2	-	-	62	review user-agent
review-user-3	-	-	45	review user-agent
user-1	-	NeedsDecision	146	vim-visual-mode-for-panes
user-2	-	NeedsDecision	117	strict-agent-slash-commands
user-3	-	NeedsDecision	86	review-done-feature-confirmation
```

## Recent Commits

```
5882060 ADD terminal visual selection so both panes can yank rendered text
936e2cf ADD reviewed-feature completion prompts so clean reviews require a close-or-continue decision
1028ed8 ADD selected-agent backend slash commands so status and fork bypass prompt sends
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
