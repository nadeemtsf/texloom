"""Compare adjacent paragraph drafts with independent sources and rendered order."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import subprocess


ENGINES = ("pdflatex", "xelatex", "lualatex")
FIXTURES = (
    ("later-before-earlier", ["Amber origin.", "Blue target.", "Copper tail."],
     ["Blue target.", "Amber origin.", "Copper tail."], ["1", "move-before", "0"]),
    ("earlier-after-neighbor", ["Short first.", "Much longer neighbor sentence.", "Last anchor."],
     ["Much longer neighbor sentence.", "Short first.", "Last anchor."], ["0", "move-after", "1"]),
    ("second-repeated-occurrence", ["Repeat body.", "Marker between.", "Repeat body.", "Final anchor."],
     ["Repeat body.", "Repeat body.", "Marker between.", "Final anchor."], ["2", "move-before", "1"]),
)


def source(paragraphs):
    body = "\r\n\r\n".join(paragraphs)
    return ("\\documentclass{article}\r\n\\pagestyle{empty}\r\n"
            "\\begin{document}\r\n" + body + "\r\n\\end{document}\r\n").encode()


def checked(args, directory, log):
    with (directory / log).open("wb") as stream:
        subprocess.run(args, cwd=directory, stdout=stream, stderr=subprocess.STDOUT, check=True)


def compile_pdf(engine, directory, text):
    directory.mkdir(parents=True)
    (directory / "main.tex").write_bytes(text)
    checked([engine, "--disable-installer", "-no-shell-escape", "-halt-on-error",
             "-interaction=nonstopmode", "main.tex"], directory, "compile-output.log")
    checked(["pdftotext", "-nopgbrk", "main.pdf", "order.txt"], directory, "text.log")
    checked(["pdftoppm", "-r", "96", "-png", "-singlefile", "main.pdf", "page"], directory, "raster.log")
    reading_order = " ".join((directory / "order.txt").read_text(encoding="utf-8").split())
    return reading_order, sha256((directory / "page.png").read_bytes()).hexdigest()


def compare(executable, root):
    results = []
    for fixture, before, expected, command in FIXTURES:
        directory = root / fixture
        directory.mkdir()
        baseline = source(before)
        baseline_path = directory / "baseline.tex"
        baseline_path.write_bytes(baseline)
        candidate = subprocess.check_output([str(executable), str(baseline_path), *command])
        reference = source(expected)
        assert baseline_path.read_bytes() == baseline, fixture
        assert candidate == reference, fixture
        for engine in ENGINES:
            original_order, original_pixels = compile_pdf(engine, directory / engine / "baseline", baseline)
            candidate_order, candidate_pixels = compile_pdf(engine, directory / engine / "candidate", candidate)
            reference_order, reference_pixels = compile_pdf(engine, directory / engine / "reference", reference)
            assert original_order == " ".join(before), (fixture, engine, original_order)
            assert candidate_order == reference_order == " ".join(expected), (fixture, engine, candidate_order)
            assert candidate_pixels == reference_pixels and candidate_pixels != original_pixels, (fixture, engine)
            results.append({
                "fixture": fixture, "engine": engine,
                "baseline_source_sha256": sha256(baseline).hexdigest(),
                "candidate_source_sha256": sha256(candidate).hexdigest(),
                "baseline_reading_order": original_order,
                "candidate_reading_order": candidate_order,
                "reference_reading_order": reference_order,
                "baseline_page_png_sha256": original_pixels,
                "candidate_page_png_sha256": candidate_pixels,
                "reference_page_png_sha256": reference_pixels,
            })
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate_executable", type=Path)
    parser.add_argument("output_directory", type=Path)
    args = parser.parse_args()
    executable = args.candidate_executable.resolve(strict=True)
    root = args.output_directory.resolve()
    root.mkdir()
    versions = {
        engine: subprocess.check_output([engine, "--version"], encoding="utf-8", errors="replace").splitlines()[0]
        for engine in ENGINES
    }
    results = compare(executable, root)
    evidence = {"schema": 1, "candidate_executable_sha256": sha256(executable.read_bytes()).hexdigest(),
                "engines": versions, "comparisons": results}
    (root / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(results)} fixture-engine comparisons passed")


if __name__ == "__main__":
    main()
