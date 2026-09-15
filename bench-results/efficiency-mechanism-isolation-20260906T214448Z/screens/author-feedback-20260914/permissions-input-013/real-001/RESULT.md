# Actual integration-profile reproduction

Completed2026-09-15 01:57:53.786961 UTC in15.191192 seconds. Both owned turns
complete, one actual git-status command runs, and the repository/index/HEAD stay
unchanged. All211 admission pins match at both endpoints. Existing ChatGPT
subscription auth; no copied credentials or normal WL change.

BUG009 reproduces cheaply: first developer message matches the saved reference
SHA6278885a5704b96b3d52083d8e755d942c07bdde5b57b5d20a919ba0d4f81e9e;
the second injects681b5ba580483b28751dbafd95e30f80ebb6ab9cc2d3ca4462f0f187d0784070,
absent from the saved reference. Native world-state permissions go from sixteen
approved prefixes to zero, with the disabled integration profile unchanged.
This establishes a small failing qualification fixture, not a repaired setup.

The once-only native core and exact public/prompt joins report no errors or
unfinished tails: three responses, two turns,47,592 raw tokens. See NATIVE-AUDIT,
NATIVE-INPUTS, ONCE-ONLY-AUDIT-COMMAND, native-core cache and TERMINAL. Reuse them;
do not invoke the native core again. No token-effect estimate follows.

Two new local scenario tests and required Cargo gates pass. The actual configured
agent executes the exact new command/resume scenario; the ready status of BUG009
is still negative because the model input fails. The diagnostic alone does not
authorize another full workflow. A new bounded repair attempt can examine whether
inheriting the persisted permission settings on resume avoids resetting them,
but it must verify the actual resulting permissions and entire developer input.
