//! TRIM and EXTEND options (#397): Quick and Standard modes (TRIMEXTENDMODE), cuTting/Boundary
//! edges, Fence, Crossing, Edge (EDGEMODE), Project (PROJMODE), eRase and Undo, and the JSON
//! forms' `fence`, `crossing`, `edgeMode` and `mode`.

use cadcraft_engine::Session;
use cadcraft_engine::doc::EntityKind;
use serde_json::json;

type Seg = ((f64, f64), (f64, f64));

fn r(v: f64) -> f64 {
    (v * 1e6).round() / 1e6 + 0.0
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

fn session(ls: &[&str]) -> Session {
    let mut s = Session::new();
    for l in ls {
        s.cmdline(&format!("LINE {l}")).unwrap();
        s.cmdline("").unwrap();
    }
    s
}

/// Two vertical edges at x = 0 and x = 10, crossed by horizontal lines at y = 0 and y = 2.
fn ladder() -> Session {
    session(&["0,-5 0,5", "10,-5 10,5", "-5,0 15,0", "-5,2 15,2"])
}

const EDGES: [Seg; 2] = [((0.0, -5.0), (0.0, 5.0)), ((10.0, -5.0), (10.0, 5.0))];

fn with_edges(more: &[Seg]) -> Vec<Seg> {
    let mut v: Vec<Seg> = EDGES.iter().chain(more).copied().collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

fn header(s: &Session, name: &str) -> i64 {
    s.doc().unwrap().header.i64(name, -1)
}

#[test]
fn quick_mode_trims_to_all_objects_and_erases_an_object_with_no_edge() {
    let mut s = ladder();
    s.cmdline("LINE 20,20 25,20").unwrap();
    s.cmdline("").unwrap();
    s.script("TRIM 5,0 22,20\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (15.0, 2.0)), ((10.0, 0.0), (15.0, 0.0))]));
}

#[test]
fn standard_mode_uses_the_selected_cutting_edges() {
    let mut s = ladder();
    // Mode Standard, cutting edge: the line at x = 0 only.
    s.script("TRIM O S 0,4\n\n5,0\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(header(&s, "TRIMEXTENDMODE"), 0);
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (15.0, 2.0))]));
    // Standard mode is remembered: the next TRIM starts by asking for the edges; Enter selects
    // all, and an object no edge crosses is kept.
    s.cmdline("LINE 20,20 25,20").unwrap();
    s.cmdline("").unwrap();
    s.script("TRIM\n\n22,20\n\n").unwrap();
    assert!(lines(&s).contains(&((20.0, 20.0), (25.0, 20.0))));
}

#[test]
fn cutting_edges_option_in_quick_mode() {
    let mut s = ladder();
    s.script("TRIM T 10,4\n\n5,2\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(header(&s, "TRIMEXTENDMODE"), -1, "cuTting edges leaves the mode alone");
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (15.0, 0.0)), ((10.0, 2.0), (15.0, 2.0))]));
}

#[test]
fn fence_trims_every_crossed_object() {
    let mut s = ladder();
    s.script("TRIM F 5,-1 5,3\n\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(
        lines(&s),
        with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (0.0, 2.0)), ((10.0, 0.0), (15.0, 0.0)), ((10.0, 2.0), (15.0, 2.0))])
    );
}

#[test]
fn fence_crossing_one_line_twice_trims_both_parts() {
    let mut s = ladder();
    // A fence round the top crosses y = 2 left of x = 0 and right of x = 10.
    s.script("TRIM F -3,1 -3,6 13,6 13,1\n\n\n").unwrap();
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (15.0, 0.0)), ((0.0, 2.0), (10.0, 2.0))]));
}

#[test]
fn crossing_window_trims_what_it_touches() {
    let mut s = ladder();
    s.script("TRIM C 4,-1 6,1\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (15.0, 2.0)), ((10.0, 0.0), (15.0, 0.0))]));
    // A pick in empty space opens a crossing window too.
    let mut s = ladder();
    s.script("TRIM 4,-4 6,1\n\n").unwrap();
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (15.0, 2.0)), ((10.0, 0.0), (15.0, 0.0))]));
}

#[test]
fn edge_extend_trims_to_an_implied_edge() {
    // A short cutting edge at x = 5 that doesn't reach y = 0.
    let ls = ["5,3 5,4", "-5,0 15,0"];
    let mut s = session(&ls);
    s.script("TRIM O S 5,3.5\n\n8,0\n\n").unwrap();
    assert_eq!(lines(&s), vec![((-5.0, 0.0), (15.0, 0.0)), ((5.0, 3.0), (5.0, 4.0))], "no implied edge: nothing to trim");
    let mut s = session(&ls);
    s.script("TRIM O S 5,3.5\n\nE E 8,0\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(header(&s, "EDGEMODE"), 1);
    assert_eq!(lines(&s), vec![((-5.0, 0.0), (5.0, 0.0)), ((5.0, 3.0), (5.0, 4.0))]);
    // EXTEND to an implied edge.
    let mut s = session(&["5,3 5,4", "0,0 2,0"]);
    s.script("EXTEND O S 5,3.5\n\nE E 1.8,0\n\n").unwrap();
    assert_eq!(lines(&s), vec![((0.0, 0.0), (5.0, 0.0)), ((5.0, 3.0), (5.0, 4.0))]);
}

#[test]
fn erase_undo_and_project() {
    let mut s = ladder();
    s.script("TRIM R 5,2\n\nP V\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (15.0, 0.0))]));
    assert_eq!(header(&s, "PROJMODE"), 2);
    // Undo takes back the last trim only, then the erase.
    let mut s = ladder();
    s.script("TRIM 5,0 5,2 U\n\n").unwrap();
    assert_eq!(lines(&s), with_edges(&[((-5.0, 0.0), (0.0, 0.0)), ((-5.0, 2.0), (15.0, 2.0)), ((10.0, 0.0), (15.0, 0.0))]));
    s.script("TRIM R 10,4\n\nU U\n\n").unwrap();
    assert_eq!(lines(&s).len(), 5);
    // The whole TRIM is one undo step.
    s.script("TRIM 5,2 F 5,-1 5,1\n\n\n").unwrap();
    s.undo().unwrap();
    assert_eq!(lines(&s).len(), 5);
}

#[test]
fn extend_fence_and_crossing() {
    let mut s = session(&["10,-5 10,5", "0,0 4,0", "0,2 4,2"]);
    s.script("EXTEND F 3,-1 3,3\n\n\n").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(lines(&s), vec![((0.0, 0.0), (10.0, 0.0)), ((0.0, 2.0), (10.0, 2.0)), ((10.0, -5.0), (10.0, 5.0))]);
    let mut s = session(&["10,-5 10,5", "0,0 4,0", "0,2 4,2"]);
    s.script("EXTEND C 3,-1 5,1\n\n").unwrap();
    assert_eq!(lines(&s), vec![((0.0, 0.0), (10.0, 0.0)), ((0.0, 2.0), (4.0, 2.0)), ((10.0, -5.0), (10.0, 5.0))]);
}

#[test]
fn json_forms_take_fence_crossing_edge_mode_and_mode() {
    let mut s = ladder();
    let r = s.execute("trim", &json!({ "fence": [[5, -1], [5, 3]] })).unwrap();
    assert_eq!(r["trimmed"], 2);
    assert_eq!(lines(&s).len(), 6);

    let mut s = ladder();
    let r = s.execute("trim", &json!({ "crossing": [[4, -1], [6, 1]] })).unwrap();
    assert_eq!(r["trimmed"], 1);

    let mut s = session(&["5,3 5,4", "-5,0 15,0"]);
    let h = s.doc().unwrap().model.last().unwrap().handle.hex();
    assert!(s.execute("trim", &json!({ "handle": h, "pick": [8, 0] })).is_err());
    s.execute("trim", &json!({ "handle": h, "pick": [8, 0], "edgeMode": "extend" })).unwrap();
    assert_eq!(lines(&s), vec![((-5.0, 0.0), (5.0, 0.0)), ((5.0, 3.0), (5.0, 4.0))]);

    let mut s = session(&["20,20 25,20"]);
    let h = s.doc().unwrap().model.last().unwrap().handle.hex();
    let r = s.execute("trim", &json!({ "handle": h, "pick": [22, 20], "mode": "quick" })).unwrap();
    assert_eq!(r["handles"], json!([]));
    assert!(lines(&s).is_empty());

    let mut s = ladder();
    assert!(s.execute("trim", &json!({ "fence": [[5, -1]] })).is_err());
    assert!(s.execute("trim", &json!({ "crossing": [[5, -1], [1, 1], [2, 2]] })).is_err());
    assert!(s.execute("trim", &json!({ "fence": [[5, -1], [5, 3]], "edgeMode": "sideways" })).is_err());
    assert!(s.execute("trim", &json!({ "fence": [[5, -1], [5, 3]], "mode": 3 })).is_err());
    assert_eq!(lines(&s).len(), 4);

    let mut s = session(&["10,-5 10,5", "0,0 4,0"]);
    let r = s.execute("extend", &json!({ "fence": [[3, -1], [3, 1]] })).unwrap();
    assert_eq!(r["extended"], 1);
    assert_eq!(lines(&s), vec![((0.0, 0.0), (10.0, 0.0)), ((10.0, -5.0), (10.0, 5.0))]);
}


/// Regression for an architectural crossing: TRIM and EXTEND must preserve
/// actual editable line endpoints through Undo, Redo and DXF serialization.
#[test]
fn architectural_trim_extend_undo_redo_and_dxf_reopen() {
    let mut trim = ladder();
    let original = lines(&trim);
    trim.script("TRIM 5,0\n\n").expect("native quick TRIM");
    assert!(trim.running.is_none());
    let trimmed=with_edges(&[
        ((-5.0,0.0),(0.0,0.0)),
        ((-5.0,2.0),(15.0,2.0)),
        ((10.0,0.0),(15.0,0.0)),
    ]);
    assert_eq!(lines(&trim),trimmed,"TRIM must remove only the picked run");
    trim.undo().expect("Undo TRIM");
    assert_eq!(lines(&trim),original,"Undo restores original crossing");
    trim.redo().expect("Redo TRIM");
    assert_eq!(lines(&trim),trimmed,"Redo restores exact trim endpoints");
    let saved=cadcraft_io::write_dxf(trim.doc().unwrap());
    let reopened=cadcraft_io::read_dxf(saved.as_bytes()).expect("reopen trimmed DXF");
    let reopened_lines={
        let mut found=reopened.model.iter().filter_map(|e|match &e.kind{
            EntityKind::Line(l)=>Some(((r(l.a.x),r(l.a.y)),(r(l.b.x),r(l.b.y)))),
            _=>None,
        }).map(|(a,b)|if a<=b{(a,b)}else{(b,a)}).collect::<Vec<_>>();
        found.sort_by(|a,b|a.partial_cmp(b).unwrap());
        found
    };
    assert_eq!(reopened_lines,trimmed,"DXF must keep editable TRIM lines");

    let mut extend=session(&["10,-5 10,5","0,0 4,0","0,2 4,2"]);
    let before=lines(&extend);
    extend.script("EXTEND F 3,-1 3,3\n\n\n").expect("native EXTEND fence");
    let after=vec![
        ((0.0,0.0),(10.0,0.0)),
        ((0.0,2.0),(10.0,2.0)),
        ((10.0,-5.0),(10.0,5.0)),
    ];
    assert_eq!(lines(&extend),after,"EXTEND must stop exactly at boundary");
    extend.undo().expect("Undo EXTEND");
    assert_eq!(lines(&extend),before);
    extend.redo().expect("Redo EXTEND");
    assert_eq!(lines(&extend),after);
}


/// Regression for the native FILLET/CHAMFER commands on an architectural
/// right-angle corner; validates real editable geometry, undo and DXF reload.
/// Existing command semantics are exercised, not replaced by a mock.
#[test]
fn architectural_fillet_chamfer_native_geometry_undo_redo_and_dxf() {
    let mut s = session(&["0,0 10,0", "10,0 10,10"]);
    let handles = s.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect::<Vec<_>>();
    assert_eq!(handles.len(), 2);
    let before = lines(&s);
    s.execute("fillet", &json!({
        "h1": handles[0], "h2": handles[1],
        "p1": [2,0], "p2": [10,8], "radius": 2.0,
        "trim": true
    })).expect("native FILLET on two perpendicular LINEs");
    let filleted = lines(&s);
    assert_ne!(filleted, before, "FILLET must trim original line geometry");
    assert_eq!(s.doc().unwrap().model.iter()
        .filter(|e| matches!(&e.kind, EntityKind::Arc(_))).count(), 1,
        "FILLET must create a native editable ARC");
    s.undo().expect("Undo FILLET");
    assert_eq!(lines(&s), before);
    s.redo().expect("Redo FILLET");
    assert_eq!(lines(&s), filleted);
    let dxf = cadcraft_io::write_dxf(s.doc().unwrap());
    let reopened = cadcraft_io::read_dxf(dxf.as_bytes()).expect("FILLET DXF roundtrip");
    assert_eq!(reopened.model.iter()
        .filter(|e| matches!(&e.kind, EntityKind::Arc(_))).count(), 1,
        "FILLET arc must remain a native CAD entity after DXF reopen");

    let mut c = session(&["0,0 10,0", "10,0 10,10"]);
    let handles = c.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect::<Vec<_>>();
    let original = lines(&c);
    c.execute("chamfer", &json!({
        "h1": handles[0], "h2": handles[1],
        "p1": [2,0], "p2": [10,8], "d1": 2.0, "d2": 3.0,
        "trim": false
    })).expect("native CHAMFER with No trim");
    assert_eq!(lines(&c).len(), 3, "CHAMFER must produce an editable LINE");
    for segment in original.iter() {
        assert!(lines(&c).contains(segment), "No trim must preserve source lines");
    }
    c.undo().expect("Undo CHAMFER");
    assert_eq!(lines(&c), original);
    c.redo().expect("Redo CHAMFER");
    assert_eq!(lines(&c).len(), 3);
    let dxf = cadcraft_io::write_dxf(c.doc().unwrap());
    let reloaded = cadcraft_io::read_dxf(dxf.as_bytes()).expect("CHAMFER DXF roundtrip");
    assert_eq!(reloaded.model.iter()
        .filter(|e| matches!(&e.kind, EntityKind::Line(_))).count(), 3);
}
