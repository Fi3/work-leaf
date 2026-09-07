# Historical command-output mechanism exposure

## Finding and scope

**WL's command-output compaction removed no captured output in this historical pair.**
All 14 delivered `work-leaf command result` prompts exactly match the corresponding
captured stdout/stderr after only the normal empty-output/newline rendering.
There are no blank-run, long-line, or total-output compaction markers at these
boundaries. Disabling those three compactors would therefore preserve these
particular delivered output strings; this is a representation identity, not an
estimate of a future behavioral effect.

The larger native command outputs in this pair occur outside that WL renderer.
Both Direct and WL retain native tool results with the same truncation-wrapper
form. Such results can be repeatedly charged as input, but the historical Direct
capture lacks per-item attribution, so there is no exact between-arm item-price
decomposition here. No primary result, frozen report, timing, runtime, or provider
was changed. Only public command/result text and safe item/usage metadata were
inspected; no private reasoning bodies were read.

## Source chain and exact rendering boundary

Source citations refer to the historical source checkout
[`efficiency-raw-token-pilot-20260906T185328Z/source`](../../efficiency-raw-token-pilot-20260906T185328Z/source)
at HEAD `41b6418ef420fbce6aab93706657bd77dba3ce51`. Its orchestrator bytes
also match the later driver reference at `3c907f7266b63efa1558213f3e49587e999b899d`.

- `src/orchestrator.rs:1090::run_command_for_agent` validates command/ownership,
  takes write locks, and calls `:1348::run_shell_command`. The latter captures
  stdout and stderr separately with piped streams and UTF-8-lossy conversion.
- `:1195–1212` selects the ordinary result or pending-tracked-diff renderer.
  `:2384::render_command_result` preserves command, status, locks, failure guard,
  and next-action guidance; `:2408–2411` applies `render_command_output`
  independently to stdout and stderr. Pending tracked diffs are appended separately.
- `:2478::render_command_output` emits `<empty>\n` for empty output, otherwise
  runs blank-run → long-line → total-character compaction and ensures a final newline.
  Constants at `:2041–2046` retain the first eight whitespace-only lines of a run;
  lines over 4,096 Unicode scalar values retain 1,600 at each edge; streams over
  12,000 retain 6,000 leading and 4,000 trailing characters plus a counted marker.
  These are character limits, not token budgets or limits on the whole prompt.
- `:1212` → `:657::send_agent_streaming_interruptible` →
  `src/codex.rs:1676::send_streaming_interruptible` sends the result to the saved
  agent thread. A known session receives the raw follow-up, without another launch
  policy. `src/codex.rs:195::request_turn_streaming` sends that input through
  `turn/start`.
- Direct's `bench-three-features-direct-common:801::run_direct_agent` and
  `:852::run_direct_agent_resume` invoke the ordinary CLI task/resume interfaces.
  They do not pass native tool results through the WL orchestrator renderer.
  WL native `exec_command` reads and its direct-tool linearizer also bypass it.

The repository paths above, `bench-agent-profile-common`, and the archived
`bench-observer/src` contain no implementation of the observed native
`Warning: truncated output` / `tokens truncated` text. The native examples
below establish actual delivered representations, not a general claim about
uninspected native truncator code or a guaranteed global token limit.

## Complete mediated-output exposure census

Let `W` be the historical WL artifact's `observation/` directory:
`../../efficiency-raw-token-pilot-20260906T185328Z/runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts/observation/`.
Let `A = W/app-server/00000422153032163608-2899396/`.
C/S denote physical lines in `A/client-to-server.raw` and
`A/server-to-client.raw`; forwarded client line numbers are identical here.

Each listed string-typed request has an accepted response, a byte-identical
forwarded request, and one exact matching native user-message text. Every output
pair has one matching saved shell capture by exact wrapper command, status, and
rendered stdout/stderr. This last association is a unique content/command witness,
not an invented typed command-execution ID in the provider request.

| C line / string RPC ID | Status | Delivered stdout / stderr chars | Matching `W/locked-commands/` capture |
| --- | ---: | ---: | --- |
| 26 / `25` | 0 | 2583 / 2697 | `00000422562015149591-2910546` |
| 46 / `45` | 101 | 2370 / 1575 | `00000422814783925331-2919059` |
| 48 / `47` | 101 | 2824 / 1701 | `00000422826506692103-2921919` |
| 52 / `51` | 0 | 307 / 279 | `00000422836020600660-2922245` |
| 64 / `63` | 101 | 2817 / 1822 | `00000422883036009175-2923908` |
| 68 / `67` | 0 | 2588 / 1945 | `00000422924243746227-2925236` |
| 88 / `87` | 1 | 8 / 183 | `00000423522393747044-2946456` |
| 90 / `89` | 0 | 307 / 279 | `00000423533676436713-2946688` |
| 103 / `102` | 0 | 307 / 279 | `00000423762490604813-2954235` |
| 110 / `109` | 101 | 8 / 1357 | `00000423804590245813-2959236` |
| 116 / `115` | 101 | 3066 / 1822 | `00000423815769252426-2959548` |
| 122 / `121` | 0 | 2746 / 1945 | `00000423870866212638-2962822` |
| 135 / `134` | 101 | 3108 / 2158 | `00000424109482269545-2969257` |
| 139 / `138` | 0 | 895 / 439 | `00000424131492123670-2973984` |

The largest delivered stdout is 3,108 characters; largest stderr is 2,697.
Both are below even the long-line threshold. All 28 stream comparisons pass.
The two 8-character stdout representations are `<empty>\n` for zero-byte captures.
Each capture's raw stream SHA matches its saved `meta.json.end` digest.
There are 35 shell-capture directories overall; **14 is the mediated prompt count,
not a claim of only 14 shell/native commands in the workflow**.

For a fully identified small example, C52 / string RPC `51` is the passing
`cargo test terminal_app_review_done_prompt_closes_and_reopens_patch_chat --test terminal_app --all-features`
result. S23862 accepts turn `01a0782f-7daf-7e40-bd96-98dc4a880e58` in thread
`01a07825-376e-7661-a88d-4be15e65dd86`. Its exact 964-character prompt is native
rollout line 225, item `msg_01a0782f-7ddc-7c30-9f9d-12d0c118cf7c`
(text SHA `e8ee9d1165e22190869d033eb52e96c971fa758be1bb61f7ccd8c34ebf880ea9`).
No output was removed; command/status/locks/guidance are part of the message price.

## Native output examples and actual charges

Native rollout locators below are physical JSONL lines under
`/home/user/.codex/sessions/2026/09/06/`. Selection is by concrete output
representation and source role, not a claim these are the largest causal costs.

1. **Direct broad search already has native truncation.** In
   `rollout-2026-09-06T22-00-16-01a0784e-abd2-7803-a0b5-e83fd25653a8.jsonl`,
   line 12 calls `exec_command`, `call_VlQfgem6QKDRny35Ax1ZwrbN`,
   with `rg -n "review|patch|done|highlight|chat|agent" src tests examples -S`
   and requested `max_output_tokens: 30000`. Line 19 is
   `fco_01a0784e-d040-7c91-82fe-ea400e24990d`: 40,212 characters / 40,216 bytes.
   Its public wrapper reports original count 66,368, 3,331 output lines, and a
   middle marker `56368 tokens truncated`; output text SHA
   `3930bc1c3876202a47a800f822c7928b2712cd1bb53209e1d7869af94c1d38e3`.
   The wrapper's original count is **not** an exact provider input-item charge.

2. **WL has the same native large-read surface.** In
   `rollout-2026-09-06T21-14-50-01a07825-10a9-77b3-bd6a-f75800663294.jsonl`,
   line 48 calls `exec_command`, `call_43LcgeqOmTX7WDQjUa4C5LXG`, for
   `sed -n '500,1815p' .../context-bundles/orchestrator-2899296-0/bundle-0.md`
   with requested `max_output_tokens: 100000`.
   Line 51, `fco_01a07825-b459-7f21-b1ba-b7bb909a2abd`, retains 40,210
   characters / 40,214 bytes, reports original count 10,809 and `809 tokens truncated`.
   Text SHA `893e006858b02d99b64877e1c75a563a0042f0b571293047362c7edf2aecbc9d`.
   This is a native read of a bundle, **not** `render_command_output`.
   Exact input attribution charges this item 9,054 tokens in each of 40 observed
   responses. The first is S476,
   `resp_0c79ac7c5de691aa016a9dbbd6849087d2988f54a2b6ebec41`;
   the next is S528,
   `resp_0c79ac7c5de691aa016a9dbbddfa8487d2b04af97b4a6cba50`.
   Both belong to turn `01a07825-37bf-7793-b533-674e10dc8d7a`.
   The item is read once at this call; 40 charges do not mean 40 executions.

3. **WL linearizer diff output also bypasses compaction.** In
   `rollout-2026-09-06T21-50-56-01a07846-1e5c-7352-8b45-819244e3c659.jsonl`,
   line 43 calls `exec_command`, `call_r6duiBGXVGF2u8nzdeYRHcV0`,
   for a seven-file `git diff --src-prefix=a/ --dst-prefix=b/` over
   `7b2ac17e7fb70e17de6ede5334a92523a78bc092..cfacd23b9c698b93cfaf4017455350fc732f60ee`.
   It requests `max_output_tokens: 70000`; line 51 item
   `fco_01a07846-f82d-7e30-8ec0-b22de6ef7859` retains 40,211 characters /
   40,215 bytes with original count 19,280 and `9280 tokens truncated`.
   Exact native payload SHA
   `a609c7ab50e87126d80f4fb4e0d9b73ed010641bd99404b123ed8d6cfa139aba`
   in the retained attribution.
   It is charged 9,992 input tokens in each of 45 observed responses, first S43999
   `resp_044a07c1a37e3e02016a9dc45adae487d28163c2f987f24265`, then S44069
   `resp_044a07c1a37e3e02016a9dc46a31ac87d29df6d5af292b219e`.
   These two are in turn `01a07846-1eb6-7022-9b8b-35ea4d92462c`;
   the subsequent resumed turn retains the same native item identity.

4. **Ordinary test output can also be longer outside the WL renderer.** Direct
   reviewer rollout `rollout-2026-09-06T21-31-02-01a07833-e768-7030-a8ae-2b7c5237fa32.jsonl`
   line 330 runs
   `CARGO_TARGET_DIR=/tmp/work-leaf-review-target cargo test --all-targets --all-features`;
   line 333 result `fco_01a0783d-ea79-7061-94e0-5c6cca73033e`,
   call `call_DXZJbu0F2NiiRi4S60uvrUBm`, retains 19,091 characters without
   a truncation marker (text SHA
   `9555df758d5a942b1e964aa7a9ff4f835d79412b1e3d6b0328c98120f8027cce`).
   WL linearizer rollout above, line 489, likewise runs
   `cargo test --all-targets --all-features`; line 492 result
   `fco_01a0784e-fe7f-7c50-84ee-b806017b4943`,
   call `call_DhbHSqCRBKF7QRlp7ZhOf6pd`, retains 19,637 characters without
   a marker (text SHA
   `3a69d95ff1ac5c1bdd6de65d52a6a2af345cb3fe0664f2db0a6609ddfca5df1b`).
   That WL item is charged exactly 4,740 input tokens in S46389
   `resp_044a07c1a37e3e02016a9dc666ba0487d2b7a3af74a770265d` and S46612
   `resp_044a07c1a37e3e02016a9dc674457887d29502d55f9969bb33`,
   both in turn `01a07847-c176-7740-90f8-f9e870975165`.
   Different feature states, review/final roles, and command scopes prevent
   interpreting this pair as a matched output-ablation comparison.

The small mediated C52 result above costs 292 input tokens in each of 38 observed
responses, first S23901
`resp_03fee1970559bc89016a9dbe57b67487d2a5dbd57caf060816`.
The earlier failing mediated C46 result (native feature-3 line 201,
`msg_01a0782f-39cc-7063-9d4c-482774747506`) costs 1,455 input tokens in each
of 40 observed responses, first S23813
`resp_03fee1970559bc89016a9dbe4476a487d285a302a5d2a77df1`.
These are complete message-item prices, not separable stdout or guidance prices.
All selected WL response attributions reconcile; their tools/instructions request
fields total 10,274 input tokens per response and are **separate from** these item
charges, already included in total input. No cache subtraction defines these claims.

## Mechanism limits and useful boundary

The active representation opportunity is **which actual source/diff/tool results
enter the conversation, and how they persist across later responses**, not removal
by the unexercised WL command compactors. Bundle/native-read representation is a
distinct boundary under separate source investigation; reviewer source assembly and
validation scope are also distinct from command-output truncation.

An output-only intervention at this WL boundary must preserve the actual command,
execution, status, locks, pending diffs, ownership, tool access, waits, required
validation, and other prompt text. Artificially verbose commands, looser command
scope, different check counts, or applying WL limits to native tools would change
other factors. No repeat of the tested acknowledgement/cardinality or command
next-action cue follows from this census. A zero-exposure compactor toggle supplies
no useful historical explanation.

Repeated charges establish retained-input processing, not repeat execution,
unnecessary work, equal intermediate quality, a causal share of the accepted
historical saving, or the token price of a hypothetical replacement. Native
truncation is not session `compacted` context summarization. The latter and missing
tails retain their separate coverage limitations in the historical audits.
Direct's attribution status remains `unknown` with
`no completed raw response attribution; coverage unknown`; native output lengths
and wrapper counts do not fill that gap.

## Provenance

All 15 historical native rollout digests matched the retained metadata; the selected
raw provider streams and every matched command stream were rehashed. The exact
per-response/item inventory is [HISTORICAL-INPUT-ATTRIBUTION.json](HISTORICAL-INPUT-ATTRIBUTION.json);
full native source paths/digests are retained there and in
[HISTORICAL-EPISODES.json](HISTORICAL-EPISODES.json).

| Source | SHA-256 |
| --- | --- |
| `HISTORICAL-INPUT-ATTRIBUTION.json` | `586533b10c4ffa94b9b2dbaf5ebf91762f8ea103b500238ae6b6e1265f0100c6` |
| `HISTORICAL-EPISODES.json` | `b833b6666e70ae8bb445c7b84e5c64d7a33e4a8fe258f0259304339c5ad9302d` |
| Historical `source/src/orchestrator.rs` | `f88aeea1e910504eeb4434291e662ff2455e64edb5b2d45f87f5c316c6aafe24` |
| Historical `source/src/codex.rs` | `ab4e418ca769505f12d80d1b457afe99bed3d4e8d98b3378fbb536eef2b71a2d` |
| Historical `source/bench-three-features-direct-common` | `489289165601e00a545f76ab0631d5e0f48d644b928376488677e1875651e386` |
| `A/client-to-server.raw` | `3be1a143ed5f66960daa92816800a8ac1ec63ae9af9ed43f1eca93ba90ad7ece` |
| `A/client-to-server.forwarded.raw` | `e53439e87beae540c08093a70a24d7f01743d458a356e6300303d916f8e5f7c7` |
| `A/server-to-client.raw` | `701a611f120e7718f1ebabfac8d4d676a0ab76da237c8c3201d0b2b76a65eb9f` |

The following `W/locked-commands/<capture>/meta.json` digests pin the original
command/status and stdout/stderr hashes used in the complete census.

| Capture | Metadata SHA-256 |
| --- | --- |
| `00000422562015149591-2910546` | `f3b88281b9c938c88a497b4307dec324208255102d6effca81d25ca2a19c9bcd` |
| `00000422814783925331-2919059` | `229894132e4e5fba1c53e147839df3b8b4e9bf321a40997cab36c805aa5bd4a3` |
| `00000422826506692103-2921919` | `740a513d058f08200e530f28b2034da083a8ad2fa8bbde86846098200a36023f` |
| `00000422836020600660-2922245` | `bb0fea8d63aca92694351cf7ff87bfda77c1fc44cb8d36d7936689473fd668c9` |
| `00000422883036009175-2923908` | `797823a510bef7951cc3cb04a6a56cfbf2053504d24db5ef6931b1a910e90be8` |
| `00000422924243746227-2925236` | `399ae00f5c88a83f6589f821e1350d3fb47942a3cdd01c51eb590f239b889a3e` |
| `00000423522393747044-2946456` | `1012d3fc930d9ac60dc70a72171db3f2935e78efdbb26c2d1a9d63452da4467f` |
| `00000423533676436713-2946688` | `785d4511907363219fae7a7e50d289539e7268ecd0ce5e03fd2e48356a9c7ac1` |
| `00000423762490604813-2954235` | `89000e540f0194d3bf0fec378f98c77e5356e2c027116a47fd1c4570b0d8571d` |
| `00000423804590245813-2959236` | `095d3445f31b41f11276875fef399fec8e54af25445bb4dc6eaffa6dc532a0d4` |
| `00000423815769252426-2959548` | `90e3fe305d727548a08c55e7c088bbca26538a66eff0f906e30b91496c14a3a7` |
| `00000423870866212638-2962822` | `7d2bdedf7c89a7c960f9e078f75ad432c52f99c4ce62a0ad4e3f0d932959125c` |
| `00000424109482269545-2969257` | `b435f02af708d5e9322bb7c48cef97eea19065b0ba1a2c03e9760eb4c3571f11` |
| `00000424131492123670-2973984` | `7d9deca742628e7ce2d61d8747a55212bbbedb7f349c1f8dff5169de5621494b` |

This is a provider-free descriptive evidence note. No agent-facing behavior changes,
real-agent verification, outcome recalculation, or new benchmark admission occur.

