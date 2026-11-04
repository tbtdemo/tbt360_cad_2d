//! IMAGEATTACH / IMAGECLIP: metadata-first raster image attachment.
//!
//! The drawing model already preserves IMAGE/IMAGEDEF data through DXF. These commands create
//! and edit that metadata without pretending to decode or render raster pixels in the engine.
//! The renderer continues to show the image frame until a host-side bitmap renderer is wired in.

use cadcraft_doc::{EntityKind, Handle, Image, LwPolyline};
use cadcraft_geom::{PolyVertex, Vec2, Vec3};
use serde_json::{Value, json};

use super::{CommandSpec, bad, bool_or, f64_or, point_param, str_param, targets};
use crate::{Result, Session};

pub(crate) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("imageattach", "Attach Raster Image", run_imageattach)
            .menu(&["Insert", "Attach Image..."])
            .alias(&["ima", "image"])
            .params("{path, at?: [x,y], width?: number, height?: number, pixels?: [w,h], rotation?: degrees, brightness?: 0..100, contrast?: 0..100, fade?: 0..100} → {handle, path, bounds}"),
        CommandSpec::new("imagetrace", "Trace CAD Geometry on Raster Image", run_imagetrace)
            .menu(&["Draw", "Image", "Trace Vector on Image"])
            .alias(&["itrace"])
            .params("{handle: image entity handle, points: [[pixelX,pixelY],...] (origin at image top-left), closed?: bool, preview?: bool} → {handle?, image, vertices, worldPoints, closed, preview}. Preview performs no edit; no automatic AI recognition."),
        CommandSpec::new("imagecalibrate", "Calibrate Raster Image by Two Control Points", run_imagecalibrate)
            .menu(&["Modify", "Image", "Calibrate"])
            .alias(&["ical"])
            .params("{handle: IMAGE entity handle, pixels: [[x0,y0],[x1,y1]] (top-left origin), world: [[X0,Y0],[X1,Y1]], preview?: bool}. Solves uniform scale, rotation and translation; never guesses units."),
        CommandSpec::new("imageprojecttrace", "Trace Perspective Image into CAD", run_imageprojecttrace)
            .menu(&["Draw", "Image", "Trace Perspective Image"])
            .alias(&["iptrace"])
            .params("{handle, pixels:[[x,y] x4], world:[[X,Y] x4], points:[[pixelX,pixelY],...], closed?:bool, preview?:bool}. Projective mapping of traced vectors; does not rectify bitmap pixels."),
        CommandSpec::new("imagecalibratefit", "Fit Raster Image to Multiple Survey Control Points", run_imagecalibratefit)
            .menu(&["Modify", "Image", "Calibrate Multi-Point"])
            .alias(&["icalfit"])
            .params("{handle, pixels:[[x,y],...] (top-left origin), world:[[X,Y],...], preview?:bool, maxResidual?:positive}. 3..32 control pairs; least-squares scale/rotation/translation with RMS and max residual. No perspective correction."),
        CommandSpec::new("imageclip", "Clip Image", run_imageclip)
            .menu(&["Modify", "Clip", "Image"])
            .alias(&["iclip"])
            .params("{handle?, points?: [[x,y],...] (image pixel coordinates), delete?: bool} → {handle, clipping, points}"),
        CommandSpec::new("imageadjust", "Adjust Raster Image", run_imageadjust)
            .menu(&["Modify", "Image", "Adjust"])
            .alias(&["iadjust"])
            .params("{handle?, brightness?:0..100, contrast?:0..100, fade?:0..100, transparency?:bool, visible?:bool}"),
        CommandSpec::new("imagetransform", "Transform Raster Image", run_imagetransform)
            .menu(&["Modify", "Image", "Transform"])
            .alias(&["itransform"])
            .params("{handle?, at?:[x,y], width?:positive, height?:positive, rotation?:degrees}"),
    ]
}

fn finite_positive(n: f64, key: &str) -> Result<f64> {
    if n.is_finite() && n > 0.0 { Ok(n) } else { Err(bad("imageattach", format!("`{key}` must be a positive number"))) }
}

fn pair(p: &Value, key: &str, default: Vec2) -> Result<Vec2> {
    let Some(a) = p.get(key) else { return Ok(default) };
    let Some(a) = a.as_array() else { return Err(bad("imageattach", format!("`{key}` must be [x,y]"))) };
    let x = a.first().and_then(Value::as_f64).ok_or_else(|| bad("imageattach", format!("`{key}[0]` must be a number")))?;
    let y = a.get(1).and_then(Value::as_f64).ok_or_else(|| bad("imageattach", format!("`{key}[1]` must be a number")))?;
    if !x.is_finite() || !y.is_finite() || x <= 0.0 || y <= 0.0 { return Err(bad("imageattach", format!("`{key}` must contain positive numbers"))) }
    Ok(Vec2::new(x, y))
}

fn image_json(h: Handle, i: &Image) -> Value {
    let a = i.insert.xy();
    let b = a + i.u.xy() * i.size.x;
    let c = b + i.v.xy() * i.size.y;
    let d = a + i.v.xy() * i.size.y;
    json!({
        "handle": h.hex(), "path": i.path, "name": i.name, "insert": [a.x, a.y],
        "corners": [[a.x,a.y],[b.x,b.y],[c.x,c.y],[d.x,d.y]],
        "pixels": [i.size.x, i.size.y], "clip": i.clip.iter().map(|p| json!([p.x,p.y])).collect::<Vec<_>>(),
        "clipping": i.clipping, "brightness": i.brightness, "contrast": i.contrast, "fade": i.fade,
    })
}

fn run_imageattach(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").map(str::trim).filter(|t| !t.is_empty()).ok_or_else(|| bad("imageattach", "`path` is required"))?.to_string();
    let at = point_param(p, "at").unwrap_or(Vec2::ZERO);
    let pixels = pair(p, "pixels", Vec2::new(1.0, 1.0))?;
    let width = finite_positive(f64_or(p, "width", pixels.x), "width")?;
    let height = finite_positive(f64_or(p, "height", pixels.y), "height")?;
    let angle = f64_or(p, "rotation", 0.0).to_radians();
    if !angle.is_finite() { return Err(bad("imageattach", "`rotation` must be a number")) }
    let (sn, cs) = angle.sin_cos();
    let u = Vec3::new(cs * width / pixels.x, sn * width / pixels.x, 0.0);
    let v = Vec3::new(-sn * height / pixels.y, cs * height / pixels.y, 0.0);
    let mut i = Image { insert: at.to3(0.0), u, v, size: pixels, path: path.clone(), ..Image::default() };
    for (key, dst) in [("brightness", &mut i.brightness), ("contrast", &mut i.contrast), ("fade", &mut i.fade)] {
        if let Some(n) = p.get(key).and_then(Value::as_u64) {
            *dst = u8::try_from(n).map_err(|_| bad("imageattach", format!("`{key}` must be 0..100")))?;
            if *dst > 100 { return Err(bad("imageattach", format!("`{key}` must be 0..100"))) }
        }
    }
    let h = s.add_entity(EntityKind::Image(i.clone()))?;
    Ok(image_json(h, &i))
}


fn run_imagetrace(s: &mut Session, p: &Value) -> Result<Value> {
    let image_handle = super::curves::handle_param(p, "handle").ok_or_else(|| bad("imagetrace", "an IMAGE entity `handle` is required"))?;
    let image = match &s.doc()?.entity(image_handle).ok_or_else(|| bad("imagetrace", "image not found"))?.kind {
        EntityKind::Image(image) => image.clone(),
        _ => return Err(bad("imagetrace", "handle must refer to a raster IMAGE entity")),
    };
    if !image.size.x.is_finite() || !image.size.y.is_finite() || image.size.x <= 0.0 || image.size.y <= 0.0 {
        return Err(bad("imagetrace", "image pixel dimensions must be positive and finite"));
    }
    let points = p.get("points").and_then(Value::as_array).ok_or_else(|| bad("imagetrace", "points must be an array of [pixelX,pixelY]"))?;
    if !(2..=2048).contains(&points.len()) { return Err(bad("imagetrace", "points must contain between 2 and 2048 positions")); }
    let mut vertices = Vec::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        let xy = point.as_array().filter(|a| a.len() == 2).ok_or_else(|| bad("imagetrace", format!("point {index} must be [x,y]")))?;
        let x = xy[0].as_f64().ok_or_else(|| bad("imagetrace", format!("point {index} x must be finite")))?;
        let y = xy[1].as_f64().ok_or_else(|| bad("imagetrace", format!("point {index} y must be finite")))?;
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 || x > image.size.x || y > image.size.y {
            return Err(bad("imagetrace", format!("point {index} must be within the image's pixel bounds")));
        }
        // Image insertion is its bottom-left CAD corner; supplied pixel coordinates start top-left.
        let world = image.insert.xy() + image.u.xy() * x + image.v.xy() * (image.size.y - y);
        if !world.x.is_finite() || !world.y.is_finite() { return Err(bad("imagetrace", "image coordinate transform overflowed")); }
        vertices.push(PolyVertex::new(world));
    }
    let closed = bool_or(p, "closed", false);
    let count = vertices.len();
    let world_points: Vec<Value> = vertices.iter().map(|v| json!([v.p.x, v.p.y])).collect();
    // AI suggestions should be reviewable before they modify the user's DWG.
    if bool_or(p, "preview", false) {
        return Ok(json!({"handle":null, "image":image_handle.hex(), "vertices":count, "worldPoints":world_points, "closed":closed, "preview":true, "units":"drawing", "origin":"image-top-left"}));
    }
    let polyline = LwPolyline { vertices, closed, const_width: 0.0, elevation: image.insert.z, plinegen: false };
    let handle = s.add_entity(EntityKind::LwPolyline(polyline))?;
    Ok(json!({"handle":handle.hex(), "image":image_handle.hex(), "vertices":count, "worldPoints":world_points, "closed":closed, "preview":false, "units":"drawing", "origin":"image-top-left", "recognition":"manual-or-agent-supplied"}))
}


fn calibration_pair(p: &Value, name: &str, cmd: &str) -> Result<[Vec2; 2]> {
    let arr = p.get(name).and_then(Value::as_array).filter(|a| a.len() == 2)
        .ok_or_else(|| bad(cmd, format!("`{name}` must have exactly two [x,y] points")))?;
    let read = |index: usize| -> Result<Vec2> {
        let xy = arr[index].as_array().filter(|a| a.len() == 2)
            .ok_or_else(|| bad(cmd, format!("`{name}[{index}]` must be [x,y]")))?;
        let x = xy[0].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(cmd, format!("`{name}[{index}].x` must be finite")))?;
        let y = xy[1].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(cmd, format!("`{name}[{index}].y` must be finite")))?;
        Ok(Vec2::new(x, y))
    };
    Ok([read(0)?, read(1)?])
}

/// Fit an IMAGE using two measured control-point correspondences.
/// The input pixels start at the top left. Preserve the original IMAGE definition and
/// solve a similarity (not a shear) so any existing IMAGECLIP remains in pixel coordinates.
fn run_imagecalibrate(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "imagecalibrate";
    let h = super::curves::handle_param(p, "handle").ok_or_else(|| bad(CMD, "an IMAGE entity handle is required"))?;
    let old = match &s.doc()?.entity(h).ok_or_else(|| bad(CMD, "image not found"))?.kind {
        EntityKind::Image(i) => i.clone(),
        _ => return Err(bad(CMD, "handle must refer to a raster IMAGE entity")),
    };
    if !old.size.x.is_finite() || !old.size.y.is_finite() || old.size.x <= 0.0 || old.size.y <= 0.0 {
        return Err(bad(CMD, "image dimensions must be positive and finite"));
    }
    let pixels = calibration_pair(p, "pixels", CMD)?;
    let world = calibration_pair(p, "world", CMD)?;
    if pixels.iter().any(|v| v.x < 0.0 || v.y < 0.0 || v.x > old.size.x || v.y > old.size.y) {
        return Err(bad(CMD, "control points must lie within the raster pixel dimensions"));
    }
    let dx = pixels[1].x - pixels[0].x;
    let dy = pixels[0].y - pixels[1].y; // pixel Y goes down, CAD Y goes up
    let wx = world[1].x - world[0].x;
    let wy = world[1].y - world[0].y;
    let image_dist = dx.hypot(dy);
    let world_dist = wx.hypot(wy);
    if !image_dist.is_finite() || !world_dist.is_finite() || image_dist <= 1.0e-9 || world_dist <= 1.0e-9 {
        return Err(bad(CMD, "two distinct pixel and world points are required"));
    }
    let scale = world_dist / image_dist;
    if !scale.is_finite() || scale <= 0.0 { return Err(bad(CMD, "calibration scale is not finite")); }
    let rotation = wy.atan2(wx) - dy.atan2(dx);
    let (sn, cs) = rotation.sin_cos();
    let u = Vec2::new(cs * scale, sn * scale);
    let v = Vec2::new(-sn * scale, cs * scale);
    let insert = world[0] - u * pixels[0].x - v * (old.size.y - pixels[0].y);
    if [u.x, u.y, v.x, v.y, insert.x, insert.y].iter().any(|n| !n.is_finite()) {
        return Err(bad(CMD, "calibration transform exceeds coordinate limits"));
    }
    let preview = bool_or(p, "preview", false);
    if !preview {
        let mut image = old;
        image.insert.x = insert.x;
        image.insert.y = insert.y;
        image.u = Vec3::new(u.x, u.y, 0.0);
        image.v = Vec3::new(v.x, v.y, 0.0);
        s.doc_mut()?.modify_entity(h, |entity| entity.kind = EntityKind::Image(image.clone()))?;
        s.touch();
    }
    Ok(json!({"image":h.hex(), "preview":preview, "pixelsPerDrawingUnit":1.0 / scale,
        "drawingUnitsPerPixel":scale, "rotationDegrees":rotation.to_degrees(),
        "insert":[insert.x,insert.y], "u":[u.x,u.y], "v":[v.x,v.y],
        "worldControlPoints":[[world[0].x,world[0].y],[world[1].x,world[1].y]]}))
}


fn calibration_points(p: &Value, name: &str, cmd: &str) -> Result<Vec<Vec2>> {
    let list = p.get(name).and_then(Value::as_array).filter(|a| (3..=32).contains(&a.len()))
        .ok_or_else(|| bad(cmd, format!("`{name}` requires 3..32 [x,y] points")))?;
    list.iter().enumerate().map(|(i, value)| {
        let a = value.as_array().filter(|a| a.len() == 2).ok_or_else(|| bad(cmd, format!("`{name}[{i}]` must be [x,y]")))?;
        let x = a[0].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(cmd, format!("`{name}[{i}].x` must be finite")))?;
        let y = a[1].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(cmd, format!("`{name}[{i}].y` must be finite")))?;
        Ok(Vec2::new(x, y))
    }).collect()
}

/// Least-squares similarity fit with multiple surveyed control points. Independent of pixel
/// resolution, this is a 2D similarity fit only: perspective/affine distortion is NOT corrected.
fn run_imagecalibratefit(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "imagecalibratefit";
    let h = super::curves::handle_param(p, "handle").ok_or_else(|| bad(CMD, "an IMAGE entity handle is required"))?;
    let old = match &s.doc()?.entity(h).ok_or_else(|| bad(CMD, "image not found"))?.kind {
        EntityKind::Image(i) => i.clone(),
        _ => return Err(bad(CMD, "handle must refer to a raster IMAGE entity")),
    };
    if !old.size.x.is_finite() || !old.size.y.is_finite() || old.size.x <= 0.0 || old.size.y <= 0.0 {
        return Err(bad(CMD, "image pixel dimensions must be finite and positive"));
    }
    let pixels = calibration_points(p, "pixels", CMD)?;
    let world = calibration_points(p, "world", CMD)?;
    if pixels.len() != world.len() { return Err(bad(CMD, "pixel and CAD control point counts must match")); }
    if pixels.iter().any(|q| q.x < 0.0 || q.y < 0.0 || q.x > old.size.x || q.y > old.size.y) {
        return Err(bad(CMD, "all pixel controls must lie inside the raster"));
    }
    let n = pixels.len() as f64;
    let mean_pixel = pixels.iter().fold(Vec2::ZERO, |sum, q| sum + Vec2::new(q.x, old.size.y - q.y)) / n;
    let mean_world = world.iter().fold(Vec2::ZERO, |sum, q| sum + *q) / n;
    let mut denominator = 0.0;
    let mut cross = 0.0;
    let mut dot = 0.0;
    for (pixel, target) in pixels.iter().zip(&world) {
        let q = Vec2::new(pixel.x, old.size.y - pixel.y) - mean_pixel;
        let w = *target - mean_world;
        denominator += q.x * q.x + q.y * q.y;
        dot += q.x * w.x + q.y * w.y;
        cross += q.x * w.y - q.y * w.x;
    }
    if !denominator.is_finite() || denominator <= 1.0e-12 { return Err(bad(CMD, "pixel controls must not all coincide")); }
    let a = dot / denominator;
    let b = cross / denominator;
    let scale = a.hypot(b);
    if !scale.is_finite() || scale <= 1.0e-12 { return Err(bad(CMD, "CAD controls collapse or produce invalid scale")); }
    let u = Vec2::new(a, b);
    let v = Vec2::new(-b, a);
    let insert = mean_world - u * mean_pixel.x - v * mean_pixel.y;
    if [u.x,u.y,v.x,v.y,insert.x,insert.y].iter().any(|x| !x.is_finite()) {
        return Err(bad(CMD, "calibration exceeds finite drawing coordinate limits"));
    }
    let residuals: Vec<f64> = pixels.iter().zip(&world).map(|(pixel, target)| {
        let y = old.size.y - pixel.y;
        let estimate = insert + u * pixel.x + v * y;
        (estimate - *target).len()
    }).collect();
    let max_error = residuals.iter().copied().fold(0.0_f64, f64::max);
    let rms_error = (residuals.iter().map(|e| e * e).sum::<f64>() / n).sqrt();
    if !max_error.is_finite() || !rms_error.is_finite() { return Err(bad(CMD, "residual is not finite")); }
    if let Some(limit) = p.get("maxResidual") {
        let limit = limit.as_f64().filter(|x| x.is_finite() && *x > 0.0).ok_or_else(|| bad(CMD, "`maxResidual` must be positive and finite"))?;
        if max_error > limit { return Err(bad(CMD, format!("max residual {max_error:.6} exceeds allowed {limit:.6} CAD units; image not modified"))); }
    }
    let preview = bool_or(p, "preview", false);
    if !preview {
        let mut image = old;
        image.insert.x = insert.x;
        image.insert.y = insert.y;
        image.u = Vec3::new(u.x, u.y, 0.0);
        image.v = Vec3::new(v.x, v.y, 0.0);
        s.doc_mut()?.modify_entity(h, |e| e.kind = EntityKind::Image(image.clone()))?;
        s.touch();
    }
    Ok(json!({"image":h.hex(),"preview":preview,"controlCount":pixels.len(),"drawingUnitsPerPixel":scale,
        "rotationDegrees":b.atan2(a).to_degrees(),"insert":[insert.x,insert.y],
        "rmsResidual":rms_error,"maxResidual":max_error,"residuals":residuals,
        "model":"least-squares-uniform-scale-rotation-translation","perspectiveCorrected":false}))
}


fn image_homography_coefficients(pixels: &[Vec2], world: &[Vec2], width: f64, height: f64) -> Result<([f64; 8], Vec2, f64)> {
    const CMD: &str = "imageprojecttrace";
    if pixels.len() != 4 || world.len() != 4 { return Err(bad(CMD, "exactly four pixel and world controls required")); }
    // Four controls must follow the quadrilateral perimeter (clockwise or counterclockwise).
    // A crossed or concave set can fold the drawing and turn finite pixel lines into invalid CAD.
    let is_convex = |points: &[Vec2]| {
        let mut winding = 0.0_f64;
        let scale = points.iter().map(|p| (*p - points[0]).len()).fold(0.0_f64, f64::max);
        if !scale.is_finite() || scale <= 1.0e-12 { return false; }
        for i in 0..4 {
            let a = points[(i + 1) % 4] - points[i];
            let b = points[(i + 2) % 4] - points[(i + 1) % 4];
            let cross = (a.x * b.y - a.y * b.x) / (scale * scale);
            if !cross.is_finite() || cross.abs() <= 1.0e-9 { return false; }
            if i != 0 && cross.signum() != winding { return false; }
            winding = cross.signum();
        }
        true
    };
    if !is_convex(pixels) || !is_convex(world) {
        return Err(bad(CMD, "the four pixel and CAD controls must form non-crossed convex quadrilaterals in matching perimeter order"));
    }
    // Normalize both domains so pivoting remains stable for large projected CAD coordinates.
    let offset = world[0];
    let world_scale = world.iter().map(|q| (*q - offset).len()).fold(0.0_f64, f64::max);
    if !world_scale.is_finite() || world_scale <= 1.0e-12 { return Err(bad(CMD, "world control points are degenerate")); }
    let mut m = [[0.0_f64; 9]; 8];
    for i in 0..4 {
        let x = pixels[i].x / width;
        let y = pixels[i].y / height;
        let u = (world[i].x - offset.x) / world_scale;
        let v = (world[i].y - offset.y) / world_scale;
        m[i * 2] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        m[i * 2 + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }
    // Gauss-Jordan with partial pivoting; reject singular/near-singular calibrations.
    for column in 0..8 {
        let mut pivot = column;
        for row in (column + 1)..8 {
            if m[row][column].abs() > m[pivot][column].abs() { pivot = row; }
        }
        if !m[pivot][column].is_finite() || m[pivot][column].abs() <= 1.0e-10 {
            return Err(bad(CMD, "projective control configuration is singular or too close to collinear"));
        }
        m.swap(column, pivot);
        let divisor = m[column][column];
        for k in column..9 { m[column][k] /= divisor; }
        for row in 0..8 {
            if row == column { continue; }
            let factor = m[row][column];
            for k in column..9 { m[row][k] -= factor * m[column][k]; }
        }
    }
    let mut coeff = [0.0_f64; 8];
    for i in 0..8 { coeff[i] = m[i][8]; }
    if coeff.iter().any(|v| !v.is_finite()) { return Err(bad(CMD, "computed projective transform is not finite")); }
    Ok((coeff, offset, world_scale))
}

fn project_image_point(x: f64, y: f64, width: f64, height: f64, coeff: &[f64; 8], offset: Vec2, world_scale: f64) -> Result<Vec2> {
    const CMD: &str = "imageprojecttrace";
    let x = x / width;
    let y = y / height;
    let denominator = coeff[6] * x + coeff[7] * y + 1.0;
    if !denominator.is_finite() || denominator.abs() <= 1.0e-10 { return Err(bad(CMD, "projected point is at infinity; check the four control points")); }
    let u = (coeff[0] * x + coeff[1] * y + coeff[2]) / denominator;
    let v = (coeff[3] * x + coeff[4] * y + coeff[5]) / denominator;
    let point = offset + Vec2::new(u * world_scale, v * world_scale);
    if !point.x.is_finite() || !point.y.is_finite() { return Err(bad(CMD, "projective coordinate overflow")); }
    Ok(point)
}

/// Perspective-correct traced *vectors* from a tilted photograph. This does not resample or
/// display image pixels; the IMAGE entity's placement and clipping are left untouched.
fn run_imageprojecttrace(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "imageprojecttrace";
    let h = super::curves::handle_param(p, "handle").ok_or_else(|| bad(CMD, "raster IMAGE handle is required"))?;
    let image = match &s.doc()?.entity(h).ok_or_else(|| bad(CMD, "image not found"))?.kind {
        EntityKind::Image(i) => i.clone(),
        _ => return Err(bad(CMD, "handle must refer to an IMAGE entity")),
    };
    if !image.size.x.is_finite() || !image.size.y.is_finite() || image.size.x <= 0.0 || image.size.y <= 0.0 {
        return Err(bad(CMD, "image dimensions must be finite and positive"));
    }
    let pixels = calibration_points(p, "pixels", CMD)?;
    let world = calibration_points(p, "world", CMD)?;
    if pixels.len() != 4 || world.len() != 4 { return Err(bad(CMD, "perspective mapping requires exactly four corresponding control points")); }
    let in_bounds = |v: &Vec2| v.x >= 0.0 && v.y >= 0.0 && v.x <= image.size.x && v.y <= image.size.y;
    if pixels.iter().any(|q| !in_bounds(q)) { return Err(bad(CMD, "pixel control points must be inside the raster")); }
    let (coeff, offset, world_scale) = image_homography_coefficients(&pixels, &world, image.size.x, image.size.y)?;
    let input = p.get("points").and_then(Value::as_array).filter(|a| (2..=2048).contains(&a.len()))
        .ok_or_else(|| bad(CMD, "points must contain between 2 and 2048 pixel vertices"))?;
    let mut vertices = Vec::with_capacity(input.len());
    for (i, point) in input.iter().enumerate() {
        let xy = point.as_array().filter(|a| a.len() == 2).ok_or_else(|| bad(CMD, format!("point {i} must be [x,y]")))?;
        let x = xy[0].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(CMD, format!("point {i} x must be finite")))?;
        let y = xy[1].as_f64().filter(|v| v.is_finite()).ok_or_else(|| bad(CMD, format!("point {i} y must be finite")))?;
        if !in_bounds(&Vec2::new(x, y)) { return Err(bad(CMD, format!("point {i} lies outside the raster"))); }
        vertices.push(PolyVertex::new(project_image_point(x, y, image.size.x, image.size.y, &coeff, offset, world_scale)?));
    }
    let controls_error = pixels.iter().zip(&world).map(|(px, expected)| {
        project_image_point(px.x, px.y, image.size.x, image.size.y, &coeff, offset, world_scale)
            .map(|got| (got - *expected).len())
    }).collect::<Result<Vec<_>>>()?;
    let max_control_error = controls_error.into_iter().fold(0.0_f64, f64::max);
    let closed = bool_or(p, "closed", false);
    let preview = bool_or(p, "preview", false);
    let points: Vec<Value> = vertices.iter().map(|v| json!([v.p.x,v.p.y])).collect();
    if preview {
        return Ok(json!({"handle":null,"image":h.hex(),"preview":true,"closed":closed,"vertices":vertices.len(),
            "worldPoints":points,"maxControlError":max_control_error,"perspectiveMapped":true,"rasterRectified":false}));
    }
    let count = vertices.len();
    let entity = LwPolyline { vertices, closed, const_width: 0.0, elevation: image.insert.z, plinegen: false };
    let handle = s.add_entity(EntityKind::LwPolyline(entity))?;
    Ok(json!({"handle":handle.hex(),"image":h.hex(),"preview":false,"closed":closed,"vertices":count,
        "worldPoints":points,"maxControlError":max_control_error,"perspectiveMapped":true,"rasterRectified":false}))
}

fn image_handle(s: &Session, p: &Value) -> Result<Handle> {
    if let Some(h) = super::curves::handle_param(p, "handle") { return Ok(h) }
    let hs = targets(s, p)?;
    hs.first().copied().ok_or_else(|| bad("imageclip", "an image handle or selected image is required"))
}

fn run_imageclip(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imageclip", "image not found"))?;
    let EntityKind::Image(image) = &old.kind else { return Err(bad("imageclip", "selected object is not an image")); };
    let size = image.size;
    let delete = bool_or(p, "delete", false);
    let points = if delete { Vec::new() } else {
        let a = p.get("points").and_then(Value::as_array).ok_or_else(|| bad("imageclip", "`points` is required unless `delete` is true"))?;
        let mut out = Vec::with_capacity(a.len());
        for q in a {
            let x = q.get(0).and_then(Value::as_f64).ok_or_else(|| bad("imageclip", "clip points must be [x,y]"))?;
            let y = q.get(1).and_then(Value::as_f64).ok_or_else(|| bad("imageclip", "clip points must be [x,y]"))?;
            if !x.is_finite() || !y.is_finite() { return Err(bad("imageclip", "clip points must be finite")) }
            out.push(Vec2::new(x, y));
        }
        if !(2..=128).contains(&out.len()) { return Err(bad("imageclip", "image clip requires 2..128 points")); }
        if out.iter().any(|q| q.x < 0.0 || q.y < 0.0 || q.x > size.x || q.y > size.y) {
            return Err(bad("imageclip", "clip points must remain inside the raster pixel dimensions"));
        }
        if out.len() == 2 && ((out[0].x - out[1].x).abs() <= f64::EPSILON || (out[0].y - out[1].y).abs() <= f64::EPSILON) {
            return Err(bad("imageclip", "rectangular clip must have nonzero width and height"));
        }
        out
    };
    s.doc_mut()?.modify_entity(h, |e| {
        if let EntityKind::Image(i) = &mut e.kind {
            i.clip = points.clone();
            i.clipping = !delete;
            if delete { i.display &= !4; } else { i.display |= 4; }
        }
    })?;
    let i = match &s.doc()?.entity(h).ok_or_else(|| bad("imageclip", "image not found after update"))?.kind {
        EntityKind::Image(i) => i.clone(),
        _ => return Err(bad("imageclip", "image not found after update")),
    };
    Ok(image_json(h, &i))
}



fn run_imageadjust(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imageadjust", "image not found"))?;
    let EntityKind::Image(old_i) = &old.kind else { return Err(bad("imageadjust", "selected object is not an image")); };
    let mut next = old_i.clone();
    for (key, dst) in [("brightness", &mut next.brightness), ("contrast", &mut next.contrast), ("fade", &mut next.fade)] {
        if let Some(value) = p.get(key) {
            let n = value.as_u64().filter(|n| *n <= 100).ok_or_else(|| bad("imageadjust", format!("`{key}` must be an integer from 0 to 100")))?;
            *dst = n as u8;
        }
    }
    if let Some(value) = p.get("transparency") {
        let on = value.as_bool().ok_or_else(|| bad("imageadjust", "`transparency` must be true or false"))?;
        if on { next.display |= 8; } else { next.display &= !8; }
    }
    let visible = p.get("visible").map(|v| v.as_bool().ok_or_else(|| bad("imageadjust", "`visible` must be true or false"))).transpose()?;
    s.doc_mut()?.modify_entity(h, |e| {
        e.kind = EntityKind::Image(next.clone());
        if let Some(v) = visible { e.common.visible = v; }
    })?;
    s.touch();
    let mut out = image_json(h, &next);
    out["transparency"] = json!((next.display & 8) != 0);
    out["visible"] = json!(s.doc()?.entity(h).is_some_and(|e| e.common.visible));
    Ok(out)
}

fn run_imagetransform(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imagetransform", "image not found"))?;
    let EntityKind::Image(old_i) = &old.kind else { return Err(bad("imagetransform", "selected object is not an image")); };
    let mut next = old_i.clone();
    let old_width = old_i.u.xy().len() * old_i.size.x;
    let old_height = old_i.v.xy().len() * old_i.size.y;
    let width = p.get("width").and_then(Value::as_f64).unwrap_or(old_width);
    let height = p.get("height").and_then(Value::as_f64).unwrap_or(old_height);
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(bad("imagetransform", "width and height must be positive finite numbers"));
    }
    let current_angle = old_i.u.y.atan2(old_i.u.x).to_degrees();
    let angle = p.get("rotation").and_then(Value::as_f64).unwrap_or(current_angle);
    if !angle.is_finite() { return Err(bad("imagetransform", "rotation must be finite degrees")); }
    if let Some(at) = p.get("at") {
        let a = at.as_array().ok_or_else(|| bad("imagetransform", "`at` must be [x,y]"))?;
        if a.len() != 2 { return Err(bad("imagetransform", "`at` must be [x,y]")); }
        let x = a[0].as_f64().unwrap_or(f64::NAN);
        let y = a[1].as_f64().unwrap_or(f64::NAN);
        if !x.is_finite() || !y.is_finite() { return Err(bad("imagetransform", "`at` coordinates must be finite")); }
        next.insert.x = x;
        next.insert.y = y;
    }
    let (sn, cs) = angle.to_radians().sin_cos();
    next.u = Vec3::new(cs * width / next.size.x, sn * width / next.size.x, 0.0);
    next.v = Vec3::new(-sn * height / next.size.y, cs * height / next.size.y, 0.0);
    s.doc_mut()?.modify_entity(h, |e| e.kind = EntityKind::Image(next.clone()))?;
    s.touch();
    let mut out = image_json(h, &next);
    out["width"] = json!(width);
    out["height"] = json!(height);
    out["rotation"] = json!(angle);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imagecalibrate_two_points_previews_then_places_image_without_changing_pixel_geometry() {
        let mut s = Session::new();
        let image = s.execute("imageattach", &json!({"path":"site.png","at":[0,0],"pixels":[200,100],"width":200,"height":100})).unwrap();
        let h = image["handle"].as_str().unwrap();
        let p = json!({"handle":h,"pixels":[[0,100],[200,100]],"world":[[10,20],[30,20]],"preview":true});
        let preview = s.execute("imagecalibrate", &p).unwrap();
        assert_eq!(preview["insert"], json!([10.0,20.0]));
        let before = s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap();
        assert_eq!(before["worldPoints"], json!([[0.0,0.0],[200.0,0.0]]));
        let mut commit = p.clone();
        commit["preview"] = json!(false);
        assert_eq!(s.execute("imagecalibrate", &commit).unwrap()["preview"], false);
        let after = s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap();
        assert_eq!(after["worldPoints"], json!([[10.0,20.0],[30.0,20.0]]));
        assert!(s.execute("imagecalibrate", &json!({"handle":h,"pixels":[[0,0],[0,0]],"world":[[0,0],[1,1]]})).is_err());
        assert!(s.execute("imagecalibrate", &json!({"handle":h,"pixels":[[0,0],[201,0]],"world":[[0,0],[1,1]]})).is_err());
    }

    #[test]
    fn imageprojecttrace_maps_tilted_photo_corners_without_modifying_raster() {
        let mut s = Session::new();
        let img = s.execute("imageattach", &json!({"path":"photo.png","pixels":[200,100],"width":200,"height":100})).unwrap();
        let h = img["handle"].as_str().unwrap();
        let mut data = json!({"handle":h, "pixels":[[0,0],[200,0],[200,100],[0,100]],
            "world":[[10,20],[40,15],[35,45],[5,40]],"points":[[0,0],[200,0],[200,100],[0,100]],"closed":true,"preview":true});
        let proposal = s.execute("imageprojecttrace", &data).unwrap();
        assert_eq!(proposal["handle"], Value::Null);
        assert_eq!(s.execute("entities", &json!({"type":"Polyline"})).unwrap()["count"], 0);
        assert!(proposal["maxControlError"].as_f64().unwrap() < 1.0e-7);
        for (actual, expected) in proposal["worldPoints"].as_array().unwrap().iter().zip(data["world"].as_array().unwrap()) {
            for j in 0..2 { assert!((actual[j].as_f64().unwrap() - expected[j].as_f64().unwrap()).abs() < 1.0e-7); }
        }
        data["preview"] = json!(false);
        let committed = s.execute("imageprojecttrace", &data).unwrap();
        assert!(committed["handle"].is_string());
        assert_eq!(s.execute("entities", &json!({"type":"Polyline"})).unwrap()["count"], 1);
        let original_image = s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap();
        assert_eq!(original_image["worldPoints"], json!([[0.0,0.0],[200.0,0.0]]));
        data["pixels"] = json!([[0,0],[0,100],[200,0],[200,100]]);
        assert!(s.execute("imageprojecttrace", &data).is_err(), "crossed quadrilateral must not fold the vector drawing");
        data["pixels"] = json!([[0,0],[0,0],[200,100],[0,100]]);
        assert!(s.execute("imageprojecttrace", &data).is_err());
        assert_eq!(s.execute("entities", &json!({"type":"Polyline"})).unwrap()["count"], 1);
    }

    #[test]
    fn imagecalibratefit_uses_four_controls_and_refuses_bad_residual_without_edit() {
        let mut s = Session::new();
        let image = s.execute("imageattach", &json!({"path":"scan.png","pixels":[200,100],"width":200,"height":100})).unwrap();
        let h = image["handle"].as_str().unwrap();
        let fit = json!({"handle":h, "pixels":[[0,100],[200,100],[0,0],[200,0]],
            "world":[[10,20],[30,20],[10,30],[30,30]],"preview":true,"maxResidual":0.001});
        let preview = s.execute("imagecalibratefit", &fit).unwrap();
        assert_eq!(preview["controlCount"], 4);
        assert!(preview["maxResidual"].as_f64().unwrap() < 1.0e-9);
        assert_eq!(s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap()["worldPoints"], json!([[0.0,0.0],[200.0,0.0]]));
        let mut approved = fit.clone();
        approved["preview"] = json!(false);
        s.execute("imagecalibratefit", &approved).unwrap();
        let mapped = s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap();
        assert!((mapped["worldPoints"][0][0].as_f64().unwrap() - 10.0).abs() < 1.0e-9);
        assert!((mapped["worldPoints"][1][0].as_f64().unwrap() - 30.0).abs() < 1.0e-9);
        let original = mapped["worldPoints"].clone();
        let mut bad = approved.clone();
        bad["world"][3] = json!([80.0,80.0]);
        assert!(s.execute("imagecalibratefit", &bad).is_err());
        assert_eq!(s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[200,100]],"preview":true})).unwrap()["worldPoints"], original);
    }

    #[test]
    fn imagecalibrate_handles_rotated_raster_and_rejects_collapsed_world_controls() {
        let mut s = Session::new();
        let image = s.execute("imageattach", &json!({"path":"rotated.png","at":[0,0],"pixels":[200,100],"width":200,"height":100})).unwrap();
        let h = image["handle"].as_str().unwrap();
        let result = s.execute("imagecalibrate", &json!({"handle":h,"pixels":[[0,100],[0,0]],"world":[[10,20],[20,20]]})).unwrap();
        assert!((result["drawingUnitsPerPixel"].as_f64().unwrap() - 0.1).abs() < 1e-12);
        let trace = s.execute("imagetrace", &json!({"handle":h,"points":[[0,100],[0,0],[200,0]],"preview":true})).unwrap();
        let positions = trace["worldPoints"].as_array().unwrap();
        let check = |idx: usize, x: f64, y: f64| {
            let p = positions[idx].as_array().unwrap();
            assert!((p[0].as_f64().unwrap() - x).abs() < 1e-9);
            assert!((p[1].as_f64().unwrap() - y).abs() < 1e-9);
        };
        check(0, 10.0, 20.0);
        check(1, 20.0, 20.0);
        check(2, 20.0, 0.0);
        assert!(s.execute("imagecalibrate", &json!({"handle":h,"pixels":[[0,100],[200,100]],"world":[[10,20],[10,20]]})).is_err());
    }

    #[test]
    fn imagetrace_maps_image_pixels_into_editable_cad_geometry() {
        let mut s = Session::new();
        let image = s.execute("imageattach", &json!({"path":"site.png","at":[10.0,20.0],"pixels":[200.0,100.0],"width":20.0,"height":10.0})).unwrap();
        let before_preview = s.doc().unwrap().entity_count();
        let proposed = s.execute("imagetrace", &json!({"handle":image["handle"],"points":[[0.0,100.0],[200.0,100.0],[200.0,0.0]],"preview":true})).unwrap();
        assert_eq!(proposed["preview"], true);
        assert_eq!(proposed["worldPoints"], json!([[10.0,20.0],[30.0,20.0],[30.0,30.0]]));
        assert_eq!(s.doc().unwrap().entity_count(), before_preview, "preview must not insert any CAD geometry");
        let traced = s.execute("imagetrace", &json!({"handle":image["handle"],"points":[[0.0,100.0],[200.0,100.0],[200.0,0.0]],"closed":false})).unwrap();
        assert_eq!(traced["vertices"], 3);
        let handle = Handle::parse_hex(traced["handle"].as_str().unwrap()).unwrap();
        let entity = s.doc().unwrap().entity(handle).unwrap();
        let EntityKind::LwPolyline(polyline) = &entity.kind else { panic!("trace must be editable CAD polyline") };
        assert_eq!(polyline.vertices[0].p, Vec2::new(10.0,20.0));
        assert_eq!(polyline.vertices[1].p, Vec2::new(30.0,20.0));
        assert_eq!(polyline.vertices[2].p, Vec2::new(30.0,30.0));
        assert!(s.execute("imagetrace", &json!({"handle":image["handle"],"points":[[0.0,0.0],[201.0,100.0]]})).is_err());
        assert!(s.execute("imagetrace", &json!({"handle":image["handle"],"points":[[0.0,0.0]]})).is_err());
        assert!(s.execute("imagetrace", &json!({"handle":traced["handle"],"points":[[0.0,0.0],[1.0,1.0]]})).is_err());
    }

    #[test]
    fn imageattach_and_imageclip_keep_dxf_metadata() {
        let mut s = Session::new();
        let r = s
            .execute("imageattach", &json!({"path": "survey/site.png", "at": [10.0, 20.0], "width": 100.0, "height": 50.0, "pixels": [2000.0, 1000.0], "rotation": 15.0}))
            .unwrap();
        let h = r["handle"].as_str().expect("image handle").to_string();
        assert_eq!(r["path"], "survey/site.png");
        assert_eq!(r["pixels"], json!([2000.0, 1000.0]));
        let c = s.execute("imageclip", &json!({"handle": h, "points": [[0.0, 0.0], [2000.0, 0.0], [2000.0, 1000.0], [0.0, 1000.0]]})).unwrap();
        assert_eq!(c["clipping"], true);
        assert!(s.execute("imageclip", &json!({"handle": h, "points": [[-1.0,0.0],[100.0,100.0]]})).is_err());
        assert!(s.execute("imageclip", &json!({"handle": h, "points": [[0.0,0.0],[0.0,100.0]]})).is_err());
        let d = s.execute("imageclip", &json!({"handle": c["handle"], "delete": true})).unwrap();
        assert_eq!(d["clipping"], false);
        let a = s.execute("imageadjust", &json!({"handle": h, "brightness": 70, "contrast": 60, "fade": 15, "transparency": true})).unwrap();
        assert_eq!(a["brightness"], 70);
        assert_eq!(a["transparency"], true);
        assert!(s.execute("imageadjust", &json!({"handle":h,"brightness":"very bright"})).is_err());
        assert!(s.execute("imageadjust", &json!({"handle":h,"contrast":101})).is_err());
        assert!(s.execute("imageadjust", &json!({"handle":h,"fade":0.5})).is_err());
        assert!(s.execute("imageadjust", &json!({"handle":h,"visible":"yes"})).is_err());
        let t = s.execute("imagetransform", &json!({"handle": a["handle"], "at":[5.0,6.0], "width":25.0, "height":10.0, "rotation":30.0})).unwrap();
        assert_eq!(t["width"], 25.0);
        assert_eq!(t["height"], 10.0);
    }
}
