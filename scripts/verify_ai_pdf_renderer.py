"""Windows smoke: exercise the exact bundled Poppler PDF raster used by AI Studio."""
import argparse
import pathlib
import struct
import subprocess
import tempfile


def make_pdf(path: pathlib.Path) -> None:
    # One-page PDF with a real horizontal line. All xref offsets computed from bytes.
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 180] /Contents 4 0 R /Resources << >> >>",
        b"<< /Length 30 >>\nstream\n0 0 0 RG 10 80 m 290 80 l S\nendstream",
    ]
    data = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for i, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f"{i} 0 obj\n".encode() + obj + b"\nendobj\n")
    xref = len(data)
    data.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        data.extend(f"{offset:010d} 00000 n \n".encode())
    data.extend(f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    path.write_bytes(data)


def verify(renderer: pathlib.Path) -> None:
    if not renderer.is_file():
        raise RuntimeError(f"Missing bundled renderer: {renderer}")
    with tempfile.TemporaryDirectory(prefix="cad360-ai-pdf-smoke-") as directory:
        directory = pathlib.Path(directory)
        source = directory / "plan.pdf"
        prefix = directory / "page"
        png = prefix.with_suffix(".png")
        make_pdf(source)
        process = subprocess.run(
            [str(renderer), "-f", "1", "-l", "1", "-singlefile",
             "-scale-to", "2400", "-png", str(source), str(prefix)],
            timeout=65, capture_output=True, check=False
        )
        if process.returncode:
            raise RuntimeError(f"Bundled Poppler failed ({process.returncode}): {process.stderr[:500]!r}")
        header = png.read_bytes()[:24]
        if header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
            raise RuntimeError("PDF conversion failed to produce a valid PNG")
        width, height = struct.unpack(">II", header[16:24])
        if not 100 <= width <= 2400 or not 100 <= height <= 2400:
            raise RuntimeError(f"Unexpected raster dimensions: {width}x{height}")
        print(f"PASS PDF-to-PNG: {width}x{height}, {png.stat().st_size} bytes, bundled exe {renderer}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("renderer", type=pathlib.Path)
    verify(parser.parse_args().renderer)
