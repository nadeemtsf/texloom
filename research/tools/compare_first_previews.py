"""Compare synthetic first-pass observer outputs; requires PyMuPDF, no app UI."""
from pathlib import Path
import hashlib
import json
import sys
import fitz

root = Path(sys.argv[1]).resolve()
rows = json.loads((root / "report.json").read_text(encoding="utf-8"))
expected = {(engine, case) for engine in ["pdflatex", "xelatex", "lualatex"]
            for case in ["plain", "references", "contents", "late-failure", "compile-error",
                         "cancel-at-preview", "mutate-after-preview"]}
assert len(rows) == len(expected) and {(r["engine"], r["case"]) for r in rows} == expected


def inspect(path):
    pages = []
    with fitz.open(path) as document:
        for page in document:
            pixels = page.get_pixmap(matrix=fitz.Matrix(96 / 72, 96 / 72), alpha=False)
            geometry = json.dumps(page.get_text("rawdict"), sort_keys=True, ensure_ascii=True).encode()
            pages.append({"text": page.get_text(), "width": pixels.width, "height": pixels.height,
                          "geometry_sha256": hashlib.sha256(geometry).hexdigest(),
                          "pixels_sha256": hashlib.sha256(pixels.samples).hexdigest()})
    return pages


comparisons = []
for row in rows:
    engine, case = row["engine"], row["case"]
    assert len(row["events"]) == (0 if case == "compile-error" else 1)
    directory = root / engine / case
    if case in ["plain", "references", "contents"]:
        first, final, baseline = [inspect(directory / name) for name in
                                  ["first-pass.pdf", "final.pdf", "baseline.pdf"]]
        assert final == baseline, (engine, case, "Observer changed final text, geometry or pixels")
        assert (first == final) == (case == "plain"), (engine, case, "Wrong first-pass expectation")
        assert row["ok"] and row["eligible"] and row["passes"] >= 2 and row["version_reused"]
        available = row["events"][0]["seconds"]
        assert 0 < available < row["seconds"]
        comparisons.append({"engine": engine, "case": case, "final_equals_baseline": True,
                            "first_equals_final": first == final, "first": first, "final": final,
                            "first_available_seconds": available, "final_return_seconds": row["seconds"]})
    elif case == "late-failure":
        assert not row["ok"] and not row["eligible"] and row["passes"] == 2
        assert inspect(directory / "first-pass.pdf")
    elif case == "compile-error":
        assert not row["ok"] and not row["eligible"] and row["passes"] == 1
    else:
        assert "error" in row and inspect(directory / "first-pass.pdf")

result = {"schema": 1, "pymupdf": fitz.VersionBind, "native_controls": len(rows),
          "final_baseline_comparisons": len(comparisons), "comparisons": comparisons,
          "scope": "Full text, character geometry and all-page 96dpi raster checks on synthetic artifacts"}
output = root / "output-comparison.json"
if output.exists():
    raise RuntimeError("Keep earlier output comparisons; choose a fresh run directory")
output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
print(f"{len(rows)} controls; {len(comparisons)} final/baseline PDF comparisons passed")
