# Pure v7 closed-input occurrence joins

Status: pure in-memory join implementation with bounded synthetic tests and completed independent source review. No actual capture, provider, private executor, Cargo, native action analysis or accounting execution is part of this unit. `DELIVERY-JOIN-VALIDATION.md` records the source cut and attributed validation observations; whole-source/frame closure remains a separate gate.

## Boundary and exact dependencies

The pure interface is:

```text
join_delivery(trace_rows, captures, native_sources, expected_run_id,
              primitive_source: bytes, event_validator_source: bytes)
  -> {errors, inputs, trace, threads, orphans, frame_relation,
      exposure_qualified:false}
```

It accepts in-memory values only. It hashes and compiles the supplied exact immutable code bytes once per invocation; no helper file reads occur inside the function. The required source identities are `audit_review_evidence.py` `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257` and `validate_automatic_refresh.py` `c7e2e09cb0b5c25b3e197f477385d9c112222b89ba4effe2f8323f44b136387f`. Compilation keeps each declared original code filename and does not use cached bytecode. The actual imports in those exact source bytes are standard-library imports and definitions, not source/capture collection. Tests may read only these two named code dependencies once; event/capture/native records remain independent in-memory fixtures.

The join invokes the event helper's `census_trace` itself, rather than treating a supplied saved `errors=[]` or `valid=true` as authorization. It reuses the exact primitive `typed_equal`, `rpc_key`, `one_text` and `native_user_index`. It does not call or transplant `join_review_inputs`, whose first-policy ownership and v5 review queue assumptions do not apply to v7. Runtime, observer and all predecessors remain immutable.

The input shapes are small projections of the existing primitive interfaces:

- A capture has `capture_id`, plus `clients`, `forwarded` and `servers`, each a list of `(physical_line_1based, decoded_frame)` pairs. These are complete decoded frame populations, not filtered turn-only lists. Line maps must be positive typed integers and strictly increasing; blanks may create gaps. Capture IDs are unique; order between different captures is not inferred from array order.
- A native source has exactly the predecessor interface: `source`, `thread_id`, `rows:[(physical_line_1based, decoded_native_row)]`. Session metadata must match that exact thread. Only explicit direct/passthrough turn ownership is usable for raw user input. Context inputs remain separately counted, not mistaken for the raw user request.
- Trace rows are the complete v7 decoded trace population, beginning with the one exact activation. Trace line mapping is a later source-wrapper concern; global sequence and process checks are owned by the called pure census.

The later closed-source caller must independently bind all captures/native sources to terminal artifacts, exact original source bytes, physical file populations, settings, model/cwd ownership, resource ceilings and endpoint hashes. This pure join cannot authenticate its caller's completeness declaration. It retains safe source locators and `exposure_qualified=false` even if every supplied input joins.

## Original/forwarded frames are not normalized away

The full original and forwarded frame populations remain supplied and counted. Pair their physical occurrences; a count mismatch or any non-metadata typed mutation is an explicit transport error. All `turn/start` fields, including ID, thread, complete input and additional settings, must be exactly type-equal. The allowed source of a difference is not established merely because its method is `initialize` or `thread/start`.

The result always has `frame_relation.status=upstream-full-frame-proof-required`. It records safe differing-frame locators and canonical frame digests; it never returns a forged normalization or frame-proof boolean. Metadata differences therefore remain pending upstream proof, not silently deleted frames. An original/forwarded turn mutation prevents that request joining.

The later source wrapper must actually invoke the existing `audit_observer_frames.py::prove_frames`, exact SHA `ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f`, on exact raw bytes, complete decisions and settings before treating these pure joins as qualified. That helper's `original_modules` requires exact collector `d3fd8c80746bf4bce565cb5f0a2e2eab29681b3aa40f89196cf95ebf344ed9ee` and primitive `34a34276…`; its settings/journal/raw/terminal membership gates cannot be replaced by a saved result or hand-made normalized stream. Its file-reading wrapper and invocation are out of this unit.

## Complete typed request and input census

For each capture, index all request/reply RPC identities by `(integer|string, value)`, never by string conversion. Duplicate request or reply identity invalidates the relevant association, rather than last-wins replacement. All original `turn/start` requests receive an output row in physical order:

- `rejected`: exactly one explicit RPC error and no result; the safe error code/identity remains, with no accepted turn invented. A reply containing both result and error is ambiguous, never an acceptance or ordinary rejection.
- `missing_reply`: no unique acceptance or rejection witness.
- `ambiguous` or `mismatched`: duplicate/conflicting identities or mutated forwarding/public/native bytes.
- `public_missing` or `native_missing`: an accepted request lacks one of its exact input witnesses.
- `joined`: one typed acceptance supplies the turn ID; one matching completed public user item supplies the public item ID; one exact explicit-turn native user item supplies its distinct native item ID. Full UTF-8 request/public/native text must match, not just its hash.

Public item ownership is `(thread_id, turn_id, public_item_id)`; native item ownership is `(native source/thread, explicit turn, native_item_id)`. A public item ID is never presented as a native item ID. Public item-started notifications are not duplicate completed inputs; duplicate/conflicting completed user items are ambiguous. Accepted `(thread,turn)` IDs must be unique across all supplied captures. Reusing an RPC integer in another capture is ordinary; integer `1`, string `"1"` and boolean `true` are distinct, with the boolean invalid. The exact pinned `rpc_key` also accepts an empty string: this pure unit preserves that predecessor boundary. The mandatory upstream `prove_frames` rejects an empty-string RPC identity before any actual exposure qualification; a pure join is not a substitute for that stricter raw-frame gate.

The predecessor deduplicates native item IDs by `(turn,text)`, not full payload. A preceding exact typed-payload comparison must reject any repeated native user item ID with conflicting content or passthrough metadata, even when its turn/text happen to agree. Only a full typed-payload-consistent replay reaches the predecessor's deduplication. Additional shape guards require JSON-typed content metadata: absent/empty text-element metadata is not interchangeable with `false`; malformed passthrough, contradictory explicit/nested turns, missing IDs, unsupported user content and conflicting item claims stay errors. Native context-kind declarations receive typed validation before the predecessor can skip context inputs. The join must not make a usage filter, infer turns from the latest context, or interpret an unknown malformed context tag as proof of an absent raw user item.

All unmatched successfully indexed public/native user records remain in shared orphan tables. A malformed native source instead remains as one explicit `unavailable-source` placeholder/error; this pure unit does not promise an individual orphan row for each invalid physical user item. The source wrapper retains and verifies the complete original source population.

The pure thread census is the union of native source thread declarations and `turn/start` request thread IDs. Thus title/linearizer or other auxiliary and usage-less threads with those witnesses remain present without a first-policy marker or role guess. An accepted `thread/start` with zero turns and no declared native source is a separate full-frame wrapper inventory obligation, not a thread enumerated by this pure join. Threads without a uniquely established factor owner use `ownership=unknown`; that is distinct from missing transport evidence and does not remove them from this census. Rejected/missing requests and incomplete sources never count as zero tokens or successful delivery.

## Exact prompt occurrences and agent ownership

The event validator returns safe hashes/body facts, not full prompt strings. The join reads original/selected full text only from the corresponding supplied in-memory trace row after the called census/event checks succeed for that row. Output retains only lengths, hashes and source locators. Index full selected prompt hashes once and verify exact bytes on every possible match. The request index includes rejected and missing-reply requests too: an identical failed attempt prevents falsely treating one later accepted copy as a globally unique occurrence.

An ownership anchor requires exactly one valid source-attested `automatic-refresh` trace occurrence of that complete selected text across the trace and exactly one global complete matching request occurrence, whose original/forwarded/public/native input chain is joined. Both occurrence counts must be one. Two identical trace writes and one accepted input cannot establish an anchor or qualify a later identity ACK. It binds the trace's exact agent ID to that exact thread. Identity ACK/command rows do not manufacture an initial owner, and copied `Agent-ID:`/`codex: Codex session` text never establishes one. Multiple consistent anchors are allowed; competing agents claiming one thread invalidate its target associations rather than accepting whichever anchor was processed first.

After anchors are collected across the whole supplied population, use per-owner/per-thread occurrence queues, with accepted ordering established by the unique native source's physical user-input lines. This ordering also works for a single thread resumed through multiple captures; capture list order or timestamps do not supply cross-capture ordering. Within a bound single thread, repeated identical selected prompts are consumed once each, and their counts must match the corresponding request occurrences; an extra identical input or trace leaves that group ambiguous, not silently paired to the first prefix. Nonmatching ordinary inputs may interleave without becoming factor events.

All matched trace sequences for a thread must preserve that native input order, including unique anchor events. A reversed unique follow-up is an order conflict, not a successful set-membership match. Identical text shared by different threads can be resolved only when independent unique anchors establish each agent's one relevant thread. If an agent has multiple anchored threads and duplicate occurrences cannot be uniquely assigned, do not invent cross-thread FIFO from global sequence/time: those target rows remain ambiguous. Unique anchors themselves remain exact, unless contradicted by an ownership/order conflict.

Every nonactivation trace row receives a safe result: `joined`, `undelivered`, `ambiguous`, `order_conflict` or `invalid`, with an input index only for a unique completed input association. An ineligible automatic event is still an identity delivery candidate, not discarded; a valid trace write without a request is `undelivered`. Unsupported/invalid trace rows remain present. This primitive reports input-chain facts, not a final delivered-exposure qualification: that requires the independent source/frame closure described above.

## Output and complexity

Safe output comprises source/capture/physical locators, exact typed IDs, fixed statuses/error codes, byte lengths and SHA identities. No source, prompt, diagnostic, command output or reasoning bodies are exported. Ambiguity uses shared occurrence-group tables with index references; it does not copy an entire candidate list into each trace row. A failure affects the relevant association and the overall completeness/error field; it never truncates another capture's rows.

Build RPC/public/native/text indexes once. Per-thread matching uses ordered queues and one occurrence-count/order pass. Hash every supplied full text once per inventory record; verify collision candidates with exact bytes. Under ordinary hash-table assumptions and no adversarial SHA collision, work and memory are linear in input bytes plus frame/event/occurrence counts. A design requiring an all-capture scan for every event, repeated owner-candidate arrays, timestamp global sorting or arbitrary bipartite matching is outside this small boundary. Ambiguous cases remain unknown instead of invoking a general matching framework.

## Draft gates

`test_join_delivery.py` contains only synthetic payloads and the two named code-dependency reads. It covers typed accepted/public/native joins, title/usage-less completeness, explicit native turns, rejected/missing requests, duplicate IDs, full typed native replay consistency, mixed result/error replies, field mutation, extra/orphan records, exact metadata type checks, absent trace delivery, repeated occurrence consumption, independent anchors, cross-thread ambiguity, reversed order, ineligible trace retention, source-byte pins and safe output. Physical native ordering is tested with capture arrays intentionally reversed. A malformed native source or physical map retains another capture's complete input rows. The complete test count and execution receipts are in `DELIVERY-JOIN-VALIDATION.md`; actual workflow source assembly and `prove_frames` execution remain later gates.
