#!/usr/bin/env python3
"""Fixed-population, create-once checks for the work-unit allocation."""

import importlib.util
import itertools
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AllocationTests(unittest.TestCase):
    def setUp(self):
        self.module = load("allocate_work_units")

    def test_all_324_allocations_have_six_of_each_and_are_valid_runner_plans(self):
        runner = load("runner_work_units")
        population = self.module.block_population()
        self.assertEqual(len(population), 18)
        seen = set()
        for first, second in itertools.product(population, repeat=2):
            choices = iter((first, second))
            plan = self.module.make_plan("units", "workflow", lambda _population: next(choices))
            runner.validate_plan(plan)
            labels = tuple(row["condition"] for row in plan["runs"])
            self.assertEqual(labels.count("control"), 6)
            self.assertEqual(labels.count("buildable-work-unit-incremental"), 6)
            self.assertEqual(plan["randomization"]["joint_population_size"], 324)
            seen.add(labels)
        self.assertEqual(len(seen), 324)

    def test_claim_precedes_two_draws_and_existing_directory_never_redraws(self):
        with tempfile.TemporaryDirectory() as temporary:
            phase = Path(temporary) / "units"
            draws = []
            def choose(population):
                self.assertTrue((phase / "ALLOCATION-CLAIM.json").exists())
                draws.append(1)
                return population[len(draws)]
            with patch.object(self.module.random.SystemRandom, "choice", side_effect=choose):
                plan = self.module.allocate(phase, "units", "workflow")
                with self.assertRaises(FileExistsError):
                    self.module.allocate(phase, "units", "workflow")
            self.assertEqual(len(draws), 2)
            self.assertEqual(json.loads((phase / "ALLOCATION.json").read_text()), plan)
            self.assertEqual(plan["allocation_provenance"]["schema"], "work-leaf-work-unit-allocation-v2")
            self.assertFalse(plan["allocation_provenance"]["provider_work_started"])

    def test_failed_draw_claim_is_retained_without_schedule_or_retry(self):
        with tempfile.TemporaryDirectory() as temporary:
            phase = Path(temporary) / "units"
            with patch.object(self.module.random.SystemRandom, "choice", side_effect=ValueError("synthetic failure")):
                with self.assertRaises(ValueError):
                    self.module.allocate(phase, "units", "workflow")
            self.assertTrue((phase / "ALLOCATION-CLAIM.json").exists())
            self.assertFalse((phase / "ALLOCATION.json").exists())
            with self.assertRaises(FileExistsError):
                self.module.allocate(phase, "units", "workflow")

    def test_wrong_or_relative_root_is_rejected_before_claim(self):
        with tempfile.TemporaryDirectory() as temporary:
            for root in (Path("relative"), Path(temporary) / "different"):
                with self.assertRaises(ValueError):
                    self.module.allocate(root, "units", "workflow")
                self.assertFalse(root.exists())


if __name__ == "__main__":
    unittest.main()
