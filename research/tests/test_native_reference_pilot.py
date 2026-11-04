from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).parents[1]/"tools"))
import native_reference_pilot as PILOT


class NativeReferencePilotTests(unittest.TestCase):
    def test_the_source_contract_distinguishes_every_declared_tamper(self):
        case=PILOT.condition("xelatex","multilingual","% synthetic style\r\n")
        baseline={name:text.encode("utf-8") for name,text in case["baseline"].items()}
        for candidate in case["candidates"]:
            actual={name:text.encode("utf-8") for name,text in candidate["sources"].items()}
            outcome=PILOT.verify_sources(baseline,actual,case["source_contract"])
            self.assertEqual(outcome["decision"],"matches_contract" if candidate["variant"]=="correct" else "differs")
        self.assertTrue(case["baseline"]["main.tex"].startswith("\ufeff"))
        self.assertIn("\r\n",case["baseline"]["main.tex"])
        self.assertEqual(case["request"]["id"],"native-target")

    def test_gold_uses_plain_tex_placement_and_keeps_the_request_on_the_second_occurrence(self):
        case=PILOT.condition("pdflatex","latin","% style\n")
        for name in ("gold_zero","gold_moved"):
            self.assertNotIn("\\TexloomObject",case[name]["main.tex"])
        self.assertIn(r"\kern12bp",case["gold_moved"]["main.tex"])
        wrong=next(x for x in case["candidates"] if x["variant"]=="wrong_first_occurrence")
        self.assertIn(PILOT.TARGET,wrong["sources"]["main.tex"])
        self.assertEqual(case["candidates"][-1]["variant"],"correct")

    def test_preparation_is_exclusive_and_records_criteria_before_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/"pilot";protocol=PILOT.prepare(root)
            self.assertEqual(len(protocol["conditions"]),5)
            self.assertFalse(protocol["final_evaluation_executed"])
            self.assertFalse((root/"native-output").exists())
            before=(root/"protocol.json").read_bytes()
            with self.assertRaises(FileExistsError):PILOT.prepare(root)
            self.assertEqual((root/"protocol.json").read_bytes(),before)


if __name__=="__main__":unittest.main()
