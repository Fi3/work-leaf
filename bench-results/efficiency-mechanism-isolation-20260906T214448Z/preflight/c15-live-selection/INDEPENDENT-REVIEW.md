# Independent live-selection prerequisite review

Root reviewed the full helper, 14 tests, real shared-table Rust caller, private
Cargo manifest, design and qualification. Reviewed source identities are
`0614db9a…`, `f2998b3e…` and `6af07137…`; the retained actual qualification is
`b99dde0702b6e9d5153707bc250ca5a56a493a2b934c3e02e5f6f4e3417a890f`.

The review found and reproduced missing timeout-failure publication and missing
original-live provenance before the implementing agent's test-first guards.
The size contract explicitly distinguishes postcreation admission from an
in-flight storage bound. The reviewed final tests cover those boundaries.

Independent `python -B -m unittest discover -s .../c15-live-selection -p
test_live_selection.py -v` passes all 14 tests in 3.126 seconds. Private
`cargo fmt --check`, offline all-target/all-feature Clippy with `-D warnings`,
and offline all-target/all-feature tests pass. The Rust target has zero unit
tests; Python tests execute the actual Rust caller. Existing root source and
its previously passing gates are unchanged, not claimed as freshly rerun here.

Read-only endpoint verification passes all 37 source and 14 artifact pins,
including the executed binary. It joins `CALLER.json`, the selected receipt,
baseline census, declared overlay and immutable bundle. The original instruction
body plus two LF bytes and exact pinned installer policy equals the effective
overlay. All 47 accepted files match both owned accepted and private preview
images. The later live tree contains 48 files, has the recorded writer commit,
and its complete census canonical digest matches both saved materialization
endpoints. The 287,837-byte bundle names the earlier accepted commit, not the
later live HEAD. All 51 pins also match after review. This check performs no
fresh materialization, project check or provider call.

`lock_selection.rs` shares one actual `FileLockTable` with its `GitPatcher` and
releases the root read-lock closure before the writer completes. This qualifies
the cooperating fixture, not a running daemon or exclusion of other-key commands
and external writers. `materialize_selected` retains separate original-live and
owned-accepted provenance; overlay installation and same-path execution are
explicitly false. The old executor specification cannot silently be treated as
the original-live execution view.

No new quadratic helper path was found: census uses indexed maps and batched
objects, with bounded body/path work, JSON sorting and Git history traversal.
Reachable history is not bounded by selected-tree size. Git's 30-second timeout
is per child, not an operation deadline; 512 MiB is postcreation admission only.
These limitations remain prerequisites for any broader runtime/resource claim.

No normal runtime, public API, architectural ownership or agent-facing workflow
is modified. The private design/qualification documents describe this boundary;
no architecture or user-workflow documentation update or real-agent invocation
is required for this prerequisite. Runtime integration and the unanswered C15
experimental-scope choice remain outside this review's qualified result.
