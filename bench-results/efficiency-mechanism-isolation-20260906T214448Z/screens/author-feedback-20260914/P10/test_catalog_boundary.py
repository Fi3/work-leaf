import unittest

from catalog_boundary import compare_catalog_boundaries


def start(turn):
    return {"type": "event_msg", "payload": {"type": "task_started", "turn_id": turn}}


def developer(catalog):
    return {
        "type": "response_item",
        "payload": {
            "type": "message",
            "role": "developer",
            "content": [{
                "type": "input_text",
                "text": "<skills_instructions>\n" + catalog + "\n</skills_instructions>",
            }],
        },
    }


def user():
    return {
        "type": "response_item",
        "payload": {"type": "message", "role": "user", "content": [
            {"type": "input_text", "text": "Execute the next admitted operation."}
        ]},
    }


class CatalogBoundaryTests(unittest.TestCase):
    def test_same_profiles_at_earlier_boundary_are_rejected(self):
        reference = [start("r1"), developer("full"), user(),
                     start("r2"), user(), start("r3"), developer("short"), user()]
        actual = [start("a1"), developer("full"), user(),
                  start("a2"), developer("short"), user(),
                  start("a3"), user()]
        self.assertFalse(compare_catalog_boundaries(
            reference, actual, ["r1", "r2", "r3"], ["a1", "a2", "a3"]
        )["matches"])

    def test_catalog_from_later_stage_cannot_qualify_author(self):
        reference = [start("r1"), developer("full"), user(),
                     start("r2"), user(),
                     start("later-review"), developer("short"), user()]
        actual = [start("a1"), developer("full"), user(),
                  start("a2"), developer("short"), user()]
        self.assertFalse(compare_catalog_boundaries(
            reference, actual, ["r1", "r2"], ["a1", "a2"]
        )["matches"])

    def test_inherited_catalog_matches_same_effective_state(self):
        reference = [start("r1"), developer("full"), user(), start("r2"), user()]
        actual = [start("a1"), developer("full"), user(),
                  start("a2"), developer("full"), user()]
        self.assertTrue(compare_catalog_boundaries(
            reference, actual, ["r1", "r2"], ["a1", "a2"]
        )["matches"])

    def test_missing_requested_turn_fails_closed(self):
        rows = [start("one"), developer("full"), user()]
        self.assertFalse(compare_catalog_boundaries(
            rows, rows, ["one", "absent"], ["one"]
        )["matches"])

    def test_extra_actual_turn_requires_an_explicit_reference(self):
        reference = [start("r1"), developer("full"), user()]
        actual = [start("a1"), developer("full"), user(), start("a2"), user()]
        self.assertFalse(compare_catalog_boundaries(
            reference, actual, ["r1"], ["a1", "a2"]
        )["matches"])

    def test_duplicate_requested_ids_fail_closed(self):
        rows = [start("one"), developer("full"), user()]
        self.assertFalse(compare_catalog_boundaries(
            rows, rows, ["one", "one"], ["one", "one"]
        )["matches"])

    def test_profile_missing_before_request_fails_closed(self):
        rows = [start("one"), user()]
        self.assertFalse(compare_catalog_boundaries(
            rows, rows, ["one"], ["one"]
        )["matches"])

    def test_source_order_is_required(self):
        rows = [start("one"), developer("full"), user(), start("two"), user()]
        self.assertFalse(compare_catalog_boundaries(
            rows, rows, ["two", "one"], ["one", "two"]
        )["matches"])


if __name__ == "__main__":
    unittest.main()
