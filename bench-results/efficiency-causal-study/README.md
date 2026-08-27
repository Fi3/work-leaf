# Efficiency causal study

This directory is the compact audit package for the Work Leaf efficiency study. It compares normal
sequential direct Codex with normal concurrent Work Leaf. The accepted comparison profile is
GPT-5.5 with `xhigh` reasoning. Rows from other profiles are display-only.

`FINAL-REPORT.md` gives the supported result and its limits. `evidence.json` contains the normalized
measurements used for every reported calculation. `provenance.json` binds those measurements to the
external recovery archive. The `frozen-original-task-scorer` directory contains the byte-identical
step-228 scorer, fixtures, result, tests, and compact test logs.

`NEXT-STEPS.md` defines the practical follow-up. Its primary analysis compares randomized condition
groups. Collection rounds control scheduling and expose failures early; they do not create
one-to-one statistical pairs.

The benchmark driver prompts and the scoring contract are separate. The committed driver prompts
are bound to commit `fc1a7e731d68a68b838c27f6e29f8709a6a2c0fa`, which was `origin/master`
when the package was frozen. The live remote ref can move and is not the benchmark candidate base.
The prompt lengths are 189, 609, and 149 bytes. The frozen scorer uses the original task prompts from
`e70c933ff0313fafb771ff214d06734845537b86`. Their lengths are 189, 204, and 149 bytes. `/fork` is
supplemental evidence for original-task quality. It is not a comparison requirement.

`verify.py` retains the original live-`origin/master` identity guard. After that remote ref moves,
the guard fails before the content audit. Do not update the frozen identity to silence that failure.

## Audit commands

Run the compact audit without writing Python bytecode:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 verify.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_verify.py
(
  cd frozen-original-task-scorer
  PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_quality_audit.py
)
```

When the recovery archive is mounted at its frozen path, also verify every external file hash:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 verify.py --check-archive
```

These commands read JSON, source text, logs, Git objects, and SHA-256 identities. They do not run a
benchmark, candidate, agent, model, or provider. Do not invoke the frozen scorer's full command-line
rescore during this audit. That path reconstructs saved candidate bundles and runs their local Cargo
fixtures. The committed `result.json` and its 64 hash-bound logs preserve that completed offline
rescore.

## External evidence

The external root is:

`/home/user/.codex/work-leaf-investigation-archive-20260824`

The main archive indexes are:

| Path below the external root | SHA-256 |
| --- | --- |
| `step1-recovery-archive/metadata/SHA256SUMS` | `ed3b2e7417c52d373c9c480823315da5a8665a1776b03921a670e10c0c9e572a` |
| `step1-recovery-archive/metadata/keep-remove-manifest.tsv` | `b62ce9af3298e69167666f3f8ed1f5dfb60cb74bb776f99599c83beaa0bd5bc8` |
| `step1-recovery-archive/metadata/candidate-assets.jsonl` | `45f87366696762d03bb36247fad40f06101601faaa83907ca89a0a73845c92f2` |
| `step3-final-replay-evidence/SHA256SUMS` | `15ab564767a6d824d4a44fbaa23b52ed50a5d8cad2e18be6b1213e98a87b6e86` |
| `step3-final-replay-evidence/20260824T230317.348966Z-literal-replay-2/replay-ledger.json` | `3826465c132f93fdd31ad99a0af7cc24dcf01f98ae6497460318378e4195626d` |

`provenance.json` gives the exact absolute-root-relative path and SHA-256 for every source consumed
by the compact evidence. Raw benchmark trees, candidate binaries, model streams, rejected material,
and old narrative records remain outside Git.
