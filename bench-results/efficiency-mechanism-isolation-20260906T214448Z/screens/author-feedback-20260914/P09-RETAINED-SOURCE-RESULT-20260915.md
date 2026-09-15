# Pre-integration retained-source feasibility

Checked 2026-09-15 02:18 UTC, within the ten-minute local scope; no generation.
A fresh bare object store at /tmp/wl-p09-retained-source.1HONB3/objects.git imports
all three saved bundles. Only that temporary store is modified; root Git and
original bundles/checkpoints remain untouched.

| Run | Recorded pre-integration HEAD | Object available after all bundle imports |
| --- | --- | --- |
| 004 | 7b16e5fbfa3f2cfd604c31405771a916f29cb938 | Yes, commit |
| 005 | 138e1b51ecd7c989c8f48b37db387fbbe3fe6dac | No |
| 006 | 67fe0fd538d138b9fe41c2e84a8e93342c26966b | No |

Exact probes are in P09-RETAINED-SOURCE-EVIDENCE-20260915.json. The bundles live
under each qualified P09 run's patches/fail (004/005) or patches/pass (006);
target identities come from observation/git-checkpoints/files/pre-linearize/head.txt.
The completed006 bundle advertises final HEADcebee95f079d12d9da3ebee7beadcd1bc874de52,
not the provisional pre-integration head.

There is no ready three-run original-source restoration through these bundles.
This does not prove every original tree is unreconstructable from every retained
artifact. It rejects the cheap bundle-reuse route for the full roster. A lone
available004 object does not qualify the resumed permission input or permit
selecting only a convenient successful continuation. No integration continuation
or complete-workflow rerun is admitted. Valid earlier stage costs and work chains
remain usable with their stated limits.
