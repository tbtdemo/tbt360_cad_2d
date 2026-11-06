//! An agent completes realistic drafting tasks through MCP only, and verifies through MCP only.

use serde_json::{Value, json};

use crate::{Headless, Server};

/// Send one JSON-RPC request to the server and return the parsed reply; fails the test if the
/// server does not answer or answers with invalid JSON.
fn call(s: &mut Server, id: u64, method: &str, params: Value) -> Value {
    let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
    let r = s.handle_line(&line).expect("reply");
    serde_json::from_str(&r).unwrap()
}

/// Call an MCP tool and return its decoded result: JSON text is parsed, an image is passed
/// through, and a tool error fails the test with the reported message.
fn tool(s: &mut Server, name: &str, args: Value) -> Value {
    let r = call(s, 99, "tools/call", json!({"name": name, "arguments": args}));
    let c = &r["result"]["content"][0];
    assert_ne!(r["result"]["isError"], json!(true), "{name} failed: {c}");
    if c["type"] == "text" { serde_json::from_str(c["text"].as_str().unwrap()).unwrap_or(Value::Null) } else { c.clone() }
}


#[test]
fn ai_perspective_trace_remains_editable_after_dxf_save_reopen() {
    let mut server = Server::new(Box::new(Headless::default()));
    let image = tool(&mut server, "execute", json!({"command":"imageattach","params":{"path":"reference.png","pixels":[200,100],"width":200,"height":100}}));
    tool(&mut server, "trace_perspective_image", json!({"handle":image["handle"],"pixels":[[0,0],[200,0],[200,100],[0,100]],
        "world":[[10,20],[40,15],[35,45],[5,40]],"points":[[0,0],[200,0],[200,100],[0,100]],"closed":true}));
    let identity = format!("cad360-ai-photo-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let path = std::env::temp_dir().join(format!("{identity}.dxf"));
    let filename = path.to_string_lossy().to_string();
    tool(&mut server, "save", json!({"path":filename}));
    assert!(path.exists());
    tool(&mut server, "new_drawing", json!({"metric":true}));
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    tool(&mut server, "open", json!({"path":filename}));
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Polyline"}))["count"], 1,
        "AI projective vector must persist as editable DXF LWPOLYLINE");
    let _ = std::fs::remove_file(path);
}

#[test]
fn agent_projects_tilted_photo_into_editable_cad_without_warping_image() {
    let mut server = Server::new(Box::new(Headless::default()));
    let img = tool(&mut server, "execute", json!({"command":"imageattach","params":{"path":"photo.png","pixels":[200,100],"width":200,"height":100}}));
    let mut args = json!({"handle":img["handle"],"pixels":[[0,0],[200,0],[200,100],[0,100]],
        "world":[[10,20],[40,15],[35,45],[5,40]],"points":[[0,0],[200,0],[200,100],[0,100]],
        "closed":true,"preview":true});
    let preview = tool(&mut server, "trace_perspective_image", args.clone());
    assert_eq!(preview["preview"], true);
    assert_eq!(preview["handle"], Value::Null);
    assert_eq!(preview["rasterRectified"], false);
    assert!(preview["maxControlError"].as_f64().unwrap() < 1.0e-7);
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    args["preview"] = json!(false);
    let applied = tool(&mut server, "trace_perspective_image", args.clone());
    assert!(applied["handle"].is_string());
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Polyline"}))["count"], 1);
    args["pixels"] = json!([[0,0],[0,0],[200,100],[0,100]]);
    let bad = call(&mut server, 20, "tools/call", json!({"name":"trace_perspective_image","arguments":args}));
    assert_eq!(bad["result"]["isError"], true);
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Polyline"}))["count"], 1);
}

#[test]
fn agent_fits_four_survey_controls_before_drawing_on_image() {
    let mut s = Server::new(Box::new(Headless::default()));
    let image = tool(&mut s, "execute", json!({"command":"imageattach","params":{"path":"survey.png","pixels":[200,100],"width":200,"height":100}}));
    let mut args = json!({"handle":image["handle"],"pixels":[[0,100],[200,100],[0,0],[200,0]],
        "world":[[10,20],[30,20],[10,30],[30,30]],"maxResidual":0.001,"preview":true});
    let preview = tool(&mut s, "fit_image_controls", args.clone());
    assert_eq!(preview["preview"], true);
    assert_eq!(preview["controlCount"], 4);
    assert!(preview["maxResidual"].as_f64().unwrap() < 1.0e-9);
    let before = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100]],"preview":true}));
    assert_eq!(before["worldPoints"], json!([[0.0,0.0],[200.0,0.0]]));
    args["preview"] = json!(false);
    tool(&mut s, "fit_image_controls", args.clone());
    let traced = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100]],"preview":false}));
    assert!((traced["worldPoints"][0][0].as_f64().unwrap() - 10.0).abs() < 1.0e-9);
    assert!((traced["worldPoints"][1][0].as_f64().unwrap() - 30.0).abs() < 1.0e-9);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 1);
    args["world"][3] = json!([95.0,85.0]);
    let refused = call(&mut s, 5, "tools/call", json!({"name":"fit_image_controls","arguments":args}));
    assert_eq!(refused["result"]["isError"], true, "agent must not replace a calibrated image with bad survey points");
}

#[test]
fn agent_calibrates_image_before_tracing_using_two_control_points() {
    let mut s = Server::new(Box::new(Headless::default()));
    let image = tool(&mut s, "execute", json!({"command":"imageattach","params":{"path":"survey.png","at":[0,0],"pixels":[200,100],"width":200,"height":100}}));
    let points = json!({"handle":image["handle"],"pixels":[[0,100],[200,100]],"world":[[10,20],[30,20]],"preview":true});
    let preview = tool(&mut s, "calibrate_image", points.clone());
    assert_eq!(preview["preview"], true);
    let before = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100]],"preview":true}));
    assert_eq!(before["worldPoints"], json!([[0.0,0.0],[200.0,0.0]]));
    let mut approved = points.clone();
    approved["preview"] = json!(false);
    let calibration = tool(&mut s, "calibrate_image", approved);
    assert_eq!(calibration["preview"], false);
    let trace = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100]],"preview":false}));
    assert_eq!(trace["worldPoints"], json!([[10.0,20.0],[30.0,20.0]]));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 1);
    let wrong = call(&mut s, 2, "tools/call", json!({"name":"calibrate_image","arguments":{"handle":image["handle"],"pixels":[[0,0]],"world":[[0,0],[10,0]]}}));
    assert_eq!(wrong["error"]["code"], -32602);
}

#[test]
fn agent_traces_an_image_into_editable_cad_via_mcp() {
    let mut s = Server::new(Box::new(Headless::default()));
    let tools = call(&mut s, 1, "tools/list", json!({}));
    assert!(tools["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "trace_image"));
    let image = tool(&mut s, "execute", json!({"command":"imageattach","params":{"path":"survey.png","at":[10,20],"pixels":[200,100],"width":20,"height":10}}));
    let preview = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100],[200,0]],"closed":true,"preview":true}));
    assert_eq!(preview["preview"], true);
    assert_eq!(preview["handle"], Value::Null);
    assert_eq!(preview["worldPoints"], json!([[10.0,20.0],[30.0,20.0],[30.0,30.0]]));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0, "AI preview must not alter the DWG");
    let traced = tool(&mut s, "trace_image", json!({"handle":image["handle"],"points":[[0,100],[200,100],[200,0]],"closed":true}));
    assert_eq!(traced["vertices"], 3);
    assert_eq!(traced["closed"], true);
    let entities = tool(&mut s, "query_entities", json!({"type":"Polyline"}));
    assert_eq!(entities["count"], 1, "MCP tracing must create actual editable CAD geometry: {entities}");
    let wrong = call(&mut s, 2, "tools/call", json!({"name":"trace_image","arguments":{"handle":image["handle"],"points":[[0,0]]}}));
    assert_eq!(wrong["error"]["code"], -32602, "MCP schema must reject a single point");
    let bad = call(&mut s, 3, "tools/call", json!({"name":"trace_image","arguments":{"handle":image["handle"],"points":[[0,0],[201,20]]}}));
    assert_eq!(bad["result"]["isError"], true, "Out-of-bounds points must not create geometry");
    let again = tool(&mut s, "query_entities", json!({"type":"Polyline"}));
    assert_eq!(again["count"], 1);
}

/// The MCP handshake works: `initialize` names the server, the `initialized` notification is
/// accepted, `tools/list` offers the command line, and an unknown method is -32601.
#[test]
fn lifecycle_and_tools() {
    let mut s = Server::new(Box::new(Headless::default()));
    let init = call(&mut s, 1, "initialize", json!({"protocolVersion": "2025-06-18"}));
    assert_eq!(init["result"]["serverInfo"]["name"], "cadcraft");
    assert!(s.handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    assert!(s.is_initialized());
    let tools = call(&mut s, 2, "tools/list", json!({}));
    assert!(tools["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "command_line"));
    let bad = call(&mut s, 3, "nope", json!({}));
    assert_eq!(bad["error"]["code"], -32601);
}

/// Guards that an agent can complete a drawing (plate, holes, layer, text) using only the MCP
/// tools, and verify it only through MCP: counts, selection queries, undo and a PNG render.
#[test]
fn agent_draws_a_plate_with_holes() {
    let mut s = Server::new(Box::new(Headless::default()));
    // Task 1: an outline via the command line, AutoCAD style.
    tool(&mut s, "command_line", json!({"text": "rectang 0,0 100,60"}));
    // Task 2: four holes with JSON.
    for (x, y) in [(10, 10), (90, 10), (10, 50), (90, 50)] {
        tool(&mut s, "execute", json!({"command": "circle", "params": {"center": [x, y], "radius": 4}}));
    }
    // Task 3: a layer and a centre line on it.
    tool(&mut s, "execute", json!({"command": "layer.new", "params": {"name": "Center", "color": "red", "current": true}}));
    tool(&mut s, "command_line", json!({"text": "line -5,30 105,30"}));
    tool(&mut s, "command_line", json!({"text": ""}));
    // Task 4: a script with text. TEXT keeps asking for further lines until an empty one, as in
    // AutoCAD, so the script ends it with a blank line (= Enter).
    let r = tool(&mut s, "script", json!({"text": "TEXT 0,-10 5 0 PLATE 100x60\n\n"}));
    assert_eq!(r["state"]["running"], Value::Null, "TEXT must be finished: {r}");
    // Verify only through MCP.
    let d = tool(&mut s, "inspect_drawing", json!({}));
    assert_eq!(d["counts"]["Circle"], 4);
    assert_eq!(d["counts"]["Polyline"], 1);
    assert_eq!(d["counts"]["Line"], 1);
    assert_eq!(d["counts"]["Text"], 1);
    let lines = tool(&mut s, "query_entities", json!({"type": "Line", "layer": "Center"}));
    assert_eq!(lines["count"], 1);
    // Task 5: undo the text, check, then render.
    let u = tool(&mut s, "execute", json!({"command": "undo"}));
    assert_eq!(u["undone"], json!(["Single Line Text"]), "{u}");
    let d = tool(&mut s, "inspect_drawing", json!({"entities": false}));
    assert!(d["counts"].get("Text").is_none());
    assert_eq!(d["counts"]["Circle"], 4);
    assert_eq!(d["counts"]["Polyline"], 1);
    assert_eq!(d["counts"]["Line"], 1);
    let lines = tool(&mut s, "query_entities", json!({"type": "Line", "layer": "Center"}));
    assert_eq!(lines["count"], 1, "the centre line must survive undoing the text");
    let img = tool(&mut s, "render", json!({"width": 320, "height": 200}));
    assert_eq!(img["type"], "image");
    assert!(img["data"].as_str().unwrap().len() > 100);
}

/// Guards that bad input is reported, not fatal: a refused command and a screenshot without a
/// UI are `isError` results, and malformed JSON is a parse error (-32700).
#[test]
fn errors_are_tool_errors_not_crashes() {
    let mut s = Server::new(Box::new(Headless::default()));
    let r = call(&mut s, 1, "tools/call", json!({"name": "execute", "arguments": {"command": "circle", "params": {"center": "x"}}}));
    assert_eq!(r["result"]["isError"], true);
    let r = call(&mut s, 2, "tools/call", json!({"name": "screenshot", "arguments": {}}));
    assert_eq!(r["result"]["isError"], true);
    assert!(s.handle_line("{not json").unwrap().contains("-32700"));
}

/// Unknown tools and arguments that don't fit a tool's schema are invalid params (-32602), never
/// guessed at; an empty batch is one Invalid Request.
#[test]
fn bad_tool_calls_are_invalid_params() {
    let mut s = Server::new(Box::new(Headless::default()));
    tool(&mut s, "command_line", json!({"text": "circle 0,0 1"}));
    let bad = [
        json!({"name": "nope", "arguments": {}}),
        json!({"name": "command_line", "arguments": {}}),
        json!({"name": "command_line", "arguments": {"text": 5}}),
        json!({"name": "command_line", "arguments": "circle 1,1 1"}),
        json!({"name": "script", "arguments": {}}),
        json!({"name": "query_entities", "arguments": {"window": [-5, -5, 5, 5]}}),
        json!({"name": "query_entities", "arguments": {"window": [[-5, -5]]}}),
        json!({"name": "query_entities", "arguments": {"window": {"min": [-5, -5], "max": [5, 5]}}}),
        json!({"name": "query_entities", "arguments": {"type": 7}}),
        json!({"name": "query_entities", "arguments": {"layer": ["0"]}}),
        json!({"name": "query_entities", "arguments": {"offset": -1}}),
        json!({"name": "query_entities", "arguments": {"selected": "true"}}),
        json!({"name": "query_entities", "arguments": {"selected": 1}}),
    ];
    for (i, params) in bad.into_iter().enumerate() {
        let r = call(&mut s, i as u64, "tools/call", params.clone());
        assert_eq!(r["error"]["code"], -32602, "{params} -> {r}");
    }
    // Nothing was typed: the circle is still the only object and no command is running.
    let d = tool(&mut s, "inspect_drawing", json!({"entities": false}));
    assert_eq!(d["entityCount"], 1, "{d}");
    assert_eq!(s.handle_line("[]").map(|r| serde_json::from_str::<Value>(&r).unwrap()["error"]["code"].clone()), Some(json!(-32600)));
}

/// `open` reads both DXF and DWG, so `tools/list` must advertise both (#1875).
#[test]
fn open_tool_description_names_dxf_and_dwg() {
    let mut s = Server::new(Box::new(Headless::default()));
    let tools = call(&mut s, 1, "tools/list", json!({}));
    let open = tools["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "open").unwrap();
    let desc = open["description"].as_str().unwrap();
    assert!(desc.contains("DXF"), "`open` must still name DXF: {desc}");
    assert!(desc.contains("DWG"), "`open` reads DWG too: {desc}");
}

/// Guards that `query_entities` advertises its `selected` filter and honours it (active-space
/// selection only, both ways) without changing the drawing.
#[test]
fn query_entities_selection_filter_is_discoverable_and_forwarded() {
    let mut s = Server::new(Box::new(Headless::default()));
    let tools = call(&mut s, 1, "tools/list", json!({}));
    let query = tools["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == "query_entities").unwrap();
    assert_eq!(query["inputSchema"]["properties"]["selected"]["type"], "boolean");
    tool(&mut s, "execute", json!({"command": "circle", "params": {"center": [0, 0], "radius": 2}}));
    tool(&mut s, "execute", json!({"command": "circle", "params": {"center": [10, 0], "radius": 2}}));
    assert_eq!(tool(&mut s, "query_entities", json!({"selected": true}))["count"], 0);
    tool(&mut s, "execute", json!({"command": "select", "params": {"window": [[-3, -3], [3, 3]]}}));
    let before = tool(&mut s, "inspect_drawing", json!({}));
    let selected = tool(&mut s, "query_entities", json!({"selected": true, "type": "Circle"}));
    assert_eq!(selected["count"], 1);
    assert_eq!(selected["entities"][0]["geometry"]["center"]["x"], 0.0);
    assert_eq!(tool(&mut s, "query_entities", json!({"selected": false}))["count"], 2);
    assert_eq!(tool(&mut s, "inspect_drawing", json!({})), before);
}

/// `count` is every match while `returned` is the page; a window matches geometry, not bounding boxes.
#[test]
fn query_entities_pages_and_windows_by_geometry() {
    let mut s = Server::new(Box::new(Headless::default()));
    for c in ["circle 0,0 1", "circle 100,100 1", "circle 200,0 1"] {
        tool(&mut s, "command_line", json!({"text": c}));
    }
    // An arc whose bounding box covers the window but whose curve stays outside it.
    tool(&mut s, "execute", json!({"command": "arc", "params": {"p1": [40, 0], "p2": [50, 10], "p3": [60, 0]}}));
    let r = tool(&mut s, "query_entities", json!({"type": "Circle", "limit": 1, "offset": 1}));
    assert_eq!((r["count"].clone(), r["returned"].clone()), (json!(3), json!(1)), "{r}");
    assert_eq!(r["entities"][0]["geometry"]["center"]["x"], 100.0, "{r}");
    let r = tool(&mut s, "query_entities", json!({"window": [[45, 1], [55, 5]]}));
    assert_eq!(r["count"], 0, "the arc passes above this window: {r}");
    let r = tool(&mut s, "query_entities", json!({"window": [[-5, -5], [55, 5]]}));
    assert_eq!(r["count"], 2, "the circle at 0,0 and the arc crossing the window: {r}");
    let r = tool(&mut s, "query_entities", json!({"window": [[-5, -5], [55, 5]], "crossing": false}));
    assert_eq!(r["count"], 1, "only the circle is entirely inside: {r}");
}

/// Guards that the `list_commands` filter matches ids, labels and aliases but not parameter
/// docs, so searching `circle` does not return QSELECT merely for mentioning circles.
#[test]
fn list_commands_filter_matches_names_not_docs() {
    let mut s = Server::new(Box::new(Headless::default()));
    let r = tool(&mut s, "list_commands", json!({"filter": "circle"}));
    let ids: Vec<&str> = r.as_array().unwrap().iter().filter_map(|c| c["id"].as_str()).collect();
    assert!(ids.contains(&"circle"), "{ids:?}");
    assert!(!ids.contains(&"qselect"), "QSELECT only mentions circles in its docs: {ids:?}");
    let r = tool(&mut s, "list_commands", json!({"filter": "pl"}));
    assert!(r.as_array().unwrap().iter().any(|c| c["id"] == "pline"), "aliases and ids match");
}

/// Guards that `find_text` returns TEXT and MTEXT with their placement, strips MTEXT formatting,
/// and filters by text, layer and window; a windowed render keeps that area's aspect ratio.
#[test]
fn find_text_reads_texts_and_render_shows_one_area() {
    let mut s = Server::new(Box::new(Headless::default()));
    call(&mut s, 1, "initialize", json!({"protocolVersion": "2025-06-18"}));
    tool(&mut s, "execute", json!({"command": "layer.new", "params": {"name": "Texto"}}));
    tool(&mut s, "execute", json!({"command": "text", "params": {"at": [1, 1], "height": 0.15, "text": "RECAMARA 1"}}));
    tool(&mut s, "execute", json!({"command": "text", "params": {"at": [50, 1], "height": 0.15, "text": "COCINA"}}));
    tool(
        &mut s,
        "execute",
        json!({"command": "mtext", "params": {"at": [1, 5], "height": 0.2, "width": 5, "text": "{\\fArial|b1;PLANTA} BAJA\\PNIVEL 0.00"}}),
    );
    tool(&mut s, "execute", json!({"command": "rectang", "params": {"p1": [0, 0], "p2": [10, 5]}}));

    let all = tool(&mut s, "find_text", json!({}));
    assert_eq!(all["count"], 3);
    let r = tool(&mut s, "find_text", json!({"contains": "recamara"}));
    assert_eq!(r["count"], 1);
    assert_eq!(r["texts"][0]["text"], "RECAMARA 1");
    assert_eq!(r["texts"][0]["at"], json!([1.0, 1.0]));
    assert_eq!(r["texts"][0]["height"], 0.15);
    // Multiline text comes back readable, without its formatting codes.
    let m = tool(&mut s, "find_text", json!({"contains": "planta"}));
    let t = m["texts"][0]["text"].as_str().unwrap();
    assert!(t.contains("PLANTA BAJA") && t.contains("NIVEL 0.00") && !t.contains('\\') && !t.contains('{'), "{t}");
    // By area: only what is inside the window.
    let w = tool(&mut s, "find_text", json!({"window": [[-1, -1], [20, 20]]}));
    assert_eq!(w["count"], 2);
    assert_eq!(tool(&mut s, "find_text", json!({"layer": "nope"}))["count"], 0);

    // A render of one area takes the area's shape.
    let r = call(&mut s, 9, "tools/call", json!({"name": "render", "arguments": {"window": [[0, 0], [10, 5]]}}));
    let c = &r["result"]["content"][0];
    assert_eq!(c["type"], "image", "{r}");
    let png = cadcraft_engine::cmd::file::base64_decode(c["data"].as_str().unwrap()).unwrap();
    let (w, h) = (u32::from_be_bytes(png[16..20].try_into().unwrap()), u32::from_be_bytes(png[20..24].try_into().unwrap()));
    assert_eq!((w, h), (1600, 800));
}

/// Guards a lazy Remote bridge: it serves tools without the app, reports how to start it
/// (`--control`), and reaches the app once that app starts listening on the port.
#[test]
fn a_lazy_bridge_starts_without_the_app_and_says_how_to_open_it() {
    // Nothing listens on a port that was just freed.
    let addr = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().to_string()
    };
    let mut s = Server::new(Box::new(crate::Remote::lazy(&addr, false)));
    let init = call(&mut s, 1, "initialize", json!({"protocolVersion": "2025-06-18"}));
    assert_eq!(init["result"]["serverInfo"]["name"], "cadcraft");
    let tools = call(&mut s, 2, "tools/list", json!({}));
    assert!(tools["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "screenshot"));
    let r = call(&mut s, 3, "tools/call", json!({"name": "command_line", "arguments": {"text": "line 0,0 1,1"}}));
    assert_eq!(r["result"]["isError"], json!(true));
    let msg = r["result"]["content"][0]["text"].as_str().unwrap();
    assert!(msg.contains("not answering") && msg.contains("--control"), "{msg}");

    // Once the app listens, the next call reaches it.
    let listener = std::net::TcpListener::bind(&addr).unwrap();
    let app = std::thread::spawn(move || {
        use std::io::{BufRead, Write};
        let (sock, _) = listener.accept().unwrap();
        let mut line = String::new();
        std::io::BufReader::new(sock.try_clone().unwrap()).read_line(&mut line).unwrap();
        let id = serde_json::from_str::<Value>(&line).unwrap()["id"].clone();
        writeln!(&sock, "{}", json!({"id": id, "ok": true, "result": {"prompt": "Command:"}})).unwrap();
    });
    let r = call(&mut s, 4, "tools/call", json!({"name": "command_line", "arguments": {"text": ""}}));
    assert_ne!(r["result"]["isError"], json!(true), "{r}");
    app.join().unwrap();
}


#[test]
fn ai_trace_quality_gate_rejects_crossing_outline_without_modifying_drawing() {
    let mut s = Server::new(Box::new(Headless::default()));
    let defs = call(&mut s, 1, "tools/list", json!({}));
    assert!(defs["result"]["tools"].as_array().unwrap().iter().any(|v| v["name"] == "trace_quality_check"));
    let bad = tool(&mut s, "trace_quality_check", json!({"points":[[0,0],[10,10],[0,10],[10,0]],"closed":true}));
    assert_eq!(bad["safeToCommit"], false);
    assert_eq!(bad["selfIntersections"], json!([[0,2]]));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    let good = tool(&mut s, "trace_quality_check", json!({"points":[[0,0],[10,0],[10,10],[0,10]],"closed":true}));
    assert_eq!(good["safeToCommit"], true);
    assert_eq!(good["segmentCount"], 4);
    assert_eq!(good["bounds"], json!([[0.0,0.0],[10.0,10.0]]));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
}

#[test]
fn ai_trace_quality_gate_rejects_short_edges_and_invalid_closed_outline() {
    let mut s = Server::new(Box::new(Headless::default()));
    let short = tool(&mut s, "trace_quality_check", json!({"points":[[0,0],[0.001,0],[10,0]],"minSegmentLength":0.01}));
    assert_eq!(short["safeToCommit"], false);
    assert_eq!(short["shortSegments"], json!([0]));
    let two_point_loop = tool(&mut s, "trace_quality_check", json!({"points":[[0,0],[10,0]],"closed":true}));
    assert_eq!(two_point_loop["safeToCommit"], false);
    assert_eq!(two_point_loop["tooFewPointsForClosedOutline"], true);
    let malformed = call(&mut s, 66, "tools/call", json!({"name":"trace_quality_check","arguments":{"points":[[0,0]]}}));
    assert_eq!(malformed["error"]["code"], -32602);
}


#[test]
fn ai_validated_image_trace_rejects_bad_geometry_before_cad_edit() {
    let mut s = Server::new(Box::new(Headless::default()));
    let image = tool(&mut s, "execute", json!({"command":"imageattach","params":{"path":"ai-photo.png","pixels":[200,100],"width":200,"height":100}}));
    let rejected = call(&mut s, 81, "tools/call", json!({"name":"trace_image_validated","arguments":{
        "handle":image["handle"],"points":[[0,0],[100,100],[0,100],[100,0]],"closed":true
    }}));
    assert_eq!(rejected["result"]["isError"], true, "bad AI polygon must be refused: {rejected}");
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    let preview = tool(&mut s, "trace_image_validated", json!({"handle":image["handle"],
        "points":[[0,100],[200,100],[200,0],[0,0]],"closed":true,"preview":true,"minSegmentLength":1}));
    assert_eq!(preview["preview"], true);
    assert_eq!(preview["worldPoints"].as_array().map(Vec::len), Some(4));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    let committed = tool(&mut s, "trace_image_validated", json!({"handle":image["handle"],
        "points":[[0,100],[200,100],[200,0],[0,0]],"closed":true,"minSegmentLength":1}));
    assert!(committed["handle"].is_string(), "valid AI trace should create editable polyline: {committed}");
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 1);
}

#[test]
fn ai_validated_perspective_trace_blocks_bad_trace_without_mutation() {
    let mut s = Server::new(Box::new(Headless::default()));
    let image = tool(&mut s, "execute", json!({"command":"imageattach","params":{"path":"tilted-ai-photo.png","pixels":[200,100],"width":200,"height":100}}));
    let mut args = json!({"handle":image["handle"],
        "pixels":[[0,0],[200,0],[200,100],[0,100]],
        "world":[[10,20],[40,15],[35,45],[5,40]],
        "points":[[0,0],[200,100],[0,100],[200,0]],
        "closed":true,"preview":false,"minSegmentLength":0.25});
    let rejected = call(&mut s, 82, "tools/call", json!({"name":"trace_perspective_image_validated","arguments":args.clone()}));
    assert_eq!(rejected["result"]["isError"], true);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    args["points"] = json!([[0,0],[200,0],[200,100],[0,100]]);
    args["preview"] = json!(true);
    let preview = tool(&mut s, "trace_perspective_image_validated", args.clone());
    assert_eq!(preview["preview"], true);
    assert_eq!(preview["worldPoints"].as_array().map(Vec::len), Some(4));
    assert!(preview["maxControlError"].as_f64().is_some_and(|e| e < 1.0e-6));
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
    args["preview"] = json!(false);
    let committed = tool(&mut s, "trace_perspective_image_validated", args);
    assert!(committed["handle"].is_string());
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 1);
}


#[test]
fn ai_world_projection_gate_rejects_crossing_preview_without_engine_commit() {
    struct MockProjection { calls: usize }
    impl crate::Backend for MockProjection {
        fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
            assert_eq!(method, "engine.execute");
            assert_eq!(params["command"], "imageprojecttrace");
            assert_eq!(params["params"]["preview"], true, "validation must never commit a failing preview");
            self.calls += 1;
            Ok(json!({"preview":true,"worldPoints":[[0,0],[10,10],[0,10],[10,0]]}))
        }
        fn has_ui(&self) -> bool { false }
        fn describe(&self) -> String { "mock projection".into() }
    }
    let mut backend = MockProjection { calls: 0 };
    let input = json!({"handle":"42","pixels":[[0,0],[100,0],[100,100],[0,100]],
        "world":[[0,0],[10,0],[10,10],[0,10]],
        "points":[[0,0],[100,0],[100,100],[0,100]],"closed":true});
    let result = crate::tools::call_tool(&mut backend, "trace_perspective_image_validated", &input).unwrap();
    assert_eq!(result["isError"], true);
    assert!(result["content"][0]["text"].as_str().unwrap().contains("world-coordinate projection"));
    assert_eq!(backend.calls, 1, "no second engine commit should be sent after world-geometry rejection");
}


#[test]
fn ai_trace_quality_large_survey_coordinates_do_not_create_false_overlaps() {
    let mut s = Server::new(Box::new(Headless::default()));
    // At 1e11, the old absolute-coordinate tolerance was 10 drawing units:
    // nonadjacent segments separated by one unit were incorrectly reported
    // as overlapping even though both segments are geometrically disjoint.
    let sound = tool(&mut s, "trace_quality_check", json!({
        "points":[[100000000000.0,0.0],[100000000001.0,0.0],[100000000002.0,0.0],[100000000003.0,0.0]],
        "closed":false,"minSegmentLength":0.1
    }));
    assert_eq!(sound["safeToCommit"], true, "disjoint collinear segments must not be rejected: {sound}");
    assert_eq!(sound["selfIntersections"], json!([]));
    let overlap = tool(&mut s, "trace_quality_check", json!({
        "points":[[100000000000.0,0.0],[100000000004.0,0.0],[100000000005.0,0.0],[100000000002.0,0.0]],
        "closed":false,"minSegmentLength":0.1
    }));
    assert_eq!(overlap["safeToCommit"], false, "truly overlapping segments still fail: {overlap}");
    assert_eq!(overlap["selfIntersections"], json!([[0,2]]));
}


#[test]
fn ai_trace_quality_large_2048_point_image_trace_filters_disjoint_segments() {
    let mut s = Server::new(Box::new(Headless::default()));
    let points: Vec<Value> = (0..2048).map(|i| json!([i as f64, (i % 2) as f64])).collect();
    let result = tool(&mut s, "trace_quality_check", json!({"points":points,"closed":false,"minSegmentLength":0.001}));
    assert_eq!(result["safeToCommit"], true, "large AI stroke must remain valid: {result}");
    assert_eq!(result["pointCount"], 2048);
    assert_eq!(result["segmentCount"], 2047);
    assert_eq!(result["selfIntersections"], json!([]));
    assert_eq!(result["intersectionCandidatePairsTested"], 0, "sweep must skip disjoint segment bounding boxes");
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Polyline"}))["count"], 0);
}

#[test]
fn ai_trace_quality_broad_phase_preserves_nonadjacent_crossings() {
    let mut s = Server::new(Box::new(Headless::default()));
    let result = tool(&mut s, "trace_quality_check", json!({
        "points":[[0,0],[10,10],[0,10],[10,0]],"closed":true
    }));
    assert_eq!(result["safeToCommit"], false);
    assert_eq!(result["selfIntersections"], json!([[0,2]]));
    assert!(result["intersectionCandidatePairsTested"].as_u64().is_some_and(|n| n >= 1));
}


#[test]
fn ai_mcp_rectangle_png_creates_closed_polyline_and_single_undo() {
    use image::{RgbImage, Rgb};
    let mut image = RgbImage::from_pixel(96, 72, Rgb([255,255,255]));
    for x in 10..=70 {
        image.put_pixel(x,10,Rgb([0,0,0]));
        image.put_pixel(x,50,Rgb([0,0,0]));
    }
    for y in 10..=50 {
        image.put_pixel(10,y,Rgb([0,0,0]));
        image.put_pixel(70,y,Rgb([0,0,0]));
    }
    let path=std::env::temp_dir().join(format!("cad360-mcp-outline-{}-{}.png",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    image.save(&path).unwrap();
    let filename=path.to_string_lossy().to_string();
    let mut server=Server::new(Box::new(Headless::default()));
    let args=json!({"path":filename,"preview":true,"closedRectangles":true,"includeDiagonals":false,"minLengthPixels":16});
    let preview=tool(&mut server,"raster_to_cad_lines",args.clone());
    assert_eq!(preview["rectanglesDetected"],1);
    assert_eq!(preview["lines"].as_array().unwrap().len(),0);
    let mut commit_args=args;
    commit_args["preview"]=json!(false);
    let result=tool(&mut server,"raster_to_cad_lines",commit_args);
    let _=std::fs::remove_file(path);
    assert_eq!(result["closedPolylinesCreated"],1);
    assert_eq!(result["editableEntitiesCreated"],1);
    let entities=tool(&mut server,"query_entities",json!({}));
    assert_eq!(entities["count"],1,"one rectangle should yield one editable CAD entity");
    let _=tool(&mut server,"execute",json!({"command":"undo","params":{"count":1}}));
    let after=tool(&mut server,"query_entities",json!({}));
    assert_eq!(after["count"],0,"AI-created closed PLINE must be removed by one Undo");
}

#[test]
fn ai_automatic_raster_to_cad_lines_from_real_png_without_manual_points() {
    use image::{GrayImage, Luma};
    let mut pixels = GrayImage::from_pixel(64, 48, Luma([255]));
    for y in 10..=12 { for x in 6..=55 { pixels.put_pixel(x, y, Luma([0])); } }
    for x in 25..=27 { for y in 18..=42 { pixels.put_pixel(x, y, Luma([0])); } }
    let name = format!("cad360-ai-auto-vector-{}-{}.png", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let path = std::env::temp_dir().join(name);
    pixels.save(&path).unwrap();
    let filename = path.to_string_lossy().to_string();
    let mut s = Server::new(Box::new(Headless::default()));
    let defs = call(&mut s, 1, "tools/list", json!({}));
    assert!(defs["result"]["tools"].as_array().unwrap().iter().any(|v| v["name"] == "raster_to_cad_lines"));
    let preview = tool(&mut s, "raster_to_cad_lines", json!({"path":filename,"minLengthPixels":12,"preview":true,"unitsPerPixel":0.5}));
    assert_eq!(preview["strokes"], 2, "must find horizontal and vertical strokes in actual PNG: {preview}");
    assert_eq!(preview["editableEntitiesCreated"], 0);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Line"}))["count"], 0);
    let committed = tool(&mut s, "raster_to_cad_lines", json!({"path":filename,"minLengthPixels":12,"preview":false,"unitsPerPixel":0.5}));
    assert_eq!(committed["editableEntitiesCreated"], 2);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Line"}))["count"], 2);
    let _ = std::fs::remove_file(path);
}

#[test]
fn ai_auto_raster_to_cad_refuses_missing_file_without_mutation() {
    let mut s = Server::new(Box::new(Headless::default()));
    let filename = format!("cad360-missing-raster-{}-nonexistent.png", std::process::id());
    let rejected = call(&mut s, 99, "tools/call", json!({"name":"raster_to_cad_lines","arguments":{"path":filename,"preview":false}}));
    assert_eq!(rejected["result"]["isError"], true);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Line"}))["count"], 0);
}


#[test]
fn ai_auto_raster_diagonal_png_creates_editable_line_without_pixel_coordinates() {
    use image::{GrayImage, Luma};
    let mut pixels = GrayImage::from_pixel(64, 64, Luma([255]));
    for x in 8..=48 { pixels.put_pixel(x, x, Luma([0])); }
    let name = format!("cad360-ai-diagonal-{}-{}.png", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let path = std::env::temp_dir().join(name);
    pixels.save(&path).unwrap();
    let filename = path.to_string_lossy().to_string();
    let mut s = Server::new(Box::new(Headless::default()));
    let disabled = tool(&mut s, "raster_to_cad_lines", json!({"path":filename,"includeDiagonals":false,"minLengthPixels":12}));
    assert_eq!(disabled["strokes"], 0, "45-degree stroke should not be falsely classified as horizontal/vertical");
    let preview = tool(&mut s, "raster_to_cad_lines", json!({"path":filename,"includeDiagonals":true,"minLengthPixels":12}));
    assert_eq!(preview["strokes"], 1, "diagonal should be recognized automatically: {preview}");
    assert_eq!(preview["editableEntitiesCreated"], 0);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Line"}))["count"], 0);
    let applied = tool(&mut s, "raster_to_cad_lines", json!({"path":filename,"includeDiagonals":true,"preview":false,"minLengthPixels":12}));
    assert_eq!(applied["editableEntitiesCreated"], 1);
    assert_eq!(tool(&mut s, "query_entities", json!({"type":"Line"}))["count"], 1);
    let _ = std::fs::remove_file(path);
}


#[test]
fn ai_arbitrary_angle_raster_to_cad_png_creates_editable_line() {
    use image::{GrayImage, Luma};
    let mut pixels=GrayImage::from_pixel(120,120,Luma([255]));
    for x in 12..=100 {
        let y=20+(f64::from(x-12)*0.577350269).round() as u32;
        pixels.put_pixel(x,y,Luma([0]));
    }
    let file=std::env::temp_dir().join(format!("cad360-pca-vector-{}-{}.png",
        std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    pixels.save(&file).unwrap();
    let path=file.to_string_lossy().to_string();
    let mut server=Server::new(Box::new(Headless::default()));
    let preview=tool(&mut server,"raster_to_cad_lines",json!({"path":path,"preview":true,"minLengthPixels":20,
        "includeDiagonals":false,"includeArbitraryAngles":true}));
    assert_eq!(preview["strokes"],1,"isolated 30-degree input line should vectorize: {preview}");
    assert_eq!(preview["editableEntitiesCreated"],0);
    assert_eq!(tool(&mut server,"query_entities",json!({"type":"Line"}))["count"],0);
    let applied=tool(&mut server,"raster_to_cad_lines",json!({"path":path,"preview":false,"minLengthPixels":20,
        "includeDiagonals":false,"includeArbitraryAngles":true}));
    assert_eq!(applied["editableEntitiesCreated"],1);
    assert_eq!(tool(&mut server,"query_entities",json!({"type":"Line"}))["count"],1);
    let _=std::fs::remove_file(file);
}


#[test]
fn ai_auto_raster_to_cad_dxf_roundtrip_preserves_editable_lines() {
    use image::{GrayImage, Luma};
    let mut pixels = GrayImage::from_pixel(96, 64, Luma([255]));
    for x in 8..=80 { pixels.put_pixel(x, 14, Luma([0])); }
    for y in 30..=55 { pixels.put_pixel(30, y, Luma([0])); }
    let suffix = format!("{}-{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let png = std::env::temp_dir().join(format!("cad360-auto-image-{suffix}.png"));
    let dxf = std::env::temp_dir().join(format!("cad360-auto-vector-{suffix}.dxf"));
    pixels.save(&png).unwrap();
    let png_path = png.to_string_lossy().to_string();
    let dxf_path = dxf.to_string_lossy().to_string();
    let mut server = Server::new(Box::new(Headless::default()));
    let preview = tool(&mut server, "raster_to_cad_lines", json!({"path":png_path,"preview":true,"minLengthPixels":12,"includeDiagonals":false}));
    assert_eq!(preview["strokes"], 2);
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Line"}))["count"], 0);
    let result = tool(&mut server, "raster_to_cad_lines", json!({"path":png_path,"preview":false,"minLengthPixels":12,"includeDiagonals":false}));
    assert_eq!(result["editableEntitiesCreated"], 2);
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Line"}))["count"], 2);
    tool(&mut server, "save", json!({"path":dxf_path}));
    assert!(dxf.exists(), "AI output must be saved as real DXF file");
    tool(&mut server, "new_drawing", json!({"metric":true}));
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Line"}))["count"], 0);
    tool(&mut server, "open", json!({"path":dxf_path}));
    assert_eq!(tool(&mut server, "query_entities", json!({"type":"Line"}))["count"], 2,
        "automatic image trace must survive save/reopen as two editable LINE entities");
    let _ = std::fs::remove_file(png);
    let _ = std::fs::remove_file(dxf);
}


#[test]
fn ai_auto_raster_mcp_failed_second_line_rolls_back_first_line() {
    use image::{GrayImage, Luma};
    use crate::Backend;
    struct FailSecondLine { inner: Headless, line_calls: usize }
    impl Backend for FailSecondLine {
        fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
            if method == "engine.execute" && params["command"] == "line" {
                self.line_calls += 1;
                if self.line_calls == 2 { return Err("injected second LINE failure".to_string()); }
            }
            self.inner.call(method, params)
        }
        fn has_ui(&self) -> bool { false }
        fn describe(&self) -> String { "test backend with injected LINE failure".into() }
    }
    let mut img = GrayImage::from_pixel(90,70,Luma([255]));
    for x in 10..=72 { img.put_pixel(x,15,Luma([0])); }
    for x in 10..=72 { img.put_pixel(x,45,Luma([0])); }
    let name=format!("cad360-ai-undo-mcp-{}-{}.png",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let file=std::env::temp_dir().join(name);
    img.save(&file).unwrap();
    let path=file.to_string_lossy().to_string();
    let mut backend=FailSecondLine { inner:Headless::default(),line_calls:0 };
    let before=backend.inner.session.doc().unwrap().entity_count();
    let result=crate::raster_vector::run(&mut backend,&json!({
        "path":path,"preview":false,"minLengthPixels":12,"includeDiagonals":false
    }));
    assert!(result.as_ref().is_err_and(|e| e.contains("rolled back")), "the MCP must rollback the partial operation: {result:?}");
    assert_eq!(backend.line_calls,2);
    assert_eq!(backend.inner.session.doc().unwrap().entity_count(),before,"no image-derived LINE may remain after failed MCP batch");
    assert!(backend.inner.session.state().unwrap().undo_ctl.group.is_none());
    let _=std::fs::remove_file(file);
}

#[test]
fn ai_auto_raster_mcp_rejects_existing_undo_group_without_changes() {
    use image::{GrayImage, Luma};
    let mut img=GrayImage::from_pixel(50,40,Luma([255]));
    for x in 5..=35 { img.put_pixel(x,15,Luma([0])); }
    let name=format!("cad360-ai-undo-protected-{}-{}.png",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let file=std::env::temp_dir().join(name);
    img.save(&file).unwrap();
    let path=file.to_string_lossy().to_string();
    let mut backend=Headless::default();
    backend.session.state_mut().unwrap().begin_undo_group();
    let before=backend.session.doc().unwrap().entity_count();
    let result=crate::raster_vector::run(&mut backend,&json!({
        "path":path,"preview":false,"minLengthPixels":12,"includeDiagonals":false
    }));
    assert!(result.as_ref().is_err_and(|e| e.contains("Undo group")), "the caller's group must not be ended: {result:?}");
    assert_eq!(backend.session.doc().unwrap().entity_count(),before);
    assert!(backend.session.state().unwrap().undo_ctl.group.is_some());
    backend.session.state_mut().unwrap().end_undo_group();
    let _=std::fs::remove_file(file);
}


#[test]
fn ai_image_mcp_inspect_exposes_revision_and_undo_safety_state() {
    let mut server=Server::new(Box::new(Headless::default()));
    let before=tool(&mut server,"inspect_drawing",json!({"entities":false}));
    assert_eq!(before["undoControl"],"all");
    assert_eq!(before["undoGroupOpen"],false);
    let uid=before["documentUid"].as_u64().expect("stable document uid");
    let epoch=before["documentEpoch"].as_u64().expect("document switch epoch");
    let revision=before["revision"].as_u64().expect("document revision");
    tool(&mut server,"execute",json!({"command":"line","params":{"points":[[10,10],[20,10]]}}));
    let after=tool(&mut server,"inspect_drawing",json!({"entities":false}));
    assert_eq!(after["documentUid"],uid);
    assert_eq!(after["documentEpoch"],epoch,"drawing edits do not count as document switches");
    assert_eq!(after["revision"],revision+1);
    assert_eq!(after["undoGroupOpen"],false);
}


#[test]
fn ai_auto_raster_mcp_document_switch_must_not_undo_unrelated_new_drawing() {
    use crate::Backend;
    use image::{GrayImage, Luma};
    struct SwitchAfterFirstLine { inner: Headless, first_line: bool, switched: bool }
    impl Backend for SwitchAfterFirstLine {
        fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
            if method == "drawing.inspect" && self.first_line && !self.switched {
                self.switched = true;
                self.inner.call("engine.execute", json!({"command":"new","params":{"metric":true}}))?;
                self.inner.call("engine.execute", json!({"command":"line","params":{"points":[[100,100],[200,100]]}}))?;
            }
            let r = self.inner.call(method, params.clone());
            if method == "engine.execute" && params["command"] == "line" && !self.switched && r.is_ok() {
                self.first_line = true;
            }
            r
        }
        fn has_ui(&self) -> bool { false }
        fn describe(&self) -> String { "injected document switch during AI import".into() }
    }
    let mut image = GrayImage::from_pixel(90,70,Luma([255]));
    for x in 10..=72 { image.put_pixel(x,15,Luma([0])); }
    for x in 10..=72 { image.put_pixel(x,45,Luma([0])); }
    let filename = format!("cad360-ai-switch-{}-{}.png",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let file = std::env::temp_dir().join(filename);
    image.save(&file).unwrap();
    let mut backend = SwitchAfterFirstLine { inner:Headless::default(),first_line:false,switched:false };
    let original_uid = backend.inner.session.state().unwrap().uid;
    let result = crate::raster_vector::run(&mut backend,&json!({"path":file.to_string_lossy(),
        "preview":false,"minLengthPixels":12,"includeDiagonals":false}));
    assert!(result.as_ref().is_err_and(|e| e.contains("no END or UNDO")), "{result:?}");
    assert!(backend.switched);
    assert_ne!(backend.inner.session.state().unwrap().uid,original_uid);
    assert_eq!(backend.inner.session.doc().unwrap().entity_count(),1,
        "switching to another drawing must not erase the unrelated LINE");
    let _ = std::fs::remove_file(file);
}

#[test]
fn stage244_server_denies_unapproved_mcp_calls_before_backend_dispatch() {
    let mut readonly=Server::with_access_mode(Box::new(Headless::default()),"readonly");
    assert_eq!(readonly.access_mode(),"readonly");
    let denied=call(&mut readonly,1,"tools/call",json!({"name":"execute",
        "arguments":{"command":"line","params":{"points":[[0,0],[5,5]]}}}));
    assert_eq!(denied["error"]["code"],-32602);
    let inspected=tool(&mut readonly,"inspect_drawing",json!({"entities":false}));
    assert_eq!(inspected["totalEntities"],0);
    let mut reviewed=Server::with_access_mode(Box::new(Headless::default()),"reviewed");
    let script=call(&mut reviewed,2,"tools/call",json!({"name":"script","arguments":{"text":"erase all"}}));
    assert_eq!(script["error"]["code"],-32602);
    let preview=call(&mut reviewed,3,"tools/call",json!({"name":"ai_draw_plan_vietnamese",
        "arguments":{"text":"vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0"}}));
    assert!(preview.get("result").is_some(),"reviewed mode must allow preview");
    let rejected=call(&mut reviewed,4,"tools/call",json!({"name":"ai_draw_plan_vietnamese",
        "arguments":{"text":"vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0",
        "preview":false,"confirm":true}}));
    assert_eq!(rejected["error"]["code"],-32602);
}
