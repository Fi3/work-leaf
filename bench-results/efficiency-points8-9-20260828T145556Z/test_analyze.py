import importlib.util
import unittest
from pathlib import Path


STUDY_DIR = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("points89_analyze", STUDY_DIR / "analyze.py")
assert SPEC is not None and SPEC.loader is not None
ANALYZE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ANALYZE)


class IntervalArithmeticTests(unittest.TestCase):
    def test_work_leaf_interval_adds_the_declared_interruption_cap(self) -> None:
        self.assertEqual(
            ANALYZE.raw_interval(12_000_000, 5),
            {"lower": 12_000_000, "upper": 14_000_000},
        )

    def test_missing_usage_cap_expands_when_prompt_headroom_is_too_small(self) -> None:
        self.assertEqual(
            ANALYZE.conservative_missing_usage_cap(59, 851_105),
            23_648_705,
        )

    def test_missing_usage_cap_keeps_the_rounded_cap_when_it_is_larger(self) -> None:
        self.assertEqual(
            ANALYZE.conservative_missing_usage_cap(49, 529_974),
            19_600_000,
        )

    def test_mean_interval_keeps_lower_and_upper_bounds_separate(self) -> None:
        self.assertEqual(
            ANALYZE.mean_interval(
                [
                    {"lower": 10, "upper": 20},
                    {"lower": 30, "upper": 50},
                ]
            ),
            {"lower": 20.0, "upper": 35.0},
        )

    def test_difference_interval_uses_opposite_endpoints(self) -> None:
        self.assertEqual(
            ANALYZE.difference_interval(
                {"lower": 30.0, "upper": 40.0},
                {"lower": 10.0, "upper": 25.0},
            ),
            {"lower": 5.0, "upper": 30.0},
        )

    def test_minimum_reduction_never_claims_a_negative_saving(self) -> None:
        self.assertEqual(ANALYZE.minimum_reduction_percent(100, 75), 25.0)
        self.assertEqual(ANALYZE.minimum_reduction_percent(100, 110), 0.0)

    def test_token_usage_ignores_observer_only_metadata(self) -> None:
        usage = {field: index for index, field in enumerate(ANALYZE.USAGE_FIELDS, 1)}
        self.assertEqual(
            ANALYZE.token_usage({**usage, "thread_count": 8}),
            usage,
        )

    def test_interruption_count_accepts_the_observer_integer_field(self) -> None:
        self.assertEqual(ANALYZE.interruption_count(57), 57)


if __name__ == "__main__":
    unittest.main()
