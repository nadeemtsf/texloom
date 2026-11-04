import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("source_contract", Path(__file__).parents[1]/"tools/source_contract.py")
ORACLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORACLE)


class SourceContractTests(unittest.TestCase):
    def setUp(self):
        self.before = "\ufeffRepeated\r\nمرحبا שלום\r\nRepeated\r\n"
        self.expected = "\ufeffRepeated\r\nمرحبا שלום\r\n\\TexloomObject{{{NEW_OBJECT_ID}}}{12}{0}{Repeated}\r\n"
        self.baseline = {"main.tex": self.before.encode("utf-8"), "notes.sty": b"% unchanged\r\n"}
        self.contract = {"schema": 1, "files": {
            "main.tex": {"before_sha256": ORACLE.sha(self.baseline["main.tex"]), "after_utf8_template": self.expected},
            "notes.sty": {"before_sha256": ORACLE.sha(self.baseline["notes.sty"]), "after_sha256": ORACLE.sha(self.baseline["notes.sty"])}}}
        self.candidate = {**self.baseline, "main.tex": self.expected.replace(ORACLE.TOKEN, "text-1234abcd").encode("utf-8")}

    def test_second_occurrence_and_every_unrelated_byte_are_bound(self):
        self.assertEqual(ORACLE.verify_sources(self.baseline, self.candidate, self.contract)["decision"], "matches_contract")
        wrong = "\ufeff\\TexloomObject{text-1234abcd}{12}{0}{Repeated}\r\nمرحبا שלום\r\nRepeated\r\n".encode("utf-8")
        self.assertEqual(ORACLE.verify_sources(self.baseline, {**self.candidate, "main.tex": wrong}, self.contract)["decision"], "differs")

    def test_line_ending_and_support_file_changes_are_detected(self):
        for modified in [{**self.candidate, "main.tex": self.candidate["main.tex"].replace(b"\r\n", b"\n")},
                         {**self.candidate, "notes.sty": b"% changed\r\n"},
                         {**self.candidate, "unexpected.tex": b"extra"}]:
            self.assertEqual(ORACLE.verify_sources(self.baseline, modified, self.contract)["decision"], "differs")

    def test_stale_baseline_is_unresolved_and_cannot_be_a_passing_refusal(self):
        stale = {**self.baseline, "main.tex": b"other revision"}
        result = ORACLE.verify_sources(stale, self.candidate, self.contract)
        self.assertEqual(result["decision"], "unresolved")
        self.assertFalse(result["source_contract_checked"])

    def test_object_id_slot_does_not_allow_injected_source(self):
        modified = {**self.candidate, "main.tex": self.expected.replace(ORACLE.TOKEN, "text-1234abcd}\\input{other}").encode("utf-8")}
        self.assertEqual(ORACLE.verify_sources(self.baseline, modified, self.contract)["decision"], "differs")

    def test_a_fresh_id_slot_cannot_reuse_an_existing_baseline_identifier(self):
        notes = b"% existing text-1234abcd\r\n"
        self.baseline["notes.sty"] = notes;self.candidate["notes.sty"] = notes
        self.contract["files"]["notes.sty"] = {"before_sha256": ORACLE.sha(notes), "after_sha256": ORACLE.sha(notes)}
        self.assertEqual(ORACLE.verify_sources(self.baseline, self.candidate, self.contract)["decision"], "differs")


if __name__ == "__main__":
    unittest.main()
