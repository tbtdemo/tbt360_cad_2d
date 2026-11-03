#!/usr/bin/env python3
"""Validate the real CADTBT360_V1.000 Rust-generated XLSX using only Python stdlib.

This is structural/semantic regression validation, not a replacement for
Microsoft Excel Print Preview and user acceptance testing on Windows.
"""
import re
import sys
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

NS = {"x": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
REQUIRED = {
    "[Content_Types].xml", "_rels/.rels", "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels", "xl/styles.xml", "xl/worksheets/sheet1.xml"
}

def main(path):
    workbook_path = Path(path)
    assert workbook_path.is_file(), f"Missing workbook: {workbook_path}"
    with zipfile.ZipFile(workbook_path) as workbook:
        assert set(workbook.namelist()) == REQUIRED, "Unexpected XLSX package parts"
        assert workbook.testzip() is None, "XLSX ZIP CRC or compression error"
        parts = {name: ET.fromstring(workbook.read(name)) for name in REQUIRED if name.endswith(".xml") or name.endswith(".rels")}
    sheet = parts["xl/worksheets/sheet1.xml"]
    book = parts["xl/workbook.xml"]
    style = parts["xl/styles.xml"]
    page = sheet.find("x:pageSetup", NS)
    assert page is not None and page.get("paperSize") == "9", "Paper must be ISO A4"
    assert page.get("orientation") == "landscape", "Report must print landscape"
    assert page.get("fitToWidth") == "1" and page.get("fitToHeight") == "0", "Fit to one page width, unlimited height"
    assert sheet.find("x:sheetPr/x:pageSetUpPr", NS).get("fitToPage") == "1"
    assert sheet.find("x:sheetViews/x:sheetView/x:pane", NS).get("ySplit") == "6"
    cols = sheet.findall("x:cols/x:col", NS)
    assert len(cols) == 8, f"Default must have eight print columns, found {len(cols)}"
    rows = sheet.findall("x:sheetData/x:row", NS)
    assert len(rows) >= 15, "Missing report title, header, data, subtotal or sign-off rows"
    merges = sheet.findall("x:mergeCells/x:mergeCell", NS)
    assert int(sheet.find("x:mergeCells", NS).get("count")) == len(merges), "Merge count mismatch"
    assert len({item.get("ref") for item in merges}) == len(merges), "Duplicate merges"
    assert len(rows[5].findall("x:c", NS)) == 8, "Excel column headers missing"
    texts = [c.text or "" for c in sheet.iter("{%s}t" % NS["x"])]
    combined = "\n".join(texts)
    for label in ["BẢNG TỔNG HỢP KHỐI LƯỢNG ĐO BÓC", "TBT - Dự án kiểm thử",
                  "Chủ đầu tư kiểm tra", "CT.001", "CT.002",
                  "CỘNG NHÓM", "NGƯỜI LẬP BIỂU", "ĐẠI DIỆN CHỦ ĐẦU TƯ"]:
        assert label in combined, f"Missing report field: {label}"
    numeric = [float(v.text) for v in sheet.findall(".//x:c/x:v", NS) if v.text]
    assert numeric.count(12.25) >= 2 and numeric.count(8.5) >= 2, "Separate sums by measurement and unit missing"
    assert not any(abs(v - 20.75) < 1e-8 for v in numeric), "Length and area must never be summed together"
    names = {item.get("name"): item.text for item in book.findall("x:definedNames/x:definedName", NS)}
    assert "_xlnm.Print_Titles" in names and "$1:$6" in names["_xlnm.Print_Titles"]
    assert "_xlnm.Print_Area" in names and re.search(r"\$A\$1:\$H\$\d+", names["_xlnm.Print_Area"])
    fmt = style.find("x:numFmts/x:numFmt", NS)
    assert fmt is not None and "#,##0.00" in fmt.get("formatCode"), "Two decimal places not retained"
    assert len(style.findall("x:cellXfs/x:xf", NS)) >= 11, "Worksheet styles missing"
    print(f"QTO XLSX verified: {workbook_path.name}, {len(rows)} rows, {len(cols)} print columns, A4 landscape.")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: verify_qto_xlsx.py <Rust-generated-workbook.xlsx>")
    main(sys.argv[1])
