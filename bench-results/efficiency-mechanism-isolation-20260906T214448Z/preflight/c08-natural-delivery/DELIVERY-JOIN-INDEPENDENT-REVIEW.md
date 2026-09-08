# Pure delivery-join independent review

Verdict: no blocking implementation finding in the reviewed pure join. This is
not a closed workflow, native population, full-frame or exposure qualification.
The function always returns `exposure_qualified=false` and requires the separate
source-bound `prove_frames` gate.

## Reviewed cut and checks

| Source | SHA-256 |
| --- | --- |
| `join_automatic_refresh_delivery.py` | `614ca2faab8790a24ec02b20cec5dcb2997ed7ae79273ec86e5304570565a402` |
| `test_join_delivery.py` | `8b8d80419fde82e1a0a9b2ea312b258e9580538a0f6ba54cd17cb745038517d7` |
| `validate_automatic_refresh.py` | `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f` |
| `test_delivery.py` | `9ac394e07c4d5ee04c6f3d03f02bbb775ef1ac5cd7679c1d40ade9b3ab071400` |
| `../../audit_review_evidence.py` | `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` |

The complete join and its tests, the relevant exact primitive/census functions,
and the complete resulting design/validation documents were read. The final
documentation qualifies the scope limits below: DESIGN SHA-256
`4ffa3c0a5073bf9707724927f609431658a68fecea10e122b6522ffe79545d24`,
VALIDATION `55b8cee83f5bbc632cfcd7ea69fbf0dd051d3574235f83c681610778986f9596`.
The wording delta was inspected (`35c77b`) and both final identities rehashed
(`628107`). Source rehash receipt `160555` matches every executable/test identity
above. No implementation correction remains requested.

Independent command, in this directory:

```text
/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -q test_join_delivery test_delivery
```

Receipt `fbebdb`: exit0, 42 tests in 0.210 seconds. A separate bounded in-memory
check replaced each synthetic capture/native dictionary field with eleven JSON
types/values: 792 cases, no uncaught exception (`e47750`, exit0, 3.77 seconds).
This is robustness sampling, not exhaustive schema proof. Only the named static
code dependencies and local synthetic test modules were loaded; no actual
capture/native payload or accounting, provider, Cargo, Git or executor operation
was used. The owner's missing-module RED and subsequent GREEN remain recorded
separately in [DELIVERY-JOIN-VALIDATION.md](DELIVERY-JOIN-VALIDATION.md).

## Introduced behavior reviewed

`frame_interfaces` and `request_interfaces` preserve typed request identities,
rejections, missing replies and duplicate/conflicting associations. Nonmetadata
forwarding mutation conservatively invalidates all turn associations in that
capture, while retaining their rows. The helper does not pretend metadata
method names prove observer rewriting.

`native_interfaces` checks complete typed repeated-user payloads before calling
the unchanged explicit-turn native index. `link_inputs` requires distinct
accepted turn identities, exact full request/public/native text, public-item
cardinality and separate public/native item IDs. A mixed result/error reply is
ambiguous. Neither native usage nor a latest-context fallback supplies ownership.

`join_occurrences` requires singleton valid refresh-trace and global request
occurrences for an anchor. Rejected/missing identical requests participate in
the occurrence count. Identity ACK/command rows and copied owner markers cannot
create an initial owner. Conflicting owners, repeated-count mismatches,
cross-thread ambiguity and native-order conflicts stay explicit. Indexed native
physical order, not capture-array order or timestamps, governs resumed threads.
Safe results omit prompt, diagnostic, source and reasoning bodies.

## Exact limits and documentation qualification

- The pure thread table contains supplied native declarations and thread IDs
  obtained from `turn/start` requests. It does **not** discover an accepted
  `thread/start` with no turn and no supplied native declaration. The later
  source wrapper must inventory all accepted thread starts, including those
  empty/usage-less threads. Describing this pure table alone as every accepted
  thread would be too strong.
- A malformed native source is retained as an invalid/unavailable source plus
  errors; its malformed users are not all individually reconstructed into orphan
  rows. The source wrapper must preserve the complete physical source inventory.
- The pinned `rpc_key` accepts empty-string RPC IDs. The mandatory full-frame
  `prove_frames` rejects them. A pure joined row therefore is not proof that an
  input satisfies the complete transport contract. Boolean RPC IDs are rejected
  in the pure join itself.
- Global census/source errors remain authoritative even when an independent row
  joins. The helper cannot attest the supplied population's completeness,
  runtime source, settings, model/cwd, actual actions or repair semantics.

No introduced production O(n²) scan was found: constant source passes and
RPC/item/text indexes make work and output linear in supplied bytes and record
counts under ordinary hash-table/no-adversarial-SHA-collision assumptions.
Ambiguity lists are shared rather than copied per event. The previously reviewed
synthetic `event_fixture` has bounded immutable-byte concatenation that would be
quadratic if expanded to unbounded sections; it is unchanged and is not a
production join algorithm or a new join regression.

No real agent-facing runtime behavior is modified by this offline helper.
Real-agent and natural source/delivery qualifications are not supplied by these
tests. The prospective wrapper draft remains unexecuted and actual source
assembly remains outside this review.
