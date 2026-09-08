# Closed-source input contract

Implementation-stage contract; not an actual-run admission. Root owns the exact
fixed scope and all three input/output paths. A reference below always means
`{"path":"/exact/canonical/file","sha256":"<64 lowercase hex>"}`. Source
assembly uses only the three closed automatic-refresh rows; no baseline or
accounting entry is accepted here. Final helper/source hashes and independent
review remain prerequisites before actual CLI invocation.

## Scope and per-run input

Scope JSON:

```json
{
  "schema": "work-leaf-c08-natural-source-scope-v1",
  "run_ids": ["automatic-refresh-01-workflow-001", "automatic-refresh-01-workflow-002", "automatic-refresh-01-workflow-003"],
  "condition": "automatic-changed-refresh-full",
  "phase_manifest": {"path": "<PHASE-MANIFEST>", "sha256": "<sha>"},
  "input_paths": {"<each run ID>": "<its absolute new input path>"},
  "output_paths": {"<each run ID>": "<its absolute exclusive output path>"},
  "limits": {"max_file_bytes":1073741824,"max_total_source_bytes":34359738368,"max_sources":20000,"max_output_rows":100000,"max_output_bytes":33554432}
}
```

The CLI separately pins the exact input bytes, avoiding a scope/input digest
cycle. The scope can contain additional provenance, but these identities are
mandatory. Each input has:

| Field | Value |
| --- | --- |
| `schema` | `work-leaf-c08-natural-source-input-v1` |
| `helper_sha256` | Final reviewed `audit_automatic_refresh_sources.py` SHA |
| `run_id` | One of the exact three scope IDs |
| `scope` | Scope reference |
| `phase_manifest`, `phase_result`, `score_manifest` | Original exact source references; all three final rows must agree with schedule and each other |
| `additional_sources` | Complete array of other immutable runtime/build/admission/trust/configuration/source references required by the root-frozen closure; the phase's `files` are also read and verified |
| `publication_sources` | Object with `bench-candidate-common` and `bench-three-features` references at exact source hashes `2971cb9729626ddc7a12bd688df8d80ee20f904bb48d52ab846c1e9f9751711f` and `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644` |
| `generated_profile` | Object with `renderer`, `profile`, `recursive_log` references: exact `bench-agent-profile-common` SHA `de9803658bc8ac41a9356436be2c8b8edfce9d7bef9176841e067b3478633d4f`, published `agent-profile.txt`, and published `recursive-codex-attempts.log` |
| `experiment`, `trace` | Original v7 manifest and full trace references at exact scheduled paths |
| `observer_config` | Original configuration file reference at the published observation root |
| `invocations` | Published `observation/process-invocations.jsonl` reference |
| `captures` | Every app-server capture, in the object shape below |
| `invocation_metadata` | Every app-server, exec-json and locked-command invocation, in the object shape below |
| `native_sessions` | Array of `{"thread_id":"<exact ID>","source":<reference>}`; includes all accepted/usage-less/zero-turn threads, never fabricated empty sources |
| `project_cwd` | Exact admitted project repository path for native contexts; non-app command cwd may be a legitimate nested test directory |
| `project_inventory` | Object with `manifest`, `baseline` references and `snapshots` array of every other original JSON snapshot reference |
| `original_reports` | Array of `{"kind":"<label>","format":"json|jsonl|text","source":<reference>}`; original report/analysis/extraction errors remain original source facts, not corrected in place |

All references point to actual published files. The original config's `root`
is retained separately. The only permitted relocation is the exact declared
results directory's `.bench-artifact-publish.<32 lowercase hex>/observation` to
that same directory's exact `<run-id>-three-feature-bench-artifacts/observation`.
The pinned driver contract and original final phase/report publication are
required. This is source-grounded publication compatibility, not independently
retained proof of historical inode equality. No captured frame or configuration
string is rewritten.

## Capture and invocation objects

An app-server `captures` object contains `path` (the actual published capture
directory) and these references: `clients`, `forwarded`, `servers`, `start`,
`child`, `end`, `settings`, `journal`, `grace`. Exact filenames are:

```text
app-server/<invocation>/client-to-server.raw
app-server/<invocation>/client-to-server.forwarded.raw
app-server/<invocation>/server-to-client.raw
app-server/<invocation>/raw-response-usage.json
app-server/<invocation>/raw-response-rewrites.jsonl
app-server/<invocation>/provider-usage-grace.jsonl
invocations/<invocation>/{start,child,end}.json
```

Each `invocation_metadata` object contains `start`, `child`, `end`, `meta`
references plus `streams:{stdin:<ref>,stdout:<ref>,stderr:<ref>}`. `meta` is the
actual capture directory's `meta.json`. App-server streams use the names above
and `server-stderr.raw`; `exec-json` and `locked-commands` use `stdin.raw`,
`stdout.raw`, `stderr.raw`. These bytes must match every terminal stream digest.
The complete three capture-kind directory populations are inventoried, not just
the app-server subset. No non-app source is dropped because its cwd differs
from the project root.

`process-invocations.jsonl` contains the typed `InvocationStart` projection;
`start.json` additionally carries source-owned `raw_response_usage` and
`project_layer_inventory_required` metadata. Only these exact supplemental
fields are projected out for the typed start comparison and are validated
separately. They are not silently removed from the original retained record.

Project inventory references point to `project-layer-inventory/manifest.jsonl`,
`pre-spawn-baseline.json` and all individual snapshots. Full snapshot/journal
bijection, original validity, entry digest and each primary pre-child time are
checked. Global-config trust/measurement exceptions remain separate original
flags; recorded project boundaries do not prove continuous immutability.

Source limits remain the inherited 1GiB/file, 32GiB/unique-source sum and 20,000
sources. Safe output limits are 100,000 metadata-list entries and 32MiB, checked
after constructing/hashing the complete canonical safe result, before
publication, including the published trailing newline. The CLI passes the exact
validated scope limits to the finalizer; a pre-scope failure uses the same fixed
limits only for its unqualified failure receipt. These are not peak-memory
guarantees. Output reservation and the
durable once-marker precede execution; no transparent retry follows a failure.
No external writer is needed.

Every retained executable's resolved target must appear in the phase `files` or
`additional_sources` with the exact digest before executable bytes are read.
Literal configured paths and
resolved target paths are both retained; resolution is checked again at closure.
This allows source-bound symlink chains without reading arbitrary undeclared
executable paths. `additional_sources` must therefore include all required real
Codex/shell/Cargo targets as well as the complete agreed 175-input provenance set.

The generated Codex profile is a separate relation, not an extant-file endpoint.
Its eight retained fields bind the original path
`<project parent>/codex-profile/codex`, recursion-log path, actual binary,
model/effort and both hashes. The helper reconstructs the pinned renderer's bytes
without a shell or launcher call. Its deliberately limited `%q` support accepts
only normalized absolute ASCII paths using letters, digits, `_./-`; unsupported
quoting is a failure, not an approximate reconstruction. The generated SHA must
equal the retained profile, observer config, report and every app-server/exec-json
start; the actual binary separately matches its admitted source, report
`actual_codex_sha256`/`codex_cli_path`, and retained profile. The published
recursive-attempt log must be empty. The original absent generated path is never
fabricated, resolved to another file, or inserted into the endpoint source set.

Source passes and identity joins are indexed; sorted directory inventories and
canonical output serialization introduce sorting, not a quadratic history scan.
Recorded project boundaries, terminal notifications and grace journal outcomes
remain finite observations, not continuous-state, fresh-usage or action-semantic
proofs. Original errors are preserved even when this separate source relation
is available.
