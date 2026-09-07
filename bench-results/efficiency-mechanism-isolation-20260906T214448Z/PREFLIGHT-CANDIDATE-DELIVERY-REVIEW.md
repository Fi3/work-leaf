# Candidate delivery verifier review

The introduced occurrence/replay checker and closed-run delivery auditor have no
remaining blocking findings at the source identities below. This is an independent
provider-free review of the three retained diagnostics, not an additional diagnostic,
benchmark observation, accounting result or causal estimate.

## Reviewed identities and boundaries

| Source | SHA-256 |
| --- | --- |
| `tests/bench_candidate_subscription_smoke.rs` | `f04b2eeac0128871ee0dd351ff36eb821bfc0128d1512b8f7dea2318e7bfc4f7` |
| `audit_candidate_delivery.py` | `ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1` |
| `test_audit_candidate_delivery.py` | `c5ae757165a23603fe1186cc76365444cbbce042e387b93f9b812e20ce348454` |

The review covers `match_trace_requests`, `closed_frame_bytes`,
`verify_capture_and_trace`, `replay_closed_candidate_handoffs`, and the new
auditor's `validate_event`, `audit_delivery`, `Sources`, `audit_sources` and
`write_new`. The candidate protocol, architecture and operator policy describe
benchmark-only interventions, unchanged normal provider behavior and separate
delivery/accounting evidence. No public API or runtime ownership change belongs
to this verifier delta; existing v1–v3 helpers and reports remain untouched.

The Rust checker establishes policy-owned agent/thread associations and consumes
each exact `(thread, selected prompt)` occurrence once, in order. Its fixed site
census requires all planned instrumented requests; only the declared final reviewer
recheck is uninstrumented. Original/forwarded requests, typed successful replies
and completed public user-item identities remain separate checks. Each thread's
last accepted turn requires its own terminal receipt. A recorded interrupt must be
forwarded and acknowledged. The offline entrypoint constructs no backend and writes
no artifacts.

The Python auditor exact-compiles its two hash-pinned predecessor sources and uses
only their occurrence/validation primitives, not their phase inference or accounting
entrypoints. It checks factor selection, UTF-8 ranges, unchanged bytes outside owned
components, actual held-repeat body identities, latest owned-launch request resupply,
typed RPC/turn identities, per-thread order, and bidirectional accepted/public-user
coverage. Identical prompt text is not treated as a globally unique identity.
Auxiliary title inputs remain explicit; native tool execution is not inferred from
the diagnostic's outer-turn count.

The closed-source wrapper requires expected source digests before reads, rehashes
sources after replay, checks the complete supplied observer app-server census against
closed start/end records and stream digests, and retains terminal failures. Input
and runtime source identities are explicit; the CLI pins its input manifest hash.
Partial physical JSONL records, changed sources, symlinks, missing captures and
nonterminal workflow receipts fail closed. Output uses exclusive create-new mode.

## Findings resolved with RED/GREEN fixtures

- Closed replay must not use the permissive live-poll reader: complete closed
  streams and traces reject incomplete or malformed final frames. Live polling
  retains its original behavior.
- C24 resupply counts include the validated `changed` field and use the latest
  preceding owned policy, not any older launch request.
- Reviewer source context legitimately contains additional `Agent-ID` metadata.
  Global delimiter uniqueness is not an ownership proof. Exact owned launch-footer
  suffix validation relies on the pinned renderer/registry semantics and preserves
  copied request text without parsing its metadata as a new owner.
- Different followup texts must preserve captured order too; exact user text is
  checked for every accepted turn, including uninstrumented followups.
- Public-message hashing occurs once per item. The former per-directive full-message
  hash would have introduced quadratic work; that finding is closed. Snapshot sorting
  is O(n log n). One remaining metadata cost is flagged under the repository review
  rule: the latest-launch check encodes its full policy again for each resupply,
  costing O(F × P) for F fixes sharing a P-byte policy, quadratic in a worst-case
  jointly growing input family. It does not change correctness, generation or
  accounting; caching that already validated byte string would remove the repeated
  encoding. The Rust diagnostic's repeated scans are bounded by six starts.

## Independent automatic and actual-source checks

`cargo test --features bench-experiments --test bench_candidate_subscription_smoke`
passed **8 automatic tests**, with the real-provider and explicit offline-replay
entrypoints intentionally ignored by this command.
`python3 -B -m unittest test_audit_candidate_delivery.py` passed **15 tests**.
Only owner-added tests were reviewed; no committed test or owner source was edited
by this reviewer.

The three actual `DELIVERY-AUDIT-INPUT.json` files were read under their hashes,
and `audit_sources` was independently executed from the exact helper bytes above.
Each fresh result equals its saved `DELIVERY-AUDIT.json` in full, has status
`available` and errors `[]`, and rechecks 18 source identities including its input
manifest. No report was rewritten.

| Retained diagnostic | Input SHA-256 | Saved output SHA-256 | Original exit retained |
| --- | --- | --- | --- |
| `preflight/candidate-v4-repeat-01` | `af308de8d3069d39ddb1e11cb1b9e762bc8918fbe3bd385df9fef4674cbe0d4e` | `6708d0dd9113db1f90e54fdb2e0bbdbc355a4107368a57dfec9355ec686842a6` | 0 |
| `preflight/candidate-v4-format-01` | `6d319a43acc953cbe9d7e5731c00e6939d22f1b90bea3c46048db6e9569969dd` | `add34f25a8b2d78610f1ecdefa00314dbd2af03d2436778573f49c962290dcf1` | 0 |
| `preflight/candidate-v4-followup-01` | `cb3b3eb06cd66bed5cbff15e8f0e5edb03b19fdc5c480e7fe627d7036c196353` | `cf9279d3b99f7640ea7d592c10122ac9cd4291f1be3996aad98003d00454f40e` | 101 |

C24's original terminal receipt hash is
`c46107709e512a58148d8eeaf849918f5123e8f78563b342b1758e4870f5a382`.
Its actual client lines C6/RPC string `5` and C12/RPC string `11` share ACK SHA
`b727985795aad66d583126e71729b092b6742bcc1249ec5e0eaba423dc4dea9a`,
but match distinct accepted turns and public user items in the same author thread:
S74 → turn `01a07bb8-f351-74f2-98cf-64c8fbcf65aa` / user S78;
S207 → turn `01a07bb9-28f9-7ba0-9a7e-17e7a0e673ee` / user S211.
The original failed global-text-uniqueness assertion is therefore separate from
the two valid delivered occurrences. The new audit retains both ACKs and the one
actual request resupply; it does not relabel the original process successful.

## Interpretation limits

The result fields named `native_user_item_id` and `native_user_line` identify
**public app-server user items**, not native rollout items. Actual native rollout
joins remain a separate audit; those namespaces must not be equated. Format
witnesses are public directive text, not independently accepted patches/commits.
Runtime source pins do not alone prove compiled-binary admission, global-config
integrity, complete physical model-call inventory or complete token accounting.
Those gates remain separate requirements under the candidate protocol.

Original diagnostics, incomplete usage, failed receipts and archive-error records
remain retained. This review inspects no private reasoning, launches no provider,
adds no controls, analyzes no parked-read costs and makes no percentage claim.
