# Independent declared-output selector review

Verdict: PASS for the provider-free selector cut; no introduced blocking finding.
This is not bridge/configuration admission, private-command qualification, an
actual-agent result, or a correction to any completed workflow.

## Reviewed identities and checks

The complete helper, actual `test_selection.py`, qualification note, prospective
derived-output design and original selector contract were read. The full helper
diff against its immutable predecessor and the relevant architecture/ownership
sections were inspected.

| Source | SHA-256 |
| --- | --- |
| `live_selection.py` | `ddfa7b6eb99bf713669caf5d3766aaa11843b24057b86329fd6985b83733dd5f` |
| `test_selection.py` | `9adae3746498dec664e107f08a7e81c5fcb668a9a3917ed9367f86dc724aaf5d` |
| `SELECTOR-VALIDATION.md` | `c60ede3fc483bbde4bb72ea80461b9cc568bf47fce57ae6ed01aaf1929a3670f` |
| `../../DESIGN-PRIVATE-PREVIEW-DERIVED-OUTPUTS.md` | `605cefb6665385833d14ff0d82d7e3213cef985c1b7eb8e60f4ae3fa8cc0b504` |
| Original selector contract | `e56863a887575ce5dab64277766a225f372d96b533c635e102d76ebdbeef311d` |

Independent `python3 -B -m unittest -q test_selection.py` from this directory
passes **18 tests in 3.615 seconds**, tool `f3c108`, exit 0. These exercise real
temporary Git repositories and private materialization, not Cargo, namespace
execution or providers. The original strict ignored-output rejection remains an
explicit passing test. Historical RED results are owner-reported in the reviewed
validation note; this review did not recreate them by reverting source.

Before/after hashes of the new sources/design and original selector/bridge match.
Endpoint check `704630` also confirms the pinned materializer
`87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2`
and Git executable
`292115a21c70326a0fa239e6d9bdee32f750cbd23d12fdef2e518ad4b451a8b3`.
The unchanged predecessor selector/bridge remain `0614db9a…` / `6af6c3f2…`.

## Source contract

`derived_declarations` (line 87) requires explicit normalized directory roots,
rejects overlap/aliases and accepted, staged or overlay descendants, and bounds
the declaration/path work. `derived_inventory` (line 126) independently requires
every untracked leaf to be Git-ignored and inside those roots. Regular single-link
leaves are metadata-only; nonignored entries, unknown ignored locations, untracked
ignore administration and local external-exclude configuration fail closed.

The NUL-delimited `check-ignore` join retains an exact rule witness per excluded
path. Its authority is accepted/effective tracked `.gitignore` or `.git/info/exclude`,
not a filename heuristic. Authority bytes are sampled around classification and
at census completion. The pinned Git wrapper clears inherited configuration;
ordinary accepted/index/overlay/admin checks remain in the census.

`capture_selection` (line 258) separately retains both derived-output endpoints
while requiring equality of every other census field. Tracked rule identities
remain in `files`, and config/exclude identities remain in administration, so
excluding the derived metadata field does not exempt their byte drift. The real
bundle/materialization test preserves the live output and excludes it from the
accepted/private images. `materialize_selected` requires the new v2 schema and
exact receipt/bundle identities; original-live and owned-accepted identities stay
separate. Default `census` remains structurally equal to the predecessor; the new
module's selected receipt is deliberately v2, not a forged v1 record.

## Limits and remaining gates

No pairwise file/history scan or O(F²)/O(E²) path was found. Membership is indexed,
inventories are sorted, and ancestor work is bounded to depth 64 and 4096 path
bytes. Strictly, constructing/stringifying all ancestor prefixes can require
O(D²) character work in path depth; this is a fixed-bounded path qualification,
not an unbounded quadratic population join. Git traversal, collected-output and
postcreation bundle limits remain the documented resource qualifications.

The selector does not prove operator authority for a particular build-output
declaration, exact executing-caller identity, a daemon lock, continuous source
atomicity or live/private environment equivalence. A future pinned v2 bridge must
bind the configuration/declaration/selector/selected record and retain failure
semantics. The broader observer/grace and actual-agent gates remain pending.
No current runtime, public API or ordinary command path is modified by this
private selector prerequisite; its local design and validation documents describe
that boundary. Only this new review file was written by the reviewer.
