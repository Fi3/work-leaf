# Pilot State

- Stage: frozen infrastructure ready for the real pilot.
- Provider workflows launched: 0.
- Completed Work Leaf runs: 0.
- Completed direct runs: 0.
- Local checks: root and observer format, Clippy, and test suites pass; scorer and shell tests pass.
- Scorer sanity: current implementation 3/3; fixed base 1/3 because literal `/status` already works there.
- Next action: run exactly one concurrent Work Leaf workflow and one direct sequential workflow together, then score both.
- Stop condition: write the provisional result and wait for user review before any larger study.
