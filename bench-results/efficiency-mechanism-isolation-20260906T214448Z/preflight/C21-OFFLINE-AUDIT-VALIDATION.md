# C21 offline audit implementation checks

The private `audit_review_evidence.py` primitives verify exact opaque payload bytes,
source-owned UTF-8 replacement ranges and native read-output representations. They
are not a complete delivered/native ownership census, token estimator or admission.
The final source-bound input/turn/call/archive inventory remains implementation work.

## Recorded test-first sequence

- Initial `python3 -B -m unittest test_review_evidence_audit.py` fails because the
  new audit module does not exist. The test defines complete and selected-inline
  archive identity, unowned-byte/owner/selection failures, full/partial reads and
  path-mention/nonzero-exit/truncated-output rejection.
- The first implementation passes all five tests.
- Three further edge tests fail: Python's generic `splitlines` does not model
  LF-delimited `sed` lines containing bare carriage returns; an arbitrary executable
  merely named `cat` must not be treated as a supported utility; an actually successful
  read of an empty artifact can establish complete zero-byte retrieval.
- LF-only splitting, explicit supported executable spellings and precise empty-output
  handling pass all eight tests. `Original token count` remains an ordinary tool header,
  not a truncation marker.
- Exact accepted/public/native input joining has its missing-function RED followed by
  ten passing tests. Explicit native turn ownership and full input text are required;
  public and native item IDs remain distinct. Usage-less threads remain in the census.
- Independent review reproduces three provenance faults before their fixes: unquoted
  shell operators/expansions accepted as literal reads, malformed present native turns
  hidden by a valid nested value, and Python boolean/integer equality aliasing JSON
  identities. Four new tests reproduce eleven failures, then pass with quote-aware
  literal command validation, independent turn-field validation and typed equality.
- A further wrapper/quoting matrix reproduces two closing-parenthesis failures before
  the literal-syntax fix. All 15 tests pass, including literal quoted shell metacharacters
  and a single explicit shell wrapper; inner evaluation remains unsupported.

Qualified primitive SHA-256: `34a34276e2f3914b35c72521a303c2cb41ec54c646299cda2b7a9c600146a257`.
Test SHA-256: `d5a58b1a001c9fd909a34b0371a614e5b194aac65277a82fc9d027e401da3afe`.

## Limits and pending gates

Read classification requires the caller to join the issued archive, actual reviewer
thread/turn and native call/output first. Exact output bytes and a supported command
are evidence of returned text, not disk-I/O monitoring or hostile-executable containment.
Unsupported aliases, compound programs and unmatched results remain unresolved.
Snapshot validation does not independently certify the renderer's reference wording;
the admitted source identity and full delivery audit own that contract.

Dictionary/queue delivery joins and typed equality are linear in inspected evidence.
Read classification scans one selected payload per invocation: repeated partial reads
can cost O(R × B), for R reads of a B-byte archive. The source collector must index
candidate paths and avoid crossing every archive with the entire native transcript.
No provider, runtime, frozen helper, old outcome or token total is changed
by these primitives. Independent review, the complete source/delivery census and
bounded real-review retrieval remain required before the experiment is ready.
