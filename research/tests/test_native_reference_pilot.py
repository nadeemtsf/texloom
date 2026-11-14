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

    def test_reference_health_detects_the_observed_extra_neighbor_line(self):
        words=[{"page":1,"text":text,"bounds":[x,y,x+8,y+5]} for text,x,y in
               [("Repeated",10,10),("target",20,10),("Repeated",10,30),("target",20,30),("Neighbor",10,50)]]
        import copy
        moved=copy.deepcopy(words)
        for index in (2,3):
            moved[index]["bounds"][0]+=12;moved[index]["bounds"][2]+=12
        self.assertEqual(PILOT.reference_word_health(words,moved)["decision"],"matches_declared_translation")
        moved[-1]["bounds"][1]+=11.955;moved[-1]["bounds"][3]+=11.955
        result=PILOT.reference_word_health(words,moved)
        self.assertEqual(result["decision"],"differs")
        self.assertAlmostEqual(result["maximum_non_target_error_bp"],11.955)
        self.assertEqual(result["maximum_target_error_bp"],0)

    def test_reference_health_will_not_pair_a_missing_or_ambiguous_target(self):
        words=[{"page":1,"text":"Repeated","bounds":[10,10,20,20]}]
        self.assertEqual(PILOT.reference_word_health(words,words)["decision"],"unresolved")
        self.assertEqual(PILOT.reference_word_health(words,[])["decision"],"unresolved")


if __name__=="__main__":unittest.main()
