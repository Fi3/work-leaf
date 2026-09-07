# V3 offline analyzer review

Scope: independent source/synthetic-test review of the prospective v3 delivery and primary
analyzer. No provider generation, phase outcome comparison, runtime edit, or frozen-report edit
belongs to this review. Finding status: no unresolved introduced-behavior findings.

## Reviewed identities

Paths are relative to this study directory; hashes are SHA-256.

| Source | SHA-256 |
| --- | --- |
| `analyze_untracked_reads.py` | `30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e` |
| `test_analyze_untracked_reads.py` | `626cd06626e91b78b6096e253bad42834d39a4fb08a0433ad26db52e69db121f` |
| `PROTOCOL-UNTRACKED-READ-INLINE.md` | `552d736a1be49ac1578bfb13b43848ba869a5cba453817be72803978ac62fd7f` |
| `DESIGN-UNTRACKED-READ-INLINE.md` | `4cdc6472f7edffa6b750d56255aa97dadb7b1967c186c88bc12393cbc1dc8833` |

## Checks

- `validate_read` (134–210) reconstructs the renderer-owned inline component from exact UTF-8
  body intervals and normal FNV64 metadata, reconstructs the successful bundle manifest, and
  requires byte identity outside that component. Eligibility follows actual bundle success,
  not merely threshold eligibility. Ineligible candidates remain identical. Explicit-bundle-only
  null components, formatter-added newlines, adversarial marker text, and Rust component-wise
  project-path ordering are represented. SHA-256 is derived offline; candidate bytes are not tokens.
- `prompt_inventory` (236–326) joins exact selected text to typed request/reply identities and
  known agent/thread scope. It distinguishes accepted, rejected, missing-terminal-reply, and
  prepared-without-captured-request states. Duplicate/ambiguous identities and incomplete reverse
  coverage invalidate delivery. Physical JSONL line positions survive blank records. Accepted
  turns are explicitly not treated as native input-item identities.
- `measurement_for`, `primary_for`, and `analyze_manifest` (379–573) require the separately
  source-pinned accounting replay and preserve its source identities. Exact helper paths, admitted
  manifest/source hashes, original configuration replay and terminal incident gates are checked.
  Failed workflows remain eligible observations if their evidence is valid; withheld, unknown,
  duplicate and unexpected rows are retained and prevent a complete primary. A validated scope
  with an unbounded tail stays unbounded rather than becoming a zero or finite estimate.
- `allocation_choices` and `randomization_test` (329–372) implement two independent 18-allocation
  blocks with four distinct mixed waves. For each of 324 assignments, the contrast-minus-observed
  coefficients cancel unchanged memberships; each remaining signed coefficient uses the correct
  lower/upper endpoint. Integer arithmetic preserves ties. The mean difference divides its signed
  sum by six; the omitted common positive factor in permutation comparisons cannot change order.
  The conservative p-value envelope is not claimed to be jointly attainable or a sampling interval.
- `passes` (375–376) requires a positive lower contrast and the exact upper-tail criterion
  `possibly_greater_or_equal * 40 <= 324`. There is only the preregistered whole-workflow raw
  input-plus-output primary, with no quality filtering or substitute significance endpoint.

Independent command, from the study directory:

```sh
python -m unittest test_analyze_untracked_reads -v
```

Result: all 24 tests pass. The cases include a complete twelve-row runner-shaped integration
fixture, exact/interval permutation checks, source/helper mismatch, incident retention, typed
delivery failures, and byte-span adversarial inputs. This is an offline helper: it does not affect
an agent-facing workflow and needs no additional real-provider call for its own execution.
Separate real-subscription runtime diagnostics and their actual-capture replay remain admission
requirements; synthetic tests alone do not discharge those requirements.

Complexity: request/reply/exposure joins are indexed; body processing is linear in represented
bytes, with project-path ordering O(s log s). The explicitly capped permutation is O(324 * 12).
No introduced O(n²)-or-worse unbounded algorithm was identified. The separately reviewed accounting
and inherited trust replay remain separate owners, not implementations certified by this note.

Documentation review covers the prospective design/protocol and relevant architecture, operator
policy and file-workflow documentation. The private offline analyzer matches those contracts;
normal file-read/public API documentation needs no additional analyzer-specific change.
