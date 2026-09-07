# C15 diagnostic 002 preparation

This is provider-free preparation for the single corrected diagnostic in
[BUILD-AND-EXECUTION-SCOPE.md](BUILD-AND-EXECUTION-SCOPE.md).
Root owns the final source freeze, admission and launch. Diagnostic001 and its
failed startup/postcapture records are untouched.

The canonical fixture root is
`/tmp/c15-real-diagnostic-002.aD9RpS`.
Its `project` is clean at HEAD
`3181e41560e66d4a533e4eca3a871fd8df9790cf`, tree
`0d2d02ef2859890ccc2a7113fd27e3b457f90c6b`.
The tree exactly matches the original initial four files: the supplied incomplete
`capped` function, dependency-free Cargo manifest/lock and `/target/` gitignore.
No test body is supplied. The sibling `preview` and pre-created
`ordinary-bundles` directories are empty and disjoint.

[PRELAUNCH-PROJECT.json](PRELAUNCH-PROJECT.json), SHA
`db48bea0cc9c6a3cf8312e3ef6d9fc3e64a9f1472a904441b4f4e9df92448b02`,
records initial mutable project state. Its receipt may be pinned, but the project
file paths must not become postlaunch immutable-source endpoints: ordinary task
edits are intended.

## Effective settings and identities

The new `CONFIG.json` is at the fixture root and has identical bytes to the
original bridge config:
`d01e198e3e26294454a1da6d1bd797166281dea4809b9954b066e2dc7f8aae46`.
It retains empty overlays, `qualification_implementation: false`, the pinned
public Cargo capsule/toolchain and helpers, 30-second private command and
180-second operation bounds. Its nine helper/executable references and public
capsule hash were rechecked without executing them.

| New evidence | SHA-256 |
| --- | --- |
| experiment.json | 3516253f3758dd3658ca9fc900f876eb7fba2de2c5e2db561527986ccf574d5b |
| ENVIRONMENT-PROPOSED.json | c5e5f6ff1debe7944aa2f6f65533de262dfcce76fbadb43e08dad6e2414e2a02 |
| observation/observer-config.json | ba9e0f4483d1d588b0627351696392e3667a7676b24ba3f4a3fd9cb50486097e |
| observation/manifest.json | ae29045498b3543bf6c9142deac0d77dc252e45cae7209c3bd5a230511412a72 |
| EXECUTABLE-CHAIN.json | 4f0d1ec383b5741e1934d13b9bd84cef2b3f37860432705f70793225a0c08532 |
| PREPARATION-SCOPE.json | c6b4f333f3deefb69dd90377c97452a4680f01b313a27fa2a84c49e9d5a585d7 |

The identity is `private-test-first-diagnostic-002`; factor condition is
`private-test-first`, observer condition `work-leaf`.
The schema-v6 descriptor names the exact new project, preview and config paths;
Python and bridge identities remain unchanged.
The observer config names actual compiled source
`0756c6efc6e2d3ea44462327e0bccbd3b04014fa`.
The independent corrected binary is root's
`/tmp/c15-real-runtime-002.DlZ5tz/private-test-first-diagnostic`,
SHA `c8383d2bbfa3d98f01d81220984cd357d2228872737b80a3aa89469015a92c10`.

## Observer setup without a probe

The explicit new config/manifest follow the frozen
`CaptureConfig`/`write_manifest` fields in the retained observer source.
Their new identity, source/project commits, owned root, random marker and actual
host timestamps are declared preparation metadata. No `initialize` or
`--version` command was invoked; initialize would perform a prohibited probe.
The existing observed `codex-cli 0.153.4` value is carried from the pinned
original config, qualified by byte-identical executable-chain checks, not
presented as a fresh measurement.

All 16 executable/entrypoint records, symlink targets and six PATH resolutions
match the predecessor after only new owned proxy-path substitution.
The same observer ELF is copied into the new `proxy-bin/codex` and
`proxy-bin/sh`; both complete hashes equal
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.
The explicit proxy PATH prefix, exact suffix, subscription wrapper,
gpt-5.5/xhigh settings, API-variable removal, raw-usage capture, 1000-ms grace,
forwarded output and project-inventory option are retained.

The copied binaries' known original secret-marker flags remain part of the
original analyzer outcome. Only a separate exact-byte source explanation is
permitted; no original flag is cleared and no observer/proxy behavior is repaired
for this retry.

At the preparation check, admission, attempt, terminal, harness result, prompt
trace and provider/private-operation evidence are absent. No provider, version,
authentication, observer initialization, bridge validation/capture/test, private
executor or fixture Cargo test was invoked. Real native/private-test/ordinary
workflow and semantic qualification remain postcapture gates.
