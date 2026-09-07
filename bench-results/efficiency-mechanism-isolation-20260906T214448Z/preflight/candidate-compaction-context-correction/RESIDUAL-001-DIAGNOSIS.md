# Candidate 001: repeated compaction metadata and interrupted tails

The corrected-nine result remains **unknown**, with null bounds. Its pre-turn compaction context proof passes, but a later nonadditive `last` repetition is outside the frozen accounting rule. The observed cumulative value at the rejection equals both the preceding cumulative value and the checker's expected value. No accounting helper, frozen report, benchmark outcome or provider behavior is modified by this diagnosis.

## Exact evidence and invariant

The source-bound record is [RESIDUAL-001-DIAGNOSIS.json](RESIDUAL-001-DIAGNOSIS.json), SHA-256 `c54aa70679fbb449921c7d0b89af89793334d5bacda4e6df7c600d128312ade3`. It includes source hashes, typed request IDs, physical line locators, five interrupted-tail witnesses and the exact provider-free reproduction commands. Only public/native identity and lifecycle metadata is exported; no private reasoning or message bodies are reproduced.

The public source is candidate001's primary capture `observation/app-server/00000484198115543375-567/server-to-client.raw`, SHA `eb6b79e39758e05f3bc06ad3de58ae14408e5d8c5bff2c18938d9cef641a9c92`. All lines below are physical, one-based. The affected user-3 thread is `01a07bd7-f780-7140-8e7f-b240f3657013`.

| Boundary | Exact public source |
| --- | --- |
| Accepted compaction-bearing turn | Reply 43734 to string RPC `64`; original request line 65 |
| Explicit compaction lifecycle | Item `01a07be7-b00c-7c31-8933-678688e5f488`, started 43737, completed 44097 |
| Named raw compaction response | 44094, `resp_007f4ca9a120523b016a9eb21b0ffc87d2ac22c56740dd793e` |
| First nonadditive notification | 44095; accepted as the one named omission, with unchanged cumulative |
| First carry-forward repetition | 44130, same accepted turn, after compaction completion |
| Further repetitions | 44327, 44367, 44489, 44579, each on its exact subsequently accepted turn |
| Next genuine raw response | 44690, `resp_007f4ca9a120523b016a9eb2bd711087d28d598de6ee1143e9`; fresh cumulative notification 44691 |

All six `last` objects have `totalTokens=77959` and every component zero. Their canonical JSON SHA is `f016206dfd317c8374510a45a3ee64a45cfcf096d8cda249bc12fa9107fb30ff`. Their complete cumulative objects have the same canonical SHA `6ba723f1baa4675fee77d0aab7cfca8eee92e95982420a9fe911bbc90d6c4322`. The five repeated notifications contain no intervening same-thread raw response after the preceding cumulative notification. This establishes repeated metadata, not five additional measured charges or documented provider context-estimate semantics.

The source-bound pure-function diagnostic invokes only the pinned `collect`, previously proved `corrected_ledger`, and unchanged `reconcile_stream`; it does not call either `audit_run` or compute a whole-workflow total/bound. Its exception trace shows:

- Frozen `accounting_untracked_reads.py::reconcile_stream` line 322 raises `nonadditive last lacks unchanged explicit compaction boundary` at public line 44130.
- `newly_omitted=None`, `nonadditive=True`, `total == prior == expected`, and the derived thread ledger is still empty.
- Lines 307–319 assign a named omission only on its first cumulative boundary. Lines 320–322 require that assignment again for every nonadditive notification.
- The exception returns through lines 362–364 before the thread-ledger loop at lines 340–345. `audit_run` then fails its empty derived-ledger/observer-thread comparison at lines 491–492. That generic outer message masks the inner metadata restriction; it is not evidence that an observer thread is actually absent.

All eight observer threads have actual raw response identities and native membership. The preserved 204 raw response identities therefore do not conflict with an empty *derived* thread ledger: the raw-identity collection precedes the failing cumulative pass.

## Missing generation stays missing

The five affected ordinary continuations end `interrupted` at public lines 44134, 44331, 44371, 44493 and 44583. Their exact interrupt requests are original/forwarded lines 69, 71, 73, 75 and 77. Grace rows 27–31 retain `configured_grace_ms=1000`, `output_resume_policy=forward`, and `forwarded-after-output-resumed`; waits are 1, 21, 1, 0 and 2 milliseconds respectively.

Each continuation has actual public assistant/reasoning item activity. The first turn's raw record is the earlier compaction response only; none has an ordinary generation response-usage record. Native user-3 lines 291/303/315/327/339 contain token-count metadata, with aborts at 293/305/317/329/341. The next native `token_usage_record` is 351, matching public response 44690. The native source SHA is `722f2338566148bc21aae916d237f4f23b143e342ebea14c4da4fbbea45af6fc`.

These repetitions neither recover missing generation nor prove its cost zero. Recognizing metadata carry-forward must not create fresh-usage evidence, satisfy a terminal-response proof, remove a gap, count a compaction twice or introduce an upper bound. A further accounting pass could still be ineligible or unbounded for independent tail rules.

## Prospective sibling derivative — design only, not admitted

A separately pinned sibling may extend the immutable corrected helper's **metadata provenance** rule, not response cardinality or arithmetic. The smallest defensible unit is a per-thread, exact originating-compaction carry state:

1. An origin exists only after the unchanged rule proves one accepted local turn, explicit native/raw compaction response identity, complete matching public lifecycle, exact additive response usage and unchanged cumulative omission. The existing native future-context proof remains mandatory where applicable.
2. A repetition must have the exact same typed full `last` and full cumulative values as that origin, the same source/thread, an exact accepted local turn, and a position after the proved origin. Its cumulative must still equal the independently tracked raw-prefix-minus-named-omission expectation. It is a retained unusable-metadata warning linked to the origin, never a response.
3. No intervening same-thread new raw response, new compaction, changed cumulative, changed `last`, conflicting identity, malformed parent/type or unresolved source boundary may carry the state forward. A new legitimate compaction must establish its own origin under the unchanged rule. Other-thread interleaving does not transfer ownership.
4. Every affected turn keeps conservative tail handling. Neither original nor repeated metadata may establish fresh usage or late-terminal recovery. All actual generated-item evidence remains in place. Existing gap multiplicity and unsupported-tail bounds remain unchanged.
5. The full stream and physical line numbers stay unchanged. A source-pinned exact function derivation may add this state and warning branch beside the existing nonadditive predicate, while retaining the original cumulative checks and response ledger code. A wrapper that silently deletes arbitrary notifications or reconstructs a zero response is unsuitable.
6. Failures must expose the inner evidence error as well as preserve the old outer failure in a separate output. No original helper/report is overwritten. Another whole-nine numerical replay requires an explicitly declared scope after independent review, not an automatic continuation of this diagnosis.

Required RED-first tests: origin absent; wrong thread or turn acceptance; RPC JSON-type collision; incomplete/conflicting compaction lifecycle; altered `last` or cumulative component; cumulative regression; intervening raw response; second compaction; later stale reuse after invalidation; source drift; all five interrupted tails with output and no response; no double charge; original valid-result equality; preserved unknown/unbounded tails; physical source-line identity. A positive repeated same-value sequence must retain all warnings while contributing no new ledger entry.

The state machine can be one forward pass with a per-thread map: O(E + T) work/storage beyond existing source hashing, no per-notification history scan. The design deliberately does not infer universal provider semantics from this one captured sequence.

## Retained outputs and limits

The immutable derivative is `accounting_compaction_context.py`, SHA `513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3`; the original accounting dependency remains `c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`.

Candidate001's supplemental receipt SHA remains `3977474cc35da56075d49cf4e90843376528d3483f8e590acad858616d5fd1b2`. The corrected-nine consolidation remains `3a33ee3f414f1b87967221cdbf4be84ac5773413dd68c971eef5d5bdedb8d595`. All sources used by the pure diagnostic and both saved outputs were rehashed unchanged at its endpoint. This record supplies no third accounting total, contribution percentage, quality exclusion, provider call or new benchmark.
