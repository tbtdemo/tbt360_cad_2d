//! Print-ready XLSX report for CAD takeoff, without introducing dependencies
//! or changing Cargo.lock. XLSX is a ZIP container of Office Open XML parts.
//! The writer deliberately uses stored ZIP entries for reliable interoperability.
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

pub const FIELDS: [(&str, f64); 12] = [
    ("STT", 8.0), ("Mã công tác", 19.0), ("Nội dung đo bóc", 43.0),
    ("Nhóm công việc", 26.0), ("Layer CAD", 21.0), ("Tầng", 12.0),
    ("Bộ môn", 17.0), ("Loại phép đo", 18.0), ("Khối lượng", 18.0),
    ("Đơn vị", 11.0), ("Ghi chú", 37.0), ("Handle nguồn", 23.0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GroupBy { #[default] WorkGroup, WorkCode, Layer, Floor, Discipline, None }
impl GroupBy {
    pub const ALL: [Self; 6] = [Self::WorkGroup, Self::WorkCode, Self::Layer, Self::Floor, Self::Discipline, Self::None];
    pub fn label(self) -> &'static str { match self {
        Self::WorkGroup => "Nhóm công việc", Self::WorkCode => "Mã công tác",
        Self::Layer => "Layer CAD", Self::Floor => "Tầng",
        Self::Discipline => "Bộ môn", Self::None => "Không phân nhóm",
    }}
}

#[derive(Clone, Debug)]
pub struct ExportConfig {
    pub columns: [bool; 12],
    pub group_by: GroupBy,
    pub project: String,
    pub investor: String,
    pub document_number: String,
    pub confirmed_only: bool,
    pub length_unit: String,
    pub decimals: usize,
}
impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            columns: [true, true, true, true, true, true, true, true, true, true, true, false],
            group_by: GroupBy::WorkGroup,
            project: String::new(), investor: String::new(), document_number: String::new(),
            confirmed_only: true, length_unit: "m".into(), decimals: 3,
        }
    }
}
impl ExportConfig {
    pub fn selected(&self) -> Vec<usize> { (0..FIELDS.len()).filter(|i| self.columns[*i]).collect() }
    pub fn valid(&self) -> bool { self.selected().len() >= 3 && self.columns[8] && self.columns[9] && self.columns[..8].iter().any(|x| *x) && self.decimals <= 6 }
    pub fn unit_scale(&self) -> f64 { match self.length_unit.as_str() { "mm" => 1000.0, "cm" => 100.0, _ => 1.0 } }
}

#[derive(Clone)]
pub struct ReportLine {
    pub group: String,
    pub fields: [String; 12],
    pub quantity: f64,
    pub unit: String,
}
fn xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 24);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"), '<' => out.push_str("&lt;"), '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"), '\'' => out.push_str("&apos;"),
            c if (c as u32) < 32 && !matches!(c, '\t' | '\n' | '\r') => {},
            _ => out.push(c),
        }
    }
    out
}
fn column(mut n: usize) -> String {
    let mut result = String::new();
    n += 1;
    while n > 0 { n -= 1; result.insert(0, (b'A' + (n % 26) as u8) as char); n /= 26; }
    result
}
fn text_cell(reference: &str, text: &str, style: usize) -> String {
    format!("<c r=\"{reference}\" s=\"{style}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>", xml(text))
}
fn number_cell(reference: &str, number: f64, style: usize) -> String {
    format!("<c r=\"{reference}\" s=\"{style}\"><v>{number:.9}</v></c>")
}
fn row_start(index: usize, height: u32) -> String {
    format!("<row r=\"{index}\" ht=\"{height}\" customHeight=\"1\">")
}
fn title_row(index: usize, label: &str, style: usize, height: u32) -> String {
    format!("{}{}{}</row>", row_start(index, height), text_cell(&format!("A{index}"), label, style), "")
}
fn sheet(config: &ExportConfig, rows: &[ReportLine]) -> (String, usize) {
    let selected = config.selected();
    let end_column = column(selected.len() - 1);
    let mut body = String::with_capacity(rows.len() * 700 + 5000);
    let mut merges = Vec::<String>::new();
    let title = "BẢNG TỔNG HỢP KHỐI LƯỢNG ĐO BÓC";
    body.push_str(&title_row(1, title, 1, 34));
    body.push_str(&title_row(2, &format!("CÔNG TRÌNH / DỰ ÁN: {}", config.project.trim()), 2, 25));
    body.push_str(&title_row(3, &format!("CHỦ ĐẦU TƯ: {}", config.investor.trim()), 2, 24));
    body.push_str(&title_row(4, &format!("SỐ HIỆU: {}     |     ĐƠN VỊ CHIỀU DÀI: {}     |     PHÂN NHÓM: {}", config.document_number, config.length_unit, config.group_by.label()), 3, 24));
    body.push_str(&title_row(5, if config.confirmed_only { "DỮ LIỆU ĐÃ XÁC NHẬN — VUI LÒNG KIỂM TRA VÀ KÝ DUYỆT TRƯỚC KHI PHÁT HÀNH" } else { "DỰ THẢO / CHƯA XÁC NHẬN — KHÔNG SỬ DỤNG LÀM HỒ SƠ PHÁT HÀNH" }, 8, 24));
    for r in 1..=5 { merges.push(format!("<mergeCell ref=\"A{r}:{end_column}{r}\"/>")); }
    body.push_str(&row_start(6, 34));
    for (i, index) in selected.iter().enumerate() { body.push_str(&text_cell(&format!("{}6", column(i)), FIELDS[*index].0, 4)); }
    body.push_str("</row>");

    let quantity_column = selected.iter().position(|index| *index == 8).unwrap_or(0);
    let mut current_group = None::<String>;
    let mut group_totals = BTreeMap::<(String, String), f64>::new();
    let mut grand_totals = BTreeMap::<(String, String), f64>::new();
    let mut excel_row = 7usize;
    let mut serial = 0usize;
    for line in rows {
        if config.group_by != GroupBy::None && current_group.as_deref() != Some(&line.group) {
            if current_group.is_some() {
                add_totals(&mut body, &mut excel_row, &mut merges, &group_totals, quantity_column, &selected, "CỘNG NHÓM");
                group_totals.clear();
            }
            body.push_str(&row_start(excel_row, 27));
            body.push_str(&text_cell(&format!("A{excel_row}"), &format!("NHÓM: {}", line.group), 6));
            body.push_str("</row>");
            merges.push(format!("<mergeCell ref=\"A{excel_row}:{end_column}{excel_row}\"/>"));
            excel_row += 1;
            current_group = Some(line.group.clone());
        }
        serial += 1;
        body.push_str(&row_start(excel_row, 26));
        for (i, index) in selected.iter().enumerate() {
            let address = format!("{}{excel_row}", column(i));
            if *index == 8 { body.push_str(&number_cell(&address, line.quantity, 5)); }
            else if *index == 0 { body.push_str(&number_cell(&address, serial as f64, 7)); }
            else { body.push_str(&text_cell(&address, &line.fields[*index], if serial % 2 == 0 { 9 } else { 0 })); }
        }
        body.push_str("</row>");
        excel_row += 1;
        *group_totals.entry((line.fields[7].clone(), line.unit.clone())).or_default() += line.quantity;
        *grand_totals.entry((line.fields[7].clone(), line.unit.clone())).or_default() += line.quantity;
    }
    if config.group_by != GroupBy::None && current_group.is_some() {
        add_totals(&mut body, &mut excel_row, &mut merges, &group_totals, quantity_column, &selected, "CỘNG NHÓM");
    }
    add_totals(&mut body, &mut excel_row, &mut merges, &grand_totals, quantity_column, &selected, "TỔNG THEO LOẠI & ĐƠN VỊ");
    // Include signature and sign-off row in the printed range.
    let last = excel_row + 2;
    body.push_str(&row_start(excel_row + 2, 28));
    body.push_str(&text_cell(&format!("A{}", excel_row + 2), "NGƯỜI LẬP BIỂU", 2));
    if selected.len() >= 4 { body.push_str(&text_cell(&format!("D{}", excel_row + 2), "NGƯỜI KIỂM TRA", 2)); }
    if selected.len() >= 7 { body.push_str(&text_cell(&format!("G{}", excel_row + 2), "ĐẠI DIỆN CHỦ ĐẦU TƯ", 2)); }
    body.push_str("</row>");

    let mut widths = String::new();
    for (i, index) in selected.iter().enumerate() {
        widths.push_str(&format!("<col min=\"{}\" max=\"{}\" width=\"{}\" customWidth=\"1\"/>", i + 1, i + 1, FIELDS[*index].1));
    }
    let merge_count = merges.len();
    let merges = merges.join("");
    let xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetPr><pageSetUpPr fitToPage="1"/></sheetPr>
<dimension ref="A1:{end_column}{}"/>
<sheetViews><sheetView showGridLines="0" workbookViewId="0"><pane ySplit="6" topLeftCell="A7" activePane="bottomLeft" state="frozen"/></sheetView></sheetViews>
<sheetFormatPr defaultRowHeight="22"/><cols>{widths}</cols><sheetData>{body}</sheetData>
<mergeCells count="{}">{merges}</mergeCells>
<printOptions headings="0" gridLines="0"/>
<pageMargins left="0.25" right="0.25" top="0.45" bottom="0.45" header="0.18" footer="0.18"/>
<pageSetup paperSize="9" orientation="landscape" fitToWidth="1" fitToHeight="0"/>
<headerFooter><oddHeader>&amp;CCAD TBT 360 — BẢNG ĐO BÓC</oddHeader><oddFooter>&amp;L{}&amp;RTrang &amp;P / &amp;N</oddFooter></headerFooter>
</worksheet>"#, excel_row + 2, merge_count, xml(&config.document_number));
    // Merged ranges, styles and pagination are all emitted in OOXML, not CSV.
    (xml, last)
}
fn add_totals(body: &mut String, row: &mut usize, merges: &mut Vec<String>, totals: &BTreeMap<(String, String), f64>, qty_idx: usize, selected: &[usize], prefix: &str) {
    for ((kind, unit), value) in totals {
        body.push_str(&row_start(*row, 27));
        // Total labels span all columns before "Khối lượng", leaving the numeric
        // total and unit columns visible and print-friendly.
        body.push_str(&text_cell(&format!("A{}", *row), &format!("{prefix} — {kind} ({unit})"), 6));
        if qty_idx > 1 { merges.push(format!("<mergeCell ref=\"A{}:{}{}\"/>", *row, column(qty_idx - 1), *row)); }
        for (i, field) in selected.iter().enumerate() {
            let pos = format!("{}{}", column(i), *row);
            if i == qty_idx { body.push_str(&number_cell(&pos, *value, 10)); }
            else if *field == 9 { body.push_str(&text_cell(&pos, unit, 6)); }
        }
        body.push_str("</row>");
        *row += 1;
    }
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for b in bytes {
        crc ^= *b as u32;
        for _ in 0..8 { crc = (crc >> 1) ^ (if crc & 1 != 0 { 0xedb8_8320 } else { 0 }); }
    }
    !crc
}
fn le16(n: u16) -> [u8; 2] { n.to_le_bytes() }
fn le32(n: u32) -> [u8; 4] { n.to_le_bytes() }
/// ZIP32 (stored) with correct local header, central directory and EOCD.
fn zip_store(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, bytes) in entries {
        let name_bytes = name.as_bytes();
        let size = u32::try_from(bytes.len()).map_err(|_| "XLSX sheet too large".to_string())?;
        let offset = u32::try_from(out.len()).map_err(|_| "XLSX archive too large".to_string())?;
        let nlen = u16::try_from(name_bytes.len()).map_err(|_| "ZIP file name too long".to_string())?;
        let crc = crc32(bytes);
        out.extend(le32(0x04034b50)); out.extend(le16(20)); out.extend(le16(0x0800));
        out.extend(le16(0)); out.extend(le16(0)); out.extend(le16(0));
        out.extend(le32(crc)); out.extend(le32(size)); out.extend(le32(size));
        out.extend(le16(nlen)); out.extend(le16(0)); out.extend(name_bytes); out.extend(bytes);
        directory.extend(le32(0x02014b50)); directory.extend(le16(20)); directory.extend(le16(20));
        directory.extend(le16(0x0800)); directory.extend(le16(0)); directory.extend(le16(0)); directory.extend(le16(0));
        directory.extend(le32(crc)); directory.extend(le32(size)); directory.extend(le32(size));
        directory.extend(le16(nlen)); directory.extend(le16(0)); directory.extend(le16(0));
        directory.extend(le16(0)); directory.extend(le16(0)); directory.extend(le32(0)); directory.extend(le32(offset));
        directory.extend(name_bytes);
    }
    let directory_offset = u32::try_from(out.len()).map_err(|_| "XLSX archive too large".to_string())?;
    let directory_size = u32::try_from(directory.len()).map_err(|_| "XLSX central directory too large".to_string())?;
    let count = u16::try_from(entries.len()).map_err(|_| "Too many workbook files".to_string())?;
    out.extend(directory);
    out.extend(le32(0x06054b50)); out.extend(le16(0)); out.extend(le16(0));
    out.extend(le16(count)); out.extend(le16(count)); out.extend(le32(directory_size));
    out.extend(le32(directory_offset)); out.extend(le16(0));
    Ok(out)
}
const STYLES: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<numFmts count="1"><numFmt numFmtId="164" formatCode="#,##0.000;[Red](#,##0.000);-"/></numFmts>
<fonts count="4">
<font><sz val="10"/><name val="Arial"/></font>
<font><b/><sz val="16"/><color rgb="FFFFFFFF"/><name val="Arial"/></font>
<font><b/><sz val="10"/><color rgb="FF193B5C"/><name val="Arial"/></font>
<font><b/><sz val="10"/><color rgb="FFFFFFFF"/><name val="Arial"/></font>
</fonts>
<fills count="7">
<fill><patternFill patternType="none"/></fill>
<fill><patternFill patternType="gray125"/></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FF173B63"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FF22609A"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFE7F1FA"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFFFF0D5"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFF2F6FB"/></patternFill></fill>
</fills>
<borders count="2"><border><left/><right/><top/><bottom/><diagonal/></border>
<border><left style="thin"><color rgb="FFCBD5E1"/></left><right style="thin"><color rgb="FFCBD5E1"/></right><top style="thin"><color rgb="FFCBD5E1"/></top><bottom style="thin"><color rgb="FFCBD5E1"/></bottom><diagonal/></border></borders>
<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
<cellXfs count="11">
<xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0" applyAlignment="1"><alignment vertical="center" wrapText="1"/></xf>
<xf numFmtId="0" fontId="1" fillId="2" borderId="0" xfId="0" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>
<xf numFmtId="0" fontId="2" fillId="4" borderId="0" xfId="0" applyAlignment="1"><alignment vertical="center"/></xf>
<xf numFmtId="0" fontId="2" fillId="0" borderId="0" xfId="0" applyAlignment="1"><alignment vertical="center"/></xf>
<xf numFmtId="0" fontId="3" fillId="3" borderId="1" xfId="0" applyAlignment="1"><alignment horizontal="center" vertical="center" wrapText="1"/></xf>
<xf numFmtId="164" fontId="0" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyAlignment="1"><alignment horizontal="right" vertical="center"/></xf>
<xf numFmtId="0" fontId="2" fillId="4" borderId="0" xfId="0" applyAlignment="1"><alignment vertical="center"/></xf>
<xf numFmtId="1" fontId="0" fillId="0" borderId="1" xfId="0" applyNumberFormat="1"><alignment horizontal="center"/></xf>
<xf numFmtId="0" fontId="2" fillId="5" borderId="0" xfId="0" applyAlignment="1"><alignment vertical="center"/></xf>
<xf numFmtId="0" fontId="0" fillId="6" borderId="1" xfId="0" applyAlignment="1"><alignment vertical="center" wrapText="1"/></xf>
<xf numFmtId="164" fontId="2" fillId="4" borderId="1" xfId="0" applyNumberFormat="1" applyAlignment="1"><alignment horizontal="right" vertical="center"/></xf>
</cellXfs>
<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>"##;

/// Emit a real Office Open XML workbook with column widths, grouping, numeric
/// values, A4 landscape fit-to-page setup, repeated headers and signature area.
pub fn write_xlsx(path: &Path, config: &ExportConfig, rows: &[ReportLine]) -> Result<(), String> {
    if !config.valid() { return Err("Phải chọn cả cột khối lượng/đơn vị và ít nhất một cột mô tả.".into()); }
    if rows.is_empty() { return Err("Không có phép đo hợp lệ để xuất XLSX.".into()); }
    if rows.iter().any(|r| !r.quantity.is_finite() || r.quantity < 0.0) { return Err("Phép đo chứa giá trị không hợp lệ.".into()); }
    if rows.len() > 150_000 { return Err("Báo cáo vượt giới hạn an toàn 150.000 phép đo; hãy chia phạm vi xuất.".into()); }
    let (worksheet, print_last) = sheet(config, rows);
    let number_picture = if config.decimals == 0 { "#,##0".to_string() } else { format!("#,##0.{}", "0".repeat(config.decimals)) };
    let number_format = format!("{number_picture};[Red]({number_picture});-");
    let styles = STYLES.replace("#,##0.000;[Red](#,##0.000);-", &number_format);
    let col = column(config.selected().len() - 1);
    let workbook = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Bang do boc" sheetId="1" r:id="rId1"/></sheets>
<definedNames><definedName name="_xlnm.Print_Titles" localSheetId="0">'Bang do boc'!$1:$6</definedName><definedName name="_xlnm.Print_Area" localSheetId="0">'Bang do boc'!$A$1:&#36;{col}&#36;{print_last}</definedName></definedNames>
</workbook>"#);
    let entries = vec![
        ("[Content_Types].xml".into(), br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/></Types>"#.to_vec()),
        ("_rels/.rels".into(), br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#.to_vec()),
        ("xl/workbook.xml".into(), workbook.into_bytes()),
        ("xl/_rels/workbook.xml.rels".into(), br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#.to_vec()),
        ("xl/styles.xml".into(), styles.into_bytes()),
        ("xl/worksheets/sheet1.xml".into(), worksheet.into_bytes()),
    ];
    let zip = zip_store(&entries)?;
    let tmp = path.with_extension("xlsx.tmp");
    let mut out = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    out.write_all(&zip).map_err(|e| e.to_string())?;
    out.sync_all().map_err(|e| e.to_string())?;
    drop(out);
    // Windows cannot rename over an existing XLSX file. Preserve the old
    // workbook in a temporary backup until the new archive is committed.
    if path.exists() {
        let backup = path.with_extension("xlsx.previous");
        if backup.exists() { std::fs::remove_file(&backup).map_err(|e| format!("Không thể thay bản sao lưu Excel: {e}"))?; }
        std::fs::rename(path, &backup).map_err(|e| format!("Không thể thay file đang mở trong Excel: {e}"))?;
        match std::fs::rename(&tmp, path) {
            Ok(()) => { let _ = std::fs::remove_file(backup); Ok(()) },
            Err(error) => {
                let restored = std::fs::rename(&backup, path).is_ok();
                Err(format!("Không thể lưu Excel mới: {error}. Khôi phục bản cũ: {restored}"))
            }
        }
    } else {
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xlsx_has_real_zip_container_and_a4_print_setup() {
        let config = ExportConfig { project: "Dự án A".into(), ..ExportConfig::default() };
        let row = ReportLine { group: "Kết cấu".into(), fields: std::array::from_fn(|i| FIELDS[i].0.into()), quantity: 3.5, unit: "m²".into() };
        let (sheet_xml, _) = sheet(&config, &[row]);
        assert!(sheet_xml.contains("orientation=\"landscape\""));
        assert!(sheet_xml.contains("fitToWidth=\"1\""));
        assert!(sheet_xml.contains("pane ySplit=\"6\""));
        assert!(sheet_xml.contains("Dự án A"));
        assert!(sheet_xml.contains("<mergeCells count=\""));
        let zip = zip_store(&[("xl/worksheets/sheet1.xml".into(), sheet_xml.into_bytes())]).unwrap();
        assert_eq!(&zip[0..4], b"PK\x03\x04");
        assert!(zip.windows(4).any(|x| x == b"PK\x05\x06"));
    }
    #[test]
    fn required_quantity_and_unit_cannot_be_hidden() {
        let mut config = ExportConfig::default();
        config.columns[8] = false;
        assert!(!config.valid());
        config.columns[8] = true;
        assert!(config.valid());
    }
    #[test]
    fn excel_xml_escapes_untrusted_descriptions() {
        assert_eq!(xml("A&B <D>"), "A&amp;B &lt;D&gt;");
        assert_eq!(column(26), "AA");
    }
}
