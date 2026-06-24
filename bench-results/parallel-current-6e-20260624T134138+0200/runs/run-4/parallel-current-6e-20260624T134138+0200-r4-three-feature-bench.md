# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T13:41:38+02:00
- finished_at: 2026-06-24T14:15:41+02:00
- duration_seconds: 2043
- benched_binary_commit: 65157b102e1fdebca6264286fdcb6bf38b1d92c9
- benched_binary_dirty: no
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
- web_ui_url: http://127.0.0.1:44653
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.kGB0mR
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1408
- changed_lines_deleted: 81
- changed_lines_total: 1489
- token_usage: linearize: input=4570127 cached_input=4316416 output=24625 reasoning_output=8627; review-user-1: input=1079301 cached_input=937472 output=16647 reasoning_output=13225; review-user-2: input=980956 cached_input=849920 output=9030 reasoning_output=6231; review-user-3: input=796286 cached_input=620544 output=14311 reasoning_output=11231; user-1: input=1079758 cached_input=924160 output=4816 reasoning_output=2627; user-2: input=1189741 cached_input=969216 output=7981 reasoning_output=4617; user-3: input=1343260 cached_input=1150976 output=8938 reasoning_output=5563
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6e-20260624T134138+0200 run 4
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-4/parallel-current-6e-20260624T134138+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-4/parallel-current-6e-20260624T134138+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-4/parallel-current-6e-20260624T134138+0200-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	248	linearize reviewed patches
review-user-1	-	-	46	review user-agent
review-user-2	-	-	44	review user-agent
review-user-3	-	-	51	review user-agent
user-1	-	NeedsDecision	112	vim-visual-mode-for-panes
user-2	-	NeedsDecision	120	selected-agent-slash-commands
user-3	-	NeedsDecision	111	review-done-feature-confirmation
```

## Recent Commits

```
2cc18df ADD selected-agent slash command hooks so provider commands bypass prompts
213fb38 ADD terminal visual selection yanks so users can copy either pane
7829602 ADD review completion confirmation so clean reviews close patch agents intentionally
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
