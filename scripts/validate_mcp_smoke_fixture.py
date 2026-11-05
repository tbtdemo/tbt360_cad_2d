#!/usr/bin/env python3
"""Validate the real single-stroke PNG encoded in Windows MCP release smoke.

No Pillow or external packages. Prevents invalid test fixtures from being
misdiagnosed as Rust/CAD failures.
"""
import base64
import re
import struct
import sys
import zlib
from pathlib import Path

def check(script: Path) -> None:
    raw = script.read_text(encoding="utf-8")
    match = re.search(r"(?m)^\$pngBase64\s*=\s*'([A-Za-z0-9+/=]+)'\s*$", raw)
    if not match:
        raise ValueError("MCP smoke PowerShell script has no embedded PNG base64")
    encoded = match.group(1)
    data = base64.b64decode(encoded, validate=True)
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("Invalid PNG signature")
    offset = 8
    idat = bytearray()
    width = height = 0
    saw_end = False
    while offset < len(data):
        if len(data) - offset < 12:
            raise ValueError("Truncated PNG chunk")
        length = struct.unpack_from(">I", data, offset)[0]
        end = offset + 12 + length
        if end > len(data):
            raise ValueError("PNG chunk extends beyond file")
        name = data[offset+4:offset+8]
        payload = data[offset+8:offset+8+length]
        crc = struct.unpack_from(">I", data, offset+8+length)[0]
        if zlib.crc32(name + payload) != crc:
            raise ValueError(f"Invalid CRC in PNG chunk {name!r}")
        if name == b"IHDR":
            width, height, depth, color, compress, filt, interlace = struct.unpack(">IIBBBBB", payload)
            if (width, height, depth, color, compress, filt, interlace) != (64, 32, 8, 2, 0, 0, 0):
                raise ValueError("Smoke PNG must be 64x32 non-interlaced RGB8")
        elif name == b"IDAT":
            idat.extend(payload)
        elif name == b"IEND":
            saw_end = True
            if end != len(data):
                raise ValueError("Trailing bytes after PNG IEND")
        offset = end
    if not saw_end or not idat:
        raise ValueError("PNG missing compressed pixels or IEND")
    decoded = zlib.decompress(bytes(idat))
    stride = width * 3
    if len(decoded) != height * (stride + 1):
        raise ValueError("Unexpected decoded PNG scanline length")
    previous = bytearray(stride)
    black = 0
    dark_rows = []
    dark_positions = []
    for y in range(height):
        start = y*(stride+1)
        filter_type = decoded[start]
        row = bytearray(decoded[start+1:start+1+stride])
        if filter_type > 4:
            raise ValueError("Invalid PNG filter")
        for i in range(stride):
            left = row[i-3] if i >= 3 else 0
            up = previous[i]
            ul = previous[i-3] if i >= 3 else 0
            if filter_type == 1:
                row[i] = (row[i] + left) & 255
            elif filter_type == 2:
                row[i] = (row[i] + up) & 255
            elif filter_type == 3:
                row[i] = (row[i] + ((left + up)//2)) & 255
            elif filter_type == 4:
                p = left + up - ul
                a, b, c = abs(p-left), abs(p-up), abs(p-ul)
                row[i] = (row[i] + (left if a <= b and a <= c else up if b <= c else ul)) & 255
        xs = [x for x in range(width) if row[x*3] <= 160 and row[x*3+1] <= 160 and row[x*3+2] <= 160]
        count = len(xs)
        if count:
            dark_rows.append((y,count))
            dark_positions.extend((x,y) for x in xs)
            black += count
        previous = row
    expected_positions = [(x,15) for x in range(6,57)]
    if dark_rows != [(15,51)] or black != 51 or dark_positions != expected_positions:
        raise ValueError(f"Smoke PNG must have exact black pixels x=6..56 on row 15: {dark_rows!r}, positions={dark_positions!r}")
    print("PASS: valid PNG 64x32 RGB8; black stroke x=6..56 at y=15 (51 pixels); vector length at 10 units/pixel = 500 units")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: validate_mcp_smoke_fixture.py scripts/verify_mcp_executable.ps1")
    check(Path(sys.argv[1]))
