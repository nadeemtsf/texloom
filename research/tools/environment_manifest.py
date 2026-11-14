"""Development environment snapshot. No compilation, app launch or final evaluation."""
import argparse
from datetime import datetime, timezone
import hashlib
from importlib import metadata
import locale
import os
from pathlib import Path
import platform
import subprocess

import json

TOOL_FLAGS = {name: ["--version"] for name in
              ("pdflatex", "xelatex", "lualatex", "synctex", "bibtex", "biber", "python", "node")}
TOOL_FLAGS.update({name: ["-v"] for name in ("pdftotext", "pdftoppm", "pdfinfo")})


def file_receipt(path):
    path = Path(path)
    if path.is_symlink() or getattr(path, "is_junction", lambda: False)():
        raise ValueError("Explicit files must not be links")
    before = path.stat()
    if not path.is_file() or before.st_size > 128 * 1024 * 1024:
        raise ValueError("Expected a regular file no larger than 128 MiB")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    after = path.stat()
    if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
        raise ValueError("Input changed while fingerprinting")
    return {"filename": path.name, "bytes": after.st_size, "sha256": digest.hexdigest()}


def redacted_version(text):
    # Retain version output, avoiding the current account's home path if a tool prints it.
    roots = {str(Path.home()), os.environ.get("USERPROFILE", "")}
    for root in sorted(filter(None, roots), key=len, reverse=True):
        text = text.replace(root, "<USER_HOME>").replace(root.replace("\\", "/"), "<USER_HOME>")
    return "\n".join(text.splitlines()[:8])[:3000]


def tool_receipt(name, path, runner=subprocess.run):
    if name not in TOOL_FLAGS:
        raise ValueError("Unsupported version-probe tool")
    result = {"name": name, "status": "unresolved", "version_arguments": TOOL_FLAGS[name]}
    try:
        before = file_receipt(path)
        result.update(before)
        probe = runner([str(path), *TOOL_FLAGS[name]], capture_output=True, timeout=10)
        result.update(exit_code=probe.returncode,
                      version=redacted_version((probe.stdout + probe.stderr).decode("utf-8", errors="replace")))
        after = file_receipt(path)
        if before != after:
            result.update(reason="Tool changed during version probe", after_probe=after)
        elif probe.returncode == 0 and result["version"]:
            result["status"] = "recorded"
        else:
            result["reason"] = "Version probe failed or returned no version"
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["reason"] = type(error).__name__
    return result


def labelled_files(entries):
    result = []
    for label, path in entries:
        item = {"label": label, "status": "unresolved"}
        try:
            item.update(file_receipt(path), status="recorded")
        except (OSError, ValueError) as error:
            item["reason"] = type(error).__name__
        result.append(item)
    return result


def capture(tools, fonts, inputs, context):
    if context not in ("development-preflight", "development-posthoc"):
        raise ValueError("This development collector cannot seal a final evaluation")
    packages = {}
    for name in ("Pillow", "pypdf"):
        try:
            packages[name] = metadata.version(name)
        except metadata.PackageNotFoundError:
            packages[name] = None
    return {
        "schema": 1, "kind": "development_environment_snapshot_not_evaluation_results",
        "recorded_utc": datetime.now(timezone.utc).isoformat(), "context": context,
        "final_evaluation_executed": False,
        "machine": {"system": platform.system(), "release": platform.release(),
                    "version": platform.version(), "architecture": platform.machine(),
                    "processor": platform.processor(), "logical_cpus": os.cpu_count(),
                    "locale": locale.getlocale(), "python": platform.python_version(),
                    "ram_bytes": None, "webview2_version": None, "display_configuration": None},
        "python_packages": packages,
        "tools": [tool_receipt(name, path) for name, path in tools],
        "declared_font_files": labelled_files(fonts), "inputs": labelled_files(inputs),
        "limitations": [
            "Font files are explicitly declared candidates, not proof of engine font resolution or fallback.",
            "A posthoc snapshot does not establish which bytes existed during an earlier experiment.",
            "Executable fingerprints do not inventory every shared library, TeX package or operating-system dependency.",
            "RAM, WebView2 and display settings remain unrecorded; this is not a sealed reference-machine manifest.",
            "No font binaries are copied and no redistribution rights are inferred."
        ]
    }


def named_path(value):
    label, separator, raw = value.partition("=")
    if not separator or not label or not raw:
        raise argparse.ArgumentTypeError("Use a short label=path")
    return label, Path(raw)


def write_manifest(path, result):
    # Existing evidence is immutable; the caller chooses a new output name.
    with Path(path).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--context", required=True, choices=("development-preflight", "development-posthoc"))
    parser.add_argument("--tool", action="append", type=named_path, default=[])
    parser.add_argument("--font", action="append", type=named_path, default=[])
    parser.add_argument("--input", action="append", type=named_path, default=[])
    args = parser.parse_args()
    if len({name for name, _ in args.tool}) != len(args.tool) or any(name not in TOOL_FLAGS for name, _ in args.tool):
        parser.error("Use unique supported tool names: " + ", ".join(sorted(TOOL_FLAGS)))
    result = capture(args.tool, args.font, args.input, args.context)
    write_manifest(args.output, result)
    unresolved = sum(item["status"] != "recorded" for key in ("tools", "declared_font_files", "inputs") for item in result[key])
    print(f"Development snapshot saved; {unresolved} unresolved declared inputs; no experiments executed.")
    return 1 if unresolved else 0


if __name__ == "__main__":
    raise SystemExit(main())
