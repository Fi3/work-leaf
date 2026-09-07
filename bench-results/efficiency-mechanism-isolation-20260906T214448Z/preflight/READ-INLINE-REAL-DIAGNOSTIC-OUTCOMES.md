# Initial real read-diagnostic outcomes

Both initial subscription diagnostics are retained failures, excluded from study observations. Their log exit is 101 after approximately 27 seconds. No retry was performed by this reviewer.

The actual read handoffs succeeded within the observed client/server streams: each case has one accepted launch-policy request and two accepted read requests, exact candidate/owned-body reconstruction, and three matching terminal interrupted turns. The unchanged v3 analyzer's pure `prompt_inventory` returned no errors, with bidirectional coverage of all three supported requests. Thread ownership for this limited replay is the single typed `thread/start` success plus the exact launch-policy `Agent-ID` footer and all matching request thread IDs; it is not a replacement for complete observer/native provenance.

| Diagnostic | Initial read request → typed reply | Selected initial read | Repeated read request → reply | Final terminal turn |
| --- | --- | --- | --- | --- |
| `read-inline-control` | client line 6 → server line 27, RPC `5` | Baseline bundle manifest, 422 bytes | client line 8 → server line 47, RPC `7` | server line 62, `01a07a7a-496a-7bb2-a64e-69e82b7c1b7e` |
| `read-inline-untracked-read-inline` | client line 6 → server line 25, RPC `5` | Inline candidate, 26,931 bytes | client line 8 → server line 45, RPC `7` | server line 60, `01a07a7a-42e2-7f62-90f8-5b53e7a9c3aa` |

Both traces retain the same exact 26,890-byte snapshot body, SHA-256 `83520a6a845145a5a87ed1fa8eca0d880be80242257f4a04e3c50fd5788f9f18`, and both complete candidates. Their repeat response is identical, 258 bytes, SHA-256 `4aacf692aa378820c8e5cbc1c2e7fb2d6dd1296a654737e8fef555d5c982ab85`. The initial policy is also identical. Candidate byte sizes are not measured token usage.

The operator omitted the primary-invocation marker. Both retained `start.json` records say `primary: false`, record a zero-millisecond usage grace, and have no raw-response-usage activation. Consequently neither capture contains the forwarded/raw-response metadata evidence required by the intended benchmark route. Both lack invocation `end.json`; `stop-app-server` reported no active primary invocation, and the smoke could not establish its required forwarding/closure receipt. The observed terminal turns do not cure these provenance and settings gaps.

Whole-accounting readiness is false. No token total, missing-usage recovery, compaction correction, primary endpoint, or treatment-effect estimate is produced from these attempts. A separate corrected diagnostic admission is required; these original failures and sources remain unchanged.

Exact response/turn/request locators, candidate/body identities, original flags, capture status and endpoint source hashes are retained in `READ-INLINE-FAILED-DIAGNOSTIC-DELIVERY.json`. The replay used `analyze_untracked_reads.py` SHA-256 `30a58a9312e2f9c641643698f53e0592c392fa8ed5fa3c0d01d9cc61ff427c3e` without modifying it. No private reasoning bodies or credentials are included in these reports.
