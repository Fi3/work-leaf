# Corrected source-input identity

The original [source-input schema](../SOURCE-INPUT-SCHEMA.md), SHA
`e7dfec8c229e8a9e65fc58d0f98ad0f82d9902f48207f297595d9b3583a89054`,
defines all fields, source limits and validation boundaries. The JSON schema
names remain `work-leaf-c08-natural-source-input-v1` and
`work-leaf-c08-natural-source-scope-v1`; no capture/native/trace payload is
modified for this derivative.

The root freezes distinct corrected scope/input/output paths. Every corrected
input's `helper_sha256` is
`7ea311907b975f25b02215c3e750eb8d69b49311c30247db6967862fe44ed092`.
The CLI argument pins that new input, whose `scope` reference pins the corrected
scope. Its input/output maps must contain the exact new paths. The existing
exclusive output/attempt guard applies; original attempts or results are never
reused or replaced.

The new helper verifies its own physical source and the original parent helper
`d05e7e7daa62601782fbcca2058f8c31a81d29ac5301407643474a09f4889f80`.
All original collector/frame/primitive/validator/join dependencies retain their
exact original physical paths and hashes. The prospective pre-execution closure
must therefore include both helper files alongside that unchanged dependency
set. No runtime, observer, accounting or actual-workflow source pin is revised.

The new result remains a source-only qualification. Original failed source
receipts and observer/accounting outcomes remain independent retained records,
not successes under another filename. Any actual corrected three-row invocation
requires the root's separate frozen scope and is not authorized by this schema.
