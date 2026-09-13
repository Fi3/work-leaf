# Global-hunk pilot root prelaunch qualification

2026-09-12 23:20 UTC. One prospective full workflow; no generation before a separately
reviewed frozen manifest and root admission. The user's “finish it” selects this
pilot, not controls, old-run continuation or automatic replacement.

Root checks:
- Required cargo fmt, strict all-target/all-feature Clippy and full Cargo tests
  pass in one command, launch2e2b53/session76263, terminaleb8e62 exit0.
  Verbose tool output is truncated; no full disk log is claimed. Normal source,
  tests and Cargo inputs have no diff. No public API/UI/architecture change.
- Corrected host16 tests and retained real-process custody10 tests pass (eedfa8).
  That chained tool command later exits1 on an unrelated no-match rg query;
  both test commands themselves complete GREEN. The prior52 inherited behavior
  tests and byte-identical saved-proposal replay remain qualified as recorded
  in the host review. The old pre-whitespace AST assertion stays explicitly
  incompatible; it also fails on the unchanged old pilot host.
- Fresh runner22 tests pass (a3dbd9), with three identity-only RED cases and
  exact three-constant inverse to the original completion runner. Independent
  rerun and inverse also pass (6b32f3). All model operations are unchanged.
- Sampler qualification returns the expected154 fixture raw (6d2c12); independent
  command-body/embedded-code equality and sole phase substitution pass (6b32f3).
  Actual runtime/archive union and partial/error handling are unchanged.
- Disposable plain-pipe process session38702 receives real tool Ctrl-C and
  returns GLOBAL_HUNK_SIGINT_OK, exit0 (97b042), before any model launch.
  Independent reviewer separately verifies the same control path (c07124).
- Review catches launch-receipt persistence outside the cancellation guard.
  Two synthetic fault tests fail before the operator-only correction
  (MONITOR-RED.json). The saved actual loop then passes six synthetic cases:
  normal closure, threshold, sampler failure, gap over60seconds, launch-receipt
  failure and first control-tool failure. Control failures remain explicit;
  no successful stop is inferred if the tool does not confirm it.
  MONITOR-GREEN.json and test_monitor_loop.js preserve this coverage.
- Subscription wrapper login status reports Logged in using ChatGPT (acef92).
  Exact native CLI/wrapper remain pinned; normal home is /home/user, no
  CODEX_HOME override, credential copying, API key or provider fallback.
  [Official authentication guidance](https://learn.chatgpt.com/docs/auth)
  supports the forced ChatGPT restriction; it does not prove quota headroom.
- Global config hash is
  7c7e9ff3b8f184ed51abd79be8ecefd4bbeab9461b8e25a2b9bcd2b75ec02d9b;
  hash only, not copied. Fresh empty runtime root:
  /tmp/standalone-global-hunk-pilot-01.DEVbgh6s (f029c9).
  Filesystems have11GiB available in/tmp and583GiB in artifact storage (acef92).
- The18-test progress guard passes. Counts remain52/3/55 and the historical
  verified combined share is not established.

The corrected matcher is agent-facing experimental behavior. Synthetic/source
checks do not mark real-agent verification green; this single real pilot is
the intended verification attempt. Full workflow success and token-effect
interpretation require terminal evidence, not prelaunch confidence.
