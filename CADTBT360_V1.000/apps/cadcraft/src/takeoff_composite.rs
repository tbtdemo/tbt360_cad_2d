//! Explicit two-stage takeoff. Never infer volumes from object counts.
//! A volume references a plan area and a vertical section segment by CAD handle
//! and geometry signature. Both must still exist unchanged when exporting.

#[derive(Clone, Debug)]
pub struct GeometricSource {
    pub handle: String,
    pub signature: String,
    pub raw_value: f64,
}
#[derive(Clone, Debug)]
pub struct PlanSectionVolume {
    pub plan: GeometricSource,
    pub section: GeometricSource,
    pub work_group: String,
    pub work_code: String,
    pub description: String,
    pub confirmed: bool,
}
/// A user must select exactly one near-vertical LINE on the cross-section.
/// This deliberately refuses slanted segments, arbitrary polylines or text
/// dimensions that could have different semantic meanings.
pub fn section_height(points: &[[f64; 2]]) -> Result<f64, String> {
    if points.len() != 2 || points.iter().flatten().any(|v| !v.is_finite()) {
        return Err("Mặt cắt phải có đúng một đoạn LINE với hai đầu mút hợp lệ.".into());
    }
    let dx = (points[1][0] - points[0][0]).abs();
    let dy = (points[1][1] - points[0][1]).abs();
    if dy <= 1.0e-10 { return Err("Chiều cao mặt cắt bằng 0, không thể bóc thể tích.".into()); }
    if dx > 0.001 * dy { return Err("Đoạn đã chọn không thẳng đứng trên mặt cắt (sai lệch X quá 0,1%).".into()); }
    Ok(dy)
}
/// Convert area in drawing units² and height in drawing units into cubic
/// units. The caller is responsible for validating the same drawing scale.
pub fn volume(area: f64, height: f64, drawing_unit_to_target: f64) -> Result<f64, String> {
    if !area.is_finite() || area <= 0.0 || !height.is_finite() || height <= 0.0
        || !drawing_unit_to_target.is_finite() || drawing_unit_to_target <= 0.0 {
        return Err("Diện tích, chiều cao hoặc tỷ lệ quy đổi không hợp lệ.".into());
    }
    let value = (area * drawing_unit_to_target * drawing_unit_to_target) * (height * drawing_unit_to_target);
    if !value.is_finite() || value <= 0.0 { return Err("Thể tích vượt phạm vi tính toán.".into()); }
    Ok(value)
}
impl GeometricSource {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"handle":self.handle,"signature":self.signature,"raw_value":self.raw_value})
    }
    pub fn read(value: &serde_json::Value) -> Option<Self> {
        let handle=value.get("handle")?.as_str()?;
        let signature=value.get("signature")?.as_str()?;
        let raw_value=value.get("raw_value")?.as_f64()?;
        if cadcraft_doc::Handle::parse_hex(handle).is_none() || signature.is_empty() || !raw_value.is_finite() || raw_value <= 0.0 { return None; }
        Some(Self{handle:handle.into(),signature:signature.into(),raw_value})
    }
}
impl PlanSectionVolume {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"plan":self.plan.json(),"section":self.section.json(),
          "work_group":self.work_group,"work_code":self.work_code,
          "description":self.description,"confirmed":self.confirmed})
    }
    pub fn read(value: &serde_json::Value) -> Option<Self> {
        let plan=GeometricSource::read(value.get("plan")?)?;
        let section=GeometricSource::read(value.get("section")?)?;
        if cadcraft_doc::Handle::parse_hex(&plan.handle) == cadcraft_doc::Handle::parse_hex(&section.handle) { return None; }
        Some(Self{
            plan, section,
            work_group:value.get("work_group").and_then(serde_json::Value::as_str).unwrap_or("").chars().take(250).collect(),
            work_code:value.get("work_code").and_then(serde_json::Value::as_str).unwrap_or("").chars().take(150).collect(),
            description:value.get("description").and_then(serde_json::Value::as_str).unwrap_or("").chars().take(1000).collect(),
            confirmed:value.get("confirmed").and_then(serde_json::Value::as_bool).unwrap_or(false),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vertical_cross_section_height_requires_unambiguous_geometry() {
        assert_eq!(section_height(&[[100.0, 0.0],[100.0, 3500.0]]).unwrap(),3500.0);
        assert!(section_height(&[[0.0, 0.0],[100.0, 3500.0]]).is_err());
        assert!(section_height(&[[0.0, 0.0],[0.0, 0.0]]).is_err());
        assert!(section_height(&[]).is_err());
    }
    #[test]
    fn volume_is_area_times_section_height_in_same_cad_units() {
        let metres = volume(24_000_000.0, 3500.0, 0.001).unwrap();
        assert!((metres - 84.0).abs() < 1e-10);
        let centimetres = volume(24_000_000.0, 3500.0, 0.1).unwrap();
        assert!((centimetres - 84_000_000.0).abs() < 1e-6);
        assert!(volume(24.0, f64::NAN, 1.0).is_err());
        assert!(volume(24.0, 3.0, 0.0).is_err());
    }
    #[test]
    fn composite_rejects_same_cad_entity_with_padded_or_lowercase_handle() {
        let plan=GeometricSource{handle:"2A".into(),signature:"Area|24".into(),raw_value:24.0};
        let section=GeometricSource{handle:"0002a".into(),signature:"Height|3".into(),raw_value:3.0};
        let saved=serde_json::json!({"plan":plan.json(),"section":section.json(),"confirmed":true});
        assert!(PlanSectionVolume::read(&saved).is_none());
        let different=GeometricSource{handle:"2B".into(),signature:"Height|3".into(),raw_value:3.0};
        assert!(PlanSectionVolume::read(&serde_json::json!({"plan":plan.json(),"section":different.json()})).is_some());
    }
    #[test]
    fn evidence_roundtrip_defaults_to_unconfirmed_not_fake_approved() {
        let source = GeometricSource {handle:"2A".into(),signature:"Area|24".into(),raw_value:24.0};
        let result = PlanSectionVolume{plan:source.clone(),section:GeometricSource{handle:"2B".into(),signature:"Length|3".into(),raw_value:3.0},
            work_group:"Kết cấu".into(),work_code:"KC.001".into(),description:"Khối bê tông".into(),confirmed:false};
        let restored=PlanSectionVolume::read(&result.json()).unwrap();
        assert!(!restored.confirmed);
        assert_eq!(restored.plan.handle,"2A");
        assert!(PlanSectionVolume::read(&serde_json::json!({"plan":source.json(),"section":source.json()})).is_none());
    }
}
