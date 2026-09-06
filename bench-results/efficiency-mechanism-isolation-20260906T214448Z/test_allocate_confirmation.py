#!/usr/bin/env python3
"""Provider-free tests of the predeclared confirmation allocation."""

import importlib.util
import itertools
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AllocationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.allocation = load("allocate_confirmation")
        cls.runner = load("runner")

    def test_population_is_exactly_all_eighteen_valid_allocations(self):
        population = self.allocation.block_population()
        expected = {slots for slots in itertools.combinations(range(1, 7), 3)
                    if slots not in {(1, 2, 3), (4, 5, 6)}}
        self.assertEqual(set(population), expected)
        self.assertEqual(len(population), 18)
        self.assertEqual(len(set(itertools.product(population, repeat=2))), 324)
        for allocation in population:
            self.assertEqual(len(allocation), 3)
            self.assertIn(len(set(allocation) & {1, 2, 3}), {1, 2})
            self.assertIn(len(set(allocation) & {4, 5, 6}), {1, 2})

    def test_every_joint_allocation_produces_a_valid_fixed_twelve_run_plan(self):
        population = self.allocation.block_population()
        for pair in itertools.product(population, repeat=2):
            chooser = Mock(side_effect=pair)
            plan = self.allocation.make_plan("confirm-fixture", "run", "ack-validation-unlimited", chooser)
            self.runner.validate_plan(plan)
            self.assertEqual(len(plan["runs"]), 12)
            self.assertEqual(plan["phase_kind"], "confirmation")
            self.assertEqual(sum(row["condition"] == "control" for row in plan["runs"]), 6)
            self.assertEqual(plan["randomization"]["actual_variant_slots_by_block"], {
                "confirm-fixture-block-01": list(pair[0]), "confirm-fixture-block-02": list(pair[1])})

    def test_independent_draws_use_the_same_complete_population_once_per_block(self):
        population = self.allocation.block_population()
        chooser = Mock(side_effect=[population[0], population[-1]])
        plan = self.allocation.make_plan("confirm-fixture", "run", "command-guidance-neutral", chooser)
        self.assertEqual(chooser.call_count, 2)
        for call in chooser.call_args_list:
            self.assertEqual(call.args, (population,))
        randomization = plan["randomization"]
        self.assertEqual(randomization["per_block_population"], [list(item) for item in population])
        self.assertEqual(randomization["per_block_population_size"], 18)
        self.assertEqual(randomization["joint_population_size"], 324)
        self.assertTrue(randomization["independent_blocks"])
        self.assertEqual(randomization["unit"], "workflow")
        self.assertEqual(randomization["scheme"], "within_block")
        self.assertTrue(randomization["mixed_waves"])

    def test_numbered_identifiers_are_condition_blind_and_unique_to_the_phase(self):
        ids = []
        for variant in self.allocation.VARIANTS:
            plan = self.allocation.make_plan("confirm-fixture", "run", variant, lambda population: population[0])
            ids.append([row["run_id"] for row in plan["runs"]])
        self.assertEqual(ids[0], ids[1])
        self.assertEqual(ids[0], [f"confirm-fixture-run-{number:03d}" for number in range(1, 13)])
        self.assertTrue(all("control" not in value and not any(v in value for v in self.allocation.VARIANTS)
                            for value in ids[0]))
        other = self.allocation.make_plan("confirm-other", "run", self.allocation.VARIANTS[0], lambda p: p[0])
        self.assertFalse(set(ids[0]) & {row["run_id"] for row in other["runs"]})

    def test_disallowed_variants_and_unsafe_identifiers_are_rejected_before_drawing(self):
        for phase, prefix, variant in (("confirmation", "run", "control"),
                                       ("confirmation", "run", "unknown"),
                                       ("../confirmation", "run", self.allocation.VARIANTS[0]),
                                       ("confirmation", "../run", self.allocation.VARIANTS[0]),
                                       ("x" * 96, "run", self.allocation.VARIANTS[0])):
            chooser = Mock()
            with self.subTest(phase=phase, prefix=prefix, variant=variant), self.assertRaises(ValueError):
                self.allocation.make_plan(phase, prefix, variant, chooser)
            chooser.assert_not_called()

    def test_cli_writer_uses_system_random_and_persists_the_original_plan(self):
        with tempfile.TemporaryDirectory() as root:
            phase_root = Path(root) / "confirm-fixture"
            population = self.allocation.block_population()
            randomizer = Mock()
            randomizer.choice.side_effect = [population[3], population[7]]
            with patch.object(self.allocation.random, "SystemRandom", return_value=randomizer) as factory:
                plan = self.allocation.allocate(phase_root, "confirm-fixture", "run", self.allocation.VARIANTS[0])
            factory.assert_called_once_with()
            self.assertEqual(randomizer.choice.call_count, 2)
            self.assertEqual(json.loads((phase_root / "ALLOCATION.json").read_text()), plan)
            self.runner.validate_plan(plan)
            self.assertTrue((phase_root / "ALLOCATION-CLAIM.json").is_file())
            self.assertFalse((phase_root / "RUN-ONCE").exists())
            self.assertEqual(plan["allocation_provenance"]["provider_work_started"], False)

    def test_existing_phase_refuses_overwrite_or_another_random_draw(self):
        with tempfile.TemporaryDirectory() as root:
            phase_root = Path(root) / "confirm-fixture"
            phase_root.mkdir()
            retained = phase_root / "ALLOCATION.json"
            retained.write_text("retained evidence\n")
            with patch.object(self.allocation.random, "SystemRandom") as factory, self.assertRaises(FileExistsError):
                self.allocation.allocate(phase_root, "confirm-fixture", "run", self.allocation.VARIANTS[0])
            factory.assert_not_called()
            self.assertEqual(retained.read_text(), "retained evidence\n")

    def test_phase_directory_name_must_match_declared_phase_before_claim(self):
        with tempfile.TemporaryDirectory() as root:
            phase_root = Path(root) / "mismatched"
            with self.assertRaises(ValueError):
                self.allocation.allocate(phase_root, "confirm-fixture", "run", self.allocation.VARIANTS[0])
            self.assertFalse(phase_root.exists())

    def test_failed_draw_leaves_claim_and_cannot_be_silently_redrawn(self):
        with tempfile.TemporaryDirectory() as root:
            phase_root = Path(root) / "confirm-fixture"
            randomizer = Mock()
            randomizer.choice.side_effect = RuntimeError("synthetic random-source failure")
            with patch.object(self.allocation.random, "SystemRandom", return_value=randomizer), self.assertRaises(RuntimeError):
                self.allocation.allocate(phase_root, "confirm-fixture", "run", self.allocation.VARIANTS[0])
            self.assertTrue((phase_root / "ALLOCATION-CLAIM.json").is_file())
            with self.assertRaises(FileExistsError):
                self.allocation.allocate(phase_root, "confirm-fixture", "run", self.allocation.VARIANTS[0])


if __name__ == "__main__":
    unittest.main()
