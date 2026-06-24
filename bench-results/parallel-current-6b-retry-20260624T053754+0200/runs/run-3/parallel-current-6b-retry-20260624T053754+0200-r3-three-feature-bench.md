# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:11:11+02:00
- duration_seconds: 1997
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
- web_ui_url: http://127.0.0.1:44773
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.MtXQNI
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1198
- changed_lines_deleted: 132
- changed_lines_total: 1330
- token_usage: linearize: input=3658624 cached_input=3445248 output=18327 reasoning_output=6864; review-user-1: input=1322142 cached_input=1176704 output=23368 reasoning_output=19971; review-user-2: input=629032 cached_input=536704 output=4472 reasoning_output=1790; review-user-3: input=701716 cached_input=608512 output=10355 reasoning_output=6769; user-1: input=1233164 cached_input=1007232 output=7412 reasoning_output=3611; user-2: input=1476305 cached_input=1365120 output=11357 reasoning_output=5738; user-3: input=1664036 cached_input=1521664 output=15512 reasoning_output=11992
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: current worktree parallel batch 6b retry run 3 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-3/parallel-current-6b-retry-20260624T053754+0200-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-3/parallel-current-6b-retry-20260624T053754+0200-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-3/parallel-current-6b-retry-20260624T053754+0200-r3-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	213	linearize reviewed patches
review-user-1	-	-	51	review user-agent
review-user-2	-	-	43	review user-agent
review-user-3	-	-	46	review user-agent
user-1	-	NeedsDecision	162	vim-visual-mode-pane-copy
user-2	-	NeedsDecision	146	strict-agent-slash-commands
user-3	-	NeedsDecision	102	review-done-chat-highlight
```

## Recent Commits

```
f4ca196 ADD visual-mode clipboard yanks for focused terminal panes
34361a8 ADD review completion prompts so patch chats can close after review
ef85d1c ADD selected-agent command execution so slash commands bypass prompt wrapping
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
