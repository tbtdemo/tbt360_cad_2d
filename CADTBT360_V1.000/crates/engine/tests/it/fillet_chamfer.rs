//! FILLET and CHAMFER options at the command line: Trim, Multiple, Undo, and CHAMFER's Angle,
//! mEthod and Polyline.

use cadcraft_engine::Session;
use cadcraft_engine::doc::EntityKind;
use serde_json::json;

type Seg = ((f64, f64), (f64, f64));

fn r(v: f64) -> f64 {
    (v * 1e6).round() / 1e6
}

/// Every line with its ends in a fixed order (rounded), sorted.
fn lines(s: &Session) -> Vec<Seg> {
    let mut v: Vec<Seg> = s
        .doc()
        .unwrap()
        .model
        .iter()
        .filter_map(|e| if let EntityKind::Line(l) = &e.kind { Some(((r(l.a.x), r(l.a.y)), (r(l.b.x), r(l.b.y)))) } else { None })
        .map(|(a, b)| if (a.0, a.1) <= (b.0, b.1) { (a, b) } else { (b, a) })
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

fn arcs(s: &Session) -> usize {
    s.doc().unwrap().model.iter().filter(|e| matches!(e.kind, EntityKind::Arc(_))).count()
}

/// A horizontal line through a vertical one, crossing at (10,0); with `top`, a second horizontal
/// line crossing the vertical one at (10,10).
fn corner(top: bool) -> Session {
    let mut s = Session::new();
    let mut ls = vec!["0,0 12,0", "10,-2 10,12"];
    if top {
        ls.push("0,10 12,10");
    }
    for l in ls {
        s.cmdline(&format!("LINE {l}")).unwrap();
        s.cmdline("").unwrap();
    }
    s
}

#[test]
fn fillet_no_trim_keeps_the_lines() {
    let mut s = corner(false);
    s.script("FILLET T N R 1 5,0 10,5\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), vec![((0.0, 0.0), (12.0, 0.0)), ((10.0, -2.0), (10.0, 12.0))]);
    assert_eq!(arcs(&s), 1);
    // The mode is remembered (TRIMMODE) and Trim switches it back.
    assert_eq!(s.doc().unwrap().header.i64("TRIMMODE", 1), 0);
    s.script("FILLET T T R 0 5,0 10,5\n").unwrap();
    assert_eq!(lines(&s), vec![((0.0, 0.0), (10.0, 0.0)), ((10.0, 0.0), (10.0, 12.0))]);
}

#[test]
fn fillet_multiple_and_undo() {
    let mut s = corner(true);
    // Two corners in one command; Undo takes the second back, Enter ends.
    s.script("FILLET M 5,0 10,5 5,10 10,5 U\n").unwrap();
    assert!(s.running.is_some(), "Multiple keeps asking");
    s.cmdline("").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), vec![((0.0, 0.0), (10.0, 0.0)), ((0.0, 10.0), (12.0, 10.0)), ((10.0, 0.0), (10.0, 12.0))]);
}

#[test]
fn chamfer_angle_and_method() {
    let mut s = corner(false);
    // Length 2 on the first line at 45 degrees from it.
    s.script("CHAMFER A 2 45 5,0 10,5\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), vec![((0.0, 0.0), (8.0, 0.0)), ((8.0, 0.0), (10.0, 2.0)), ((10.0, 2.0), (10.0, 12.0))]);
    assert_eq!(s.doc().unwrap().header.i64("CHAMMODE", 0), 1);
    // mEthod switches back to the two distances.
    let mut s = corner(false);
    s.script("CHAMFER A 2 45 E D D 1 3 5,0 10,5\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), vec![((0.0, 0.0), (9.0, 0.0)), ((9.0, 0.0), (10.0, 3.0)), ((10.0, 3.0), (10.0, 12.0))]);
}

#[test]
fn chamfer_no_trim_and_multiple() {
    let mut s = corner(true);
    s.script("CHAMFER T N D 1 1 M 5,0 10,5 5,10 10,5\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(
        lines(&s),
        vec![
            ((0.0, 0.0), (12.0, 0.0)),
            ((0.0, 10.0), (12.0, 10.0)),
            ((9.0, 0.0), (10.0, 1.0)),
            ((9.0, 10.0), (10.0, 9.0)),
            ((10.0, -2.0), (10.0, 12.0))
        ]
    );
}

#[test]
fn chamfer_polyline() {
    let mut s = Session::new();
    s.script("PLINE 0,0 10,0 10,10 0,10 C\n").unwrap();
    s.script("CHAMFER D 1 2 P 5,0\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    let d = s.doc().unwrap();
    let pl = d.model.iter().find_map(|e| if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }).unwrap();
    let pts: Vec<(f64, f64)> = pl.vertices.iter().map(|v| (r(v.p.x), r(v.p.y))).collect();
    assert_eq!(pts.len(), 8, "{pts:?}");
    // The corner at (10,0): 1 back along the incoming edge, 2 along the outgoing one.
    assert!(pts.contains(&(9.0, 0.0)) && pts.contains(&(10.0, 2.0)), "{pts:?}");
}

#[test]
fn json_chamfer_angle_and_trim() {
    let mut s = corner(false);
    let hs: Vec<String> = s.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
    s.execute("chamfer", &json!({ "h1": hs[0], "p1": [5, 0], "h2": hs[1], "p2": [10, 5], "length": 2, "angle": 45, "trim": false })).unwrap();
    assert_eq!(lines(&s), vec![((0.0, 0.0), (12.0, 0.0)), ((8.0, 0.0), (10.0, 2.0)), ((10.0, -2.0), (10.0, 12.0))]);
}


/// Architectural closed polyline FILLET must stay a native bulged polyline
/// through Undo, Redo and DXF reload, not four rasterized corner drawings.
#[test]
fn architectural_closed_polyline_fillet_bulge_undo_redo_dxf_roundtrip() {
    let mut s = Session::new();
    s.script("PLINE 0,0 6000,0 6000,4000 0,4000 C\n").unwrap();
    let h = s.doc().unwrap().model.iter()
        .find(|e| matches!(&e.kind, EntityKind::LwPolyline(_)))
        .expect("native PLINE")
        .handle.hex();
    let before = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("original closed polyline");
    assert!(before.closed);
    assert_eq!(before.vertices.len(), 4);

    let result = s.execute("fillet", &json!({
        "handle": h, "polyline": true, "radius": 100.0
    })).expect("FILLET all four corners");
    assert_eq!(result["filleted"], 4, "all four rectangle corners must be filleted");
    let rounded = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("rounded native polyline");
    assert!(rounded.closed, "FILLET must preserve closed status");
    assert_eq!(rounded.vertices.len(), 8, "four fillets add four polyline vertices");
    assert_eq!(rounded.vertices.iter().filter(|v| v.bulge.abs() > 1e-9).count(), 4,
        "FILLET arcs must be represented with editable bulges");

    s.undo().expect("Undo polyline FILLET");
    let undone = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("restored polyline");
    assert_eq!(undone.vertices.len(), 4);
    assert!(undone.vertices.iter().all(|v| v.bulge.abs() < 1e-9));
    s.redo().expect("Redo polyline FILLET");
    let restored = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("redone polyline");
    assert_eq!(restored.vertices.len(), 8);

    let dxf = cadcraft_io::write_dxf(s.doc().unwrap());
    let reopened = cadcraft_io::read_dxf(dxf.as_bytes()).expect("reopen filleted DXF");
    let imported = reopened.model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("DXF must retain editable LWPOLYLINE");
    assert!(imported.closed);
    assert_eq!(imported.vertices.len(), 8);
    assert_eq!(imported.vertices.iter().filter(|v| v.bulge.abs() > 1e-9).count(), 4,
        "DXF must preserve native arc bulges without flattening curves");
}


/// A closed architectural perimeter must remain one editable LWPOLYLINE after
/// the native distance CHAMFER, including the exact cut-back coordinates.
#[test]
fn architectural_closed_polyline_chamfer_exact_geometry_and_dxf_reopen() {
    let mut s = Session::new();
    s.script("PLINE 0,0 6000,0 6000,4000 0,4000 C\n").unwrap();
    let handle = s.doc().unwrap().model.iter()
        .find(|e| matches!(&e.kind, EntityKind::LwPolyline(_)))
        .expect("native closed PLINE").handle.hex();
    let result = s.execute("chamfer", &json!({
        "handle": handle, "polyline": true, "d1": 100.0, "d2": 100.0
    })).expect("native distance CHAMFER on all corners");
    assert_eq!(result["chamfered"], 4);
    let edited = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("chamfered LWPOLYLINE");
    assert!(edited.closed);
    let points: Vec<(f64,f64)> = edited.vertices.iter()
        .map(|v| (r(v.p.x), r(v.p.y))).collect();
    assert_eq!(points.len(), 8);
    for point in [(100.0,0.0), (5900.0,0.0),
                  (6000.0,100.0), (6000.0,3900.0),
                  (5900.0,4000.0), (100.0,4000.0),
                  (0.0,3900.0), (0.0,100.0)] {
        assert!(points.contains(&point), "missing chamfer endpoint: {point:?}");
    }
    assert!(edited.vertices.iter().all(|v| v.bulge.abs() < 1e-9),
        "Straight chamfers must not accidentally become arcs");
    s.undo().expect("undo chamfer");
    let undo = s.doc().unwrap().model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("restored perimeter");
    assert_eq!(undo.vertices.len(), 4);
    s.redo().expect("redo chamfer");
    let dxf = cadcraft_io::write_dxf(s.doc().unwrap());
    let reopened = cadcraft_io::read_dxf(dxf.as_bytes()).expect("chamfer DXF reload");
    let restored = reopened.model.iter().find_map(|e| {
        if let EntityKind::LwPolyline(p) = &e.kind { Some(p.clone()) } else { None }
    }).expect("editable polyline in DXF");
    assert!(restored.closed);
    let reopened_points: Vec<(f64,f64)> = restored.vertices.iter()
        .map(|v| (r(v.p.x), r(v.p.y))).collect();
    assert_eq!(reopened_points, points, "DXF must preserve each chamfer endpoint");
}
