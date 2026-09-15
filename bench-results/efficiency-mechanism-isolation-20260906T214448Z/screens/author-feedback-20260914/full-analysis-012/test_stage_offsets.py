import copy
import unittest

import stage_offsets as s


def ledger(name, author=10, review=20, fix=30, integration=40):
    values = dict(initial_author=author, review=review, author_fix=fix, integration=integration)
    records = {name+"-"+c:{"usage":{"input_tokens":v,"output_tokens":0}} for c,v in values.items()}
    return {"run_id":name,"status":"qualified_observed_ledger","errors":[],
        "workflow_result":"pass","native_usage":{"raw_input_plus_output":sum(values.values())},
        "capture_qualification":{"unknown_unfinished_tail_observed":False},
        "records":records,
        "turns":[{"category":c,"native_response_ids":[name+"-"+c]} for c in values],
        "category_totals":{c:{"native_usage":{"raw_input_plus_output":v}} for c,v in values.items()}}


class OffsetTests(unittest.TestCase):
    def test_later_offsets_are_subtracted_and_no_cost_is_counted_twice(self):
        reference = ledger("reference", author=100, review=100, fix=100, integration=100)
        runs = [ledger(n, author=200, review=20, fix=100, integration=100) for n in ("one","two","three")]
        result = s.summarize(reference, runs)
        self.assertEqual(result["net_delta_raw"], {"numerator":20,"denominator":1})
        self.assertEqual(result["stage_delta_raw"]["initial_author"], {"numerator":100,"denominator":1})
        self.assertEqual(result["stage_delta_raw"]["review"], {"numerator":-80,"denominator":1})
        self.assertIsNone(result["historical_explained_share"])

    def test_partial_or_failed_run_is_retained_not_removed_from_mean(self):
        runs = [ledger(n) for n in ("one","two","three")]
        runs[1]["workflow_result"]="fail"
        runs[1]["capture_qualification"]["unknown_unfinished_tail_observed"]=True
        result = s.summarize(ledger("reference"),runs)
        self.assertEqual(len(result["outcomes"]),3)
        self.assertFalse(result["complete_workflow_contrast"])
        self.assertIsNone(result["net_delta_raw"])
        self.assertEqual(result["recorded_attempt_mean_raw"],{"numerator":100,"denominator":1})

    def test_duplicate_cross_run_or_unassigned_response_blocks_partition(self):
        for corruption in ("duplicate","unassigned","false_total"):
            runs=[ledger(n) for n in ("one","two","three")]
            if corruption=="duplicate":
                runs[1]=copy.deepcopy(runs[0]);runs[1]["run_id"]="two"
            elif corruption=="unassigned":
                runs[0]["turns"].pop()
            else:
                runs[0]["category_totals"]["review"]["native_usage"]["raw_input_plus_output"]+=1
            with self.subTest(corruption=corruption),self.assertRaises(ValueError):
                s.summarize(ledger("reference"),runs)

    def test_missing_row_never_becomes_success_only_estimator(self):
        with self.assertRaises(ValueError):
            s.summarize(ledger("reference"),[ledger("one"),ledger("two")])


if __name__ == "__main__":
    unittest.main()
