"""Prepare the pinned Windows development worker dependency; no runtime Python."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

WHEEL_SHA256 = "149fd5c6397b8df8bf7911a93506eff0be874f877afe7ac936cf5d37d21a6a06"
DLL_SHA256 = "69f1e860f6742fbaafcc4db80e785eec9a4ee0da36ea6d1dc0867ea0f18baea6"


def create_checked(path, data):
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError(f"Existing worker dependency differs: {path.name}")
        return
    with path.open("xb") as handle:
        handle.write(data)


def prepare(wheel, output):
    wheel, output = Path(wheel), Path(output)
    if wheel.stat().st_size > 30_000_000 or hashlib.sha256(wheel.read_bytes()).hexdigest() != WHEEL_SHA256:
        raise ValueError("Pinned Windows PDFium wheel checksum mismatch")
    output.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(wheel) as archive:
        # Read exact entries, never extract caller-controlled archive paths.
        # Historical Python reference, separate from the active native runtime.
        pin = json.loads((Path(__file__).resolve().parent.parent / "pdfium-proof-pin-156.json").read_text(encoding="utf8"))
        if pin["wheel_sha256"] != WHEEL_SHA256 or pin["dll_sha256"] != DLL_SHA256:
            raise ValueError("PDFium pin files disagree")
        entries = {"pypdfium2_raw/pdfium.dll": "pdfium.dll"}
        for entry, name in entries.items():
            if archive.getinfo(entry).file_size > 30_000_000:
                raise ValueError("Worker dependency exceeds extraction budget")
            data = archive.read(entry)
            if name == "pdfium.dll" and hashlib.sha256(data).hexdigest() != DLL_SHA256:
                raise ValueError("Pinned PDFium DLL checksum mismatch")
            create_checked(output / name, data)
        notice = "\n\n".join("===== " + name + " =====\n" + archive.read(name).decode(pin["notice_encodings"].get(name, "utf8")) for name in pin["notices"])
        create_checked(output / "PDFIUM-NOTICE.txt", notice.encode("utf8"))
    manifest = {"schema": 1, "purpose": "development inspection experiment; not a certified runtime dependency",
                "platform": "windows-x86_64", "pdfium": "156.0.8076.0", "binding_source": "pypdfium2-5.14.0",
                "wheel_sha256": WHEEL_SHA256, "dll_sha256": DLL_SHA256}
    create_checked(output / "pdfium-manifest.json", (json.dumps(manifest, indent=2) + "\n").encode())
    print(json.dumps(manifest))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: prepare_pdfium.py WHEEL.whl OUTPUT_DIRECTORY")
    prepare(*sys.argv[1:])
