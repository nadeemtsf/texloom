import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("environment_manifest", Path(__file__).parents[1]/"tools/environment_manifest.py")
MANIFEST = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MANIFEST)


class EnvironmentManifestTests(unittest.TestCase):
    def test_bytes_are_bound_without_publishing_the_parent_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"synthetic-font.ttf"
            path.write_bytes(b"fixture bytes, not a real font")
            receipt = MANIFEST.file_receipt(path)
            self.assertEqual(receipt["filename"], path.name)
            self.assertNotIn(directory, str(receipt))
            path.write_bytes(b"changed font bytes")
            self.assertNotEqual(MANIFEST.file_receipt(path)["sha256"], receipt["sha256"])

    def test_missing_or_changing_tools_are_not_recorded_as_resolved(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"fixture.exe"
            self.assertEqual(MANIFEST.tool_receipt("pdflatex", path)["status"], "unresolved")
            path.write_bytes(b"first version")
            def changing(command, **kwargs):
                path.write_bytes(b"changed executable")
                return subprocess.CompletedProcess(command, 0, b"Version 1", b"")
            result = MANIFEST.tool_receipt("pdflatex", path, runner=changing)
            self.assertEqual(result["status"], "unresolved")
            self.assertIn("after_probe", result)
            with self.assertRaises(ValueError):
                MANIFEST.tool_receipt("arbitrary-project-command", path)

    def test_existing_evidence_is_not_overwritten_and_final_freeze_is_not_inferred(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"manifest.json"
            result = MANIFEST.capture([], [], [], "development-posthoc")
            MANIFEST.write_manifest(path, result)
            before = path.read_bytes()
            with self.assertRaises(FileExistsError):
                MANIFEST.write_manifest(path, {"different": "data"})
            self.assertEqual(path.read_bytes(), before)
            self.assertFalse(result["final_evaluation_executed"])
            self.assertIsNone(result["machine"]["display_configuration"])
            with self.assertRaises(ValueError):
                MANIFEST.capture([], [], [], "final")


if __name__ == "__main__":
    unittest.main()
