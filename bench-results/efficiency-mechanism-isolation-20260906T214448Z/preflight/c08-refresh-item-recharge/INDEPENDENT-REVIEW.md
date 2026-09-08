# C08 refresh-item adapter independent review

The pure adapter has no introduced source/test blocker within its explicit
materialized-input boundary. Actual file provenance, qualified lower-result
consumption, once-only reservation and bounded publication belong to the future
root caller; this review is not an extraction or admission.

Root read the complete259-line adapter, all20 tests, design and validation,
plus the unchanged extractor and recharge functions it uses. Source identities
are `eb7c7916029cb6ee8e6b920d3bdf91bd413da0243ef177aba3646b87a8b88fa1`
for `refresh_recharge.py` and
`44af7127c33f22e583756ab45c44c214820e117e39051974fbf329ae1e1c9fd7`
for its tests. Independent command:

```sh
/usr/bin/timeout --kill-after=5s 30s /usr/bin/python3.14 -B -m unittest -v test_refresh_recharge
```

From this directory, receipt`2fef44` passes20 tests in0.118s. Endpoint hashes
remain identical (`404ea3`). These are synthetic extractor calls, never actual
saved-response extraction, provider work or workflow accounting.

The exact2/6/3 eligible native user items retain distinct full-text, native-
payload and renderer-owned-body identities. The automatic-refresh category is
not mislabeled as a requested read. Native item inventory uses explicit known
turns. All corresponding post-introduction completed responses remain in the
window, including no-hit, missing, contradictory and unknown metadata. Each
qualified response map is bound by its complete identity/count/hash; its usage
totals and all baseline records are not interpreted.

Extraction calls the unchanged source-pinned implementation once per prepared
run. Whole-item projection retains unknown responses, unmatched identities,
full extraction hashes and temporal contradictions. Source text and model
reasoning bodies are not exported. Positive repeated charges would establish
whole-input-item recurrence, not a token price for a file-body substring or a
net counterfactual effect.

Identity joins use indexes and constant-pass scans, with canonical/identity
sorting. No introduced quadratic production path was found. Existing post-
construction row/byte limits are not peak-memory or source-loading guarantees;
the caller must separately bound its complete success/failure output envelope.

No agent-facing runtime, public interface, prompt, launcher or provider setting
is affected. Real-agent and Cargo verification do not apply to this isolated
offline Python adapter. The current design accurately documents these limits;
no runtime architecture documentation change is required.
