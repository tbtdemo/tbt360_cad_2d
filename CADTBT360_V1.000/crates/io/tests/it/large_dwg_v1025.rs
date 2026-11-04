//! V1.025 large-drawing regression: structural integrity and diagnostic timings.
//! Timings are diagnostic only; Windows hosted-runner performance varies widely.

use std::time::Instant;

use cadcraft_doc::{Common, Drawing, EntityKind, Line, Space};
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
