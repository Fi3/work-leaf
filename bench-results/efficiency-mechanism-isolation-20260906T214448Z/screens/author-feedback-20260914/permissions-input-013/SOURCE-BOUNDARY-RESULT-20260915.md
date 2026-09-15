# Project-trust source mismatch

Checked2026-09-15 03:07 UTC, within the twenty-minute source discriminator.
The16initial native approved prefixes exactly equal the tracked checkout file
.codex/rules/default.rules, SHA6a4de4831b92510a5c7e1071bebc6c5a4f9191c3a5c93817077b1c6b30e6067e.
The frozen short-catalog config explicitly trusts the reference cwd but contains
no project-trust entry for the diagnostic cwd or its ancestors. The previous
home-directory-only rule search missed this project-level source. Its dated
negative statement does not establish absence of rules in the checkout.

[Official rules documentation](https://learn.chatgpt.com/docs/agent-configuration/rules)
requires a trusted project layer for loading project-local rules. The available
source therefore supplies a concrete repair hypothesis: carry reference-equivalent
trust for the exact current checkout into both fresh and resumed invocations.
Native evidence alone does not prove the internal reason fresh16 becomes resumed0.
No global rule/configuration or old native history is edited.

SOURCE-BOUNDARY-EVIDENCE-20260915.json and its command retain exact source pins,
all16patterns and both native world-state records. Three new fail-first tests
qualify exact-path quoting and rejection of broad/relative/traversal/foreign
identities; four older probe tests and required Cargo gates pass. The small
real-004 diagnostic has its own scope and must pass complete input equality
before this candidate is called a repair. It is not a token-saving hypothesis
or permission to replace the P09 workflows. BUG009 remains open pending result.
