//! Road earthworks by stationing and measured cross-section area.
//! Average-end-area (AEA) method: V_i = (A_i + A_(i+1)) / 2 * (Km_(i+1)-Km_i).
//! Station distances are metres along the alignment; area is measured from
//! explicit, evidence-linked CAD closed geometry (drawing units squared).
//! Cut and fill are separate. We never derive one from the other.
use super::takeoff_composite::GeometricSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarthworkKind { Cut, Fill }
impl EarthworkKind {
    pub fn label(self) -> &'static str { match self { Self::Cut => "Đào", Self::Fill => "Đắp" } }
    fn code(self) -> &'static str { match self { Self::Cut => "cut", Self::Fill => "fill" } }
}
#[derive(Clone, Debug)]
pub struct RoadCrossSection {
    pub chainage_m: f64,
    pub name: String,
    pub cut: Option<GeometricSource>,
    pub fill: Option<GeometricSource>,
    pub confirmed: bool,
}
#[derive(Clone, Debug)]
pub struct RoadInterval {
    pub start_m: f64,
    pub end_m: f64,
    pub length_m: f64,
    pub kind: EarthworkKind,
    pub area_start_m2: f64,
    pub area_end_m2: f64,
    pub volume_m3: f64,
    pub source_start: String,
    pub source_end: String,
}

pub fn station_to_metres(value: &str) -> Result<f64, String> {
    let s = value.trim().replace(',', ".");
    let n = if let Some((km, metre)) = s.split_once('+') {
        if km.contains('+') || metre.contains('+') || km.trim().is_empty() || metre.trim().is_empty() {
            return Err("Lý trình phải có dạng Km12+345.67.".into());
        }
        let km = km.trim().trim_start_matches("Km").trim_start_matches("km").trim();
        let km: u64 = km.parse().map_err(|_| "Km phải là số nguyên không âm.".to_string())?;
        let metre: f64 = metre.trim().parse().map_err(|_| "Mét sau dấu + không hợp lệ.".to_string())?;
        if !metre.is_finite() || !(0.0..1000.0).contains(&metre) {
            return Err("Phần mét trong lý trình phải từ 0 đến dưới 1000.".into());
        }
        (km as f64) * 1000.0 + metre
    } else {
        s.trim_start_matches("Km").trim_start_matches("km").trim().parse::<f64>()
            .map_err(|_| "Nhập lý trình theo mét hoặc Km12+345.67.".to_string())?
    };
    if !n.is_finite() || n < 0.0 || n > 1.0e9 { return Err("Lý trình ngoài giới hạn 0..1.000.000 km.".into()); }
    Ok(n)
}
pub fn station_label(metres: f64) -> String {
    let km = (metres / 1000.0).floor() as u64;
    format!("Km{}+{:06.2}", km, metres - km as f64 * 1000.0)
}
pub fn validate_section(section: &RoadCrossSection) -> Result<(), String> {
    if !section.chainage_m.is_finite() || section.chainage_m < 0.0 || section.chainage_m > 1.0e9 {
        return Err("Lý trình mặt cắt không hợp lệ.".into());
    }
    if section.cut.is_none() && section.fill.is_none() { return Err("Mỗi trắc ngang cần nguồn hình học diện tích đào hoặc đắp.".into()); }
    if let (Some(cut), Some(fill)) = (&section.cut, &section.fill) {
        if cut.handle == fill.handle { return Err("Không thể sử dụng cùng một vùng CAD cho cả đào và đắp.".into()); }
    }
    for source in [&section.cut, &section.fill].into_iter().flatten() {
        if source.raw_value <= 0.0 || !source.raw_value.is_finite() || source.signature.trim().is_empty() ||
            cadcraft_doc::Handle::parse_hex(&source.handle).is_none() {
            return Err("Nguồn diện tích trắc ngang bị thiếu handle/geometry hợp lệ.".into());
        }
    }
    Ok(())
}
/// Only an explicitly recorded zero-area state can be a station endpoint with
/// no cut (or no fill); it cannot be inferred from a missing/invalid CAD shape.
/// This first version requires corresponding measured area at both stations
/// and raises an error instead of silently assuming a zero area.
pub fn intervals(sections: &[RoadCrossSection], drawing_unit_to_m: f64,
    confirmed_only: bool) -> Result<Vec<RoadInterval>, String> {
    if !drawing_unit_to_m.is_finite() || drawing_unit_to_m <= 0.0 {
        return Err("Chưa hiệu chuẩn tỷ lệ hình học bản vẽ sang mét.".into());
    }
    let mut sorted = sections.to_vec();
    sorted.sort_by(|a, b| a.chainage_m.total_cmp(&b.chainage_m));
    for section in &sorted { validate_section(section)?; }
    for pair in sorted.windows(2) {
        if pair[1].chainage_m - pair[0].chainage_m <= 1.0e-6 {
            return Err("Trùng hoặc quá gần lý trình trắc ngang; không được tính thể tích hai lần.".into());
        }
    }
    let mut out = Vec::new();
    for pair in sorted.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        if confirmed_only && (!left.confirmed || !right.confirmed) { continue; }
        let length_m = right.chainage_m - left.chainage_m;
        for kind in [EarthworkKind::Cut, EarthworkKind::Fill] {
            let (a,b) = match kind { EarthworkKind::Cut => (&left.cut, &right.cut), EarthworkKind::Fill => (&left.fill, &right.fill) };
            match (a, b) {
                (None, None) => {},
                (Some(a), Some(b)) => {
                    if a.handle == b.handle {
                        return Err("Hai cọc liên tiếp không được dùng chung một đối tượng diện tích CAD.".into());
                    }
                    let area_start_m2 = a.raw_value * drawing_unit_to_m.powi(2);
                    let area_end_m2 = b.raw_value * drawing_unit_to_m.powi(2);
                    let volume_m3 = (area_start_m2 + area_end_m2) * length_m / 2.0;
                    if !volume_m3.is_finite() || volume_m3 <= 0.0 {
                        return Err("Khối lượng trắc ngang không hợp lệ hoặc tràn số.".into());
                    }
                    out.push(RoadInterval{
                        start_m:left.chainage_m,end_m:right.chainage_m,length_m,kind,
                        area_start_m2,area_end_m2,volume_m3,
                        source_start:a.handle.clone(),source_end:b.handle.clone()
                    });
                },
                _ => return Err(format!("Thiếu diện tích {} tại một đầu đoạn {} → {}. Không tự gán bằng 0.",
                    kind.label(),station_label(left.chainage_m),station_label(right.chainage_m))),
            }
        }
    }
    Ok(out)
}
/// Select only complete intervals with both surveyed endpoints in the
/// explicitly declared station range; no partial-interval interpolation.
pub fn within_range(interval: &RoadInterval, range: Option<(f64, f64)>) -> bool {
    range.is_none_or(|(start,end)|interval.start_m >= start - 1.0e-6 && interval.end_m <= end + 1.0e-6)
}
pub fn parse_range(start: &str, end: &str) -> Result<Option<(f64, f64)>, String> {
    if start.trim().is_empty() && end.trim().is_empty() { return Ok(None); }
    if start.trim().is_empty() || end.trim().is_empty() {
        return Err("Phải khai báo đủ cọc đầu và cọc cuối hoặc để trống cả hai.".into());
    }
    let from=station_to_metres(start)?;
    let to=station_to_metres(end)?;
    if to <= from + 1.0e-6 { return Err("Cọc cuối phải lớn hơn cọc đầu.".into()); }
    Ok(Some((from,to)))
}

impl RoadCrossSection {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"chainage_m":self.chainage_m,"name":self.name,
            "cut":self.cut.as_ref().map(GeometricSource::json),
            "fill":self.fill.as_ref().map(GeometricSource::json),"confirmed":self.confirmed})
    }
    pub fn read(v: &serde_json::Value) -> Option<Self> {
        let section=Self {
            chainage_m:v.get("chainage_m")?.as_f64()?,
            name:v.get("name").and_then(serde_json::Value::as_str).unwrap_or("").chars().take(180).collect(),
            cut:v.get("cut").and_then(GeometricSource::read),
            fill:v.get("fill").and_then(GeometricSource::read),
            confirmed:v.get("confirmed").and_then(serde_json::Value::as_bool).unwrap_or(false),
        };
        validate_section(&section).ok()?;
        Some(section)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn source(handle:&str,area:f64)->GeometricSource {
        GeometricSource{handle:handle.into(),signature:format!("Area:{area}"),raw_value:area}
    }
    fn section(m:f64,cut:Option<GeometricSource>,fill:Option<GeometricSource>)->RoadCrossSection {
        RoadCrossSection{chainage_m:m,name:station_label(m),cut,fill,confirmed:true}
    }
    #[test]
    fn km_notation_is_metres_and_rejects_bad_stationing() {
        assert_eq!(station_to_metres("Km12+345,50").unwrap(),12345.5);
        assert_eq!(station_to_metres("12+000").unwrap(),12000.0);
        assert_eq!(station_to_metres("1250.5").unwrap(),1250.5);
        assert_eq!(station_label(12345.5),"Km12+345.50");
        for invalid in ["12+1000","Km-1+100","3+abc","Km3+","NaN",""] {
            assert!(station_to_metres(invalid).is_err(),"{invalid}");
        }
    }
    #[test]
    fn average_end_area_road_cut_and_fill_are_separated() {
        // CAD unit = millimetre, chainages = metre; 20/30 m² cut,
        // 5/7 m² fill across 20 m gives 500 and 120 m³.
        let sections=[
            section(1000.0,Some(source("1A",20e6)),Some(source("1B",5e6))),
            section(1020.0,Some(source("2A",30e6)),Some(source("2B",7e6)))
        ];
        let result=intervals(&sections,0.001,true).unwrap();
        assert_eq!(result.len(),2);
        assert_eq!(result[0].kind,EarthworkKind::Cut);
        assert!((result[0].volume_m3 - 500.0).abs() < 1e-8);
        assert_eq!(result[1].kind,EarthworkKind::Fill);
        assert!((result[1].volume_m3 - 120.0).abs() < 1e-8);
    }
    #[test]
    fn rejects_duplicate_stations_and_missing_or_reused_cad_geometry() {
        let left=section(1000.0,Some(source("1A",20.0)),None);
        assert!(intervals(&[left.clone(),section(1000.0,Some(source("2A",22.0)),None)],1.0,true).is_err());
        assert!(intervals(&[left.clone(),section(1020.0,None,Some(source("2B",1.0)))],1.0,true).is_err());
        assert!(intervals(&[left,section(1020.0,Some(source("1A",25.0)),None)],1.0,true).is_err());
    }
    #[test]
    fn pending_cross_sections_do_not_enter_confirmed_quantities() {
        let first=section(0.0,Some(source("1A",20.0)),None);
        let mut second=section(20.0,Some(source("2A",22.0)),None);
        second.confirmed=false;
        assert_eq!(intervals(&[first.clone(),second.clone()],1.0,true).unwrap().len(),0);
        assert_eq!(intervals(&[first,second],1.0,false).unwrap().len(),1);
    }
    #[test]
    fn station_range_keeps_only_complete_intervals_not_partial_guesses() {
        let intervals=intervals(&[
            section(1000.0,Some(source("1A",10.0)),None),
            section(1020.0,Some(source("2A",20.0)),None),
            section(1040.0,Some(source("3A",30.0)),None)
        ],1.0,true).unwrap();
        let range=parse_range("Km1+020","Km1+040").unwrap();
        let selected=intervals.iter().filter(|i|within_range(i,range)).collect::<Vec<_>>();
        assert_eq!(selected.len(),1);
        assert_eq!(selected[0].start_m,1020.0);
        assert!(parse_range("Km1+020","").is_err());
        assert!(parse_range("Km1+040","Km1+020").is_err());
    }
    #[test]
    fn section_roundtrip_retains_evidence_but_not_implicit_approval() {
        let s=section(500.0,Some(source("AA",15.0)),None);
        let restored=RoadCrossSection::read(&s.json()).unwrap();
        assert_eq!(restored.chainage_m,500.0);
        assert_eq!(restored.cut.unwrap().handle,"AA");
        let unconfirmed=RoadCrossSection::read(&serde_json::json!({"chainage_m":500.0,"cut":source("AA",15.0).json()})).unwrap();
        assert!(!unconfirmed.confirmed);
    }
}
