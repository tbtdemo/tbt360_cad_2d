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
        b"<< /Length 27 >>\nstream\n0 0 0 RG 10 80 m 290 80 l S\nendstream",
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



def png_sample_brightness(path: pathlib.Path, pixels: list[tuple[int, int]]) -> list[int]:
    """Decode selected 8-bit RGB/RGBA PNG pixels, including PNG row filters."""
    import zlib
    data=path.read_bytes()
    pos=8; compressed=bytearray(); width=height=channels=None
    while pos+12<=len(data):
        size=struct.unpack(">I",data[pos:pos+4])[0]; kind=data[pos+4:pos+8]
        chunk=data[pos+8:pos+8+size]; pos+=size+12
        if kind==b"IHDR":
            width,height,depth,color=struct.unpack(">IIBB",chunk[:10])
            if depth!=8 or color not in (2,6): raise RuntimeError("Unexpected PNG color mode")
            channels=3 if color==2 else 4
        elif kind==b"IDAT": compressed.extend(chunk)
        elif kind==b"IEND": break
    if not compressed or width is None or width*height>6_000_000: raise RuntimeError("Unsupported PDF raster dimensions")
    raw=zlib.decompress(compressed); stride=width*channels
    if len(raw)!=height*(stride+1): raise RuntimeError("Invalid PNG scanline length")
    prev=bytearray(stride); samples={}; offset=0
    target_rows={y for x,y in pixels}
    for y in range(max(target_rows)+1):
        filter_type=raw[offset]; row=bytearray(raw[offset+1:offset+1+stride]); offset+=stride+1
        for j in range(stride):
            a=row[j-channels] if j>=channels else 0
            b=prev[j]; c=prev[j-channels] if j>=channels else 0
            if filter_type==1: predictor=a
            elif filter_type==2: predictor=b
            elif filter_type==3: predictor=(a+b)//2
            elif filter_type==4:
                p=a+b-c
                predictor=min((a,b,c),key=lambda value:abs(value-p))
            elif filter_type==0: predictor=0
            else: raise RuntimeError("Unexpected PNG row filter")
            row[j]=(row[j]+predictor)&255
        if y in target_rows:
            for x,yy in pixels:
                if yy==y: samples[(x,yy)]=sum(row[x*channels:x*channels+3])
        prev=row
    return [samples[p] for p in pixels]


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
        line_y=round(height*100/180)
        brightness=png_sample_brightness(png,[(width//2,height//5)]+[(width//2,y) for y in range(line_y-8,line_y+9)])
        if brightness[0]<600 or min(brightness[1:])>300:
            raise RuntimeError("PDF vector line or white background missing from raster image")
        print(f"PASS PDF-to-PNG: {width}x{height}, {png.stat().st_size} bytes, bundled exe {renderer}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("renderer", type=pathlib.Path)
    verify(parser.parse_args().renderer)
