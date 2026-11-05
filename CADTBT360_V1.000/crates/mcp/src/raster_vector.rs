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

/// A conservative column-like motif: a small near-square closed outline with
/// two INTERIOR orthogonal hatch strokes crossing one another. Ordinary empty
/// rectangles, large rooms, wall intersections and ambiguous hatches are not
/// column evidence; this never classifies a structural element.
fn column_hatch_candidates(rects: &[RectangleOutline], lines: &[Line]) -> Vec<Value> {
    if rects.len()>128 || lines.len()>512 {return Vec::new();}
    let mut results=Vec::new();
    for rect in rects {
        let [[x0,y0],[x1,_],[_,y1],_]=rect.corners;
        let w=x1-x0;let h=y1-y0;
        if !(12.0..=120.0).contains(&w) || !(12.0..=120.0).contains(&h) || !(0.60..=1.65).contains(&(w/h)) {continue;}
        let mid_x=(x0+x1)*0.5;let mid_y=(y0+y1)*0.5;
        let mut horizontal=Vec::new();let mut vertical=Vec::new();
        for (i,l) in lines.iter().enumerate() {
            if rect.edges.contains(&i) {continue;}
            let lx0=l.x0.min(l.x1);let lx1=l.x0.max(l.x1);
            let ly0=l.y0.min(l.y1);let ly1=l.y0.max(l.y1);
            let horiz=(l.y0-l.y1).abs()<=0.75 && (l.y0-mid_y).abs()<=2.0
                && lx0>x0+2.0 && lx1<x1-2.0 && lx1-lx0>=w*0.28;
            let vert=(l.x0-l.x1).abs()<=0.75 && (l.x0-mid_x).abs()<=2.0
                && ly0>y0+2.0 && ly1<y1-2.0 && ly1-ly0>=h*0.28;
            if horiz {horizontal.push((i,lx0,lx1,(l.y0+l.y1)*0.5));}
            if vert {vertical.push((i,ly0,ly1,(l.x0+l.x1)*0.5));}
        }
        // Exactly one horizontal and vertical internal hatch stroke must cross.
        if horizontal.len()!=1 || vertical.len()!=1 {continue;}
        let (hi,hlo,hhi,hy)=horizontal[0];let (vi,vlo,vhi,vx)=vertical[0];
        if vx<hlo || vx>hhi || hy<vlo || hy>vhi {continue;}
        results.push(json!({"outlinePixels":rect.corners,
            "sourceOutlineEdges":rect.edges,"sourceHatchEdges":[hi,vi],
            "widthPixels":w,"heightPixels":h,
            "classification":"closed-cross-hatched-column-like-outline-unverified",
            "requiresHumanReview":true}));
        if results.len()>=64 {break;}
    }
    results
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

/// Detect crossings and T-junctions among already proposed wall centre lines.
/// This is a Preview-only QA hint; no lines are extended, trimmed or modified.
/// A margin ensures tiny endpoint noise is not classified as a true junction.
fn wall_centerline_junctions(axes: &[Line]) -> Vec<Value> {
    if axes.len() > 128 { return Vec::new(); }
    let horizontal = |l: &Line| (l.y0-l.y1).abs() <= 0.25 && l.x1-l.x0 >= 16.0;
    let vertical = |l: &Line| (l.x0-l.x1).abs() <= 0.25 && l.y1-l.y0 >= 16.0;
    let mut result=Vec::new();
    for (i,a) in axes.iter().enumerate() {
        for (j,b) in axes.iter().enumerate().skip(i+1) {
            let (h,v)=if horizontal(a) && vertical(b) {(a,b)}
                else if vertical(a) && horizontal(b) {(b,a)}
                else {continue};
            let x=v.x0; let y=h.y0;
            let margin=1.0;
            if x < h.x0-margin || x > h.x1+margin || y < v.y0-margin || y > v.y1+margin {continue;}
            let h_end=(x-h.x0).abs()<=margin || (x-h.x1).abs()<=margin;
            let v_end=(y-v.y0).abs()<=margin || (y-v.y1).abs()<=margin;
            if h_end && v_end {continue;} // two tips touching are not a guaranteed junction
            result.push(json!({"centerlineIndices":[i,j],"pointPixels":[x,y],
                "kind":if h_end || v_end {"T-candidate"} else {"cross-candidate"}}));
            if result.len()>=128 {return result;}
        }
    }
    result
}

/// Report short interruptions as possible openings only; no auto-bridge.
/// A blank span may be a door, window, damaged scan or drafting break.
fn opening_gap_candidates(lines: &[Line], min_gap: f64, max_gap: f64) -> Vec<Value> {
    if lines.len()>256 {return Vec::new();}
    let mut gaps=Vec::new();
    for (i,a) in lines.iter().enumerate() {
        for (j,b) in lines.iter().enumerate().skip(i+1) {
            let horiz=(a.y1-a.y0).abs()<0.25 && (b.y1-b.y0).abs()<0.25 && (a.y0-b.y0).abs()<0.75;
            let vert=(a.x1-a.x0).abs()<0.25 && (b.x1-b.x0).abs()<0.25 && (a.x0-b.x0).abs()<0.75;
            if !horiz && !vert {continue;}
            let (a0,a1,b0,b1)=if horiz {(a.x0,a.x1,b.x0,b.x1)} else {(a.y0,a.y1,b.y0,b.y1)};
            if a1-a0<16.0 || b1-b0<16.0 {continue;}
            let (begin,end)=if a1<b0 {(a1,b0)} else if b1<a0 {(b1,a0)} else {continue};
            let gap=end-begin;
            if gap<min_gap || gap>max_gap {continue;}
            let fixed=if horiz {(a.y0+b.y0)*0.5} else {(a.x0+b.x0)*0.5};
            // Do not suggest a clean opening if other extracted geometry actually
            // occupies the candidate span (a third fragment or crossing wall).
            // This avoids false "door" hints in crowded architectural scans.
            let obstructed=lines.iter().enumerate().any(|(k,other)| {
                if k==i || k==j {return false;}
                if horiz {
                    let same=(other.y0-fixed).abs()<0.75 && (other.y1-fixed).abs()<0.75
                        && other.x0<end-0.75 && other.x1>begin+0.75;
                    let crosses=(other.x0-other.x1).abs()<0.75
                        && other.x0>begin+0.75 && other.x0<end-0.75
                        && other.y0<=fixed+0.75 && other.y1>=fixed-0.75;
                    same || crosses
                } else {
                    let same=(other.x0-fixed).abs()<0.75 && (other.x1-fixed).abs()<0.75
                        && other.y0<end-0.75 && other.y1>begin+0.75;
                    let crosses=(other.y0-other.y1).abs()<0.75
                        && other.y0>begin+0.75 && other.y0<end-0.75
                        && other.x0<=fixed+0.75 && other.x1>=fixed-0.75;
                    same || crosses
                }
            });
            if obstructed {continue;}
            let from=if horiz {json!([begin,fixed])} else {json!([fixed,begin])};
            let to=if horiz {json!([end,fixed])} else {json!([fixed,end])};
            // Duplicate raster strokes can propose one physical gap repeatedly.
            // Deduplicate geometrically, but keep sourceEdges for traceability.
            let duplicate=gaps.iter().any(|prior: &Value| {
                let p=&prior["fromPixels"]; let q=&prior["toPixels"];
                let near=|a:&Value,b:&Value| a.as_f64().zip(b.as_f64()).is_some_and(|(x,y)|(x-y).abs()<=0.75);
                near(&p[0],&from[0]) && near(&p[1],&from[1])
                    && near(&q[0],&to[0]) && near(&q[1],&to[1])
            });
            if duplicate {continue;}
            gaps.push(json!({"sourceEdges":[i,j],"gapPixels":gap,
                "fromPixels":from,"toPixels":to,
                "classification":"unverified-gap-not-door"}));
            if gaps.len()==128 {return gaps;}
        }
    }
    gaps
}

/// Pair two aligned gaps on nearby parallel wall edges. A single interrupted
/// stroke is insufficient evidence of a through-wall opening. These are geometric
/// candidate guides, not classified doors/windows or automatically inferred walls.
fn paired_wall_openings(gaps: &[Value], min_spacing: f64, max_spacing: f64) -> Vec<Value> {
    if gaps.len() > 128 || min_spacing < 1.0 || max_spacing < min_spacing { return Vec::new(); }
    let mut possibilities: Vec<(usize, usize, [[f64; 2]; 4], f64)> = Vec::new();
    for (i, a) in gaps.iter().enumerate() {
        for (j, b) in gaps.iter().enumerate().skip(i + 1) {
            let read = |v: &Value, key: &str| -> Option<[f64; 2]> {
                let p = &v[key];
                Some([p[0].as_f64()?, p[1].as_f64()?])
            };
            let (Some(a0), Some(a1), Some(b0), Some(b1)) =
                (read(a,"fromPixels"), read(a,"toPixels"), read(b,"fromPixels"), read(b,"toPixels")) else { continue };
            let ah = (a0[1]-a1[1]).abs() <= 0.75;
            let bh = (b0[1]-b1[1]).abs() <= 0.75;
            let av = (a0[0]-a1[0]).abs() <= 0.75;
            let bv = (b0[0]-b1[0]).abs() <= 0.75;
            if !(ah && bh || av && bv) { continue; }
            let (spacing, start_error, end_error) = if ah && bh {
                ((a0[1]-b0[1]).abs(), (a0[0]-b0[0]).abs(), (a1[0]-b1[0]).abs())
            } else {
                ((a0[0]-b0[0]).abs(), (a0[1]-b0[1]).abs(), (a1[1]-b1[1]).abs())
            };
            if spacing < min_spacing || spacing > max_spacing || start_error > 1.5 || end_error > 1.5 { continue; }
            let width_a=(a1[0]-a0[0]).hypot(a1[1]-a0[1]);
            let width_b=(b1[0]-b0[0]).hypot(b1[1]-b0[1]);
            if width_a < 3.0 || (width_a-width_b).abs() > 1.5 { continue; }
            // Avoid pairing gaps from the same physical scan edge.
            let vertices=[a0,a1,b1,b0];
            possibilities.push((i,j,vertices,spacing));
        }
    }
    // Do not guess where one raster gap could pair with multiple other edges.
    let mut counts=vec![0usize;gaps.len()];
    for (i,j,_,_) in &possibilities { counts[*i]+=1; counts[*j]+=1; }
    possibilities.into_iter().filter(|(i,j,_,_)|counts[*i]==1 && counts[*j]==1)
        .take(64).map(|(i,j,outline,spacing)|json!({
            "sourceGapIndices":[i,j],"outlinePixels":outline,
            "wallThicknessPixels":spacing,"classification":"paired-through-wall-gap-unverified",
            "requiresHumanReview":true
        })).collect()
}

/// Add observable, axis-aligned door-leaf-like evidence to geometric opening candidates.
/// A line from one end of an opening extending perpendicular into the room may be
/// a door leaf, but is NOT proof of a door: no semantic door block is created.
/// If more than one stroke matches, the result is deliberately ambiguous.
fn door_leaf_evidence(openings: &mut [Value], lines: &[Line]) {
    if lines.len() > 1000 { return; }
    for candidate in openings {
        let Some(corners) = candidate["outlinePixels"].as_array() else {continue};
        if corners.len() != 4 {continue;}
        let read = |idx: usize| -> Option<[f64;2]> {
            Some([corners[idx][0].as_f64()?, corners[idx][1].as_f64()?])
        };
        let (Some(a),Some(b),Some(c),Some(d))=(read(0),read(1),read(2),read(3)) else {continue};
        let horizontal=(a[1]-b[1]).abs()<=0.75 && (c[1]-d[1]).abs()<=0.75;
        let vertical=(a[0]-b[0]).abs()<=0.75 && (c[0]-d[0]).abs()<=0.75;
        if !horizontal && !vertical {continue;}
        let width=if horizontal {(b[0]-a[0]).abs()} else {(b[1]-a[1]).abs()};
        let thickness=candidate["wallThicknessPixels"].as_f64().unwrap_or(0.0);
        if width<4.0 || thickness<=0.0 {continue;}
        let mut matching=Vec::new();
        for (index,l) in lines.iter().enumerate() {
            let (lo,hi,coord) = if horizontal {
                if (l.x0-l.x1).abs()>0.75 {continue;}
                (l.y0.min(l.y1),l.y0.max(l.y1), (l.x0+l.x1)*0.5)
            } else {
                if (l.y0-l.y1).abs()>0.75 {continue;}
                (l.x0.min(l.x1),l.x0.max(l.x1), (l.y0+l.y1)*0.5)
            };
            let len=hi-lo;
            if len<width*0.65 || len>width*1.35 || len<=thickness+1.0 {continue;}
            let corners_xy=[a,b,c,d];
            let hinge=corners_xy.iter().find(|p| {
                let along=if horizontal {p[0]} else {p[1]};
                let at=if horizontal {p[1]} else {p[0]};
                // The leaf starts at an opening's corner and extends OUT of the
                // wall thickness. A short jamb spanning the wall is not enough.
                (coord-along).abs()<=1.25 && ((lo-at).abs()<=1.25 || (hi-at).abs()<=1.25)
                    && {
                        let wall_lo=if horizontal {a[1].min(d[1])} else {a[0].min(b[0])};
                        let wall_hi=if horizontal {a[1].max(d[1])} else {a[0].max(b[0])};
                        lo < wall_lo-1.0 || hi > wall_hi+1.0
                    }
            });
            if let Some(p)=hinge {
                matching.push(json!({"sourceLineIndex":index,"hingePixels":p,"leafLengthPixels":len,
                    "classification":"possible-door-leaf-stroke-unverified"}));
            }
        }
        if matching.len()==1 {
            candidate["doorLeafEvidence"]=matching.remove(0);
            candidate["doorLeafCandidate"]=json!(true);
        } else {
            candidate["doorLeafCandidate"]=json!(false);
            if matching.len()>1 {candidate["doorLeafAmbiguous"]=json!(true);}
        }
    }
}

/// Recognize two short parallel sash strokes fully inside a paired wall-edge gap.
/// This is an optical drafting clue, NOT a verified architectural window.
fn window_sash_evidence(openings: &mut [Value], lines: &[Line]) {
    if lines.len()>1000 {return;}
    for candidate in openings {
        if candidate["doorLeafCandidate"]==true || candidate["doorLeafAmbiguous"]==true {continue;}
        let Some(corners)=candidate["outlinePixels"].as_array() else {continue};
        if corners.len()!=4 {continue;}
        let read=|idx:usize|->Option<[f64;2]> {
            Some([corners[idx][0].as_f64()?,corners[idx][1].as_f64()?])
        };
        let (Some(a),Some(b),Some(c),Some(d))=(read(0),read(1),read(2),read(3)) else {continue};
        let horizontal=(a[1]-b[1]).abs()<=0.75 && (c[1]-d[1]).abs()<=0.75;
        let vertical=(a[0]-b[0]).abs()<=0.75 && (c[0]-d[0]).abs()<=0.75;
        if !horizontal && !vertical {continue;}
        let (along_min,along_max,cross_min,cross_max)=if horizontal {
            (a[0].min(b[0]),a[0].max(b[0]),a[1].min(d[1]),a[1].max(d[1]))
        } else {
            (a[1].min(b[1]),a[1].max(b[1]),a[0].min(b[0]),a[0].max(b[0]))
        };
        let width=along_max-along_min;let thickness=cross_max-cross_min;
        if width<5.0 || thickness<4.0 {continue;}
        let mut sash:Vec<(usize,f64)>=Vec::new();
        for (idx,l) in lines.iter().enumerate() {
            let (lo,hi,cross)=if horizontal {
                if (l.y0-l.y1).abs()>0.75 {continue;}
                (l.x0.min(l.x1),l.x0.max(l.x1),(l.y0+l.y1)*0.5)
            } else {
                if (l.x0-l.x1).abs()>0.75 {continue;}
                (l.y0.min(l.y1),l.y0.max(l.y1),(l.x0+l.x1)*0.5)
            };
            // Only near-complete sash strokes INSIDE the wall opening count.
            if cross<=cross_min+1.0 || cross>=cross_max-1.0 {continue;}
            if (lo-along_min).abs()>1.5 || (hi-along_max).abs()>1.5 {continue;}
            if (hi-lo-width).abs()>1.5 {continue;}
            sash.push((idx,cross));
        }
        // Two clean, separated internal rails are evidence; three or more are
        // ambiguous and a single stroke could simply be a dimension line.
        if sash.len()!=2 || (sash[0].1-sash[1].1).abs()<1.0 {continue;}
        candidate["windowSashCandidate"]=json!(true);
        candidate["windowSashEvidence"]=json!({
            "sourceLineIndices":[sash[0].0,sash[1].0],
            "railSpacingPixels":(sash[0].1-sash[1].1).abs(),
            "classification":"possible-window-sash-rails-unverified",
            "requiresHumanReview":true
        });
    }
}

/// Draw a quarter-circle swing drafting guide only for a single unambiguous
/// observed leaf anchored to a geometrically paired wall opening. The source
/// LINE is retained; the ARC is not a semantic door/block.
fn door_swing_arcs(openings: &[Value], lines: &[Line], ox: f64, oy: f64, height: u32, scale: f64) -> Vec<Value> {
    let mut arcs=Vec::new();
    for opening in openings {
        if opening["doorLeafCandidate"]!=true || opening["doorLeafAmbiguous"]==true {continue;}
        let leaf=&opening["doorLeafEvidence"];
        let Some(index)=leaf["sourceLineIndex"].as_u64().and_then(|v|usize::try_from(v).ok()) else {continue};
        let Some(line)=lines.get(index) else {continue};
        let (Some(hx),Some(hy))=(leaf["hingePixels"][0].as_f64(),leaf["hingePixels"][1].as_f64()) else {continue};
        let Some(corners)=opening["outlinePixels"].as_array() else {continue};
        let width=opening["outlinePixels"][0][0].as_f64().zip(opening["outlinePixels"][1][0].as_f64())
            .zip(opening["outlinePixels"][0][1].as_f64().zip(opening["outlinePixels"][1][1].as_f64()))
            .map(|((x0,x1),(y0,y1))|(x1-x0).hypot(y1-y0)).unwrap_or(0.0);
        if width<=0.0 || !width.is_finite() {continue;}
        let ends=[[line.x0,line.y0],[line.x1,line.y1]];
        let dist=|p:[f64;2]|(p[0]-hx).hypot(p[1]-hy);
        let leaf_end=if dist(ends[0])<=1.5 {ends[1]} else if dist(ends[1])<=1.5 {ends[0]} else {continue};
        let leaf_length=dist(leaf_end);
        if (leaf_length-width).abs() > (width*0.10).max(1.5) {continue;}
        // The opening's other endpoint must be on the same edge as the hinge.
        let target=corners.iter().filter_map(|p|Some([p[0].as_f64()?,p[1].as_f64()?]))
            .filter(|p|dist(*p)>width*0.7 && (dist(*p)-width).abs()<=1.5)
            .find(|p|{
                let vx=p[0]-hx; let vy=p[1]-hy;
                (vx*(leaf_end[0]-hx)+vy*(leaf_end[1]-hy)).abs()<=width*width*0.10
            });
        let Some(target)=target else {continue};
        let hcad=[ox+hx*scale,oy+(f64::from(height)-hy)*scale];
        if hcad.iter().any(|v|!v.is_finite()||v.abs()>1.0e12) {continue;}
        let angle=|p:[f64;2]|(hy-p[1]).atan2(p[0]-hx).to_degrees().rem_euclid(360.0);
        let a=angle(leaf_end); let b=angle(target);
        let delta=(b-a).rem_euclid(360.0);
        let (start,end)=if (delta-90.0).abs()<=12.0 {(a,b)}
            else if (delta-270.0).abs()<=12.0 {(b,a)}
            else {continue};
        let radius=width*scale;
        if !radius.is_finite() || radius<=1e-9 || radius>1.0e12 {continue;}
        let opposite=[ox+target[0]*scale,oy+(f64::from(height)-target[1])*scale];
        // Stable within the original source image and recognition settings, independent
        // of world origin/scale and subsequent reviewed Block width/hinge/swing changes.
        let source_key=format!("{:.3},{:.3}:{:.3},{:.3}:{}",hx,hy,target[0],target[1],index);
        arcs.push(json!({"center":hcad,"radius":radius,"start":start,"end":end,
            "oppositeHingeCAD":opposite,"leafAngleDegrees":a,"openingAngleDegrees":b,
            "sourceDoorKey":source_key,"sourceLineIndex":index,
            "classification":"confirmed-drafting-arc-not-door-block"}));
    }
    arcs
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
    // Missing calibration is explicitly documented as 1 CAD unit/pixel; malformed supplied
    // calibration must never silently reset to 1.0 and produce geometry at the wrong scale.
    let scale = match args.get("unitsPerPixel") {
        None => 1.0,
        Some(value) => value.as_f64().ok_or("unitsPerPixel must be a finite positive number; no CAD changes made")?,
    };
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
    // Reject malformed optional values instead of silently interpreting e.g. -1 or "4" as zero.
    // Zero disables bridging and must be intentional, especially around possible door openings.
    let gap_pixels = match args.get("maxGapPixels") {
        None => 0,
        Some(value) => value.as_u64().ok_or("maxGapPixels must be an integer from 0..12; no CAD changes made")?,
    };
    if gap_pixels > 12 { return Err("maxGapPixels must be 0..12; larger gaps may represent doors or openings".into()); }
    // Never display a possible doorway and simultaneously fill the same span.
    // Reject contradictory options before any editable geometry is proposed.
    if args["openingGapHints"].as_bool().unwrap_or(false) && gap_pixels > 0 {
        return Err("openingGapHints=true requires maxGapPixels=0 to preserve possible door/opening gaps; no CAD changes made".into());
    }
    let gap_bound = |key: &str, default: f64| -> Result<f64, String> {
        match args.get(key) {
            None => Ok(default),
            Some(v) => v.as_f64().ok_or_else(|| format!("{key}: opening gap must be a number")),
        }
    };
    let opening_min=gap_bound("minOpeningGapPixels",3.0)?;
    let opening_max=gap_bound("maxOpeningGapPixels",40.0)?;
    if !opening_min.is_finite() || !opening_max.is_finite() || opening_min < 1.0
        || opening_max > 120.0 || opening_min > opening_max {
        return Err("Opening gap pixels must satisfy 1 <= minimum <= maximum <= 120; no CAD changes made".into());
    }
    let mut gaps_hint=if args["openingGapHints"].as_bool().unwrap_or(false) {
        opening_gap_candidates(&lines,opening_min,opening_max)
    } else {Vec::new()};
    // Give the user measurable CAD dimensions in addition to pixel-space hints.
    // These values never create a door, close a gap or alter the source lines.
    for gap in &mut gaps_hint {
        let from=&gap["fromPixels"];
        let to=&gap["toPixels"];
        let convert=|p: &Value| -> Option<Value> {
            let (x,y)=(p[0].as_f64()?,p[1].as_f64()?);
            let (wx,wy)=(ox+x*scale,oy+(f64::from(height)-y)*scale);
            if !wx.is_finite() || !wy.is_finite() || wx.abs()>1.0e12 || wy.abs()>1.0e12 {return None;}
            Some(json!([wx,wy]))
        };
        let (Some(from_cad),Some(to_cad))=(convert(from),convert(to)) else {
            return Err("Opening gap CAD coordinates exceed safe bounds; no CAD changes made".into());
        };
        let width=gap["gapPixels"].as_f64().unwrap_or(0.0)*scale;
        if !width.is_finite() || width>1.0e12 {
            return Err("Opening gap CAD width exceeds safe bounds; no CAD changes made".into());
        }
        gap["fromCAD"]=from_cad;
        gap["toCAD"]=to_cad;
        gap["widthCAD"]=json!(width);
    }
    let paired_enabled=args["pairedWallOpeningHints"].as_bool().unwrap_or(false);
    if paired_enabled && !args["openingGapHints"].as_bool().unwrap_or(false) {
        return Err("pairedWallOpeningHints requires openingGapHints=true; no CAD changes made".into());
    }
    let mut paired_openings=if paired_enabled {
        paired_wall_openings(&gaps_hint,2.0,40.0)
    } else {Vec::new()};
    if args["doorLeafHints"].as_bool().unwrap_or(false) {
        if !paired_enabled {return Err("doorLeafHints requires pairedWallOpeningHints=true; no CAD changes made".into());}
        door_leaf_evidence(&mut paired_openings,&lines);
    }
    if args["windowSashHints"].as_bool().unwrap_or(false) {
        if !paired_enabled {return Err("windowSashHints requires pairedWallOpeningHints=true; no CAD changes made".into());}
        // Always check for competing door leaves before proposing a window sash,
        // even when the door-leaf UI option is switched off.
        if !args["doorLeafHints"].as_bool().unwrap_or(false) {door_leaf_evidence(&mut paired_openings,&lines);}
        window_sash_evidence(&mut paired_openings,&lines);
    }
    for opening in &mut paired_openings {
        let Some(points)=opening["outlinePixels"].as_array() else {continue};
        let mut cad=Vec::new();
        for p in points {
            let (Some(x),Some(y))=(p[0].as_f64(),p[1].as_f64()) else {return Err("Invalid opening guide coordinates".into())};
            let (wx,wy)=(ox+x*scale,oy+(f64::from(height)-y)*scale);
            if !wx.is_finite() || !wy.is_finite() || wx.abs()>1.0e12 || wy.abs()>1.0e12 {
                return Err("Opening guide CAD coordinates exceed safe bounds; no CAD changes made".into());
            }
            cad.push(json!([wx,wy]));
        }
        opening["outlineCAD"]=json!(cad);
        opening["wallThicknessCAD"]=json!(opening["wallThicknessPixels"].as_f64().unwrap_or(0.0)*scale);
    }
    let create_door_arcs=args["createDoorSwingArcs"].as_bool().unwrap_or(false);
    if create_door_arcs && (!args["doorLeafHints"].as_bool().unwrap_or(false)
        || !paired_enabled || !args["confirmDoorSwingArcs"].as_bool().unwrap_or(false)) {
        return Err("createDoorSwingArcs requires doorLeafHints=true, pairedWallOpeningHints=true and confirmDoorSwingArcs=true; no CAD changes made".into());
    }
    let door_arcs=if create_door_arcs {door_swing_arcs(&paired_openings,&lines,ox,oy,height,scale)} else {Vec::new()};
    let create_door_blocks=args["createDoorBlocks"].as_bool().unwrap_or(false);
    if create_door_blocks && (!create_door_arcs || !args["confirmDoorBlocks"].as_bool().unwrap_or(false)) {
        return Err("createDoorBlocks requires createDoorSwingArcs=true and confirmDoorBlocks=true; no CAD changes made".into());
    }
    // A block consists of a newly drawn door-leaf LINE and swing ARC, not the
    // traced raster source edge: source evidence stays available for comparison.
    // The user's reviewed door width changes only the NEW native LINE/ARC
    // drafted for the Block. It never stretches or closes raster source edges.
    let override_width=match args.get("doorWidthOverrideCAD") {
        None => None,
        Some(value) => {
            let width=value.as_f64().ok_or("doorWidthOverrideCAD must be a positive finite CAD dimension; no edits made")?;
            if !width.is_finite() || !(0.000001..=1_000_000_000.0).contains(&width) {
                return Err("doorWidthOverrideCAD must be within 0.000001..1000000000 CAD units; no edits made".into());
            }
            if !create_door_blocks {return Err("doorWidthOverrideCAD requires reviewed createDoorBlocks=true; no edits made".into());}
            Some(width)
        }
    };
    // Per-candidate reviewed widths take precedence over the optional common width.
    // Keys are zero-based preview candidate indices; fail closed on stale/invalid indices.
    let mut widths_by_door=std::collections::BTreeMap::<usize,f64>::new();
    if let Some(value)=args.get("doorWidthOverridesCAD") {
        if !create_door_blocks {return Err("doorWidthOverridesCAD requires reviewed createDoorBlocks=true; no edits made".into());}
        let fields=value.as_object().ok_or("doorWidthOverridesCAD must be an object keyed by zero-based door index; no edits made")?;
        if fields.len()>64 {return Err("doorWidthOverridesCAD supports at most 64 reviewed doors; no edits made".into());}
        for (key,value) in fields {
            let index=key.parse::<usize>().map_err(|_|format!("doorWidthOverridesCAD key {key:?} must be a zero-based door index"))?;
            if index.to_string()!=*key || index>=door_arcs.len() {
                return Err(format!("doorWidthOverridesCAD index {key:?} is not a current Preview door; no edits made"));
            }
            let width=value.as_f64().ok_or_else(||format!("doorWidthOverridesCAD[{key}] must be a positive finite number"))?;
            if !width.is_finite() || !(0.000001..=1_000_000_000.0).contains(&width) {
                return Err(format!("doorWidthOverridesCAD[{key}] must be within 0.000001..1000000000 CAD units; no edits made"));
            }
            widths_by_door.insert(index,width);
        }
    }
    // Per-door switches act on freshly drafted Block geometry only. A Preview
    // candidate index is validated so edits cannot silently target another door.
    let parse_door_flips=|key: &str| -> Result<std::collections::BTreeSet<usize>,String> {
        let mut selected=std::collections::BTreeSet::new();
        if let Some(raw)=args.get(key) {
            if !create_door_blocks {return Err(format!("{key} requires reviewed createDoorBlocks=true; no edits made"));}
            let Some(items)=raw.as_array() else {return Err(format!("{key} must be an array of zero-based door indices"))};
            if items.len()>64 {return Err(format!("{key} supports at most 64 reviewed doors"));}
            for item in items {
                let Some(index)=item.as_u64().and_then(|x|usize::try_from(x).ok()) else {return Err(format!("{key} indices must be nonnegative integers"))};
                if index>=door_arcs.len() || !selected.insert(index) {
                    return Err(format!("{key} contains a stale or duplicate door index {index}; no edits made"));
                }
            }
        }
        Ok(selected)
    };
    let flip_hinges=parse_door_flips("doorHingeFlips")?;
    let flip_swings=parse_door_flips("doorSwingFlips")?;
    // When the operator reviews candidates individually, reject any changed list/order
    // rather than silently applying a previous door's width/hinge/swing to a new candidate.
    if let Some(value)=args.get("expectedDoorSourceKeys") {
        if !create_door_blocks {return Err("expectedDoorSourceKeys requires createDoorBlocks=true; no edits made".into());}
        let keys=value.as_array().ok_or("expectedDoorSourceKeys must be a list of the original Preview source keys")?;
        if keys.len()!=door_arcs.len() || keys.iter().enumerate().any(|(i,key)|key.as_str()!=door_arcs[i]["sourceDoorKey"].as_str()) {
            return Err("AI door candidates changed since the reviewed Preview; clear reviewed door edits and Preview again; no CAD changes made".into());
        }
    }
    let mut door_blocks=Vec::new();
    if create_door_blocks {
        for (door_index,arc) in door_arcs.iter().enumerate() {
            let (Some(radius),Some(mut leaf_angle),Some(mut opening_angle))=(arc["radius"].as_f64(),
                arc["leafAngleDegrees"].as_f64(),arc["openingAngleDegrees"].as_f64()) else {continue};
            let hinge_flipped=flip_hinges.contains(&door_index);
            let swing_flipped=flip_swings.contains(&door_index);
            let point=if hinge_flipped {&arc["oppositeHingeCAD"]} else {&arc["center"]};
            let (Some(cx),Some(cy))=(point[0].as_f64(),point[1].as_f64()) else {continue};
            if hinge_flipped {opening_angle=(opening_angle+180.0).rem_euclid(360.0);}
            if swing_flipped {leaf_angle=(leaf_angle+180.0).rem_euclid(360.0);}
            let delta=(opening_angle-leaf_angle).rem_euclid(360.0);
            let (start,end)=if (delta-90.0).abs()<=12.0 {(leaf_angle,opening_angle)}
                else if (delta-270.0).abs()<=12.0 {(opening_angle,leaf_angle)}
                else {return Err("Door hinge/swing override does not define a 90-degree drafting arc; no CAD edits made".into())};
            let reviewed_radius=widths_by_door.get(&door_index).copied().or(override_width).unwrap_or(radius);
            let angle=leaf_angle.to_radians();
            let x=cx+reviewed_radius*angle.cos(); let y=cy+reviewed_radius*angle.sin();
            if !x.is_finite() || !y.is_finite() || cx.abs()>1.0e12 || cy.abs()>1.0e12 || x.abs()>1.0e12 || y.abs()>1.0e12 {
                return Err("Door block draft coordinates are outside CAD safety bounds; no changes made".into());
            }
            let mut drafted_arc=arc.clone();
            drafted_arc["center"]=json!([cx,cy]);
            drafted_arc["radius"]=json!(reviewed_radius);
            drafted_arc["start"]=json!(start);
            drafted_arc["end"]=json!(end);
            door_blocks.push(json!({"base":[cx,cy],"leaf":[[cx,cy],[x,y]],"arc":drafted_arc,
                "widthCAD":reviewed_radius,"sourceOpeningWidthCAD":radius,
                "sourceDoorKey":arc["sourceDoorKey"],                "doorIndex":door_index,"hingeFlipped":hinge_flipped,"swingFlipped":swing_flipped,
                "widthOverridden":widths_by_door.contains_key(&door_index) || override_width.is_some(),
                "classification":"human-confirmed-cad-drafting-block-not-semantic-door"}));
        }
    }
    let create_window_blocks=args["createWindowBlocks"].as_bool().unwrap_or(false);
    if create_window_blocks && (!args["windowSashHints"].as_bool().unwrap_or(false) ||
        !args["confirmWindowBlocks"].as_bool().unwrap_or(false)) {
        return Err("createWindowBlocks requires windowSashHints=true and confirmWindowBlocks=true; no CAD changes made".into());
    }
    if create_window_blocks && args["createWindowGuidePolylines"].as_bool().unwrap_or(false) {
        return Err("Choose window Block or window PLINE guide, not both, to avoid duplicate geometry; no CAD changes made".into());
    }
    // The source raster lines stay separate. Only the NEW closed frame and two
    // sash LINEs are grouped into a human-confirmed CAD BLOCK/INSERT.
    let mut window_block_drafts=Vec::new();
    if create_window_blocks {
        for opening in paired_openings.iter().filter(|o|o["windowSashCandidate"]==true) {
            let Some(frame)=opening["outlineCAD"].as_array() else {continue};
            if frame.len()!=4 {continue;}
            let Some(indices)=opening["windowSashEvidence"]["sourceLineIndices"].as_array() else {continue};
            if indices.len()!=2 {continue;}
            let mut rails=Vec::new();
            for index in indices {
                let Some(source)=index.as_u64().and_then(|i|usize::try_from(i).ok()).and_then(|i|lines.get(i)) else {
                    return Err("Window sash source line index changed; no CAD edits made".into());
                };
                let points=[[source.x0,source.y0],[source.x1,source.y1]];
                let world:Vec<[f64;2]>=points.iter().map(|p|[ox+p[0]*scale,oy+(f64::from(height)-p[1])*scale]).collect();
                if world.iter().flatten().any(|x|!x.is_finite()||x.abs()>1.0e12) {
                    return Err("Window sash coordinates exceed safe CAD bounds; no edits made".into());
                }
                rails.push(json!(world));
            }
            window_block_drafts.push(json!({"base":frame[0],"frame":frame,"rails":rails,
                "classification":"human-confirmed-window-drafting-block-not-semantic-window"}));
        }
    }
    let create_window_guides=args["createWindowGuidePolylines"].as_bool().unwrap_or(false);
    if create_window_guides && (!args["windowSashHints"].as_bool().unwrap_or(false)
        || !args["confirmWindowGuides"].as_bool().unwrap_or(false)) {
        return Err("createWindowGuidePolylines requires windowSashHints=true and confirmWindowGuides=true; no CAD changes made".into());
    }
    let window_guides:Vec<Value>=if create_window_guides {
        paired_openings.iter().filter(|v|v["windowSashCandidate"]==true)
            .map(|v|v["outlineCAD"].clone()).collect()
    } else {Vec::new()};
    let create_opening_guides=args["createOpeningGuidePolylines"].as_bool().unwrap_or(false);
    if create_opening_guides && (!paired_enabled || !args["confirmOpeningGuides"].as_bool().unwrap_or(false)) {
        return Err("createOpeningGuidePolylines requires pairedWallOpeningHints=true and confirmOpeningGuides=true; no CAD changes made".into());
    }
    let source_strokes = lines.len();
    let bridged_gaps = bridge_collinear_gaps(&mut lines, gap_pixels as f64);
    // Pixel spacing is user-calibrated. Keep narrow, finite limits so the
    // detector cannot silently match opposite room edges or huge image spans.
    let numeric_spacing = |key: &str, fallback: f64| -> Result<f64, String> {
        match args.get(key) {
            None => Ok(fallback),
            Some(value) => value.as_f64().ok_or_else(|| format!("{key}: wall edge pixel spacing must be a number")),
        }
    };
    let min_wall_spacing=numeric_spacing("minWallSpacingPixels",2.0)?;
    let max_wall_spacing=numeric_spacing("maxWallSpacingPixels",40.0)?;
    if !min_wall_spacing.is_finite() || !max_wall_spacing.is_finite()
        || min_wall_spacing < 1.0 || max_wall_spacing > 80.0
        || max_wall_spacing < min_wall_spacing {
        return Err("Wall edge pixel spacing must satisfy 1 <= minimum <= maximum <= 80; no CAD changes made".into());
    }
    let wall_hints = if args["wallPairHints"].as_bool().unwrap_or(false) {
        parallel_wall_candidates(&lines, min_wall_spacing, max_wall_spacing)
    } else { Vec::new() };
    let column_hints=args["columnHatchHints"].as_bool().unwrap_or(false);
    let all_rectangles=if args["closedRectangles"].as_bool().unwrap_or(false) || column_hints {closed_rectangles(&lines)} else {Vec::new()};
    let mut column_candidates=if column_hints {column_hatch_candidates(&all_rectangles,&lines)} else {Vec::new()};
    let candidates=if args["closedRectangles"].as_bool().unwrap_or(false) {all_rectangles} else {Vec::new()};
    let mut replacement = vec![false; lines.len()];
    for outline in &candidates { for &index in &outline.edges { replacement[index] = true; } }
    let create_axes=args["createWallCenterlines"].as_bool().unwrap_or(false);
    if create_axes && !args["wallPairHints"].as_bool().unwrap_or(false) {
        return Err("createWallCenterlines requires wallPairHints=true to review candidate pairs".into());
    }
    let axes=if create_axes {wall_centerline_candidates(&lines,&wall_hints,&replacement)} else {Vec::new()};
    let junctions=wall_centerline_junctions(&axes);
    let count = lines.len();
    if count > max_lines as usize {
        return Err(format!("Found {count} strokes, exceeding maxLines={max_lines}; increase minLengthPixels or limit the input region, no CAD changes made"));
    }
    // Build and validate *all* CAD coordinates before any mutation.
    for candidate in &mut column_candidates {
        let Some(points)=candidate["outlinePixels"].as_array() else {continue};
        let mut vertices=Vec::new();
        for p in points {
            let (Some(x),Some(y))=(p[0].as_f64(),p[1].as_f64()) else {return Err("Invalid column outline pixels; no CAD changes made".into())};
            let (cx,cy)=(ox+x*scale,oy+(f64::from(height)-y)*scale);
            if !cx.is_finite()||!cy.is_finite()||cx.abs()>1.0e12||cy.abs()>1.0e12 {
                return Err("Column hint exceeds safe CAD coordinate bounds; no changes made".into());
            }
            vertices.push(json!([cx,cy]));
        }
        candidate["outlineCAD"]=json!(vertices);
        candidate["widthCAD"]=json!(candidate["widthPixels"].as_f64().unwrap_or(0.0)*scale);
        candidate["heightCAD"]=json!(candidate["heightPixels"].as_f64().unwrap_or(0.0)*scale);
    }
    let create_column_guides=args["createColumnGuidePolylines"].as_bool().unwrap_or(false);
    if create_column_guides && (!column_hints || !args["confirmColumnGuides"].as_bool().unwrap_or(false)) {
        return Err("createColumnGuidePolylines requires columnHatchHints=true and confirmColumnGuides=true; no CAD changes made".into());
    }
    let column_guide_polylines:Vec<Value>=if create_column_guides {
        column_candidates.iter().map(|v|v["outlineCAD"].clone()).collect()
    } else {Vec::new()};
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
    // Separate small JSON objects avoid serde_json::json! expansion beyond
    // Rust's default macro recursion limit as AI preview fields grow.
    let mut result=json!({
        "preview":true,"recognition":"automatic-raster-line-strokes",
        "model":"deterministic-pixel-vectorization-not-neural-AI",
        "imageSize":[width,height],"strokes":count,"sourceStrokes":source_strokes,
        "bridgedGaps":bridged_gaps,"maxGapPixels":gap_pixels,
        "openingGapCandidateCount":gaps_hint.len(),"openingGapCandidates":gaps_hint,
        "pairedWallOpeningCandidateCount":paired_openings.len(),
        "doorLeafCandidateCount":paired_openings.iter().filter(|v|v["doorLeafCandidate"]==true).count(),
        "windowSashCandidateCount":paired_openings.iter().filter(|v|v["windowSashCandidate"]==true).count()
    });
    let opening_guides_preview=if create_opening_guides {paired_openings.iter().map(|v|v["outlineCAD"].clone()).collect::<Vec<_>>()} else {Vec::new()};
    let architectural=json!({
        "pairedWallOpeningCandidates":paired_openings,
        "doorLeafHintsRequested":args["doorLeafHints"].as_bool().unwrap_or(false),
        "windowSashHintsRequested":args["windowSashHints"].as_bool().unwrap_or(false),
        "windowGuidePolylineCount":window_guides.len(),
        "windowGuidePolylinesRequested":create_window_guides,
        "windowGuidePolylines":window_guides,
        "windowBlockDraftCount":window_block_drafts.len(),
        "windowBlocksRequested":create_window_blocks,
        "windowBlockDrafts":window_block_drafts,
        "doorSwingArcsProposed":door_arcs.len(),"doorSwingArcsRequested":create_door_arcs,
        "doorSwingArcs":door_arcs,
        "doorBlockDraftCount":door_blocks.len(),"doorBlocksRequested":create_door_blocks,
        "doorBlockDrafts":door_blocks
    });
    let geometry=json!({
        "openingGuidePolylines":opening_guides_preview,
        "openingGuidePolylinesRequested":create_opening_guides,
        "minOpeningGapPixels":opening_min,"maxOpeningGapPixels":opening_max,
        "wallPairCandidateCount":wall_hints.len(),"wallPairHints":wall_hints,
        "minWallSpacingPixels":min_wall_spacing,"maxWallSpacingPixels":max_wall_spacing,
        "wallCenterlinesProposed":axes.len(),"wallCenterlines":world_axes,
        "wallCenterlineJunctionCount":junctions.len(),"wallCenterlineJunctions":junctions,
        "lines":world,"closedPolylines":world_rectangles,"rectanglesDetected":candidates.len(),
        "columnHatchCandidateCount":column_candidates.len(),
        "columnHatchCandidates":column_candidates,
        "columnHatchHintsRequested":column_hints,
        "columnGuidePolylineCount":column_guide_polylines.len(),
        "columnGuidePolylines":column_guide_polylines,
        "columnGuidePolylinesRequested":create_column_guides,
        "closedRectangles":args["closedRectangles"].as_bool().unwrap_or(false),
        "editableEntitiesCreated":0,"source":path,
        "caution":"Review vectors before committing; line recognition is not semantic building interpretation"
    });
    let Some(dst)=result.as_object_mut() else {unreachable!("preview JSON is an object")};
    for extra in [architectural,geometry] {
        let Value::Object(fields)=extra else {unreachable!("preview JSON section is an object")};
        dst.extend(fields);
    }
    Ok(result)
}

/// MCP entry point. The same preview function powers the desktop's AI drawing panel.
pub fn run(b: &mut dyn Backend, args: &Value) -> Result<Value, String> {
    let proposal = preview(args)?;
    if args["createWindowBlocks"].as_bool().unwrap_or(false) {
        return Err("createWindowBlocks is currently supported only by the desktop AI panel; MCP Block commit requires transaction ownership validation".into());
    }
    if args["createDoorBlocks"].as_bool().unwrap_or(false) {
        return Err("createDoorBlocks is currently supported by the desktop AI panel; MCP must use ARC drafts until block transaction ownership is implemented".into());
    }
    if args["preview"].as_bool().unwrap_or(true) { return Ok(proposal); }
    let Some(world) = proposal["lines"].as_array() else { return Err("Raster preview has no CAD lines".to_string()) };
    let rectangles = proposal["closedPolylines"].as_array().cloned().unwrap_or_default();
    let axes=proposal["wallCenterlines"].as_array().cloned().unwrap_or_default();
    let opening_guides=proposal["openingGuidePolylines"].as_array().cloned().unwrap_or_default();
    let window_guides=proposal["windowGuidePolylines"].as_array().cloned().unwrap_or_default();
    let column_guides=proposal["columnGuidePolylines"].as_array().cloned().unwrap_or_default();
    let door_arcs=proposal["doorSwingArcs"].as_array().cloned().unwrap_or_default();
    if world.is_empty() && rectangles.is_empty() && axes.is_empty() && opening_guides.is_empty() && window_guides.is_empty() && column_guides.is_empty() && door_arcs.is_empty() { return Err("No vector strokes detected; no CAD changes made".into()); }
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
        .chain(axes.iter().map(|v| ("line",json!({"points":v}))))
        .chain(opening_guides.iter().map(|v| ("pline",json!({"vertices":v,"closed":true}))))
        .chain(window_guides.iter().map(|v| ("pline",json!({"vertices":v,"closed":true}))))
        .chain(column_guides.iter().map(|v| ("pline",json!({"vertices":v,"closed":true}))))
        .chain(door_arcs.iter().map(|v| ("arc",json!({"center":v["center"],"radius":v["radius"],"start":v["start"],"end":v["end"]})))).collect();
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
        "rectanglesDetected":proposal["rectanglesDetected"],"closedPolylinesCreated":rectangles.len(),"openingGuidePolylinesCreated":opening_guides.len(),"windowGuidePolylinesCreated":window_guides.len(),"columnGuidePolylinesCreated":column_guides.len(),"doorSwingArcsCreated":door_arcs.len(),
        "wallCenterlinesCreated":axes.len(),
        "editableEntitiesCreated":handles.len(),
        "entities":handles,"source":proposal["source"],
        "caution":"Lines are grouped in one Undo action; review before saving DWG"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raster_vector_rejects_invalid_explicit_cad_scale_before_geometry() {
        let path=std::env::temp_dir().join(format!("cad360-scale-options-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let img=GrayImage::from_pixel(64,32,image::Luma([255]));
        img.save(&path).unwrap();
        for invalid in [json!("2"),json!(null),json!(false),json!(0),json!(-1)] {
            let args=json!({"path":path.to_string_lossy(),"unitsPerPixel":invalid});
            let error=preview(&args).expect_err("explicit invalid calibration must never fall back to CAD unit scale 1");
            assert!(error.contains("unitsPerPixel"),"bad value={invalid}, error={error}");
        }
        for valid in [json!(0.5),json!(2.0)] {
            let args=json!({"path":path.to_string_lossy(),"unitsPerPixel":valid});
            assert!(preview(&args).is_ok(),"valid scale {valid} should be accepted");
        }
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn raster_vector_rejects_malformed_gap_bridging_options_before_cad_creation() {
        let path=std::env::temp_dir().join(format!("cad360-gap-options-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let img=GrayImage::from_pixel(64,32,image::Luma([255]));
        img.save(&path).unwrap();
        for invalid in [json!(-1),json!(2.5),json!("4"),json!(null),json!(true)] {
            let args=json!({"path":path.to_string_lossy(),"maxGapPixels":invalid});
            let error=preview(&args).expect_err("invalid bridging options must not silently become zero");
            assert!(error.contains("maxGapPixels must be an integer"),"bad value={invalid}, error={error}");
        }
        let valid=preview(&json!({"path":path.to_string_lossy(),"maxGapPixels":0}));
        let _=std::fs::remove_file(path);
        assert!(valid.is_ok(),"explicit zero must still disable bridging: {valid:?}");
    }

    #[test]
    fn ai_real_png_paired_wall_opening_preview_and_editable_guide() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        let path=std::env::temp_dir().join(format!("cad360-paired-opening-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"preview":true,
            "minLengthPixels":16,"includeDiagonals":false,"openingGapHints":true,
            "pairedWallOpeningHints":true,"maxGapPixels":0,"unitsPerPixel":2.0,
            "origin":[100.0,200.0]});
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["openingGapCandidateCount"],2);
        assert_eq!(proposal["pairedWallOpeningCandidateCount"],1);
        assert_eq!(proposal["pairedWallOpeningCandidates"][0]["wallThicknessCAD"],14.0);
        assert_eq!(proposal["pairedWallOpeningCandidates"][0]["classification"],"paired-through-wall-gap-unverified");
        assert_eq!(proposal["lines"].as_array().unwrap().len(),4,"source wall edges must not be joined");
        assert_eq!(proposal["openingGuidePolylines"].as_array().unwrap().len(),0,"preview has no guide without opt-in");
        args["createOpeningGuidePolylines"]=json!(true);
        assert!(preview(&args).unwrap_err().contains("confirmOpeningGuides=true"));
        args["confirmOpeningGuides"]=json!(true);
        let confirmed=preview(&args).unwrap();
        assert_eq!(confirmed["openingGuidePolylines"].as_array().unwrap().len(),1);
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        let result=run(&mut backend,&args).unwrap();
        assert_eq!(result["openingGuidePolylinesCreated"],1);
        assert_eq!(result["editableEntitiesCreated"],5,"four original LINEs and one human-confirmed PLINE guide");
        let inspected=backend.call("drawing.inspect",json!({"entities":false})).unwrap();
        assert_eq!(inspected["entityCount"],5);
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_real_png_door_arc_commit_creates_native_arc_after_confirmation() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-door-arc-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,
            "includeDiagonals":false,"openingGapHints":true,"pairedWallOpeningHints":true,
            "doorLeafHints":true,"maxGapPixels":0,"unitsPerPixel":2.0,
            "createDoorSwingArcs":true});
        assert!(preview(&args).unwrap_err().contains("confirmDoorSwingArcs=true"));
        args["confirmDoorSwingArcs"]=json!(true);
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["doorSwingArcsProposed"],1);
        assert_eq!(proposal["doorSwingArcs"][0]["classification"],"confirmed-drafting-arc-not-door-block");
        let source_lines=proposal["lines"].as_array().unwrap().len();
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        let result=run(&mut backend,&args).unwrap();
        assert_eq!(result["doorSwingArcsCreated"],1);
        assert_eq!(result["editableEntitiesCreated"].as_u64(),Some((source_lines+1) as u64));
        let current=backend.call("drawing.inspect",json!({"entities":false})).unwrap();
        assert_eq!(current["entityCount"].as_u64(),Some((source_lines+1) as u64));
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_single_png_door_hinge_and_swing_flips_are_explicit_and_preserve_source() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-reviewed-hinge-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,
            "maxGapPixels":0,"unitsPerPixel":2.0,"createDoorSwingArcs":true,
            "confirmDoorSwingArcs":true,"createDoorBlocks":true,"confirmDoorBlocks":true});
        let original=preview(&args).unwrap();
        assert_eq!(original["doorBlockDraftCount"],1);
        let orig=&original["doorBlockDrafts"][0];
        assert_eq!(orig["hingeFlipped"],false);
        assert_eq!(orig["swingFlipped"],false);
        args["doorHingeFlips"]=json!([0]);
        let reversed=preview(&args).unwrap();
        let door=&reversed["doorBlockDrafts"][0];
        assert_eq!(door["hingeFlipped"],true);
        assert_eq!(door["widthCAD"],orig["widthCAD"]);
        assert_ne!(door["base"],orig["base"]);
        assert_eq!(reversed["lines"],original["lines"]);
        args["doorHingeFlips"]=json!([]);
        args["doorSwingFlips"]=json!([0]);
        let swung=preview(&args).unwrap();
        assert_eq!(swung["doorBlockDrafts"][0]["swingFlipped"],true);
        assert_eq!(swung["doorBlockDrafts"][0]["base"],orig["base"]);
        assert_eq!(swung["doorBlockDrafts"][0]["widthCAD"],orig["widthCAD"]);
        assert_eq!(swung["lines"],original["lines"]);
        for invalid in [json!([-1]),json!([1]),json!([0,0]),json!({"0":true}),json!(null)] {
            args["doorHingeFlips"]=invalid;
            assert!(preview(&args).is_err(),"invalid hinge index must not silently alter the drawing");
        }
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_stale_per_door_review_keys_must_not_retarget_a_different_candidate() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-stale-door-review-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,
            "maxGapPixels":0,"createDoorSwingArcs":true,"confirmDoorSwingArcs":true,
            "createDoorBlocks":true,"confirmDoorBlocks":true});
        let first=preview(&args).unwrap();
        let key=first["doorBlockDrafts"][0]["sourceDoorKey"].as_str().unwrap().to_string();
        args["expectedDoorSourceKeys"]=json!([key]);
        args["doorWidthOverridesCAD"]=json!({"0":30.0});
        assert_eq!(preview(&args).unwrap()["doorBlockDrafts"][0]["widthCAD"],30.0);
        args["expectedDoorSourceKeys"]=json!(["mismatched-image:another-door"]);
        let err=preview(&args).unwrap_err();
        assert!(err.contains("door candidates changed"),"{err}");
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_two_png_doors_have_independent_widths_and_validate_stale_review() {
        let mut img=image::RgbImage::from_pixel(245,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for (begin,end) in [(5,39),(51,95),(135,169),(181,225)] {
                for x in begin..=end {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            }
        }
        for x in [39,169] {for y in 3..=15 {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        let path=std::env::temp_dir().join(format!("cad360-two-door-widths-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,
            "maxGapPixels":0,"maxOpeningGapPixels":20.0,"unitsPerPixel":2.0,
            "createDoorSwingArcs":true,"confirmDoorSwingArcs":true,"createDoorBlocks":true,"confirmDoorBlocks":true});
        let original=preview(&args).unwrap();
        assert_eq!(original["doorBlockDraftCount"],2,"expect exactly two independent openings");
        let original_lines=original["lines"].clone();
        args["doorWidthOverridesCAD"]=json!({"1":36.0});
        let changed=preview(&args).unwrap();
        assert_eq!(changed["doorBlockDraftCount"],2);
        assert_eq!(changed["doorBlockDrafts"][0]["widthCAD"],24.0);
        assert_eq!(changed["doorBlockDrafts"][0]["widthOverridden"],false);
        assert_eq!(changed["doorBlockDrafts"][1]["widthCAD"],36.0);
        assert_eq!(changed["doorBlockDrafts"][1]["arc"]["radius"],36.0);
        assert_eq!(changed["doorBlockDrafts"][1]["sourceOpeningWidthCAD"],24.0);
        assert_eq!(changed["lines"],original_lines,"reviewed dimensions must not alter vectorized wall/door source strokes");
        for invalid in [json!({"2":30}),json!({"-1":30}),json!({"01":30}),json!({"0":null}),json!({"0":-2}),json!([30.0])] {
            args["doorWidthOverridesCAD"]=invalid;
            assert!(preview(&args).is_err(),"invalid or stale per-door width must fail closed");
        }
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_per_door_width_override_changes_only_selected_door_geometry_and_rejects_stale_indices() {
        let gap=|start:f64,end:f64,y:f64|json!({"fromPixels":[start,y],"toPixels":[end,y]});
        let mut openings=paired_wall_openings(&[
            gap(39.5,51.5,15.5),gap(39.5,51.5,22.5),
            gap(79.5,91.5,15.5),gap(79.5,91.5,22.5)
        ],2.0,40.0);
        let leaves=[
            Line{x0:39.5,y0:3.5,x1:39.5,y1:15.5},
            Line{x0:79.5,y0:3.5,x1:79.5,y1:15.5}
        ];
        door_leaf_evidence(&mut openings,&leaves);
        let arcs=door_swing_arcs(&openings,&leaves,0.0,0.0,48,2.0);
        assert_eq!(arcs.len(),2,"independent doors must retain a stable preview order");
        assert_eq!(arcs[0]["radius"],24.0);
        assert_eq!(arcs[1]["radius"],24.0);
    }

    #[test]
    fn ai_reviewed_door_width_changes_only_new_block_leaf_and_arc_not_image_strokes() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-reviewed-door-width-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,
            "maxGapPixels":0,"createDoorSwingArcs":true,"confirmDoorSwingArcs":true,
            "createDoorBlocks":true,"confirmDoorBlocks":true,"unitsPerPixel":2.0});
        let original=preview(&args).unwrap();
        assert_eq!(original["doorBlockDraftCount"],1);
        assert_eq!(original["doorBlockDrafts"][0]["widthCAD"],24.0);
        args["doorWidthOverrideCAD"]=json!(30.0);
        let modified=preview(&args).unwrap();
        assert_eq!(modified["doorBlockDrafts"][0]["widthCAD"],30.0);
        assert_eq!(modified["doorBlockDrafts"][0]["sourceOpeningWidthCAD"],24.0);
        assert_eq!(modified["doorBlockDrafts"][0]["arc"]["radius"],30.0);
        assert_eq!(modified["doorBlockDrafts"][0]["widthOverridden"],true);
        assert_eq!(modified["lines"],original["lines"],"the PNG source strokes must never be resized");
        assert_eq!(modified["openingGapCandidates"],original["openingGapCandidates"]);
        for value in [json!("30"),json!(null),json!(0),json!(-1),json!(1.0e12)] {
            args["doorWidthOverrideCAD"]=value;
            assert!(preview(&args).is_err(),"invalid reviewed width must fail before editing CAD");
        }
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_door_block_drafts_require_review_and_keep_traced_lines_separate() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-door-block-draft-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,
            "maxGapPixels":0,"createDoorSwingArcs":true,"confirmDoorSwingArcs":true,
            "createDoorBlocks":true});
        assert!(preview(&args).unwrap_err().contains("confirmDoorBlocks=true"));
        args["confirmDoorBlocks"]=json!(true);
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["doorBlockDraftCount"],1);
        assert_eq!(proposal["doorBlockDrafts"][0]["classification"],"human-confirmed-cad-drafting-block-not-semantic-door");
        assert!(proposal["lines"].as_array().unwrap().len()>=4,"original raster sources must remain for review");
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        assert!(run(&mut backend,&args).unwrap_err().contains("desktop AI panel"),
            "MCP block transactions must fail closed until ownership/undo is validated");
        assert_eq!(backend.session.doc().unwrap().entity_count(),0);
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_confirmed_door_swing_arc_is_editable_and_refuses_missing_confirmation() {
        let gap=|y:f64|json!({"fromPixels":[39.5,y],"toPixels":[51.5,y]});
        let mut candidates=paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        let leaf=Line{x0:39.5,y0:3.5,x1:39.5,y1:15.5};
        door_leaf_evidence(&mut candidates,&[leaf]);
        assert_eq!(candidates[0]["doorLeafCandidate"],true);
        let arcs=door_swing_arcs(&candidates,&[leaf],100.0,200.0,48,2.0);
        assert_eq!(arcs.len(),1,"a single observed perpendicular leaf must allow a 90-degree swing guide");
        assert_eq!(arcs[0]["center"],json!([179.0,265.0]));
        assert_eq!(arcs[0]["radius"],24.0);
        assert!((arcs[0]["end"].as_f64().unwrap()-arcs[0]["start"].as_f64().unwrap()).rem_euclid(360.0)-90.0<1e-9);
        let mut no_leaf=paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        assert!(door_swing_arcs(&no_leaf,&[leaf],0.0,0.0,48,1.0).is_empty());
        door_leaf_evidence(&mut no_leaf,&[leaf,Line{x0:51.5,y0:3.5,x1:51.5,y1:15.5}]);
        assert!(door_swing_arcs(&no_leaf,&[leaf],0.0,0.0,48,1.0).is_empty(),"ambiguous door evidence must not draw");
    }

    #[test]
    fn ai_door_leaf_evidence_requires_a_perpendicular_outward_stroke_at_opening_corner() {
        let gap=|y:f64|json!({"fromPixels":[39.5,y],"toPixels":[51.5,y]});
        let mut candidate=paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        assert_eq!(candidate.len(),1);
        // A 12 px leaf from the top-left hinge extends outside the 7 px wall.
        let leaf=Line{x0:39.5,y0:3.5,x1:39.5,y1:15.5};
        door_leaf_evidence(&mut candidate,&[leaf]);
        assert_eq!(candidate[0]["doorLeafCandidate"],true);
        assert_eq!(candidate[0]["doorLeafEvidence"]["sourceLineIndex"],0);
        assert_eq!(candidate[0]["classification"],"paired-through-wall-gap-unverified",
            "leaf hints must never upgrade the opening to a confirmed door");
        // Jamb only between both wall edges is not door-leaf evidence.
        let mut jamb=paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        door_leaf_evidence(&mut jamb,&[Line{x0:39.5,y0:15.5,x1:39.5,y1:22.5}]);
        assert_eq!(jamb[0]["doorLeafCandidate"],false);
        let mut ambiguous=paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        door_leaf_evidence(&mut ambiguous,&[leaf,Line{x0:51.5,y0:3.5,x1:51.5,y1:15.5}]);
        assert_eq!(ambiguous[0]["doorLeafCandidate"],false);
        assert_eq!(ambiguous[0]["doorLeafAmbiguous"],true);
    }

    #[test]
    fn ai_real_png_door_leaf_hint_is_preview_only_and_preserves_native_geometry() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-door-leaf-hint-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"preview":true,"minLengthPixels":8,
            "includeDiagonals":false,"openingGapHints":true,"pairedWallOpeningHints":true,
            "doorLeafHints":true,"maxGapPixels":0,"unitsPerPixel":2.0});
        let proposed=preview(&args).unwrap();
        assert_eq!(proposed["pairedWallOpeningCandidateCount"],1);
        assert_eq!(proposed["doorLeafCandidateCount"],1,"the perpendicular leaf stroke is measurable evidence");
        assert_eq!(proposed["editableEntitiesCreated"],0,"preview must not edit a CAD document");
        assert_eq!(proposed["openingGuidePolylines"].as_array().unwrap().len(),0,"no automatic door or guide");
        args["doorLeafHints"]=json!(false);
        let without=preview(&args).unwrap();
        assert_eq!(without["doorLeafCandidateCount"],0);
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_two_aligned_wall_edge_gaps_propose_one_opening_outline() {
        let gap=|y:f64|json!({"fromPixels":[25.0,y],"toPixels":[35.0,y],"gapPixels":10.0});
        let result=paired_wall_openings(&[gap(10.0),gap(16.0)],2.0,40.0);
        assert_eq!(result.len(),1);
        assert_eq!(result[0]["outlinePixels"],json!([[25.0,10.0],[35.0,10.0],[35.0,16.0],[25.0,16.0]]));
        assert_eq!(result[0]["wallThicknessPixels"],6.0);
        assert_eq!(result[0]["classification"],"paired-through-wall-gap-unverified");
    }

    #[test]
    fn ai_window_sash_needs_two_inner_parallel_rails_and_no_competing_door_leaf() {
        let gap=|y:f64|json!({"fromPixels":[39.5,y],"toPixels":[51.5,y]});
        let baseline=||paired_wall_openings(&[gap(15.5),gap(22.5)],2.0,40.0);
        let rail1=Line{x0:39.5,y0:17.5,x1:51.5,y1:17.5};
        let rail2=Line{x0:39.5,y0:20.5,x1:51.5,y1:20.5};
        let mut good=baseline();
        window_sash_evidence(&mut good,&[rail1,rail2]);
        assert_eq!(good[0]["windowSashCandidate"],true);
        assert_eq!(good[0]["windowSashEvidence"]["sourceLineIndices"],json!([0,1]));
        let mut one=baseline();
        window_sash_evidence(&mut one,&[rail1]);
        assert_ne!(one[0]["windowSashCandidate"],true);
        let mut extra=baseline();
        window_sash_evidence(&mut extra,&[rail1,rail2,Line{x0:39.5,y0:19.0,x1:51.5,y1:19.0}]);
        assert_ne!(extra[0]["windowSashCandidate"],true);
        let mut door=baseline();
        let leaf=Line{x0:39.5,y0:3.5,x1:39.5,y1:15.5};
        door_leaf_evidence(&mut door,&[leaf,rail1,rail2]);
        window_sash_evidence(&mut door,&[leaf,rail1,rail2]);
        assert_eq!(door[0]["doorLeafCandidate"],true);
        assert_ne!(door[0]["windowSashCandidate"],true,"door evidence must suppress window guesses");
    }

    #[test]
    fn ai_column_hatch_requires_closed_near_square_with_two_crossing_interior_strokes() {
        let box_rect=RectangleOutline{edges:[0,1,2,3],corners:[[10.0,10.0],[42.0,10.0],[42.0,42.0],[10.0,42.0]]};
        let edges=[Line{x0:10.0,y0:10.0,x1:42.0,y1:10.0},Line{x0:10.0,y0:42.0,x1:42.0,y1:42.0},
            Line{x0:10.0,y0:10.0,x1:10.0,y1:42.0},Line{x0:42.0,y0:10.0,x1:42.0,y1:42.0}];
        assert!(column_hatch_candidates(&[box_rect.clone()],&edges).is_empty(),"empty rectangle is not column evidence");
        let h=Line{x0:21.0,y0:26.0,x1:31.0,y1:26.0};
        let v=Line{x0:26.0,y0:21.0,x1:26.0,y1:31.0};
        let mut with_cross=edges.to_vec();with_cross.extend([h,v]);
        let columns=column_hatch_candidates(&[box_rect.clone()],&with_cross);
        assert_eq!(columns.len(),1);
        assert_eq!(columns[0]["sourceHatchEdges"],json!([4,5]));
        with_cross.push(Line{x0:19.0,y0:27.0,x1:33.0,y1:27.0});
        assert!(column_hatch_candidates(&[box_rect],&with_cross).is_empty(),"multiple hatch interpretations must not guess");
    }

    #[test]
    fn ai_column_hatch_png_preview_requires_confirmation_and_keeps_source_strokes() {
        let mut img=image::RgbImage::from_pixel(90,80,image::Rgb([255,255,255]));
        for x in 20..=52 {for y in [16,48] {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        for y in 16..=48 {for x in [20,52] {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        for x in 31..=41 {img.put_pixel(x,32,image::Rgb([0,0,0]));}
        for y in 27..=37 {img.put_pixel(36,y,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-column-hatch-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "closedRectangles":false,"columnHatchHints":true,"unitsPerPixel":2.0,"maxGapPixels":0});
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["columnHatchCandidateCount"],1);
        assert_eq!(proposal["closedPolylines"].as_array().unwrap().len(),0,
            "column hints may not enable general closed rectangle conversion");
        assert_eq!(proposal["columnGuidePolylineCount"],0,"column guides are never automatic");
        assert!(proposal["lines"].as_array().unwrap().len()>=6,"source pixels must remain editable after Commit");
        args["createColumnGuidePolylines"]=json!(true);
        assert!(preview(&args).unwrap_err().contains("confirmColumnGuides=true"));
        args["confirmColumnGuides"]=json!(true);
        let confirmed=preview(&args).unwrap();
        assert_eq!(confirmed["columnGuidePolylineCount"],1);
        let source_count=confirmed["lines"].as_array().unwrap().len();
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        let result=run(&mut backend,&args).unwrap();
        assert_eq!(result["columnGuidePolylinesCreated"],1);
        assert_eq!(result["editableEntitiesCreated"].as_u64(),Some((source_count+1) as u64));
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_real_png_confirmed_window_block_draft_preserves_source_lines_and_rejects_mcp_commit() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        for y in [17,20] {for x in 39..=51 {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        let path=std::env::temp_dir().join(format!("cad360-window-block-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"windowSashHints":true,
            "maxGapPixels":0,"unitsPerPixel":2.0,"createWindowBlocks":true});
        assert!(preview(&args).unwrap_err().contains("confirmWindowBlocks=true"));
        args["confirmWindowBlocks"]=json!(true);
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["windowBlockDraftCount"],1);
        let draft=&proposal["windowBlockDrafts"][0];
        assert_eq!(draft["frame"].as_array().unwrap().len(),4);
        assert_eq!(draft["rails"].as_array().unwrap().len(),2);
        assert_eq!(draft["classification"],"human-confirmed-window-drafting-block-not-semantic-window");
        assert!(proposal["lines"].as_array().unwrap().len()>=6,"original raster sash and wall lines remain");
        assert_eq!(proposal["editableEntitiesCreated"],0);
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        assert!(run(&mut backend,&args).unwrap_err().contains("desktop AI panel"));
        assert_eq!(backend.session.doc().unwrap().entity_count(),0);
        args["createWindowGuidePolylines"]=json!(true);
        args["confirmWindowGuides"]=json!(true);
        assert!(preview(&args).unwrap_err().contains("not both"));
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_window_guide_requires_confirmation_before_native_closed_pline() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in [17,20] {for x in 39..=51 {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        let path=std::env::temp_dir().join(format!("cad360-window-guide-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,"includeDiagonals":false,
            "openingGapHints":true,"pairedWallOpeningHints":true,"windowSashHints":true,
            "maxGapPixels":0,"unitsPerPixel":2.0,"createWindowGuidePolylines":true});
        assert!(preview(&args).unwrap_err().contains("confirmWindowGuides=true"));
        args["confirmWindowGuides"]=json!(true);
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["windowGuidePolylineCount"],1);
        let original_lines=proposal["lines"].as_array().unwrap().len();
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        let result=run(&mut backend,&args).unwrap();
        assert_eq!(result["windowGuidePolylinesCreated"],1);
        assert_eq!(result["editableEntitiesCreated"].as_u64(),Some((original_lines+1) as u64));
        assert_eq!(backend.session.doc().unwrap().entity_count(),original_lines+1);
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_real_png_window_sash_hint_is_preview_only_and_preserves_source_geometry() {
        let mut img=image::RgbImage::from_pixel(110,48,image::Rgb([255,255,255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
            for x in 51..=95 {img.put_pixel(x,y,image::Rgb([0,0,0]));}
        }
        for y in [17,20] {for x in 39..=51 {img.put_pixel(x,y,image::Rgb([0,0,0]));}}
        let path=std::env::temp_dir().join(format!("cad360-window-sash-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let args=json!({"path":path.to_string_lossy(),"minLengthPixels":8,
            "includeDiagonals":false,"openingGapHints":true,"pairedWallOpeningHints":true,
            "windowSashHints":true,"maxGapPixels":0,"unitsPerPixel":2.0});
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["pairedWallOpeningCandidateCount"],1);
        assert_eq!(proposal["windowSashCandidateCount"],1);
        assert_eq!(proposal["editableEntitiesCreated"],0);
        assert!(proposal["lines"].as_array().unwrap().len()>=6);
        assert_eq!(proposal["windowSashHintsRequested"],true);
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_single_or_ambiguous_opening_gap_must_not_invent_a_door() {
        let gap=|start:f64,end:f64,y:f64|json!({"fromPixels":[start,y],"toPixels":[end,y]});
        assert!(paired_wall_openings(&[gap(25.0,35.0,10.0)],2.0,40.0).is_empty());
        assert!(paired_wall_openings(&[gap(25.0,35.0,10.0),gap(25.0,35.0,16.0),gap(25.0,35.0,18.0)],2.0,40.0).is_empty());
        assert!(paired_wall_openings(&[gap(25.0,35.0,10.0),gap(41.0,51.0,16.0)],2.0,40.0).is_empty());
    }

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
    fn ai_wall_centerline_real_png_keeps_both_edges_and_adds_editable_centerline() {
        let mut img=image::RgbImage::from_pixel(128,64,image::Rgb([255,255,255]));
        for x in 10..=110 {
            img.put_pixel(x,12,image::Rgb([0,0,0]));
            img.put_pixel(x,22,image::Rgb([0,0,0]));
        }
        let path=std::env::temp_dir().join(format!("cad360-wall-axis-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"preview":true,"minLengthPixels":16,
            "includeDiagonals":false,"wallPairHints":true,"createWallCenterlines":true,"unitsPerPixel":10.0});
        let proposal=preview(&args).unwrap();
        assert_eq!(proposal["wallPairCandidateCount"],1);
        assert_eq!(proposal["wallCenterlinesProposed"],1);
        assert_eq!(proposal["lines"].as_array().unwrap().len(),2,"original edges must be kept");
        assert_eq!(proposal["wallCenterlines"],json!([[[105.0,465.0],[1105.0,465.0]]]));
        let mut backend=crate::backend::Headless::default();
        args["preview"]=json!(false);
        let committed=run(&mut backend,&args).unwrap();
        let _=std::fs::remove_file(&path);
        assert_eq!(committed["wallCenterlinesCreated"],1);
        assert_eq!(committed["editableEntitiesCreated"],3);
        let inspected=backend.call("drawing.inspect",json!({"entities":false})).unwrap();
        assert_eq!(inspected["entityCount"],3);
        backend.call("engine.execute",json!({"command":"undo","params":{"count":1}})).unwrap();
        let after=backend.call("drawing.inspect",json!({"entities":false})).unwrap();
        assert_eq!(after["entityCount"],0,"undo must roll back both source edges and the new centerline");
    }

    #[test]
    fn ai_wall_centerline_rejects_creation_without_hint_review() {
        let img=image::RgbImage::from_pixel(32,16,image::Rgb([255,255,255]));
        let path=std::env::temp_dir().join(format!("cad360-wall-gate-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let args=json!({"path":path.to_string_lossy(),"preview":true,"minLengthPixels":12,
            "includeDiagonals":false,"createWallCenterlines":true,"wallPairHints":false});
        let err=preview(&args).unwrap_err();
        let _=std::fs::remove_file(&path);
        assert!(err.contains("requires wallPairHints=true"));
    }

    #[test]
    fn ai_wall_spacing_range_filters_pairs_without_rescaling_source_geometry() {
        let lines=vec![
            Line{x0:10.5,y0:10.5,x1:110.5,y1:10.5},
            Line{x0:10.5,y0:20.5,x1:110.5,y1:20.5}
        ];
        assert_eq!(parallel_wall_candidates(&lines,2.0,40.0).len(),1);
        assert!(parallel_wall_candidates(&lines,11.0,40.0).is_empty());
        assert!(parallel_wall_candidates(&lines,1.0,9.0).is_empty());
    }

    #[test]
    fn ai_wall_spacing_validation_refuses_impossible_values_before_preview() {
        let img=image::RgbImage::from_pixel(32,16,image::Rgb([255,255,255]));
        let path=std::env::temp_dir().join(format!("cad360-spacing-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        for (lo,hi) in [(0.0,40.0),(10.0,9.0),(1.0,81.0),(f64::NAN,40.0)] {
            let args=json!({"path":path.to_string_lossy(),"preview":true,
                "wallPairHints":true,"minWallSpacingPixels":lo,"maxWallSpacingPixels":hi});
            let e=preview(&args).expect_err("invalid wall spacing must be rejected");
            assert!(e.contains("pixel spacing"),"unexpected validation error: {e}");
        }
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_opening_gap_bounds_and_world_width_are_calibrated() {
        let mut img=image::RgbImage::from_pixel(110,40,image::Rgb([255,255,255]));
        for x in 5..=39 {img.put_pixel(x,17,image::Rgb([0,0,0]));}
        for x in 51..=95 {img.put_pixel(x,17,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-opening-size-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut args=json!({"path":path.to_string_lossy(),"preview":true,"minLengthPixels":16,
            "includeDiagonals":false,"openingGapHints":true,"maxGapPixels":0,
            "unitsPerPixel":10.0,"origin":[100.0,200.0],"minOpeningGapPixels":10.0,"maxOpeningGapPixels":20.0});
        let v=preview(&args).unwrap();
        assert_eq!(v["openingGapCandidateCount"],1);
        let gap=&v["openingGapCandidates"][0];
        assert_eq!(gap["gapPixels"],12.0);
        assert_eq!(gap["widthCAD"],120.0);
        assert_eq!(gap["fromCAD"],json!([495.0,425.0]));
        assert_eq!(gap["toCAD"],json!([615.0,425.0]));
        args["minOpeningGapPixels"]=json!(13.0);
        assert_eq!(preview(&args).unwrap()["openingGapCandidateCount"],0);
        args["minOpeningGapPixels"]=json!(0.0);
        assert!(preview(&args).unwrap_err().contains("Opening gap pixels"));
        args["minOpeningGapPixels"]=json!(15.0);
        args["maxOpeningGapPixels"]=json!(10.0);
        assert!(preview(&args).unwrap_err().contains("Opening gap pixels"));
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_opening_gap_real_png_refuses_to_bridge_possible_door() {
        let mut img=image::RgbImage::from_pixel(110,40,image::Rgb([255,255,255]));
        for x in 5..=39 {img.put_pixel(x,17,image::Rgb([0,0,0]));}
        for x in 51..=95 {img.put_pixel(x,17,image::Rgb([0,0,0]));}
        let path=std::env::temp_dir().join(format!("cad360-keep-opening-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let base=json!({"path":path.to_string_lossy(),"preview":true,
            "minLengthPixels":16,"includeDiagonals":false,"openingGapHints":true,"maxGapPixels":0});
        let proposed=preview(&base).expect("non-mutating preview");
        assert_eq!(proposed["openingGapCandidateCount"],1,"should mark a single potential opening");
        assert_eq!(proposed["lines"].as_array().unwrap().len(),2,"never bridge a possible doorway");
        let mut wrong=base;
        wrong["maxGapPixels"]=json!(12);
        let err=preview(&wrong).expect_err("cannot request both opening detection and auto-bridge");
        assert!(err.contains("requires maxGapPixels=0"),"{err}");
        let _=std::fs::remove_file(&path);
    }

    #[test]
    fn ai_opening_gap_duplicate_raster_strokes_do_not_duplicate_one_gap() {
        let left=Line{x0:5.5,y0:20.5,x1:35.5,y1:20.5};
        let right=Line{x0:47.5,y0:20.5,x1:87.5,y1:20.5};
        let duplicated=vec![left,right,left,right];
        let hints=opening_gap_candidates(&duplicated,3.0,40.0);
        assert_eq!(hints.len(),1,"duplicate extracted strokes represent just one physical gap");
        assert_eq!(hints[0]["gapPixels"],12.0);
    }

    #[test]
    fn ai_opening_gap_different_openings_are_not_deduplicated() {
        let lines=vec![
            Line{x0:5.5,y0:10.5,x1:35.5,y1:10.5},
            Line{x0:47.5,y0:10.5,x1:87.5,y1:10.5},
            Line{x0:5.5,y0:30.5,x1:35.5,y1:30.5},
            Line{x0:47.5,y0:30.5,x1:87.5,y1:30.5},
        ];
        let hints=opening_gap_candidates(&lines,3.0,40.0);
        assert_eq!(hints.len(),2,"independent openings on parallel walls must stay distinct");
    }

    #[test]
    fn ai_opening_gap_refuses_crossing_wall_or_third_fragment() {
        let first=Line{x0:5.5,y0:20.5,x1:35.5,y1:20.5};
        let second=Line{x0:55.5,y0:20.5,x1:95.5,y1:20.5};
        assert_eq!(opening_gap_candidates(&[first,second],3.0,40.0).len(),1);
        let crossing=Line{x0:45.5,y0:4.5,x1:45.5,y1:37.5};
        assert!(opening_gap_candidates(&[first,second,crossing],3.0,40.0).is_empty(),
            "Do not suggest a door when a perpendicular wall passes through the proposed opening");
        let fragment=Line{x0:40.5,y0:20.5,x1:50.5,y1:20.5};
        // There may be shorter secondary gaps between three fragments, but
        // never imply that the large interval from 35.5 to 55.5 is empty.
        let hints=opening_gap_candidates(&[first,second,fragment],3.0,40.0);
        assert!(hints.iter().all(|g|g["gapPixels"].as_f64().unwrap_or(0.0)<20.0));
    }

    #[test]
    fn ai_opening_gap_vertical_crossing_is_not_a_clear_doorway() {
        let first=Line{x0:30.5,y0:5.5,x1:30.5,y1:35.5};
        let second=Line{x0:30.5,y0:55.5,x1:30.5,y1:95.5};
        let cross=Line{x0:12.5,y0:45.5,x1:60.5,y1:45.5};
        assert_eq!(opening_gap_candidates(&[first,second],3.0,40.0).len(),1);
        assert!(opening_gap_candidates(&[first,second,cross],3.0,40.0).is_empty());
    }

    #[test]
    fn ai_opening_gap_hints_preserve_unjoined_wall_segments() {
        let lines=vec![
            Line{x0:5.5,y0:10.5,x1:35.5,y1:10.5},
            Line{x0:47.5,y0:10.5,x1:87.5,y1:10.5},
        ];
        let before=lines.len();
        let hints=opening_gap_candidates(&lines,3.0,40.0);
        assert_eq!(hints.len(),1);
        assert_eq!(hints[0]["gapPixels"],12.0);
        assert_eq!(hints[0]["classification"],"unverified-gap-not-door");
        assert_eq!(lines.len(),before,"preview must not close or change the source lines");
    }

    #[test]
    fn ai_opening_gap_hints_ignore_parallel_separated_and_touching_lines() {
        let lines=vec![
            Line{x0:5.5,y0:10.5,x1:35.5,y1:10.5},
            Line{x0:47.5,y0:16.5,x1:87.5,y1:16.5},
            Line{x0:35.5,y0:10.5,x1:70.5,y1:10.5},
        ];
        assert!(opening_gap_candidates(&lines,3.0,40.0).is_empty());
    }

    #[test]
    fn ai_wall_junction_preview_finds_t_and_cross_candidates_without_drawing() {
        let axes=vec![
            Line{x0:10.0,y0:20.0,x1:110.0,y1:20.0},
            Line{x0:60.0,y0:0.0,x1:60.0,y1:70.0},
            Line{x0:90.0,y0:20.0,x1:90.0,y1:70.0},
        ];
        let junctions=wall_centerline_junctions(&axes);
        assert_eq!(junctions.len(),2);
        assert_eq!(junctions[0]["kind"],"cross-candidate");
        assert_eq!(junctions[0]["pointPixels"],json!([60.0,20.0]));
        assert_eq!(junctions[1]["kind"],"T-candidate");
        assert_eq!(junctions[1]["pointPixels"],json!([90.0,20.0]));
    }

    #[test]
    fn ai_wall_junction_refuses_disjoint_and_tip_only_lines() {
        let axes=vec![
            Line{x0:10.0,y0:20.0,x1:50.0,y1:20.0},
            Line{x0:60.0,y0:0.0,x1:60.0,y1:70.0},
            Line{x0:50.0,y0:20.0,x1:50.0,y1:50.0},
        ];
        assert!(wall_centerline_junctions(&axes).is_empty(),
            "Nearby or tip-only geometry must not be treated as a confirmed wall junction");
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
