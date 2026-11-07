use super::*;

/// Snap name always starts with the native English name. Vietnamese
/// translation is an aid, never a change to native OSNAP mode or hit point.
fn bilingual_snap_label(english:&str)->String {
    let vietnamese=crate::i18n::tr("vi",english);
    if vietnamese!=english {format!("{english} ({vietnamese})")}
    else {english.to_owned()}
}

/// Keep OSNAP name readable at canvas corners (including HiDPI): marker
/// remains at the exact engine snap point, only its label is repositioned.
fn snap_label_rect(at:Pos2,size:egui::Vec2,clip:Rect)->Rect {
    let width=size.x.min(clip.width().max(0.0));
    let height=size.y.min(clip.height().max(0.0));
    let x=if at.x+10.0+width>clip.right() {at.x-width-10.0}
        else{at.x+10.0};
    let y=if at.y+10.0+height>clip.bottom() {at.y-height-10.0}
        else{at.y+10.0};
    Rect::from_min_size(pos2(
        x.clamp(clip.left(),(clip.right()-width).max(clip.left())),
        y.clamp(clip.top(),(clip.bottom()-height).max(clip.top()))),
        vec2(width,height))
}

pub(super) fn draw_snap_marker(p: &egui::Painter, at: Pos2, hit: &SnapHit) {
    let t = Tokens::get();
    let s = crate::theme::snap_marker_half();
    let st = Stroke::new(2.0, t.snap);
    use snap::mode;
    match hit.mode {
        mode::END => {
            p.rect_stroke(Rect::from_center_size(at, vec2(2.0 * s, 2.0 * s)), 0.0, st, egui::StrokeKind::Middle);
        }
        mode::MID => {
            p.add(Shape::closed_line(vec![at + vec2(0.0, -s), at + vec2(s, s * 0.8), at + vec2(-s, s * 0.8)], st));
        }
        mode::CEN | mode::GCEN => {
            p.circle_stroke(at, s, st);
        }
        mode::QUA => {
            p.add(Shape::closed_line(vec![at + vec2(0.0, -s), at + vec2(s, 0.0), at + vec2(0.0, s), at + vec2(-s, 0.0)], st));
        }
        mode::INT | mode::APP => {
            p.line_segment([at + vec2(-s, -s), at + vec2(s, s)], st);
            p.line_segment([at + vec2(s, -s), at + vec2(-s, s)], st);
            if hit.mode == mode::APP {
                p.rect_stroke(Rect::from_center_size(at, vec2(2.0 * s, 2.0 * s)), 0.0, st, egui::StrokeKind::Middle);
            }
        }
        mode::PER => {
            p.line(vec![at + vec2(-s, -s), at + vec2(-s, s), at + vec2(s, s)], st);
            p.line(vec![at + vec2(-s, 0.0), at + vec2(0.0, 0.0), at + vec2(0.0, s)], st);
        }
        mode::TAN => {
            p.circle_stroke(at, s * 0.8, st);
            p.line_segment([at + vec2(-s, -s), at + vec2(s, -s)], st);
        }
        mode::NOD => {
            p.circle_stroke(at, s, st);
            p.line_segment([at + vec2(-s, -s), at + vec2(s, s)], st);
            p.line_segment([at + vec2(s, -s), at + vec2(-s, s)], st);
        }
        mode::INS => {
            p.line(
                vec![
                    at + vec2(-s, -s),
                    at + vec2(0.0, -s),
                    at + vec2(0.0, 0.0),
                    at + vec2(s, 0.0),
                    at + vec2(s, s),
                    at + vec2(-s, s),
                    at + vec2(-s, -s),
                ],
                st,
            );
        }
        _ => {
            p.line(vec![at + vec2(-s, -s), at + vec2(s, -s), at + vec2(-s, s), at + vec2(s, s), at + vec2(-s, -s)], st);
        }
    }
    // Paint the label once, with its background. Drawing bare text first leaves
    // black ghost pixels when the label background is recomputed over the canvas.
    let galley = p.layout_no_wrap(bilingual_snap_label(hit.name), crate::theme::small(), Color32::BLACK);
    let r = snap_label_rect(at,galley.size()+vec2(6.0,2.0),p.clip_rect());
    p.rect_filled(r, 2.0, Color32::from_rgb(0xff, 0xff, 0xe1));
    p.galley(r.min + vec2(3.0, 1.0), galley, Color32::BLACK);
}

/// Acquired tracking points (small `+`, `//` for a Parallel line), the dotted paths the cursor
/// is on and the tooltip naming them ("Endpoint: 5.0000 < 90°").
pub(super) fn draw_tracking(p: &egui::Painter, xf: &Xf, acquired: &[snap::tracking::Acquired], track: Option<&snap::tracking::Track>, far: f64) {
    use snap::tracking::{Acquired, PathShape};
    let t = Tokens::get();
    let st = Stroke::new(1.0, t.snap);
    for a in acquired {
        let c = xf.to_screen(a.at());
        if let Acquired::Parallel { .. } = a {
            p.line_segment([c + vec2(-5.0, 3.0), c + vec2(-1.0, -5.0)], st);
            p.line_segment([c + vec2(1.0, 3.0), c + vec2(5.0, -5.0)], st);
        } else {
            p.line_segment([c + vec2(-4.0, 0.0), c + vec2(4.0, 0.0)], st);
            p.line_segment([c + vec2(0.0, -4.0), c + vec2(0.0, 4.0)], st);
        }
    }
    let Some(tr) = track else { return };
    let mut tips = Vec::new();
    for path in &tr.paths {
        // Around the cursor only (a path's origin may be far off screen).
        let pts: Vec<Pos2> = match path.shape {
            PathShape::Ray { origin, dir } => {
                let back = (tr.point - origin).dot(dir).clamp(0.0, far);
                vec![xf.to_screen(tr.point - dir * back), xf.to_screen(tr.point + dir * far)]
            }
            PathShape::Line { dir, .. } => vec![xf.to_screen(tr.point - dir * far), xf.to_screen(tr.point + dir * far)],
            PathShape::Circle { center, radius, .. } => {
                let (a0, span) = (center.angle_to(tr.point), (far / radius).min(std::f64::consts::PI));
                (0..=96).map(|i| xf.to_screen(Vec2::polar(center, radius, a0 - span + span * f64::from(i) / 48.0))).collect()
            }
            PathShape::Ellipse { center, major, ratio, .. } => {
                let e = cadcraft_geom::Ellipse::full(center, major, ratio);
                let (t0, span) = (e.param_of(tr.point), (far / (major.len() * ratio.min(1.0))).min(std::f64::consts::PI));
                (0..=96).map(|i| xf.to_screen(e.at_param(t0 - span + span * f64::from(i) / 48.0))).collect()
            }
        };
        p.extend(Shape::dotted_line(&pts, t.snap, 6.0, 1.0));
        let (distance, angle) = (
            cadcraft_engine::units::format_distance(path.from.dist(tr.point), 2, 4),
            (path.from.angle_to(tr.point).to_degrees().round() as i64).rem_euclid(360),
        );
        tips.push(if path.name == "Polar" {
            crate::tf!("Polar: {distance} < {angle}°", distance = distance, angle = angle)
        } else {
            crate::tf!("{name}: {distance} < {angle}°", name = crate::i18n::t(path.name), distance = distance, angle = angle)
        });
    }
    if let Some(o) = tr.object {
        tips.push(crate::i18n::t(o).to_string());
    }
    let at = xf.to_screen(tr.point);
    if tr.paths.len() > 1 || tr.object.is_some() {
        p.line_segment([at + vec2(-4.0, -4.0), at + vec2(4.0, 4.0)], st);
        p.line_segment([at + vec2(4.0, -4.0), at + vec2(-4.0, 4.0)], st);
    }
    tooltip(p, at + vec2(16.0, 18.0), &tips.join(", "));
}

/// The rubber-band and snap-adjusted cursor point for the current prompt (the engine's
/// `Session::snap_cursor`: object snaps, acquiring tracking points, grid, tracking paths, ortho).
pub(super) fn effective_point(app: &mut CadApp, raw: Vec2, xf: &Xf, ctx: &egui::Context) -> Vec2 {
    let prompt = app.session.current_prompt();
    let wants_point = prompt.as_ref().is_some_and(cadcraft_engine::pointmod::accepts);
    let base = prompt.as_ref().and_then(|p| p.base);
    let deferred = prompt.as_ref().is_some_and(|p| p.deferred);
    let corner = prompt.as_ref().is_some_and(|p| p.corner);
    app.canvas.snap = None;
    app.canvas.track = None;
    if !wants_point || app.session.pending_window.is_some() {
        if !wants_point {
            app.session.tracking.clear();
            // Show actual object-snap candidates while hovering, even before a draw command.
            // Do not silently move the click cursor or alter idle selection: snap is only
            // applied to input points while the engine explicitly requests a point.
            if app.session.pending_window.is_none() {
                let osmode=app.session.settings.osmode;
                if osmode!=0 && osmode & snap::mode::OFF==0 {
                    let aperture=app.session.settings.aperture / xf.scale;
                    if let Ok(st)=app.session.state() {
                        let hit=snap::osnap(&st.doc,&st.edit_space(),raw,aperture,osmode,None,false);
                        app.canvas.snap=hit;
                    }
                }
            }
        }
        return raw;
    }
    let (now, shift) = ctx.input(|i| (i.time, i.modifiers.shift));
    let q =
        snap::tracking::CursorQuery { base, deferred, corner, aperture: app.session.settings.aperture / xf.scale, tol: 6.0 / xf.scale, now, shift };
    let r = app.session.snap_cursor(raw, &q);
    if r.wait {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(snap::tracking::DWELL / 4.0));
    }
    app.canvas.snap = r.snap;
    app.canvas.track = r.track;
    r.point
}

/// Dispatch one of the earlier physical releases from a slow UI frame.  Each
/// release needs its OWN point-prompt OSNAP/ortho/track resolution: sending the
/// raw pointer world coordinate bypasses snaps, while reusing the final hover
/// hit would collapse several fast clicks onto the same vertex.
pub(super) fn input_earlier_snapped_point(app: &mut CadApp, raw: Vec2, xf: &Xf, ctx: &egui::Context) -> cadcraft_engine::Result<()> {
    let p = effective_point(app, raw, xf, ctx);
    app.canvas.cursor = Some(p);
    app.session.cursor = p;
    app.session.cursor_deferred = app.canvas.snap.and_then(|h| h.deferred);
    // Match the normal click path, including single-pick END/NON overrides
    // and Tab cycling, rather than trusting the final cursor in the batch.
    override_cursor(app, raw, xf);
    match app.canvas.snap {
        Some(hit) => app.session.input_snapped(hit),
        None => app.session.input(Input::Point(app.canvas.cursor.unwrap_or(p))),
    }
}

fn tooltip(p: &egui::Painter, at: Pos2, text: &str) {
    let galley = p.layout_no_wrap(text.to_string(), crate::theme::small(), Color32::from_rgb(0x20, 0x20, 0x20));
    let r = Rect::from_min_size(at, galley.size() + vec2(8.0, 4.0));
    p.rect_filled(r, 2.0, Color32::from_rgb(0xf4, 0xf4, 0xf4));
    p.galley(r.min + vec2(4.0, 2.0), galley, Color32::BLACK);
}

/// Half the side of the pick box: the pick aperture (PICKBOX × 1.5 pixels, as `select::pick` is
/// called with), so the square shows what a click hits. `None` when PICKBOX is 0 (no box).
/// The candidate Tab has cycled to, if cycling: a Tab press starts or advances it, and the
/// cursor leaving the aperture where it started ends it.
fn snap_cycle(app: &mut CadApp, raw: Vec2, ap: f64) -> Option<usize> {
    let tab = std::mem::take(&mut app.canvas.snap_tab);
    let c = &mut app.canvas.snap_cycle;
    if c.is_some_and(|c| c.anchor.dist(raw) > ap) {
        *c = None;
    }
    if tab {
        *c = Some(c.map_or(SnapCycle { anchor: raw, index: 1 }, |c| SnapCycle { index: c.index + 1, ..c }));
    }
    c.map(|c| c.index)
}

/// The cursor under a one-pick object snap override (`Session::snap_override`: END, NON…) or
/// while Tab cycles the candidates: the snap of that mode (or the cycled candidate) replaces
/// what the running snaps found; with none, a running snap no longer holds the cursor.
pub(super) fn override_cursor(app: &mut CadApp, raw: Vec2, xf: &Xf) {
    let over = app.session.snap_override();
    let ap = app.session.settings.aperture / xf.scale;
    let cycle = snap_cycle(app, raw, ap);
    let prompt = app.session.current_prompt();
    let wants_point = prompt.as_ref().is_some_and(cadcraft_engine::pointmod::accepts);
    if (over.is_none() && cycle.is_none()) || !wants_point || app.session.pending_window.is_some() {
        return;
    }
    let base = prompt.as_ref().and_then(|p| p.base);
    let deferred = prompt.as_ref().is_some_and(|p| p.deferred);
    let s = &app.session.settings;
    let osmode = over.unwrap_or(s.osmode) | if s.osnaphatch { snap::mode::HATCH } else { 0 };
    let Ok(st) = app.session.state() else { return };
    let hit = match cycle {
        Some(i) => {
            let cands = cadcraft_engine::pointmod::snap_candidates(&st.doc, &st.edit_space(), raw, ap, osmode, base, deferred);
            cands.get(i % cands.len().max(1)).copied()
        }
        None => snap::osnap(&st.doc, &st.edit_space(), raw, ap, osmode, base, deferred),
    };
    let p = match (hit, app.canvas.snap) {
        (Some(h), _) => {
            app.canvas.snap = Some(h);
            h.point
        }
        (None, Some(_)) => {
            app.canvas.snap = None;
            raw
        }
        (None, None) => return,
    };
    app.canvas.cursor = Some(p);
    app.session.cursor = p;
    app.session.cursor_deferred = app.canvas.snap.and_then(|h| h.deferred);
}

fn pickbox_half(pickbox: f64) -> Option<f32> {
    let half = (pickbox * 1.5) as f32;
    (half.is_finite() && half >= 1.0).then_some(half)
}

pub(super) fn draw_crosshair(app: &CadApp, p: &egui::Painter, rect: Rect, at: Pos2, pickbox: bool, ink: Color32) {
    let st = Stroke::new(1.0, ink);
    let len = (rect.height() * app.session.settings.cursorsize as f32 / 100.0).max(12.0);
    let pb = if pickbox { pickbox_half(app.session.settings.pickbox) } else { None };
    let gap = pb.unwrap_or(0.0);
    // Along the isometric plane's axes in isometric drafting.
    if app.session.settings.isodraft {
        for a in cadcraft_engine::snap::iso::cursor_axes(&app.session.settings) {
            let dir = vec2(a.cos() as f32, -(a.sin() as f32));
            p.line_segment([at - dir * len, at - dir * gap], st);
            p.line_segment([at + dir * gap, at + dir * len], st);
        }
    } else {
        p.line_segment([pos2(at.x - len, at.y), pos2(at.x - gap, at.y)], st);
        p.line_segment([pos2(at.x + gap, at.y), pos2(at.x + len, at.y)], st);
        p.line_segment([pos2(at.x, at.y - len), pos2(at.x, at.y - gap)], st);
        p.line_segment([pos2(at.x, at.y + gap), pos2(at.x, at.y + len)], st);
    }
    if let Some(pb) = pb {
        p.rect_stroke(Rect::from_center_size(at, vec2(pb * 2.0, pb * 2.0)), 0.0, st, egui::StrokeKind::Middle);
    }
}

#[cfg(test)]
mod snap_label_layout_tests {
    use super::*;

    #[test]
    fn batched_physical_clicks_each_use_their_own_native_endpoint_snap() {
        use crate::Services;
        use cadcraft_doc::EntityKind;
        let mut app = CadApp::new(cadcraft_engine::Session::new(), Services::default());
        app.session.cmdline("LINE 0,0 10,0").unwrap();
        app.session.cmdline("").unwrap();
        app.session.settings.osmode = cadcraft_engine::snap::mode::END;
        app.session.settings.aperture = 12.0;
        app.session.cmdline("LINE").unwrap();
        let ctx = egui::Context::default();
        let xf = Xf {
            rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0)),
            center: Vec2::new(5.0, 0.0),
            scale: 20.0,
        };
        // Both releases arrive before a repaint, at distinct endpoint locations.
        // They must not be raw (9.9, 0.1)/(0.1, 0.1) or the last hover's snap.
        input_earlier_snapped_point(&mut app, Vec2::new(9.9, 0.1), &xf, &ctx)
            .expect("first OSNAP click");
        input_earlier_snapped_point(&mut app, Vec2::new(0.1, 0.1), &xf, &ctx)
            .expect("second OSNAP click");
        let lines: Vec<_> = app.session.doc().unwrap().model.iter().filter_map(|e| {
            if let EntityKind::Line(line) = &e.kind {
                Some((line.a.xy(), line.b.xy()))
            } else { None }
        }).collect();
        assert_eq!(lines.len(), 2, "native LINE must be added by two snapped clicks");
        assert!(lines.iter().any(|(a,b)|
            a.near(Vec2::new(10.0,0.0),1e-9) &&
            b.near(Vec2::new(0.0,0.0),1e-9)),
            "each fast physical release must preserve its own exact END snap: {lines:?}");
    }

    #[test]
    fn fast_click_osnap_aperture_is_screen_pixel_stable_across_zoom_levels() {
        use crate::Services;
        use cadcraft_doc::EntityKind;
        for scale in [5.0, 20.0, 100.0] {
            let mut app = CadApp::new(cadcraft_engine::Session::new(), Services::default());
            app.session.cmdline("LINE 0,0 10,0").unwrap();
            app.session.cmdline("").unwrap();
            app.session.settings.osmode = cadcraft_engine::snap::mode::END;
            app.session.settings.aperture = 12.0;
            app.session.cmdline("LINE").unwrap();
            let ctx = egui::Context::default();
            let xf = Xf { rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0)),
                center: Vec2::new(5.0, 0.0), scale };
            // The first click misses the actual endpoint by 3 screen pixels,
            // which must snap at every zoom. The second misses the opposite end
            // by 3 pixels. No stale snap may leak between physical events.
            input_earlier_snapped_point(&mut app, Vec2::new(10.0 - 3.0 / scale, 0.0), &xf, &ctx)
                .expect("END snap on first batched click");
            input_earlier_snapped_point(&mut app, Vec2::new(3.0 / scale, 0.0), &xf, &ctx)
                .expect("END snap on second batched click");
            let lines: Vec<_> = app.session.doc().unwrap().model.iter().filter_map(|e| {
                if let EntityKind::Line(line) = &e.kind { Some((line.a.xy(), line.b.xy())) } else { None }
            }).collect();
            assert!(lines.iter().any(|(a,b)|
                a.near(Vec2::new(10.0,0.0),1e-9) && b.near(Vec2::new(0.0,0.0),1e-9)),
                "END must follow world coordinates with constant screen-pixel aperture at scale {scale}: {lines:?}");
        }
    }

    #[test]
    fn fast_click_midpoint_and_circle_center_become_native_line_endpoints() {
        use crate::Services;
        use cadcraft_doc::{Circle, Common, EntityKind, Space};
        use cadcraft_geom::Vec3;
        let mut app = CadApp::new(cadcraft_engine::Session::new(), Services::default());
        app.session.cmdline("LINE 0,0 10,0").unwrap();
        app.session.cmdline("").unwrap();
        // A circle centre is not itself a drawn pixel, so this checks the
        // engine's true CEN snap instead of an approximate nearest selection.
        app.session.doc_mut().unwrap().add(
            &Space::Model, Common::default(),
            EntityKind::Circle(Circle { center: Vec3::new(20.0, 0.0, 0.0), radius: 2.0 })
        ).unwrap();
        app.session.settings.osmode =
            cadcraft_engine::snap::mode::MID | cadcraft_engine::snap::mode::CEN;
        app.session.settings.aperture = 12.0;
        app.session.cmdline("LINE").unwrap();
        let ctx = egui::Context::default();
        let xf = Xf { rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0,600.0)),
            center: Vec2::new(10.0,0.0), scale: 20.0 };
        input_earlier_snapped_point(&mut app, Vec2::new(5.1,0.1), &xf, &ctx)
            .expect("first release must snap to line MID");
        input_earlier_snapped_point(&mut app, Vec2::new(20.1,0.1), &xf, &ctx)
            .expect("second release must snap to circle CEN");
        let actual: Vec<_> = app.session.doc().unwrap().model.iter().filter_map(|e| {
            if let EntityKind::Line(l) = &e.kind { Some((l.a.xy(),l.b.xy())) } else { None }
        }).collect();
        assert!(actual.iter().any(|(a,b)|
            a.near(Vec2::new(5.0,0.0),1e-9) && b.near(Vec2::new(20.0,0.0),1e-9)),
            "canvas must send distinct native MID/CEN snap points, not raw positions: {actual:?}");
    }

    #[test]
    fn fast_canvas_intersection_snap_is_exact_and_osnap_off_keeps_raw_input() {
        use crate::Services;
        use cadcraft_doc::EntityKind;
        for disabled in [false, true] {
            let mut app = CadApp::new(cadcraft_engine::Session::new(), Services::default());
            app.session.cmdline("LINE 0,0 10,0").unwrap();
            app.session.cmdline("").unwrap();
            app.session.cmdline("LINE 5,-5 5,5").unwrap();
            app.session.cmdline("").unwrap();
            app.session.settings.osmode =
                if disabled { cadcraft_engine::snap::mode::INT | cadcraft_engine::snap::mode::END | cadcraft_engine::snap::mode::OFF }
                else { cadcraft_engine::snap::mode::INT | cadcraft_engine::snap::mode::END };
            app.session.settings.aperture = 12.0;
            app.session.cmdline("LINE").unwrap();
            let ctx = egui::Context::default();
            let xf = Xf { rect: Rect::from_min_size(Pos2::ZERO,vec2(800.0,600.0)),
                center: Vec2::new(5.0,0.0), scale: 20.0 };
            let raw_first = Vec2::new(5.1,0.1);
            let raw_second = Vec2::new(0.1,0.1);
            // The same canvas path decides both the marker location and
            // the geometry sent to the native CAD command. An OSNAP OFF
            // marker must never pretend a snap occurred.
            let shown_first = effective_point(&mut app, raw_first, &xf, &ctx);
            if disabled {
                assert!(app.canvas.snap.is_none(), "OSNAP OFF must suppress marker");
                assert!(shown_first.near(raw_first, 1e-9));
            } else {
                let hit = app.canvas.snap.expect("INT marker for first click");
                assert_eq!(hit.mode,cadcraft_engine::snap::mode::INT);
                assert!(hit.point.near(Vec2::new(5.0,0.0),1e-9));
                assert!(shown_first.near(hit.point,1e-9),
                    "marker and effective cursor must share one engine point");
            }
            input_earlier_snapped_point(&mut app,raw_first,&xf,&ctx)
                .expect("first fast mouse release");
            let shown_second = effective_point(&mut app,raw_second,&xf,&ctx);
            if disabled {
                assert!(app.canvas.snap.is_none());
                assert!(shown_second.near(raw_second,1e-9));
            } else {
                let hit = app.canvas.snap.expect("END marker for second click");
                assert_eq!(hit.mode,cadcraft_engine::snap::mode::END);
                assert!(hit.point.near(Vec2::new(0.0,0.0),1e-9));
                assert!(shown_second.near(hit.point,1e-9));
            }
            input_earlier_snapped_point(&mut app,raw_second,&xf,&ctx)
                .expect("second fast mouse release");
            let actual: Vec<_> = app.session.doc().unwrap().model.iter().filter_map(|e| {
                if let EntityKind::Line(l) = &e.kind { Some((l.a.xy(),l.b.xy())) } else { None }
            }).collect();
            assert_eq!(actual.len(),3);
            if disabled {
                assert!(actual.iter().any(|(a,b)| a.near(Vec2::new(5.1,0.1),1e-9)
                    && b.near(Vec2::new(0.1,0.1),1e-9)),
                    "OSNAP OFF must not force an intersection or endpoint: {actual:?}");
            } else {
                assert!(actual.iter().any(|(a,b)| a.near(Vec2::new(5.0,0.0),1e-9)
                    && b.near(Vec2::new(0.0,0.0),1e-9)),
                    "OSNAP INT/END must create an exact native LINE: {actual:?}");
            }
        }
    }

    #[test]
    fn native_snap_names_remain_english_first_with_vietnamese_explanation() {
        for (english,translation) in [
            ("Endpoint","Điểm cuối"),("Midpoint","Trung điểm"),
            ("Center","Tâm"),("Intersection","Giao điểm"),
            ("Perpendicular","Vuông góc"),("Tangent","Tiếp tuyến"),
        ] {
            assert_eq!(bilingual_snap_label(english),format!("{english} ({translation})"));
        }
        assert_eq!(bilingual_snap_label("UnknownNativeSnap"),"UnknownNativeSnap");
    }

    #[test]
    fn osnap_labels_fit_all_canvas_corners_without_changing_snap_location() {
        let canvas=Rect::from_min_max(pos2(100.0,80.0),pos2(550.0,310.0));
        for at in [pos2(101.0,81.0),pos2(549.0,81.0),
                   pos2(101.0,309.0),pos2(549.0,309.0),
                   pos2(330.0,190.0)] {
            let before=at;
            let label=snap_label_rect(at,vec2(118.0,18.0),canvas);
            assert!(label.left()>=canvas.left()&&label.right()<=canvas.right());
            assert!(label.top()>=canvas.top()&&label.bottom()<=canvas.bottom());
            assert_eq!(at,before,"the native OSNAP coordinate must not change");
        }
    }
}
