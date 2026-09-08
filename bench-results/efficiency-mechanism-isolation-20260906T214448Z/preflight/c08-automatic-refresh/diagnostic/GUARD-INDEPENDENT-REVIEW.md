# Independent C08 diagnostic guard review

Verdict: PASS for the local guards and locked dependency graph after the
repeated-ACK correction. No remaining introduced blocker was found. This review
does not independently review its author's wrapper implementation or establish
real-agent, native-delivery, semantic, accounting or admission qualification.

## Exact scope

The complete root-owned `guards.rs`, five `guard_tests.rs` cases, private Cargo
manifest/lockfile, current proposed scope, and unchanged D3 `Budget` and
`settled_turns` dependencies were read. No Cargo or provider was run during this
independent review.

| Source | SHA-256 |
| --- | --- |
| `guards.rs` | `3bc5cbda8042aab4b70d92d6bc9cd04385180b657317c41d8d191fee46e62785` |
| `guard_tests.rs` | `32b919a61fdcda365e7dbc33c97991841b0ce200dc332c1267ef9fea14108d3b` |
| `Cargo.toml` | `dcb845e0aa5999bd154df598896f01b2e5498efadc266c8e7b7e45b553fcd272` |
| `Cargo.lock` | `524d78fb7d362ca19a29eb23a2507e43d010177060e4bd6d9b2d78fcf0a77277` |
| D3 `guards.rs` | `92263d07347251f50b53fcd17c0d58bab0855062ab9acf8537a610c8d0f245f7` |
| D3 `Cargo.toml` | `974ccf2dc6b9df6968ec32c9773d1fd5c020178739c3f89e047bf3be96700aaf` |
| D3 `Cargo.lock` | `f702c77dee349e5ffdf48f8da8bd719bcc7cb48b2e4df93391f89695c1230279` |

Read-only structural check `084779` verifies all 31 D3 package records, versions,
checksums and dependency lists unchanged. The sole added record is the local C08
diagnostic package. Both `work-leaf` path dependencies resolve to the intended
detached repository, and the D3 dependency resolves to its unchanged sibling
source. Lockfile equality is not a claim that the local Work Leaf source equals
the older D3 runtime; the intended C08 runtime needs its own final source pins.

## Finding and closure

The first cut required globally unique full ACK/result text. Two ordinary
accepted repairs to the same file can produce identical ACK inputs on distinct
calls, causing a valid six-call chain to fail. Root reports actual regression
RED `9b5586`, then five guard tests GREEN `7599c5`.

The reviewed correction indexes full input strings into occurrence queues,
consumes each selected occurrence once, requires increasing call order, and
rejects unused occurrences of joined text. The new regression proves two ACKs
map to distinct ordinals, while omitting the second trace row fails. Ownership,
full-byte equality, error rejection and later successful-check requirements
remain intact. This closes the reported P2 without weakening duplicate identity
or missing-evidence handling for that chain.

## Guard boundaries

`initial_read_owned` requires the exact complete ordinary read and one unindented
standalone read directive; fenced, conflicting and duplicate directives fail.
`recovery_chain` requires 5–8 successful, consecutively numbered, same-author
calls with one launch, one exact initial read, and one selected eligible refresh.
Ordered UTF-8-valid component ranges preserve all intervening and trailing bytes.
Exactly one current-body component must equal the held stimulus source and identify
the changed, available `src/lib.rs` snapshot. Unknown component identities fail.

Later trace ACK/result inputs must match complete local inputs. A new ACK resets
earlier validation; only the exact focused command with status zero and no
timeout header can qualify the last call. Its final reply must contain the sole
unindented DONE directive. Actual CommandChat processing is an additional harness
predicate, not inferred by this pure function. D3 transport settlement remains
latest-turn transport evidence, not complete native/public lifecycle proof.

There is no new unbounded quadratic join. Input occurrence queues are indexed;
component comparison walks ordered ranges, and remaining whole-candidate scans
are bounded by eight calls. Retained trace size is not thereby a memory bound.
The guard explicitly returns false native/semantic claims; source authenticity,
all accepted/public/native inputs, command execution, complete lifecycle and
actual test purpose remain mandatory separate postcapture evidence. Real-agent
verification and exact source/build/configuration admission are still pending.

Only this review record was written in the independent task. Wrapper RED/GREEN
results belong to the separate implementation handoff, not this verdict.
