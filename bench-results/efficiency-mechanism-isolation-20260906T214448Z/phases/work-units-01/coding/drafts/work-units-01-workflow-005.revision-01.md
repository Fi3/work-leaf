# Workflow 005 classification revision provenance

The current review candidate is `work-units-01-workflow-005.revision-01.json`,
SHA256 `0aebe4480810dceb3d418be72fb2c0c5481c855a8f54602c9ba8ba9e737383cc`.
The original `work-units-01-workflow-005.json` remains retained at SHA256
`e9428e27ac96e2152aa184863d150ac77f4b543ea71ff43f0845b45eae60ffe5`.

Two independently reviewed evidence corrections define this revision:

- ACK7's repeated-cycle test at server physical JSONL line 31676 asserts the
  initial-cycle DONE? state, then after reopening asserts the second question,
  the reviewer-summary text, and finally the second-cycle DONE? state. The
  client-line-73 result fails at the summary assertion; the second-cycle DONE?
  assertion was not reached. Its rationale and prior-test locator distinguish
  these observations. The validation-repair label is unchanged.
- ACK9's client physical JSONL line 96 delivers `bundle-12.md`, not bundle 11.
  Its prior-state reference uses that delivered file, SHA256
  `6cf8cb47c031dc8902c79375cfdc2721837c47d21ba3c8210bedf32890efd409`,
  lines 817–824, 1112–1119 and 1197–1202. These contain the wrapped renderer,
  logical-row snapshot and logical-row styling supporting the same repair
  rationale. The review-repair label is unchanged.

The exact JSON delta consists of six values under
`/runs/work-units-01-workflow-005` (zero-based indexes):

- `/6/rationale`
- `/6/evidence/7/locator`
- `/8/evidence/6/locator`
- `/8/evidence/7/path`
- `/8/evidence/7/sha256`
- `/8/evidence/7/locator`

Recursive comparison verifies all other fields, ACK identities, order, labels,
rationales and evidence are identical. All revision evidence hashes and all
258 packet source hashes were rechecked. The original packet remains at SHA256
`5da59338e82c8152ed09ce648a689e053087534bb72ba2a0ecbea5b6c6528dda`.
Independent delta review is required before freeze. No retained capture,
frozen helper, protocol, earlier classification or later workflow was modified
or inspected for this correction; no generation or condition comparison ran.
