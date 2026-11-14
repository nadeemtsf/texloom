import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("summary", Path(__file__).parents[1]/"tools/summarize_annotations.py")
SUMMARY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SUMMARY)


def row(case, truth="valid", decision="refused", adjudication=None):
    return {"case_id": case, "document_id": "synthetic-document", "study_arm": "natural_request",
            "contract_family": "displacement", "variant": "unit-test-variant", "engine_version": "unit-test-engine",
            "language": "unit-test", "evidence_origin": "synthetic_unit_test_not_an_experiment",
            "oracle_truth": truth, "system_decision": decision, "declared_in_scope": True,
            "adjudicated_correct_if_accepted": adjudication,
            "source_outside_patch_exact": True if decision == "accepted" else None,
            "unrelated_rendered_content_preserved": True if decision == "accepted" else None}


class AnnotationSummaryTests(unittest.TestCase):
    def test_deny_all_has_no_accepted_error_fraction_and_no_useful_acceptance(self):
        result = SUMMARY.summarize([row("valid"), row("unsafe", "unsafe")])["groups"][0]
        self.assertIsNone(result["metrics"]["incorrect_acceptance_fraction"]["fraction"])
        self.assertEqual(result["metrics"]["correct_valid_acceptance"]["fraction"], 0)
        self.assertEqual(result["metrics"]["valid_refusal"]["fraction"], 1)
        self.assertEqual(SUMMARY.summarize([])["status"], "empty_no_results")

    def test_unexecuted_cases_produce_no_success_rates(self):
        result = SUMMARY.summarize([row("pending", decision="not_run")])["groups"][0]
        self.assertIsNone(result["metrics"]);self.assertEqual(result["safety_gate"], "incomplete")

    def test_pending_adjudication_produces_error_bounds_and_an_unresolved_gate(self):
        result = SUMMARY.summarize([row("unknown", decision="accepted")])["groups"][0]
        rate = result["metrics"]["incorrect_acceptance_fraction"]
        self.assertIsNone(rate["fraction"]);self.assertEqual((rate["lower_bound"], rate["upper_bound"]), (0, 1))
        self.assertEqual(result["safety_gate"], "unresolved")

    def test_compile_failures_are_not_valid_refusals_and_faults_fail_the_gate(self):
        data = [row("good", decision="accepted", adjudication=True), row("refusal"),
                row("build-failure", decision="compile_failure"), row("bad", truth="unsafe", decision="accepted", adjudication=False)]
        result = SUMMARY.summarize(data)["groups"][0]
        self.assertEqual(result["metrics"]["correct_valid_acceptance"]["fraction"], 1/3)
        self.assertEqual(result["metrics"]["valid_refusal"]["fraction"], 1/3)
        self.assertEqual(result["safety_gate"], "fail")

    def test_comparison_arms_are_separate_and_duplicate_trials_are_rejected(self):
        a = row("same");b = copy.deepcopy(a);b["study_arm"] = "injected_candidate_diagnostic"
        self.assertEqual(len(SUMMARY.summarize([a, b])["groups"]), 2)
        with self.assertRaises(ValueError): SUMMARY.summarize([a, a])


if __name__ == "__main__":
    unittest.main()
