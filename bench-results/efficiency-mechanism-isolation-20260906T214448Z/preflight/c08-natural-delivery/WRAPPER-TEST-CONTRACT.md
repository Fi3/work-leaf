# Finite source-wrapper tests

`test_sources.py` exercises the private wrapper with independent temporary
sources and pinned static code only. It reads no saved workflow payload, native
rollout or accounting output. Actual three-run invocation requires the root's
separate fixed source scope and final review; passing these tests does not
authorize it. [SOURCE-VALIDATION.md](SOURCE-VALIDATION.md) records the test-first
receipts and exact reviewed cut.

## Private boundaries

- `collect_closed_captures` calls unchanged `collect_captures` and full raw-byte
  `prove_frames`; independent directory/metadata checks cover every invocation
  kind. `check_invocation_streams` binds raw bytes and typed `meta.json` to each
  terminal. The smallest unit start omits execute-level provenance fields; it is
  not a complete observer/executable qualification.
- `check_native_membership` covers typed accepted thread/turn identities,
  zero-turn and usage-less threads, explicit native contexts and typed replay
  equality. It does not infer owner, usage or action semantics.
- `check_turn_closure` and `check_project_inventory` bind terminal/grace and
  snapshot/journal records. Failed workflows do not acquire invented final
  snapshots. Recorded boundaries are not continuous immutability evidence.
- `check_generated_profile` reconstructs only the source-pinned ASCII-path
  subset of the generated launcher's bytes. It executes no shell or provider.
  The real binary endpoint, generated path, report/config/start hashes and
  retained empty recursion log have separate checks.
- `execute` checks all three original terminal rows and exact scope, source,
  output limits, publication relation, executable, report and project contracts.
  A complete synthetic fixture calls the real source/frame/join functions,
  without a supplied `valid=true` proof or mocked native/frame validator.
- `main` reserves an exclusive output and durable once-marker before execution
  using Python's own writer. Missing parents, symlinks, existing destinations,
  preparation failure and final-publication failure are tested. Publication
  failure retains the marker and prohibits a transparent second execution;
  this is not crash-proof storage.

## Coverage and limits

The 31 source tests cover malformed tails/JSON, source drift/limits, directory
population, missing captures, metadata and full-frame mutations, all-kind raw
stream/end joins, native membership/context identity, grace/terminal closure,
project snapshot population/timing, stage-to-published path constraints, original
object/array reports, generated-profile source and field mutations, exact output
limits, complete execute/CLI behavior and once-only publication.

Static dependencies include the old collector/frame helper and its original
collector/primitives, the frozen pure v7 helper/primitives, and three exact driver
source files. Their original filenames and captured code bytes are used. The
negative frame fixtures rebind their temporary upstream digests to reach the
intended frame predicate; no actual capture is rewritten.

Source limits are inherited and explicitly scope-bound. Output limits apply
after complete safe-result construction and canonical hashing, include the LF,
and never silently clip; they are not general peak-memory bounds. Identity
lookups use maps/sets. Directory and canonical-output ordering use sorting, with
no introduced production quadratic history scan. Original observer, runtime,
baseline and accounting files are outside test ownership.

This helper is offline-only and changes no agent-facing workflow; no real-agent
verification is appropriate for these synthetic source tests. Full source
qualification of the already closed natural workflows remains a distinct task.
