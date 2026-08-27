# Point-7 Preflight

## Infrastructure Scope

The benchmark-only patch gives each top-level provider process its own writable `TMPDIR`. The Work
Leaf watchdog also treats growth in the captured app-server response stream as progress. No file
under `src/`, no frozen task text, and no quality-scoring fixture changed.

## Automated Checks

- The new regression failed before the patch because `bench-progress-common` did not exist.
- `cargo fmt` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo test --all-targets --all-features` passed.
- One unrelated context-bundle test failed once, then passed both in isolation and in the complete
  rerun. No related source was changed.

## Real Write Smoke

The bounded smoke passed on its first attempt. It used the profiled GPT-5.5/xhigh Codex wrapper, the
direct benchmark's `workspace-write` arguments, and a run-local provider temporary directory. Codex
created only `smoke.txt`, containing exactly `WORK_LEAF_REAL_AGENT_WRITE_OK` and one newline.

- exit code: 0
- Codex CLI: 0.149.1
- raw tokens: 56,918
- uncached tokens: 7,254
- recursive provider attempts: 0
- stderr bytes: 0
- JSONL SHA-256: `ec8823ac68a2f56b9aef0e94a3c9b1c439d22620bc403d07f0c1af0e2cb1b523`

The exact stream, empty stderr, and machine-readable result are under `preflight/`.
