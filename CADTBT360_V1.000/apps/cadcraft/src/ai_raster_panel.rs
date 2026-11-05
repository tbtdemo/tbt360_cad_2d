//! Native desktop workflow for automatic PNG/JPEG-to-editable-CAD draft lines.
//! Uses the same preview function as the MCP agent: no second vision implementation.

use cadcraft_ui_egui::CadApp;
use serde_json::{Value, json};

#[derive(Default)]
pub struct State {
    pub open: bool,
    path: String,
    preview: Option<Value>,
    error: Option<String>,
    min_length: u32,
    scale: f64,
    include_diagonals: bool,
    arbitrary_angles: bool,
}

impl State {
    pub fn open(&mut self) {
        self.open = true;
        if self.min_length == 0 {
            self.min_length = 16;
            self.scale = 1.0;
            self.include_diagonals = true;
        }
    }

    fn params(&self) -> Value {
        json!({"path":self.path,"preview":true,"minLengthPixels":self.min_length,
            "unitsPerPixel":self.scale,"includeDiagonals":self.include_diagonals,
            "includeArbitraryAngles":self.arbitrary_angles,"maxLines":250})
    }
}

pub fn window(ctx: &egui::Context, cad: &mut CadApp, state: &mut State) {
    if !state.open { return; }
    let mut open = state.open;
    egui::Window::new("AI từ ảnh → CAD | TBT 360")
        .open(&mut open).resizable(true).default_width(520.0)
        .show(ctx, |ui| {
            ui.label("Tự nhận dạng các nét thẳng trong PNG/JPEG để tạo LINE chỉnh sửa được.");
            ui.label("Đây là vector hóa pixel, chưa tự hiểu tường/cửa/kích thước.");
            ui.horizontal(|ui| {
                if ui.button("Chọn ảnh PNG/JPG…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png","jpg","jpeg"]).pick_file()
                {
                    state.path = path.to_string_lossy().into_owned();
                    state.preview = None;
                    state.error = None;
                }
                ui.label(state.path.rsplit(['/', '\\']).next().unwrap_or("(Chưa chọn ảnh)"));
            });
            ui.horizontal(|ui| {
                ui.label("Nét tối thiểu (px)");
                if ui.add(egui::DragValue::new(&mut state.min_length).range(2..=4096)).changed() { state.preview = None; }
                ui.label("Đơn vị CAD/pixel");
                if ui.add(egui::DragValue::new(&mut state.scale).speed(0.01).range(0.000001..=1_000_000.0)).changed() { state.preview = None; }
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut state.include_diagonals, "Nét 45°").changed() { state.preview = None; }
                if ui.checkbox(&mut state.arbitrary_angles, "Góc bất kỳ (thử nghiệm)").changed() { state.preview = None; }
            });
            if ui.button("1. Nhận dạng và xem trước").clicked() {
                match cadcraft_mcp::raster_vector::preview(&state.params()) {
                    Ok(value) => { state.preview = Some(value); state.error = None; }
                    Err(e) => { state.preview = None; state.error = Some(e); }
                }
            }
            if let Some(error) = &state.error {
                ui.colored_label(egui::Color32::RED, error);
            }
            let Some(proposal) = &state.preview else { return; };
            let count = proposal["strokes"].as_u64().unwrap_or(0);
            ui.separator();
            ui.label(format!("Đã nhận dạng: {count} nét. Kiểm tra trước khi ghi vào bản vẽ."));
            if let Some(lines) = proposal["lines"].as_array() {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().max(200.0), 220.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, egui::Color32::from_gray(27));
                let mut min_x = f64::INFINITY; let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY; let mut max_y = f64::NEG_INFINITY;
                for segment in lines {
                    for point in segment.as_array().into_iter().flatten() {
                        if let (Some(x), Some(y)) = (point[0].as_f64(), point[1].as_f64()) {
                            min_x = min_x.min(x); max_x = max_x.max(x);
                            min_y = min_y.min(y); max_y = max_y.max(y);
                        }
                    }
                }
                let span_x = (max_x - min_x).max(1.0e-9);
                let span_y = (max_y - min_y).max(1.0e-9);
                let factor = (f64::from(rect.width()) * 0.90 / span_x)
                    .min(f64::from(rect.height()) * 0.90 / span_y);
                for segment in lines {
                    let (Some(a), Some(b)) = (segment.get(0), segment.get(1)) else { continue };
                    let (Some(x0), Some(y0), Some(x1), Some(y1)) =
                        (a[0].as_f64(), a[1].as_f64(), b[0].as_f64(), b[1].as_f64()) else { continue };
                    let to_screen = |x: f64, y: f64| egui::pos2(
                        (f64::from(rect.center().x) + ((x - (min_x + max_x) * 0.5) * factor)) as f32,
                        (f64::from(rect.center().y) - ((y - (min_y + max_y) * 0.5) * factor)) as f32);
                    painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                        egui::Stroke::new(1.5, egui::Color32::LIGHT_GREEN));
                }
            }
            ui.label("Bản xem trước chưa sửa dữ liệu CAD. Hãy lưu bản vẽ trước khi thực hiện một lô nét lớn.");
            if ui.add_enabled(count > 0, egui::Button::new("2. Tạo LINE chỉnh sửa trong CAD")).clicked() {
                // Keep the whole AI batch in a single user-visible Undo step. Do not
                // unexpectedly close somebody else's active undo group.
                let undo_ready = cad.session.state().map(|d| {
                    d.undo_ctl.control == cadcraft_engine::undoctl::Control::All && d.undo_ctl.group.is_none()
                }).unwrap_or(false);
                if !undo_ready {
                    state.error = Some("Không thể tạo nhóm AI Undo: hãy kết thúc nhóm UNDO đang mở và bật UNDO Control All.".to_string());
                    return;
                }
                if let Err(e) = cad.session.state_mut().map(|d| d.begin_undo_group()) {
                    state.error = Some(format!("Không thể bắt đầu nhóm UNDO: {e}"));
                    return;
                }
                let mut created = 0usize;
                let mut failure = None;
                if let Some(lines) = proposal["lines"].as_array() {
                    for line in lines {
                        match cad.run("line", json!({"points":line})) {
                            Ok(_) => created += 1,
                            Err(e) => { failure = Some(e); break; }
                        }
                    }
                }
                if let Err(e) = cad.session.state_mut().map(|d| d.end_undo_group()) {
                    failure.get_or_insert_with(|| format!("Không thể kết thúc nhóm UNDO: {e}"));
                }
                if let Some(problem) = failure {
                    // Only the AI group was opened above. The first snapshot
                    // of that group predates the entire operation. Roll back
                    // automatically instead of leaving half a traced image.
                    if created > 0 {
                        match cad.run("undo", json!({"count":1})) {
                            Ok(_) => state.error = Some(format!("Đã hoàn tác nhóm AI sau lỗi: {problem}")),
                            Err(rollback) => state.error = Some(format!("Lỗi AI sau {created} LINE: {problem}. Hoàn tác tự động thất bại: {rollback}. Kiểm tra bản vẽ ngay.")),
                        }
                    } else {
                        state.error = Some(format!("AI không tạo nét nào: {problem}"));
                    }
                } else {
                    state.error = None;
                    cad.set_status(format!("AI ảnh → CAD: đã tạo {created} LINE trong một nhóm UNDO. Kiểm tra trước khi lưu DWG."));
                    state.preview = None;
                    state.open = false;
                }
            }
        });
    state.open = open && state.open;
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn ai_raster_undo_rolls_back_partial_batch_when_second_line_fails() {
        let mut engine = cadcraft_engine::Session::new();
        let before_count = engine.doc().unwrap().entity_count();
        let before_undo = engine.state().unwrap().undo.len();
        engine.state_mut().unwrap().begin_undo_group();
        engine.execute("line", &json!({"points":[[0,0],[5,0]]})).unwrap();
        assert!(engine.execute("line", &json!({"points":[[9,9],[9,9]]})).is_err());
        engine.state_mut().unwrap().end_undo_group();
        assert_eq!(engine.doc().unwrap().entity_count(), before_count + 1);
        engine.execute("undo", &json!({"count":1})).unwrap();
        assert_eq!(engine.doc().unwrap().entity_count(), before_count, "failed AI batch must not leave any draft LINE");
        assert_eq!(engine.state().unwrap().undo.len(), before_undo);
        assert!(engine.state().unwrap().undo_ctl.group.is_none());
    }

    #[test]
    fn ai_raster_multi_line_group_is_one_undo_step() {
        let mut engine = cadcraft_engine::Session::new();
        let before = engine.state().unwrap().undo.len();
        assert!(engine.state().unwrap().undo_ctl.group.is_none());
        engine.state_mut().unwrap().begin_undo_group();
        engine.execute("line", &json!({"points":[[0,0],[5,0]]})).unwrap();
        engine.execute("line", &json!({"points":[[0,2],[5,2]]})).unwrap();
        assert_eq!(engine.state().unwrap().undo_ctl.group, Some(before));
        engine.state_mut().unwrap().end_undo_group();
        assert_eq!(engine.state().unwrap().undo.len(), before + 1);
        assert!(engine.state().unwrap().undo_ctl.group.is_none());
    }

    #[test]
    fn ai_raster_panel_defaults_to_preview_and_never_requests_direct_edit() {
        let mut state = State::default();
        state.open();
        assert!(state.open);
        assert_eq!(state.min_length, 16);
        assert!(state.include_diagonals);
        assert_eq!(state.params()["preview"], true);
        assert_eq!(state.params()["includeArbitraryAngles"], false);
    }

    #[test]
    fn ai_raster_panel_can_enable_arbitrary_angles_without_changing_preview_gate() {
        let mut state = State::default();
        state.open();
        state.arbitrary_angles = true;
        state.scale = 0.5;
        assert_eq!(state.params()["includeArbitraryAngles"], true);
        assert_eq!(state.params()["unitsPerPixel"], 0.5);
        assert_eq!(state.params()["preview"], true);
    }
}
