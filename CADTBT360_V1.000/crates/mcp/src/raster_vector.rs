//! Conservative automatic raster-to-CAD line extraction for scanned plans.
//! This is pixel-based vectorization, NOT a trained vision model and not semantic BIM recognition.
//! It detects dominant horizontal/vertical dark strokes. Preview is the default;
//! a commit creates editable native LINE entities through the same engine as manual CAD.

use crate::backend::Backend;
use image::GrayImage;
use serde_json::{Value, json};
use std::path::Path;

const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_IMAGE_SIDE: u32 = 4096;
const MAX_IMAGE_PIXELS: u64 = 12_000_000;

#[derive(Clone, Copy, Debug)]
struct Stroke {
    axis: u32,
    start: u32,
    end: u32,
    last: u32,
    sum_axis: u64,
    rows: u32,
}

#[derive(Clone, Copy, Debug)]
struct Line {
    x0: f64, y0: f64, x1: f64, y1: f64,
}

fn strokes(img: &GrayImage, threshold: u8, min_length: u32, horizontal: bool) -> Vec<Line> {
    let (width, height) = img.dimensions();
    let (along, across) = if horizontal { (width, height) } else { (height, width) };
    let mut active: Vec<Stroke> = Vec::new();
    let mut finished = Vec::new();
    for axis in 0..across {
        let mut runs = Vec::new();
        let mut start = None;
        for pos in 0..=along {
            let dark = pos < along && {
                let pixel = if horizontal { img.get_pixel(pos, axis) } else { img.get_pixel(axis, pos) };
                pixel[0] <= threshold
            };
            if dark {
                if start.is_none() { start = Some(pos); }
            } else if let Some(s) = start.take() {
                if pos - s >= min_length { runs.push((s, pos - 1)); }
            }
        }
        let mut matched = vec![false; active.len()];
        for (start, end) in runs {
            let candidate = active.iter().enumerate().find(|(idx, group)| {
                !matched[*idx] && group.last + 1 == axis
                    && group.start.abs_diff(start) <= 2 && group.end.abs_diff(end) <= 2
            }).map(|(idx, _)| idx);
            if let Some(idx) = candidate {
                let g = &mut active[idx];
                g.last = axis;
                g.sum_axis += u64::from(axis);
                g.rows += 1;
                g.start = g.start.min(start);
                g.end = g.end.max(end);
                matched[idx] = true;
            } else {
                active.push(Stroke { axis, start, end, last: axis, sum_axis: u64::from(axis), rows: 1 });
                matched.push(true);
            }
        }
        let mut i = 0;
        while i < active.len() {
            if matched[i] { i += 1; }
            else { finished.push(active.swap_remove(i)); matched.swap_remove(i); }
        }
    }
    finished.extend(active);
    finished.into_iter().map(|g| {
        let center = g.sum_axis as f64 / f64::from(g.rows) + 0.5;
        let lo = f64::from(g.start) + 0.5;
        let hi = f64::from(g.end) + 0.5;
        if horizontal { Line { x0: lo, y0: center, x1: hi, y1: center } }
        else { Line { x0: center, y0: lo, x1: center, y1: hi } }
    }).collect()
}

pub fn run(b: &mut dyn Backend, args: &Value) -> Result<Value, String> {
    let path = args["path"].as_str().filter(|s| !s.trim().is_empty()).ok_or("path is required")?;
    let file = Path::new(path);
    let metadata = std::fs::metadata(file).map_err(|e| format!("Cannot inspect raster source: {e}"))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES { return Err("Raster must be a file of at most 16 MiB".into()); }
    let bytes = std::fs::read(file).map_err(|e| format!("Cannot read raster source: {e}"))?;
    let decoded = image::load_from_memory(&bytes).map_err(|e| format!("Cannot decode PNG/JPEG raster: {e}"))?;
    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 || width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS {
        return Err("Raster dimensions must be 1..4096 with at most 12 million pixels".into());
    }
    let threshold = args["threshold"].as_u64().unwrap_or(160);
    if threshold > 254 { return Err("threshold must be 0..254".into()); }
    let min_length = args["minLengthPixels"].as_u64().unwrap_or(12);
    if !(2..=4096).contains(&min_length) { return Err("minLengthPixels must be 2..4096".into()); }
    let max_lines = args["maxLines"].as_u64().unwrap_or(250);
    if !(1..=1000).contains(&max_lines) { return Err("maxLines must be 1..1000".into()); }
    let scale = args["unitsPerPixel"].as_f64().unwrap_or(1.0);
    if !scale.is_finite() || scale <= 0.0 || scale > 1.0e9 { return Err("unitsPerPixel must be positive and finite".into()); }
    let origin = args["origin"].as_array().cloned().unwrap_or_else(|| vec![json!(0), json!(0)]);
    if origin.len() != 2 { return Err("origin must be [x,y]".into()); }
    let (Some(ox), Some(oy)) = (origin[0].as_f64(), origin[1].as_f64()) else { return Err("origin coordinates must be numbers".into()); };
    if !ox.is_finite() || !oy.is_finite() || ox.abs() > 1.0e12 || oy.abs() > 1.0e12 { return Err("origin exceeds finite CAD coordinate limits".into()); }
    let preview = args["preview"].as_bool().unwrap_or(true);
    let threshold = threshold as u8;
    let gray = decoded.to_luma8();
    let mut lines = strokes(&gray, threshold, min_length as u32, true);
    lines.extend(strokes(&gray, threshold, min_length as u32, false));
    let count = lines.len();
    if count > max_lines as usize {
        return Err(format!("Found {count} strokes, exceeding maxLines={max_lines}; increase minLengthPixels or limit the input region, no CAD changes made"));
    }
    // Build and validate *all* CAD coordinates before any mutation.
    let world: Vec<Value> = lines.iter().map(|l| {
        let convert = |x: f64, y: f64| (ox + x * scale, oy + (f64::from(height) - y) * scale);
        let (x0, y0) = convert(l.x0, l.y0);
        let (x1, y1) = convert(l.x1, l.y1);
        json!([[x0,y0],[x1,y1]])
    }).collect();
    if world.iter().any(|v| v[0][0].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[0][1].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[1][0].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[1][1].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)) {
        return Err("Computed CAD coordinates are outside safety bounds; no changes made".into());
    }
    if preview {
        return Ok(json!({"preview":true,"recognition":"automatic-raster-straight-strokes","model":"deterministic-pixel-vectorization-not-neural-AI",
            "imageSize":[width,height],"strokes":count,"lines":world,"editableEntitiesCreated":0}));
    }
    // Native LINE objects, not a screenshot, image overlay, or noneditable tracing.
    let mut handles = Vec::new();
    for pts in &world {
        let response = b.call("engine.execute", json!({"command":"line","params":{"points":pts}}))?;
        handles.push(response);
    }
    Ok(json!({"preview":false,"recognition":"automatic-raster-straight-strokes",
        "imageSize":[width,height],"strokes":count,"editableEntitiesCreated":handles.len(),
        "entities":handles,"source":path,
        "caution":"Axis-aligned stroke detection only; verify output against the image before saving DWG"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raster_vector_detects_one_thick_horizontal_stroke_as_one_line() {
        let mut img = GrayImage::from_pixel(48, 32, image::Luma([255]));
        for y in 10..=12 { for x in 4..=40 { img.put_pixel(x,y,image::Luma([0])); } }
        let h = strokes(&img, 160, 12, true);
        assert_eq!(h.len(), 1);
        assert!((h[0].y0 - 11.5).abs() < 1.0e-9);
        assert_eq!(strokes(&img,160,12,false).len(), 0);
    }
}
