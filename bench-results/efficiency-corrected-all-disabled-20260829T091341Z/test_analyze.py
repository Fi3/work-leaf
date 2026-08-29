#!/usr/bin/env python3

import unittest

import analyze


class AnalyzeTests(unittest.TestCase):
    def test_cap_uses_the_larger_conservative_ceiling(self) -> None:
        result = analyze.cap_audit(2, 30_000)

        self.assertEqual(result["rounded_missing_raw_token_cap"], 800_000)
        self.assertEqual(result["context_output_and_prompt_upper_bound"], 802_800)
        self.assertEqual(result["applied_missing_raw_token_cap"], 802_800)
        self.assertEqual(result["cap_headroom"], 0)

    def test_difference_interval_propagates_both_uncertain_ranges(self) -> None:
        self.assertEqual(
            analyze.difference_interval(
                {"lower": 10, "upper": 20}, {"lower": 3, "upper": 8}
            ),
            {"lower": 2.0, "upper": 17.0},
        )

    def test_usage_add_reconciles_a_mislabeled_reviewer_thread(self) -> None:
        self.assertEqual(
            analyze.usage_add(
                {"input_tokens": 10, "output_tokens": 2},
                {"input_tokens": 4, "output_tokens": 1},
            ),
            {"input_tokens": 14, "output_tokens": 3},
        )

    def test_review_summary_requires_git_and_marker_first(self) -> None:
        state = {
            "snapshot": {
                "sessions": [
                    {
                        "id": f"review-user-{number}",
                        "lines": [
                            "@work-leaf locks run . -- git show abc",
                            "NO_FINDINGS\n@work-leaf done",
                        ],
                    }
                    for number in range(1, 4)
                ]
            }
        }

        summary = analyze.review_summary(state)

        self.assertTrue(summary["all_reconstructed_from_git"])
        self.assertTrue(
            all(row["marker_before_done"] for row in summary["sessions"])
        )

    def test_review_summary_rejects_done_before_marker(self) -> None:
        state = {
            "snapshot": {
                "sessions": [
                    {
                        "id": f"review-user-{number}",
                        "lines": [
                            "@work-leaf locks run . -- git show abc",
                            "@work-leaf done\nNO_FINDINGS",
                        ],
                    }
                    for number in range(1, 4)
                ]
            }
        }

        with self.assertRaisesRegex(ValueError, "no review marker"):
            analyze.review_summary(state)


if __name__ == "__main__":
    unittest.main()
