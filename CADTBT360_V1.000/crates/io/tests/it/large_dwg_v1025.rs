//! V1.025 large-drawing regression: structural integrity and diagnostic timings.
//! Timings are diagnostic only; Windows hosted-runner performance varies widely.

use std::time::Instant;

use cadcraft_color::Color;
use cadcraft_doc::{Circle, Common, Drawing, EntityKind, Layer, Line, Space};
use cadcraft_geom::Vec3;
use cadcraft_io::{SaveOptions, Version, read, write_with};

fn verify_dwg_roundtrip(count: usize) {
    let started = Instant::now();
    let mut drawing = Drawing::new_metric();
    for i in 0..count {
        let x = (i % 250) as f64 * 10.0;
        let y = (i / 250) as f64 * 10.0;
        drawing.add(&Space::Model, Common::default(), EntityKind::Line(Line {
            a: Vec3::new(x, y, 0.0),
            b: Vec3::new(x + 5.0, y + 2.0, 0.0),
        })).expect("add line");
    }
    assert_eq!(drawing.model.len(), count);
    let generate_ms = started.elapsed().as_millis();

    let write_started = Instant::now();
    let dwg = write_with(&drawing, "v1025-large.dwg", &SaveOptions { version: Some(Version::R2018), binary: false }).expect("write R2018 DWG");
    let write_ms = write_started.elapsed().as_millis();
    assert!(!dwg.is_empty());
    assert_eq!(cadcraft_dwg::version(&dwg).as_deref(), Some("AC1032"));

    let reopen_started = Instant::now();
    let reopened = read(&dwg, "v1025-large.dwg").expect("reopen R2018 DWG");
    let read_ms = reopen_started.elapsed().as_millis();
    assert_eq!(reopened.model.len(), count, "DWG bridge silently lost or duplicated geometry");

    let mut total_lines = 0usize;
    let mut sum_start_x = 0.0f64;
    let mut sum_start_y = 0.0f64;
    for e in reopened.model.iter() {
        match &e.kind {
            EntityKind::Line(line) => {
                total_lines += 1;
                sum_start_x += line.a.x;
                sum_start_y += line.a.y;
                assert!((line.b.x - line.a.x - 5.0).abs() < 1e-7);
                assert!((line.b.y - line.a.y - 2.0).abs() < 1e-7);
            }
            kind => panic!("unexpected entity type in large DWG: {kind:?}"),
        }
    }
    assert_eq!(total_lines, count);
    let expected_sum_x: f64 = (0..count).map(|i| (i % 250) as f64 * 10.0).sum();
    let expected_sum_y: f64 = (0..count).map(|i| (i / 250) as f64 * 10.0).sum();
    assert!((sum_start_x - expected_sum_x).abs() < 1e-4);
    assert!((sum_start_y - expected_sum_y).abs() < 1e-4);
    eprintln!("V1.025 large DWG: entities={} bytes={} generate_ms={generate_ms} write_ms={write_ms} read_ms={read_ms}", count, dwg.len());
}

#[test]
fn v1025_large_dwg_r2018_preserves_entities() {
    verify_dwg_roundtrip(10_000);
}

#[test]
fn v1025_medium_dwg_r2018_50000_entities() {
    verify_dwg_roundtrip(50_000);
}

#[test]
fn v1025_stress_dwg_r2018_100000_entities() {
    verify_dwg_roundtrip(100_000);
}

#[test]
fn v1025_mixed_dwg_r2018_20000_entities_and_layers() {
    const COUNT: usize = 20_000;
    let started = Instant::now();
    let mut drawing = Drawing::new_metric();
    drawing.layers.push(Layer { name: "ROAD-LINES".into(), color: Color::Index(1), ..Layer::default() });
    drawing.layers.push(Layer { name: "ROAD-CIRCLES".into(), color: Color::Index(5), ..Layer::default() });
    for i in 0..COUNT {
        let x = (i % 200) as f64 * 5.0;
        let y = (i / 200) as f64 * 5.0;
        let (layer, kind) = if i % 2 == 0 {
            ("ROAD-LINES", EntityKind::Line(Line { a: Vec3::new(x, y, 0.0), b: Vec3::new(x + 2.0, y + 1.0, 0.0) }))
        } else {
            ("ROAD-CIRCLES", EntityKind::Circle(Circle { center: Vec3::new(x, y, 0.0), radius: 2.5 }))
        };
        drawing.add(&Space::Model, Common { layer: layer.into(), ..Common::default() }, kind).expect("create mixed entity");
    }
    let dwg = write_with(&drawing, "v1025-mixed.dwg", &SaveOptions { version: Some(Version::R2018), binary: false }).expect("write mixed DWG");
    let saved_at = started.elapsed().as_millis();
    let reopened = read(&dwg, "v1025-mixed.dwg").expect("reopen mixed DWG");
    let finished_at = started.elapsed().as_millis();
    assert_eq!(reopened.model.len(), COUNT);
    assert_eq!(reopened.layer("ROAD-LINES").expect("line layer").color, Color::Index(1));
    assert_eq!(reopened.layer("ROAD-CIRCLES").expect("circle layer").color, Color::Index(5));
    let mut line_count = 0usize;
    let mut circle_count = 0usize;
    for e in reopened.model.iter() {
        match &e.kind {
            EntityKind::Line(line) => {
                assert_eq!(e.common.layer, "ROAD-LINES");
                assert!((line.b.x - line.a.x - 2.0).abs() < 1e-7);
                assert!((line.b.y - line.a.y - 1.0).abs() < 1e-7);
                line_count += 1;
            }
            EntityKind::Circle(circle) => {
                assert_eq!(e.common.layer, "ROAD-CIRCLES");
                assert!((circle.radius - 2.5).abs() < 1e-7);
                circle_count += 1;
            }
            kind => panic!("mixed DWG changed entity kind: {kind:?}"),
        }
    }
    assert_eq!((line_count, circle_count), (10_000, 10_000));
    eprintln!("V1.025 mixed DWG: entities={COUNT} bytes={} save_elapsed_ms={saved_at} total_elapsed_ms={finished_at}", dwg.len());
}
