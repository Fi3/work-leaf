# Independent compaction-context derivative review

Result: no remaining introduced correctness or scope finding at these exact bytes:

- `accounting_compaction_context.py`: `513c8632803656d4f582a4ab714e35d9e0f95b56a4355b3b26c2927b07d2bfa3`.
- `test_compaction_context.py`: `bbc93570c5b08e86b58a978022cc68347f70e76d187e295404eeae1c587cd94b`.
- `README.md`: `780a632dd336cb0851eca43290fd0a8ae2f2222ef6ff3aa1217a3fefe2c124e9`.

Independent command: `python -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/candidate-compaction-context-correction -p 'test_compaction_context.py'` — **22 passed**. Review covers the complete new helper, tests and README, not unrelated historical behavior.

Six review findings are closed with the owner's reproducing regressions and the inspected final guards: intervening generated `event_msg` records; contradictory explicit public model/effort/cwd; reuse against altered native metadata; false-valued non-array `text_elements`; explicit null/invalid source digests; repeated whole-capture indexing across qualifying threads. The last issue has one shared public index per workflow rather than an added O(U × C) capture scan. Supported metadata forwarding is source-proven by the unchanged pinned complete capture-provenance checker, not a supplied permission boolean; turn inputs remain exact.

The only native eligibility extension is a single exact matching-turn predicate substitution into SHA-pinned `audit_compaction.py`. The proof is bound to native physical line, response ID/full payload, full native rows and metadata. `native_ledger` retains its original function code under a private native-module binding. No rows are inserted/reordered, no cumulative arithmetic is replaced, and no other original error is waived. Already-valid native records take the original path. No observer, provider, prompt, wait, config or runtime behavior changes.

Actual source verification was read-only against `ELIGIBILITY-001.json` (`5df3f136e57a3ac43320b47c58ba2e037fa07c5673cb7065acaae04234c24308`). All **25** recorded endpoint source hashes match. Independently checked native response line 276 → exact explicit marker 277 → context 284 → user 285 against accepted RPC string `64`, public compaction start 43737 → raw response 44094 → matching completion 44097 → public user 44099. Response/turn/thread/item identities, model/effort/cwd, exact full original/public/native input and its hash match. Intervening records are the supported metadata/compaction-completion/context envelope, not generated agent output. Public and native user IDs remain distinct.

This review neither invokes `accounting.audit_run` nor computes/exports corrected workflow totals. The original failed native-prefix diagnostics and original common-accounting UNKNOWN remain retained. The separately declared corrected all-outcome accounting scope is a later parent-owned action; eligibility alone is not an endpoint, causal claim, or waiver of a failed workflow. Runtime real-agent verification is inapplicable to this provider-free evidence derivative.

Complexity: shared capture indexes and disjoint qualifying spans avoid the identified event-pair cross product. Repeated exact dependency compilation/hashing remains O(T × D), with D the fixed pinned dependency bytes; this is disclosed. Endpoint hashes are not a hostile-writer or continuous-integrity guarantee.
