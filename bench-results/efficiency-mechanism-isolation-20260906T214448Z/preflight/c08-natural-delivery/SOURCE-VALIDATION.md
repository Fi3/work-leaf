# Source-wrapper validation

Status: synthetic source gates pass; independent review and the root's fixed
actual-input scope remain required. No actual source audit, observer operation,
accounting call, provider invocation, Cargo build or private executor was run by
this implementation task.

## Exact cut and command

- `audit_automatic_refresh_sources.py`: `d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80`.
- `test_sources.py`: `25c532f240bc49f070f318029a967e531d15e6ce1a055923e6fa94ec62470502`.
- Frozen join: `614ca2faab8790a24ec02b20cec5dcb2997ed7ae79273ec86e5304570565a402`.
- Frozen event validator: `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f`.

From this directory:

```sh
/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -q test_sources test_join_delivery test_delivery
```

Final tool receipt `0edc2f`: exit 0, 73 tests in 0.354s: 31 source tests plus 24
join and 18 renderer-validator tests. Temporary fixtures call the unchanged
source/frame implementation and frozen pure join, including the complete CLI
publication path. They are not an actual-run source qualification.

## Retained test-first evidence

| Boundary | Observed RED | GREEN |
| --- | --- | --- |
| Missing source wrapper / execute contract | `d7c268`, `79e2d9`: missing module | `ed2b43`: 19 tests |
| Project inventory and grace/terminal functions | `766a15`: four missing functions | `4b7666`: 23 tests |
| Exact ledger projection and published-root relation | `c31322`: three failures plus missing relation | `3bce16`: 24 tests |
| Scope limits and all-kind streams/meta | `ea2077`: missing functions | `32e79b`: 26 tests |
| Original JSON-array report retention | `79b7cf`: missing function | `c692bd`: 27 tests |
| Complete synthetic execute and CLI | Existing source functions exercised without proof mocks | `9209b8`: 29 tests |
| CLI explicit declared limits and published LF bound | `058407`, `665990`: explicit kwargs and byte-bound assertions fail | `a05f7a`: 29 tests |
| Removed generated profile source relation | `d9c4cb`: missing function, nine subcase errors | `7b583f`: two tests |
| Full CLI with removed generated launcher | `4fe2d4`: both complete cases fail `undeclared-executable-source` | `0edc2f`: 73 combined tests |

The early missing-module drafts were source-only and held during the active
wave. Root released temporary-source execution only after all three natural
workflows closed. No actual payload was used to produce a passing fixture.
Source-only metadata inspection identified legitimate original JSON arrays,
start-record projection and cleaned-up generated profiles; these are explicit
source contracts, not ad hoc deletion of mismatches.

## Qualifications

The [input schema](SOURCE-INPUT-SCHEMA.md) separates canonical retained files,
the source-grounded stage publication relation and the generated profile's
byte reconstruction. The latter pins the full original renderer, accepts only
its unchanged ASCII `%q` path subset, binds all eight retained fields and
report/config/start hashes, and never executes or recreates the launcher.

Original observer errors, failed workflows and incomplete measurement flags
remain in the safe result. Source availability is not measurement validity,
complete usage, action semantics or a savings estimate. Zero-turn native
declarations are checked separately from the pure join; the full original raw
frame proof precedes delivery joins. Boundaries in the project journal and
grace log do not prove continuous source immutability or rederive usage timing.

Source reads use the unchanged bounded, endpoint-verifying collector. Lookup
passes are indexed; deterministic directory and canonical-object output use
sorting. No new quadratic production history scan was identified. Output row
and byte ceilings are postconstruction admission gates, not peak-memory limits.
Failure does not authorize retry. The reserved output/attempt is retained on
publication failure; crash-proof result storage is not claimed.
