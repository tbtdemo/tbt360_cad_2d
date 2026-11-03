//! The drawing area: grid, entities, selection highlighting, grips, rubber bands, object-snap
//! markers, crosshair cursor, ViewCube, UCS icon and mouse handling.

use cadcraft_color::Rgb;
use cadcraft_doc::Handle;
use cadcraft_engine::Input;
use cadcraft_engine::snap::{self, SnapHit};
use cadcraft_geom::{Bounds2, Vec2};
use cadcraft_render::{DisplayList, Kind};
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::CadApp;
use crate::theme::Tokens;

mod cursor;
mod cycle;
mod draw;
mod dyn_boxes;
mod grid;
mod grips;
mod list;
mod pick;
mod viewport;
mod widgets;

use cursor::{draw_crosshair, draw_snap_marker, draw_tracking, effective_point, override_cursor};
use cycle::{CycleList, cycle_badge, cycle_click, cycle_list, over_canvas_menu, shift_space_cycle};
pub(crate) use draw::{Look, draw_highlight, draw_list};
use draw::draw_list_filtered;
use draw::{draw_drag_gpu, draw_highlight_gpu, draw_list_gpu, list_mesh_key, selection_area_fill};
use dyn_boxes::dynamic_input;
pub(crate) use dyn_boxes::length_units;
use grid::draw_grid;
use grips::{click_grip, grip_at, grip_menu, grip_options, grip_rect};
#[cfg(test)]
use grips::usable_hot_grips;
use list::{PrimIndex, ensure_list, rebuilt_preview};
#[cfg(test)]
use list::{preview_options, prims_of};
use pick::{edit_text_at, hatch_at, wheel_zoom_points};
#[cfg(test)]
use viewport::viewport_highlight_tolerance;
use viewport::{VpContent, VpHighlight, draw_vp_content, ensure_vp_content};
use widgets::{draw_tiles, draw_ucs_icon, draw_viewcube, tile_rects, tile_xf, viewport_label};

/// World ↔ screen transform for the canvas rect.
#[derive(Clone, Copy, Debug)]
pub struct Xf {
    pub rect: Rect,
    pub center: Vec2,
    /// Pixels (points) per drawing unit.
    pub scale: f64,
}

impl Xf {
    pub fn to_screen(&self, p: Vec2) -> Pos2 {
        let c = self.rect.center();
        pos2(c.x + ((p.x - self.center.x) * self.scale) as f32, c.y - ((p.y - self.center.y) * self.scale) as f32)
    }
    pub fn to_world(&self, p: Pos2) -> Vec2 {
        let c = self.rect.center();
        Vec2::new(self.center.x + f64::from(p.x - c.x) / self.scale, self.center.y - f64::from(p.y - c.y) / self.scale)
    }
    pub fn world_bounds(&self) -> Bounds2 {
        Bounds2::new(self.to_world(self.rect.left_bottom()), self.to_world(self.rect.right_top()))
    }
}

/// What a display list was built from: drawing uid, revision, tessellation band, hatch level of
/// detail band, lineweight display, document and active viewport view (see [`ensure_list`]).
type ListKey = (u64, u64, i32, i32, bool, bool, usize, u64);

/// Whether two list keys are for the same drawing content (they may differ only in the view).
fn same_content(a: &ListKey, b: &ListKey) -> bool {
    (a.0, a.1, a.4, a.5, a.6, a.7) == (b.0, b.1, b.4, b.5, b.6, b.7)
}

#[derive(Default)]
pub struct CanvasState {
    pub list: Option<DisplayList>,
    key: ListKey,
    /// The area `list` was built for (hatch patterns exist only inside it); a view leaving it
    /// rebuilds the list.
    window: Option<Bounds2>,
    /// A rebuild for a new view running on a worker thread (see [`rebuild`]); `list` is drawn
    /// meanwhile.
    job: Option<rebuild::Job>,
    /// A background view rebuild that failed. Avoid spawning it every frame
    /// until the view key changes; keep the previous list visible meantime.
    failed_rebuild_key: Option<ListKey>,
    /// Bumped whenever `list` is rebuilt (identifies it for [`PrimIndex`]).
    list_gen: u64,
    prim_index: PrimIndex,
    /// Set when the app runs on wgpu: entities are drawn by [`crate::gpu`]; otherwise on the CPU.
    pub gpu: Option<crate::gpu::GpuTarget>,
    /// The wgpu device, for background rebuilds to create their buffers (see [`rebuild`]).
    pub gpu_device: Option<egui_wgpu::wgpu::Device>,
    /// The mesh last handed to the GPU (see [`crate::gpu::CanvasCallback::key`]).
    mesh_key: Option<u64>,
    mesh_slot: crate::gpu::MeshSlot,
    mesh_origin: Vec2,
    /// The dragged objects of a MOVE/COPY/ROTATE/SCALE preview, uploaded once per drag (see
    /// [`draw_drag_gpu`]).
    drag_key: Option<u64>,
    drag_slot: crate::gpu::MeshSlot,
    drag_origin: Vec2,
    /// The selection highlight on the GPU (see [`draw_highlight_gpu`]).
    hl_key: Option<u64>,
    hl_slot: crate::gpu::MeshSlot,
    hl_origin: Vec2,
    /// Selected infinite lines (clipped and drawn on the CPU each frame).
    hl_infinite: Vec<usize>,
    /// The selection and hover highlights inside the active MSPACE viewport (see [`VpHighlight`]).
    vp_sel: VpHighlight,
    vp_hover: VpHighlight,
    /// Model space seen through the active MSPACE viewport (see [`VpContent`]).
    vp: VpContent,
    /// Indices of infinite-line primitives in `list` (drawn on the CPU in GPU mode).
    infinite: Vec<usize>,
    /// Milliseconds spent converting the display list to vertex data (last rebuild).
    pub mesh_ms: f64,
    pub rect: Option<Rect>,
    pub xf: Option<Xf>,
    pub hover: Option<Handle>,
    pub snap: Option<SnapHit>,
    /// Effective cursor point after snaps/ortho/polar.
    pub cursor: Option<Vec2>,
    /// The polar or object snap tracking path(s) the cursor is on.
    pub track: Option<snap::tracking::Track>,
    pan_last: Option<Pos2>,
    /// True when the latest scroll came from a trackpad (or other precise-scrolling device):
    /// scrolling then pans and pinch zooms. A notched mouse wheel clears it so the wheel zooms.
    scroll_pans: bool,
    /// Hot grips (object, grip index): the ones Shift+clicked while idle, then the ones the
    /// running GRIP edit works on (the engine's grip modes take them from there).
    pub hot_grips: Vec<(Handle, usize)>,
    /// Grip dimensional input: the value fields of the grip being stretched.
    pub grip_dyn: crate::grip_dyn::GripDyn,
    /// A GRIP edit was running last frame (its grips go cold when it ends).
    grip_editing: bool,
    /// The grip whose menu (Stretch, Add Vertex, Lengthen…) a right-click opened.
    grip_menu: Option<(Handle, usize)>,
    /// Dynamic block grips of the selection (cached), the one being dragged, and the list grip
    /// whose choices are open (and whether it opened this frame).
    pub(crate) dyn_grips: crate::dyngrips::Cache,
    pub(crate) dyn_hot: Option<(Handle, i32, cadcraft_engine::doc::dynblock::GripPart)>,
    pub(crate) dyn_menu: Option<(Handle, cadcraft_engine::doc::dynblock::DynGrip)>,
    pub(crate) dyn_menu_fresh: bool,
    /// Associative-array parameter grips and the compact editor opened from one.
    pub(crate) array_grips: crate::arraygrips::Cache,
    pub(crate) array_grip_editor: Option<crate::arraygrips::Editor>,
    /// Zoom to extents once the canvas size is known (after opening a drawing).
    pub zoom_pending: bool,
    pub build_ms: f64,
    pub draw_ms: f64,
    hover_at: Option<Pos2>,
    /// Constraint glyphs for the parametric overlay (cached per drawing revision).
    pub param: crate::parametric::Cache,
    /// A selection window opened by a press-and-drag, to close where the button is let go.
    drag_window: bool,
    /// The right-click that opened the shortcut menu held Shift or Ctrl: it is the object snap
    /// menu ([`crate::context_menu`]).
    pub snap_menu: bool,
    /// Tab was pressed over a snap marker: show the next candidate ([`SnapCycle`]).
    pub snap_tab: bool,
    pub snap_cycle: Option<SnapCycle>,
    /// The selection cycling list a click over overlapping objects opened (SELECTIONCYCLING 2).
    cycle: Option<CycleList>,
    /// Selection cycling's badge shows beside the cursor, and where it was last worked out.
    cycle_badge: bool,
    cycle_badge_at: Option<Pos2>,
    /// Shift+Space stepping the highlight through the objects under the cursor (selection
    /// cycling's temporary override): where it started, the objects, which one is highlighted.
    hover_cycle: Option<(Pos2, Vec<Handle>, usize)>,
    /// What the cycling list and the Shift+Space highlight were made for (see [`CycleKey`]).
    cycle_key: Option<CycleKey>,
}

/// The command, prompt, drawing (and its revision) and view that the objects under the cursor
/// were listed for: when any of them changes, the list and the Shift+Space highlight are stale.
type CycleKey = (Option<String>, Option<String>, u64, u64, Vec2, f64);

impl CanvasState {
    /// The active MSPACE viewport's contents when they are drawn apart from the layout's list
    /// (see [`VpContent`]): where they are on paper at the viewport's current view, and how many
    /// times they have been built.
    pub fn viewport_content(&self) -> Option<(Bounds2, u64)> {
        self.vp.key.and(self.vp.paper_bounds()).map(|b| (b, self.vp.builds))
    }

    /// The rows of the open selection cycling list (screen rectangle, object), top to bottom.
    pub fn cycle_rows(&self) -> Vec<(Rect, Handle)> {
        self.cycle.as_ref().map(|c| c.rows.iter().copied().zip(c.items.iter().map(|(h, _, _)| *h)).collect()).unwrap_or_default()
    }

    /// Whether a rebuild for the current view is still running in the background (the canvas
    /// shows the previous list until it lands).
    pub fn building(&self) -> bool {
        self.job.is_some()
    }

    /// Put a finished background build in place, with its GPU vertex data when it has some.
    fn install(&mut self, key: ListKey, window: Option<Bounds2>, built: rebuild::Built) {
        self.list_gen = self.list_gen.wrapping_add(1);
        if let Some(m) = built.mesh {
            if let Ok(mut slot) = self.mesh_slot.lock() {
                *slot = Some(m.data);
            }
            self.mesh_key = Some(list_mesh_key(&key, self.list_gen, m.origin, m.bg, m.lwdisplay));
            self.mesh_origin = m.origin;
            self.mesh_ms = m.ms;
            self.infinite = m.infinite;
        }
        self.list = Some(built.list);
        self.failed_rebuild_key = None;
        self.key = key;
        self.window = window;
        self.build_ms = built.build_ms;
    }
}

/// Tab cycling through the candidate snaps under the cursor: where it started and which
/// candidate shows. Moving the cursor out of the aperture ends it.
#[derive(Clone, Copy, Debug)]
pub struct SnapCycle {
    anchor: Vec2,
    index: usize,
}

/// Do not reinterpret the last click as an idle selection after an earlier
/// release in the same UI frame has finished the interactive CAD command.
fn may_dispatch_last_click(clicked: bool, finished_earlier: bool, menu_or_dialog: bool) -> bool {
    clicked && !finished_earlier && !menu_or_dialog
}

#[cfg(test)]
mod point_batch_tests {
    use super::{may_dispatch_last_click, Xf};
    use cadcraft_geom::Vec2;
    use egui::{pos2, Rect};
    #[test]
    fn batched_releases_use_distinct_world_coordinates() {
        let xf = Xf { rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(100.0, 100.0)), center: Vec2::ZERO, scale: 10.0 };
        let a = xf.to_world(pos2(30.0, 50.0));
        let b = xf.to_world(pos2(70.0, 50.0));
        assert!((a.x - b.x).abs() > 1.0);
        assert_eq!(a.y, b.y);
    }
    #[test]
    fn completed_command_does_not_turn_final_release_into_selection() {
        assert!(may_dispatch_last_click(true, false, false));
        assert!(!may_dispatch_last_click(true, true, false));
        assert!(!may_dispatch_last_click(true, false, true));
        assert!(!may_dispatch_last_click(false, false, false));
    }
}

/// Collect actual complete primary-button gestures, matching each release to its
/// own press. A toolbar/menu press followed by a release over the canvas must
/// never become a CAD point. For a press from an earlier frame use egui's
/// press-origin fallback, but do not reuse that origin for subsequent presses.
fn primary_canvas_releases(events: &[egui::Event], rect: Rect, earlier_press_on_canvas: bool) -> Vec<Pos2> {
    let mut pressed_here = None;
    let mut carried_press = earlier_press_on_canvas;
    let mut releases = Vec::new();
    for event in events {
        if let egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, .. } = event {
            if *pressed {
                // An explicit press replaces any fallback carried from the last frame.
                carried_press = false;
                pressed_here = Some(rect.contains(*pos));
            } else {
                let was_pressed_here = pressed_here.take().unwrap_or(carried_press);
                carried_press = false;
                if was_pressed_here && rect.contains(*pos) {
                    releases.push(*pos);
                }
            }
        }
    }
    releases
}

#[cfg(test)]
mod primary_gesture_tests {
    use super::primary_canvas_releases;
    use egui::{pos2, vec2, Event, Modifiers, PointerButton, Rect};

    fn primary(pos: egui::Pos2, pressed: bool) -> Event {
        Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
    }

    #[test]
    fn accepts_two_fast_clicks_at_distinct_canvas_positions() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let events = [primary(pos2(15.0, 20.0), true), primary(pos2(15.0, 20.0), false), primary(pos2(75.0, 80.0), true), primary(pos2(75.0, 80.0), false)];
        assert_eq!(primary_canvas_releases(&events, rect, false), vec![pos2(15.0, 20.0), pos2(75.0, 80.0)]);
    }

    #[test]
    fn excludes_toolbar_press_and_canvas_release_even_when_followed_by_valid_click() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let events = [primary(pos2(-20.0, 20.0), true), primary(pos2(20.0, 20.0), false), primary(pos2(70.0, 70.0), true), primary(pos2(70.0, 70.0), false)];
        assert_eq!(primary_canvas_releases(&events, rect, true), vec![pos2(70.0, 70.0)]);
    }

    #[test]
    fn carried_press_cannot_authorize_multiple_releases() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let events = [primary(pos2(20.0, 20.0), false), primary(pos2(50.0, 50.0), false)];
        assert_eq!(primary_canvas_releases(&events, rect, true), vec![pos2(20.0, 20.0)]);
    }

    #[test]
    fn explicit_press_outside_canvas_overrides_carried_press() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let events = [primary(pos2(-20.0, 20.0), true), primary(pos2(30.0, 30.0), false)];
        assert!(primary_canvas_releases(&events, rect, true).is_empty());
    }

    #[test]
    fn switching_tiles_requires_press_and_release_in_same_destination_tile() {
        let left = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let right = Rect::from_min_size(pos2(100.0, 0.0), vec2(100.0, 100.0));
        let cross_tile = [primary(pos2(40.0, 30.0), true), primary(pos2(150.0, 30.0), false)];
        assert!(primary_canvas_releases(&cross_tile, right, false).is_empty());
        let right_tile = [primary(pos2(145.0, 30.0), true), primary(pos2(145.0, 30.0), false)];
        assert!(primary_canvas_releases(&right_tile, left, false).is_empty());
        assert_eq!(primary_canvas_releases(&right_tile, right, false), vec![pos2(145.0, 30.0)]);
    }

    #[test]
    fn toolbar_press_cannot_activate_an_inactive_tile() {
        let target = Rect::from_min_size(pos2(100.0, 0.0), vec2(100.0, 100.0));
        let events = [primary(pos2(15.0, 20.0), true), primary(pos2(150.0, 20.0), false)];
        assert!(primary_canvas_releases(&events, target, true).is_empty());
    }

    #[test]
    fn prior_frame_press_origin_is_only_used_for_unpaired_release() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        assert_eq!(primary_canvas_releases(&[primary(pos2(25.0, 25.0), false)], rect, true), vec![pos2(25.0, 25.0)]);
        assert!(primary_canvas_releases(&[primary(pos2(25.0, 25.0), false)], rect, false).is_empty());
    }
}

/// Filter releases *before* batching point prompts. A click over a canvas
/// overlay is never a drawing point, even when several events share a frame.
fn eligible_point_releases(releases: &[Pos2], mut eligible: impl FnMut(Pos2) -> bool) -> Vec<Pos2> {
    releases.iter().copied().filter(|pos| eligible(*pos)).collect()
}

#[cfg(test)]
mod eligible_point_release_tests {
    use super::eligible_point_releases;
    use egui::pos2;

    #[test]
    fn preserves_the_order_of_valid_clicks_while_excluding_menu_release() {
        let clicks = [pos2(12.0, 12.0), pos2(55.0, 55.0), pos2(90.0, 90.0)];
        let filtered = eligible_point_releases(&clicks, |pos| pos != pos2(55.0, 55.0));
        assert_eq!(filtered, vec![pos2(12.0, 12.0), pos2(90.0, 90.0)]);
    }

    #[test]
    fn an_overlay_only_burst_cannot_submit_a_cad_point() {
        let filtered = eligible_point_releases(&[pos2(50.0, 50.0)], |_| false);
        assert!(filtered.is_empty());
    }
}

/// After a fast click burst, pointer.hover_pos may already describe the
/// next frame or another UI surface. The last physical release is the actual
/// CAD point, not that later cursor position.
fn point_input_position(prompting_for_point: bool, last_canvas_release: Option<Pos2>, hover: Option<Pos2>) -> Option<Pos2> {
    if prompting_for_point { last_canvas_release.or(hover) } else { hover }
}

#[cfg(test)]
mod point_event_position_tests {
    use super::point_input_position;
    use egui::pos2;

    #[test]
    fn uses_last_physical_release_not_later_pointer_hover() {
        assert_eq!(point_input_position(true, Some(pos2(20.0, 30.0)), Some(pos2(160.0, 180.0))), Some(pos2(20.0, 30.0)));
    }

    #[test]
    fn idle_selection_and_keyboard_prompts_keep_the_live_pointer() {
        assert_eq!(point_input_position(false, Some(pos2(20.0, 30.0)), Some(pos2(160.0, 180.0))), Some(pos2(160.0, 180.0)));
        assert_eq!(point_input_position(true, None, Some(pos2(30.0, 40.0))), Some(pos2(30.0, 40.0)));
    }
}

/// A physical release must not be dispatched as a second selection click when
/// it finishes an owned drag, or when ESC/focus loss invalidates the gesture.
fn may_dispatch_canvas_click(drag_active: bool, drag_stopped: bool, cancelled: bool) -> bool {
    !drag_active && !drag_stopped && !cancelled
}

#[cfg(test)]
mod click_drag_arbitration_tests {
    use super::may_dispatch_canvas_click;

    #[test]
    fn click_click_selection_remains_available_without_a_drag() {
        assert!(may_dispatch_canvas_click(false, false, false));
    }

    #[test]
    fn ending_a_drag_cannot_also_dispatch_an_idle_click() {
        assert!(!may_dispatch_canvas_click(true, true, false));
        assert!(!may_dispatch_canvas_click(false, true, false));
    }

    #[test]
    fn escape_and_focus_loss_cancel_click_dispatch() {
        assert!(!may_dispatch_canvas_click(false, false, true));
    }
}

/// A physical release belongs to the canvas only when its press began there.
fn raw_release_for_point_prompt(started_on_canvas: bool, released_on_canvas: bool) -> bool {
    started_on_canvas && released_on_canvas
}

#[cfg(test)]
mod point_release_tests {
    use super::raw_release_for_point_prompt;
    #[test]
    fn first_release_on_canvas_is_accepted_but_toolbar_release_is_not() {
        assert!(raw_release_for_point_prompt(true, true));
        assert!(!raw_release_for_point_prompt(false, true));
        assert!(!raw_release_for_point_prompt(true, false));
        assert!(!raw_release_for_point_prompt(false, false));
    }
}

/// A lost pointer release must never leave a selection window active for the next click.
fn should_cancel_drag_window(active: bool, escape: bool, focused: bool, released: bool, release_inside: bool) -> bool {
    active && (escape || !focused || (released && !release_inside))
}

#[cfg(test)]
mod stuck_drag_tests {
    use super::should_cancel_drag_window;
    #[test]
    fn escape_focus_loss_and_outside_release_cancel_drag() {
        assert!(should_cancel_drag_window(true, true, true, false, true));
        assert!(should_cancel_drag_window(true, false, false, false, true));
        assert!(should_cancel_drag_window(true, false, true, true, false));
        assert!(!should_cancel_drag_window(true, false, true, true, true));
        assert!(!should_cancel_drag_window(false, true, false, true, false));
    }
}

/// Draw the drawing area and handle its input (zoom and pan, picking, the crosshair) for this
/// frame.
///
/// It also owns the cursor-visibility contract: the OS cursor is hidden only while the pointer is
/// over the drawing area *and* the canvas is the topmost layer under it, so the drawn crosshair
/// replaces the pointer over the drawing alone. A menu bar, an open menu (including its frame, not
/// just its items), a tooltip or a floating palette above the canvas keeps the normal OS cursor
/// visible (#1701), and so do the menus painted on the canvas itself (see [`over_canvas_menu`]).
pub fn show(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = Tokens::get();
    let full = ui.available_rect_before_wrap();
    // Tiled model viewports (VPORTS): the current tile is the drawing area below; `draw_tiles`
    // draws the others.
    let tiles = tile_rects(app, full);
    let rect = tiles.iter().find(|(_, tile)| tile.active).map_or(full, |(r, _)| *r);
    let resp = ui.interact(full, ui.scope_id().with("canvas"), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    app.canvas.rect = Some(rect);
    app.session.viewport_px = (f64::from(rect.width()).max(1.0), f64::from(rect.height()).max(1.0));
    if app.canvas.zoom_pending {
        app.canvas.zoom_pending = false;
        let _ = app.session.zoom_extents();
    }
    let Ok(st) = app.session.state() else { return };
    let view = st.view();
    let scale = f64::from(rect.height()) / view.height.max(1e-12);
    let xf = Xf { rect, center: view.center, scale };
    app.canvas.xf = Some(xf);

    // ---------- input ----------
    let (hover_pos, scroll, mods, middle_down, pressed_primary, pressed_secondary, dbl_middle, dbl_primary) = ui.input(|i| {
        (
            i.pointer.hover_pos(),
            i.smooth_scroll_delta,
            i.modifiers,
            i.pointer.middle_down(),
            i.pointer.primary_clicked(),
            i.pointer.secondary_clicked(),
            i.pointer.button_double_clicked(egui::PointerButton::Middle),
            i.pointer.button_double_clicked(egui::PointerButton::Primary),
        )
    });
    // The drawing area alone: a menu popup's frame (between its items and its border), a tooltip
    // or a floating palette lives on a layer above the canvas, where `hovered()` alone still
    // counts the canvas. Over those the OS cursor stays visible and the crosshair stays put, as
    // over the menu bar (#1701).
    // Use actual pointer position and topmost layer: hovered() may be false on the first press while egui transfers focus.
    let inside = hover_pos.is_some_and(|p| rect.contains(p) && ui.ctx().layer_id_at(p) == Some(ui.layer_id()));
    // The menus the canvas paints itself (selection cycling's rows, Dynamic Input's options) are
    // menus too, but they share the canvas's layer: `layer_id_at` reports the drawing under them,
    // so only their rectangles tell them apart from it ([`over_canvas_menu`]). They are excluded
    // from the cursor contract below, though not from the canvas's input: a click on a row is
    // taken by the row.
    let on_menu = hover_pos.is_some_and(|p| over_canvas_menu(app, p));
    // Whether Ctrl is held right now. Written every frame (not just while a command prompts), so
    // previews that read `Session::ctrl_down` always see the current state. The engine
    // deliberately does not carry modifiers through `Input`, which is UI-agnostic (see PR
    // #407's Limits note: modifiers used to be dropped on the floor at this point).
    app.session.ctrl_down = mods.ctrl;
    // Which device is scrolling? On macOS, trackpads (and Magic Mouse) report precise point
    // deltas; notched mouse wheels report lines. The answer is sticky because egui spreads a wheel
    // notch over several frames. Elsewhere (browsers, Wayland smooth-scroll mice) ordinary wheels
    // also report points, so the wheel keeps zooming there.
    ui.input(|i| {
        for e in &i.raw.events {
            if let egui::Event::MouseWheel { unit, .. } = e {
                app.canvas.scroll_pans = cfg!(target_os = "macos") && *unit == egui::MouseWheelUnit::Point;
            }
        }
    });
    if inside && app.canvas.scroll_pans && scroll != egui::Vec2::ZERO {
        // Two-finger swipe pans: the drawing follows the fingers, like a middle-drag.
        // Pinch (and Cmd+scroll) still zoom via `zoom_delta` below.
        if let Ok(st) = app.session.state_mut() {
            let v = st.view();
            st.set_view_quiet(cadcraft_engine::View {
                center: v.center - Vec2::new(f64::from(scroll.x) / scale, -f64::from(scroll.y) / scale),
                height: v.height,
            });
        }
    }
    let opts = ui.ctx().options(|o| o.input_options);
    let wheel = ui.input(|i| wheel_zoom_points(&i.raw.events, &opts, rect.height()));
    if inside
        && !app.canvas.scroll_pans
        && wheel != 0.0
        && let Some(hp) = hover_pos
    {
        // Zoom with the mouse wheel about the cursor, each notch at once.
        let f = app.session.wheel_zoom_factor(f64::from(wheel));
        let about = xf.to_world(hp);
        let _ = app.session.zoom_about(f, about);
    }
    let zoom_pinch = ui.input(|i| crate::animation::zoom_delta(i, &opts, app.shows_animations()));
    if inside
        && (zoom_pinch - 1.0).abs() > 1e-4
        && let Some(hp) = hover_pos
    {
        let _ = app.session.zoom_about(f64::from(zoom_pinch), xf.to_world(hp));
    }
    // Pan with the middle button (or Shift+right-drag / two-finger drag on trackpads is scroll).
    let middle_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Middle));
    let focused = ui.input(|i| i.focused);
    if middle_down
        && focused
        && (app.canvas.pan_last.is_some() || (middle_pressed && inside))
        && let Some(hp) = hover_pos
    {
        if let Some(last) = app.canvas.pan_last {
            let d = hp - last;
            if let Ok(st) = app.session.state_mut() {
                let v = st.view();
                st.set_view_quiet(cadcraft_engine::View {
                    center: v.center - Vec2::new(f64::from(d.x) / scale, -f64::from(d.y) / scale),
                    height: v.height,
                });
            }
        }
        app.canvas.pan_last = Some(hp);
    } else {
        app.canvas.pan_last = None;
    }
    if dbl_middle && inside {
        let _ = app.session.zoom_extents();
    }
    // ZOOM Realtime: dragging up zooms in, down zooms out.
    if resp.dragged_by(egui::PointerButton::Primary) && app.session.realtime_zoom_active() {
        let _ = app.session.realtime_zoom(f64::from(resp.drag_delta().y));
    }
    // Switching native CAD viewports must be based on a real click inside the
    // target viewport, not a global toolbar click or a drag from another tile.
    // The same physical press/release matching is used for drawing points.
    if pressed_primary
        && let Some(hp) = hover_pos
        && !rect.contains(hp)
        && ui.ctx().layer_id_at(hp) == Some(ui.layer_id())
        && !over_canvas_menu(app, hp)
        && let Some((i, (target_rect, _))) = tiles.iter().enumerate().find(|(_, (r, _))| r.contains(hp))
    {
        let clicked_target = ui.input(|input| {
            let earlier_press = input.pointer.press_origin().is_some_and(|origin| target_rect.contains(origin));
            !primary_canvas_releases(&input.events, *target_rect, earlier_press).is_empty()
        });
        if clicked_target {
            if let Err(e) = app.session.click_tile(i) {
                app.session.echo(e.to_string());
            }
            ui.ctx().request_repaint();
            return;
        }
    }
    // Re-read the view after navigation.
    let view = app.session.state().map(|s| s.view()).unwrap_or(view);
    let scale = f64::from(rect.height()) / view.height.max(1e-12);
    let xf = Xf { rect, center: view.center, scale };
    app.canvas.xf = Some(xf);
    // The paper (display) transform; equal to `xf` except inside an MSPACE viewport.
    let (xf_paper, active_vp) = match app.session.state() {
        Ok(st) => {
            let pv = st.paper_view();
            let xp = Xf { rect, center: pv.center, scale: f64::from(rect.height()) / pv.height.max(1e-12) };
            (if st.mspace.is_some() { xp } else { xf }, st.active_viewport().map(|(_, v)| v))
        }
        Err(_) => (xf, None),
    };
    // A clipped active viewport's outline (picks and its heavy border follow it).
    let active_clip = app.session.state().ok().and_then(|st| st.active_viewport().and_then(|(h, v)| cadcraft_render::viewport_clip(&st.doc, h, &v)));
    // Double-click a viewport to work inside it; double-click the sheet outside it to return.
    if inside
        && dbl_primary
        && !app.session.prompting()
        && matches!(app.session.layout_space(), cadcraft_doc::Space::Paper(_))
        && let Some(hp) = hover_pos
    {
        let pp = xf_paper.to_world(hp);
        let in_rect = |v: &cadcraft_doc::Viewport| (pp.x - v.center.x).abs() <= v.width / 2.0 && (pp.y - v.center.y).abs() <= v.height / 2.0;
        let in_active = active_vp.as_ref().is_some_and(|v| active_clip.as_ref().map_or(in_rect(v), |c| c.contains(pp)));
        if !in_active {
            app.canvas.hot_grips.clear();
            app.session.pending_window = None;
            app.canvas.drag_window = false;
            if app.run("mspace", serde_json::json!({"at": [pp.x, pp.y]})).is_err() && active_vp.is_some() {
                let _ = app.run("pspace", serde_json::json!({}));
            }
            app.session.pending_window = None;
            app.canvas.drag_window = false;
            app.canvas.list = None;
            return;
        }
    }
    // Double-click text to edit it, as in AutoCAD: TEXTEDIT with that object already picked.
    if inside
        && dbl_primary
        && app.session.running.is_none()
        && let Some(hp) = hover_pos
        && edit_text_at(app, xf.to_world(hp))
    {
        return;
    }

    // Double-click a hatch (no command running) to edit it in the Hatch Edit dialog. The click is
    // used up: it must not also start a grip edit on the hatch's centre grip below.
    let mut hatch_dialog = false;
    if inside
        && dbl_primary
        && app.session.running.is_none()
        && let Some(hp) = hover_pos
        && let Some(h) = hatch_at(app, xf.to_world(hp), scale)
    {
        app.session.set_selection(vec![h]);
        crate::hatchdlg::open_edit(app);
        hatch_dialog = true;
    }

    // While a command asks for a point, every click places one, however fast the drafting (#815).
    // egui drops a click whose button comes up a few pixels from where it went down (a drag to it),
    // and eframe hands a slow frame's clicks over together, which egui reports as one.
    let point_prompt = app.session.current_prompt().is_some_and(|p| cadcraft_engine::pointmod::accepts(&p))
        && app.session.pending_window.is_none()
        && !app.session.realtime_zoom_active();
    // A point prompt accepts a physical release even when egui did not classify
    // the gesture as a click (small movement or focus transfer from the toolbar).
    // Require the press to originate on this canvas to avoid stealing a menu click.
    let releases = ui.input(|i| {
        let earlier_press_on_canvas = i.pointer.press_origin().is_some_and(|origin| rect.contains(origin));
        primary_canvas_releases(&i.events, rect, earlier_press_on_canvas)
    });
    // Only a coordinate command needs the extra release list. Avoid an
    // allocation and repeated UI layer probes on idle hover/zoom/selection.
    let point_releases = if point_prompt {
        eligible_point_releases(&releases, |pos| {
            rect.contains(pos) && ui.ctx().layer_id_at(pos) == Some(ui.layer_id()) && !over_canvas_menu(app, pos)
        })
    } else {
        Vec::new()
    };
    let valid_last_release = point_releases.last().copied();
    let point_hover_pos = point_input_position(point_prompt, valid_last_release, hover_pos);
    let point_inside = if point_prompt && valid_last_release.is_some() { true } else { inside };
    let point_on_menu = if point_prompt && valid_last_release.is_some() { false } else { on_menu };
    let raw_canvas_release = raw_release_for_point_prompt(true, valid_last_release.is_some());
    // During a point prompt, a global egui click can belong to a toolbar or
    // palette. Only matched canvas gestures or the canvas response may place
    // geometry. Keep normal click handling for Dynamic Input option menus.
    let pressed_primary = if point_prompt && app.cmd.options.is_none() {
        raw_canvas_release || resp.clicked_by(egui::PointerButton::Primary)
    } else {
        pressed_primary || (point_prompt && raw_canvas_release)
    };
    let mut command_finished_in_batch = false;
    if point_prompt && point_inside && !point_on_menu && pressed_primary && !middle_down && app.cmd.options.is_none() && app.cmd.buffer.is_empty() {
        // The last release is dispatched by the regular cursor path below. Earlier
        // releases must use their own geometry rather than the final cursor's snap:
        // reusing the last snap would place multiple points at the same coordinate.
        // A command may finish on an earlier event (for example a two-point
        // operation). Remaining clicks must not start an accidental selection.
        // The main path will handle only the last release if the prompt is still active.
        if let Some((_, earlier)) = point_releases.split_last() {
            for pos in earlier {
                if !app.session.current_prompt().is_some_and(|prompt| cadcraft_engine::pointmod::accepts(&prompt)) {
                    command_finished_in_batch = true;
                    break;
                }
                let w = xf.to_world(*pos);
                if let Err(e) = app.session.input(Input::Point(w)) {
                    app.session.echo(e.to_string());
                }
                // Moving from a point prompt to another input type is also an end to
                // the point batch: do not feed a coordinate into a Yes/No or selection prompt.
                if !app.session.current_prompt().is_some_and(|prompt| cadcraft_engine::pointmod::accepts(&prompt)) {
                    command_finished_in_batch = true;
                    break;
                }
            }
        }
    }

    let raw_world = point_hover_pos.filter(|_| point_inside).map(|p| xf.to_world(p));
    let dyn_frame = crate::cmdline::dyn_frame(app);
    if let Some(w) = raw_world {
        let mut eff = effective_point(app, w, &xf, ui.ctx());
        // A locked Dynamic Input value (`40,` or `40<` typed) holds the cursor and overrides snaps.
        if let Some(f) = dyn_frame
            && let Some(e) = crate::dyninput::parse(&app.cmd.buffer, &f)
            && e.first_locked().is_some()
        {
            eff = crate::dyninput::constrain(&e, &f, eff, app.cmd.second_locked());
            app.canvas.snap = None;
            app.canvas.track = None;
        }
        // Grip dimensional input: typed lengths and angles hold the grip and override snaps.
        let grip_held = crate::grip_dyn::constrain(app, eff);
        if let Some(q) = grip_held {
            eff = q;
            app.canvas.snap = None;
            app.canvas.track = None;
        }
        app.canvas.cursor = Some(eff);
        app.session.cursor = eff;
        app.session.cursor_deferred = app.canvas.snap.and_then(|h| h.deferred);
        // Typed or menu snap overrides (END, NON…) and Tab cycling; a locked Dynamic Input value wins.
        if dyn_frame.and_then(|f| crate::dyninput::parse(&app.cmd.buffer, &f)).is_none_or(|e| e.first_locked().is_none()) {
            override_cursor(app, w, &xf);
        }
        // Typed grip dimensions win over snap overrides too.
        if let Some(q) = grip_held {
            app.canvas.cursor = Some(q);
            app.session.cursor = q;
            app.canvas.snap = None;
            app.session.cursor_deferred = None;
        }
    } else {
        app.canvas.cursor = None;
        app.canvas.snap = None;
        app.session.cursor_deferred = None;
    }

    // A click on the Dynamic Input options menu picks the keyword; elsewhere it closes the menu.
    let mut picked = false;
    if pressed_primary && app.cmd.options.is_some() {
        match hover_pos.and_then(|h| app.cmd.option_at(h)) {
            Some(k) => {
                crate::cmdline::pick_option(app, k);
                picked = true;
            }
            None => app.cmd.options = None,
        }
    }
    // A double-click with no command running (on a layout viewport, say) leaves no selection
    // window behind; while a command asks for points, every click still counts (#815).
    let idle_dbl = dbl_primary && app.session.running.is_none();
    if idle_dbl {
        app.session.pending_window = None;
        app.canvas.drag_window = false;
    }
    // Never consume a click on an egui menu or an overlay owned by the canvas.
    let pressed_primary = may_dispatch_last_click(pressed_primary, command_finished_in_batch, picked || hatch_dialog || idle_dbl || point_on_menu);
    let key: CycleKey = (
        app.session.running.as_ref().map(|r| r.id.clone()),
        app.session.current_prompt().map(|p| p.message),
        app.session.state().map_or(0, |st| st.uid),
        app.session.state().map_or(0, |st| st.revision),
        xf.center,
        xf.scale,
    );
    if app.canvas.cycle_key.as_ref() != Some(&key) {
        app.canvas.cycle = None;
        app.canvas.hover_cycle = None;
        app.canvas.cycle_key = Some(key);
    }
    // A drag-owned selection must end as a drag, not a second idle click.
    // egui can report both clicked and drag_stopped in the same slow frame.
    // Escape/focus loss wins over all pending point and selection dispatch.
    let cancel_canvas_interaction = ui.input(|i| i.key_pressed(egui::Key::Escape) || !i.focused);
    let drag_window_at_frame_start = app.canvas.drag_window;
    let point_drag_stopped = !point_prompt && resp.drag_stopped_by(egui::PointerButton::Primary);
    let click_is_not_selection_drag = may_dispatch_canvas_click(drag_window_at_frame_start, point_drag_stopped, cancel_canvas_interaction);
    let cycled = pressed_primary && click_is_not_selection_drag && !middle_down && cycle_click(app, &xf, hover_pos, raw_world.filter(|_| inside), mods.shift);
    // Clicks. A paired point release remains valid even if the pointer has
    // moved off-canvas before egui paints this (slow) frame.
    if point_inside
        && pressed_primary
        && click_is_not_selection_drag
        && !middle_down
        && !cycled
        && let Some(p) = app.canvas.cursor
    {
        if app.session.prompting() {
            // Ctrl held while picking the point of a grip edit makes copies (the Copy option).
            if mods.ctrl && app.session.running.as_ref().is_some_and(|r| r.id == "grip") {
                let _ = app.session.input(Input::Keyword("Copy".into()));
            }
            // Clicking places the point; a value typed into the Dynamic Input boxes is used up.
            if let Some(f) = dyn_frame
                && crate::dyninput::parse(&app.cmd.buffer, &f).is_some()
            {
                app.cmd.buffer.clear();
            }
            // Preserve a snap's viewport origin for DIST, and its deferred curve where needed.
            let result = match app.canvas.snap {
                Some(hit) => app.session.input_snapped(hit),
                None => app.session.input(Input::Point(p)),
            };
            if let Err(e) = result {
                app.session.echo(e.to_string());
            }
        } else if let Some(g) = crate::arraygrips::at(app, &xf, hover_pos) {
            crate::arraygrips::click(app, g);
        } else if mods.ctrl
            && crate::arraygrips::ctrl_remove_item(app, raw_world.unwrap_or(p), xf.scale)
        {
            // Ctrl+click on an associative-array item suppresses that item through the engine.
        } else if let Some(g) = crate::dyngrips::at(app, &xf, hover_pos) {
            crate::dyngrips::click(app, g);
        } else if let Some(g) = grip_at(app, &xf, hover_pos) {
            click_grip(app, g, mods.shift);
        } else if let Err(e) = app.session.idle_click(raw_world.unwrap_or(p), mods.shift) {
            app.session.echo(e.to_string());
        }
    }
    // Press-and-drag selection window (PICKDRAG = 2; click-click still works): a drag
    // that starts while objects are being selected opens a window where the button went down,
    // and letting go closes it. Drags on grips and drags while drawing are left alone.
    // Always disarm a stale drag when the pointer is released outside the canvas,
    // the window loses focus, or Escape cancels an interaction. Otherwise a later
    // click may accidentally complete an old crossing/window selection.
    let (primary_down, primary_released, escape, window_focused, release_pos) = ui.input(|i| (
        i.pointer.button_down(egui::PointerButton::Primary),
        i.pointer.button_released(egui::PointerButton::Primary),
        i.key_pressed(egui::Key::Escape),
        i.focused,
        i.pointer.interact_pos(),
    ));
    let release_is_canvas = release_pos.is_some_and(|p| rect.contains(p) && ui.ctx().layer_id_at(p) == Some(ui.layer_id()) && !over_canvas_menu(app, p));
    if should_cancel_drag_window(app.canvas.drag_window, escape, window_focused, primary_released, release_is_canvas) {
        app.canvas.drag_window = false;
        app.session.pending_window = None;
    }
    if !app.session.settings.pickdrag && app.canvas.drag_window {
        // PICKDRAG was disabled mid-gesture: discard only the drag-owned pending
        // window, not a click-click window initiated by another interaction mode.
        app.canvas.drag_window = false;
        app.session.pending_window = None;
    }
    if app.session.settings.pickdrag
        && !point_prompt
        && !escape
        && window_focused
        && !on_menu
        && resp.drag_started_by(egui::PointerButton::Primary)
        && !middle_down
        && app.canvas.cycle.is_none()
        && app.session.pending_window.is_none()
        && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        && rect.contains(origin)
        && ui.ctx().layer_id_at(origin) == Some(ui.layer_id())
        && !over_canvas_menu(app, origin)
        && grip_at(app, &xf, Some(origin)).is_none()
        && crate::arraygrips::at(app, &xf, Some(origin)).is_none()
        && crate::dyngrips::at(app, &xf, Some(origin)).is_none()
    {
        app.canvas.drag_window = app.session.begin_window(xf.to_world(origin));
    }
    if app.canvas.drag_window && (resp.drag_stopped_by(egui::PointerButton::Primary) || (primary_released && !primary_down)) {
        app.canvas.drag_window = false;
        // A release over a palette or outside the drawing cannot finish a CAD
        // selection window. Never leave its pending first corner alive.
        if release_is_canvas && app.session.pending_window.is_some() {
            if let Some(end) = ui.input(|i| i.pointer.interact_pos()) {
                let end = xf.to_world(end);
                let done = if app.session.prompting() { app.session.input(Input::Point(end)) } else { app.session.idle_click(end, mods.shift) };
                if let Err(e) = done {
                    app.session.echo(e.to_string());
                }
            } else {
                app.session.pending_window = None;
            }
        } else {
            app.session.pending_window = None;
        }
    }
    // Right-click acts as Enter while a command runs (the classic CAD default; held, it opens the
    // command menu); otherwise it opens the shortcut menu (Repeat, Clipboard, …).
    // Shift/Ctrl+right-click opens the object snap menu.
    let idle = !app.session.prompting();
    if inside && pressed_secondary {
        app.canvas.snap_menu = mods.shift || mods.ctrl;
    }
    if crate::context_menu::in_command(app, ui, inside, pressed_secondary) {
        let _ = app.session.input(Input::Enter);
    }
    // Right-click on a grip with options (polyline vertex, line end…) opens the grip menu.
    if inside && pressed_secondary && idle && !app.canvas.snap_menu {
        app.canvas.grip_menu = grip_at(app, &xf, hover_pos).filter(|g| !grip_options(app, *g).is_empty());
    }
    crate::arraygrips::menu(app, ui.ctx(), &xf);
    crate::dyngrips::menu(app, ui.ctx(), &xf);
    if idle && !app.canvas.snap_menu && app.canvas.grip_menu.is_some() {
        grip_menu(app, &resp);
    } else if idle || app.canvas.snap_menu {
        crate::context_menu::show(app, &resp);
    }

    // Hover highlight (throttled to cursor movement).
    // The cursor picks objects (pick box, rollover highlight): "Select objects" and single-object
    // prompts, or no command with PICKFIRST on; SELECTIONPREVIEW says in which of those it shows.
    let selecting = app.session.previews_selection();
    if inside && selecting && app.session.pending_window.is_none() {
        if hover_pos != app.canvas.hover_at
            && let (Some(w), Ok(st)) = (raw_world, app.session.state())
        {
            let filter = app.session.settings.previewfilter;
            app.canvas.hover = cadcraft_engine::select::pick(&st.doc, &st.edit_space(), w, app.session.settings.pickbox.max(1.0) * 1.5 / scale)
                .filter(|h| st.doc.entity(*h).is_none_or(|e| !cadcraft_engine::select::preview_filtered(&st.doc, e, filter)));
            app.canvas.hover_at = hover_pos;
        }
    } else {
        app.canvas.hover = None;
        app.canvas.hover_at = None;
    }
    // The open selection cycling list highlights the object of the row under the pointer.
    if app.canvas.cycle.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.canvas.cycle = None;
    }
    if let Some(c) = &app.canvas.cycle {
        app.canvas.hover = hover_pos.and_then(|hp| c.row_at(hp));
        app.canvas.hover_at = None;
    }
    shift_space_cycle(app, ui, hover_pos.filter(|_| inside), raw_world);
    if inside && hover_pos != app.canvas.cycle_badge_at {
        app.canvas.cycle_badge = raw_world.is_some_and(|w| app.session.cycling_badge(w));
        app.canvas.cycle_badge_at = hover_pos;
    } else if !inside {
        app.canvas.cycle_badge = false;
        app.canvas.cycle_badge_at = None;
    }

    // ---------- paint ----------
    painter.rect_filled(rect, 0.0, t.canvas);
    let t0 = crate::now_ms();
    let mut views = vec![(xf_paper.world_bounds(), 1.0 / xf_paper.scale)];
    views.extend(tiles.iter().filter(|(r, tile)| !tile.active && r.width() >= 1.0 && r.height() >= 1.0).map(|(r, tile)| {
        let xf = tile_xf(*r, tile);
        (xf.world_bounds(), 1.0 / xf.scale)
    }));
    ensure_list(app, &xf_paper, &views, ui.ctx());
    let sheet = app.canvas.list.as_ref().and_then(|l| l.sheet);
    let surround = Color32::from_rgb(0x50, 0x57, 0x63);
    let (bg, paper_rect) = match sheet {
        Some(sh) => {
            // Paper space: grey surround, the sheet with a shadow, the printable area dashed.
            painter.rect_filled(rect, 0.0, surround);
            let a = xf_paper.to_screen(Vec2::new(0.0, sh.size.y));
            let b = xf_paper.to_screen(Vec2::new(sh.size.x, 0.0));
            let paper = Rect::from_two_pos(a, b);
            painter.rect_filled(paper.translate(vec2(5.0, 5.0)), 0.0, Color32::from_black_alpha(110));
            painter.rect_filled(paper, 0.0, t.paper);
            let pa = Rect::from_two_pos(
                xf_paper.to_screen(Vec2::new(sh.printable.min.x, sh.printable.max.y)),
                xf_paper.to_screen(Vec2::new(sh.printable.max.x, sh.printable.min.y)),
            );
            let pts = [pa.left_top(), pa.right_top(), pa.right_bottom(), pa.left_bottom(), pa.left_top()];
            painter.extend(Shape::dashed_line(&pts, Stroke::new(1.0, Color32::from_gray(150)), 4.0, 4.0));
            (Rgb(t.paper.r(), t.paper.g(), t.paper.b()), Some(paper))
        }
        None => {
            blockedit::wash(app, &painter, rect);
            draw_grid(app, &painter, &xf);
            (Rgb(t.canvas.r(), t.canvas.g(), t.canvas.b()), None)
        }
    };
    let sel = app.session.selection();
    // How the object under the cursor and the selected ones are highlighted (PREVIEWEFFECT,
    // HIGHLIGHT, SELECTIONEFFECT); with HIGHLIGHT off only their grips show.
    let (preview_look, selection_look) = (Look::preview(&app.session.settings), Look::selection(&app.session.settings));
    // The active viewport's contents, drawn apart from the layout's list, under its own objects.
    // `masked`: its outline is in the stencil buffer this frame.
    let mut masked = false;
    if let Some(f) = ensure_vp_content(app, &xf_paper, ui.ctx()) {
        let gpu = app.canvas.gpu.is_some();
        let clipped = painter.with_clip_rect(f.on_screen.intersect(rect));
        draw_vp_content(&mut app.canvas.vp, gpu, &clipped, &f, &xf_paper, bg, app.session.settings.lwdisplay);
        masked = gpu && f.outline.is_some();
    }
    let active_tile_index = tiles.iter().position(|(_, tile)| tile.active).unwrap_or(0);
    let active_filter = app.tile_render_filters.get(active_tile_index).filter(|f| f.enabled).cloned();
    if let Some(filter) = active_filter {
        if let Some(list) = &app.canvas.list {
            draw_list_filtered(&painter, &xf_paper, list, bg, app.session.settings.lwdisplay, Some(&filter.primary_handles), 1.0);
            draw_list_filtered(&painter, &xf_paper, list, bg, app.session.settings.lwdisplay, Some(&filter.reference_handles), filter.reference_opacity);
        }
    } else if app.canvas.gpu.is_some() {
        draw_list_gpu(&mut app.canvas, &painter, &xf_paper, bg, app.session.settings.lwdisplay);
    } else if let Some(list) = &app.canvas.list {
        draw_list(&painter, &xf_paper, list, bg, app.session.settings.lwdisplay);
    }
    if let Some(vp) = &active_vp {
        // The active viewport: a heavy border, highlights in model units clipped to it.
        let r = Rect::from_two_pos(
            xf_paper.to_screen(Vec2::new(vp.center.x - vp.width / 2.0, vp.center.y + vp.height / 2.0)),
            xf_paper.to_screen(Vec2::new(vp.center.x + vp.width / 2.0, vp.center.y - vp.height / 2.0)),
        );
        let heavy = Stroke::new(3.0, Color32::from_rgb(0x2a, 0x2a, 0x2a));
        match &active_clip {
            Some(c) => {
                let mut pts: Vec<egui::Pos2> = c.polygon().iter().map(|q| xf_paper.to_screen(*q)).collect();
                pts.extend(pts.first().copied());
                painter.add(egui::Shape::line(pts, heavy));
            }
            None => {
                painter.rect_stroke(r, 0.0, heavy, egui::StrokeKind::Middle);
            }
        }
        if let Ok(st) = app.session.state() {
            // A clipped viewport's outline bounds (its outline is masked when `masked`).
            let bounds = active_clip.as_ref().map_or(r, |c| {
                let b = c.bounds();
                Rect::from_two_pos(xf_paper.to_screen(Vec2::new(b.min.x, b.max.y)), xf_paper.to_screen(Vec2::new(b.max.x, b.min.y)))
            });
            let clipped = painter.with_clip_rect(bounds.intersect(rect));
            let c = &mut app.canvas;
            if let Some(h) = c.hover
                && !sel.contains(&h)
            {
                c.vp_hover.update(st, &[h], scale);
                let list = &c.vp_hover.list;
                draw_highlight(&clipped, &xf, list, &(0..list.prims.len()).collect::<Vec<_>>(), t.hover, preview_look);
            }
            if let Some(look) = selection_look
                && !sel.is_empty()
            {
                c.vp_sel.update(st, &sel, scale);
                let list = &c.vp_sel.list;
                let all = || (0..list.prims.len()).collect::<Vec<_>>();
                if c.gpu.is_some() {
                    // Shares the GPU highlight layer with the layout's own selection; the high bit
                    // keeps this list's generations apart from the layout list's.
                    let list_id = c.vp_sel.list_gen | 1 << 63;
                    let stencil = if masked { crate::gpu::Stencil::Inside } else { crate::gpu::Stencil::Off };
                    let hl = (&mut c.hl_key, &c.hl_slot, &mut c.hl_origin, &mut c.hl_infinite);
                    draw_highlight_gpu(&clipped, &xf, list, list_id, all, &sel, t.selection, look, hl, stencil);
                    draw_highlight(&clipped, &xf, list, &c.hl_infinite, t.selection, look);
                } else {
                    draw_highlight(&clipped, &xf, list, &all(), t.selection, look);
                }
            }
        }
    } else if let CanvasState { list: Some(list), list_gen, prim_index, hover, gpu, hl_key, hl_slot, hl_origin, hl_infinite, .. } = &mut app.canvas {
        if let Some(h) = *hover
            && !sel.contains(&h)
        {
            draw_highlight(&painter, &xf, list, &prim_index.prims(*list_gen, list, &[h]), t.hover, preview_look);
        }
        if let Some(look) = selection_look {
            if gpu.is_some() && !sel.is_empty() {
                let list_id = *list_gen;
                let hl = (hl_key, &*hl_slot, hl_origin, &mut *hl_infinite);
                draw_highlight_gpu(
                    &painter,
                    &xf,
                    list,
                    list_id,
                    || prim_index.prims(list_id, list, &sel),
                    &sel,
                    t.selection,
                    look,
                    hl,
                    crate::gpu::Stencil::Off,
                );
                draw_highlight(&painter, &xf, list, hl_infinite, t.selection, look);
            } else {
                draw_highlight(&painter, &xf, list, &prim_index.prims(*list_gen, list, &sel), t.selection, look);
            }
        }
    }
    app.canvas.draw_ms = crate::now_ms() - t0;
    compare::paint(app, ui.ctx(), &painter, &xf_paper, rect);
    blockedit::paint(app, ui.ctx(), &painter, &xf_paper, rect);
    // Constraint bars and dynamic dimensional constraints (model space, or inside a viewport).
    if sheet.is_none() || active_vp.is_some() {
        let clip = match &active_vp {
            Some(vp) => Rect::from_two_pos(
                xf_paper.to_screen(Vec2::new(vp.center.x - vp.width / 2.0, vp.center.y + vp.height / 2.0)),
                xf_paper.to_screen(Vec2::new(vp.center.x + vp.width / 2.0, vp.center.y - vp.height / 2.0)),
            )
            .intersect(rect),
            None => rect,
        };
        let p = painter.with_clip_rect(clip);
        crate::parametric::draw_overlay(app, ui, &p, &xf);
    }
    // Grips on selected objects (when idle or editing grips).
    let grip_edit = app.session.running.as_ref().is_some_and(|r| r.id == "grip");
    if (!grip_edit && (app.canvas.grip_editing || app.session.prompting())) || sel.is_empty() {
        app.canvas.hot_grips.clear();
    }
    if !grip_edit {
        app.canvas.dyn_hot = None;
    }
    app.canvas.grip_editing = grip_edit;
    if (!app.session.prompting() || grip_edit)
        && app.session.shows_grips(sel.len())
        && let Ok(st) = app.session.state()
    {
        let gs = app.session.settings.gripsize as f32;
        for h in sel.iter().take(200) {
            if let Some(e) = st.doc.entity(*h) {
                let grips = e.kind.grips();
                let count = grips.len();
                for (gi, g) in grips.into_iter().enumerate() {
                    let sp = xf.to_screen(g);
                    if rect.contains(sp) {
                        let r = grip_rect(&e.kind, count, gi, sp, gs);
                        let hot = app.canvas.hot_grips.contains(&(*h, gi));
                        let warm = !hot && hover_pos.is_some_and(|hp| r.expand(2.0).contains(hp));
                        painter.rect_filled(
                            r,
                            0.0,
                            if hot {
                                t.grip_hot
                            } else if warm {
                                Color32::from_rgb(0xff, 0x7f, 0x9f)
                            } else {
                                t.grip
                            },
                        );
                        painter.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_rgb(0x10, 0x30, 0x80)), egui::StrokeKind::Middle);
                    }
                }
            }
        }
        crate::arraygrips::draw(app, &painter, &xf, rect, hover_pos);
        crate::dyngrips::draw(app, &painter, &xf, rect, hover_pos);
    }
    // Rubber band preview of the active command.
    // `xf` is the space being edited: paper space, or model space inside the active viewport.
    let drag = match (app.canvas.cursor, app.canvas.gpu.is_some()) {
        (Some(c), true) => app.session.drag(c),
        _ => None,
    };
    if let Some((drag, extra)) = drag {
        if let Ok(st) = app.session.state() {
            let serial = app.session.clipboard_source.as_ref().map_or(0, |c| c.serial);
            let transparency = app.session.settings.transparency_display;
            draw_drag_gpu(&mut app.canvas, &painter, &xf, &st.doc, &drag, (&app.session.clipboard, serial), bg, transparency);
            let list = rebuilt_preview(&st.doc, extra.iter(), scale);
            draw_list(&painter, &xf, &list, bg, false);
        }
    } else if let (Some(c), true) = (app.canvas.cursor, app.session.prompting()) {
        let ents = app.session.preview(c);
        if !ents.is_empty()
            && let Ok(st) = app.session.state()
        {
            let list = rebuilt_preview(&st.doc, ents.iter(), scale);
            draw_list(&painter, &xf, &list, bg, false);
        }
    }
    // Pending selection window.
    if let (Some(pw), Some(c)) = (app.session.pending_window, raw_world) {
        let a = xf.to_screen(pw.corner);
        let b = xf.to_screen(c);
        let r = Rect::from_two_pos(a, b);
        let crossing = c.x < pw.corner.x;
        if let Some(fill) = selection_area_fill(&app.session.settings, crossing) {
            painter.rect_filled(r, 0.0, fill);
        }
        let st = Stroke::new(1.0, if crossing { t.crossing_stroke } else { t.window_stroke });
        if crossing {
            let pts = vec![r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
            painter.extend(Shape::dashed_line(&pts, st, 5.0, 3.0));
        } else {
            painter.rect_stroke(r, 0.0, st, egui::StrokeKind::Middle);
        }
    }
    // Acquired tracking points and the polar / tracking paths under the cursor.
    if app.canvas.cursor.is_some() {
        draw_tracking(&painter, &xf, &app.session.tracking.acquired, app.canvas.track.as_ref(), view.height * 4.0);
    }
    if let (Some(hit), Some(_)) = (app.canvas.snap, app.canvas.cursor) {
        draw_snap_marker(&painter, xf.to_screen(hit.point), &hit);
    }
    draw_tiles(app, ui, &tiles);
    // UCS icon, ViewCube and viewport label.
    if app.ui.show_ucs_icon {
        draw_ucs_icon(&painter, rect);
        crate::viewcube::ucs_icon(app, ui, rect);
    }
    if app.ui.show_viewcube && sheet.is_none() {
        draw_viewcube(app, ui, rect);
    }
    viewport_label(&painter, rect);
    // Typing and Escape reach the command line even when its overlay is hidden.
    crate::cmdline::keyboard(app, ui.ctx());
    // Command line overlay.
    if app.ui.show_command_line {
        crate::cmdline::show(app, ui, full);
    }
    // Crosshair cursor.
    if inside && !on_menu {
        // Hide the OS cursor under the drawn crosshair, unless assistive tech needs it (#39).
        ui.ctx().set_cursor_icon(if app.ui.system_cursor { egui::CursorIcon::Crosshair } else { egui::CursorIcon::None });
        if let Some(hp) = hover_pos {
            // Use the painted sheet even in MSPACE, where the editing transform follows a
            // model viewport. The whole cursor takes one colour from its centre.
            let ink = paper_rect.map_or(t.crosshair, |paper| crate::theme::crosshair_color(if paper.contains(hp) { t.paper } else { surround }));
            draw_crosshair(app, &painter, rect, hp, selecting, ink);
            if app.canvas.cycle_badge && app.canvas.cycle.is_none() {
                cycle_badge(&painter, hp);
            }
        }
    }
    if inside
        && app.session.settings.dynmode
        && let Some(hp) = hover_pos
    {
        // A stretched grip shows its dimensions instead of the pointer boxes; a prompt asking for
        // a distance shows its one field and keeps the prompt and its options.
        let fields = crate::grip_dyn::draw(app, &painter);
        if !fields || app.session.current_prompt().is_some_and(|p| p.distance) {
            dynamic_input(app, &painter, hp, fields);
        }
    }
    // Over the crosshair, so the row under the pointer reads clearly.
    cycle_list(app, &painter);
    // No continuous repaint while hovering: input events repaint by themselves. Repainting every
    // frame kept a vsync swapchain's queue full (every frame waited behind queued ones) and would
    // spin the CPU with the low-latency present mode (`gpu::surface_config`).
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests;

/// The Block Editor's tint, base point and bar.
mod blockedit;
/// The COMPARE overlay.
mod compare;

/// Background rebuilds of the display list for a new view.
mod rebuild;
