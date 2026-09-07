# Workflow 003 citation-owner revision

The current semantic draft is `work-units-01-workflow-003.revision-01.json`,
SHA256 `340d3df991be717e31bea8e6232dce904d63b8419e55a9d4f20701f4ba06322c`.
The retained original is `work-units-01-workflow-003.json`,
SHA256 `11c3d3b46cdb5590d0ad6ab10e00846acfdb9505a24f0fad13c10fe07da61e26`.

Independent review identified a nonexistent owner name in the patch-source evidence
locators for ACKs 2, 3 and 9. The retained phase source
`infrastructure/evidence/src/patch.rs` declares `GitPatcher` at line 54,
its implementation at line 59, and `apply_edit_with_locks` at line 150.
The source SHA256 is
`aa9a0df515aa7977a8b52b7dfd6151d53ee424adb7f7d5f5a83d0ba339aa5fda`.

The revision differs from the original only by three exact locator substitutions:
`PatchApplier::apply_edit_with_locks` → `GitPatcher::apply_edit_with_locks`.
Byte comparison verifies that every other byte is identical. All 15 ACK identities,
labels, rationales, evidence paths/hashes and source line numbers are unchanged.
The original and packet remain retained; no frozen source, protocol, provider workflow,
or other run evidence is modified. Independent delta review is required before freeze.
