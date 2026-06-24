# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:48:26+02:00
- duration_seconds: 2032
- benched_binary_commit: 65a71fe8bd1a9a2adc4173d0775526514c01a76e
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
- web_ui_url: http://127.0.0.1:32829
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.xtzuwc
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1329
- changed_lines_deleted: 99
- changed_lines_total: 1428
- token_usage: linearize: input=5564293 cached_input=5262592 output=25924 reasoning_output=11398; review-user-1: input=833795 cached_input=759296 output=10618 reasoning_output=8445; review-user-2: input=607231 cached_input=576000 output=10454 reasoning_output=7159; review-user-3: input=734541 cached_input=571520 output=8648 reasoning_output=6071; user-1: input=998186 cached_input=885376 output=6848 reasoning_output=4413; user-2: input=2198329 cached_input=1952384 output=13952 reasoning_output=9442; user-3: input=480433 cached_input=269184 output=3984 reasoning_output=1806
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 1
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-1/parallel-current-6d-20260624T121434+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-1/parallel-current-6d-20260624T121434+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-1/parallel-current-6d-20260624T121434+0200-r1-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	270	linearize reviewed patches
review-user-1	-	-	33	review user-agent
review-user-2	-	-	75	review user-agent
review-user-3	-	-	40	review user-agent
user-1	-	NeedsDecision	96	vim-visual-mode-selection
user-2	-	NeedsDecision	129	strict-slash-command-execution
user-3	-	NeedsDecision	78	patch-chat-done-confirmation
```

## Recent Commits

```
8258010 ADD vim-style visual selections so terminal panes can yank text
3823863 ADD strict selected-agent slash routing so provider commands bypass sends
945a664 ADD reviewed-feature completion prompts so users can close finished chats
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
