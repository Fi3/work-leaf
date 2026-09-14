# P02 local qualification

2026-09-14. No provider call in this qualification.

Passed: test_author_inverse.py 14; test_resource_sample.py 3;
author-feedback/test_monitor.py 6; P02/test_catalog_dispatch.py 7.
Retained full discovery: standalone-serialized-host-20260912 88;
standalone-host-20260912 74; c15-real-diagnostic 6. Some inherited host
cases overlap; these are suite counts, not independent experiment counts.
The printed initialization/monitor failures are expected negative-path cases.

EXACT-SPANS.json is generated from the unchanged frozen adapter, not manually
edited output. Its B-only and A+B host digests both equal
cc1dd780a5f72f5f6cec32d2032e06b2f6c33c42fa0636c1886df9c098c3a25f.
Their prompts equal b8c613dc94bbca968261b1d1f5811d8da0fa08f32b160b0c77098c855eba7b5c
and b8d29422767ab87a338972b8da8296fc393d5bf16f5ecaedde1fae3509ffb15c.
Exact inverse tests preserve all non-target source bytes.

BUG003-RED.txt records six failing assertions before dispatch correction;
BUG003-GREEN.txt records all seven tests passing. The preserved P01 module is
not edited. P02/catalog_dispatch.py selects its startup namespace only for
fresh exec; option values, literal prompts and unknown/incomplete commands have
explicit tests. Actual P03/P04 native resume is the required real verification,
so BUG003 remains pending until that evidence exists.

Current source inspection confirms independent v7 runtime and external
observer wait configuration, but not a qualified restored historical episode.
The single-schema runtime condition and WL buildability gate prevent treating
the existing standalone A/B inverse as A+B+C08+C25. The manifest preserves
these joint blockers; working unit tests do not establish a joint token effect.

Cargo fmt, strict all-target/all-feature Clippy and all-target/all-feature
tests passed earlier this turn; no Rust source changes since those checks.
Normal WL runtime/public APIs are unaffected. The private dispatch affects
benchmark agent launch and retains its pending real-workflow verification.
