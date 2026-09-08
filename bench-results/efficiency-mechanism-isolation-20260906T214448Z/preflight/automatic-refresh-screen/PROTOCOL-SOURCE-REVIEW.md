# C08 prospective protocol and source compatibility review

No source/protocol blocker was found at this cutoff. This is a bounded read-only review, not phase admission. The final prepared manifest, actual checkout/configuration and immutable dependency closure require separate prelaunch verification. The reviewer authored the private C08 renderer and therefore does not independently certify that implementation; the separately owned runtime review remains the authority for that gate.

## Exact reviewed inputs

| Input | SHA-256 |
| --- | --- |
| `PROTOCOL-AUTOMATIC-REFRESH-SCREEN.md` | `889fc49bdc40224cc9616a32cc743a351040c82e02210f700d2920d022f8fa2d` |
| `BASELINE-COMPATIBILITY.md` | `d57e51b3670ea04580fde5fab3ae4a95ff7a9aa217a6114e0d09d32b4c08e68e` |
| `BUILD-ATTESTATION.json` | `161e0858875782d999cb1833d2593d9ce43a2360feea52e43cec18cdbe4fde2c` |
| Unchanged engine | `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e` |
| C08 runner | `eeba247844dfa073754b01060d50de7898728d422aab27ff69af42d97b8b15fe` |
| Reused exact configuration qualification | `b789b3c3c2d359fa34a65765ec97b8638d5b9da7c70bb4c6e73abcf8be69d159` |

The complete protocol, compatibility note and build attestation were read, together with the referenced C15/saved-W source qualifications and tooltip-state qualification. The separate [runner review](../C08-RUNNER-INDEPENDENT-REVIEW.md) binds the adapter's full source review and six provider-free automatic checks; no test was rerun here.

## Independently checked source facts

Read-only tool `4771fa` matched all 50 attested inputs against four authorities: the actual private source tree, actual published root files, private build commit `e497ff5df5bf84e169aa4460953383639df956f8`, and published commit `d0a22e78326fccd60e1639fef34da75cfca8947c`. Both declared release binaries matched their hashes: Work Leaf `9a257e52c89bbc82dba24f91632a7541a9969b0bbb60f12545782c0628f7de9b` and orchestrator `149488946159308cd358e3393e6f01c1640885c6b53a59d9e111672c8549a554`.

Tool `704ba1` independently compared all 47 previous C15 build inputs: 45 are identical; only `src/bench_experiment.rs` and `src/orchestrator.rs` differ. The three additional inputs are the production `src/bench_automatic_refresh.rs` and two cfg(test) modules. This matches the declared incremental source boundary. The actual driver checkout remains clean at `3c907f7266b63efa1558213f3e49587e999b899d`; all five driver script hashes match the saved-W compatibility table. No helper, driver, source or configuration bytes were changed by this review.

Current global configuration was read only for its SHA-256, which is exactly `d36b9caec082759492578037173fe3bca099aad63f8e2553314601d2506e366d`. It matches the retained qualification's exact current-byte scope. This is not a new blanket exclusion of trust/tooltip settings and does not waive later drift. The prior parsed reconstruction remains separately source-grounded; fresh actual checkout inventories and eligible ancestors remain per-launch gates.

## Contract consistency and remaining admission gates

The protocol declares three complete modified-only workflows in one wave and randomizes launch order only. `runner_automatic_refresh.py::make_plan/validate_plan` enforce that exact population and sole v7 condition; the unchanged engine owns environment, scheduling, stop handling, manifest verification and all retained outcomes. `runner_work_units.py::run_environment` retains the 7200/1800/300-second stage/busy/idle limits, subscription transport and existing observer timing. The ordinary four-field v7 experiment does not invoke the C15 dynamic daemon binding.

The selection boundary agrees with the renderer: an actual ordinary rejected edit/patch, tracked changed snapshot and available nonempty diff within the existing 48-KiB bound. The two response-local coherence spans and current-text replacement are declared. No controlled host edit, forced failure, extra read or prescribed repair belongs to a natural screen workflow. Default/prior-schema paths and unavailable/oversized/unchanged/untracked behavior remain explicit non-targets. Private v6 enrollment is gated on `SCHEMA_V6`; v4 candidate selection and v5 archive selection are likewise inactive under v7.

All six saved W controls, including failed 010, remain in the descriptive comparison; no fresh controls or replacement authority is inferred. Their already retained source-bound receipts are reused, not recalculated here. New-row accounting follows a separately declared closed-capture scope, exact source/response identities and original finite/unbounded/ineligible rules. No warning is waived by a passing source/delivery qualification, and no command count or body size is treated as a saving percentage.

Before generation, the actual frozen phase still must bind these bytes, both executables, the full path-preserving helper/scorer/driver/transport closure, current config and all qualification receipts. Exact public/native identity and semantic closure of the sole real C08 qualification must be reviewed, not inferred from exit 0. Resource thresholds, API-key stripping, fresh independent checkouts and no unrelated build load are explicit operational gates. This review performs no preparation, schedule allocation, Cargo invocation, provider call, collector, executor, accounting replay or baseline extraction.
