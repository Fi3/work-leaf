# BUG001 — corrected input representation, 2026-09-14

The actual input-002 base-instruction objects and developer-content arrays are exactly equal
to the saved reference. Both canonical base hashes are a22c77a33898c3d4f13d2521626357ad3afc3650d199aa0749f01d5113bff7b7;
both newline-joined developer hashes are 077f23d17df246409154a9190ab8d10ce56f12e90ec7ef682cce47ff2da2ffe7,
at 7,817 UTF-8 bytes. The first comparison mixed reference canonical-object/new text-only
base hashes and reference newline joins/new concatenation. That was a report-verifier defect,
not a model-input difference.

The preliminary comparison is retained as ACTUAL-INPUT-COMPARISON-002-PRELIMINARY.json.
INPUT-VERIFIER-CORRECTION-002.json records exact object equality and both matching representations.
The corrected ACTUAL-INPUT-COMPARISON-002.json is authoritative for input qualification.
NATIVE-AUDIT-002.json remains the immutable once-only usage audit; its embedded preliminary
input booleans are superseded by the correction, not by another accounting replay.

The two regression checks fail first in BUG001-RED.txt and pass in BUG001-GREEN.txt.
Native usage remains 15,873 raw, one response, zero audit errors, zero model tools and all
74 admission source hashes unchanged. No generated response was repeated. This report-only
fix changes no agent-facing behavior; the existing real diagnostic supplies its input evidence.
The extra defect is handled under the user's standing bug-fix authority, not added research scope.
