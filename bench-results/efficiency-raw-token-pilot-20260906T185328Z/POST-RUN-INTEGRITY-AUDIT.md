# Post-run integrity audit

The frozen executed artifacts retain their admitted hashes, but full post-run
verification **fails** because the external global Codex configuration changed.
This is a retained protocol deviation, not a green environment-integrity result.
The captured model, effort, provider identifier and launch permissions agree with
the declared profiles; that narrower finding does not establish that the unknown
configuration change was harmless.

## Exact mismatch and scope

The following read-only command returns exit 2 with
`ValueError: frozen external-identity-only digest changed`:

```sh
python3 -B bench-results/efficiency-measurement-gate-20260906/first_batch.py verify \
  --batch-root bench-results/efficiency-raw-token-pilot-20260906T185328Z
```

Checking every `FIRST-BATCH-MANIFEST.json` file entry with `sha256sum --check`
isolates exactly one mismatch among 43 entries:

- Path: `/home/user/.codex/config.toml`, role `external-identity-only`.
- Admitted SHA-256: `08c66bacf2e55a906e357117a382c8669af43cb73291a350c292ebe5b7988ac3`.
- Post-run SHA-256: `c84b27799cca6b4776e14096acc15d5bdee6a09053bbb09e10c0ffefda3d35a9`.
- Filesystem metadata: owner/group `user:user`, mode `600`, size 30,852 bytes,
  inode 23,158,897; birth, modification and change timestamps all
  `2026-09-06T19:50:56.328348011Z`.

The other 42 entries match: frozen runtime binaries, observer, provider wrapper,
drivers, analysis/scorer evidence, and the external Codex launcher and native
binary. The clean source clone still has no tracked or untracked changes. The
manifest and schedule hashes still match `RUN-ONCE/admission.json`.

The global configuration was hash-pinned, not copied. This audit did not print or
copy its contents, read credentials, modify configuration, or attempt a replacement
provider run. No before/after setting-level diff is available from that identity
pin. The timestamp coincides with Work Leaf's linearization launch:
`logs/work-leaf-001.log` lines 198–200 record force-linearize at 19:50:51 UTC,
Launching at 19:50:56, and WaitingForReply at 19:51:01. This is temporal
corroboration only; neither timestamps nor file ownership identify the writer,
the changed keys, or the reason for the write.

## Actual captured profiles

Both published `observation/rollout-metadata.jsonl` inventories were replayed
against their exact listed files under `/home/user/.codex/sessions/`. All 15
source-file hashes match. Projections of only `session_meta` and `turn_context`
fields establish:

| Captured field | Direct | Work Leaf |
| --- | --- | --- |
| Session records | 7 | 8 |
| Provider identifier / CLI version | `openai` / `0.153.4`, all 7 | `openai` / `0.153.4`, all 8 |
| Turn-context records | 17 | 76 |
| Model / effort / approval | `gpt-5.5` / `xhigh` / `never`, all 17 | `gpt-5.5` / `xhigh` / `never`, all 76 |
| Turn-context sandbox modes | 8 workspace-write, 7 read-only, 2 danger-full-access | 74 read-only, 2 danger-full-access |

Turn-context records are configuration observations, not an exact count of
provider responses or independent trials.

Direct's process inventory contains 16 captured CLI launch/resume invocations,
all with exit 0. Every invocation supplies `--model gpt-5.5`,
`--ask-for-approval never`, and JSON output; all seven implement/fix invocations
request workspace-write, all seven review invocations request read-only, and
both linearization invocations request danger-full-access. Ten of these 16
processes start after the recorded configuration modification time. Their
captured flags and resulting rollout profiles retain those settings.

Work Leaf has one captured app-server process, started at 19:14:49 UTC. Its raw
client stream contains eight `thread/start` and 76 `turn/start` requests. Every
request retains `gpt-5.5` and approval `never`; thread starts request seven
read-only and one full-access sandbox, and turn starts request 74 read-only and
two full-access sandboxes. All eight thread-start responses report provider
`openai`, model `gpt-5.5`, effort `xhigh` and the corresponding permissions. An
app-server started before the modification is not proof that later configuration
loads cannot observe changed settings.

The subscription route is supported by the unchanged frozen wrapper
`infrastructure/provider/codex`: it removes the listed API-key/endpoint override
environment variables and passes `forced_login_method="chatgpt"`,
`model_provider="openai"`, `model="gpt-5.5"`, and
`model_reasoning_effort="xhigh"` to `/usr/bin/codex`. The unchanged
`first_batch.py::run_environment` and `bench-agent-profile-common` establish the
runner-to-profile-wrapper-to-subscription-wrapper chain. Both driver reports
record the subscription-wrapper hash
`0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078`.
Every captured Direct invocation records profile-wrapper hash
`25003467998e8d883f0a02da9da79a17955926bff9025ca1cee8aaab7facbde2`;
the Work Leaf app-server records
`c04015d8d3a1e91783951f40aa2dd7f07855e1629be50e43a721ef54d5ab3b98`,
each matching its driver report. The temporary generated profile wrappers are
absent after normal driver cleanup; the recorded hashes and frozen generator
remain. No captured profile contradicts the declared subscription-only route.
This is launch-chain and effective-profile evidence, not an independent account,
network-endpoint or billing receipt audit.

## Interpretation limits

Explicit command-line overrides outrank user-level configuration in the documented
configuration precedence. This supports the narrow matching-profile finding,
not immutability of all other settings. [Official configuration documentation](https://learn.chatgpt.com/docs/config-file/config-basic).

The identity-only pin cannot establish which other values changed: for example,
project trust/configuration loading, instructions, optional features/tools, or
settings not represented in the retained profile fields. These are unresolved
possibilities, not findings that any particular setting changed. Identical
model/effort/provider identifiers do not reconstruct every effective setting or
explain the quality/token difference.

The numerical observations and saved quality outcomes remain inspectable, with
this deviation attached. They do not become an exact same-environment comparison,
a causal explanation, or evidence that the configuration drift had no effect.
The original manifest, schedule, run reports and frozen analysis remain intact;
this audit neither changes eligibility rules nor authorizes further runs.

## Hashed evidence

Paths below are relative to this batch. `D` abbreviates
`runs/direct-001/direct-001-three-feature-sequential-bench-artifacts/observation`;
`W` abbreviates
`runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts/observation`;
`A` abbreviates `W/app-server/00000422153032163608-2899396`.

| Evidence | SHA-256 |
| --- | --- |
| `FIRST-BATCH-MANIFEST.json` | `5de3e31bf4ebc06ecd330d67778eaa24b8321a67e848977911f3f761eafa3842` |
| `SCHEDULE.json` | `06336b8f9c3f147a540d9e26ec83e1e3ddfa83707556b70d3c0bfbe83633a48f` |
| `RUN-ONCE/admission.json` | `32a4f65dfb0a344c1b627ad3d72557c37f796622216461be9e308cf44e600156` |
| `D/process-invocations.jsonl` | `1ea7948e87b909d516844d567dc954b814f4453b39c6975dd484a92c62c7096d` |
| `W/process-invocations.jsonl` | `35631ba71c1d3be03d14baee0f06c3a0755c1b0d64b3efcf789d92436b38ad32` |
| `D/rollout-metadata.jsonl` | `375c9bab0377c781a5ea8c1fd02b7da7a52d06b2647251d8033b6a784316a4a8` |
| `W/rollout-metadata.jsonl` | `d4692c2fdfe8639bacabd931dae54886139fe45a9bf74a1110b2d02c92e80e9e` |
| `A/client-to-server.raw` | `3be1a143ed5f66960daa92816800a8ac1ec63ae9af9ed43f1eca93ba90ad7ece` |
| `A/server-to-client.raw` | `701a611f120e7718f1ebabfac8d4d676a0ab76da237c8c3201d0b2b76a65eb9f` |
| `logs/work-leaf-001.log` | `a5d6a305b14b22979046d1cf368ebcf7a244ebc2c0067fd2b34276f9fef02ffe` |

The external launcher and native binary hashes remain respectively
`61b0194f3bb6534439c8d26a3ed57d0805f84b884588b761795323eeb92fcf70`
and `56ef98ab4032d317ab26e9b5e5a175650717351edb16ed9cde0cb6d1734d62da`,
at the exact paths listed in the manifest. The rollout inventories retain each
individual source path and digest; no credentials or configuration copies are
included in this audit.
