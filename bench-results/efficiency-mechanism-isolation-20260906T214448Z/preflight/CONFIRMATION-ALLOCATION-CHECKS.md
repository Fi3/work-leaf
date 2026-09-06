# Confirmation allocation checks

`allocate_confirmation.py` is an offline implementation of the frozen twelve-workflow design.
It does not select a candidate, inspect outcomes, or invoke a provider. The caller supplies the
eligible variant after the screen; the resulting `ALLOCATION.json` is input to `runner.py prepare`.
Allocation and provider admission are separate steps.

The helper draws independently from all eighteen permitted six-slot allocations for each of two
blocks. Each block has three control and three variant workflows, split into two mixed waves of
three. Numbered run identities do not depend on the drawn labels. A fresh directory is claimed
before either draw; a failed draw leaves that claim and cannot silently become a reroll.

Verification: the missing-helper test failed before implementation; all nine tests pass under both
implementer and independent reviewer execution. Exhaustive test coverage submits all 324 joint
allocations to the frozen runner's actual `validate_plan`, and checks refusal before random draws
for existing phase directories. The reviewer reports no remaining findings.

The test intentionally enumerates the Cartesian product of the eighteen block allocations
(18² = 324 cases). Production performs two draws from a fixed eighteen-element population and
does not enumerate a growing quadratic or combinatorial joint space.

No real-agent workflow is affected: this helper only creates an offline allocation record. The
runtime, transport, model settings, treatment transformations, and live phase remain unchanged.

Source identities:

- Helper SHA-256: `57c2188bf3f641a12eaa8942ace1d8bc5ce3aa5cbe7a8fc8e0fd1573af121ff6`.
- Test SHA-256: `7b83820df4ef9cc5e46b16517b56e842255fca31390e6f898dab97679a4a2616`.
