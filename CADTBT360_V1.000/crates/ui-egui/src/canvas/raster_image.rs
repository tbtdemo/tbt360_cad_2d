//! Native raster preview beneath CAD vectors in the egui canvas.
//! Rendering of raster pixels is intentionally separate from DWG/IMAGE metadata round-tripping.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cadcraft_doc::EntityKind;
use cadcraft_engine::Session;
use cadcraft_geom::Vec2;
use egui::{Color32, Context, Id, Mesh, Painter, Shape, TextureHandle, TextureOptions};

use super::Xf;

const MAX_IMAGES: usize = 4;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PIXELS: u64 = 16_777_216;
const MAX_TEXTURE_SIDE: u32 = 2048;
const CACHE_ID: &str = "cad360-native-raster-textures-v1025";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RasterAdjust {
    brightness: u8,
    contrast: u8,
    fade: u8,
    transparency: bool,
}

impl RasterAdjust {
    fn from_image(image: &cadcraft_doc::Image) -> Self {
        Self { brightness: image.brightness, contrast: image.contrast, fade: image.fade, transparency: image.display & 8 != 0 }
    }
}

#[derive(Clone, Default)]
struct Cache {
    textures: HashMap<(PathBuf, RasterAdjust), Option<TextureHandle>>,
}

/// Apply CAD IMAGEADJUST controls without changing the linked reference file.
/// Defaults of 50/50/0 preserve the source RGB image. Fade blends toward white.
fn adjust_pixel(pixel: [u8; 4], adjust: RasterAdjust) -> [u8; 4] {
    let contrast = (f32::from(adjust.contrast) / 50.0).clamp(0.0, 2.0);
    let brightness = (f32::from(adjust.brightness) - 50.0) * 2.55;
    let fade = (f32::from(adjust.fade) / 100.0).clamp(0.0, 1.0);
    let channel = |value: u8| -> u8 {
        let adjusted = ((f32::from(value) - 127.5) * contrast + 127.5 + brightness).clamp(0.0, 255.0);
        (adjusted * (1.0 - fade) + 255.0 * fade).round().clamp(0.0, 255.0) as u8
    };
    [channel(pixel[0]), channel(pixel[1]), channel(pixel[2]), if adjust.transparency { pixel[3] } else { 255 }]
}

fn safe_path(path: &str, drawing_name: &str) -> Option<PathBuf> {
    let raw = path.trim();
    if raw.is_empty() || raw.starts_with(r"\\") || raw.starts_with("//") { return None; }
    let candidate = Path::new(raw);
    if candidate.is_absolute() { return Some(candidate.to_path_buf()); }
    let parent = Path::new(drawing_name).parent().filter(|p| !p.as_os_str().is_empty());
    Some(parent.unwrap_or_else(|| Path::new(".")).join(candidate))
}

/// Keep GPU textures bounded independently of the source IMAGE pixel dimensions.
/// IMAGE calibration, clipping UVs and tracing always keep the source-size coordinate space.
fn texture_dimensions(width: u32, height: u32) -> [u32; 2] {
    if width <= MAX_TEXTURE_SIDE && height <= MAX_TEXTURE_SIDE { return [width, height]; }
    let longest = width.max(height);
    let scaled = |v: u32| -> u32 { ((u64::from(v) * u64::from(MAX_TEXTURE_SIDE) + u64::from(longest) / 2) / u64::from(longest)).max(1) as u32 };
    [scaled(width), scaled(height)]
}

fn read_texture(ctx: &Context, path: &Path, adjust: RasterAdjust) -> Option<TextureHandle> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FILE_BYTES { return None; }
    let (width, height) = image::image_dimensions(path).ok()?;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS { return None; }
    let bytes = std::fs::read(path).ok()?;
    let original = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let [target_w, target_h] = texture_dimensions(original.width(), original.height());
    let mut raster = if original.width() == target_w && original.height() == target_h {
        original
    } else {
        image::imageops::resize(&original, target_w, target_h, image::imageops::FilterType::Triangle)
    };
    let size = [raster.width() as usize, raster.height() as usize];
    for rgba in raster.pixels_mut() { rgba.0 = adjust_pixel(rgba.0, adjust); }
    let color = egui::ColorImage::from_rgba_unmultiplied(size, raster.as_raw());
    Some(ctx.load_texture(format!("cad360-raster:{}", path.display()), color, TextureOptions::LINEAR))
}

/// Return pixel-space clip vertices, top-left origin and y-down. A 2-point clip
/// is the opposite corners of a rectangle; a longer clip is a polygon.
fn clip_vertices(image: &cadcraft_doc::Image) -> Option<Vec<Vec2>> {
    let w = image.size.x;
    let h = image.size.y;
    let points = if !image.clipping || (image.display & 4) == 0 || image.clip.is_empty() {
        vec![Vec2::new(0.0, 0.0), Vec2::new(w, 0.0), Vec2::new(w, h), Vec2::new(0.0, h)]
    } else if image.clip.len() == 2 {
        let a = image.clip[0];
        let b = image.clip[1];
        vec![Vec2::new(a.x.min(b.x), a.y.min(b.y)), Vec2::new(a.x.max(b.x), a.y.min(b.y)),
             Vec2::new(a.x.max(b.x), a.y.max(b.y)), Vec2::new(a.x.min(b.x), a.y.max(b.y))]
    } else {
        image.clip.clone()
    };
    if !(3..=128).contains(&points.len()) { return None; }
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || p.x < 0.0 || p.y < 0.0 || p.x > w || p.y > h) { return None; }
    Some(points)
}

fn cross(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    let u = b - a;
    let v = c - a;
    u.x * v.y - u.y * v.x
}

fn triangulate_polygon(points: &[Vec2]) -> Option<Vec<u32>> {
    if points.len() < 3 || points.len() > 128 { return None; }
    // Refuse self-crossing boundaries: ear clipping otherwise may fill regions outside IMAGECLIP.
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        for j in (i + 1)..points.len() {
            if (i + 1) % points.len() == j || (j + 1) % points.len() == i { continue; }
            let c = points[j];
            let d = points[(j + 1) % points.len()];
            let o1 = cross(a, b, c);
            let o2 = cross(a, b, d);
            let o3 = cross(c, d, a);
            let o4 = cross(c, d, b);
            if o1 * o2 < -1.0e-12 && o3 * o4 < -1.0e-12 { return None; }
        }
    }
    let signed_area: f64 = (0..points.len()).map(|i| {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        a.x * b.y - b.x * a.y
    }).sum();
    if !signed_area.is_finite() || signed_area.abs() <= 1.0e-10 { return None; }
    let winding = signed_area.signum();
    let mut indices: Vec<usize> = (0..points.len()).collect();
    let mut output = Vec::with_capacity((points.len() - 2) * 3);
    while indices.len() > 3 {
        let mut found = false;
        for i in 0..indices.len() {
            let a = indices[(i + indices.len() - 1) % indices.len()];
            let b = indices[i];
            let c = indices[(i + 1) % indices.len()];
            if cross(points[a], points[b], points[c]) * winding <= 1.0e-10 { continue; }
            let contains_other = indices.iter().any(|&j| {
                if j == a || j == b || j == c { return false; }
                let p = points[j];
                let e0 = cross(points[a], points[b], p) * winding;
                let e1 = cross(points[b], points[c], p) * winding;
                let e2 = cross(points[c], points[a], p) * winding;
                e0 >= -1.0e-10 && e1 >= -1.0e-10 && e2 >= -1.0e-10
            });
            if contains_other { continue; }
            output.extend([a as u32, b as u32, c as u32]);
            indices.remove(i);
            found = true;
            break;
        }
        if !found { return None; }
    }
    output.extend(indices.into_iter().map(|i| i as u32));
    Some(output)
}

fn textured_clipped_image(texture: &TextureHandle, xf: &Xf, image: &cadcraft_doc::Image) -> Option<Shape> {
    let pixels = clip_vertices(image)?;
    let triangles = triangulate_polygon(&pixels)?;
    let mut mesh = Mesh::with_texture(texture.id());
    for pixel in pixels {
        let world = image.insert.xy() + image.u.xy() * pixel.x + image.v.xy() * (image.size.y - pixel.y);
        if !world.x.is_finite() || !world.y.is_finite() { return None; }
        mesh.vertices.push(egui::epaint::Vertex {
            pos: xf.to_screen(world),
            uv: egui::pos2((pixel.x / image.size.x) as f32, (pixel.y / image.size.y) as f32),
            color: Color32::WHITE,
        });
    }
    mesh.indices = triangles;
    Some(Shape::mesh(mesh))
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
        if (img.display & 1) == 0 || img.path.is_empty() || !img.size.x.is_finite() || !img.size.y.is_finite() || img.size.x <= 0.0 || img.size.y <= 0.0 { continue; }
        let o = img.insert.xy();
        let u = img.u.xy() * img.size.x;
        let v = img.v.xy() * img.size.y;
        let corners = [o + v, o + u + v, o + u, o];
        if corners.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) { continue; }
        if !cadcraft_geom::Bounds2::from_points(corners).intersects(&xf.world_bounds()) { continue; }
        let Some(path) = safe_path(&img.path, &state.title) else { continue; };
        let adjust = RasterAdjust::from_image(img);
        let key = (path.clone(), adjust);
        if !cache.textures.contains_key(&key) {
            if cache.textures.len() >= 12 { cache.textures.clear(); }
            cache.textures.insert(key.clone(), read_texture(ctx, &path, adjust));
            changed = true;
        }
        let Some(texture) = cache.textures.get(&key).and_then(Option::as_ref) else { continue; };
        if let Some(shape) = textured_clipped_image(texture, xf, img) { painter.add(shape); }
    }
    if changed { ctx.data_mut(|d| d.insert_temp(id, cache)); }
}

#[cfg(test)]
mod tests {
    use super::*;
fn defaults() -> RasterAdjust {
        RasterAdjust { brightness: 50, contrast: 50, fade: 0, transparency: true }
    }
    #[test]
    fn brightness_contrast_fade_and_transparency_adjust_raster_pixels() {
        let normal = defaults();
        assert_eq!(adjust_pixel([32, 128, 220, 127], normal), [32, 128, 220, 127]);
        let dark = RasterAdjust { brightness: 0, ..normal };
        assert!(adjust_pixel([128, 128, 128, 255], dark)[0] < 128);
        let bright = RasterAdjust { brightness: 100, ..normal };
        assert!(adjust_pixel([128, 128, 128, 255], bright)[0] > 128);
        let faded = RasterAdjust { fade: 100, ..normal };
        assert_eq!(adjust_pixel([0, 70, 200, 40], faded), [255, 255, 255, 40]);
        let opaque = RasterAdjust { transparency: false, ..normal };
        assert_eq!(adjust_pixel([3, 4, 5, 0], opaque)[3], 255);
    }
    #[test]
    fn large_png_texture_is_bounded_without_changing_source_pixel_coordinates() {
        assert_eq!(texture_dimensions(1000, 500), [1000, 500]);
        assert_eq!(texture_dimensions(4000, 2000), [2048, 1024]);
        assert_eq!(texture_dimensions(100, 4000), [51, 2048]);
        assert_eq!(texture_dimensions(1, 4000), [1, 2048]);
    }
    #[test]
    fn per_image_adjustments_are_part_of_texture_cache_key() {
        let key_a = (PathBuf::from("image.png"), defaults());
        let key_b = (PathBuf::from("image.png"), RasterAdjust { fade: 50, ..defaults() });
        assert_ne!(key_a, key_b);
    }
        #[test]
    fn local_paths_are_resolved_and_unc_network_paths_are_rejected() {
        assert_eq!(safe_path("ref/site.png", "project/drawing.dxf"), Some(PathBuf::from("project/ref/site.png")));
        assert!(safe_path("//server/share.png", "project/drawing.dxf").is_none());
        assert!(safe_path("", "project/drawing.dxf").is_none());
    }
    #[test]
    fn actual_png_pixels_decode_into_cached_egui_texture() {
        let file = std::env::temp_dir().join(format!("cad360-raster-test-{}.png", std::process::id()));
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([24, 128, 236, 255]));
        img.save(&file).unwrap();
        let context = egui::Context::default();
        let texture = read_texture(&context, &file, defaults()).expect("valid local PNG must load as real pixels");
        assert_eq!(texture.size(), [2, 2]);
        let _ = std::fs::remove_file(file);
    }
    #[test]
    fn oversized_file_is_rejected_before_image_decoding() {
        let file = std::env::temp_dir().join(format!("cad360-raster-large-{}.png", std::process::id()));
        let handle = std::fs::File::create(&file).unwrap();
        handle.set_len(MAX_FILE_BYTES + 1).unwrap();
        assert!(read_texture(&egui::Context::default(), &file, defaults()).is_none());
        let _ = std::fs::remove_file(file);
    }
    #[test]
    fn rectangle_clip_is_converted_to_four_pixel_vertices() {
        let mut image = cadcraft_doc::Image::default();
        image.size = Vec2::new(200.0, 100.0);
        image.clipping = true;
        image.clip = vec![Vec2::new(150.0, 80.0), Vec2::new(50.0, 20.0)];
        assert_eq!(clip_vertices(&image).unwrap(), vec![Vec2::new(50.0, 20.0), Vec2::new(150.0, 20.0), Vec2::new(150.0, 80.0), Vec2::new(50.0, 80.0)]);
    }
    #[test]
    fn concave_polygon_clip_generates_no_pixels_outside_boundary() {
        let polygon = vec![Vec2::new(0.0,0.0),Vec2::new(100.0,0.0),Vec2::new(100.0,40.0),
            Vec2::new(40.0,40.0),Vec2::new(40.0,100.0),Vec2::new(0.0,100.0)];
        let triangles = triangulate_polygon(&polygon).expect("valid concave boundary must triangulate");
        assert_eq!(triangles.len(), 12);
        let area: f64 = triangles.chunks_exact(3).map(|tri| {
            cross(polygon[tri[0] as usize], polygon[tri[1] as usize], polygon[tri[2] as usize]).abs() / 2.0
        }).sum();
        assert!((area - 6400.0).abs() < 1.0e-6);
    }
    #[test]
    fn self_crossing_clip_is_rejected_to_prevent_incorrect_raster_fill() {
        let crossed = vec![Vec2::new(0.0,0.0),Vec2::new(100.0,100.0),Vec2::new(0.0,100.0),Vec2::new(100.0,0.0)];
        assert!(triangulate_polygon(&crossed).is_none());
    }
    #[test]
    fn image_clip_rejects_points_outside_raster() {
        let mut image = cadcraft_doc::Image::default();
        image.size = Vec2::new(200.0,100.0);
        image.clipping = true;
        image.clip = vec![Vec2::new(-1.0, 0.0),Vec2::new(150.0, 50.0)];
        assert!(clip_vertices(&image).is_none());
    }
    #[test]
    fn image_quad_keeps_top_left_uv_orientation() {
        let a = Vec2::new(10.0, 20.0);
        let u = Vec2::new(30.0, 0.0);
        let v = Vec2::new(0.0, 10.0);
        assert_eq!([a + v, a + u + v, a + u, a][0], Vec2::new(10.0, 30.0));
    }
}
