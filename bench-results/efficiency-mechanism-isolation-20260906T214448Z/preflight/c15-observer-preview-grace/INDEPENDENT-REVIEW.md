# Source-bound preview grace: independent review

Recorded at 2026-09-08 11:11:13 UTC. Scope: introduced observer behavior in
`/tmp/c15-observer-preview.0Wb07l`, its new tests, README and architecture description.
No implementation edits, Cargo runs, providers, private execution, saved-workflow extraction or
accounting replay were performed by this review.

Verdict: no remaining introduced source-correctness blocker. Real-agent grace-chain verification
remains an explicit readiness finding; this is not admission or a green real-agent result.

## Exact reviewed cut

The complete diff and new modules/tests were read. All 11 source/binary endpoints in
[VALIDATION.json](VALIDATION.json) independently match (`75efdc`, exit 0). That receipt has SHA-256
`fb7c8c997b368b58d8c164f2dde93a62250915e1813b2147ee08e16a067338c1`;
[VALIDATION.md](VALIDATION.md) has SHA-256
`b9b66d1211f0ab03b5a0266a03c6b2d5359bf61db19878b74a7344b41f4d8d47`.
The principal source identities are:

| Source | SHA-256 |
| --- | --- |
| `bench-observer/build.rs` | `201197313acd1871ef5d03741558ba6e9357ea2ee7b978b2d2ae05d4a86e5423` |
| `bench-observer/src/preview_grammar.rs` | `21baa1916a528c965439f1eba6be8529eea0501ed223ca51e2c3bd7ed9e12ff0` |
| `bench-observer/src/lib.rs` | `f7fd561875edd57913cdeb97c815e1ef353e107c367da3970d08687aa6e68ca1` |
| `bench-observer/src/preview_grammar_tests.rs` | `88876af6e4f38a43a1112ce63224305bff63b0b3e0e6f2a194ad872e5633f5b3` |

## Behavior and boundaries

- `build.rs` verifies both the whole private runtime source (`1700f26d…`) and exact parser
  slice (`1583e715…`), then compiles unchanged extracted bytes. The module does not maintain a
  second parser or change a public runtime/observer DTO. The extra build dependency uses the
  existing locked `sha2` version; `Cargo.lock` is unchanged.
- `run_captured_process` → `Enrollment::from_environment` requires the explicit slice digest,
  primary Work Leaf app-server, nonzero existing grace, and matching v6 activation/run manifest
  before spawning the child. Absent opt-in keeps the ordinary behavior and artifact shape.
  This manifest binding is not a substitute for independent runtime/config admission.
- `Enrollment::publish` uses exclusive, no-follow settings publication before start delivery.
  `finish` binds settings, start, forwarded requests and grace journal. `saved_mode` reads those
  closed identities, not the analyzer's environment; malformed opted-in evidence remains an
  analysis error. Original captures are not retroactively enrolled.
- `ProviderUsageGrace::observe_server_value` admits the exact complete accumulated preview to
  the existing state machine. Carried cumulative usage does not release the interrupt; fresh
  usage, resumed output, terminal completion and timeout retain their existing meanings. The
  configuration still determines the grace duration and resume policy; this module does not
  force a one-second setting or suppress resumed output.
- Live and offline completed-message accumulation preserves the two-newline join and strict
  aggregate-prefix rule. Earlier commentary, indentation, quoting/fences, malformed typed
  metadata or mixed trailing text do not become valid preview envelopes. This deliberately
  does not repair the separate cumulative-commentary/runtime integration limitation.
- `analyze_app_server` uses the saved mode only at controller delivery replay's directive
  boundary. Provider observations, cumulative arithmetic and missing-usage flags remain separate;
  the synthetic regression compares every provider observation, not just a final total.

## Checks and remaining qualification

The retained final tool output, inspected rather than rerun here, reports fmt `2557f5`, clippy
`b63e67`, and all-target/all-feature tests `85eb8c` → `bf5f9c`: 146 passed, 0 failed and one
environment-only child ignored at top level (invoked by its automatic parent). Sixteen new
automatic tests cover exact parser/source identity, actual environment gating, saved-environment
independence, create-new metadata, tampering, maximum envelope, default behavior and a real local
pipe's unchanged carried counters → fresh usage → byte-identical interrupt forwarding. The
retained REDs and final cut are distinguished in the owner's receipt.

Complexity finding: reparsing K accumulated completed items costs O(K × B), worst-case quadratic.
Both resulting docs explicitly flag it. The 8 MiB live envelope limit is not a total memory or
work bound, and existing offline message retention is not claimed bounded. No additional pairwise
turn/history scan is introduced by the metadata or saved-choice joins.

A separately authorized real configured-agent chain must still demonstrate actual preview
eligibility, captured grace ordering and forwarding under its frozen settings. Local pipes and
synthetic replay do not establish that result. Historical C15 skipped-grace boundaries, original
capture flags and existing study outcomes remain unchanged.
