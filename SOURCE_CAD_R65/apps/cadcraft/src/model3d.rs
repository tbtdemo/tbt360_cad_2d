//! IFC inspection preview. Full tessellation requires a separately packaged IfcConvert runtime.
use std::{collections::BTreeMap, io::{BufRead, BufReader}, path::PathBuf};

#[derive(Default)]
pub struct State {
    path: Option<PathBuf>,
    file_name: String,
    counts: BTreeMap<String, usize>,
    points: Vec<[f32; 3]>,
    error: Option<String>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
}

fn point_from_step(line: &str) -> Option<[f32; 3]> {
    let upper = line.to_ascii_uppercase();
    let start = upper.find("IFCCARTESIANPOINT((")? + "IFCCARTESIANPOINT((".len();
    let end = line[start..].find(')')? + start;
    let values: Vec<f32> = line[start..end].split(',').map(|v| v.trim().parse::<f32>()).collect::<Result<_, _>>().ok()?;
    if values.len() != 3 || !values.iter().all(|v| v.is_finite()) { return None; }
    Some([values[0], values[1], values[2]])
}

impl State {
    fn open(&mut self, path: PathBuf) -> Result<(), String> {
        let file = std::fs::File::open(&path).map_err(|e| format!("Không mở được IFC: {e}"))?;
        let mut reader = BufReader::new(file);
        let mut first = String::new();
        reader.read_line(&mut first).map_err(|e| e.to_string())?;
        if !first.trim_start_matches('\u{feff}').trim().starts_with("ISO-10303-21") { return Err("Tệp không phải định dạng STEP IFC (ISO-10303-21)".into()); }
        let mut counts = BTreeMap::new();
        let mut points = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 { break; }
            let upper = line.to_ascii_uppercase();
            if let Some(pos) = upper.find("=IFC") {
                if let Some(end) = upper[pos + 1..].find('(') {
                    let kind = &upper[pos + 1..pos + 1 + end];
                    if kind.starts_with("IFC") && kind.len() < 80 && kind.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        *counts.entry(kind.to_owned()).or_insert(0) += 1;
                    }
                }
            }
            if points.len() < 8000 {
                if let Some(p) = point_from_step(&line) { points.push(p); }
            }
        }
        self.file_name = path.file_name().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        self.path = Some(path);
        self.counts = counts;
        self.points = points;
        self.error = None;
        self.zoom = 1.0;
        Ok(())
    }
}

pub fn panel(ui: &mut egui::Ui, state: &mut State) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("3D / IFC");
        if ui.button("Mở tệp IFC...").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Industry Foundation Classes", &["ifc"]).pick_file() {
                if let Err(error) = state.open(path) { state.error = Some(error); }
            }
        }
        if ui.button("Đặt lại góc nhìn").clicked() { state.yaw = 0.5; state.pitch = 0.35; state.zoom = 1.0; }
    });
    if let Some(error) = &state.error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    if state.path.is_none() {
        ui.label("Chọn tệp .ifc để đọc danh mục đối tượng, tọa độ và xem sơ bộ phân bố điểm.");
        ui.label("Để hiển thị Solid/Mesh thực, cần cài đặt IfcConvert từ IfcOpenShell và tích hợp renderer.");
        return;
    }
    ui.label(format!("Tệp: {}", state.file_name));
    ui.horizontal(|ui| {
        ui.label(format!("Loại IFC: {}", state.counts.len()));
        ui.label(format!("Điểm tọa độ đọc: {}", state.points.len()));
        ui.label("Kéo chuột để xoay, cuộn để phóng to");
    });
    ui.colored_label(egui::Color32::YELLOW, "Chế độ xem sơ bộ tọa độ IFC — KHÔNG phải hình học Solid/Mesh hoặc kết quả đo bóc.");
    ui.separator();
    let width = ui.available_width().max(200.0);
    let height = ui.available_height().max(180.0) * 0.75;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::drag());
    if response.dragged() {
        let delta = response.drag_delta();
        state.yaw += delta.x * 0.008;
        state.pitch = (state.pitch + delta.y * 0.008).clamp(-1.4, 1.4);
    }
    if response.hovered() {
        let scroll = ui.ctx().input(|input| input.raw_scroll_delta.y);
        state.zoom = (state.zoom * (1.0 + scroll * 0.001)).clamp(0.05, 30.0);
    }
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 25, 35));
    if !state.points.is_empty() {
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in &state.points { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
        let center = [(lo[0]+hi[0])*0.5, (lo[1]+hi[1])*0.5, (lo[2]+hi[2])*0.5];
        let span = (0..3).map(|k| hi[k] - lo[k]).fold(1.0_f32, f32::max);
        let scale = rect.width().min(rect.height()) * 0.8 * state.zoom / span;
        let (sy, cy) = state.yaw.sin_cos();
        let (sp, cp) = state.pitch.sin_cos();
        for p in &state.points {
            let x = p[0] - center[0]; let y = p[1] - center[1]; let z = p[2] - center[2];
            let rx = x * cy - y * sy; let ry = x * sy + y * cy;
            let rz = z * cp - ry * sp;
            let pos = rect.center() + egui::vec2(rx * scale, -rz * scale);
            if rect.contains(pos) { painter.circle_filled(pos, 1.3, egui::Color32::from_rgb(90, 194, 245)); }
        }
    } else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Không tìm thấy IFCCARTESIANPOINT 3D trong tệp", egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
    }
    egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
        ui.collapsing("Thống kê IFC Entities", |ui| {
            for (kind, count) in &state.counts { ui.label(format!("{kind}: {count}")); }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_ifc_3d_point_and_rejects_2d() {
        assert_eq!(point_from_step("#42=IFCCARTESIANPOINT((1.5,-2,3.0));"), Some([1.5,-2.0,3.0]));
        assert!(point_from_step("#1=IFCCARTESIANPOINT((1,2));").is_none());
    }
}
