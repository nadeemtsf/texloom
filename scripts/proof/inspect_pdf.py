"""Development-only PDFium oracle; never imported or bundled by the application."""
import hashlib
import json
import sys
from contextlib import closing
from pathlib import Path

import pypdfium2 as pdfium
import pypdfium2.raw as raw


def inspect_pdf(source, output):
    output = Path(output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    result = {"renderer": "pdfium", "pdfium": str(pdfium.PDFIUM_INFO),
              "binding": str(pdfium.PYPDFIUM_INFO), "pdfSha256": hashlib.sha256(Path(source).read_bytes()).hexdigest(),
              "pages": [], "records": [], "graphics": [], "rasters": []}
    with closing(pdfium.PdfDocument(source)) as document:
        if len(document) > 20:
            raise ValueError("Proof fixtures are limited to 20 pages")
        for page_index in range(len(document)):
            with closing(document[page_index]) as page:
                width, height = page.get_size()
                crop = page.get_cropbox()
                if page.get_rotation() != 0 or abs(crop[0]) > 1e-5 or abs(crop[1]) > 1e-5:
                    raise ValueError("Nonzero crop origin or rotation needs the coordinate adapter")
                result["pages"].append({"width": width, "height": height, "crop": list(crop),
                                        "rotation": page.get_rotation(),
                                        "annotations": raw.FPDFPage_GetAnnotCount(page)})
                with closing(page.get_textpage()) as text:
                    for index in range(text.count_chars()):
                        value = text.get_text_range(index, 1)
                        if not value.strip():
                            continue
                        left, bottom, right, top = text.get_charbox(index)
                        result["records"].append({"text": value, "page": page_index+1,
                            "x": left, "y": height-top, "width": right-left, "height": top-bottom})
                for obj in page.get_objects():
                    if obj.type == raw.FPDF_PAGEOBJ_TEXT:
                        continue
                    left, bottom, right, top = obj.get_bounds()
                    result["graphics"].append({"type": obj.type, "page": page_index+1,
                        "x": left, "y": height-top, "width": right-left, "height": top-bottom})
                if width * height * 4 > 16_000_000:
                    raise ValueError("Proof page exceeds raster budget")
                bitmap = page.render(scale=2, rev_byteorder=False)
                try:
                    data = bytes(bitmap.buffer)
                    raster = output.with_name(f"{output.stem}-p{page_index+1}.raster")
                    raster.write_bytes(data)
                    result["rasters"].append({"path": str(raster), "width": bitmap.width,
                        "height": bitmap.height, "stride": bitmap.stride, "channels": bitmap.n_channels,
                        "scale": 2, "sha256": hashlib.sha256(data).hexdigest()})
                finally:
                    bitmap.close()
    output.write_text(json.dumps(result, ensure_ascii=False), encoding="utf-8")
    return result


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: inspect_pdf.py INPUT.pdf OUTPUT.json")
    try:
        inspect_pdf(sys.argv[1], sys.argv[2])
    except Exception as error:
        print(f"PDFium inspection failed: {error}", file=sys.stderr)
        raise SystemExit(1)
