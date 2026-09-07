# ACK semantic-coding procedure

This is an operational elaboration of the frozen labels and evidence requirements in
`PROTOCOL-WORK-UNITS.md`, not a new outcome, allocation rule, or protocol amendment.
The layout inspection uses only completed `phases/screen-01` artifacts. It assigns no
labels to that screen and inspects no active work-unit outcomes or condition contrasts.

## Unit and preparation

The coding unit is each accepted successful ACK in
`analyze_work_units.py::prompt_inventory(...)["acks"]`, not a commit, changed file,
assistant item, completed ACK-to-done episode, or model-response count. A primary ACK
remains present after a later generation failure. The distinct `work-leaf patch already
applied` prompt is not a successful ACK. Every primary ACK receives exactly one final
label; semantic uncertainty never removes its automatic primary count.

For a terminally published workflow, verify its frozen manifest row, trace, observer
provenance, and complete bidirectional prompt inventory before preparing a packet.
Use the pure inventory function to collect identities without computing contrasts.
Do not run the whole-phase inference path to guide coding. An incomplete or unverifiable
inventory is an inventory failure, not an empty list or an unresolved semantic label.

Code completed workflows in frozen run-ID order and ACK request order, without inspecting
condition averages, token totals, p-values, or a desired number of increments. Omit condition
labels and A/B/C treatment prose from coding packets where they are not needed for identity
checking; do not claim complete blinding if other context reveals assignment. Keep stage and
agent identities because ownership and review routing matter. Freeze each completed packet
and its labels; retain any correction as an explicit revision with its evidence-based reason.

## Identity and evidence packet

1. **Anchor the accepted ACK.** Preserve the helper's `ack_id`, `capture`, typed
   `rpc_id`, `thread_id`, `turn_id`, `agent_id`, `client_line`, `reply_line`, `trace_line`,
   `sequence`, prompt hashes and original `files_receipt`. Cite the full-file SHA256 and
   one-based physical JSONL line plus JSON pointer for each raw request, accepted reply,
   and trace row. Preserve RPC number versus string identity. The helper already constructs
   `ack_id` from run/capture/RPC/thread/turn identity; do not invent a parallel identifier.

2. **Find the preceding submitted edit group, not the ACK's generated response.**
   The ACK acknowledges earlier edits; its accepted turn is where the next action begins.
   Index raw `item/completed` messages by capture/thread/turn/item ID and retain the exact
   structured-edit or unified-diff body locations. Reconstruct the same agent's accepted
   request/reply sequence and returned directive prefix using recorded messages and ordinary
   source control flow (`src/orchestrator.rs:685`, `src/cli.rs:1035`). Successful directives
   in one processed reply can share one ACK (`src/orchestrator.rs:717`, `982`).
   Account for intervening read/command/rejection follow-ups and routed messages rather than
   attaching the nearest patch text by timestamp. Raw items after the ordinary interrupt
   may not have been processed: do not include them merely because they precede a later ACK.

3. **Establish the content and prior state.** Compare candidate saved edits with the
   ACK's file receipt, captured file-read/context-bundle content and digests, prior accepted
   work, and the frozen feature request. Record what requested behavior was absent before
   this group and what the submitted changes implement. Original task and instruction
   bytes are available in frozen driver/evidence files and launch requests. Saved Git
   checkpoint manifests, source-context excerpts, format patches and bundles are corroboration,
   not automatic application receipts. The final bundle may contain only linearized history;
   a reflog hash does not prove that the old commit object or its complete diff is retained.
   Use exact objects/content when available; otherwise state the missing prior-state link.

4. **Identify the reason for repair from actual prior evidence.** Locate the same
   agent's earlier failing command result or delivered reviewer-fix prompt and match the
   failing behavior to the edit, not merely its reason/title. Captured reviewer requests
   include scope/commit context; fix prompts quote findings (`src/cli.rs:1187`, `1266`).
   For validation, cite the delivered command-result prompt's status/stdout/stderr and the
   original command directive. A later passing check corroborates the repair but cannot
   create a preceding failure. A general final gate does not establish why an earlier edit
   occurred. Locked-command `meta.json` plus stream hashes corroborate execution; inspected
   screen metadata has no agent/turn join, so command text and timing alone are not an exact
   cross-process link when repeated or concurrent commands are possible.

5. **Record the next action separately.** Starting with the accepted ACK turn, preserve
   actual next directive/item IDs, any command/read result request and following accepted
   turn, until the next handoff or done as needed. This is the continuation mechanism chain,
   not a substitute for identifying the edits acknowledged by the current ACK. Do not infer
   one completed model response from one assistant item or accepted provider turn.

The driver has no separate typed `PatchApplied` log. `final-state.json` session `lines` and
human-readable commit messages help locate content, but their order alone is not a typed
commit-to-ACK identity join. Do not manufacture such a join. Record a supported candidate
group with exact source locators and the basis for uniqueness; retain all competing candidates
when uniqueness cannot be established. Missing group/prior-state evidence makes coding unresolved.

## Frozen labels

| Label | Required content/evidence finding | Secondary indicator |
| --- | --- | --- |
| `remaining-work` | The acknowledged group implements genuinely remaining requested feature work, supported by requirements and prior state; it is not a mixed repair group. | 1 |
| `validation-repair` | It repairs a demonstrated validation failure in already submitted work, with no new requested feature work. | 0 |
| `review-repair` | It addresses captured review-requested repair/evidence, with no new requested feature work. | 0 |
| `formatting-only` | The changes are only formatting repair, demonstrated by the actual edits and prior state. | 0 |
| `no-op-replayed` | The accepted successful-ACK group is established as replay/no-op rather than new requested feature work. Do not use this label merely because an earlier attempt was rejected. | 0 |
| `unresolved` | Mixed purposes, conflicting candidates, missing prior state, or insufficient evidence to choose the frozen category. Explain the specific ambiguity. | [0, 1] |

Required tests and mutually dependent implementation in one coherent feature increment do not
become multiple units. A review request to add missing behavior can raise a real remaining-work
versus repair ambiguity; preserve it rather than decide from the presence of a review title.
Do not devise a new precedence rule to force mixed categories into a known indicator. No fixed
file count, patch count, desired number of units, treatment label, cost, or final quality score
is a coding criterion. All stages, costly repairs and failed workflow outcomes remain in scope.

## File/hash/locator contract and freeze

The final classification file has schema `work-leaf-work-unit-classification-v1`, the exact
`phase_manifest_sha256`, and `runs`, a map covering every frozen run ID. Each value is a list
of `{ack_id, label, rationale, evidence}` records. A genuinely empty verified ACK inventory
has an empty list; a not-yet-finished workflow is not preclassified as empty. Each evidence
entry is `{path, sha256, locator}` with a lowercase full-file SHA256 and nonempty locator.

Use absolute canonical paths inside the phase directory: the frozen analyzer resolves relative
paths against the repository, not the phase. Examples of precise locators are
`JSONL line 123; /params/input/0/text; typed RPC id string "7"`,
`JSONL line 456; /params/item/text; thread/turn/item identities ...`, and
`/snapshot/sessions/2/lines/19; session id ...`. Record actual values, not these placeholders.
Use blob/commit/path identity plus line or hunk offsets for retained source. The helper validates
path/hash and ACK coverage, but does not interpret locator semantics: the coder/reviewer must
check that the cited location supports the rationale.

A phase-local derived packet can retain a safe projection/index, source hashes, identity links
and uncertainty notes; cite the underlying retained sources too. Never copy credentials or
private reasoning bodies. External native files are not valid final evidence paths by themselves;
use retained phase-local evidence or a separately provenance-bound safe projection. Do not
modify active checkouts, retained captures, manifests, binaries, helpers, or frozen protocol files.

Before any condition contrast, independently check complete one-to-one ACK coverage, every source
hash and locator, labels/rationales, and all mixed/unknown cases. Freeze the classification file
and evidence inventory with hashes. The unchanged helper maps known remaining-work to 1, other
known categories to 0 and unresolved to [0,1]; its fixed-sequence gate controls whether the
secondary randomization test is used. Semantic coding does not change the primary or token bounds.
Index capture and identity records once; avoid rescanning the entire stream per ACK.

## Inspected layout witnesses

The following existing files establish layout only, not treatment results. Paths are relative to
`phases/screen-01/runs/screen-01-001/screen-01-001-three-feature-bench-artifacts/`:

- `observation/git-checkpoints/manifest.jsonl`: label/head plus a filename-to-hash map;
  SHA256 `1caefba168bac1e5c07dbbd05c3516dd92c6c93eacac03e47d7e7b8550d61322`.
- `observation/context-bundles/manifest.jsonl`: archived path, bundle SHA256 and file snapshots
  containing path/digest/byte metadata;
  SHA256 `662a33e7acb2bc3ca381e827aae2e9aa233464fb9d23bd3a385477370ff750c7`.
- `observation/locked-commands/00000434586042068497-3314728/meta.json`: start/end,
  invocation identity, argv/cwd and stream hashes, without a typed agent/turn field;
  SHA256 `09581f79550a32d5fae5039ae97e9e8d25ec246bf3a75bb06f3f06606714668c`.
- `final-state.json`: `/snapshot/sessions` entries contain `id`, `feature` and string `lines`,
  not typed per-message application events;
  SHA256 `c97dcc49bc593a936acb8e245a5a0caaad10547939c1c1fa7a1e199e627f8789`.

Real verification is unnecessary for this documentation-only procedure: it changes no executable
agent workflow and launches no generation. Frozen runtime verification remains the phase's authority.
