//! MCP tool definitions and dispatch.

use serde_json::{Value, json};

use crate::backend::Backend;

/// The `tools/list` payload: one definition (name, description, input schema) per MCP tool,
/// including the UI-only tools when `has_ui` says a window is available.
pub fn tool_definitions(has_ui: bool) -> Value {
    let mut tools = vec![
        json!({"name": "command_line", "description": "Type one line at the CADCraft command line, exactly as a user would at an AutoCAD-style prompt. Starts commands by name or alias (LINE, L, CIRCLE, C, PLINE, RECTANG, MOVE, TRIM, OFFSET, ZOOM…) and answers the active prompt: points `x,y`, relative `@dx,dy`, polar `@dist<angle`, distances, keywords (e.g. `c` for Close), empty text = Enter. Spaces act as Enter, so `circle 0,0 5` works in one call. Returns the new prompt and command-line output, and `error` when the line was refused.",
            "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}}),
        json!({"name": "script", "description": "Run a multi-line script (like an AutoCAD .scr file): one or more inputs per line, blank line = Enter.",
            "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}}),
        json!({"name": "execute", "description": "Run any command with JSON parameters (no prompts or dialogs). Use list_commands to find ids and parameter docs. Examples: line {points:[[0,0],[10,0]]}, circle {center:[0,0], radius:5}, layer.new {name:\"Walls\", color:\"red\", current:true}, move {handles:[\"10A\"], delta:[5,0]}, zoom {mode:\"extents\"}.",
            "inputSchema": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}),
        json!({"name":"ai_draw_plan_vietnamese","description":"CAD 360 AI deterministic multi-step Vietnamese drawing plan. Supply 2-8 native commands through newline-delimited drafts (LINE, CIRCLE, RECTANG, PLINE), or one high-level room pair: vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0 (one outer RECTANG plus one shared LINE, no duplicate room boundaries). Preview is default; commit requires confirm:true and exact document identity/revision from preview. Executes all steps as one guarded Undo group and attempts safe rollback on failure. Does not interpret arbitrary natural language.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"preview":{"type":"boolean"},"confirm":{"type":"boolean"},"expectedDocumentUid":{"type":"integer","minimum":0},"expectedDocumentEpoch":{"type":"integer","minimum":0},"expectedRevision":{"type":"integer","minimum":0}},"required":["text"]}}),
        json!({"name":"ai_draw_vietnamese","description":"CAD 360 Vietnamese drafting bridge (bounded deterministic grammar, not a general LLM). Supports bounded LINE, CIRCLE, RECTANG, PLINE, plus dimensioned room 'vẽ phòng rộng 6000 dài 4000 tại 0,0' and wall 'vẽ tường từ 0,0 đến 6000,0 dày 200'. Room and wall are editable CAD outlines, NOT semantically verified BIM geometry. Preview by default; to commit, set preview:false, confirm:true and expectedDocumentUid, expectedDocumentEpoch, expectedRevision obtained from a fresh preview. Executes exactly one native CAD engine command, never saves or modifies DWG structure.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"preview":{"type":"boolean"},"confirm":{"type":"boolean"},"expectedDocumentUid":{"type":"integer","minimum":0},"expectedDocumentEpoch":{"type":"integer","minimum":0},"expectedRevision":{"type":"integer","minimum":0}},"required":["text"]}}),
        json!({"name": "list_commands", "description": "List commands with ids, labels, menu paths, aliases and parameter docs. Optional filter: a substring of the command id, label or an alias.",
            "inputSchema": {"type": "object", "properties": {"filter": {"type": "string"}}}}),
        json!({"name": "inspect_drawing", "description": "Summary of the active drawing: entity counts, extents, layers, styles, blocks, layouts, selection, undo history and (optionally) entities with handles and geometry.",
            "inputSchema": {"type": "object", "properties": {"entities": {"type": "boolean"}, "limit": {"type": "integer"}}}}),
        json!({"name": "query_entities", "description": "Entities filtered by type (Line, Circle, Polyline…), layer, current selection (selected:true; active space only) and/or a window [[x0,y0],[x1,y1]] (objects whose geometry is inside or crosses it; crossing:false = entirely inside). Returns count (all matches), returned (this page) and entities; page with offset and limit (default 500).",
            "inputSchema": {"type": "object", "properties": {"type": {"type": "string"}, "layer": {"type": "string"}, "selected": {"type": "boolean"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "crossing": {"type": "boolean"}, "limit": {"type": "integer", "minimum": 0}, "offset": {"type": "integer", "minimum": 0}}}}),
        json!({"name": "render", "description": "Render the active drawing to a PNG and return it as an image: fitted to the extents by default, or just the area `window` [[x0,y0],[x1,y1]] (drawing units), e.g. one sheet or one room of a large drawing. With a window and no size the image is 1600 px wide with the window's shape.",
            "inputSchema": {"type": "object", "properties": {"width": {"type": "integer", "minimum": 16}, "height": {"type": "integer", "minimum": 16}, "fit": {"type": "boolean"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "path": {"type": "string"}}}}),
        json!({"name": "calibrate_image", "description": "Calibrate an attached raster IMAGE from two pixel/world control-point pairs; computes uniform scale, rotation and placement. Call preview:true first, confirm, then preview:false to update only that image.", "inputSchema": {"type": "object", "properties": {"handle": {"type": "string"}, "pixels": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2}, "world": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2}, "preview": {"type": "boolean"}}, "required": ["handle", "pixels", "world"]}}),
        json!({"name":"trace_perspective_image","description":"Map 2-2048 pixel points on a tilted photo into editable CAD geometry with four pixel-to-world control correspondences. Projective vector mapping only, not image bitmap rectification; preview:true does not edit the DWG.", "inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"pixels":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":4,"maxItems":4},"world":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":4,"maxItems":4},"points":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":2,"maxItems":2048},"closed":{"type":"boolean"},"preview":{"type":"boolean"}},"required":["handle","pixels","world","points"]}}),
        json!({"name":"fit_image_controls","description":"Fit raster IMAGE to 3-32 measured pixel/world control points using least-squares scale, rotation and translation. Returns RMS/max residual. Set maxResidual to block inaccurate commits; use preview:true first. Does not correct perspective.", "inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"pixels":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":3,"maxItems":32},"world":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":3,"maxItems":32},"preview":{"type":"boolean"},"maxResidual":{"type":"number","minimum":0}},"required":["handle","pixels","world"]}}),
        json!({"name": "trace_quality_check", "description": "Quality-gate a candidate vector trace from image AI BEFORE sending it to trace_image/trace_perspective_image. Geometry only: detect short or duplicate edges and self-intersections, report bounds and segment lengths, without modifying CAD. This does NOT perform visual recognition or prove the trace matches the pixels.", "inputSchema": {"type": "object", "properties": {"points": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2048}, "closed": {"type": "boolean"}, "minSegmentLength": {"type": "number", "minimum": 0}}, "required": ["points"]}}),
        json!({"name": "trace_image", "description": "Convert points in an attached IMAGE pixel coordinate system into a real editable CAD polyline. Vision inference must happen outside CAD.", "inputSchema": {"type": "object", "properties": {"handle": {"type": "string"}, "points": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2048}, "closed": {"type": "boolean"}, "preview": {"type": "boolean"}}, "required": ["handle", "points"]}}),
        json!({"name":"trace_image_validated","description":"AI-safe IMAGE pixel-to-editable-CAD polyline: performs geometry quality checks and rejects malformed, short-segment, or self-crossing traces BEFORE any drawing edit. Supports preview:true; vision recognition must happen outside CAD. minSegmentLength is measured in input IMAGE pixels.", "inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"points":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":2,"maxItems":2048},"closed":{"type":"boolean"},"preview":{"type":"boolean"},"minSegmentLength":{"type":"number","minimum":0}},"required":["handle","points"]}}),
        json!({"name":"trace_perspective_image_validated","description":"AI-safe perspective-photo trace: quality-check pixel-space outline before mapping into editable CAD using 4 control pairs. Rejects invalid geometry without modifying CAD, supports preview:true. minSegmentLength uses input pixels; preview and world-space inspection remain recommended.", "inputSchema":{"type":"object","properties":{"handle":{"type":"string"},"pixels":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":4,"maxItems":4},"world":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":4,"maxItems":4},"points":{"type":"array","items":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"minItems":2,"maxItems":2048},"closed":{"type":"boolean"},"preview":{"type":"boolean"},"minSegmentLength":{"type":"number","minimum":0}},"required":["handle","pixels","world","points"]}}),
        json!({"name":"raster_to_cad_lines","description":"AUTOMATIC image-to-CAD vectorization: read a local PNG/JPEG, detect horizontal, vertical and 45-degree dark strokes plus optional isolated arbitrary-angle line candidates, optionally bridge collinear gaps with maxGapPixels=0..12 (default 0; disabled to preserve doors and openings), and optionally convert four complete rectangular outlines to editable closed PLINE when closedRectangles=true (default false), avoiding duplicate LINE edges. Broken boundaries are never silently closed. Optional wallPairHints=true reports candidate parallel edge pairs in preview only; these are geometric hints, not recognized BIM walls. Optional openingGapHints=true reports short interruptions in collinear line segments as unverified openings, never inserts a missing door/window or bridges the gap. Configurable minOpeningGapPixels/maxOpeningGapPixels default 3..40 (valid 1..120, minimum<=maximum); opening candidates include world-space fromCAD/toCAD and measured widthCAD based on unitsPerPixel. createWallCenterlines=true additionally proposes native editable LINE centerlines only for unique, non-ambiguous pairs, requires wallPairHints=true and preserves source edge lines; never generates BIM walls. Optional minWallSpacingPixels/maxWallSpacingPixels (default 2..40, allowed 1..80, min<=max) narrow candidate pairing for different raster scales. No manual pixel points needed. Default preview:true returns world-space lines and changes nothing; preview:false commits. This is deterministic pixel extraction, not neural understanding of walls, doors or dimensions. Save as DXF/DWG after review.", "inputSchema":{"type":"object","properties":{"path":{"type":"string"},"preview":{"type":"boolean"},"threshold":{"type":"integer","minimum":0},"minLengthPixels":{"type":"integer","minimum":2},"unitsPerPixel":{"type":"number","minimum":0},"origin":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"maxLines":{"type":"integer","minimum":1},"includeDiagonals":{"type":"boolean"},"includeArbitraryAngles":{"type":"boolean"},"maxGapPixels":{"type":"integer","minimum":0,"maximum":12},"closedRectangles":{"type":"boolean"},"wallPairHints":{"type":"boolean"},"createWallCenterlines":{"type":"boolean"},"minWallSpacingPixels":{"type":"number","minimum":1},"maxWallSpacingPixels":{"type":"number","minimum":1},"openingGapHints":{"type":"boolean"},"minOpeningGapPixels":{"type":"number","minimum":1,"maximum":120},"maxOpeningGapPixels":{"type":"number","minimum":1,"maximum":120}},"required":["path"]}}),
        json!({"name": "find_text", "description": "Read the drawing's texts: TEXT, MTEXT (formatting removed), block attributes and multileaders, each with its text, insertion point `at`, height, rotation, style, layer and handle. Filter by `contains` (any case), `layer` and/or `window` [[x0,y0],[x1,y1]] around the insertion points; page with offset and limit (default 200). Use it to find room names, sheet titles and notes without reading all the geometry.",
            "inputSchema": {"type": "object", "properties": {"contains": {"type": "string"}, "layer": {"type": "string"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "limit": {"type": "integer", "minimum": 0}, "offset": {"type": "integer", "minimum": 0}}}}),
        json!({"name": "new_drawing", "description": "Create a new drawing (imperial by default, or metric).",
            "inputSchema": {"type": "object", "properties": {"metric": {"type": "boolean"}}}}),
        json!({"name": "open", "description": "Open a DXF or DWG file.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}}),
        json!({"name": "save", "description": "Save the drawing (DXF or DWG by extension). A .svg, .png or .pdf path exports instead: the drawing keeps its name and unsaved changes.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}}),
        json!({"name": "cancel", "description": "Press Escape: cancel the running command (or clear the selection).", "inputSchema": {"type": "object", "properties": {}}}),
    ];
    if has_ui {
        tools.push(json!({"name": "screenshot", "description": "Screenshot the CADCraft window (PNG).", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}}));
        tools.push(json!({"name": "ui_inspect", "description": "UI state: panels, canvas rect, view, cursor and snap, performance.", "inputSchema": {"type": "object", "properties": {}}}));
        tools.push(json!({"name": "ui_click", "description": "Click in the drawing area at world coordinates (or screen with space:\"screen\"). Right button = Enter.", "inputSchema": {"type": "object", "properties": {"x": {"type": "number"}, "y": {"type": "number"}, "space": {"type": "string"}, "button": {"type": "string"}}, "required": ["x", "y"]}}));
    }
    json!({ "tools": tools })
}

/// Validate candidate AI vectorization geometry without touching a drawing. The result
/// describes geometric defects only, never pixel-level recognition accuracy.
fn check_trace_quality(args: &Value) -> Value {
    let Some(points) = args["points"].as_array() else { return json!({"safeToCommit":false,"error":"missing points array"}) };
    let closed = args["closed"].as_bool().unwrap_or(false);
    let min_len = args["minSegmentLength"].as_f64().unwrap_or(1.0e-8);
    let mut pts = Vec::with_capacity(points.len());
    for (i, item) in points.iter().enumerate() {
        let Some(x) = item[0].as_f64() else { return json!({"safeToCommit":false,"error":format!("point {i} x is not a finite number")}) };
        let Some(y) = item[1].as_f64() else { return json!({"safeToCommit":false,"error":format!("point {i} y is not a finite number")}) };
        if !x.is_finite() || !y.is_finite() || x.abs() > 1.0e12 || y.abs() > 1.0e12 {
            return json!({"safeToCommit":false,"error":format!("point {i} is non-finite or outside CAD safety bounds")});
        }
        pts.push((x, y));
    }
    if !min_len.is_finite() || min_len < 0.0 || min_len > 1.0e12 {
        return json!({"safeToCommit":false,"error":"invalid minSegmentLength"});
    }
    let seg_count = pts.len() - usize::from(!closed);
    let mut too_short = Vec::new();
    let mut min_found = f64::INFINITY;
    let mut max_found: f64 = 0.0;
    for i in 0..seg_count {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        min_found = min_found.min(len);
        max_found = max_found.max(len);
        if len <= min_len { too_short.push(i); }
    }
    // Broad phase: sort segment bounding boxes by their minimum X. Only boxes
    // overlapping on both axes can intersect; avoids O(n²) orientation work on
    // the large, spatially distributed polylines returned by image recognition.
    // A dense set of overlapping boxes still has O(n²) worst-case cost.
    let mut boxes = Vec::with_capacity(seg_count);
    for i in 0..seg_count {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        boxes.push((i, a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1)));
    }
    boxes.sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    let mut intersections = Vec::new();
    let mut candidate_pairs = 0usize;
    for (position, a) in boxes.iter().enumerate() {
        for b in boxes.iter().skip(position + 1) {
            if b.1 > a.2 { break; }
            if b.3 > a.4 || b.4 < a.3 { continue; }
            let (i, j) = (a.0.min(b.0), a.0.max(b.0));
            // Adjacent segments share a vertex by design; exclude those pairs.
            if j == i + 1 || (closed && i == 0 && j + 1 == seg_count) { continue; }
            candidate_pairs += 1;
            if trace_segments_intersect(pts[i], pts[(i + 1) % pts.len()], pts[j], pts[(j + 1) % pts.len()]) {
                intersections.push((i, j));
            }
        }
    }
    intersections.sort_unstable();
    let intersections: Vec<Value> = intersections.into_iter().map(|(i, j)| json!([i, j])).collect();
    let x_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let y_min = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let x_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let y_max = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let too_few = closed && pts.len() < 3;
    json!({
        "safeToCommit": too_short.is_empty() && intersections.is_empty() && !too_few,
        "pointCount": pts.len(), "segmentCount": seg_count, "closed": closed,
        "minimumSegmentLength": min_found, "maximumSegmentLength": max_found,
        "shortSegments": too_short, "selfIntersections": intersections,
        "intersectionCandidatePairsTested": candidate_pairs,
        "tooFewPointsForClosedOutline": too_few, "bounds": [[x_min,y_min],[x_max,y_max]],
        "scope": "Geometry validation only; raster/image recognition fidelity NOT_VERIFIED"
    })
}

/// Intersect two nonadjacent closed 2D segments, including collinear overlap.
fn trace_segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    fn orient(a: (f64,f64), b: (f64,f64), c: (f64,f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }
    fn between(a: (f64,f64), b: (f64,f64), p: (f64,f64)) -> bool {
        // Use exact segment bounds for collinear overlap. A tolerance based on
        // absolute drawing coordinates can exceed a whole CAD unit near 1e12
        // and falsely reject two separate traced objects as overlapping.
        // Near-zero segment lengths are already rejected by the quality gate.
        p.0 >= a.0.min(b.0) && p.0 <= a.0.max(b.0)
            && p.1 >= a.1.min(b.1) && p.1 <= a.1.max(b.1)
    }
    let o1 = orient(a,b,c); let o2 = orient(a,b,d);
    let o3 = orient(c,d,a); let o4 = orient(c,d,b);
    if (o1 > 0.0 && o2 < 0.0 || o1 < 0.0 && o2 > 0.0)
        && (o3 > 0.0 && o4 < 0.0 || o3 < 0.0 && o4 > 0.0) { return true; }
    (o1 == 0.0 && between(a,b,c)) || (o2 == 0.0 && between(a,b,d))
        || (o3 == 0.0 && between(c,d,a)) || (o4 == 0.0 && between(c,d,b))
}

/// An atomic quality gate at the MCP dispatch boundary: no engine command is sent
/// when AI-generated pixel geometry fails. Preserve ordinary trace commands for
/// backwards compatibility; agents should prefer the validated variants.
fn validated_trace(b: &mut dyn Backend, args: &Value, command: &str) -> Result<Value, String> {
    let quality = check_trace_quality(args);
    if quality["safeToCommit"] != true {
        return Err(format!("AI trace rejected before CAD edit: {quality}"));
    }
    // This threshold is a quality-gate parameter, not a CAD engine command field.
    let mut params = args.clone();
    if let Some(map) = params.as_object_mut() {
        map.remove("minSegmentLength");
    }
    // The image pixels can be geometrically sound while an invalid calibration
    // or perspective transformation creates degenerate or crossing CAD geometry.
    // Always ask the engine for a non-mutating mapped-world preview before edit.
    params["preview"] = json!(true);
    let preview = b.call("engine.execute", json!({"command":command,"params":params.clone()}))?;
    let Some(world_points) = preview["worldPoints"].as_array() else {
        return Err("AI trace rejected: the engine preview has no worldPoints; no drawing edit attempted".to_string());
    };
    if world_points.len() != args["points"].as_array().map_or(0, Vec::len) {
        return Err("AI trace rejected: transformed point count does not match input".to_string());
    }
    let mapped = json!({"points":world_points,"closed":args["closed"].as_bool().unwrap_or(false),"minSegmentLength":0.0});
    let world_quality = check_trace_quality(&mapped);
    if world_quality["safeToCommit"] != true {
        return Err(format!("AI trace rejected after world-coordinate projection, before CAD edit: {world_quality}"));
    }
    if args["preview"].as_bool().unwrap_or(false) {
        return Ok(preview);
    }
    params["preview"] = json!(false);
    b.call("engine.execute", json!({"command":command,"params":params}))
}

/// Strict natural-language bridge: only explicit coordinates can become native CAD commands.
/// No generative guessing, no background state, and no silent edit on the preview path.
fn parse_vietnamese_draft(text: &str) -> Result<(String,Value),String> {
    if text.len()>500 {return Err("AI CAD: lệnh vượt 500 ký tự".into());}
    let cleaned=text.trim().to_lowercase();
    let words:Vec<&str>=cleaned.split_whitespace().collect();
    let parse_point=|value:&str|->Option<[f64;2]> {
        let (x,y)=value.split_once(',')?;
        let (x,y)=(x.parse::<f64>().ok()?,y.parse::<f64>().ok()?);
        (x.is_finite()&&y.is_finite()&&x.abs()<=1.0e9&&y.abs()<=1.0e9).then_some([x,y])
    };
    let argument=|keyword:&str|->Option<&str> {words.iter().position(|w|*w==keyword).and_then(|i|words.get(i+1).copied())};
    // A dimensioned architectural drafting primitive. The "wall" is a CAD
    // closed polyline, never a verified BIM wall; openings are not inferred.
    if (cleaned.starts_with("vẽ tường ") || cleaned.starts_with("ve tuong ")) && words.len()==8 {
        let start=argument("từ").or_else(||argument("tu")).and_then(parse_point).ok_or("AI CAD: cần tọa độ đầu tường dạng x,y")?;
        let end=argument("đến").or_else(||argument("den")).and_then(parse_point).ok_or("AI CAD: cần tọa độ cuối tường dạng x,y")?;
        let thickness=argument("dày").or_else(||argument("day")).and_then(|v|v.parse::<f64>().ok()).ok_or("AI CAD: cần chiều dày tường dạng số")?;
        let dx=end[0]-start[0];let dy=end[1]-start[1];let length=dx.hypot(dy);
        if !length.is_finite()||length<1.0||!thickness.is_finite()||!(0.01..=1_000_000.0).contains(&thickness) {
            return Err("AI CAD: chiều dài tường phải >=1 và chiều dày >0, <=1000000".into());
        }
        let nx=-dy/length*thickness*0.5;let ny=dx/length*thickness*0.5;
        let vertices=[[start[0]+nx,start[1]+ny],[end[0]+nx,end[1]+ny],
            [end[0]-nx,end[1]-ny],[start[0]-nx,start[1]-ny]];
        if vertices.iter().flatten().any(|v|!v.is_finite()||v.abs()>1.0e9) {
            return Err("AI CAD: tọa độ mép tường vượt phạm vi an toàn".into());
        }
        return Ok(("pline".into(),json!({"vertices":vertices,"closed":true})));
    }
    if (cleaned.starts_with("vẽ phòng ") || cleaned.starts_with("ve phong ")) && words.len()==8 {
        let width=argument("rộng").or_else(||argument("rong")).and_then(|v|v.parse::<f64>().ok()).ok_or("AI CAD: cần chiều rộng phòng")?;
        let length=argument("dài").or_else(||argument("dai")).and_then(|v|v.parse::<f64>().ok()).ok_or("AI CAD: cần chiều dài phòng")?;
        let origin=argument("tại").or_else(||argument("tai")).and_then(parse_point).ok_or("AI CAD: cần gốc phòng x,y")?;
        if !width.is_finite()||!length.is_finite()||!(0.01..=1_000_000.0).contains(&width)||!(0.01..=1_000_000.0).contains(&length) {
            return Err("AI CAD: kích thước phòng phải dương và <=1000000".into());
        }
        let p2=[origin[0]+width,origin[1]+length];
        if p2.iter().any(|v|!v.is_finite()||v.abs()>1.0e9) {return Err("AI CAD: phòng vượt giới hạn tọa độ".into());}
        return Ok(("rectang".into(),json!({"p1":origin,"p2":p2})));
    }
    if (cleaned.starts_with("vẽ đường thẳng ") || cleaned.starts_with("ve duong thang ")) && words.len()==7 {
        let start=argument("từ").or_else(||argument("tu")).and_then(parse_point).ok_or("AI CAD: cần tọa độ bắt đầu dạng x,y")?;
        let end=argument("đến").or_else(||argument("den")).and_then(parse_point).ok_or("AI CAD: cần tọa độ kết thúc dạng x,y")?;
        if (start[0]-end[0]).hypot(start[1]-end[1])<1.0e-9 {return Err("AI CAD: đường thẳng không được có chiều dài bằng 0".into());}
        return Ok(("line".into(),json!({"points":[start,end]})));
    }
    if (cleaned.starts_with("vẽ hình chữ nhật ") || cleaned.starts_with("ve hinh chu nhat ")) && words.len()==8 {
        let start=argument("từ").or_else(||argument("tu")).and_then(parse_point).ok_or("AI CAD: cần tọa độ góc thứ nhất")?;
        let end=argument("đến").or_else(||argument("den")).and_then(parse_point).ok_or("AI CAD: cần tọa độ góc đối diện")?;
        if (start[0]-end[0]).abs()<=1.0e-9||(start[1]-end[1]).abs()<=1.0e-9 {
            return Err("AI CAD: hình chữ nhật phải có chiều rộng và chiều cao khác 0".into());
        }
        return Ok(("rectang".into(),json!({"p1":start,"p2":end})));
    }
    if cleaned.starts_with("vẽ đa tuyến qua ") || cleaned.starts_with("ve da tuyen qua ") {
        let prefix=if cleaned.starts_with("vẽ đa tuyến qua ") {"vẽ đa tuyến qua "} else {"ve da tuyen qua "};
        let body=&cleaned[prefix.len()..];
        let mut points=Vec::new();
        for chunk in body.split(';') {
            let point=parse_point(chunk.trim()).ok_or("AI CAD: đa tuyến phải là danh sách x,y ngăn cách bằng dấu ;")?;
            if points.last()==Some(&point) {return Err("AI CAD: đa tuyến có hai đỉnh liên tiếp trùng nhau".into());}
            points.push(point);
            if points.len()>128 {return Err("AI CAD: tối đa 128 đỉnh".into());}
        }
        if points.len()<2 {return Err("AI CAD: đa tuyến cần ít nhất 2 đỉnh".into());}
        return Ok(("pline".into(),json!({"vertices":points,"closed":false})));
    }
    if (cleaned.starts_with("vẽ hình tròn ") || cleaned.starts_with("ve hinh tron ")) && words.len()==8 {
        let center=argument("tâm").or_else(||argument("tam")).and_then(parse_point).ok_or("AI CAD: cần tọa độ tâm dạng x,y")?;
        let radius=argument("kính").and_then(|v|v.parse::<f64>().ok()).ok_or("AI CAD: cần bán kính dạng số dương")?;
        if !radius.is_finite()||radius<=0.0||radius>1.0e9 {return Err("AI CAD: bán kính không hợp lệ".into());}
        return Ok(("circle".into(),json!({"center":center,"radius":radius})));
    }
    Err("AI CAD: câu lệnh ngoài phạm vi hỗ trợ hoặc thiếu tọa độ: LINE, CIRCLE, RECTANG, PLINE dạng danh sách x,y phân cách bởi dấu ;".into())
}

fn ai_draw_vietnamese(b: &mut dyn Backend, args: &Value) -> Result<Value,String> {
    let prompt=args["text"].as_str().ok_or("AI CAD: thiếu nội dung lệnh")?;
    let (command,params)=parse_vietnamese_draft(prompt)?;
    let drawing=b.call("drawing.inspect",json!({"entities":false}))?;
    let uid=drawing["documentUid"].as_u64().ok_or("AI CAD: không lấy được documentUid")?;
    let epoch=drawing["documentEpoch"].as_u64().ok_or("AI CAD: không lấy được documentEpoch")?;
    let rev=drawing["revision"].as_u64().ok_or("AI CAD: không lấy được revision")?;
    let plan=json!({"mode":"native-engine-command","command":command,"params":params,
        "documentUid":uid,"documentEpoch":epoch,"revision":rev,
        "requiresConfirmation":true,"canUndo":true,"source":"bounded-vietnamese-command-parser"});
    if args["preview"].as_bool().unwrap_or(true) {return Ok(json!({"previewOnly":true,"editableEntitiesCreated":0,"plan":plan}));}
    if args["confirm"]!=true {return Err("AI CAD: cần confirm=true để thực hiện lệnh đã xem trước".into());}
    if args["expectedDocumentUid"].as_u64()!=Some(uid)||args["expectedDocumentEpoch"].as_u64()!=Some(epoch)||args["expectedRevision"].as_u64()!=Some(rev) {
        return Err("AI CAD: bản vẽ đã thay đổi hoặc thiếu dấu phiên bản; hãy Preview lại trước khi thực hiện".into());
    }
    if drawing["undoControl"]!="all"||drawing["undoGroupOpen"]!=false {return Err("AI CAD: Undo chưa sẵn sàng hoặc đang có nhóm Undo khác".into());}
    let cmdline=b.call("cmdline.state",json!({}))?;
    if cmdline["running"].as_str().is_some() {return Err("AI CAD: hãy kết thúc lệnh CAD đang chạy".into());}
    let result=b.call("engine.execute",json!({"command":command,"params":params}))?;
    Ok(json!({"previewOnly":false,"executed":true,"command":command,"result":result,
        "documentUid":uid,"previousRevision":rev,"undo":"Use native CAD UNDO for this single command"}))
}

/// Stage 229: finite plan grammar, one CAD command per line. Never split on ';':
/// that separator is part of PLINE vertex input, not a multi-command delimiter.
/// Stage 231: two adjacent rectangular rooms, with ONE outer footprint and
/// ONE shared partition instead of two overlapping room boundary polylines.
/// The dividing LINE is a drafting guide, not an inferred BIM wall/door.
fn parse_two_adjacent_rooms(text: &str) -> Result<Option<Vec<(String,Value)>>,String> {
    let lower=text.trim().to_lowercase();
    if !lower.starts_with("vẽ hai phòng liền kề ") {return Ok(None);}
    let words:Vec<&str>=lower.split_whitespace().collect();
    if words.len()!=13 || words[5]!="rộng" || words[7]!="và" || words[9]!="dài" || words[11]!="tại" {
        return Err("AI CAD: cú pháp phải là 'vẽ hai phòng liền kề rộng W1 và W2 dài L tại X,Y'".into());
    }
    let width1=words[6].parse::<f64>().map_err(|_|"AI CAD: W1 không hợp lệ")?;
    let width2=words[8].parse::<f64>().map_err(|_|"AI CAD: W2 không hợp lệ")?;
    let length=words[10].parse::<f64>().map_err(|_|"AI CAD: L không hợp lệ")?;
    let parts:Vec<&str>=words[12].split(',').collect();
    if parts.len()!=2 {return Err("AI CAD: điểm gốc phải là X,Y".into());}
    let x=parts[0].parse::<f64>().map_err(|_|"AI CAD: X không hợp lệ")?;
    let y=parts[1].parse::<f64>().map_err(|_|"AI CAD: Y không hợp lệ")?;
    if [width1,width2,length].iter().any(|v|!v.is_finite()||!(0.01..=1_000_000.0).contains(v)) ||
        [x,y].iter().any(|v|!v.is_finite()||v.abs()>1.0e9) {
        return Err("AI CAD: kích thước hoặc tọa độ phòng vượt phạm vi an toàn".into());
    }
    let right=x+width1+width2;let mid=x+width1;let top=y+length;
    if [right,mid,top].iter().any(|v|!v.is_finite()||v.abs()>1.0e9) {
        return Err("AI CAD: tọa độ sau tính toán vượt phạm vi an toàn".into());
    }
    Ok(Some(vec![
        ("rectang".into(),json!({"p1":[x,y],"p2":[right,top]})),
        ("line".into(),json!({"points":[[mid,y],[mid,top]]}))
    ]))
}

fn parse_vietnamese_plan(text: &str) -> Result<Vec<(String,Value)>,String> {
    if text.len()>4000 {return Err("AI CAD: kế hoạch vượt 4000 ký tự".into());}
    let commands:Vec<&str>=text.lines().map(str::trim).filter(|line|!line.is_empty()).collect();
    if commands.is_empty() || commands.len()>8 {return Err("AI CAD: kế hoạch phải có 1 đến 8 dòng, mỗi dòng một yêu cầu".into());}
    let mut steps=Vec::new();
    for (index,command) in commands.iter().enumerate() {
        if let Some(adjacent)=parse_two_adjacent_rooms(command).map_err(|e|format!("AI CAD: bước {}: {e}",index+1))? {
            steps.extend(adjacent);
        } else {
            let item=parse_vietnamese_draft(command).map_err(|e|format!("AI CAD: bước {}: {e}",index+1))?;
            steps.push(item);
        }
        if steps.len()>8 {return Err("AI CAD: kế hoạch tạo quá 8 lệnh CAD".into());}
    }
    if steps.len()<2 {return Err("AI CAD: kế hoạch cần ít nhất 2 lệnh CAD; dùng ai_draw_vietnamese cho một lệnh".into());}
    Ok(steps)
}

/// Execute a confirmed small plan as ONE Undo transaction. Before every step and
/// before END/rollback, validate ownership to avoid undoing another user's edits.
fn ai_draw_plan_vietnamese(b: &mut dyn Backend, args: &Value) -> Result<Value,String> {
    let prompt=args["text"].as_str().ok_or("AI CAD: thiếu kế hoạch vẽ")?;
    let steps=parse_vietnamese_plan(prompt)?;
    let drawing=b.call("drawing.inspect",json!({"entities":false}))?;
    let uid=drawing["documentUid"].as_u64().ok_or("AI CAD: không có documentUid")?;
    let epoch=drawing["documentEpoch"].as_u64().ok_or("AI CAD: không có documentEpoch")?;
    let revision=drawing["revision"].as_u64().ok_or("AI CAD: không có revision")?;
    let summaries:Vec<Value>=steps.iter().map(|(command,params)|json!({"command":command,"params":params})).collect();
    let plan=json!({"steps":summaries,"stepCount":steps.len(),"documentUid":uid,
        "documentEpoch":epoch,"revision":revision,"requiresConfirmation":true,
        "transaction":"single-undo-group","source":"bounded-vietnamese-parser-v1"});
    if args["preview"].as_bool().unwrap_or(true) {
        return Ok(json!({"previewOnly":true,"editableEntitiesCreated":0,"plan":plan}));
    }
    if args["confirm"]!=true {return Err("AI CAD: phải xác nhận confirm=true trước khi thực thi kế hoạch".into());}
    if args["expectedDocumentUid"].as_u64()!=Some(uid)||
        args["expectedDocumentEpoch"].as_u64()!=Some(epoch)||
        args["expectedRevision"].as_u64()!=Some(revision) {
        return Err("AI CAD: bản vẽ hoặc revision thay đổi; hãy xem Preview mới".into());
    }
    if drawing["undoControl"]!="all"||drawing["undoGroupOpen"]!=false {
        return Err("AI CAD: Undo Control All chưa sẵn sàng hoặc có giao dịch khác".into());
    }
    let cmdline=b.call("cmdline.state",json!({}))?;
    if cmdline["running"].as_str().is_some() {
        return Err("AI CAD: phải kết thúc lệnh CAD đang chạy".into());
    }
    let group=|b:&mut dyn Backend,mode:&str|b.call("engine.execute",json!({"command":"undo","params":{"mode":mode}}));
    group(b,"begin")?;
    let mut results=Vec::new();
    let mut failure=None;
    for (command,params) in &steps {
        let expected=revision.checked_add(results.len() as u64);
        let latest=b.call("drawing.inspect",json!({"entities":false}));
        match latest {
            Ok(ref state) if state["documentUid"].as_u64()==Some(uid)
                && state["documentEpoch"].as_u64()==Some(epoch)
                && state["revision"].as_u64()==expected
                && state["undoGroupOpen"]==true && state["undoControl"]=="all" => {}
            Ok(_) => {failure=Some("Tài liệu bị thay đổi hoặc mất quyền giao dịch trong khi AI đang vẽ".to_string());break;}
            Err(error) => {failure=Some(format!("Không kiểm tra được phiên bản tài liệu: {error}"));break;}
        }
        match b.call("engine.execute",json!({"command":command,"params":params})) {
            Ok(value)=>results.push(value),
            Err(error)=>{failure=Some(format!("Lệnh {command} thất bại: {error}"));break;}
        }
    }
    let owned=b.call("drawing.inspect",json!({"entities":false})).is_ok_and(|state|
        state["documentUid"].as_u64()==Some(uid) && state["documentEpoch"].as_u64()==Some(epoch)
        && state["revision"].as_u64()==revision.checked_add(results.len() as u64)
        && state["undoGroupOpen"]==true && state["undoControl"]=="all");
    if !owned {return Err("AI CAD: mất quyền sở hữu Undo group hoặc tài liệu đã bị sửa ngoài luồng; không gửi END/UNDO để tránh ảnh hưởng dữ liệu khác. Hãy kiểm tra bản vẽ".into());}
    if let Err(error)=group(b,"end") {return Err(format!("AI CAD: đóng Undo group không thành công: {error}. Hãy kiểm tra tài liệu trước khi lưu"));}
    if let Some(error)=failure {
        if results.is_empty() {return Err(format!("AI CAD: kế hoạch dừng trước khi tạo đối tượng: {error}"));}
        let safe=b.call("drawing.inspect",json!({"entities":false})).is_ok_and(|state|
            state["documentUid"].as_u64()==Some(uid)&&state["documentEpoch"].as_u64()==Some(epoch)
            && state["revision"].as_u64()==revision.checked_add(results.len() as u64)
            && state["undoGroupOpen"]==false&&state["undoControl"]=="all");
        if !safe {return Err(format!("AI CAD: {error}. Không thể tự Undo do tài liệu đã thay đổi; hãy kiểm tra bản vẽ"));}
        return match b.call("engine.execute",json!({"command":"undo","params":{"count":1}})) {
            Ok(_)=>Err(format!("AI CAD: đã rollback {} bước sau lỗi: {error}",results.len())),
            Err(rollback)=>Err(format!("AI CAD: {error}; Undo tự động thất bại: {rollback}. Kiểm tra bản vẽ trước khi lưu")),
        };
    }
    Ok(json!({"previewOnly":false,"executed":true,"stepCount":results.len(),
        "results":results,"documentUid":uid,"previousRevision":revision,
        "transaction":"single-undo-group","undo":"one native CAD UNDO"}))
}

/// Wrap a JSON value as a successful MCP result containing pretty-printed text.
fn text(v: &Value) -> Value {
    json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(v).unwrap_or_default()}]})
}

/// Wrap an error message as an MCP tool error result (`isError`), not a protocol error.
fn err_text(e: &str) -> Value {
    json!({"content": [{"type": "text", "text": e}], "isError": true})
}

/// Whether `v` fits the subset of JSON Schema the tool definitions use: `type`, `items`,
/// `minItems`, `maxItems` and `minimum`.
fn fits(v: &Value, schema: &Value) -> bool {
    let typed = match schema["type"].as_str() {
        Some("string") => v.is_string(),
        // JSON Schema integers include numbers such as 2.0.
        Some("integer") => v.as_f64().is_some_and(|f| f.fract() == 0.0),
        Some("number") => v.is_number(),
        Some("boolean") => v.is_boolean(),
        Some("array") => v.is_array(),
        Some("object") => v.is_object(),
        _ => true,
    };
    let len = v.as_array().map(Vec::len);
    typed
        && schema["minimum"].as_f64().is_none_or(|m| v.as_f64().is_none_or(|f| f >= m))
        && schema["minItems"].as_u64().is_none_or(|m| len.is_none_or(|n| n as u64 >= m))
        && schema["maxItems"].as_u64().is_none_or(|m| len.is_none_or(|n| n as u64 <= m))
        && (schema["items"].is_null() || v.as_array().is_none_or(|a| a.iter().all(|i| fits(i, &schema["items"]))))
}

/// Arguments checked against the tool's input schema: an object, required properties present and
/// each given property fitting its schema (`null` counts as not given). `Err` is the
/// invalid-params message.
fn check_args(name: &str, args: &Value) -> Result<Value, String> {
    let defs = tool_definitions(true);
    let schema = defs["tools"].as_array().and_then(|t| t.iter().find(|t| t["name"] == name)).map(|t| t["inputSchema"].clone());
    let Some(schema) = schema else { return Err(format!("unknown tool `{name}`")) };
    let a = match args {
        Value::Null => json!({}),
        Value::Object(_) => args.clone(),
        _ => return Err(format!("`arguments` of `{name}` must be an object")),
    };
    for key in schema["required"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        if a.get(key).is_none_or(Value::is_null) {
            return Err(format!("`{name}` needs `{key}`"));
        }
    }
    for (key, prop) in schema["properties"].as_object().into_iter().flatten() {
        if let Some(v) = a.get(key).filter(|v| !v.is_null())
            && !fits(v, prop)
        {
            return Err(format!("`{key}` of `{name}` must match {prop}"));
        }
    }
    Ok(a)
}

/// Whether a command-catalog entry's id, label or one of its aliases contains `f` (lower case).
fn command_matches(c: &Value, f: &str) -> bool {
    let has = |s: &str| s.to_ascii_lowercase().contains(f);
    ["id", "label"].iter().any(|k| c[*k].as_str().is_some_and(has))
        || c["aliases"].as_array().is_some_and(|a| a.iter().filter_map(Value::as_str).any(has))
}

/// Run a tool. `Err` is a protocol error (unknown tool, or arguments that don't fit its input
/// schema), answered with JSON-RPC invalid params; a failing tool is an `isError` result.
pub fn call_tool(b: &mut dyn Backend, name: &str, args: &Value) -> Result<Value, String> {
    let a = check_args(name, args)?;
    let r = match name {
        "command_line" => b.call("cmdline.input", a),
        "script" => b.call("cmdline.script", a),
        "execute" => b.call(
            "engine.execute",
            json!({"command": a.get("command").cloned().unwrap_or(Value::Null), "params": a.get("params").cloned().unwrap_or(json!({}))}),
        ),
        "ai_draw_vietnamese" => ai_draw_vietnamese(b, &a),
        "ai_draw_plan_vietnamese" => ai_draw_plan_vietnamese(b, &a),
        "list_commands" => b.call("engine.commands", json!({})).map(|v| {
            let f = a.get("filter").and_then(Value::as_str).map(str::to_ascii_lowercase);
            match (v, f) {
                (Value::Array(items), Some(f)) => Value::Array(items.into_iter().filter(|c| command_matches(c, &f)).collect()),
                (v, _) => v,
            }
        }),
        "inspect_drawing" => b.call("drawing.inspect", a),
        "query_entities" => b.call("engine.execute", json!({"command": "entities", "params": a})),
        "trace_perspective_image" => b.call("engine.execute", json!({"command": "imageprojecttrace", "params": a})),
        "fit_image_controls" => b.call("engine.execute", json!({"command": "imagecalibratefit", "params": a})),
        "calibrate_image" => b.call("engine.execute", json!({"command": "imagecalibrate", "params": a})),
        "trace_quality_check" => return Ok(text(&check_trace_quality(&a))),
        "trace_image_validated" => validated_trace(b, &a, "imagetrace"),
        "trace_perspective_image_validated" => validated_trace(b, &a, "imageprojecttrace"),
        "trace_image" => b.call("engine.execute", json!({"command": "imagetrace", "params": a})),
        "raster_to_cad_lines" => crate::raster_vector::run(b, &a),
        "find_text" => b.call("engine.execute", json!({"command": "texts", "params": a})), 
        "new_drawing" => b.call("engine.execute", json!({"command": "new", "params": a})),
        "open" => b.call("app.open", a),
        "save" => b.call("app.save", a),
        "cancel" => b.call("cmdline.key", json!({"key": "escape"})),
        "screenshot" => b.call("ui.screenshot", a),
        "ui_inspect" => b.call("ui.inspect", json!({})),
        "ui_click" => b.call("ui.pointer", a),
        "render" => {
            return Ok(match b.call("ui.render", a) {
                Ok(v) => match v.get("pngBase64").and_then(Value::as_str) {
                    Some(data) => json!({"content": [{"type": "image", "data": data, "mimeType": "image/png"}]}),
                    None => text(&v),
                },
                Err(e) => err_text(&e),
            });
        }
        other => return Err(format!("unknown tool `{other}`")),
    };
    Ok(match r {
        Ok(v) => text(&v),
        Err(e) => err_text(&e),
    })
}

#[cfg(test)]
mod ai_draft_vietnamese_tests {
    use super::parse_vietnamese_draft;
    use serde_json::json;

    #[test]
    fn ai_vietnamese_draft_parses_native_line_and_circle() {
        assert_eq!(parse_vietnamese_draft("vẽ đường thẳng từ 0,0 đến 100,50").unwrap(),("line".to_string(),json!({"points":[[0.0,0.0],[100.0,50.0]]})));
        assert_eq!(parse_vietnamese_draft("vẽ hình tròn tâm 10,20 bán kính 25").unwrap(),("circle".to_string(),json!({"center":[10.0,20.0],"radius":25.0})));
        assert_eq!(parse_vietnamese_draft("vẽ hình chữ nhật từ 0,0 đến 100,50").unwrap(),("rectang".to_string(),json!({"p1":[0.0,0.0],"p2":[100.0,50.0]})));
        assert_eq!(parse_vietnamese_draft("vẽ đa tuyến qua 0,0; 100,0; 100,50").unwrap(),("pline".to_string(),json!({"vertices":[[0.0,0.0],[100.0,0.0],[100.0,50.0]],"closed":false})));
        assert_eq!(parse_vietnamese_draft("vẽ phòng rộng 6000 dài 4000 tại 0,0").unwrap(),("rectang".to_string(),json!({"p1":[0.0,0.0],"p2":[6000.0,4000.0]})));
        assert_eq!(parse_vietnamese_draft("vẽ tường từ 0,0 đến 6000,0 dày 200").unwrap(),("pline".to_string(),json!({"vertices":[[0.0,100.0],[6000.0,100.0],[6000.0,-100.0],[0.0,-100.0]],"closed":true})));
        assert_eq!(parse_vietnamese_draft("vẽ tường từ 0,0 đến 0,6000 dày 200").unwrap(),("pline".to_string(),json!({"vertices":[[-100.0,0.0],[-100.0,6000.0],[100.0,6000.0],[100.0,0.0]],"closed":true})));
    }

    #[test]
    fn ai_vietnamese_plan_parses_four_native_commands() {
        let source="vẽ đường thẳng từ 0,0 đến 10,0\nvẽ hình tròn tâm 10,10 bán kính 5\nvẽ hình chữ nhật từ 0,0 đến 50,30\nvẽ đa tuyến qua 0,0; 15,0; 15,20";
        let steps=super::parse_vietnamese_plan(source).unwrap();
        assert_eq!(steps.len(),4);
        assert_eq!(steps[0].0,"line");
        assert_eq!(steps[1].0,"circle");
        assert_eq!(steps[2].0,"rectang");
        assert_eq!(steps[3].0,"pline");
        assert_eq!(steps[3].1["vertices"].as_array().map(Vec::len),Some(3));
    }

    #[test]
    fn ai_vietnamese_room_wall_plan_produces_closed_native_outlines() {
        let plan=super::parse_vietnamese_plan("vẽ phòng rộng 6000 dài 4000 tại 0,0\nvẽ tường từ 0,0 đến 6000,0 dày 200").unwrap();
        assert_eq!(plan.len(),2);
        assert_eq!(plan[0].0,"rectang");
        assert_eq!(plan[0].1["p2"],json!([6000.0,4000.0]));
        assert_eq!(plan[1].0,"pline");
        assert_eq!(plan[1].1["closed"],true);
        assert_eq!(plan[1].1["vertices"].as_array().map(Vec::len),Some(4));
        assert_eq!(plan[1].1["vertices"][0],json!([0.0,100.0]));
        assert_eq!(plan[1].1["vertices"][2],json!([6000.0,-100.0]));
    }

    #[test]
    fn ai_vietnamese_adjacent_rooms_generate_one_envelope_and_shared_boundary() {
        let steps=super::parse_vietnamese_plan("vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0").unwrap();
        assert_eq!(steps.len(),2);
        assert_eq!(steps[0],("rectang".to_string(),json!({"p1":[0.0,0.0],"p2":[7000.0,4000.0]})));
        assert_eq!(steps[1],("line".to_string(),json!({"points":[[3000.0,0.0],[3000.0,4000.0]]})));
        assert!(super::parse_vietnamese_plan("vẽ hai phòng liền kề rộng 0 và 4000 dài 4000 tại 0,0").is_err());
        assert!(super::parse_vietnamese_plan("vẽ hai phòng liền kề rộng 3000 và 4000 dài NaN tại 0,0").is_err());
        assert!(super::parse_vietnamese_plan("vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 1000000000,0").is_err());
    }

    #[test]
    fn ai_vietnamese_plan_is_atomic_on_parser_validation() {
        assert!(super::parse_vietnamese_plan("vẽ đường thẳng từ 0,0 đến 10,0").is_err());
        assert!(super::parse_vietnamese_plan("vẽ đường thẳng từ 0,0 đến 10,0\nxóa tất cả").is_err());
        let too_many=vec!["vẽ đường thẳng từ 0,0 đến 10,0";9].join("\n");
        assert!(super::parse_vietnamese_plan(&too_many).is_err());
    }

    #[test]
    fn ai_vietnamese_draft_rejects_missing_or_unsafe_coordinates() {
        for prompt in ["vẽ đường thẳng từ 0,0 đến 0,0","vẽ đường thẳng từ 0,0 đến NaN,2",
            "vẽ hình tròn tâm 0,0 bán kính -5","vẽ hình tròn tâm 0,0 bán kính 0",
            "vẽ tất cả tường và xóa bản vẽ","vẽ đường thẳng từ 0,0 đến 100000000000,0", "vẽ hình chữ nhật từ 0,0 đến 0,50", "vẽ đa tuyến qua 0,0;0,0", "vẽ tường từ 0,0 đến 0,0 dày 200", "vẽ tường từ 0,0 đến 6000,0 dày -200", "vẽ phòng rộng 0 dài 500 tại 0,0"] {
            assert!(parse_vietnamese_draft(prompt).is_err(),"{prompt} must be refused");
        }
    }
}
