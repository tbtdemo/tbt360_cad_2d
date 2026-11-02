//! GPU-ready indexed geometry with an origin rebased around the BIM model.
//! This is the upload preparation stage, NOT the wgpu draw pipeline.
use crate::ifc_mesh::Mesh;

#[derive(Clone, Debug, Default)]
pub struct Batch {
    pub origin: [f64; 3],
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub triangle_group: Vec<usize>,
}

impl Batch {
    /// Build a compact index stream for BIM visibility without losing the original triangle IDs.
    /// Selection and picking may use source_triangles to resolve IFC GlobalId/group metadata.
    pub fn visible_indices<F>(&self, mut visible: F) -> (Vec<u32>, Vec<usize>)
    where F: FnMut(usize, usize) -> bool {
        let mut indices = Vec::new();
        let mut source_triangles = Vec::new();
        for (triangle, group) in self.triangle_group.iter().copied().enumerate() {
            if !visible(triangle, group) { continue; }
            let start = triangle * 3;
            if let Some(slice) = self.indices.get(start..start + 3) {
                indices.extend_from_slice(slice);
                source_triangles.push(triangle);
            }
        }
        (indices, source_triangles)
    }

    pub fn from_mesh(mesh: &Mesh) -> Result<Self, String> {
        if mesh.vertices.is_empty() { return Err("Mesh không có đỉnh".into()); }
        if mesh.vertices.len() > u32::MAX as usize { return Err("Mesh vượt giới hạn GPU u32 indices".into()); }
        if mesh.face_groups.len() != mesh.faces.len() { return Err("Mesh thiếu identity cho các mặt OBJ".into()); }
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for p in &mesh.vertices {
            for k in 0..3 {
                let value = f64::from(p[k]);
                if !value.is_finite() { return Err("Tọa độ GPU không hữu hạn".into()); }
                lo[k] = lo[k].min(value); hi[k] = hi[k].max(value);
            }
        }
        let origin = [0, 1, 2].map(|k| lo[k] + (hi[k] - lo[k]) * 0.5);
        let mut positions = Vec::with_capacity(mesh.vertices.len());
        for p in &mesh.vertices {
            let rebased = [0, 1, 2].map(|k| (f64::from(p[k]) - origin[k]) as f32);
            if !rebased.iter().all(|value| value.is_finite()) { return Err("Tọa độ GPU sau offset không hợp lệ".into()); }
            positions.push(rebased);
        }
        let mut indices = Vec::with_capacity(mesh.faces.len().saturating_mul(3));
        for face in &mesh.faces {
            for &vertex in face {
                if vertex >= positions.len() { return Err("Face OBJ trỏ đến đỉnh không tồn tại".into()); }
                indices.push(u32::try_from(vertex).map_err(|_| "GPU index vượt giới hạn")?);
            }
        }
        Ok(Self { origin, positions, indices, triangle_group: mesh.face_groups.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_batch_rebases_large_coordinates_and_keeps_identity() {
        let mesh = Mesh { vertices: vec![[1_000_000.0,2_000_000.0,0.0], [1_000_010.0,2_000_000.0,0.0], [1_000_000.0,2_000_010.0,0.0]], faces: vec![[0,1,2]], face_groups: vec![4], groups: vec![] };
        let batch = Batch::from_mesh(&mesh).unwrap();
        assert_eq!(batch.indices, vec![0,1,2]);
        assert_eq!(batch.triangle_group, vec![4]);
        assert_eq!(batch.positions[0], [-5.0,-5.0,0.0]);
        assert_eq!(batch.positions[2], [-5.0,5.0,0.0]);
    }
    #[test]
    fn visibility_preserves_source_triangle_identity() {
        let mesh = Mesh { vertices: vec![[0.0;3],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]], faces: vec![[0,1,2],[1,3,2],[0,2,3]], face_groups: vec![0,2,2], groups: vec![] };
        let batch = Batch::from_mesh(&mesh).unwrap();
        let (indices, triangles) = batch.visible_indices(|source, group| group == 2 && source != 1);
        assert_eq!(indices, vec![0,2,3]);
        assert_eq!(triangles, vec![2]);
    }

    #[test]
    fn rejects_invalid_group_and_face_indices() {
        let mesh = Mesh { vertices: vec![[0.0;3]], faces: vec![[0,1,2]], face_groups: vec![0], groups: vec![] };
        assert!(Batch::from_mesh(&mesh).is_err());
        let mesh = Mesh { vertices: vec![[0.0;3]], faces: vec![[0,0,0]], face_groups: vec![], groups: vec![] };
        assert!(Batch::from_mesh(&mesh).is_err());
    }
}
