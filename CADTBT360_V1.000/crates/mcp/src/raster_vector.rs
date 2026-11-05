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

/// Detect dark strokes along the two 45-degree diagonals. Scan adjacent
/// diagonal tracks, merge runs with matching endpoints, and emit their centerline.
fn diagonal_strokes(img: &GrayImage, threshold: u8, min_length: u32, falling: bool) -> Vec<Line> {
    let (width, height) = img.dimensions();
    if width < 2 || height < 2 { return Vec::new(); }
    let mut active: Vec<Stroke> = Vec::new();
    let mut finished = Vec::new();
    // Track coordinate is y-x for descending screen lines, y+x for ascending.
    // Offset avoids signed indices in the scan loops.
    for track in 0..width + height - 1 {
        let mut runs = Vec::new();
        let mut start = None;
        for x in 0..=width {
            let y = if falling {
                i64::from(track) + i64::from(x) - i64::from(width - 1)
            } else {
                i64::from(track) - i64::from(x)
            };
            let dark = x < width && (0..i64::from(height)).contains(&y)
                && img.get_pixel(x, y as u32)[0] <= threshold;
            if dark {
                if start.is_none() { start = Some(x); }
            } else if let Some(beg) = start.take() {
                if x - beg >= min_length { runs.push((beg, x - 1)); }
            }
        }
        let mut matched = vec![false; active.len()];
        for (start, end) in runs {
            let candidate = active.iter().enumerate().find(|(idx, group)| {
                !matched[*idx] && group.last + 1 == track
                    && group.start.abs_diff(start) <= 2 && group.end.abs_diff(end) <= 2
            }).map(|(idx, _)| idx);
            if let Some(idx) = candidate {
                let g = &mut active[idx];
                g.last = track;
                g.sum_axis += u64::from(track);
                g.rows += 1;
                g.start = g.start.min(start);
                g.end = g.end.max(end);
                matched[idx] = true;
            } else {
                active.push(Stroke { axis: track, start, end, last: track, sum_axis: u64::from(track), rows: 1 });
                matched.push(true);
            }
        }
        let mut idx = 0;
        while idx < active.len() {
            if matched[idx] { idx += 1; }
            else { finished.push(active.swap_remove(idx)); matched.swap_remove(idx); }
        }
    }
    finished.extend(active);
    finished.into_iter().map(|g| {
        let t = g.sum_axis as f64 / f64::from(g.rows);
        let x0 = f64::from(g.start) + 0.5;
        let x1 = f64::from(g.end) + 0.5;
        let convert_y = |x: f64| if falling { t + x - f64::from(width - 1) } else { t - x + 1.0 };
        Line { x0, y0: convert_y(x0), x1, y1: convert_y(x1) }
    }).collect()
}

/// Extract long, isolated strokes at arbitrary angles using 8-connected
/// dark-pixel components and principal-component (PCA) line fitting. Complex
/// branched/crossed components are intentionally refused (not guessed as walls).
/// Axis-aligned and 45-degree components remain handled by the legacy scans.
fn arbitrary_angle_strokes(img: &GrayImage, threshold: u8, min_length: u32) -> Vec<Line> {
    use std::collections::VecDeque;
    let (width, height) = img.dimensions();
    let Some(n) = (width as usize).checked_mul(height as usize) else { return Vec::new() };
    let mut visited = vec![false; n];
    let mut result = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let index = y as usize * width as usize + x as usize;
            if visited[index] { continue; }
            visited[index] = true;
            if img.get_pixel(x,y)[0] > threshold { continue; }
            let mut queue = VecDeque::new();
            queue.push_back((x,y));
            let mut pts: Vec<(f64,f64)> = Vec::new();
            while let Some((cx,cy)) = queue.pop_front() {
                pts.push((f64::from(cx)+0.5, f64::from(cy)+0.5));
                for dy in -1_i32..=1 {
                    for dx in -1_i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = i64::from(cx) + i64::from(dx);
                        let ny = i64::from(cy) + i64::from(dy);
                        if nx < 0 || ny < 0 || nx >= i64::from(width) || ny >= i64::from(height) { continue; }
                        let next = ny as usize * width as usize + nx as usize;
                        if visited[next] { continue; }
                        visited[next] = true;
                        if img.get_pixel(nx as u32,ny as u32)[0] <= threshold {
                            queue.push_back((nx as u32,ny as u32));
                        }
                    }
                }
            }
            if pts.len() < min_length as usize || pts.len() > 200_000 { continue; }
            let count = pts.len() as f64;
            let (sum_x,sum_y) = pts.iter().fold((0.0,0.0),|(sx,sy),p|(sx+p.0,sy+p.1));
            let center = (sum_x/count,sum_y/count);
            let (xx,yy,xy) = pts.iter().fold((0.0,0.0,0.0),|(xx,yy,xy),p|{
                let dx=p.0-center.0; let dy=p.1-center.1;
                (xx+dx*dx, yy+dy*dy, xy+dx*dy)
            });
            let angle = 0.5 * (2.0*xy).atan2(xx-yy);
            let (ux,uy) = (angle.cos(),angle.sin());
            let mut min_t=f64::INFINITY; let mut max_t=f64::NEG_INFINITY;
            let mut max_deviation=0.0_f64;
            for (px,py) in &pts {
                let dx=*px-center.0; let dy=*py-center.1;
                let t=dx*ux+dy*uy;
                min_t=min_t.min(t); max_t=max_t.max(t);
                max_deviation=max_deviation.max((-dx*uy+dy*ux).abs());
            }
            let length=max_t-min_t;
            if length < f64::from(min_length.saturating_sub(1)) || max_deviation > 2.5 { continue; }
            // Avoid duplicate lines already recognized at 0°, 45° or 90°.
            let principal=(angle.to_degrees().rem_euclid(180.0));
            if [0.0_f64,45.0,90.0,135.0,180.0].iter().any(|v|(principal-v).abs()<5.0) { continue; }
            result.push(Line {
                x0:center.0+min_t*ux, y0:center.1+min_t*uy,
                x1:center.0+max_t*ux, y1:center.1+max_t*uy
            });
        }
    }
    result
}

/// Shared non-mutating vectorization used by MCP and the desktop GUI.
pub fn preview(args: &Value) -> Result<Value, String> {
    let path = args["path"].as_str().filter(|s| !s.trim().is_empty()).ok_or("path is required")?;
    let file = Path::new(path);
    let metadata = std::fs::metadata(file).map_err(|e| format!("Cannot inspect raster source: {e}"))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES { return Err("Raster must be a file of at most 16 MiB".into()); }
    let bytes = std::fs::read(file).map_err(|e| format!("Cannot read raster source: {e}"))?;
    // Read the image header first: rejecting oversized compressed images AFTER
    // decoding could allocate huge buffers for a small, malicious PNG/JPEG file.
    let format = image::guess_format(&bytes).map_err(|e| format!("Cannot identify PNG/JPEG format: {e}"))?;
    if !matches!(format, image::ImageFormat::Png | image::ImageFormat::Jpeg) { return Err("Only PNG/JPEG input is supported".into()); }
    let reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
    let (width, height) = reader.into_dimensions().map_err(|e| format!("Cannot read image dimensions: {e}"))?;
    if width == 0 || height == 0 || width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS {
        return Err("Raster dimensions must be 1..4096 with at most 12 million pixels".into());
    }
    let decoded = image::load_from_memory_with_format(&bytes, format).map_err(|e| format!("Cannot decode PNG/JPEG raster: {e}"))?;
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
    let threshold = threshold as u8;
    let gray = decoded.to_luma8();
    let mut lines = strokes(&gray, threshold, min_length as u32, true);
    lines.extend(strokes(&gray, threshold, min_length as u32, false));
    // Optional: angled 45° screen strokes, including the opposite slope.
    if args["includeDiagonals"].as_bool().unwrap_or(true) {
        lines.extend(diagonal_strokes(&gray, threshold, min_length as u32, true));
        lines.extend(diagonal_strokes(&gray, threshold, min_length as u32, false));
    }
    if args["includeArbitraryAngles"].as_bool().unwrap_or(false) {
        lines.extend(arbitrary_angle_strokes(&gray, threshold, min_length as u32));
    }
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
    Ok(json!({"preview":true,"recognition":"automatic-raster-line-strokes","model":"deterministic-pixel-vectorization-not-neural-AI",
        "imageSize":[width,height],"strokes":count,"lines":world,"editableEntitiesCreated":0,
        "source":path,"caution":"Review vectors before committing; line recognition is not semantic building interpretation"}))
}

/// MCP entry point. The same preview function powers the desktop's AI drawing panel.
pub fn run(b: &mut dyn Backend, args: &Value) -> Result<Value, String> {
    let proposal = preview(args)?;
    if args["preview"].as_bool().unwrap_or(true) { return Ok(proposal); }
    let Some(world) = proposal["lines"].as_array() else { return Err("Raster preview has no CAD lines".to_string()) };
    if world.is_empty() { return Err("No vector strokes detected; no CAD changes made".into()); }
    // Never end a group that another command/user already opened. The caller
    // must have a clean command prompt and full Undo support.
    let state = b.call("drawing.inspect", json!({"entities":false}))?;
    if state["undoControl"] != "all" || state["undoGroupOpen"] != false {
        return Err("AI image import requires UNDO Control All and no active Undo group".into());
    }
    let command = b.call("cmdline.state", json!({}))?;
    if command["running"].as_str().is_some() {
        return Err("Finish the active CAD command before importing image vectors".into());
    }
    let Some(document_uid) = state["documentUid"].as_u64() else { return Err("CAD document identity unavailable; no edits made".into()) };
    let Some(document_epoch) = state["documentEpoch"].as_u64() else { return Err("CAD document epoch unavailable; no edits made".into()) };
    let Some(revision) = state["revision"].as_u64() else { return Err("CAD document revision unavailable; no edits made".into()) };
    let group = |backend: &mut dyn Backend, mode: &str| backend.call("engine.execute", json!({"command":"undo","params":{"mode":mode}}));
    group(b, "begin")?;
    let mut handles = Vec::new();
    let mut failure = None;
    for pts in world {
        // Guard against an asynchronous document switch or external edit in a
        // connected GUI while the MCP agent is sending its line commands.
        let latest = b.call("drawing.inspect", json!({"entities":false}));
        match latest {
            Ok(st) if st["documentUid"].as_u64() == Some(document_uid)
                && st["documentEpoch"].as_u64() == Some(document_epoch)
                && st["revision"].as_u64() == revision.checked_add(handles.len() as u64) => {}
            Ok(_) => { failure = Some("CAD document changed during AI image import".to_string()); break; }
            Err(e) => { failure = Some(format!("Cannot verify active CAD document: {e}")); break; }
        }
        match b.call("engine.execute", json!({"command":"line","params":{"points":pts}})) {
            Ok(response) => handles.push(response),
            Err(e) => { failure = Some(e); break; }
        }
    }
    // Revalidate the target before END or rollback. If the app switched
    // drawings, END/UNDO would affect the *new* document, not the AI import.
    // If another edit was interleaved, UNDO could erase work not owned by AI.
    let ownership = b.call("drawing.inspect", json!({"entities":false}));
    let owned = ownership.as_ref().is_ok_and(|st|
        st["documentUid"].as_u64() == Some(document_uid)
        && st["documentEpoch"].as_u64() == Some(document_epoch)
        && st["revision"].as_u64() == revision.checked_add(handles.len() as u64)
        && st["undoGroupOpen"] == true
        && st["undoControl"] == "all"
    );
    if !owned {
        let detail = ownership.err().unwrap_or_else(|| "drawing switched, changed externally, or Undo group was closed".into());
        return Err(format!("AI import interrupted: {detail}. For safety, no END or UNDO was sent to the current drawing; inspect the original drawing before continuing"));
    }
    let end_result = group(b, "end");
    if let Err(err) = end_result { failure.get_or_insert(format!("Cannot close AI Undo group: {err}")); }
    if let Some(problem) = failure {
        if !handles.is_empty() {
            // Verify ownership again: another GUI request can be interleaved
            // between the END call and this rollback.
            let safe_to_undo = b.call("drawing.inspect", json!({"entities":false})).is_ok_and(|st|
                st["documentUid"].as_u64() == Some(document_uid)
                && st["documentEpoch"].as_u64() == Some(document_epoch)
                && st["revision"].as_u64() == revision.checked_add(handles.len() as u64)
                && st["undoGroupOpen"] == false
            );
            if !safe_to_undo {
                return Err(format!("AI image import failed after {} LINE entities: {problem}. Drawing changed before rollback; automatic UNDO suppressed to protect unrelated edits", handles.len()));
            }
            match b.call("engine.execute", json!({"command":"undo","params":{"count":1}})) {
                Ok(_) => return Err(format!("AI image import rolled back {} LINE entities: {problem}", handles.len())),
                Err(rollback) => return Err(format!("AI image import failed after {} LINE entities: {problem}; automatic Undo failed: {rollback}. Inspect drawing before saving", handles.len())),
            }
        }
        return Err(format!("AI image import stopped without creating LINE entities: {problem}"));
    }
    Ok(json!({"preview":false,"recognition":"automatic-raster-line-strokes",
        "imageSize":proposal["imageSize"],"strokes":proposal["strokes"],"editableEntitiesCreated":handles.len(),
        "entities":handles,"source":proposal["source"],
        "caution":"Lines are grouped in one Undo action; review before saving DWG"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raster_vector_detects_diagonal_stroke_without_manual_points() {
        let mut img = GrayImage::from_pixel(64, 64, image::Luma([255]));
        for x in 8..=48 { img.put_pixel(x,x,image::Luma([0])); }
        let lines = diagonal_strokes(&img, 160, 12, true);
        assert_eq!(lines.len(), 1, "45 degree down-right line should be recognized");
        assert!(lines[0].x1 - lines[0].x0 > 35.0);
        assert!(lines[0].y1 - lines[0].y0 > 35.0);
        assert!(diagonal_strokes(&img, 160, 12, false).is_empty());
    }

    #[test]
    fn raster_vector_detects_opposite_diagonal_stroke() {
        let mut img = GrayImage::from_pixel(64, 64, image::Luma([255]));
        for x in 8..=48 { img.put_pixel(x, 56-x, image::Luma([0])); }
        let lines = diagonal_strokes(&img, 160, 12, false);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].x1 > lines[0].x0 && lines[0].y1 < lines[0].y0);
    }
    #[test]
    fn raster_vector_detects_thirty_degree_line_with_pca() {
        let mut img = GrayImage::from_pixel(120, 120, image::Luma([255]));
        for x in 12..=100 {
            let y=20+(f64::from(x-12)*0.577350269).round() as u32;
            img.put_pixel(x,y,image::Luma([0]));
        }
        let result=arbitrary_angle_strokes(&img,160,20);
        assert_eq!(result.len(),1,"30-degree isolated stroke should become one vector");
        let l=result[0];
        assert!((l.y1-l.y0).abs()>40.0);
        assert!((l.x1-l.x0).abs()>80.0);
    }
    #[test]
    fn raster_vector_does_not_misclassify_solid_rectangle_as_thin_line() {
        let mut img=GrayImage::from_pixel(100,100,image::Luma([255]));
        for y in 20..=45 { for x in 10..=85 {img.put_pixel(x,y,image::Luma([0]));} }
        assert!(arbitrary_angle_strokes(&img,160,20).is_empty());
    }
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
