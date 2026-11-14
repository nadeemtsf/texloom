"""Research-oracle diagnostics; these are not application correctness trials."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

from PIL import Image

SPEC = importlib.util.spec_from_file_location("reference_oracle", Path(__file__).parents[1]/"tools/reference_oracle.py")
ORACLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORACLE)


class ReferenceOracleTests(unittest.TestCase):
    def test_same_words_at_wrong_repeated_occurrence_are_geometrically_different(self):
        expected = [{"page": 1, "text": "Repeated", "bounds": [0, 10, 20, 20]},
                    {"page": 1, "text": "Repeated", "bounds": [12, 40, 32, 50]}]
        wrong = [{"page": 1, "text": "Repeated", "bounds": [12, 10, 32, 20]},
                 {"page": 1, "text": "Repeated", "bounds": [0, 40, 20, 50]}]
        report = ORACLE.compare_words(expected, wrong)
        self.assertTrue(report["text_multiplicities_equal"])
        self.assertFalse(report["word_geometry_matches"])

    def test_duplicate_text_is_not_lost_by_set_comparison(self):
        word = {"page": 1, "text": "Same", "bounds": [0, 0, 10, 10]}
        report = ORACLE.compare_words([word], [word, word])
        self.assertFalse(report["text_multiplicities_equal"])

    def test_a_single_changed_channel_outside_the_target_is_counted(self):
        with tempfile.TemporaryDirectory() as tmp:
            a, b = Path(tmp)/"a.png", Path(tmp)/"b.png"
            image = Image.new("RGB", (20, 20), "white")
            image.save(a);image.putpixel((19, 19), (255, 255, 254));image.save(b)
            result = ORACLE.compare_raster(a, b)
            self.assertFalse(result["exact_pixels"])
            self.assertEqual(result["changed_pixels"], 1)
            self.assertEqual(result["maximum_channel_difference"], 1)

    def test_missing_or_malformed_pdf_is_unresolved_not_a_successful_refusal(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            malformed = root/"malformed.pdf";malformed.write_bytes(b"%PDF-1.7\nnot a PDF")
            for index, source in enumerate([root/"absent.pdf", malformed]):
                result = ORACLE.compare_pdfs(source, malformed, root/str(index), {})
                self.assertEqual(result["decision"], "unresolved")
                self.assertTrue((root/str(index)/"comparison.json").exists())

    def test_prior_evidence_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with self.assertRaises(FileExistsError):
                ORACLE.compare_pdfs(root/"a.pdf", root/"b.pdf", root, {})


if __name__ == "__main__":
    unittest.main()
