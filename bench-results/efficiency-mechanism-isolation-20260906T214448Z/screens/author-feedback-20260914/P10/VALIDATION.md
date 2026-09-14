# Retained qualification and fix checks

Verified 2026-09-14 before publication/commit:

- cargo fmt: pass.
- cargo clippy --all-targets --all-features -- -D warnings: pass, no warnings.
- cargo test --all-targets --all-features: pass; existing explicitly ignored
  namespace/manual qualification cases retain their declared status.
- Five progress guards: 26 + 17 + 30 + 9 + 1 tests pass.
- P10/test_catalog_boundary.py: 8 pass; seven boundary regressions fail against
  the extracted pre-fix pooled-profile check before correction.
- P02/test_catalog_dispatch.py: 7 pass; its original fail-first receipt remains.
- P01/test_input_comparison.py: 2 pass.
- Source/report git diff --check: pass before artifact staging. Frozen captured
  prompts/proposals and command/test stdout contain their original trailing
  whitespace/blank EOF lines; the full artifact diff reports those. Their bytes
  are preserved for hash joins rather than normalized to satisfy a style check.

Task definitions, all prior main publication checkpoints and prior bug events
compare equal to their committed values. Only P10's completion is appended to
the main histories. Serialization escape style does not alter previous JSON
values. Research remains unfinished with 13 obligations.

The boundary verifier is an analysis-only linear replay over saved events and
explicit selected turn IDs. It does not change provider input, WL implementation
or a real-agent workflow; the actual P03/P04 native records are its replay test.
Its equality result concerns the catalog section, not all effective input or
semantic equivalence between different actions.

The agent-facing resume dispatcher has actual verification through P04's saved
host invocation-0002: exec --color never resume resumes the same native thread
and completes exit0. That verifies command dispatch only; the completed input
audit rejects corresponding historical catalog timing. No additional real-agent
diagnostic was spent for BUG005. P01's earlier exact initial-input diagnostic is
retained separately; it does not prove subsequent resume fidelity.

No normal WL/Rust code or public API changes belong to this research publication.
OpenAI Docs guidance informed the earlier local loader/setup checks; actual saved
native input, rather than documentation alone, decides experiment qualification.
