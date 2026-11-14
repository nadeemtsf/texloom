"""Offline reference comparison, independent of Texloom's planner and PDFium.

This compares PDFs with a separately authored reference. It does not authorize
app edits or establish that the reference expresses the intended source edit.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

from PIL import Image, ImageChops
from pypdf import PdfReader
from pypdf.errors import PdfReadError


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def executable(value, default):
    found = shutil.which(value or default)
    if not found:
        raise ValueError(f"Required research tool unavailable: {value or default}")
    return Path(found).resolve()


def tool_receipt(path):
    result = subprocess.run([str(path), "-v"], capture_output=True, timeout=10)
    version = (result.stdout + result.stderr).decode("utf-8", errors="replace")
    return {"path": str(path), "sha256": digest(path), "version": version[:2000]}


def page_contract(path):
    if path.suffix.lower() != ".pdf" or path.stat().st_size > 50_000_000:
        raise ValueError("Expected a PDF no larger than 50 MB")
    reader = PdfReader(path, strict=True)
    if reader.is_encrypted or not 1 <= len(reader.pages) <= 8:
        raise ValueError("Encrypted PDF or page budget requires adjudication")
    pages = []
    for page in reader.pages:
        media = tuple(float(x) for x in page.mediabox)
        crop = tuple(float(x) for x in page.cropbox)
        if (not all(math.isfinite(x) for x in media + crop)
                or media != crop or media[:2] != (0, 0)
                or page.rotation != 0 or float(page.get("/UserUnit", 1)) != 1
                or page.get("/Annots")):
            raise ValueError("Unsupported page coordinates/annotations require adjudication")
        if min(media[2:]) <= 0 or media[2] * media[3] * 4 > 8_000_000:
            raise ValueError("Invalid dimensions or raster budget exceeded")
        pages.append(media)
    return pages


def words_from_xml(path):
    if path.stat().st_size > 16_000_000:
        raise ValueError("Text evidence exceeds byte budget")
    tree = ET.parse(path)
    pages = [e for e in tree.iter() if e.tag.rsplit("}", 1)[-1] == "page"]
    words = []
    for number, page in enumerate(pages, 1):
        for element in page.iter():
            if element.tag.rsplit("}", 1)[-1] != "word":
                continue
            bounds = [float(element.attrib[k]) for k in ("xMin", "yMin", "xMax", "yMax")]
            if not all(math.isfinite(x) for x in bounds):
                raise ValueError("Nonfinite word geometry")
            words.append({"page": number, "text": element.text or "", "bounds": bounds})
    if len(words) > 100_000:
        raise ValueError("Word budget exceeded")
    return sorted(words, key=lambda w: (w["page"], w["bounds"][1], w["bounds"][0]))


def compare_words(expected, candidate, tolerance=0.02):
    if not math.isfinite(tolerance) or tolerance < 0:
        raise ValueError("Invalid geometry tolerance")
    left = Counter((w["page"], w["text"]) for w in expected)
    right = Counter((w["page"], w["text"]) for w in candidate)
    text_equal = left == right
    # This bounded pilot uses isolated lines with stable reading order. A future
    # ambiguous/reordered layout needs separate annotations, not guessed pairing.
    aligned = (len(expected) == len(candidate) and all(
        (a["page"], a["text"]) == (b["page"], b["text"])
        for a, b in zip(expected, candidate)))
    error = max((abs(x-y) for a, b in zip(expected, candidate)
                 for x, y in zip(a["bounds"], b["bounds"])), default=0) if aligned else None
    return {"text_multiplicities_equal": text_equal, "reading_order_pairing": aligned,
            "maximum_word_bounds_error_bp": error,
            "word_geometry_matches": aligned and error <= tolerance,
            "expected_word_count": len(expected), "candidate_word_count": len(candidate)}


def compare_raster(expected, candidate):
    with Image.open(expected) as left, Image.open(candidate) as right:
        if left.size != right.size:
            return {"exact_pixels": False, "reason": "different raster dimensions"}
        difference = ImageChops.difference(left.convert("RGB"), right.convert("RGB"))
        changed = 0 if difference.getbbox() is None else sum(
            any(pixel) for pixel in difference.getdata())
        return {"exact_pixels": changed == 0, "changed_pixels": changed,
                "total_pixels": left.width * left.height,
                "maximum_channel_difference": max(hi for _, hi in difference.getextrema())}


def inspect(path, output, tools, page_count):
    output.mkdir()
    commands = [
        [str(tools["text"]), "-bbox-layout", "-enc", "UTF-8", str(path), str(output/"text.xml")],
        [str(tools["raster"]), "-f", "1", "-l", str(page_count), "-r", "144", "-png", str(path), str(output/"page")],
    ]
    for index, command in enumerate(commands):
        with (output/f"tool-{index}.log").open("wb") as log:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=45)
        if result.returncode:
            raise ValueError(f"Independent PDF tool failed; see {output.name}/tool-{index}.log")
    rasters = sorted(output.glob("page-*.png"), key=lambda p: int(p.stem.rsplit("-", 1)[-1]))
    if len(rasters) != page_count:
        raise ValueError("Renderer did not produce every page")
    return words_from_xml(output/"text.xml"), rasters


def compare_pdfs(expected, candidate, output, tools, tolerance=0.02):
    expected, candidate, output = Path(expected).resolve(), Path(candidate).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema": 1, "kind": "offline_reference_comparison_not_app_acceptance",
              "decision": "unresolved", "expected_pdf_sha256": None,
              "candidate_pdf_sha256": None, "geometry_tolerance_bp": tolerance,
              "raster_contract": "exact full-page opaque RGB pixels at 144 dpi",
              "source_intent_oracle_checked": False}
    try:
        left_pages, right_pages = page_contract(expected), page_contract(candidate)
        report.update(expected_pdf_sha256=digest(expected), candidate_pdf_sha256=digest(candidate))
        report["page_contract_equal"] = left_pages == right_pages
        if left_pages != right_pages:
            report.update(decision="differs", reason="Page count or dimensions differ")
        else:
            left_words, left_rasters = inspect(expected, output/"expected", tools, len(left_pages))
            right_words, right_rasters = inspect(candidate, output/"candidate", tools, len(right_pages))
            report.update(compare_words(left_words, right_words, tolerance))
            report["rasters"] = [compare_raster(a, b) for a, b in zip(left_rasters, right_rasters)]
            matches = (report["text_multiplicities_equal"] and report["word_geometry_matches"]
                       and all(x["exact_pixels"] for x in report["rasters"]))
            report["decision"] = "matches_reference" if matches else "differs"
        if digest(expected) != report["expected_pdf_sha256"] or digest(candidate) != report["candidate_pdf_sha256"]:
            raise ValueError("PDF input changed during independent comparison")
    except (OSError, ValueError, PdfReadError, subprocess.SubprocessError, ET.ParseError) as error:
        report["decision"] = "unresolved"
        report["reason"] = f"{type(error).__name__}: {error}"
    (output/"comparison.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("expected", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("output", type=Path, help="New directory; earlier evidence is never overwritten")
    parser.add_argument("--pdftotext")
    parser.add_argument("--pdftoppm")
    args = parser.parse_args()
    tools = {"text": executable(args.pdftotext, "pdftotext"),
             "raster": executable(args.pdftoppm, "pdftoppm")}
    report = compare_pdfs(args.expected, args.candidate, args.output, tools)
    (args.output/"tools.json").write_text(json.dumps(
        {key: tool_receipt(value) for key, value in tools.items()}, indent=2)+"\n", encoding="utf-8")
    print(json.dumps(report))
    return 0 if report["decision"] == "matches_reference" else 1


if __name__ == "__main__":
    raise SystemExit(main())
