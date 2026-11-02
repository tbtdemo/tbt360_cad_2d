//! Lightweight OBJ triangle reader for IfcConvert exports.
//! Geometry is bounded to avoid uncontrolled memory use with untrusted BIM files.
use std::{collections::HashMap, io::{BufRead, BufReader, Read}, path::Path};
const MAX_OBJ_LINE: usize = 4_000_000;

#[derive(Clone, Default)]
pub struct Mesh { pub vertices: Vec<[f32;3]>, pub faces: Vec<[usize;3]>, pub face_groups: Vec<usize>, pub groups: Vec<String> }

fn face_index(token: &str, count: usize) -> Option<usize> {
    let value: isize = token.split('/').next()?.parse().ok()?;
    let index = if value < 0 { count.checked_sub(value.unsigned_abs())? } else { usize::try_from(value).ok()?.checked_sub(1)? };
    (index < count).then_some(index)
}

impl Mesh {
    pub fn read(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("Không đọc được OBJ: {e}"))?;
        Self::read_from(BufReader::new(file))
    }

    fn read_from<R: BufRead>(mut reader: R) -> Result<Self, String> {
        let mut mesh = Self::default();
        mesh.groups.push("Mặc định".into());
        let mut current_group = 0usize;
        let mut group_indices = HashMap::<String, usize>::new();
        group_indices.insert("Mặc định".to_string(), 0);
        let mut line = Vec::new();
        loop {
            line.clear();
            let size = reader.by_ref().take((MAX_OBJ_LINE + 1) as u64).read_until(b'\n', &mut line).map_err(|e| e.to_string())?;
            if size == 0 { break; }
            if size > MAX_OBJ_LINE { return Err("Dòng OBJ vượt giới hạn 4 MB".into()); }
            let line = std::str::from_utf8(&line).map_err(|_| "OBJ không phải UTF-8 hợp lệ")?;
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("o" | "g") => {
                    let name = parts.collect::<Vec<_>>().join(" ");
                    if !name.is_empty() {
                        if let Some(group_index) = group_indices.get(&name) {
                            current_group = *group_index;
                        } else {
                            if mesh.groups.len() >= 100_000 {
                                return Err("OBJ vượt giới hạn 100.000 nhóm đối tượng".into());
                            }
                            current_group = mesh.groups.len();
                            group_indices.insert(name.clone(), current_group);
                            mesh.groups.push(name);
                        }
                    }
                }
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
                        let a = mesh.vertices[ids[0]]; let b = mesh.vertices[ids[i]]; let c = mesh.vertices[ids[i+1]];
                        let ab = [f64::from(b[0])-f64::from(a[0]),f64::from(b[1])-f64::from(a[1]),f64::from(b[2])-f64::from(a[2])];
                        let ac = [f64::from(c[0])-f64::from(a[0]),f64::from(c[1])-f64::from(a[1]),f64::from(c[2])-f64::from(a[2])];
                        let cross = [ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]];
                        let area_sq = cross.iter().map(|v| v*v).sum::<f64>();
                        if !area_sq.is_finite() || area_sq <= 1e-24 { continue; }
                        if mesh.faces.len() >= 2_000_000 { return Err("OBJ vượt giới hạn 2 triệu tam giác".into()); }
                        mesh.faces.push([ids[0],ids[i],ids[i+1]]);
                        mesh.face_groups.push(current_group);
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
    #[test] fn same_obj_name_reuses_group_identity() {
        let data = b"v 0 0 0\nv 1 0 0\nv 0 1 0\no Beam-1\nf 1 2 3\ng Beam-1\nf 1 2 3\n";
        let mesh = Mesh::read_from(std::io::Cursor::new(data)).unwrap();
        assert_eq!(mesh.face_groups, vec![1, 1]);
        assert_eq!(mesh.groups.len(), 2);
    }
    #[test] fn preserves_obj_group_for_every_triangulated_face() {
        let data = b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\no Beam-A\nf 1 2 3 4\ng Column-B\nf 1 3 4\n";
        let mesh = Mesh::read_from(std::io::Cursor::new(data)).unwrap();
        assert_eq!(mesh.faces.len(), 3);
        assert_eq!(mesh.face_groups.len(), mesh.faces.len());
        assert_eq!(mesh.groups[mesh.face_groups[0]], "Beam-A");
        assert_eq!(mesh.groups[mesh.face_groups[1]], "Beam-A");
        assert_eq!(mesh.groups[mesh.face_groups[2]], "Column-B");
    }
    #[test] fn parses_valid_quad_and_skips_degenerate_face() {
        let obj = b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\nf 1 2 2\n";
        // The duplicate-index face is rejected; valid quads still triangulate.
        assert!(Mesh::read_from(std::io::Cursor::new(obj)).is_err());
        let good = b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = Mesh::read_from(std::io::Cursor::new(good)).unwrap();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.faces.len(), 2);
    }
    #[test] fn skips_collinear_triangles_and_preserves_valid_faces() {
        let data = b"v 0 0 0\nv 1 0 0\nv 2 0 0\nv 0 1 0\nf 1 2 3\nf 1 2 4\n";
        let mesh = Mesh::read_from(std::io::Cursor::new(data)).unwrap();
        assert_eq!(mesh.faces.len(), 1);
        assert_eq!(mesh.faces[0], [0,1,3]);
    }
    #[test] fn handles_large_finite_coordinates_without_f32_cross_overflow() {
        let data = b"v 0 0 0\nv 100000000000000000000 0 0\nv 0 100000000000000000000 0\nf 1 2 3\n";
        let mesh = Mesh::read_from(std::io::Cursor::new(data)).unwrap();
        assert_eq!(mesh.faces.len(), 1);
    }
    #[test] fn rejects_oversized_line_before_allocating_entire_record() {
        let input = format!("v {} 0 0\n", "1".repeat(MAX_OBJ_LINE));
        assert!(Mesh::read_from(std::io::Cursor::new(input)).is_err());
    }
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
