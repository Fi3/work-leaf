# C21 observer-frame derivative qualification

The derivative is provider-free and separate from admitted runtime and frozen collector sources. It qualifies the known observer frame metadata, not the skipped focused diagnostic check or full scenario success.

Exact review target:

- Helper `audit_observer_frames.py`: `ccfb4cc3fe2a24c6496f4a749e6c95f147d6b5fefa7fd4e91dedc6f40be1961f`.
- Tests `test_observer_frames.py`: `7549dc5a391ef691015a3813c5a4d04e3f01f0ace921f249ab5ca003419f0496`.
- Design: `4939c45bbdf5366b4734075ccf929040d5e38cc23fe9f7f1f6882744c9e66246`.

The original frozen collector actual run is retained at `../REVIEW-EVIDENCE-SOURCE-AUDIT.json`, SHA `05cb78aa4af6d2309e5be8040dc83e740e7cc8f055d64b26bd44ac41eb32201e`, with `original/forwarded frames differ`. Source inspection identifies exactly three changed physical client frames: initialize line 1, thread/start lines 3 and 8. The former appends one raw-item notification exclusion; the latter two enable experimental raw events. All three turn/start requests and all other frames are byte-identical. No actual model prompt drift is inferred from that original checker failure.

New tests initially fail because the derivative module is absent. A further observed RED verifies that a disabled raw route fails but still retains the already verified failed terminal; the corresponding output-retention fix leaves the route rejected. Final command:

```sh
python -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/review-evidence-native-diagnostic-001/postcapture/observer-frame-correction -p test_observer_frames.py
```

Result: **9 passed**. `python -m py_compile` on both new Python files passes. Tests cover the exact permitted metadata pair, unchanged metadata identity, typed RPC identities, full journal coverage, prompt/order/capability/notification changes, unsupported settings, byte-only reserialization, proof binding, disabled-route failure, and the complete actual closed-source collector branch. The last case preserves the original audit bytes and exit 101.

Actual provider-free CLI:

```sh
python -B audit_observer_frames.py --input /absolute/INPUT.json \
  --input-sha256 88b9e79e0bb8979eaf2929240da055d743a7ef885ee1097d834bf4b14e317eb0 \
  --output /absolute/new-derived-audit.json
```

The retained create-new result is `DERIVED-AUDIT.json`, SHA `05077798a114039f91ed7191655980d27ef3f576144599de320ff27d35c01101`: exit 0, status available, no errors, **complete** issued-archive read, **31** source identities checked at the endpoint, and original terminal exit **101** retained. It uses the original source input unchanged (`../REVIEW-EVIDENCE-SOURCE-INPUT.json`, SHA `4d327b179ebf61419035dc965fac26030c5f5ffec97d97fcbe261309adf03f51`). No original collector or primitive byte is rewritten; the report records the separately compiled single-predicate primitive digest.

The independent native witness `../INDEPENDENT-NATIVE-ACCESS-WITNESS.json` (`800b6e3f431922bfd1fb1d7cc4aa9400e68f9f4f0a26d376648b2f2ec5a2eecf`) covers both exact captured native threads, including the usage-less author, and all three accepted/public/native user inputs. Reviewer user line 7 → function_call line 11 → function_call_output line 14 uses explicit same-turn metadata and call ID `call_ImY5jxZsBNY3IN345zxheUsf`; the complete 6,221-byte output body equals the issued archive SHA `804b3d045c0f7a562894ad6041e42b6fb67adff9e91c310cda1243bfe329e567`.

Original observer analyze/extract exit 2 results are retained separately as `../OBSERVER-ANALYZE-ORIGINAL.json` and `../OBSERVER-EXTRACT-ORIGINAL.json`. Missing interrupted usage, executable marker scan flags and the usage-less same-cwd thread discrepancy are not erased by delivery/retrieval qualification. This derivative performs no usage accounting, quality test, provider launch or corrected total. The actual diagnostic remains failed because its reviewer returned `NO_FINDINGS` without the required scripted focused check. Full opaque-context delivery and native retrieval are a narrower verified behavior.

Added frame/source work is linear in bytes and records. The original collector's explicitly flagged partial-read primitive O(R × B) remains unchanged. No agent-facing workflow is affected by this offline evidence helper, so a new provider verification is neither required nor authorized for it. Independent review of the derivative is still a separate gate.
