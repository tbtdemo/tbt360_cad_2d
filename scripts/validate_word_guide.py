#!/usr/bin/env python3
"""Release gate: bundled CAD 360 Word guide must be complete and formatted.

Standard Python only. Validates real Word tables and distinct hands-on lessons
rather than accepting an AI-only or broken ZIP/XML document.
"""
from __future__ import annotations
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from zipfile import ZipFile

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
def validate(path: Path) -> None:
    with ZipFile(path) as z:
        if z.testzip() is not None:
            raise ValueError("Corrupt DOCX zip member")
        root = ET.fromstring(z.read("word/document.xml"))
        core = ET.fromstring(z.read("docProps/core.xml"))
        dc = "{http://purl.org/dc/elements/1.1/}"
        titles = [(e.text or "") for e in core.iter(dc+"title")]
        if not any("Giao trinh tong hop" in title for title in titles):
            raise ValueError(f"General CAD 360 manual has incorrect Word document title: {titles}")
    paragraphs = ["".join(t.text or "" for t in p.iter(W+"t")) for p in root.iter(W+"p")]
    all_text = "\n".join(paragraphs)
    tables = list(root.iter(W+"tbl"))
    if not any((p.find(W+"pPr") is not None and p.find(W+"pPr").find(W+"pStyle") is not None and p.find(W+"pPr").find(W+"pStyle").get(W+"val") == "Heading3") for p in root.iter(W+"p")):
        raise ValueError("Word manual has no third-level instructional heading styles")
    if len(tables) < 2:
        raise ValueError(f"Expected at least two native Word tables; got {len(tables)}")
    for n in range(1,16):
        if f"Bài {n:02d}" not in all_text:
            raise ValueError(f"Missing unique full-product hands-on lesson Bài {n:02d}")
    required = ("CAD", "ĐO BÓC", "BIM", "IFC", "XREF", "DXF", "DWG", "XLSX", "AI", "MCP")
    missing = [term for term in required if term.casefold() not in all_text.casefold()]
    if missing:
        raise ValueError(f"Missing whole-product features in Word manual: {missing}")
    if len(all_text) < 16000:
        raise ValueError(f"Word manual too short: {len(all_text)} characters")
    print(f"PASS: complete Word manual, {len(tables)} native tables, 15 unique hands-on lessons, {len(all_text)} chars")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: validate_word_guide.py PATH_TO_COMPLETE_USER_GUIDE.docx")
    validate(Path(sys.argv[1]))
