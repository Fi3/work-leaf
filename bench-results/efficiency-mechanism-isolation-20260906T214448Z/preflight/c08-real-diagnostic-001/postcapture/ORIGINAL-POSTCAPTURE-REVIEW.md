# Original C08 observer postcapture

The original analyzer and prerequisite-satisfied extractor each executed once
under the fixed [scope](POSTCAPTURE-SCOPE.json), with180-second external limits
and5-second kill grace. Both returned2. All184 declared source endpoints match
before and after both commands. No provider, accounting helper, reanalysis or
replacement execution occurred.

[Analyzer output](OBSERVER-ANALYZE-ORIGINAL.json) retains
`capture_complete:false`, five interrupted turns without complete usage, and
eight marker findings in the two copied observer ELF proxy binaries. Marker
identity is distinct from unknown usage; neither finding is cleared here.

[Extractor output](OBSERVER-EXTRACT-ORIGINAL.json) reports zero observed/matched
threads and one unobserved-cwd thread,
`01a080be-48ad-7bb0-9c3f-4afd2f06e5ec`. Its original error says that this rollout
shares the observed cwd but is absent from process capture. These outputs remain
unchanged. The separate full public/native witness must inspect accepted
usage-less thread identities directly; it does not rewrite this extractor's
membership rule, error or original counts.

The actual harness closed five calls and processed DONE, with exit0 and no
source endpoint errors. That is not proof of complete token accounting. No raw
total, invented missing-response charge or saving estimate is produced by this
qualification. Exact argv/times/output hashes are retained in each original
`OBSERVER-*-ORIGINAL-EXECUTION.json` receipt.
