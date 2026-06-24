# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:34:40+02:00
- duration_seconds: 2012
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
- web_ui_url: http://127.0.0.1:43427
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.tw1YPq
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 14
- changed_lines_added: 1302
- changed_lines_deleted: 55
- changed_lines_total: 1357
- token_usage: linearize: input=5659064 cached_input=5399168 output=28990 reasoning_output=10853; review-user-1: input=1021331 cached_input=972672 output=12413 reasoning_output=9291; review-user-2: input=2065668 cached_input=1904256 output=17151 reasoning_output=10334; review-user-3: input=832039 cached_input=746624 output=20385 reasoning_output=16343; user-1: input=866669 cached_input=765824 output=4226 reasoning_output=1781; user-2: input=2441638 cached_input=2346112 output=15942 reasoning_output=10525; user-3: input=1771990 cached_input=1603200 output=12534 reasoning_output=7690
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 12 run 3
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-3/parallel-current-12-20260624T010102+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-3/parallel-current-12-20260624T010102+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-3/parallel-current-12-20260624T010102+0200-r3-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	341	linearize reviewed patches
review-user-1	-	-	51	review user-agent
review-user-2	-	-	88	review user-agent
review-user-3	-	-	49	review user-agent
user-1	-	NeedsDecision	87	vim-visual-mode-for-panes
user-2	-	NeedsDecision	192	strict-slash-command-execution
user-3	-	NeedsDecision	199	review-done-feature-confirmation
```

## Recent Commits

```
d5594ae ADD vim visual selections so terminal panes can yank focused text
f462397 ADD review completion confirmation so clean patch chats can close or reopen
7327e4f ADD strict selected-agent slash commands so backend commands avoid prompt wrapping
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
