# Normal Work Leaf observer verification

This bounded diagnostic exercises the unchanged Work Leaf directive detector and Codex backend,
with the observer's response-usage metadata opt-in. It is not a three-feature benchmark observation.
The saved native-tool handoff probe is a different scenario and is not reused as this result.

## Configuration

- Existing ChatGPT subscription login, confirmed by `codex login status` before launch.
- Installed `/usr/bin/codex` 0.153.4, GPT-5.5, `xhigh`; no model substitution.
- `subscription-codex` strips API-key and alternate-endpoint overrides and forces ChatGPT login.
  Credentials remain in the existing home; none are copied into the repository or scratch directory.
- Read-only sandbox; an empty scratch project; normal restricted-agent prompt policy.
- Observer raw-response metadata opt-in enabled, 1,000 ms pre-forward grace, default `forward` on
  resumed output. No native tools, protocol redesign, or added cancellation grace.
- Two model turns maximum in one provider thread. First: a complete Work Leaf directive with
  requested continued output, handled by the product detector. Second: raw same-session follow-up
  to normal completion. A test-local call budget rejects additional launch/send requests.
- Overall 180-second bound with process shutdown. The observer's captured app-server child is
  stopped before backend shutdown so its proxy can finalize capture metadata.

## Admission and report

The ignored `tests/observer_subscription_smoke.rs` scenario is run only with explicit activation.
Record its exact command, exit status, elapsed time, original/forwarded requests, grace decisions,
response identities and usage, cumulative totals, and unresolved interruption count. Request
rewrites may only affect initialization notification capabilities and thread-start metadata opt-in.

A working capture requires observed raw completion metadata, valid unique-response arithmetic,
same-thread launch/follow-up, original interrupt bytes, unchanged grace policy, and finalized
capture inventory. A working capture is **not** proof that cancelled responses have usage. Report
unresolved tails even if all completed raw records reconcile with cumulative totals.

No full benchmark is launched if this diagnostic leaves the accounting precision problem
unresolved. Do not repeat it for a favorable accounting outcome or extend its wait after seeing
missing usage. A local setup/authentication failure is recorded without switching to API keys.
