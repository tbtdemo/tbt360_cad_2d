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
        json!({"name":"raster_to_cad_lines","description":"AUTOMATIC image-to-CAD vectorization: read a local PNG/JPEG, detect horizontal, vertical and 45-degree dark strokes plus optional isolated arbitrary-angle line candidates, optionally bridge collinear gaps with maxGapPixels=0..12 (default 0; disabled to preserve doors and openings), and optionally convert four complete rectangular outlines to editable closed PLINE when closedRectangles=true (default false), avoiding duplicate LINE edges. Broken boundaries are never silently closed. Optional wallPairHints=true reports candidate parallel edge pairs in preview only; these are geometric hints, not recognized BIM walls. createWallCenterlines=true additionally proposes native editable LINE centerlines only for unique, non-ambiguous pairs, requires wallPairHints=true and preserves source edge lines; never generates BIM walls. No manual pixel points needed. Default preview:true returns world-space lines and changes nothing; preview:false commits. This is deterministic pixel extraction, not neural understanding of walls, doors or dimensions. Save as DXF/DWG after review.", "inputSchema":{"type":"object","properties":{"path":{"type":"string"},"preview":{"type":"boolean"},"threshold":{"type":"integer","minimum":0},"minLengthPixels":{"type":"integer","minimum":2},"unitsPerPixel":{"type":"number","minimum":0},"origin":{"type":"array","items":{"type":"number"},"minItems":2,"maxItems":2},"maxLines":{"type":"integer","minimum":1},"includeDiagonals":{"type":"boolean"},"includeArbitraryAngles":{"type":"boolean"},"maxGapPixels":{"type":"integer","minimum":0,"maximum":12},"closedRectangles":{"type":"boolean"},"wallPairHints":{"type":"boolean"},"createWallCenterlines":{"type":"boolean"}},"required":["path"]}}),
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
