//! IFC inspection preview. Full tessellation requires a separately packaged IfcConvert runtime.
use std::{collections::BTreeMap, io::{BufRead, BufReader}, path::PathBuf};

#[derive(Default)]
pub struct State {
    path: Option<PathBuf>,
    file_name: String,
    counts: BTreeMap<String, usize>,
    points: Vec<[f32; 3]>,
    mesh: Option<crate::ifc_mesh::Mesh>,
    mesh_error: Option<String>,
    error: Option<String>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    wireframe: bool,
    show_points: bool,
    conversion: Option<std::sync::mpsc::Receiver<Result<crate::ifc_mesh::Mesh, String>>>,
    cancel_conversion: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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
        self.mesh = None;
        self.mesh_error = None;
        self.error = None;
        self.zoom = 1.0;
        Ok(())
    }
}

fn triangle_visible_on_screen(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2, rect: egui::Rect) -> bool {
    if ![a,b,c].iter().all(|p| p.x.is_finite() && p.y.is_finite()) { return false; }
    let min = egui::pos2(a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y));
    let max = egui::pos2(a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y));
    egui::Rect::from_min_max(min, max).intersects(rect)
}

pub fn panel(ui: &mut egui::Ui, state: &mut State) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("3D / IFC");
        if ui.add_enabled(state.conversion.is_none(), egui::Button::new("Mở tệp IFC...")).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Industry Foundation Classes", &["ifc"]).pick_file() {
                if let Err(error) = state.open(path) { state.error = Some(error); }
            }
        }
        if ui.add_enabled(state.conversion.is_none(), egui::Button::new("Nạp mesh OBJ...")).clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Wavefront OBJ", &["obj"]).pick_file() {
                match crate::ifc_mesh::Mesh::read(&path) {
                    Ok(mesh) => { state.mesh = Some(mesh); state.mesh_error = None; }
                    Err(error) => state.mesh_error = Some(error),
                }
            }
        }
        if ui.add_enabled(state.conversion.is_none() && state.path.is_some(), egui::Button::new("Chuyển IFC → Mesh")).clicked() {
            if let Some(path) = state.path.clone() {
                let tool = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("tools").join("IfcConvert.exe")));
                match tool.filter(|p| p.is_file()) {
                    None => state.mesh_error = Some("Chưa có tools/IfcConvert.exe đi kèm CAD360. Cần build và đóng gói IfcOpenShell cho Windows.".into()),
                    Some(tool) => {
                        let (tx, rx) = std::sync::mpsc::channel();
                        state.conversion = Some(rx);
                        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        state.cancel_conversion = Some(cancel.clone());
                        state.mesh_error = None;
                        let context = ui.ctx().clone();
                        std::thread::spawn(move || {
                            let mut output = std::env::temp_dir();
                            let serial = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|t| t.as_nanos()).unwrap_or(0);
                            output.push(format!("cad360-ifc-{}-{serial}.obj", std::process::id()));
                            let result = (|| -> Result<crate::ifc_mesh::Mesh, String> {
                                let mut child = std::process::Command::new(tool).arg(&path).arg(&output).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map_err(|e| format!("Không khởi chạy được IfcConvert: {e}"))?;
                                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
                                loop {
                                    if cancel.load(std::sync::atomic::Ordering::Relaxed) { let _ = child.kill(); let _ = child.wait(); return Err("Đã hủy chuyển đổi IFC".into()); }
                                    match child.try_wait() {
                                        Ok(Some(status)) if status.success() => break,
                                        Ok(Some(status)) => return Err(format!("IfcConvert kết thúc với mã lỗi: {status}")),
                                        Ok(None) if std::time::Instant::now() >= deadline => { let _ = child.kill(); let _ = child.wait(); return Err("IfcConvert quá thời hạn 180 giây và đã bị dừng".into()); }
                                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                                        Err(error) => { let _ = child.kill(); let _ = child.wait(); return Err(format!("Không kiểm tra được tiến trình IFC: {error}")); }
                                    }
                                }
                                if cancel.load(std::sync::atomic::Ordering::Relaxed) { return Err("Đã hủy chuyển đổi IFC".into()); }
                                let metadata = std::fs::metadata(&output).map_err(|e| format!("Thiếu OBJ sau chuyển đổi: {e}"))?;
                                if metadata.len() == 0 || metadata.len() > 750_000_000 { return Err("OBJ rỗng hoặc vượt giới hạn 750 MB".into()); }
                                crate::ifc_mesh::Mesh::read(&output)
                            })();
                            let _ = std::fs::remove_file(&output);
                            let _ = tx.send(result);
                            context.request_repaint();
                        });
                    }
                }
            }
        }
        if state.conversion.is_some() && ui.button("Hủy chuyển đổi").clicked() { if let Some(cancel) = &state.cancel_conversion { cancel.store(true, std::sync::atomic::Ordering::Relaxed); } }
        ui.checkbox(&mut state.wireframe, "Khung dây");
        ui.checkbox(&mut state.show_points, "Hiển thị đỉnh");
        if ui.button("Đặt lại góc nhìn").clicked() { state.yaw = 0.5; state.pitch = 0.35; state.zoom = 1.0; }
    });
    if let Some(receiver) = &state.conversion {
        match receiver.try_recv() {
            Ok(Ok(mesh)) => { state.mesh = Some(mesh); state.mesh_error = None; state.conversion = None; state.cancel_conversion = None; }
            Ok(Err(error)) => { state.mesh_error = Some(error); state.conversion = None; state.cancel_conversion = None; }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => { state.mesh_error = Some("Tiến trình chuyển đổi IFC bị ngắt".into()); state.conversion = None; state.cancel_conversion = None; }
            Err(std::sync::mpsc::TryRecvError::Empty) => { ui.spinner(); ui.label("Đang chuyển IFC thành mesh, giao diện vẫn có thể thao tác..."); ui.ctx().request_repaint_after(std::time::Duration::from_millis(150)); }
        }
    }
    if let Some(error) = &state.error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    if let Some(error) = &state.mesh_error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    if state.path.is_none() && state.mesh.is_none() {
        ui.label("Chọn tệp .ifc để đọc danh mục đối tượng, tọa độ và xem sơ bộ phân bố điểm.");
        ui.label("Có thể nạp trực tiếp OBJ hoặc dùng IfcConvert để tạo mesh từ IFC.");
        return;
    }
    if !state.file_name.is_empty() { ui.label(format!("Tệp: {}", state.file_name)); }
    if let Some(mesh) = &state.mesh { ui.label(format!("Mesh: {} đỉnh • {} tam giác", mesh.vertices.len(), mesh.faces.len())); }
    ui.horizontal(|ui| {
        ui.label(format!("Loại IFC: {}", state.counts.len()));
        ui.label(format!("Điểm tọa độ đọc: {}", state.points.len()));
        ui.label("Kéo chuột để xoay, cuộn để phóng to");
    });
    if state.mesh.is_none() { ui.colored_label(egui::Color32::YELLOW, "Chế độ xem sơ bộ tọa độ IFC — chưa có Solid/Mesh thực."); }
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
    let painter = ui.painter_at(rect).with_clip_rect(rect);
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 25, 35));
    let displayed: &[[f32; 3]] = state.mesh.as_ref().map(|m| m.vertices.as_slice()).unwrap_or(&state.points);
    if !displayed.is_empty() {
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in displayed { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
        let center = [0, 1, 2].map(|k| (f64::from(lo[k]) + f64::from(hi[k])) * 0.5);
        let span = (0..3).map(|k| f64::from(hi[k]) - f64::from(lo[k])).fold(1.0_f64, f64::max);
        let scale = f64::from(rect.width().min(rect.height())) * 0.8 * f64::from(state.zoom) / span;
        let (sy, cy) = state.yaw.sin_cos();
        let (sp, cp) = state.pitch.sin_cos();
        let project = |p: &[f32;3]| {
            let x = f64::from(p[0]) - center[0]; let y = f64::from(p[1]) - center[1]; let z = f64::from(p[2]) - center[2];
            let (sy, cy) = (f64::from(sy), f64::from(cy)); let (sp, cp) = (f64::from(sp), f64::from(cp));
            let rx = x * cy - y * sy; let ry = x * sy + y * cy;
            let rz = z * cp - ry * sp;
            let pos = rect.center() + egui::vec2((rx * scale) as f32, (-rz * scale) as f32);
            pos
        };
        if let Some(mesh) = &state.mesh {
            let mut faces = mesh.faces.iter().take(100_000).map(|triangle| {
                let depth = triangle.iter().map(|&i| { let p = mesh.vertices[i]; let x = f64::from(p[0])-center[0]; let y = f64::from(p[1])-center[1]; let z = f64::from(p[2])-center[2]; let ry = x*f64::from(sy)+y*f64::from(cy); ry*f64::from(cp)+z*f64::from(sp) }).sum::<f64>() / 3.0;
                (depth, triangle)
            }).collect::<Vec<_>>();
            faces.sort_by(|a,b| a.0.total_cmp(&b.0));
            for (_, triangle) in faces {
                let a = project(&mesh.vertices[triangle[0]]);
                let b = project(&mesh.vertices[triangle[1]]);
                let c = project(&mesh.vertices[triangle[2]]);
                if !triangle_visible_on_screen(a, b, c, rect) { continue; }
                if state.wireframe {
                    for (u, v) in [(a, b), (b, c), (c, a)] { painter.line_segment([u, v], egui::Stroke::new(0.7, egui::Color32::from_rgb(130, 195, 240))); }
                } else {
                    painter.add(egui::Shape::convex_polygon(vec![a, b, c], egui::Color32::from_rgba_unmultiplied(52, 120, 165, 90), egui::Stroke::new(0.5, egui::Color32::from_rgb(105, 185, 230))));
                }
            }
            if state.show_points { for p in displayed.iter().take(20_000) { let pos = project(p); if rect.contains(pos) { painter.circle_filled(pos, 1.0, egui::Color32::from_rgb(245, 205, 110)); } } }
        } else {
            for p in displayed { let pos = project(p); if rect.contains(pos) { painter.circle_filled(pos, 1.3, egui::Color32::from_rgb(90, 194, 245)); } }
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
    fn viewport_culling_rejects_nonfinite_and_outside_faces() {
        let view = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0));
        assert!(triangle_visible_on_screen(egui::pos2(10.0, 10.0), egui::pos2(30.0, 10.0), egui::pos2(10.0, 30.0), view));
        assert!(!triangle_visible_on_screen(egui::pos2(120.0, 120.0), egui::pos2(140.0, 120.0), egui::pos2(120.0, 140.0), view));
        assert!(!triangle_visible_on_screen(egui::pos2(f32::NAN, 0.0), egui::pos2(10.0, 0.0), egui::pos2(0.0, 10.0), view));
    }

    #[test]
    fn parses_ifc_3d_point_and_rejects_2d() {
        assert_eq!(point_from_step("#42=IFCCARTESIANPOINT((1.5,-2,3.0));"), Some([1.5,-2.0,3.0]));
        assert!(point_from_step("#1=IFCCARTESIANPOINT((1,2));").is_none());
    }
}
