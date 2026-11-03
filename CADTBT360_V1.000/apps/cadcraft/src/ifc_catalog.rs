//! Explicit IFC-to-OBJ metadata mapping. OBJ alone cannot guarantee IFC identity.
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Default)]
pub struct Item { pub global_id: String, pub storey: String, pub discipline: String, pub ifc_type: String }

#[derive(Clone, Debug, Default)]
pub struct Catalog { pub by_group: BTreeMap<String, Item> }

impl Catalog {
    pub fn read(path: &Path) -> Result<Self, String> {
        let len = std::fs::metadata(path).map_err(|e| format!("Không đọc được kích thước ánh xạ IFC: {e}"))?.len();
        if len == 0 || len > 16 * 1024 * 1024 { return Err("Ánh xạ IFC rỗng hoặc vượt giới hạn 16 MiB".into()); }
        let json = std::fs::read_to_string(path).map_err(|e| format!("Không đọc được ánh xạ IFC: {e}"))?;
        let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| format!("JSON ánh xạ IFC không hợp lệ: {e}"))?;
        let items = value.get("objects").and_then(|x| x.as_array()).ok_or("Cần mảng objects trong ánh xạ IFC")?;
        if items.len() > 100_000 { return Err("Quá 100.000 đối tượng trong ánh xạ IFC".into()); }
        let mut by_group = BTreeMap::new();
        for item in items {
            let field = |name| item.get(name).and_then(|v| v.as_str()).unwrap_or("").trim();
            let group = field("obj_group");
            let guid = field("global_id");
            if group.is_empty() || group.len() > 512 { return Err("Tên nhóm OBJ trống hoặc quá dài".into()); }
            if guid.len() != 22 || !guid.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'$') { return Err(format!("GlobalId IFC không hợp lệ tại nhóm {group}")); }
            if ["storey", "discipline", "ifc_type"].iter().any(|name| field(name).len() > 256) { return Err(format!("Metadata IFC quá dài tại nhóm {group}")); }
            let entry = Item { global_id: guid.into(), storey: field("storey").into(), discipline: field("discipline").into(), ifc_type: field("ifc_type").into() };
            if by_group.insert(group.to_string(), entry).is_some() { return Err(format!("Trùng ánh xạ nhóm OBJ: {group}")); }
        }
        Ok(Self { by_group })
    }

    pub fn get(&self, group: &str) -> Option<&Item> { self.by_group.get(group) }
}

pub fn matches(item: Option<&Item>, guid: &str, storey: &str, discipline: &str) -> bool {
    if guid.is_empty() && storey.is_empty() && discipline.is_empty() { return true; }
    item.is_some_and(|x| (guid.is_empty() || x.global_id.contains(guid)) && (storey.is_empty() || x.storey == storey) && (discipline.is_empty() || x.discipline == discipline))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn strict_filters_exclude_unmapped_geometry() {
        let item = Item { global_id: "3h7B1JG5D4QQ4W8gcZQ0Xa".into(), storey: "Tầng 2".into(), discipline: "Kết cấu".into(), ifc_type: "IfcBeam".into() };
        assert!(matches(Some(&item), "3h7B", "Tầng 2", "Kết cấu"));
        assert!(!matches(Some(&item), "", "Tầng 1", ""));
        assert!(!matches(None, "3h7B", "", ""));
        assert!(matches(None, "", "", ""));
    }
}
