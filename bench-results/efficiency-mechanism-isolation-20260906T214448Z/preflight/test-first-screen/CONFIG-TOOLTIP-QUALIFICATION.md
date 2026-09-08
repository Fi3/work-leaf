# Exact prelaunch tooltip-state qualification

PASS: the sole remaining parsed configuration difference is
`tui.model_availability_nux["gpt-6-astra"]`, previous integer **3** versus
current **4**. This is the internal startup-tooltip state described in the
[official configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference),
not a model or reasoning-effort selection. OpenAI Docs supplied that field
classification; the identity proof is the local exact-hash comparison below.

Independent read-only operator `2ef93e` (exit 0) uniquely joins all 16 declared
trust rows to their four retained source files, verifies each current value is
exactly `{trust_level: trusted}`, and removes only those keys in memory. It then
sets only the Astra tooltip entry to 3. The complete reconstructed values match:

| Complete value | Reconstructed and saved W SHA-256 |
| --- | --- |
| Parsed configuration | `e91e44495343edc3d4ced57caa5bb296c01f0447688d27a1577f8d1243a3a389` |
| Original four-notice behavioral projection | `78cd609bb3fb021622bfb6cf84afd8954c4169d18b7f2f8c4de6156ef0735512` |

The parsed result also equals the retained 09:06 reconstruction receipt
(`CONFIG-RECONSTRUCTION.json`, SHA `f86dd26ee1383a08506d3aafde71f8419cd9f2406582df1a460682c3594d7e05`).
That receipt records prior raw SHA `21bcb4825009b9561fa200a806ccc7d68993cbd4e718342b257a594dfaf31908`;
this review does not claim access to an old raw configuration backup.
Current raw SHA `d36b9caec082759492578037173fe3bca099aad63f8e2553314601d2506e366d`
and all 12 retained source/evidence endpoints match before and after the check.
Global `gpt-6-astra`/`max`, the other two tooltip counters (4), and every other
parsed setting remain unchanged by the reconstruction.

Benchmark model/effort are separately pinned to `gpt-5.5`/`xhigh` by
`runner_work_units.py::run_environment` (line 197). The saved
`infrastructure/driver-source/bench-three-features` resolves those explicit
variables before global-config fallbacks (lines 641 and 657). It passes the model
to the daemon (line 1110); `prepare_bench_codex_profile` (line 725) delegates to
`bench-agent-profile-common::bench_prepare_codex_profile`, whose generated Codex
wrapper supplies both `-c model` and `-c model_reasoning_effort` (lines 33–45;
additional source hash checked by `4746e4`). The reviewed `bind_daemon.py::bind` (line 177) changes only
the experiment-manifest environment path. These source bytes are pinned in the
[qualification JSON](CONFIG-TOOLTIP-QUALIFICATION.json).

A fresh phase baseline can retain the actual current tooltip value 4. The
original engine's `config_snapshot`, `config_drift`, and trust-classifier
admission remain unchanged (`runner_work_units.py`, lines 103, 130, 475).
There is no blanket `tui` exclusion or new tooltip exemption: an unexplained
later configuration change still reaches the existing drift gate.

The original 09:06 receipt and failed prelaunch probes remain byte-identical.
The separate nine- and 270-candidate tests (`4a69f5`, `3bc26a`) correctly had no
match because their permitted prior tooltip values omitted 3. Parent discovery
`b56a08` reports one match in its finite 6,480 candidates; this independent check
verifies only that one proposed substitution. The JSON retains the exact
operator source, results, proof-row physical locators, and source hashes.
No configuration write, CLI/model invocation, provider call, source change, or
extra benchmark observation occurs in this qualification.
