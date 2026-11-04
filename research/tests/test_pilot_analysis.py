import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("analysis", Path(__file__).parents[1]/"tools/analyze_oracle_pilot.py")
ANALYSIS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ANALYSIS)


class PilotAnalysisTests(unittest.TestCase):
    def setUp(self):
        self.protocol = {"conditions": [{"engine": "synthetic", "multilingual_neighbors": False}],
                         "expected_control_decisions": {"correct": "matches_reference", "color": "differs"}}
        self.report = {"kind": "development_oracle_feasibility_not_final_evaluation", "status": "pass", "conditions": [
            {"engine": "synthetic", "multilingual_neighbors": False, "reference": {"ok": True}, "controls": [
                {"variant": name, "expected": expected, "actual": expected, "build": {"ok": True},
                 "comparison": {"page_contract_equal": True, "text_multiplicities_equal": True, "word_geometry_matches": True}}
                for name, expected in self.protocol["expected_control_decisions"].items()]}]}

    def test_geometry_and_compile_diagnostics_admit_color_fault_but_full_reference_does_not(self):
        variants = ANALYSIS.analyze(self.report, self.protocol)["variants"]
        self.assertEqual(variants["compile_only"]["known_fault_controls_admitted"], 1)
        self.assertEqual(variants["text_and_geometry"]["known_fault_controls_admitted"], 1)
        self.assertEqual(variants["full_reference"]["known_fault_controls_admitted"], 0)

    def test_finished_flag_does_not_make_a_missing_condition_or_control_complete(self):
        for altered in [[], [copy.deepcopy(self.report["conditions"][0])]]:
            report = copy.deepcopy(self.report);report["conditions"] = altered
            if altered: report["conditions"][0]["controls"].pop()
            with self.assertRaises(ValueError): ANALYSIS.analyze(report, self.protocol)

    def test_missing_geometry_evidence_is_unresolved_not_a_passing_refusal(self):
        self.report["conditions"][0]["controls"][0]["comparison"] = {}
        variants = ANALYSIS.analyze(self.report, self.protocol)["variants"]
        self.assertEqual(variants["text_and_geometry"]["unresolved"], 1)
        self.assertEqual(variants["text_and_geometry"]["correct_controls_not_admitted"], 0)

    def test_an_unexecuted_control_is_unresolved_in_every_comparison(self):
        self.report["conditions"][0]["controls"][0]["actual"] = "not_run"
        variants = ANALYSIS.analyze(self.report, self.protocol)["variants"]
        for totals in variants.values():
            self.assertEqual(totals["unresolved"], 1)
            self.assertEqual(totals["correct_controls_not_admitted"], 0)


if __name__ == "__main__":
    unittest.main()
