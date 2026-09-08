# Independent finite C15 prelaunch review

PASS for the prepared three-row phase. This receipt is outside the immutable
manifest and does not retroactively add admission pins. The review performs no
provider, private executor, Cargo, accounting, or benchmark launch work.

`PHASE-MANIFEST.json` SHA:
`00ff0668de698b75a5200d4145b9919a560d04ec082e505f4bac51e0488a417d`.
The exact three `private-test-first` rows occupy one wave, maximum concurrency 3,
with replacements disabled. The original fixed schedule is intact.

## Executed checks

The source-reviewed, unchanged runner verification command returned exit 0,
`provider_work_started: false` (tool `33ebc8`, 0.136089 seconds):

```sh
python3 -B bench-results/efficiency-mechanism-isolation-20260906T214448Z/runner_test_first.py verify --phase-root bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/test-first-01
```

Independent read-only operator `c5fb0f` (exit 0) rehashed all **187** immutable
input entries and checked canonical regular-file endpoints. All **47** current
build source files match the attestation and their manifest-owned frozen copies.
Build attestation SHA is
`a9c0179feffd0ed4a082c6c04436c11b7e0273cf21f8d2370f32847349462240`,
source commit `78e91b12abbe79bab4f2b1e3e0e53b8b74ddc615`.
No build or prior Rust gate is rerun.

All five retained source-driver scripts equal their frozen copies. The
subscription-only provider wrapper is
`0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`.
The manifest contains 79 external identities, 89 frozen evidence files, five
source and five frozen drivers, three experiment templates, two runtime entries,
and one each observer, provider wrapper, protocol and scorer configuration.

## Exact launch chain

| Artifact | SHA-256 |
| --- | --- |
| `launch-inputs/work-leaf-orchestrator-real` | `6c5ccec1e20f5004211b56edc4ade335271054468d4ee6adab45cc92cab27d63` |
| `infrastructure/bin/work-leaf` | `0e2330478d57bbbb1ab26ccf4152f37f2e6ae6a29b4e55faac9d12bde34cc3db` |
| `infrastructure/bin/bench-observer` | `238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386` |
| Generated orchestrator shim, both staged and infrastructure copies | `523c0a72c3e233fdf848d667d6ab1519f6e6c3de49feca0d9ba9f63ab83f0fd9` |
| `launch-inputs/bind_daemon.py` | `aa1800d574df9c7dee1ad899619594060a68cd28fe61d5a2f0ed9d63e16e3155` |
| `launch-inputs/BRIDGE-CONFIG.json` | `5e1e5a204445f7ccaec057f446d87ce060fe98783f560dfd577ea0f3f0fd982a` |

The three real binaries above are executable ELF files with link count 1. The
orchestrator shim is explicitly a Python `-I` bootstrap, not an ELF. The runner
verifier reconstructs its exact template index and bytes; all three templates
point to the separately retained real daemon outside driver cleanup.
Each template has the declared static `work-leaf-c15-launch-template-v1` schema
and the same 68 dependency identities. Dynamic v6 manifests and binding attempts
do not yet exist at the inspected cutoff.

Template hashes, workflow 001/002/003 respectively:

- `50a042aa3da937899f79c211bbcd674c66e7e2525e3b520d7533032558c1b2ea`
- `1f7549295c17d5bbd62f93401a6576d705868e3be7821d179ca2c33cc46c5067`
- `e3d34d8c0747118c6e7c4e95507b5e85479291ad336f022118f0d1105dda16c4`

## Configuration and unlaunched cutoff

Both copied tooltip qualification artifacts equal the final published bytes:
JSON `b789b3c3c2d359fa34a65765ec97b8638d5b9da7c70bb4c6e73abcf8be69d159`,
Markdown `acc57272959acf5c920fc65055f916402e6d18190c286ec6922f0cdb65a3f499`.
There is no publication-timing discrepancy (`5cb77c`, confirmed `c5fb0f`).
Current global raw SHA remains
`d36b9caec082759492578037173fe3bca099aad63f8e2553314601d2506e366d`
and equals the actual fresh phase baseline. No configuration contents are copied
or printed; the original configuration/trust drift monitor remains unchanged.

At `c5fb0f`, canonical `/tmp/c15-full-screen.UPWGw7` is empty; all three owned
private-preview directories are empty; each runtime, results, prompt-trace and
launch-output path is absent. `RUN-ONCE` is absent and `launches`,
`launch-validation`, and `prompt-events` are empty. These are sampled prelaunch
facts, not a claim that later admitted execution cannot populate them.

The first independent endpoint operator (`d4436d`) stopped on an erroneous
checker assumption that current build source paths must be direct manifest
members. They are correctly retained as frozen-evidence copies. Read-only
`34f7ec` established all 47 current hashes and all 47 copied memberships; the
corrected membership check passed in `c5fb0f`. No source, manifest, or artifact
was repaired, overwritten, or waived.

The bounded wrapper source review and automated qualification remain those in
`preflight/c15-full-screen-launch/INDEPENDENT-REVIEW.md`. Real dynamic binding in
this full benchmark is still an observation required from the first admitted
workflow, not a result of this prelaunch check. Existing documented complexity
qualifications and private-workflow/non-timing-only limits remain applicable.
