# Untracked project-read representation

Status: implementation design for a distinct benchmark-only factor. This document does not admit
provider observations, draw an allocation, or spend a confirmatory testing allocation. A separate
frozen protocol and phase manifest govern those actions. Earlier studies remain immutable.

## Question and source boundary

Does returning full requested source text in the conversation, instead of a context-bundle
manifest with on-demand access, increase the input carried into subsequent model responses?

`src/orchestrator.rs::send_file_read_response` reads requested files under the ordinary locks and
calls `split_repeated_file_reads`. Its `exact_snapshots` bucket contains paths without a tracked
snapshot for that agent. It is not strictly a first-ever-read bucket: accepted authored edits
clear affected snapshots, allowing later requests to enter that bucket again.

`render_file_read_response` ordinarily bundles that bucket when its total UTF-8 text exceeds
24 KiB or any single snapshot exceeds 16 KiB. Successful `ContextBundleStore::write` leaves the
exact text available in the normal temporary bundle; the delivered component is a compact manifest.
Write failure falls back to the existing inline formatter. Tracked changed and unchanged reads
are separate components, and explicit requests for bundle files are a separate prefix.

The condition names are `control` and `untracked-read-inline`, admitted only through a new private
v3 experiment schema. The treatment replaces the successfully bundled untracked-project component
with `render_file_read_response_inline` applied to the same already-held exact snapshots. It does
not expand the requested path set or invent source content.

## Isolation contract

Ordinary baseline rendering runs once in every condition. Bundle creation, its counter, paths,
content, failure/fallback behavior and lifetime are unchanged. Both active v3 conditions construct
the inline candidate from held snapshots after that rendering. No second filesystem read, second
bundle allocation, new provider request or different request coalescing is part of the experiment.

The renderer exposes its owned exact-component byte range and whether bundle writing succeeded.
The final delivery range accounts for any existing explicit-bundle prefix. Only that range can
differ; the prefix, tracked-repeat sections, failure diagnostics and all other bytes are shared
unchanged between the two candidate strings. Empty, small, repeat-only, explicit-bundle-only and
bundle-write-failed responses are ineligible and have identical candidates.

Tracking, snapshot clearing, `--force` compatibility, permissions, locks, event order, follow-up
dispatch, automatic conflict refreshes and command output retain their normal behavior. Launch
policies, patch acknowledgments, validation, review, linearization and final checks are unchanged.
The work-unit-policy treatment is not active in v3. There is no file-name, benchmark-feature,
temporary-path or observed-output special case.

Full text replaces a compact representation; its direct input cost and any consequent changes in
inspection, responses, compaction or failure are the factor under test. No padding, forced extra
inspection, artificial truncation or treatment-specific context/model setting is permitted. Large
request failures are retained outcomes, not grounds for clipping, exclusion or automatic replacement.

## Evidence and compatibility

Default builds exclude all experimental hooks. Feature-enabled inactive builds retain ordinary
behavior. Active v1 and v2 manifests preserve their existing accepted conditions, evidence schemas
and transformations. Invalid v3 activation or evidence-writing failure prevents delivery.

Every active v3 read event constructs and records both complete candidate strings in the same field
order, then records the selected candidate identity and byte length. It does not serialize a second
large forwarded copy only in treatment. The delivered provider string must equal the selected saved
candidate exactly. Snapshot metadata includes ordered normalized paths, byte lengths and the normal
FNV64 digest, plus owned byte ranges and baseline bundle success/path. Offline evidence hashes use
SHA-256; FNV64 labels are never described as cryptographic hashes. No credential or private reasoning
body belongs in this evidence.

The independent delivery audit joins actual accepted typed RPC requests, thread/turn identities and
native user-item identities. Per-response input attribution follows those items and actual later
bundle-tool results; repeated charging is not repeated execution. Candidate byte differences are not
claimed to be measured token differences. Missing response usage and compaction scope remain explicit.
Symmetric instrumentation is structural, not a claim that wall-clock time or stochastic work is equal.

## Required checks

New tests run RED before implementation. Coverage includes exact default/control identity; v1/v2
compatibility; threshold boundaries; mixed untracked/repeated/missing reads; duplicate/coalesced paths;
separate agents; snapshot clearing after edits; `--force`; empty/non-ASCII/no-final-newline text;
explicit bundle reads; unchanged automatic refresh/command/policy output; successful and failed bundle
writes; unchanged later allocation paths; identical non-factor prefixes/suffixes; candidate selection;
and activation/trace failure propagation before send. Existing committed tests remain unchanged.

Required format, all-target/all-feature clippy with warnings denied, and all-target/all-feature tests
must pass. Independent review checks architecture and documentation plus complexity: rendering and
evidence construction must be linear in the captured bytes and snapshot count, not pairwise copying.
A bounded real subscription-backed scenario must verify actual bundled control and inline treatment
delivery, ordinary continuation and non-target behavior. Diagnostic identities are not study data.

This factor can establish an effect of source representation and its downstream context use. It
does not by itself identify a unique share of the historical approximately 50% workflow difference;
that bridge requires actual action/response/input evidence rather than subtraction of unrelated means.
