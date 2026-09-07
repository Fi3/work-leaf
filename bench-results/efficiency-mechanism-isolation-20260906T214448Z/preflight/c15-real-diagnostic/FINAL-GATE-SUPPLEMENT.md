# Final standalone gate and retained-output checks

Root's standalone `cargo fmt` returns exit 0 (`b632f0`, 0.172 s). Relevant
source/Cargo/architecture diff is clean (`35e772`), and all 159 final source pins
remain unchanged (`c0979e`). Root's standalone `cargo check --no-default-features`
also returns exit 0 without warnings (`e7ff7c`, 0.65-second Cargo output).

The owner's earlier compound formatting command did not separately retain its
first command's numeric status; its earlier standalone default check was not the
final cut. Those original qualifications remain in
`../c15-runtime-bridge/GATE-EXECUTION-RECEIPTS.json`, SHA-256
`7b64ef2acfbe9d4a172bb9dc1e7c5a85656ff4cce1cbb1f93a2b400e41222a63`.
These separate final root checks supply direct evidence without rewriting the
original implementation report or misattributing a later command's status.

Root independently parses the complete retained 56,052-character final test
output (`ca6673`): all 44 successful target summaries total 529 passed, zero
failed and 20 ignored. Final clippy and default-library completion receipts are
retained separately; default library summary is 50 passed, zero failures.
The full test suite is not repeated for this saved-output review.

This closes final local gate provenance. No real provider work is part of these
checks, and actual native/private semantic qualification remains separate.
