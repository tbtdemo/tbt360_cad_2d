#!/usr/bin/env python3
"""Prevent CAD 360 Windows packaging from falling back to an AI-only guide."""
from __future__ import annotations
import re
import sys
from pathlib import Path

CHAPTERS = {
    1: "CÀI ĐẶT", 2: "GIAO DIỆN", 3: "VẼ", 4: "SỬA",
    5: "LAYER", 6: "KÍCH THƯỚC", 7: "ĐO BÓC", 8: "ĐO BÓC",
    9: "EXCEL", 10: "IFC", 11: "BIM", 12: "AI", 13: "MCP",
    14: "PHỤC HỒI", 15: "DỰ ÁN",
}
REQUIRED_TERMS = (
    "LINE", "OFFSET", "TRIM", "Layer", "BLOCK", "XREF",
    "DIMLINEAR", "DXF", "DWG", "QTO", "MẶT CẮT", "Lý trình",
    "AEA", "XLSX", "IFC", "Issue", "Clash", "raster_to_cad_lines",
    "UNDO", "Headless", "Remote", "SHA256",
)

def validate(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    if len(text) < 18000:
        raise ValueError(f"Whole-product guide is suspiciously short ({len(text)} chars)")
    numbered = {int(n):title for n,title in re.findall(r"(?mi)^\s*#{1,4}\s*CHƯƠNG\s+(\d+)[.]\s+(.+)$",text)}
    # Master guide uses an overview table; detailed numbered modules below.
    if not numbered:
        numbered = {int(n):title for n,title in re.findall(r"(?mi)^\s*\|\s*(\d+)[.]\s*([^|]+)",text)}
    for n,name in CHAPTERS.items():
        if n not in numbered or name.casefold() not in numbered[n].casefold():
            raise ValueError(f"Missing or mislabelled whole-product Chapter {n}: {name}")
    missing = [term for term in REQUIRED_TERMS if term.casefold() not in text.casefold()]
    if missing:
        raise ValueError(f"Whole-product guide missing essential features: {missing}")
    lessons = {int(n) for n in re.findall(r"(?mi)^\s*#{2,4}\s*Bài\s+(\d+)\s+[-—]",text)}
    if lessons != set(range(1,16)):
        raise ValueError(f"Expected 15 distinct beginner-to-advanced lessons; found {sorted(lessons)}")
    if "Chưa nghiệm thu" not in text or "PDF" not in text:
        raise ValueError("Guide does not distinguish unverified capabilities")
    print(f"PASS: full CAD 360 manual; {len(numbered)} feature chapters, {len(lessons)} unique lessons, {len(text)} chars")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: validate_full_user_guide.py <manual.md>")
    validate(Path(sys.argv[1]))
