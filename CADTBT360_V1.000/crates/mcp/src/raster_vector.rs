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

/// Join short interruptions in collinear horizontal/vertical raster strokes.
/// Opt-in only: photographs often have genuine gaps (doors, openings), which
/// must not be closed automatically. Never merges diagonal or parallel lines.
fn bridge_collinear_gaps(lines: &mut Vec<Line>, max_gap: f64) -> usize {
    if max_gap <= 0.0 { return 0; }
    let mut merged = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let a = lines[i];
        let a_h = (a.y1 - a.y0).abs() < 1e-9 && a.x1 > a.x0;
        let a_v = (a.x1 - a.x0).abs() < 1e-9 && a.y1 > a.y0;
        if !a_h && !a_v { i += 1; continue; }
        let mut j = 0;
        let mut found = false;
        while j < lines.len() {
            if j == i { j += 1; continue; }
            let b = lines[j];
            let b_h = (b.y1 - b.y0).abs() < 1e-9 && b.x1 > b.x0;
            let b_v = (b.x1 - b.x0).abs() < 1e-9 && b.y1 > b.y0;
            let gap = if a_h && b_h && (a.y0 - b.y0).abs() < 0.25 {
                if a.x1 <= b.x0 { b.x0 - a.x1 } else if b.x1 <= a.x0 { a.x0 - b.x1 } else { 0.0 }
            } else if a_v && b_v && (a.x0 - b.x0).abs() < 0.25 {
                if a.y1 <= b.y0 { b.y0 - a.y1 } else if b.y1 <= a.y0 { a.y0 - b.y1 } else { 0.0 }
            } else { f64::INFINITY };
            if gap <= max_gap {
                if a_h {
                    lines[i] = Line { x0:a.x0.min(b.x0),y0:a.y0,x1:a.x1.max(b.x1),y1:a.y0 };
                } else {
                    lines[i] = Line { x0:a.x0,y0:a.y0.min(b.y0),x1:a.x0,y1:a.y1.max(b.y1) };
                }
                lines.remove(j);
                if j < i { i -= 1; }
                merged += 1;
                found = true;
                break;
            }
            j += 1;
        }
        if !found { i += 1; }
    }
    merged
}

#[derive(Clone, Copy, Debug)]
struct RectangleOutline {
    edges: [usize; 4],
    corners: [[f64; 2]; 4],
}

/// Identify only four complete, axis-aligned edges that meet at corners.
/// Missing walls, gaps or intersections are deliberately not inferred.
/// Detection is capped to prevent quadratic growth on large drawings.
fn closed_rectangles(lines: &[Line]) -> Vec<RectangleOutline> {
    if lines.len() > 256 { return Vec::new(); }
    let mut result = Vec::new();
    let mut used = vec![false; lines.len()];
    let near = |a: f64, b: f64| (a-b).abs() <= 1.5;
    for (top_idx, top) in lines.iter().enumerate() {
        if used[top_idx] || !near(top.y0, top.y1) || top.x1-top.x0 < 4.0 { continue; }
        for (bottom_idx, bottom) in lines.iter().enumerate().skip(top_idx+1) {
            if used[bottom_idx] || !near(bottom.y0, bottom.y1) || bottom.x1-bottom.x0 < 4.0 { continue; }
            if !near(top.x0,bottom.x0) || !near(top.x1,bottom.x1) || (bottom.y0-top.y0).abs() < 4.0 { continue; }
            let (lo_y,hi_y) = if top.y0 <= bottom.y0 { (top.y0,bottom.y0) } else { (bottom.y0,top.y0) };
            let lo_x = (top.x0+bottom.x0)/2.0;
            let hi_x = (top.x1+bottom.x1)/2.0;
            let vertical = |x: f64| lines.iter().enumerate().find(|(i,l)| {
                !used[*i] && *i != top_idx && *i != bottom_idx
                    && near(l.x0,l.x1) && near(l.x0,x)
                    && near(l.y0,lo_y) && near(l.y1,hi_y)
            }).map(|(i,_)|i);
            let (Some(left_idx),Some(right_idx)) = (vertical(lo_x),vertical(hi_x)) else { continue };
            if left_idx == right_idx { continue; }
            used[top_idx] = true; used[bottom_idx] = true;
            used[left_idx] = true; used[right_idx] = true;
            result.push(RectangleOutline {
                edges:[top_idx,bottom_idx,left_idx,right_idx],
                corners:[[lo_x,lo_y],[hi_x,lo_y],[hi_x,hi_y],[lo_x,hi_y]]
            });
            break;
        }
    }
    result
}

/// Conservative geometric parallel-edge hints; not semantic wall recognition.
/// Pair only axis-aligned segments of similar span with substantial overlap.
fn parallel_wall_candidates(lines: &[Line], min_spacing: f64, max_spacing: f64) -> Vec<Value> {
    if lines.len() > 256 || !(min_spacing > 0.0 && max_spacing >= min_spacing) { return Vec::new(); }
    let mut candidates = Vec::new();
    let horizontal = |l: &Line| (l.y0-l.y1).abs() < 0.25 && l.x1-l.x0 >= 16.0;
    let vertical = |l: &Line| (l.x0-l.x1).abs() < 0.25 && l.y1-l.y0 >= 16.0;
    for (i,a) in lines.iter().enumerate() {
        for (j,b) in lines.iter().enumerate().skip(i+1) {
            let same_h = horizontal(a) && horizontal(b);
            let same_v = vertical(a) && vertical(b);
            if !same_h && !same_v { continue; }
            let spacing = if same_h { (a.y0-b.y0).abs() } else { (a.x0-b.x0).abs() };
            if spacing < min_spacing || spacing > max_spacing { continue; }
            let (a0,a1,b0,b1) = if same_h { (a.x0,a.x1,b.x0,b.x1) } else { (a.y0,a.y1,b.y0,b.y1) };
            let start = a0.max(b0); let end = a1.min(b1);
            let shorter = (a1-a0).min(b1-b0);
            if shorter <= 0.0 || end-start < 16.0 || (end-start)/shorter < 0.8 { continue; }
            // Only a suggestion with two source indices. The user chooses whether
            // they are indeed a wall; no geometry is created or discarded here.
            candidates.push(json!({"edges":[i,j],"orientation":if same_h {"horizontal"} else {"vertical"},
                "spacingPixels":spacing,"overlapPixels":end-start}));
            if candidates.len() >= 128 { return candidates; }
        }
    }
    candidates
}

/// Construct centre lines only for unambiguous one-to-one edge pairs.
/// Existing source edges are kept; these are editable draft centre lines, not BIM walls.
fn wall_centerline_candidates(lines: &[Line], pairs: &[Value], excluded: &[bool]) -> Vec<Line> {
    let mut uses=vec![0usize;lines.len()];
    for p in pairs {
        let Some(edges)=p["edges"].as_array() else { continue };
        if edges.len()!=2 { continue; }
        if let (Some(a),Some(b))=(edges[0].as_u64(),edges[1].as_u64()) {
            if let (Some(a),Some(b))=(usize::try_from(a).ok(),usize::try_from(b).ok()) {
                if a<lines.len() && b<lines.len() { uses[a]+=1; uses[b]+=1; }
            }
        }
    }
    let mut output=Vec::new();
    for p in pairs {
        let Some(edges)=p["edges"].as_array() else { continue };
        if edges.len()!=2 { continue; }
        let (Some(a),Some(b))=(edges[0].as_u64().and_then(|v|usize::try_from(v).ok()),
                              edges[1].as_u64().and_then(|v|usize::try_from(v).ok())) else { continue };
        if a>=lines.len() || b>=lines.len() || a==b || uses[a]!=1 || uses[b]!=1
            || excluded.get(a).copied().unwrap_or(true) || excluded.get(b).copied().unwrap_or(true) { continue; }
        let (left,right)=(lines[a],lines[b]);
        let is_horizontal=(left.y0-left.y1).abs()<0.25 && (right.y0-right.y1).abs()<0.25;
        let line=if is_horizontal {
            Line{x0:left.x0.max(right.x0), y0:(left.y0+right.y0)*0.5,
                 x1:left.x1.min(right.x1), y1:(left.y0+right.y0)*0.5}
        } else {
            Line{x0:(left.x0+right.x0)*0.5,y0:left.y0.max(right.y0),
                 x1:(left.x0+right.x0)*0.5,y1:left.y1.min(right.y1)}
        };
        if (line.x1-line.x0).hypot(line.y1-line.y0)>=16.0 {output.push(line);}
    }
    output
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
    let gap_pixels = args["maxGapPixels"].as_u64().unwrap_or(0);
    if gap_pixels > 12 { return Err("maxGapPixels must be 0..12; larger gaps may represent doors or openings".into()); }
    let source_strokes = lines.len();
    let bridged_gaps = bridge_collinear_gaps(&mut lines, gap_pixels as f64);
    let wall_hints = if args["wallPairHints"].as_bool().unwrap_or(false) {
        parallel_wall_candidates(&lines, 2.0, 40.0)
    } else { Vec::new() };
    let candidates = if args["closedRectangles"].as_bool().unwrap_or(false) { closed_rectangles(&lines) } else { Vec::new() };
    let mut replacement = vec![false; lines.len()];
    for outline in &candidates { for &index in &outline.edges { replacement[index] = true; } }
    let create_axes=args["createWallCenterlines"].as_bool().unwrap_or(false);
    if create_axes && !args["wallPairHints"].as_bool().unwrap_or(false) {
        return Err("createWallCenterlines requires wallPairHints=true to review candidate pairs".into());
    }
    let axes=if create_axes {wall_centerline_candidates(&lines,&wall_hints,&replacement)} else {Vec::new()};
    let count = lines.len();
    if count > max_lines as usize {
        return Err(format!("Found {count} strokes, exceeding maxLines={max_lines}; increase minLengthPixels or limit the input region, no CAD changes made"));
    }
    // Build and validate *all* CAD coordinates before any mutation.
    let world_rectangles: Vec<Value> = candidates.iter().map(|outline| {
        let vertices: Vec<Value> = outline.corners.iter().map(|[x,y]| json!([ox + x * scale, oy + (f64::from(height)-y)*scale])).collect();
        json!(vertices)
    }).collect();
    let world_axes: Vec<Value> = axes.iter().map(|l| json!([[ox+l.x0*scale,oy+(f64::from(height)-l.y0)*scale],
        [ox+l.x1*scale,oy+(f64::from(height)-l.y1)*scale]])).collect();
    let world: Vec<Value> = lines.iter().enumerate().filter(|(i,_)| !replacement[*i]).map(|(_,l)| {
        let convert = |x: f64, y: f64| (ox + x * scale, oy + (f64::from(height) - y) * scale);
        let (x0, y0) = convert(l.x0, l.y0);
        let (x1, y1) = convert(l.x1, l.y1);
        json!([[x0,y0],[x1,y1]])
    }).collect();
    if world_axes.iter().flat_map(|v|v.as_array().into_iter().flatten()).any(|p|
        p[0].as_f64().is_none_or(|n|!n.is_finite() || n.abs()>1.0e12)
        || p[1].as_f64().is_none_or(|n|!n.is_finite() || n.abs()>1.0e12))
        || world_rectangles.iter().flat_map(|v| v.as_array().into_iter().flatten()).any(|p|
        p[0].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || p[1].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12))
        || world.iter().any(|v| v[0][0].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[0][1].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[1][0].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)
        || v[1][1].as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 1.0e12)) {
        return Err("Computed CAD coordinates are outside safety bounds; no changes made".into());
    }
    Ok(json!({"preview":true,"recognition":"automatic-raster-line-strokes","model":"deterministic-pixel-vectorization-not-neural-AI",
        "imageSize":[width,height],"strokes":count,"sourceStrokes":source_strokes,
        "bridgedGaps":bridged_gaps,"maxGapPixels":gap_pixels,
        "wallPairHints":wall_hints,"wallPairCandidateCount":wall_hints.len(),
        "wallCenterlines":world_axes,"wallCenterlinesProposed":axes.len(),
        "lines":world,"closedPolylines":world_rectangles,"rectanglesDetected":candidates.len(),
        "closedRectangles":args["closedRectangles"].as_bool().unwrap_or(false),
        "editableEntitiesCreated":0,
        "source":path,"caution":"Review vectors before committing; line recognition is not semantic building interpretation"}))
}

/// MCP entry point. The same preview function powers the desktop's AI drawing panel.
pub fn run(b: &mut dyn Backend, args: &Value) -> Result<Value, String> {
    let proposal = preview(args)?;
    if args["preview"].as_bool().unwrap_or(true) { return Ok(proposal); }
    let Some(world) = proposal["lines"].as_array() else { return Err("Raster preview has no CAD lines".to_string()) };
    let rectangles = proposal["closedPolylines"].as_array().cloned().unwrap_or_default();
    let axes=proposal["wallCenterlines"].as_array().cloned().unwrap_or_default();
    if world.is_empty() && rectangles.is_empty() && axes.is_empty() { return Err("No vector strokes detected; no CAD changes made".into()); }
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
    let operations: Vec<(&str, Value)> = rectangles.iter().map(|v| ("pline", json!({"vertices":v,"closed":true})))
        .chain(world.iter().map(|v| ("line",json!({"points":v}))))
        .chain(axes.iter().map(|v| ("line",json!({"points":v})))).collect();
    for (command_id, params) in operations {
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
        match b.call("engine.execute", json!({"command":command_id,"params":params})) {
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
        "imageSize":proposal["imageSize"],"strokes":proposal["strokes"],
        "rectanglesDetected":proposal["rectanglesDetected"],"closedPolylinesCreated":rectangles.len(),
        "wallCenterlinesCreated":axes.len(),
        "editableEntitiesCreated":handles.len(),
        "entities":handles,"source":proposal["source"],
        "caution":"Lines are grouped in one Undo action; review before saving DWG"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raster_vector_bridges_only_short_collinear_interruptions() {
        let mut image = GrayImage::from_pixel(80, 40, image::Luma([255]));
        for x in 5..=25 { image.put_pixel(x, 10, image::Luma([0])); }
        for x in 29..=55 { image.put_pixel(x, 10, image::Luma([0])); }
        let original = strokes(&image, 160, 16, true);
        assert_eq!(original.len(), 2);
        let mut disabled = original.clone();
        assert_eq!(bridge_collinear_gaps(&mut disabled, 0.0), 0);
        assert_eq!(disabled.len(), 2);
        let mut enabled = original;
        assert_eq!(bridge_collinear_gaps(&mut enabled, 4.0), 1);
        assert_eq!(enabled.len(), 1);
        assert!((enabled[0].x0 - 5.5).abs() < 1e-9);
        assert!((enabled[0].x1 - 55.5).abs() < 1e-9);
    }

    #[test]
    fn raster_vector_gap_bridging_keeps_distant_and_parallel_lines_separate() {
        let mut lines = vec![
            Line{x0:5.5,y0:10.5,x1:25.5,y1:10.5},
            Line{x0:45.5,y0:10.5,x1:65.5,y1:10.5},
            Line{x0:29.5,y0:12.5,x1:40.5,y1:12.5},
            Line{x0:5.5,y0:10.5,x1:20.5,y1:20.5},
        ];
        assert_eq!(bridge_collinear_gaps(&mut lines, 4.0), 0);
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn raster_rectangular_png_preview_and_headless_commit_create_one_closed_polyline() {
        let mut img = image::RgbImage::from_pixel(96, 72, image::Rgb([255,255,255]));
        for x in 10..=70 {
            img.put_pixel(x,10,image::Rgb([0,0,0]));
            img.put_pixel(x,50,image::Rgb([0,0,0]));
        }
        for y in 10..=50 {
            img.put_pixel(10,y,image::Rgb([0,0,0]));
            img.put_pixel(70,y,image::Rgb([0,0,0]));
        }
        let path=std::env::temp_dir().join(format!("cad360-rect-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let args=json!({"path":path.to_string_lossy(),"closedRectangles":true,"includeDiagonals":false,
            "minLengthPixels":16,"unitsPerPixel":10.0,"preview":true});
        let value=preview(&args).unwrap();
        assert_eq!(value["rectanglesDetected"],1,"must find exactly one closed rectangular outline: {value}");
        assert_eq!(value["lines"].as_array().unwrap().len(),0,"outline edges must not duplicate the PLINE");
        assert_eq!(value["closedPolylines"],json!([[[105.0,615.0],[705.0,615.0],[705.0,215.0],[105.0,215.0]]]));
        let mut backend=crate::backend::Headless::default();
        let mut commit=args;
        commit["preview"]=json!(false);
        let outcome=run(&mut backend,&commit).unwrap();
        let _=std::fs::remove_file(&path);
        assert_eq!(outcome["closedPolylinesCreated"],1);
        assert_eq!(outcome["editableEntitiesCreated"],1);
        let current=backend.call("drawing.inspect",json!({"entities":false})).unwrap();
        assert_eq!(current["entityCount"],1,"closed outline must become one editable CAD entity");
    }

    #[test]
    fn raster_rectangular_png_with_open_side_does_not_invent_wall() {
        let mut img = image::RgbImage::from_pixel(96, 72, image::Rgb([255,255,255]));
        for x in 10..=70 {
            img.put_pixel(x,10,image::Rgb([0,0,0]));
            img.put_pixel(x,50,image::Rgb([0,0,0]));
        }
        for y in 10..=50 { img.put_pixel(10,y,image::Rgb([0,0,0])); }
        // The right-hand edge is intentionally missing.
        let path=std::env::temp_dir().join(format!("cad360-open-rect-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let args=json!({"path":path.to_string_lossy(),"closedRectangles":true,"includeDiagonals":false,
            "minLengthPixels":16,"preview":true});
        let value=preview(&args).unwrap();
        let _=std::fs::remove_file(&path);
        assert_eq!(value["rectanglesDetected"],0);
        assert_eq!(value["closedPolylines"].as_array().unwrap().len(),0);
        assert_eq!(value["lines"].as_array().unwrap().len(),3,"retain three source strokes as LINE");
    }

    #[test]
    fn ai_wall_centerline_is_only_created_for_unique_parallel_pair() {
        let lines=vec![
            Line{x0:10.5,y0:10.5,x1:110.5,y1:10.5},
            Line{x0:10.5,y0:20.5,x1:110.5,y1:20.5},
        ];
        let pairs=parallel_wall_candidates(&lines,2.0,40.0);
        let center=wall_centerline_candidates(&lines,&pairs,&[false,false]);
        assert_eq!(center.len(),1);
        assert!((center[0].y0-15.5).abs()<1e-9);
        assert!((center[0].x1-center[0].x0-100.0).abs()<1e-9);
        assert!(wall_centerline_candidates(&lines,&pairs,&[true,false]).is_empty());
        let mut ambiguous=lines;
        ambiguous.push(Line{x0:10.5,y0:30.5,x1:110.5,y1:30.5});
        let possible=parallel_wall_candidates(&ambiguous,2.0,40.0);
        assert!(wall_centerline_candidates(&ambiguous,&possible,&[false;3]).is_empty(),
            "Three nearby parallel edges are ambiguous; do not invent a wall centre line");
    }

    #[test]
    fn ai_parallel_wall_candidate_hints_find_two_near_equal_parallel_lines() {
        let lines = vec![
            Line{x0:10.5,y0:10.5,x1:110.5,y1:10.5},
            Line{x0:10.5,y0:20.5,x1:110.5,y1:20.5},
            Line{x0:150.5,y0:10.5,x1:250.5,y1:10.5},
        ];
        let candidates=parallel_wall_candidates(&lines,2.0,40.0);
        assert_eq!(candidates.len(),1);
        assert_eq!(candidates[0]["edges"],json!([0,1]));
        assert_eq!(candidates[0]["spacingPixels"],10.0);
        assert_eq!(candidates[0]["overlapPixels"],100.0);
    }

    #[test]
    fn ai_parallel_wall_hints_refuse_disjoint_or_crossed_lines() {
        let lines = vec![
            Line{x0:10.5,y0:10.5,x1:100.5,y1:10.5},
            Line{x0:120.5,y0:20.5,x1:200.5,y1:20.5},
            Line{x0:50.5,y0:0.5,x1:50.5,y1:80.5},
            Line{x0:100.5,y0:0.5,x1:100.5,y1:80.5},
        ];
        assert!(parallel_wall_candidates(&lines,2.0,40.0).is_empty());
    }

    #[test]
    fn raster_closed_rectangles_recognizes_four_edges_and_refuses_missing_side() {
        let full = vec![
            Line{x0:10.5,y0:10.5,x1:70.5,y1:10.5},
            Line{x0:10.5,y0:50.5,x1:70.5,y1:50.5},
            Line{x0:10.5,y0:10.5,x1:10.5,y1:50.5},
            Line{x0:70.5,y0:10.5,x1:70.5,y1:50.5}
        ];
        let outlines=closed_rectangles(&full);
        assert_eq!(outlines.len(),1);
        assert_eq!(outlines[0].edges,[0,1,2,3]);
        assert_eq!(outlines[0].corners,[[10.5,10.5],[70.5,10.5],[70.5,50.5],[10.5,50.5]]);
        assert!(closed_rectangles(&full[..3]).is_empty(), "three edges are not a closed room");
        let mut broken=full;
        broken[3].y1=42.5;
        assert!(closed_rectangles(&broken).is_empty(), "a gap may be a doorway; never silently close it");
    }

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
    fn raster_vector_preview_real_png_has_exact_scaled_world_coordinates() {
        // The Windows Setup/Portable MCP smoke uses the same 64x32 pattern.
        // Verify the core raster decoder + pixel-to-CAD transform independently
        // so a failing CLI smoke can be localized to packaging or protocol.
        let mut img = image::RgbImage::from_pixel(64, 32, image::Rgb([255,255,255]));
        for x in 6..=56 { img.put_pixel(x, 15, image::Rgb([0,0,0])); }
        let unique = format!("cad360-raster-coordinate-{}-{}.png", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let path = std::env::temp_dir().join(unique);
        img.save(&path).expect("write real PNG fixture");
        let args = json!({"path":path.to_string_lossy(), "preview":true,
            "minLengthPixels":16,"includeDiagonals":false,"unitsPerPixel":10.0});
        let result = preview(&args);
        let _ = std::fs::remove_file(&path);
        let value = result.expect("raster preview must read real 64x32 PNG");
        assert_eq!(value["imageSize"], json!([64,32]));
        assert_eq!(value["strokes"], 1);
        assert_eq!(value["editableEntitiesCreated"], 0);
        assert_eq!(value["lines"], json!([[[65.0,165.0],[565.0,165.0]]]));
        let points = value["lines"][0].as_array().expect("one line with two endpoints");
        let x0 = points[0][0].as_f64().expect("start x");
        let x1 = points[1][0].as_f64().expect("end x");
        assert!((x1-x0-500.0).abs() < 1e-9);
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
