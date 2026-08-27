# Evidence Storage

All exact raw provider streams, process records, reports, patches, scorer logs, and analysis outputs
are stored normally in this study.

The Work Leaf observer also generated a derived app-server frame index at:

`runs/work-leaf/pilot-pair-001-work-leaf-three-feature-bench-artifacts/observation/app-server/00002493652441802062-1870/frames.jsonl`

That index is 108,276,974 bytes, which exceeds the usual 100 MiB single-file limit of Git hosting
services. Its uncompressed local copy is ignored, and the repository stores the exact gzip archive
`frames.jsonl.gz` beside it.

- uncompressed SHA-256: `afb5584be02628e48ad8422adbe5892095db91ce37d7614a319af620b4025d99`
- compressed size: 13,940,637 bytes
- compressed SHA-256: `66e14b32631721a6c3eb12f03c55356f6c0fbc3e0578a256b62fe6373314e910`

For a fresh checkout, `gzip -dc frames.jsonl.gz > frames.jsonl` restores the byte-identical derived
index before running tools that expect the uncompressed path. The index is derived from the retained
raw app-server capture; no provider event or admitted attempt is omitted.
