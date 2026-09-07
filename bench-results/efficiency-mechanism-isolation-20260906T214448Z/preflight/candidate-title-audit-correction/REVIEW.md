# Independent title-delivery correction review

The separately versioned offline title verifier has no blocking finding. Review
is limited to the introduced first-title exception and its source-bound wrapper;
it does not certify token accounting, native rollout membership, configuration
trust, binary admission or candidate outcomes.

## Reviewed identities

| Artifact | SHA-256 |
| --- | --- |
| `audit_title_delivery.py` | `e8b71f024e8035b5c034e8cf16f092b61fef3944bef93a0f7eddc252c84a6e84` |
| `test_title_delivery.py` | `0aa501710262fda12952f996e630f79a31953d16ae38a9bd9195c66206e149f5` |
| `README.md` | `bbc4623c7162e8ab63b8780f740c1b24142cec0858d3e06b0341176036ae49e6` |
| Retained original auditor | `ab9570bb0e87674fa738aa0dc6e51702ef7ac6ab97b17092d1f546a60b3889e1` |
| Retained original test fixture | `c5ae757165a23603fe1186cc76365444cbbce042e387b93f9b812e20ce348454` |
| Executed derived source | `25354c9a8a2ad360ba1e2bac0f0512d61f811c176129dc43d19136762c755e4b` |

## Behavior and independent checks

The two exact, unique source substitutions remember whether the thread is new
and permit only that already-owned first title request. The original exact-policy
text/agent join remains before the exception. Later title requests still require
the exact title prefix. Ordered occurrences, original/forwarded equality, typed
acceptance, bidirectional public-user delivery, factor spans, complete capture
inventory, source hashes and terminal failure retention remain unchanged.

`src/cli.rs::generate_chat_title` calls `run_system_agent_turn`, which launches the
hidden agent once and sends raw followups thereafter. `src/codex.rs::launch_streaming`
injects the first policy; known-session `send_streaming` retains the raw followup.
The exception matches this attested source chain without changing any agent-facing
behavior. No new provider verification is necessary for this offline-only patch.

The independent provider-free invocation used `timeout --kill-after=5s 120s
python3 -B -`, exact-compiled the final test bytes, and ran all **5 new tests**.
It then assigned the unchanged fixture's private `subject` to the derivative's
private module and ran all **15 original tests**. Both suites passed. The wrapper
regression preserves the original title rejection and terminal exit 1, checks
the added verifier/source identities and rejects output overwrite. No committed
fixture or original helper was edited.

All three closed diagnostic inputs were independently replayed through
`audit_sources` under a 180-second provider-free bound. Each replay passed with
19 endpoint-rechecked source identities. After removing only the derivative
identity record and its own added source entry, each result equaled its original
saved report exactly:

| Diagnostic | Input SHA-256 | Original report SHA-256 | Retained exit |
| --- | --- | --- | ---: |
| repeat | `af308de8d3069d39ddb1e11cb1b9e762bc8918fbe3bd385df9fef4674cbe0d4e` | `6708d0dd9113db1f90e54fdb2e0bbdbc355a4107368a57dfec9355ec686842a6` | 0 |
| format | `6d319a43acc953cbe9d7e5731c00e6939d22f1b90bea3c46048db6e9569969dd` | `add34f25a8b2d78610f1ecdefa00314dbd2af03d2436778573f49c962290dcf1` | 0 |
| followup | `cb3b3eb06cd66bed5cbff15e8f0e5edb03b19fdc5c480e7fe627d7036c196353` | `cf9279d3b99f7640ea7d592c10122ac9cd4291f1be3996aad98003d00454f40e` | 101 |

These diagnostic replays contain no title launch; they are non-title regression
checks, not evidence that those fixtures exercised the corrected path. The new
full-policy fixture covers that path. The README separately identifies the
authorized live title-only witnesses as prefix evidence, not closed-file or
whole-workflow analysis.

## Complexity and documentation

The correction performs a constant number of linear source substitutions and
adds constant work per request plus one verifier-source read. It introduces no
quadratic traversal or weaker source/membership gate. Pre-existing auditor
complexity is outside this correction's behavior.

The README requires preserving the original auditor and its output first, reusing
the exact input manifest, and writing the derivative to a distinct create-new
destination with separate source identities. Public app-server item IDs remain
explicitly distinct from native rollout IDs. Runtime architecture and benchmark
admission are unchanged; the analysis workflow belongs in this supplemental
README, not a rewritten frozen protocol. `git diff --check` passed. No provider
calls, active candidate cost inspection or frozen-file changes occurred in this
review.
