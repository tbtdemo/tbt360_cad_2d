//! Independent process for real DXF clipboard IO: IoHooks uses OnceLock.
use cadcraft_engine::Session;
use cadcraft_engine::cmd::clipboard;
use cadcraft_engine::cmd::file::{IoHooks, set_io};
use cadcraft_engine::doc::{Common, Drawing, Entity, EntityKind, Line, RawTag};
use cadcraft_engine::geom::Vec3;
use serde_json::json;

fn line(d: &mut Drawing, layer: &str, a: (f64, f64), b: (f64, f64)) -> Entity {
    Entity { handle: d.new_handle(), common: Common { layer: layer.into(), ..Common::default() }, kind: EntityKind::Line(Line { a: Vec3::new(a.0, a.1, 0.0), b: Vec3::new(b.0, b.1, 0.0) }) }
}

#[test]
fn system_clipboard_real_dxf_preserves_bim_scope_and_repairs_identity_on_paste() {
    set_io(IoHooks {
        read: |bytes,name| cadcraft_io::read(bytes,name).map_err(|e|e.to_string()),
        write: |d,name| cadcraft_io::write(d,name).map_err(|e|e.to_string()),
        plot: None,
    });

    let mut source=Drawing::new_metric();
    let mut e=line(&mut source,"0",(0.0,0.0),(10.0,0.0));
    let old=e.handle;
    e.common.xdata=vec![
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision=7;entity={}",old.hex())},
    ];
    source.model.push(e);
    // Diagnose source DXF serialization separately from COPYBASE and system clipboard.
    let source_dxf = cadcraft_io::write(&source, "source-probe.dxf").expect("source DXF serialization");
    let source_text = String::from_utf8(source_dxf).expect("source ASCII DXF");
    assert!(source_text.contains("TBT360_BIM"), "DXF writer lost BIM metadata before COPYBASE");

    let mut a=Session::empty();
    a.open_drawing(source,"A",None);
    a.execute("copybase",&json!({"handles":[old.hex()],"base":[0,0]})).unwrap();
    assert!(a.clipboard.iter().any(|e| e.common.xdata.iter().any(|t| t.code == 1001 && t.value == "TBT360_BIM")), "copybase lost source BIM XData before DXF serialization");
    // Reproduce the clipboard intermediary directly to separate DXF exporter
    // loss from adoption of entity metadata during transport.
    let mut probe = Drawing::new_metric();
    let source_copy = a.clipboard.first().expect("copybase selection").clone();
    let mut transported = source_copy.clone();
    transported.handle = probe.new_handle();
    probe.model.push(transported);
    let probe_text = String::from_utf8(cadcraft_io::write(&probe, "clipboard-probe.dxf").unwrap()).unwrap();
    assert!(probe_text.contains("TBT360_BIM"), "intermediate drawing lost XData without adopt");
    let text=clipboard::system_text(&a).expect("real DXF clipboard text");
    assert!(text.contains("TBT360_BIM"));

    let mut b=Session::empty();
    b.open_drawing(Drawing::new_metric(),"B",None);
    assert!(clipboard::load_system_text(&mut b,&text));
    b.execute("pasteclip",&json!({"at":[0,0]})).unwrap();

    let pasted=b.doc().unwrap().model.iter().next().unwrap();
    // Handles are drawing-local. Separate drawings may legitimately allocate
    // the same numeric handle; validate identity against the pasted entity instead.
    assert_eq!(b.doc().unwrap().model.iter().filter(|e| e.handle == pasted.handle).count(), 1);
    let at=pasted.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
    let payload=&pasted.common.xdata[at+1].value;
    assert!(payload.contains("floor=2;discipline=ELEC;owner=u1;revision=1"), "PASTE must retain BIM scope but initialize a new object revision; got: {payload}");
    assert!(payload.contains(&format!("entity={}",pasted.handle.hex())));
}

