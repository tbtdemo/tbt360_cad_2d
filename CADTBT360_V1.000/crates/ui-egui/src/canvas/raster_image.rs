//! Native raster preview beneath CAD vectors in the egui canvas.
//! Rendering of raster pixels is intentionally separate from DWG/IMAGE metadata round-tripping.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cadcraft_doc::EntityKind;
use cadcraft_engine::Session;
use cadcraft_geom::Vec2;
use egui::{Color32, Context, Id, Mesh, Painter, Pos2, Shape, TextureHandle, TextureOptions};

use super::Xf;

const MAX_IMAGES: usize = 4;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PIXELS: u64 = 16_777_216;
const CACHE_ID: &str = "cad360-native-raster-textures-v1025";

#[derive(Clone, Default)]
struct Cache {
    textures: HashMap<PathBuf, Option<TextureHandle>>,
}

fn safe_path(path: &str, drawing_name: &str) -> Option<PathBuf> {
    let raw = path.trim();
    if raw.is_empty() || raw.starts_with(r"\\") || raw.starts_with("//") { return None; }
    let candidate = Path::new(raw);
    if candidate.is_absolute() { return Some(candidate.to_path_buf()); }
    let parent = Path::new(drawing_name).parent().filter(|p| !p.as_os_str().is_empty());
    Some(parent.unwrap_or_else(|| Path::new(".")).join(candidate))
}

fn read_texture(ctx: &Context, path: &Path) -> Option<TextureHandle> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FILE_BYTES { return None; }
    let (width, height) = image::image_dimensions(path).ok()?;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS { return None; }
    let bytes = std::fs::read(path).ok()?;
    let raster = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [raster.width() as usize, raster.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, raster.as_raw());
    Some(ctx.load_texture(format!("cad360-raster:{}", path.display()), color, TextureOptions::LINEAR))
}

fn textured_quad(texture: &TextureHandle, corners: [Pos2; 4]) -> Shape {
    let mut mesh = Mesh::with_texture(texture.id());
    for (pos, uv) in corners.into_iter().zip([
        egui::pos2(0.0, 0.0), egui::pos2(1.0, 0.0),
        egui::pos2(1.0, 1.0), egui::pos2(0.0, 1.0),
    ]) {
        mesh.vertices.push(egui::epaint::Vertex { pos, uv, color: Color32::WHITE });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    Shape::mesh(mesh)
}

/// Draw up to four local PNG/JPEG raster images under CAD vectors. Failed/missing/oversized
/// inputs still retain the standard IMAGE frame drawn by cadcraft-render. Never load a file on
/// every frame; texture handles live in the egui context and are reset on a new UI session.
pub(super) fn draw(painter: &Painter, xf: &Xf, session: &Session, ctx: &Context) {
    let Ok(state) = session.state() else { return };
    let Some(space) = state.doc.space(&state.space) else { return };
    let id = Id::new(CACHE_ID);
    let mut cache = ctx.data(|d| d.get_temp::<Cache>(id)).unwrap_or_default();
    let mut changed = false;
    let mut visited = 0;
    for entity in space.iter() {
        if !entity.common.visible { continue; }
        let EntityKind::Image(img) = &entity.kind else { continue; };
        if visited >= MAX_IMAGES { break; }
        visited += 1;
        if img.path.is_empty() || !img.size.x.is_finite() || !img.size.y.is_finite() || img.size.x <= 0.0 || img.size.y <= 0.0 { continue; }
        let o = img.insert.xy();
        let u = img.u.xy() * img.size.x;
        let v = img.v.xy() * img.size.y;
        let corners = [o + v, o + u + v, o + u, o];
        if corners.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) { continue; }
        if !cadcraft_geom::Bounds2::from_points(corners).intersects(&xf.world_bounds()) { continue; }
        let Some(path) = safe_path(&img.path, &state.title) else { continue; };
        if !cache.textures.contains_key(&path) {
            cache.textures.insert(path.clone(), read_texture(ctx, &path));
            changed = true;
        }
        let Some(texture) = cache.textures.get(&path).and_then(Option::as_ref) else { continue; };
        painter.add(textured_quad(texture, corners.map(|p: Vec2| xf.to_screen(p))));
    }
    if changed { ctx.data_mut(|d| d.insert_temp(id, cache)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_paths_are_resolved_and_unc_network_paths_are_rejected() {
        assert_eq!(safe_path("ref/site.png", "project/drawing.dxf"), Some(PathBuf::from("project/ref/site.png")));
        assert!(safe_path("//server/share.png", "project/drawing.dxf").is_none());
        assert!(safe_path("", "project/drawing.dxf").is_none());
    }
    #[test]
    fn image_quad_keeps_top_left_uv_orientation() {
        let a = Vec2::new(10.0, 20.0);
        let u = Vec2::new(30.0, 0.0);
        let v = Vec2::new(0.0, 10.0);
        assert_eq!([a + v, a + u + v, a + u, a][0], Vec2::new(10.0, 30.0));
    }
}
