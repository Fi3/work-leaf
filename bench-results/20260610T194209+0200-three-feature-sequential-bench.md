# Three-Feature Direct Codex Bench

- result: pass
- bench_mode: sequential
- started_at: 2026-06-10T19:42:09+02:00
- finished_at: 2026-06-10T20:46:36+02:00
- duration_seconds: 3867
- benched_binary_commit: n/a-direct-codex-baseline
- bench_driver_commit: d04d3e3253b037e41289347b995371b869c39f2b
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-threads
- codex_version: codex_bin=/usr/bin/codex
codex-cli 0.138.0
- agent_model: gpt-5.5
- agent_model_source: codex sdk config/read
- requested_agent_model: default
- sdk_python: /home/user/src/work-leaf/target/work-leaf-codex-sdk-venv/bin/python
- no_read_permission: n/a-direct-codex
- web_ui_url: n/a-direct-codex
- read_permission_mode: direct Codex filesystem access (workspace-write)
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-sequential-bench.fBdycP
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- review_round_limit: 0
- commits_after_base: 3
- changed_files: 16
- changed_lines_added: 1549
- changed_lines_deleted: 125
- changed_lines_total: 1674
- token_usage: linearize: input=1462983 cached_input=1321472 output=7049 reasoning_output=2086; sequential-feature-1-fix-1: input=6530892 cached_input=6309504 output=37663 reasoning_output=17395; sequential-feature-1-implement: input=5433232 cached_input=5295232 output=35232 reasoning_output=16400; sequential-feature-1-review-1: input=930491 cached_input=832256 output=11452 reasoning_output=7512; sequential-feature-1-review-2: input=1137205 cached_input=1005056 output=14575 reasoning_output=10265; sequential-feature-2-fix-1: input=12788137 cached_input=12564608 output=39771 reasoning_output=17531; sequential-feature-2-implement: input=9785849 cached_input=9588224 output=32952 reasoning_output=14266; sequential-feature-2-review-1: input=933418 cached_input=804096 output=11374 reasoning_output=6500; sequential-feature-2-review-2: input=2003477 cached_input=1837312 output=16501 reasoning_output=9362; sequential-feature-3-fix-1: input=7070180 cached_input=6889984 output=28320 reasoning_output=12801; sequential-feature-3-implement: input=4587161 cached_input=4435840 output=21860 reasoning_output=10475; sequential-feature-3-review-1: input=474384 cached_input=416384 output=9870 reasoning_output=6418; sequential-feature-3-review-2: input=920071 cached_input=843136 output=14863 reasoning_output=9775
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: direct Codex sequential benchmark completed review, linearize, and final checks
- operator_notes: Direct Codex sequential baseline with persistent implementer/reviewer Codex chats after d04d3e3.
- artifacts: /home/user/src/work-leaf/bench-results/20260610T194209+0200-three-feature-sequential-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/20260610T194209+0200-three-feature-sequential-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/20260610T194209+0200-three-feature-sequential-bench-artifacts/patches/pass

## Recent Commits

```
4c92ab9 ADD reviewed-feature completion prompts to close finished chats
7e97147 ADD selected-agent backend commands to keep slash controls local
b9b8eea ADD pane-focused terminal controls for agent chat navigation
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
