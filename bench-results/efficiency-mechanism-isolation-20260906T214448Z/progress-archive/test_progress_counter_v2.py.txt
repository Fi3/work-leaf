"""Full-analysis progress guard. No model, benchmark or WL runtime invocation."""

import copy
import json
import pathlib
import re
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
FIXED_TOTAL = 55
BASELINE_IDS = frozenset(
    "C01 C02 C03 C04 C06 C07 C08 C09 C10 C12 C13 C14 C15 C16 C17 C19 C20 C21 "
    "C24 C25 C28 C29 C30 C31 C32 C33 C36 C37 X01 X02 X03 X05 X07 M02 J02 J04 "
    "J10 J12 N01 J04-STAGE-REACH-20260912".split()
) | {f"V{number:02}" for number in range(4, 16)}
REQUIRED_CRITERIA = {
    "G01": {"upstream_causal_chain_or_specific_benchmark_error",
            "historical_exposure_and_alternatives", "supporting_evidence_verified"},
    "G02": {"non_wl_recipe_or_correction_procedure",
            "reproduction_result_and_all_outcomes", "non_target_and_measurement_checks"},
    "G03": {"historical_population_units_and_denominator",
            "missing_usage_and_comparability_limits", "boundary_and_joint_coverage",
            "joint_residual_and_downstream_offsets", "independent_final_audit"},
}
FIXED_IDS = BASELINE_IDS | REQUIRED_CRITERIA.keys()


def validate_ledger(ledger):
    """Validate evidence declarations and counts, not the scientific truth of a claim."""
    if ledger.get("schema") != 2 or ledger.get("fixed_total") != FIXED_TOTAL:
        raise ValueError("the full-analysis ledger has a fixed total of 55")
    completed, pending = ledger["completed"], ledger["pending"]
    identities = completed + pending
    if len(identities) != FIXED_TOTAL or set(identities) != FIXED_IDS:
        raise ValueError("missing, duplicate or added fixed record ID")
    if not BASELINE_IDS <= set(completed):
        raise ValueError("completed historical records cannot be reopened")

    history = ledger["history"]
    previous_ids = BASELINE_IDS
    if not history or set(history[0]["completed"]) != BASELINE_IDS:
        raise ValueError("the audited 52/3 checkpoint must remain")
    for entry in history:
        current = (entry["done"], entry["todo"])
        current_ids = set(entry["completed"])
        if any(type(value) is not int or value < 0 for value in current):
            raise ValueError("counts must be nonnegative integers")
        if (sum(current) != FIXED_TOTAL or len(current_ids) != current[0]
                or len(entry["completed"]) != current[0]
                or not previous_ids <= current_ids <= FIXED_IDS):
            raise ValueError("total drift, duplicate history or backward progress")
        previous_ids = current_ids
    if previous_ids != set(completed):
        raise ValueError("history does not match record statuses")
    if (history[-1]["done"], history[-1]["todo"]) != (len(completed), len(pending)):
        raise ValueError("history does not match counts")

    sources = ledger["completed_record_sources"]
    source_ids = [identity for group in sources.values() for identity in group["ids"]]
    if len(source_ids) != len(BASELINE_IDS) or set(source_ids) != BASELINE_IDS:
        raise ValueError("each audited completed record requires a source")
    if any(not group["source"] for group in sources.values()):
        raise ValueError("completed source references cannot be empty")

    deliverables = ledger.get("deliverables", {})
    if set(deliverables) != set(REQUIRED_CRITERIA):
        raise ValueError("the unfinished analysis deliverables cannot be omitted")
    for identity, names in REQUIRED_CRITERIA.items():
        item = deliverables[identity]
        if set(item["criteria"]) != names:
            raise ValueError("final acceptance criteria cannot be removed")
        if identity in completed:
            if (item["status"] != "verified" or not item["evidence"]
                    or not item["verified_at"]
                    or any(value != "pass" for value in item["criteria"].values())):
                raise ValueError("final deliverables need verified evidence for every criterion")
        elif item["status"] not in {"pending", "checking"}:
            raise ValueError("pending deliverable status contradicts the counts")
    if "G02" in completed and "G01" not in completed:
        raise ValueError("reproduction must verify the identified explanation or error")
    if "G03" in completed and not {"G01", "G02"} <= set(completed):
        raise ValueError("final reconciliation requires explanation and reproduction")

    analysis = ledger["analysis"]
    final_done = set(completed) & REQUIRED_CRITERIA.keys()
    if final_done and analysis["branch"] not in {
            "verified_mechanism", "verified_benchmark_error"}:
        raise ValueError("completed deliverables must identify an accepted conclusion branch")
    if not pending:
        audit = analysis.get("independent_audit")
        if (analysis["status"] != "complete" or not analysis["conclusion_evidence"]
                or not isinstance(audit, dict) or audit.get("status") != "pass"
                or not audit.get("reviewer") or not audit.get("evidence")):
            raise ValueError("zero TODO requires a complete, independently audited analysis")
    elif analysis["status"] != "unfinished":
        raise ValueError("the analysis remains unfinished while any deliverable is pending")

    supplementary_ids = set()
    for item in ledger["supplementary"]:
        if (item["parent"] not in FIXED_IDS or item["id"] in FIXED_IDS
                or item["id"] in supplementary_ids):
            raise ValueError("substeps need distinct IDs and an existing parent")
        if item["status"] in {"pending", "checking"} and item["parent"] in completed:
            raise ValueError("unfinished substeps belong to an unfinished deliverable")
        supplementary_ids.add(item["id"])
    return len(completed), len(pending)


def read_counts(first_line):
    match = re.fullmatch(r"# DONE: (\d+) \| TODO: (\d+) \| TOTAL: (\d+)", first_line)
    if match is None:
        raise ValueError("DONE / TODO / TOTAL must occupy the first line")
    done, todo, total = map(int, match.groups())
    if total != FIXED_TOTAL or done + todo != FIXED_TOTAL or done < len(BASELINE_IDS):
        raise ValueError("DONE + TODO = TOTAL = 55; completed records cannot decrease")
    return done, todo


def validate_published_progress(ledger, publication_history):
    """Require retained publication checkpoints, not only self-consistent live history.

    Publication appends preserve old checkpoints. This catches ledger-only rollback;
    it is not tamperproof against deliberately rewriting both files or this validator.
    """
    counts = validate_ledger(ledger)
    if (not isinstance(publication_history, dict)
            or type(publication_history.get("schema")) is not int
            or publication_history.get("schema") != 1
            or publication_history.get("fixed_total") != FIXED_TOTAL):
        raise ValueError("a separate fixed-55 publication checkpoint archive is required")
    checkpoints = publication_history.get("checkpoints")
    expected = [{key: entry[key] for key in ("done", "todo", "completed")}
                for entry in ledger["history"]]
    if (not isinstance(checkpoints, list) or not checkpoints
            or any(not isinstance(entry, dict)
                   or type(entry.get("done")) is not int
                   or type(entry.get("todo")) is not int for entry in checkpoints)
            or checkpoints != expected):
        raise ValueError("publication checkpoints differ: retain published history and append before reporting")
    return counts


class ProgressCounterTests(unittest.TestCase):
    def actual(self):
        return json.loads(pathlib.Path(__file__).with_name("PROGRESS-CHECKLIST.json").read_text())

    def publications(self):
        return json.loads(pathlib.Path(__file__).with_name(
            "PROGRESS-PUBLICATION-HISTORY.json").read_text())

    def publication_fixture(self, ledger):
        return {"schema": 1, "fixed_total": FIXED_TOTAL, "checkpoints": [
            {key: copy.deepcopy(entry[key]) for key in ("done", "todo", "completed")}
            for entry in ledger["history"]]}

    def ledger(self):
        # Mutation tests use the frozen baseline even after real final deliverables close.
        fixture = self.actual()
        fixture["completed"] = list(fixture["history"][0]["completed"])
        fixture["pending"] = list(REQUIRED_CRITERIA)
        fixture["history"] = fixture["history"][:1]
        fixture["supplementary"] = []
        fixture["analysis"] = {
            "status": "unfinished", "branch": None,
            "conclusion_evidence": [], "independent_audit": None,
        }
        for identity, item in fixture["deliverables"].items():
            item.update(status="pending", evidence=[], verified_at=None)
            item["criteria"] = dict.fromkeys(REQUIRED_CRITERIA[identity], "pending")
        return fixture

    def complete(self, ledger, identity):
        ledger["pending"].remove(identity)
        ledger["completed"].append(identity)
        item = ledger["deliverables"][identity]
        item.update(status="verified", verified_at="test-only timestamp",
                    evidence=["test-only evidence"])
        item["criteria"] = dict.fromkeys(REQUIRED_CRITERIA[identity], "pass")
        ledger["analysis"]["branch"] = "verified_mechanism"
        ledger["history"].append({
            "done": len(ledger["completed"]), "todo": len(ledger["pending"]),
            "completed": list(ledger["completed"]),
        })

    def test_zero_todo_without_analysis_deliverables_is_rejected(self):
        legacy = json.loads(pathlib.Path(__file__).with_name(
            "PROGRESS-CHECKLIST-SUPERSEDED-49.json").read_text())
        self.assertEqual(legacy["pending"], [])
        self.assertNotIn("deliverables", legacy)
        with self.assertRaises(ValueError):
            validate_ledger(legacy)

    def test_actual_audited_ledger_and_evidence_sources(self):
        ledger = self.actual()
        validate_published_progress(ledger, self.publications())
        sources = [group["source"] for group in ledger["completed_record_sources"].values()]
        sources += [source for item in ledger["deliverables"].values()
                    for source in item["evidence"]]
        sources += ledger["analysis"]["conclusion_evidence"]
        audit = ledger["analysis"]["independent_audit"]
        if audit:
            sources.append(audit["evidence"])
        for source in sources:
            self.assertTrue((ROOT / source).is_file(), source)

    def test_header_and_live_note_match_task_ledger(self):
        counts = validate_published_progress(self.actual(), self.publications())
        header = (ROOT / "hypotesis.md").read_text().splitlines()[0]
        self.assertEqual(read_counts(header), counts)
        self.assertIn(header.removeprefix("# "),
                      (ROOT / "ephemeral-note.md").read_text().splitlines()[2])

    def test_first_pending_table_matches_actual_pending_ids(self):
        validate_published_progress(self.actual(), self.publications())
        opening = "\n".join((ROOT / "hypotesis.md").read_text().splitlines()[:35])
        ids = re.findall(r"^\| (G\d\d) \| TODO \|", opening, re.MULTILINE)
        self.assertEqual(ids, self.actual()["pending"])

    def test_prior_inflated_and_backward_headers_are_rejected(self):
        for line in ("# DONE: 50 | TODO: 1 | TOTAL: 51",
                     "# DONE: 49 | TODO: 0 | TOTAL: 49",
                     "# DONE: 51 | TODO: 4 | TOTAL: 55",
                     "# DONE: 53 | TODO: 3 | TOTAL: 55"):
            with self.subTest(line=line), self.assertRaises(ValueError):
                read_counts(line)

    def test_added_or_duplicate_task_is_rejected(self):
        for added in ("V16", "V15"):
            ledger = self.ledger()
            ledger["completed"].append(added)
            with self.subTest(added=added), self.assertRaises(ValueError):
                validate_ledger(ledger)

    def test_changed_total_or_backward_history_is_rejected(self):
        changed_total = self.ledger()
        changed_total["fixed_total"] += 1
        backward = self.ledger()
        backward["history"].append({"done": 51, "todo": 4,
                                   "completed": backward["completed"][:-1]})
        for ledger in (changed_total, backward):
            with self.assertRaises(ValueError):
                validate_ledger(ledger)

    def test_completed_baseline_cannot_be_erased_by_truncating_history(self):
        ledger = self.ledger()
        ledger["completed"].remove("V15")
        ledger["pending"].append("V15")
        ledger["history"] = [{"done": 51, "todo": 4,
                              "completed": list(ledger["completed"])}]
        with self.assertRaises(ValueError):
            validate_ledger(ledger)

    def test_followup_substeps_do_not_change_counts(self):
        ledger = self.ledger()
        counts = validate_ledger(ledger)
        ledger["supplementary"].append({
            "id": "test-only-followup", "parent": "G01", "status": "checking"})
        self.assertEqual(validate_ledger(ledger), counts)

    def test_required_deliverables_and_criteria_cannot_be_removed(self):
        missing_deliverable = self.ledger()
        del missing_deliverable["deliverables"]["G03"]
        missing_criterion = self.ledger()
        del missing_criterion["deliverables"]["G03"]["criteria"]["joint_residual_and_downstream_offsets"]
        for ledger in (missing_deliverable, missing_criterion):
            with self.assertRaises(ValueError):
                validate_ledger(ledger)

    def test_only_evidence_backed_final_deliverable_advances_progress(self):
        ledger = self.ledger()
        self.complete(ledger, "G01")
        self.assertEqual(validate_ledger(ledger), (53, 2))
        ledger["deliverables"]["G01"]["evidence"] = []
        with self.assertRaises(ValueError):
            validate_ledger(ledger)

    def test_reopening_completed_deliverable_is_rejected(self):
        ledger = self.ledger()
        self.complete(ledger, "G01")
        ledger["completed"].remove("G01")
        ledger["pending"].append("G01")
        ledger["history"].append({"done": 52, "todo": 3,
                                  "completed": list(ledger["completed"])})
        with self.assertRaises(ValueError):
            validate_ledger(ledger)

    def test_zero_todo_requires_final_independent_audit(self):
        ledger = self.ledger()
        for identity in REQUIRED_CRITERIA:
            self.complete(ledger, identity)
        with self.assertRaises(ValueError):
            validate_ledger(ledger)
        ledger["analysis"].update(
            status="complete", conclusion_evidence=["test-only report"],
            independent_audit={"status": "pass", "reviewer": "test-only reviewer",
                               "evidence": "test-only independent report"})
        self.assertEqual(validate_ledger(ledger), (55, 0))
        for field in ("conclusion_evidence", "independent_audit"):
            invalid = copy.deepcopy(ledger)
            invalid["analysis"][field] = None
            with self.subTest(field=field), self.assertRaises(ValueError):
                validate_ledger(invalid)

    def test_unfinished_substeps_cannot_hide_under_completed_records(self):
        ledger = self.ledger()
        ledger["supplementary"].append({
            "id": "test-only-followup", "parent": "V13", "status": "checking"})
        with self.assertRaises(ValueError):
            validate_ledger(ledger)

    def test_published_completion_survives_ledger_history_truncation(self):
        baseline = self.ledger()
        advanced = copy.deepcopy(baseline)
        self.complete(advanced, "G01")
        publications = self.publication_fixture(advanced)
        self.assertEqual(validate_published_progress(advanced, publications), (53, 2))
        # Restoring the earlier ledger also truncates history and clears G01 evidence.
        # Internal ledger validation alone accepts this backward publication.
        self.assertEqual(validate_ledger(baseline), (52, 3))
        with self.assertRaisesRegex(ValueError, "publication checkpoint"):
            validate_published_progress(baseline, publications)

    def test_progress_requires_corresponding_publication_append(self):
        ledger = self.ledger()
        publications = self.publication_fixture(ledger)
        self.complete(ledger, "G01")
        with self.assertRaisesRegex(ValueError, "publication checkpoint"):
            validate_published_progress(ledger, publications)
        publications["checkpoints"].append(self.publication_fixture(ledger)["checkpoints"][-1])
        self.assertEqual(validate_published_progress(ledger, publications), (53, 2))

    def test_missing_or_mismatched_publication_checkpoints_are_rejected(self):
        ledger = self.ledger()
        correct = self.publication_fixture(ledger)
        invalid = [None, {}, dict(correct, schema=2), dict(correct, fixed_total=56),
                   dict(correct, checkpoints=[]), dict(correct, checkpoints="invalid")]
        mismatched = copy.deepcopy(correct)
        mismatched["checkpoints"][0]["completed"][0] = "G01"
        invalid.append(mismatched)
        missing_field = copy.deepcopy(correct)
        del missing_field["checkpoints"][0]["todo"]
        invalid.append(missing_field)
        for publications in invalid:
            with self.subTest(publications=publications), self.assertRaises(ValueError):
                validate_published_progress(ledger, publications)

    def test_unpublished_substeps_do_not_need_new_count_checkpoints(self):
        ledger = self.ledger()
        publications = self.publication_fixture(ledger)
        ledger["supplementary"].append({
            "id": "test-only-substep", "parent": "G01", "status": "checking"})
        self.assertEqual(validate_published_progress(ledger, publications), (52, 3))


if __name__ == "__main__":
    unittest.main()
