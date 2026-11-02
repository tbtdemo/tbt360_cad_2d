//! Lightweight OBJ triangle reader for IfcConvert exports.
//! Geometry is bounded to avoid uncontrolled memory use with untrusted BIM files.
use std::{io::{BufRead, BufReader}, path::Path};
const MAX_OBJ_LINE: usize = 4_000_000;

#[derive(Clone, Default)]
pub struct Mesh { pub vertices: Vec<[f32;3]>, pub faces: Vec<[usize;3]> }

fn face_index(token: &str, count: usize) -> Option<usize> {
    let value: isize = token.split('/').next()?.parse().ok()?;
    let index = if value < 0 { count.checked_sub(value.unsigned_abs())? } else { usize::try_from(value).ok()?.checked_sub(1)? };
    (index < count).then_some(index)
}

impl Mesh {
    pub fn read(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("Không đọc được OBJ: {e}"))?;
        let mut mesh = Self::default();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| e.to_string())?;
            if line.len() > MAX_OBJ_LINE { return Err("Dòng OBJ vượt giới hạn 4 MB".into()); }
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    if mesh.vertices.len() >= 1_000_000 { return Err("OBJ vượt giới hạn 1 triệu đỉnh".into()); }
                    let values: Vec<f32> = parts.take(3).map(str::parse::<f32>).collect::<Result<_,_>>().map_err(|_| "OBJ có đỉnh không hợp lệ")?;
                    if values.len() != 3 || !values.iter().all(|x| x.is_finite()) { return Err("Tọa độ OBJ không hợp lệ".into()); }
                    mesh.vertices.push([values[0],values[1],values[2]]);
                }
                Some("f") => {
                    let tokens: Vec<&str> = parts.collect();
                    if tokens.len() < 3 || tokens.len() > 128 { return Err("OBJ có mặt không hợp lệ".into()); }
                    let ids = tokens.iter().map(|v| face_index(v,mesh.vertices.len()).ok_or_else(|| "Chỉ số mặt OBJ không hợp lệ".to_string())).collect::<Result<Vec<_>,_>>()?;
                    if ids.windows(2).any(|pair| pair[0] == pair[1]) || ids[0] == *ids.last().unwrap_or(&usize::MAX) { return Err("Mặt OBJ có các đỉnh trùng nhau".into()); }
                    for i in 1..ids.len()-1 {
                        if mesh.faces.len() >= 2_000_000 { return Err("OBJ vượt giới hạn 2 triệu tam giác".into()); }
                        mesh.faces.push([ids[0],ids[i],ids[i+1]]);
                    }
                }
                _ => {}
            }
        }
        if mesh.vertices.is_empty() || mesh.faces.is_empty() { return Err("OBJ không có hình học tam giác để hiển thị".into()); }
        Ok(mesh)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_invalid_vertices_and_faces_without_panicking() {
        assert!(face_index("0", 3).is_none());
        assert!(face_index("-4", 3).is_none());
        assert!(face_index("4", 3).is_none());
        assert!(face_index("abc", 3).is_none());
    }
    #[test] fn reads_positive_and_negative_indices() {
        assert_eq!(face_index("1/2/3", 3),Some(0));
        assert_eq!(face_index("-1",3),Some(2));
        assert_eq!(face_index("0",3),None);
        assert_eq!(face_index("4",3),None);
    }
}
