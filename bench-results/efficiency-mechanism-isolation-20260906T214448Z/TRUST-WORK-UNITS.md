# Prospective own-workflow trust integrity

`runner_work_units.py` freezes `trust_work_units.py` with the phase, records its SHA256,
and applies `TrustClassifier` only to future work-unit phases. Historical supervisors,
screen flags, raw configuration hashes, parsed hashes, behavioral hashes, and the
legacy `behavioral_config_drift_detected` flag retain their original meanings.
`unexplained_config_drift_detected` is a separate, sticky admission gate. A failure
withholds new workflows without terminating active workflows or replacing outcomes.

An exception requires exactly a new `projects.<cwd>.trust_level = "trusted"` entry
for an already launched row's exact primary AppServer working directory. The actual
observation configuration must identify that row and study; a runtime path prefix is
insufficient. Removing only such additions must reproduce the complete baseline
parsed configuration hash. Other additions, removals, preexisting entry changes,
unknown fields, missing evidence, and unsupported shapes fail closed.

The live proof requires preceding read-only thread starts, the first prescribed
full-access thread start, the exact forwarded request (with only the existing raw-event
opt-in), a successful reply with the same typed RPC ID, a timestamped matching thread
notification, and matching native session/permission/turn context. Pre-spawn and
pre-linearize project inventories must identify the same canonical Git project root,
be valid, and have equal entry hashes. A bounded read-only rehash checks the actual
project layers at the transition; no Git snapshot, config edit, provider call, prompt,
message, or provider response wait is introduced by classification.

An absent or incomplete native attestation can be explicitly pending for at most 30
seconds from its first-seen snapshot, but only after all other core evidence and
actual project-layer identity are verified. `trust-pending.jsonl` retains that initial
core prefix proof and deadline. New admissions wait; active generation is untouched.
The later verified proof links its pending identity and has its own cutoff. Wrong or
malformed native evidence is a failure, not a pending exception. Offline replay checks
both stages and verifies that no workflow started during the pending interval.

`trust-evidence.jsonl` retains safe identity metadata and SHA256/byte lengths of the
exact capture prefixes available at each live proof. It contains no copied global
configuration, native prompt contents, or credentials. `trust-final.json` retains the
original snapshot records and their proof references. Evidence discovery follows the
actual row's staging or published observation directory, so artifact publication can
rename the staging directory without breaking source identity.

`audit(phase_root)` independently replays the source prefixes and their protocol
semantics, checks the complete final inventory journal (including later appends), and
requires the current external global configuration to match the recorded final raw
hash. It reconstructs the baseline and every historical parsed hash in memory by
removing/reinstating only the verified additions. It returns `valid`, safe `errors`,
source identities, and verified snapshot indices. The analyzer must import the
hash-pinned helper and call this function; a stored success boolean is insufficient.

This audit is **not self-contained global configuration replay**: the matching external
final file is required because configuration contents are not archived. A later raw
edit or unavailable file invalidates replay. A parsed notice-bookkeeping change
retains its legacy label but the future live gate rejects it with
`parsed-bookkeeping-history-not-replayable`; no admitted parsed history is knowingly
accepted without an exact reconstruction path. Snapshots establish identity only
at their recorded boundaries, not continuous immutability or a general effective
configuration resolution claim.

Offline verification:

```sh
python3 -B bench-results/efficiency-mechanism-isolation-20260906T214448Z/test_trust_work_units.py
python3 -B bench-results/efficiency-mechanism-isolation-20260906T214448Z/test_runner_work_units.py
```
