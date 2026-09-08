# Automatic-refresh runner review

No introduced blocker was found in the private runner adapter. The complete new runner/tests/validation note and predecessor runner/tests were read, their exact diff compared, and the pinned engine's plan, preparation, verification, environment, launch, trust/stop and closure call paths inspected. This verdict excludes the reviewer's own C08 runtime implementation and is not benchmark admission or real-agent qualification.

| Source | SHA-256 |
|---|---|
| `runner_automatic_refresh.py` | `eeba247844dfa073754b01060d50de7898728d422aab27ff69af42d97b8b15fe` |
| `test_runner_automatic_refresh.py` | `dce066f671109504cab32cb7359e9f8d2e804934b28bf00ba530cedae96bad69` |
| `runner_candidates.py` | `ecac7753fb535c59974deedf6a04f0ce11b6e05a19f8c4112416ab7e29b7cb03` |
| `runner_work_units.py` | `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e` |
| `preflight/C08-RUNNER-VALIDATION.md` | `7a495fffba51f87c9b28b0db3b3c17003c77f5818802add574a5bb917668b7e0` |

The independent provider-free command was `python3 -B -X pycache_prefix=/tmp/c08-runner-independent-no-pyc -m unittest test_runner_automatic_refresh test_runner_candidates -v`, from the study directory. Receipt `7350f2` reports six passed, zero failures,0.098s. All five listed source hashes remain unchanged afterward (`3b4a05`). No CLI allocation/prepare/run, actual schedule shuffle, Cargo, provider, executor, accounting or source mutation occurred.

## Introduced behavior

`make_plan` constructs exactly three distinct workflow IDs of `automatic-changed-refresh-full`, one block and one wave, under private schema v7. `validate_plan` first applies the unchanged engine validator and then compares the entire declaration to that exact construction. Added/reduced/reversed rows, controls/Direct, replacements, confirmation labeling and misleading randomization metadata fail. The randomization description is truthful: `mixed_waves=false`, condition count3, only launch order randomized. The inherited engine's `prepare` shuffles the three already-declared IDs within their sole wave; there is no treatment/control randomization or new baseline allocation.

The engine is compiled from verified exact bytes into a private module with its real original `__file__`. Only the private module's condition/schema/plan/verifier bindings are substituted. A separate import of the original engine retains v2 conditions and schema; the six tests also verify that its subscription environment is unchanged. API overrides and inherited Work Leaf overrides are stripped, the existing subscription home is retained, and driver argv, timeouts, grace, scheduling, original outcome classification and trust/stop behavior remain inherited.

`prepare` adds the adapter and engine to the evidence set before original engine publication and requires the actual path-preserving evidence root. The manifest retains the true engine's runner hash. `verify_manifest` first runs the unchanged full engine checks, then requires both adapter and engine frozen entries with exact expected path, `frozen-evidence` role and digest, and rechecks executing source bytes. The engine independently freezes/checks its actual trust helper dependency. No mutable dynamic checkout binding is required by this four-field v7 runtime manifest, unlike the separately scoped C15 launch adapter.

Original `run_phase` uses the substituted verifier in its existing global namespace before admission, each wave and closure. It retains three concurrent workflow slots, create-new RUN-ONCE/outputs, signal handling, pending-trust waits, failure retention and publication. A fixed three-row declaration does not guarantee all three launch when original integrity/stop gates intervene; withheld identities remain in the result.

No new quadratic algorithm is introduced. Fixed-plan work is constant-size; dependency verification is indexed plus byte hashing. The existing engine O(W×E) frozen-input work remains, with one wave here. Source loading/metadata checking is not a guarantee of provider success, full effective-configuration equivalence or bounded provider usage.

## Documentation and limits

The validation note accurately separates source/tests from admission and real-agent delivery. The module docstring correctly limits randomization to launch order and disclaims controls/replacements. The private runner changes no public integration surface or documented runtime ownership; no additional architecture edit is required for this adapter. Its present tests cover plan and environment identity, not a new complete prepared-phase integration replay; the source diff leaves that inherited machinery unchanged.

The actual one-author qualification and any later fixed three-workflow screen require their own source/protocol/admission gates. This review neither schedules the screen nor interprets the concurrently running diagnostic. Only this new review record was written.
