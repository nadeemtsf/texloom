"""Prepare the pinned Windows development worker dependency; no runtime Python."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

WHEEL_SHA256 = "90dbb2ac07be53219f56be09961eb95cf2473f834d01a42d901d13ccfad64b4c"
DLL_SHA256 = "b2c15cd4537d8efaf23b85b3353abe7a0b863e9c52b2f017dd87bb86a55f9c72"


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
        entries = {"pypdfium2_raw/pdfium.dll": "pdfium.dll",
                   "pypdfium2-4.30.0.dist-info/LicenseRef-PdfiumThirdParty.txt": "PDFIUM-NOTICE.txt"}
        for entry, name in entries.items():
            if archive.getinfo(entry).file_size > 30_000_000:
                raise ValueError("Worker dependency exceeds extraction budget")
            data = archive.read(entry)
            if name == "pdfium.dll" and hashlib.sha256(data).hexdigest() != DLL_SHA256:
                raise ValueError("Pinned PDFium DLL checksum mismatch")
            create_checked(output / name, data)
    manifest = {"schema": 1, "purpose": "development inspection experiment; not a certified runtime dependency",
                "platform": "windows-x86_64", "pdfium": "126.0.6462.0", "binding_source": "pypdfium2-4.30.0",
                "wheel_sha256": WHEEL_SHA256, "dll_sha256": DLL_SHA256}
    create_checked(output / "pdfium-manifest.json", (json.dumps(manifest, indent=2) + "\n").encode())
    print(json.dumps(manifest))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: prepare_pdfium.py WHEEL.whl OUTPUT_DIRECTORY")
    prepare(*sys.argv[1:])
