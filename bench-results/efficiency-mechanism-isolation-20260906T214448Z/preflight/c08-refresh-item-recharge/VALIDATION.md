# C08 refresh-item adapter synthetic qualification

The private adapter has 20 passing synthetic tests. No saved target payload, response charge, workflow ledger, baseline result, provider, observer, extractor-on-real-data, Git operation or Cargo job was executed by this qualification.

Exact source cut:

| File | SHA-256 |
| --- | --- |
| `refresh_recharge.py` | `eb7c7916029cb6ee8e6b920d3bdf91bd413da0243ef177aba3646b87a8b88fa1` |
| `test_refresh_recharge.py` | `44af7127c33f22e583756ab45c44c214820e117e39051974fbf329ae1e1c9fd7` |

The command, run inside this directory, is:

```sh
/usr/bin/timeout --kill-after=5s 30s python3 -B -m unittest -v test_refresh_recharge
```

Receipts:

- Initial RED `207daa`: tests existed before implementation; import failed because `refresh_recharge` did not exist.
- Initial GREEN `146adf`: 18 tests passed in the printed 0.132 seconds.
- Focused RED `935af3`: a known response with a changed thread was silently excluded; a later target could be charged before introduction while marked exact; projection rejection raised before returning the full extraction hash. Two failures and one error were retained before the narrow fixes.
- Final GREEN `53dbd5`: all 20 tests passed in the printed 0.124 seconds.
- Source cutoff hashes were read in `5ec099`.

Tests invoke the real unchanged extractor only on synthetic in-memory metadata. They cover complete 2/6/3 targets and repeated whole-item charges, native/public identity separation, three distinct text/payload/body hash guards, automatic-refresh user kind, explicit conflicting/boolean turns, failed no-usage accepted turns, lower-map identity, source subproof failures, missing attribution, identical/conflicting duplicates, foreign/malformed response identities, no-hit response retention, stream/order checks, native-without-raw coverage, privacy, temporal eligibility and post-extraction projection limits.

The adapter affects no agent-facing workflow, public API, Rust runtime, prompt, provider configuration or benchmark launcher. No real-agent scenario or Cargo verification is appropriate for this isolated offline unit. `docs/architecture.md` was inspected; its runtime ownership and documented behavior are unchanged. Independent source review and a root-owned fixed actual execution scope remain pending. Synthetic GREEN is not an actual C08 item-retention finding or an admission to extract saved charges.
