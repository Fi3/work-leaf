# Batch 1 API Credit Failure

## What Happened

Batch 1 started `direct-001` and `wl-000-001` at approximately 02:31 CEST on 2026-08-29. Both
workflows reached GPT-5.5 through the exact-usage proxy and initially received successful responses.
The provider then began returning failed stored responses with this error:

```text
code: credit_balance_exhausted
message: You have no credits remaining.
```

The failed responses have no token usage. A direct provider lookup confirmed the same status and
error for one direct response and three Work Leaf responses. The batch was stopped at approximately
02:38 CEST because further calls could not succeed.

## Preserved Evidence

At interruption:

| Attempt | Exact completed responses | Failed responses without usage | Requests still unfinished at stop | Exact partial raw tokens |
| --- | ---: | ---: | ---: | ---: |
| `direct-001` | 7 | 1 | 1 | 358,624 |
| `wl-000-001` | 20 | 3 | 3 | 801,633 |

The failed response IDs are preserved in each attempt's `exact-usage/responses.jsonl`. The started
requests and response identities are preserved in `exact-usage/requests.jsonl`. The temporary
checkouts, benchmark logs, admissions, and partial provider records remain on disk.

Neither workflow reached candidate publication, final validation, or quality scoring. The partial
token values are not benchmark observations and must not enter Point 8 or Point 9 calculations.

## Decision

No later batch was launched. `direct-001` and `wl-000-001` remain immutable interrupted attempts;
they are not deleted or silently retried. After API credits are available, separately named
replacement attempts must run through the same frozen launch contract. Production Work Leaf, the
task, evaluator, model settings, and benchmark implementation remain unchanged.
