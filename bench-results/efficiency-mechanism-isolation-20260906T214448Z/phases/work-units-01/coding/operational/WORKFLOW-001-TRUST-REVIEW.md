# Workflow 001 first trust-transition review

## Result and scope

The first recorded trust transition for `work-units-01-workflow-001` passes the
independent, read-only RPC/native-prefix and project-inventory review. The review
uses the exact frozen classifier source bytes. It is an operational integrity
review, not a benchmark outcome, token comparison, mediator classification, or
whole-phase analysis.

The retained phase manifest SHA256 is
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
The frozen classifier SHA256 is
`37122d613c96b21fc4cd078ad8dfd7784e0fa60c1f430bb2ff78cf6d82ea88b8`.

## Exact identities

- Workflow: `work-units-01-workflow-001`.
- Pending ID: `d83dc29648c17d63356a390238457744d0a8113b2df5cc3cd8f8fab5ba2b2691`.
- Verified proof ID: `0e15fdeba140bbd5615b0d2a7ae3a37c93afad2034c5410ddbc06e895d1882c7`.
- The proof's `resolves_pending_id` equals that pending ID; both record hashes independently match their contents.
- Primary invocation: `00000441344929586777-3537090`.
- Typed RPC request identity: string `82`, not numeric `82`.
- Provider thread: `01a07966-5d8b-7e00-929d-303d37d7ec9c`.
- Exact owned working directory: `/tmp/work-leaf-mechanism-work-units.PoGOl1/work-units-01-workflow-001/work-leaf-3feature-bench.7XcXDh/repo`.

The original full-access request, its allowed raw-metadata-only forwarded form,
successful same-typed-ID reply, matching thread notification, native session
identity, native permission instruction and native turn context pass independent
semantic replay. This review does not infer a syscall-level writer identity from
the timing.

## Timing and preserved states

All times are UTC on 2026-09-07.

| Event | Time |
| --- | --- |
| Matching thread started | 01:05:46.957000 |
| Native session metadata timestamp | 01:05:46.989 |
| First pending configuration snapshot | 01:05:46.997017 |
| Native turn-context timestamp | 01:05:48.217 |
| Later verified configuration snapshot | 01:05:57.064451 |
| Fixed pending deadline | 01:06:16.997017 |

Resolution takes **10.067434 seconds**, within the fixed 30-second deadline.
The native turn context genuinely postdates the first pending cutoff. The initial
seven-source core proof therefore does not pretend to contain native attestation;
the later eight-source verified proof supplies that separate evidence.

The initial snapshot retains `pending_own_workflow_native_attestation`,
`allows_admission: false`, and `pending_transition: true`. The later snapshot
separately records `verified_own_workflow_trust_transition`,
`allows_admission: true`, and `pending_transition: false`.
Both snapshots, and the inspected journal through 01:19:19.275311 UTC, retain the
original `behavioral_or_unknown` legacy label. No legacy flag is retrospectively
cleared. There are no `before-launch:` journal snapshots during this pending
interval.

## Project-layer evidence

The four retained snapshots are `checkpoint:base`, `pre-spawn`,
`checkpoint:pre-linearize`, and `checkpoint:final`. All four have
`valid: true`, empty errors, and independently recomputed entry SHA256:

`08b328252e86541a1d1343ad84f1e599eb6a2bb9fa9334a0564ed04e3eb29370`.

The pre-spawn record precedes the actual child start, and the pre-linearize
inventory precedes the captured full-access transition. Snapshot equality is not
a claim of continuous filesystem immutability.

## Safe source locators and prefix identities

Proof records are in phase-local `trust-pending.jsonl` and
`trust-evidence.jsonl`. Their original capture-time staging locators remain in
those records. The publication-resolved observation directory is:

`/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/work-units-01/runs/work-units-01-workflow-001/work-units-01-workflow-001-three-feature-bench-artifacts/observation`.

In the table, `<observation>` means that publication-resolved directory.
Each digest covers exactly the listed byte prefix, **not** a claim that the
eventual whole file has that SHA256. Prefix verification survives the retained
staging-to-publication rename. The eight verified prefixes total 15,905,005 bytes.

| Record | Locator | Prefix bytes | Prefix SHA256 |
| --- | --- | ---: | --- |
| proof | `<observation>/observer-config.json` | 1612 | `7d3fb5a6b786e126767cbd2180582bad3355319f3e6610333fc528f59459a128` |
| proof | `<observation>/invocations/00000441344929586777-3537090/start.json` | 1307 | `c78c285fe28e730e324e7269f16af790a1368fee8d6cb7aab774ed1daf3e0c30` |
| proof | `<observation>/invocations/00000441344929586777-3537090/child.json` | 115 | `c3302cef820c002a98419297f64d53863de16eba4645a2c1a67b7dc8e6cb11ac` |
| proof | `<observation>/app-server/00000441344929586777-3537090/client-to-server.raw` | 453770 | `ede300ba0655ea4d6940d9a47f1b001af1e2c8763cb33a8f5599ecf4c9584e10` |
| proof | `<observation>/app-server/00000441344929586777-3537090/client-to-server.forwarded.raw` | 454060 | `716c5ef48b37edde41a8afeabb0f11dd1b687d2607c4e641c0c27e307d31f2a6` |
| proof | `<observation>/app-server/00000441344929586777-3537090/server-to-client.raw` | 14922619 | `603aa1352838b29390e56238191b2da46fd9d51263882ff37db33659b7d2337b` |
| proof | `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T03-05-46-01a07966-5d8b-7e00-929d-303d37d7ec9c.jsonl` | 66540 | `4128fcb4a9c0ac0ab71d11df01174ddd3dc45fe32263fc862d0f1f240cf0b3e0` |
| proof | `<observation>/project-layer-inventory/manifest.jsonl` | 4982 | `c95aa4528bf10d1783c301645fc28676d3d66bcc9ab4e991671368dfaf9fcf45` |
| pending | `<observation>/observer-config.json` | 1612 | `7d3fb5a6b786e126767cbd2180582bad3355319f3e6610333fc528f59459a128` |
| pending | `<observation>/invocations/00000441344929586777-3537090/start.json` | 1307 | `c78c285fe28e730e324e7269f16af790a1368fee8d6cb7aab774ed1daf3e0c30` |
| pending | `<observation>/invocations/00000441344929586777-3537090/child.json` | 115 | `c3302cef820c002a98419297f64d53863de16eba4645a2c1a67b7dc8e6cb11ac` |
| pending | `<observation>/app-server/00000441344929586777-3537090/client-to-server.raw` | 453770 | `ede300ba0655ea4d6940d9a47f1b001af1e2c8763cb33a8f5599ecf4c9584e10` |
| pending | `<observation>/app-server/00000441344929586777-3537090/client-to-server.forwarded.raw` | 454060 | `716c5ef48b37edde41a8afeabb0f11dd1b687d2607c4e641c0c27e307d31f2a6` |
| pending | `<observation>/app-server/00000441344929586777-3537090/server-to-client.raw` | 14922619 | `603aa1352838b29390e56238191b2da46fd9d51263882ff37db33659b7d2337b` |
| pending | `<observation>/project-layer-inventory/manifest.jsonl` | 4982 | `c95aa4528bf10d1783c301645fc28676d3d66bcc9ab4e991671368dfaf9fcf45` |

## Deferred checks and non-actions

Recorded normalized parsed hashes equal the frozen baseline hash. Global
configuration contents are not read during this review; independent reconstruction
of the complete external global configuration remainder and every historical
parsed snapshot remains deferred until phase closure. That full replay requires
the matching external final global file and is not made self-contained by these
hashes.

No credential material is inspected or copied. No whole-phase analyzer, token
total, primary outcome, semantic classification, or interim contrast is inspected.
No provider is launched, interrupted, or reconfigured, and active workflows remain
untouched. This report preserves the completed bounded review; it does not rerun
that audit or alter any helper, protocol, or earlier evidence.
