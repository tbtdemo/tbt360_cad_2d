#!/usr/bin/env python3
"""Build the bundled Vietnamese CAD 360 Word user guide without network dependencies.

Usage: python docs/user-guide/build_word_guide.py INPUT.md OUTPUT.docx
Works with the standard CPython included on GitHub's Windows runners.
"""
from __future__ import annotations

import re
import sys
from html import escape
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

MAIN_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"

def para(text: str, style: str = "Normal") -> str:
    # Strip only Markdown markup. Underscores are literal parts of CAD file names,
    # SHA-related labels and PowerShell paths, so they MUST survive DOCX generation.
    safe = escape(re.sub(r"(?<!\\)(?:\\*\\*|`)", "", text.strip()))
    return ('<w:p><w:pPr><w:pStyle w:val="' + style + '"/></w:pPr>'
            '<w:r><w:t xml:space="preserve">' + safe + '</w:t></w:r></w:p>')

def table_xml(rows: list[list[str]]) -> str:
    """Convert Markdown table rows to real Word table cells, never pipe-separated text."""
    if not rows:
        return ""
    columns = max(len(row) for row in rows)
    width = max(720, 9638 // columns)
    grid = "".join(f'<w:gridCol w:w="{width}"/>' for _ in range(columns))
    parts = ['<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/>'
             '<w:tblBorders><w:top w:val="single" w:color="C6D3DF" w:sz="6"/>'
             '<w:left w:val="single" w:color="C6D3DF" w:sz="4"/>'
             '<w:bottom w:val="single" w:color="C6D3DF" w:sz="6"/>'
             '<w:right w:val="single" w:color="C6D3DF" w:sz="4"/>'
             '<w:insideH w:val="single" w:color="D9E3EB" w:sz="4"/>'
             '<w:insideV w:val="single" w:color="D9E3EB" w:sz="4"/>'
             '</w:tblBorders></w:tblPr><w:tblGrid>' + grid + '</w:tblGrid>']
    for index, row in enumerate(rows):
        parts.append('<w:tr>' + ('<w:trPr><w:tblHeader w:val="true"/></w:trPr>' if index == 0 else ''))
        for col in range(columns):
            value = row[col] if col < len(row) else ""
            shade = '<w:shd w:fill="E8F1F8"/>' if index == 0 else ('<w:shd w:fill="F6F9FC"/>' if index % 2 == 0 else '')
            parts.append(f'<w:tc><w:tcPr><w:tcW w:w="{width}" w:type="dxa"/>{shade}</w:tcPr>'
                         + para(value, "TableHeader" if index == 0 else "TableText") + '</w:tc>')
        parts.append('</w:tr>')
    return "".join(parts) + '</w:tbl>'

def document_xml(markdown: str) -> str:
    paragraphs: list[str] = []
    pending_table: list[list[str]] = []
    in_code = False
    def flush_table() -> None:
        if pending_table:
            paragraphs.append(table_xml(pending_table))
            pending_table.clear()
    for raw in markdown.splitlines():
        line = raw.strip()
        if line.startswith("```"):
            flush_table()
            in_code = not in_code
            continue
        if line.startswith("|") and not in_code:
            cells = [part.strip() for part in line.strip("|").split("|")]
            if cells and all(re.fullmatch(r":?-{3,}:?", c) for c in cells):
                continue
            pending_table.append(cells)
            continue
        flush_table()
        if not line:
            paragraphs.append("<w:p/>")
            continue
        if in_code:
            paragraphs.append(para(raw, "Code"))
            continue
        if line.startswith("# "):
            paragraphs.append(para(line[2:], "Title"))
        elif line.startswith("## "):
            paragraphs.append(para(line[3:], "Heading1"))
        elif line.startswith("### "):
            paragraphs.append(para(line[4:], "Heading2"))
        elif line.startswith("#### "):
            paragraphs.append(para(line[5:], "Heading3"))
        elif re.match(r"^\d+[.]\s", line):
            paragraphs.append(para(line, "ListParagraph"))
        elif line.startswith("- ") or line.startswith("- ["):
            paragraphs.append(para("• " + line[2:], "ListParagraph"))
        elif line.startswith(">"):
            paragraphs.append(para(line[1:].strip(), "Quote"))
        elif line.startswith("---"):
            paragraphs.append("<w:p/>")
        else:
            paragraphs.append(para(line))
    flush_table()
    body = "".join(paragraphs)
    return ('<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<w:document xmlns:w="' + MAIN_NS + '"><w:body>' + body
            + '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/>'
            + '<w:pgMar w:top="1134" w:right="1134" w:bottom="1134" w:left="1134"/>'
            + '</w:sectPr></w:body></w:document>')

STYLES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/><w:sz w:val="21"/></w:rPr></w:rPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:spacing w:after="110" w:line="290" w:lineRule="auto"/></w:pPr></w:style>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:pPr><w:spacing w:after="270"/></w:pPr><w:rPr><w:b/><w:color w:val="17365D"/><w:sz w:val="35"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="260" w:after="130"/></w:pPr><w:rPr><w:b/><w:color w:val="17547D"/><w:sz w:val="28"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="160" w:after="90"/></w:pPr><w:rPr><w:b/><w:color w:val="146E7B"/><w:sz w:val="23"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="130" w:after="70"/></w:pPr><w:rPr><w:b/><w:color w:val="275375"/><w:sz w:val="21"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="320"/><w:spacing w:after="70"/></w:pPr></w:style>
<w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="350" w:right="280"/></w:pPr><w:rPr><w:i/><w:color w:val="52525B"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/><w:basedOn w:val="Normal"/><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/><w:sz w:val="17"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="TableHeader"><w:name w:val="Table Header"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="40"/></w:pPr><w:rPr><w:b/><w:sz w:val="18"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="TableText"><w:name w:val="Table Text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="40"/></w:pPr><w:rPr><w:sz w:val="18"/></w:rPr></w:style>
</w:styles>"""

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"""

ROOT_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"""

DOC_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"""

CORE = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title>CAD 360 Viet Nam V1.025 - Vietnamese User Guide</dc:title>
<dc:creator>TBT Viet Nam</dc:creator>
</cp:coreProperties>"""

def build(source: Path, dest: Path) -> None:
    if not source.is_file() or source.stat().st_size < 1000:
        raise ValueError("User guide source is missing or unexpectedly small")
    markdown = source.read_text(encoding="utf-8")
    required = ("Bước 1", "Bước 2", "Bước 3", "Preview", "UNDO", "DXF")
    missing = [item for item in required if item not in markdown]
    if missing:
        raise ValueError(f"User guide is incomplete: missing {missing}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    # Version follows the actual source manual, never a stale hardcoded 1.025.
    match = re.search(r"V1[.]\d{3}", dest.name.upper())
    if not match:
        match = re.search(r"V1[.]\d{3}", markdown)
    release = match.group(0) if match else "VERSION_UNVERIFIED"
    core_title = "CAD 360 Viet Nam " + release + " - " + ("Giao trinh tong hop CAD 2D, QTO, IFC, BIM, AI, MCP" if "GIAO_TRINH" in source.name.upper() else "Huong dan AI tu anh thanh CAD")
    core = CORE.replace("CAD 360 Viet Nam V1.025 - Vietnamese User Guide", escape(core_title))
    with ZipFile(dest, "w", compression=ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", CONTENT_TYPES)
        z.writestr("_rels/.rels", ROOT_RELS)
        z.writestr("word/document.xml", document_xml(markdown))
        z.writestr("word/styles.xml", STYLES)
        z.writestr("word/_rels/document.xml.rels", DOC_RELS)
        z.writestr("docProps/core.xml", core)
    with ZipFile(dest) as z:
        bad = z.testzip()
        required_parts = {"[Content_Types].xml", "_rels/.rels", "word/document.xml", "word/styles.xml"}
        if bad or not required_parts.issubset(z.namelist()):
            raise ValueError(f"Invalid DOCX output: {bad}")
        document = z.read("word/document.xml")
        if "|" in markdown and "<w:tbl>" not in document.decode("utf-8"):
            raise ValueError("Markdown contains table cells but DOCX contains no actual Word tables")
    if dest.stat().st_size < 2000:
        raise ValueError("Generated DOCX unexpectedly small")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("Usage: build_word_guide.py INPUT.md OUTPUT.docx")
    build(Path(sys.argv[1]), Path(sys.argv[2]))
    print(f"Built user guide: {sys.argv[2]}")
