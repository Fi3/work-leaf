# Independent private-executor review

Recorded 2026-09-07 16:23 UTC. Scope: the standalone provider-free C15 executor
qualification, not runtime admission or a causal observation.

Verdict: no blocking finding in the reviewed helper, tests and qualification
claims. The full source and tests, [qualification](QUALIFICATION.md), and both
[initial](../../DESIGN-TEST-FIRST-ISOLATION.md) and
[detailed](../../DESIGN-TEST-FIRST-ISOLATION-DETAIL.md) designs were inspected.
No implementation or existing evidence was edited by this review.

## Independent checks

From this directory, `python -B -m unittest -v test_executor.py` passed all 11
tests independently, including a final repeat in 1.528 seconds. These invoke the
actual local sandbox and offline compiler, not providers. The retained
[qualification receipt](QUALIFICATION-001.json) separately contains 11 returned
executions, all closed, including the expected Cargo failure, successful
unchanged-test continuation, timeout, cancellation and output-limit outcomes.
Test count and execution count describe different inventories.

`executor.py::run_preview` validates the pinned executable and caller-owned
mount specification before starting bubblewrap. The initially empty filesystem,
explicit read-only runtime mounts, three writable private trees, cleared
environment, isolated namespaces and held mount descriptors support the tested
boundary. `validate_owned_trees` rejects external hardlink aliases and special
files before spawn. The tests check shared/evidence/configuration surrogates,
absolute and symlink paths, spawned children, inherited descriptors/environment,
network and nested user namespaces. Detached-child tests include a tagged host
process census after ordinary return, timeout and cancellation.

The output prefix is bounded and explicitly marked when truncated by the
executor. Uncertain closure raises with preservation required; the helper does
not remove source or evidence directories. No unconfined fallback exists.

Complexity flag required by repository review policy: extra-mount overlap
validation is O(M²), with at most 32 additional mounts plus fixed runtime mounts.
This is a documented, small admission bound, not transcript-dependent quadratic
work. The owned-tree traversal is O(F), bounded to 200,000 entries/depth 64, with
keyed inode counts; output collection is linear and explicitly limited.

## Limits retained

The launcher must be trusted to supply owned writable roots and only public
read-only inputs. Source/system mutation by a hostile host actor is outside this
qualification; endpoint hashes and held descriptors are not continuous immutable
source attestation. No actual project snapshot materialization, Git equivalence,
current dependency-cache replay, test-purpose classification or treatment
integration is implemented here. The real offline Rust fixture uses no external
dependencies; the read-only cache is a surrogate.

The architecture/public interfaces and normal agent workflow are unaffected by
this private helper. Its qualification note accurately documents the relevant
operating constraints; no production architecture or operator-policy change is
required for this scope. No real-agent workflow is affected, so this review does
not launch a provider or claim the separate C15 real-agent gate passed. Runtime
integration, source/proposal/test identity and actual-project qualification
remain separate requirements. No token costs or saving percentages were read or
computed for this review.

## Exact reviewed identities

| File | SHA-256 |
| --- | --- |
| `executor.py` | `16e3238e1bd647c9d4780db97c554ddf9579226c913995e2e18873bf7fcb24ae` |
| `test_executor.py` | `7972e12652552252a29d1f87ca840b3fe17f484d424ec97c0ecdcdcad621f1f9` |
| `QUALIFICATION.md` | `270c3ed92ab038067f155db8c0ad31698c051e4a47de8238e5e8949f97b97552` |
| `QUALIFICATION-001.json` | `1898a8782bf91be4288bfbf47170af60d8af5e6d4b8c64cf8a821e7af8132622` |
| `DESIGN-TEST-FIRST-ISOLATION.md` | `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493` |
| `DESIGN-TEST-FIRST-ISOLATION-DETAIL.md` | `9e4911e63cbd5fac54475552dc883d15c4448eaf149a4e8e2a134df8328240ff` |

Helper, test, qualification and receipt hashes were rechecked at closeout.
