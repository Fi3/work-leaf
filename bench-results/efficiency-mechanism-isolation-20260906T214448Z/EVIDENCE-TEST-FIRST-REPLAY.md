# C15 fixed-artifact behavioral replay

Recorded at 2026-09-07 14:21:05 UTC. This is one provider-free historical regression replay,
not a benchmark workflow, a new control, a model observation or a token estimate.

## Result

**Qualified:** the captured Direct-002 terminal visual-character test reproduces its
behavioral RED on the exact reconstructed pre-implementation source, then GREEN after
the captured accepted implementation sequence. The command and the entire selected
`tests/terminal_app.rs` file are byte-identical across both executions. Its assertion
still requires the decoded clipboard text `aft`; it is not deleted, weakened or replaced.

The replay leaves the shared runtime/source tree unchanged. It does not establish how
a model would respond to privately supplied RED feedback, prove a natural WL test-first
variant, or assign any portion of the historical saving to test timing.

## Starting-tree qualification and retained alternatives

The [C15 design](DESIGN-TEST-FIRST-ISOLATION.md) first names step4-direct-003's slash chain
N114→N122→N128/N136→N146. Its native session and clean feature-1 checkpoint agree on
`b700def1cf3bc40e321327f4a9733ef2474d8710`. That provisional commit is absent from the
current repository, retained corrected-study source repository and the final bundle
imported into the new isolated object database. The original runtime checkout is absent.
The bundle advertises only final HEAD `4e388e1de35d40628a33c1f35716b19fef642cf6`
and requires base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`; a later tree is not
accepted as a substitute. No test from that case was executed.

The first inspected fallback, step4-direct-001 visual-mode N115/N121→N262, has the
recoverable common base, but N129 changes the second test's input sequence between
its two-test RED and GREEN commands. It remains a genuine historical behavioral RED,
but was not used as a strict unchanged-selected-test replay. No test from it was executed.

The next already-captured behavioral case, Direct-002 feature1, supplies the selected
unchanged terminal test. This choice follows tree/test identity qualification, not cost,
condition ranking or generated replacement data.

## Exact source and replay prefix

Native source N is
[rollout-2026-08-28T20-10-15-01a04990-b481-7702-b8b3-0ee09945eb44.jsonl](/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T20-10-15-01a04990-b481-7702-b8b3-0ee09945eb44.jsonl:1),
SHA-256 `d61f6577a67a6bf0f92912c06ec1e9c6efec563378cd76f2562d0d64a241020b`.
Only public call/input/output and session identity records are used.
N1 pins CLI `0.149.1`, author thread `01a04990-b481-7702-b8b3-0ee09945eb44`,
original cwd `/home/user/.codex/work-leaf-points89-runtime-20260828T145556Z/direct-002/work-leaf-3feature-sequential-bench.9OcvT9/repo`,
and starting commit `c92a0b7060a36eac6db2d869b85e589a7a9480f9`.

The retained [base checkpoint manifest](../efficiency-points8-9-20260828T145556Z/runs/direct-002/points89-direct-002-three-feature-sequential-bench-artifacts/observation/git-checkpoints/manifest.jsonl)
has SHA-256 `5b2ebcc7d228dac6ab36e6938d54ec8a54c00185b28a61d728cd1b8266ef4c15`.
Its six base files match their recorded hashes. `head.txt` names that same commit
(SHA `8351597f8d4963c753559d811fcec3e9aba9f7c65fb635996a722cabf1a2a2b8`);
`status.txt`, `index.diff` and `worktree.diff` are all empty
(SHA `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).
N77→N85 independently returns a clean `git status --short`.
All other pre-edit commands inspected are reads; no production edit precedes the new tests.
The tracked source prefix is the actual base plus the accepted test patches below,
not a later final bundle or an inferred feature-equivalent tree.

The private checkout is `/tmp/work-leaf-c15-replay.TWeEqK/repo`, created with
`mktemp -d /tmp/work-leaf-c15-replay.XXXXXX`, `git init`, a local fetch of the exact
base from `/home/user/src/work-leaf`, and `git checkout --detach <base>`.
The checked-out base tree is `acb6583fbb0e25347f70ee44cbdacafd6df95e37`;
SHA-256 of raw Git commit/tree object bodies is respectively
`4d5329d4dff53a4ba18a6f66d2ca420f80c9e0151690b4af32f162074f972f01` /
`99bfd2abde3d7a938fe7fae01ab0656ec44c11737be5e2467e637d2106db4e79`.
The new checkout was clean before replay edits. It has its own Git administration;
the shared repository's Git index was not used to stage replay changes.

Accepted N120/N174/N180 are applied before RED: two new harness tests, the new terminal
test, and its clipboard decoding helper. N126/N156 are rejected attempts, not applied
writes. After RED, all accepted N208–N328 edits are applied in native order; rejected
N264 is not an applied delta. Every patch is passed to `apply_patch`; the only payload
transformation prefixes each relative `*** Update File:` target with the isolated cwd.
No patch body is rewritten and every applied call has its captured successful output.

| Native input → output | Exact call ID | Original patch-text SHA-256 |
| --- | --- | --- |
| 120 → 122 | `call_IxXwNff5XaduHlsuIImGECDZ` | `f5b3e3a05e0168d19a4ecd1ff8c831168dda25f81d1623c5ed37b5bb63988e03` |
| 174 → 176 | `call_TRdwXZrTwn2r1kGj2XG5xSaa` | `68caf9b8a533cb353b17fa9432e9d6f3208506a88844474a93eb4860d86f8a90` |
| 180 → 182 | `call_0T1uqK11F299qWxtQ1vksKRu` | `9e98798ec99a5a8860446758018597263bbd977b3f5c94c13e22ec31b2eb307d` |
| 208 → 210 | `call_0Yh1tE3USMJj4Nx7i6gvff3i` | `0c6090daabb9049eccd1fb812091351600861baae92944ac41ec0e1339d881b2` |
| 216 → 218 | `call_glHz5Ug3ItVPqOXXv1QLRkvW` | `0716a0d367d54cfba401cc16a287ce282e57b52fea43c3baa97617453e99b0c9` |
| 224 → 226 | `call_gmO68dZcVFZtVW0Qv0XqNo6H` | `b189de147a20b07f7f8d60a2c95651157612b97b5512d4d8f92e29977a5e424f` |
| 232 → 234 | `call_Lw7HiqdFLdWSgsNN3GLxcrOb` | `f1e545b5445468849e94ac140eebeb2d822445c847b71d207db7cff98fc1b3c6` |
| 240 → 242 | `call_DTBU1nWx7oyiIK5DBYH5VYl4` | `26593453f4b28fa88c7365bf1ba168e0985f57f331c1b2ba8584838d02b08fb8` |
| 248 → 250 | `call_V1oGSCGYOdVCXXupwxKhEGwP` | `6fe00311bebd369a5dd83425478b661da4e01e67f68c1ba1f66ff928d7565ec6` |
| 256 → 258 | `call_06ERR3vDQNbWW5DSUTa24WnJ` | `b3c79dfbb436e98638d8475ed25ed69d59fd56a4ece8443199a4bdf8e214387c` |
| 290 → 292 | `call_FmgSD8Z5xfe0vAWSExE9watO` | `dc21e6163571d3e2c8b1b3c92e98cf7caeb9b7920f31f396455412270cde3167` |
| 296 → 298 | `call_F6YDNRrefMTuFSXFuhiIm55C` | `25714b871094e36e9091b3a849d0fd1436e8c00423a77f446b21ff320cd6288b` |
| 304 → 306 | `call_XZ35XfHcm5vkii2jtMWqC0GX` | `23fd92db07520d2bea2d167c47f4dbb0ea85eda71b90714bffa0fa030a08fe7b` |
| 312 → 314 | `call_2gMeLz5l42h54JKEuSsXQSHw` | `f3076a6081953b5d2f58d4adf1fbbf21fbc9702637b0011e38203f1c6ff00fd9` |
| 328 → 330 | `call_bfbxwCdz51cz6hkPTPhC3eOl` | `dd3efcdd0e3c38fa864335d69ba0c3f2cff7039f7d42d57a2e4153ac1e08c8a5` |

N328 changes only the unrelated `tests/ui_harness.rs` block-test expectation
from the two-line `> wo` / ` par` string to `> wo` / ` use`. It is retained in the reconstructed GREEN prefix,
not concealed as production code or used to repair the selected terminal test.
The intervening historical harness commands N320 and N334 are not rerun; this bounded
replay executes only the selected terminal command twice.

## Exact command and outcomes

Historical N196 and N342 contain the same command:

```sh
cargo test --test terminal_app visual_character_yank -- --nocapture
```

N196 call `call_bM2oKGBAL9KDxOO1tgeizeTf` returns N198 item
`fco_01a04995-a1ad-7410-b36e-46599668bea1`: exit101, one executed test,
missing OSC52 clipboard output at `tests/terminal_app.rs:1059:10`.
N342 call `call_PAQu67JoANaZ5RIEkolbVf5b` returns N344 item
`fco_01a0499a-4698-7250-982c-5ce09b537b1d`: exit0, that same one test passes.
N198/N344 public output-string SHA-256:
`78ddea0d46adf7b4b256e847007ea4f8c35f47ecc481f06d5580be15c6e37f0d` /
`f22ab3f789e1d0c2dbfdbf6676f1103fb160921359cad3d68a7b6d9d90b67081`.

Both actual replay invocations use that exact Cargo command, with the same declared
execution envelope and cwd `/tmp/work-leaf-c15-replay.TWeEqK/repo`:

```sh
env CARGO_NET_OFFLINE=true \
  CARGO_TARGET_DIR=/tmp/work-leaf-c15-replay.TWeEqK/target \
  timeout --signal=TERM --kill-after=1s 290s \
  cargo test --test terminal_app visual_character_yank -- --nocapture
```

No network package fetch or provider command is needed. The selected test instantiates
the existing `FakeBackend::from_replies(VecDeque::new())` and exercises terminal input/
rendering only; it does not run the separately defined Codex-backed tests.
Compiler/toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`,
`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`. This is a fixed-source dependency replay on the
present compiler, not a claim to reproduce the complete historical execution environment.
The two independent 290-second timeouts with one-second kill grace cap their combined
process-execution budget below 600 seconds. Neither fires; Cargo reports 5.96s and 1.97s
to finish the two test builds, and 0.00s test execution for each. These rounded build/test
figures are not asserted to be exact aggregate wall time.

Actual RED receipt, exit101 (compiler progress omitted, outcome unchanged):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.96s
     Running tests/terminal_app.rs (/tmp/work-leaf-c15-replay.TWeEqK/target/debug/deps/terminal_app-4ec6028fec5a70db)

running 1 test

thread 'terminal_app_visual_character_yank_emits_clipboard_sequence' (1027) panicked at tests/terminal_app.rs:1059:10:
frame should start with an OSC 52 clipboard update
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test terminal_app_visual_character_yank_emits_clipboard_sequence ... FAILED

failures:

failures:
    terminal_app_visual_character_yank_emits_clipboard_sequence

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test terminal_app`

```

Actual GREEN receipt, exit0:

```text
   Compiling work-leaf v0.1.0 (/tmp/work-leaf-c15-replay.TWeEqK/repo)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.97s
     Running tests/terminal_app.rs (/tmp/work-leaf-c15-replay.TWeEqK/target/debug/deps/terminal_app-4ec6028fec5a70db)

running 1 test
test terminal_app_visual_character_yank_emits_clipboard_sequence ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s


```

This is a behavioral assertion failure, not a missing API, syntax failure, zero-test
filter or failing pre-existing assertion. The implementation enables visual selection,
clipboard serialization and the existing terminal adapter's text-context route.

## Byte preservation and limits

The entire selected `tests/terminal_app.rs` is 44,052 bytes before and after implementation,
SHA-256 `3a625e94afb8e69e15ee386e4807fc719c6dfb51775b4a3a1e371cde9ff05787`.
The selected `#[test]` block before the next test marker is 451 bytes,
SHA-256 `f38f85cb3aa18bfba00b99d277d60a0ebeaca76960d56102237c4d7e50643c21`.
`Cargo.toml` / `Cargo.lock` remain unchanged:
`01fdea4445393186362a5c8b67f46b3fd472f49f6748daaf5df745702d2266dd` /
`ffd7c0afbc27f9599345589178c6385974179458ac908161cff60f8611946758`.

Final private implementation file hashes are:
`src/ui.rs` `84d4cf8b9e4fda3bf1e54dc941b0925d3b5b74bcfd0bc23fc1433d14ed1e4c0d`;
`src/terminal_app.rs` `a7d9daa0e5b1e49629662ce74e509cd7ed203ebe10f7f263a589a09180035b8a`;
`src/ui_harness.rs` `6a32186375a098ce73a9351264e6356e32d39ca8aaf18163c86d27e0685ccd0b`.
The unrelated private harness test file changes from
`bf77bfcf825e53e03b884b60b669a04141c51d1bc254804e908228481a3c19f2` to
`164d0a1eca8c0dcbeff242b00be025f776fc97d6613c0a2cfaf26f6128584e3a`
solely as recorded by N328.

Before/after SHA inventories cover all 75 then-tracked files selected by
`git ls-files -z -- src tests Cargo.toml Cargo.lock web-ui docs/architecture.md AGENTS.md`.
The path sets and every file hash are equal. SHA-256 of the compact, sorted-key
JSON `{relative_path:sha256}` inventory is
`0c02791f90749b216f57e3318b098c2ab530dda4ea813bd46b3089315a157eea` on both sides.
Parent/sibling writers explicitly remained outside those runtime/source files during
the replay. Their concurrent study notes and new offline helper files are not falsely
covered by an “entire repository immutable” claim. This task writes no shared source,
existing tests, config or old evidence; only this new report is published in the study.
Native N was rehashed at endpoint with no mismatch. The disposable checkout/build output
is retained at its exact private path; no cleanup/deletion occurred.

The unrecoverable slash bundle has SHA-256
`7cddb8b1197f9dc93b9565d447d2c082f421884aa4c3608cc202679e02cb6a84`,
and its checkpoint manifest has
`b8b5b6b79a76761498225d73d5c236c1ee45aa838b2f32427b3879a90e63f9fa`.
Planning/census pins remain `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493`
and `894130bacdce03adb1b86173a05e03162a57f1b3c91ebbefc54205a692659901`.

**Qualified contribution:** an actual held test can execute its missing-behavior RED
privately, retain exactly the same test, and pass with the captured implementation
without publishing a known-red shared source tree. This demonstrates the fixed-artifact
dependency and feasibility of isolated execution, not that a natural private-WL preview
is already implemented or causally saves tokens. Existing historical outcomes, rejected
attempts and unrelated repair remain intact; no percentages, costs or quality gate
were recomputed. No current agent-facing workflow changes, so this evidence-only replay
requires no new real-agent verification.
