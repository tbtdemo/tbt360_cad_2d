//! IFC inspection preview. Full tessellation requires a separately packaged IfcConvert runtime.
use crate::ifc_catalog;
use std::{collections::{BTreeMap, BTreeSet}, io::{BufRead, BufReader}, path::PathBuf};

const MAX_IFC_PREVIEW_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_IFC_PREVIEW_LINE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Default)]
pub struct State {
    path: Option<PathBuf>,
    file_name: String,
    counts: BTreeMap<String, usize>,
    points: Vec<[f32; 3]>,
    mesh: Option<crate::ifc_mesh::Mesh>,
    mesh_error: Option<String>,
    gpu_batch: Option<crate::ifc_gpu::Batch>,
    gpu_error: Option<String>,
    error: Option<String>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    wireframe: bool,
    show_points: bool,
    selected_triangle: Option<usize>,
    select_group: bool,
    hidden_groups: BTreeSet<usize>,
    isolated_group: Option<usize>,
    catalog: ifc_catalog::Catalog,
    global_id_filter: String,
    storey_filter: String,
    discipline_filter: String,
    catalog_error: Option<String>,
    conversion: Option<std::sync::mpsc::Receiver<Result<crate::ifc_mesh::Mesh, String>>>,
    cancel_conversion: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

fn read_bounded_line<Reader: BufRead>(
    reader: &mut Reader,
    destination: &mut Vec<u8>,
    maxBytes: usize,
) -> Result<usize, String>
{
    destination.clear();
    loop
    {
        let available = reader
            .fill_buf()
            .map_err(|error| format!("Không đọc được dữ liệu: {error}"))?;

        if available.is_empty()
        {
            return Ok(destination.len());
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let bytesToCopy = newline.map_or(available.len(), |index| index + 1);

        if destination.len().saturating_add(bytesToCopy) > maxBytes
        {
            return Err(format!("Dòng dữ liệu vượt giới hạn {maxBytes} bytes"));
        }

        destination.extend_from_slice(&available[..bytesToCopy]);
        reader.consume(bytesToCopy);

        if newline.is_some()
        {
            return Ok(destination.len());
        }
    }
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
        let file = std::fs::File::open(&path)
            .map_err(|error| format!("Không mở được IFC: {error}"))?;
        let file_size = file
            .metadata()
            .map_err(|error| format!("Không đọc được kích thước IFC: {error}"))?
            .len();

        if file_size > MAX_IFC_PREVIEW_FILE_BYTES {
            log::warn!("IFC preview rejected oversized file: {} bytes", file_size);
            return Err("IFC vượt giới hạn 512 MiB cho chế độ xem sơ bộ".into());
        }

        let mut reader = BufReader::new(file);
        let mut first_buffer = Vec::new();
        read_bounded_line(
            &mut reader,
            &mut first_buffer,
            MAX_IFC_PREVIEW_LINE_BYTES,
        )?;

        if first_buffer.len() > MAX_IFC_PREVIEW_LINE_BYTES {
            return Err("Header IFC vượt giới hạn 4 MiB".into());
        }

        let first = std::str::from_utf8(&first_buffer)
            .map_err(|_| "Header IFC không phải UTF-8 hợp lệ")?;
        if !first.trim_start_matches('\u{feff}').trim().starts_with("ISO-10303-21") { return Err("Tệp không phải định dạng STEP IFC (ISO-10303-21)".into()); }
        let mut counts = BTreeMap::new();
        let mut points = Vec::new();
        let mut line_buffer = Vec::new();
        loop {
            line_buffer.clear();
            let bytes_read = read_bounded_line(
                &mut reader,
                &mut line_buffer,
                MAX_IFC_PREVIEW_LINE_BYTES,
            )?;

            if bytes_read == 0 {
                break;
            }

            if bytes_read > MAX_IFC_PREVIEW_LINE_BYTES {
                log::warn!("IFC preview rejected a line longer than 4 MiB");
                return Err("Dòng IFC vượt giới hạn 4 MiB".into());
            }

            let line = std::str::from_utf8(&line_buffer)
                .map_err(|_| "Nội dung IFC không phải UTF-8 hợp lệ")?;
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
        self.gpu_batch = None;
        self.gpu_error = None;
        self.selected_triangle = None;
        self.hidden_groups.clear();
        self.isolated_group = None;
        self.catalog = ifc_catalog::Catalog::default();
        self.catalog_error = None;
        self.global_id_filter.clear();
        self.storey_filter.clear();
        self.discipline_filter.clear();
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

fn triangle_barycentric(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> Option<[f64; 3]> {
    let cross = |u: egui::Pos2, v: egui::Pos2, w: egui::Pos2| f64::from(v.x-u.x)*f64::from(w.y-u.y)-f64::from(v.y-u.y)*f64::from(w.x-u.x);
    let area = cross(a,b,c);
    if !area.is_finite() || area.abs() < 1e-8 { return None; }
    let weights = [cross(p,b,c)/area, cross(a,p,c)/area, cross(a,b,p)/area];
    weights.iter().all(|x| x.is_finite() && *x >= -1e-6 && *x <= 1.0 + 1e-6).then_some(weights)
}

fn point_in_triangle(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> bool {
    let cross = |u: egui::Pos2, v: egui::Pos2, w: egui::Pos2| f64::from(v.x-u.x)*f64::from(w.y-u.y)-f64::from(v.y-u.y)*f64::from(w.x-u.x);
    let area = cross(a,b,c);
    if !area.is_finite() || area.abs() < 1e-8 { return false; }
    let x = cross(a,b,p)/area; let y = cross(b,c,p)/area; let z = cross(c,a,p)/area;
    x >= -1e-6 && y >= -1e-6 && z >= -1e-6
}

fn group_is_visible(group: usize, hidden: &BTreeSet<usize>, isolated: Option<usize>) -> bool {
    !hidden.contains(&group) && isolated.is_none_or(|active| active == group)
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
                    Ok(mesh) => { match crate::ifc_gpu::Batch::from_mesh(&mesh) { Ok(batch) => { state.gpu_batch = Some(batch); state.gpu_error = None; }, Err(error) => { state.gpu_batch = None; state.gpu_error = Some(error); } }; state.mesh = Some(mesh); state.path = None; state.file_name = path.file_name().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(); state.counts.clear(); state.points.clear(); state.selected_triangle = None; state.hidden_groups.clear(); state.isolated_group = None; state.catalog = ifc_catalog::Catalog::default(); state.catalog_error = None; state.global_id_filter.clear(); state.storey_filter.clear(); state.discipline_filter.clear(); state.zoom = 1.0; state.mesh_error = None; state.error = None; }
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
        ui.checkbox(&mut state.select_group, "Chọn cả nhóm đối tượng");
        if let (Some(mesh), Some(index)) = (&state.mesh, state.selected_triangle) {
            if let Some(&group) = mesh.face_groups.get(index) {
                if ui.button("Ẩn nhóm đã chọn").clicked() { state.hidden_groups.insert(group); state.selected_triangle = None; }
                if ui.button("Cô lập nhóm đã chọn").clicked() { state.isolated_group = Some(group); state.hidden_groups.remove(&group); }
            }
        }
        if ui.button("Hiện tất cả").clicked() { state.hidden_groups.clear(); state.isolated_group = None; }
        if ui.button("Nạp ánh xạ IFC...").clicked() { if let Some(path) = rfd::FileDialog::new().add_filter("BIM mapping JSON", &["json"]).pick_file() { match ifc_catalog::Catalog::read(&path) { Ok(c) => { state.catalog = c; state.catalog_error = None; }, Err(e) => state.catalog_error = Some(e) } } }
        if ui.button("Đặt lại góc nhìn").clicked() { state.yaw = 0.5; state.pitch = 0.35; state.zoom = 1.0; }
    });
    if let Some(receiver) = &state.conversion {
        match receiver.try_recv() {
            Ok(Ok(mesh)) => { match crate::ifc_gpu::Batch::from_mesh(&mesh) { Ok(batch) => { state.gpu_batch = Some(batch); state.gpu_error = None; }, Err(error) => { state.gpu_batch = None; state.gpu_error = Some(error); } }; state.mesh = Some(mesh); state.selected_triangle = None; state.hidden_groups.clear(); state.isolated_group = None; state.mesh_error = None; state.conversion = None; state.cancel_conversion = None; }
            Ok(Err(error)) => { state.mesh_error = Some(error); state.conversion = None; state.cancel_conversion = None; }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => { state.mesh_error = Some("Tiến trình chuyển đổi IFC bị ngắt".into()); state.conversion = None; state.cancel_conversion = None; }
            Err(std::sync::mpsc::TryRecvError::Empty) => { ui.spinner(); ui.label("Đang chuyển IFC thành mesh, giao diện vẫn có thể thao tác..."); ui.ctx().request_repaint_after(std::time::Duration::from_millis(150)); }
        }
    }
    if let Some(error) = &state.error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    if let Some(error) = &state.mesh_error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    if let Some(error) = &state.gpu_error { ui.colored_label(egui::Color32::YELLOW, format!("GPU staging: {error}")); }
    if let Some(batch) = &state.gpu_batch
    {
      let groups = state
        .mesh
        .as_ref()
        .map(|mesh| mesh.groups.as_slice())
        .unwrap_or(&[]);

      let visibleTriangleCount = batch.filtered_triangle_count(
        groups,
        &state.catalog,
        &state.hidden_groups,
        state.isolated_group,
        &state.global_id_filter,
        &state.storey_filter,
        &state.discipline_filter,
      );

      let plannedDrawCount = crate::ifc_gpu::Batch::draw_batch_count(
        visibleTriangleCount,
        150_000,
      );

      ui.small(format!(
        "GPU staging: {} đỉnh, {} / {} tam giác qua lọc, {} draw batches (chưa upload wgpu)",
        batch.positions.len(),
        visibleTriangleCount,
        batch.indices.len() / 3,
        plannedDrawCount,
      ));
    }
    if let Some(error) = &state.catalog_error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
    ui.horizontal_wrapped(|ui| { ui.label("IFC GlobalId"); ui.add(egui::TextEdit::singleline(&mut state.global_id_filter).desired_width(150.0)); ui.label("Tầng"); ui.add(egui::TextEdit::singleline(&mut state.storey_filter).desired_width(110.0)); ui.label("Bộ môn"); ui.add(egui::TextEdit::singleline(&mut state.discipline_filter).desired_width(120.0)); if ui.button("Xóa lọc").clicked() { state.global_id_filter.clear(); state.storey_filter.clear(); state.discipline_filter.clear(); } });
    if state.catalog.by_group.is_empty() { ui.small("Chưa có ánh xạ IFC: khi lọc, các nhóm OBJ chưa được định danh sẽ bị ẩn. Tải JSON xuất từ pipeline IFC để đối chiếu chính xác."); }
    if state.path.is_none() && state.mesh.is_none() {
        ui.label("Chọn tệp .ifc để đọc danh mục đối tượng, tọa độ và xem sơ bộ phân bố điểm.");
        ui.label("Có thể nạp trực tiếp OBJ hoặc dùng IfcConvert để tạo mesh từ IFC.");
        return;
    }
    if !state.file_name.is_empty() { ui.label(format!("Tệp: {}", state.file_name)); }
    if let Some(mesh) = &state.mesh {
        ui.label(format!("Mesh: {} đỉnh • {} tam giác", mesh.vertices.len(), mesh.faces.len()));
        let mapped = mesh.groups.iter().filter(|g| state.catalog.get(g).is_some()).count();
        let filtered = mesh.groups.iter().filter(|g| ifc_catalog::matches(state.catalog.get(g), &state.global_id_filter, &state.storey_filter, &state.discipline_filter)).count();
        ui.label(format!("Nhóm OBJ: {} • ánh xạ IFC: {} • qua lọc: {}", mesh.groups.len(), mapped, filtered));
        if filtered == 0 && (!state.global_id_filter.is_empty() || !state.storey_filter.is_empty() || !state.discipline_filter.is_empty()) { ui.colored_label(egui::Color32::YELLOW, "Không có nhóm OBJ nào phù hợp bộ lọc hoặc chưa có ánh xạ GlobalId."); }
    }
    ui.horizontal(|ui| {
        ui.label(format!("Loại IFC: {}", state.counts.len()));
        ui.label(format!("Điểm tọa độ đọc: {}", state.points.len()));
        ui.label("Kéo chuột để xoay, cuộn để phóng to");
    });
    if state.mesh.is_none() { ui.colored_label(egui::Color32::YELLOW, "Chế độ xem sơ bộ tọa độ IFC — chưa có Solid/Mesh thực."); }
    ui.separator();
    let width = ui.available_width().max(200.0);
    let height = ui.available_height().max(180.0) * 0.75;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());
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
    let has_ifc_filter = !state.global_id_filter.is_empty() || !state.storey_filter.is_empty() || !state.discipline_filter.is_empty();
    let displayed: &[[f32; 3]] = state.mesh.as_ref().map(|m| m.vertices.as_slice()).unwrap_or_else(|| if has_ifc_filter { &[] } else { &state.points });
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
            // Apply BIM visibility before the CPU painter budget: filtered elements otherwise disappear if beyond the first 100k mesh faces.
            let visible_faces = mesh.faces.iter().enumerate().filter(|(index, _)| {
                let group = mesh.face_groups.get(*index).copied().unwrap_or(0);
                group_is_visible(group, &state.hidden_groups, state.isolated_group) && ifc_catalog::matches(mesh.groups.get(group).and_then(|name| state.catalog.get(name)), &state.global_id_filter, &state.storey_filter, &state.discipline_filter)
            });
            let mut faces = visible_faces.take(100_001).map(|(index, triangle)| {
                let depth = triangle.iter().map(|&i| { let p = mesh.vertices[i]; let x = f64::from(p[0])-center[0]; let y = f64::from(p[1])-center[1]; let z = f64::from(p[2])-center[2]; let ry = x*f64::from(sy)+y*f64::from(cy); ry*f64::from(cp)+z*f64::from(sp) }).sum::<f64>() / 3.0;
                (depth, index, triangle)
            }).collect::<Vec<_>>();
            let truncated = faces.len() > 100_000;
            faces.truncate(100_000);
            if truncated { painter.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, "Đang giới hạn 100.000 mặt hiển thị; cần GPU để xem đầy đủ", egui::FontId::proportional(12.0), egui::Color32::YELLOW); }
            faces.sort_by(|a,b| a.0.total_cmp(&b.0));
            let click = response.clicked().then(|| response.interact_pointer_pos()).flatten();
            let mut picked: Option<(f64, usize)> = None;
            let selected_group = state.selected_triangle.and_then(|i| mesh.face_groups.get(i)).copied();
            for (_, index, triangle) in faces {
                let group = mesh.face_groups.get(index).copied().unwrap_or(0);
                if !group_is_visible(group, &state.hidden_groups, state.isolated_group) { continue; }
                let mapped = mesh.groups.get(group).and_then(|name| state.catalog.get(name));
                if !ifc_catalog::matches(mapped, &state.global_id_filter, &state.storey_filter, &state.discipline_filter) { continue; }
                let a = project(&mesh.vertices[triangle[0]]);
                let b = project(&mesh.vertices[triangle[1]]);
                let c = project(&mesh.vertices[triangle[2]]);
                if !triangle_visible_on_screen(a, b, c, rect) { continue; }
                if let Some(pointer) = click {
                    if let Some(weights) = triangle_barycentric(pointer, a, b, c) {
                        // Camera depth is interpolated per pixel, not from the triangle centroid.
                        let depth_at = triangle.iter().zip(weights).map(|(&vertex, weight)| {
                            let p = mesh.vertices[vertex];
                            let x = f64::from(p[0])-center[0]; let y = f64::from(p[1])-center[1]; let z = f64::from(p[2])-center[2];
                            let ry = x*f64::from(sy)+y*f64::from(cy);
                            (ry*f64::from(cp)+z*f64::from(sp))*weight
                        }).sum::<f64>();
                        if depth_at.is_finite() && picked.is_none_or(|(current, _)| depth_at > current) { picked = Some((depth_at, index)); }
                    }
                }
                let selected = state.selected_triangle == Some(index) || (state.select_group && selected_group == Some(group));
                if selected { painter.add(egui::Shape::convex_polygon(vec![a, b, c], egui::Color32::from_rgba_unmultiplied(240, 180, 45, 150), egui::Stroke::new(2.0, egui::Color32::YELLOW))); continue; }
                if state.wireframe {
                    for (u, v) in [(a, b), (b, c), (c, a)] { painter.line_segment([u, v], egui::Stroke::new(0.7, egui::Color32::from_rgb(130, 195, 240))); }
                } else {
                    painter.add(egui::Shape::convex_polygon(vec![a, b, c], egui::Color32::from_rgba_unmultiplied(52, 120, 165, 90), egui::Stroke::new(0.5, egui::Color32::from_rgb(105, 185, 230))));
                }
            }
            if click.is_some() { state.selected_triangle = picked.map(|(_, index)| index); ui.ctx().request_repaint(); }
            if state.show_points { for p in displayed.iter().take(20_000) { let pos = project(p); if rect.contains(pos) { painter.circle_filled(pos, 1.0, egui::Color32::from_rgb(245, 205, 110)); } } }
        } else {
            for p in displayed { let pos = project(p); if rect.contains(pos) { painter.circle_filled(pos, 1.3, egui::Color32::from_rgb(90, 194, 245)); } }
        }
    } else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, if has_ifc_filter && state.mesh.is_none() { "Cần nạp mesh có ánh xạ IFC trước khi lọc cấu kiện" } else { "Không tìm thấy IFCCARTESIANPOINT 3D trong tệp" }, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
    }
    if let (Some(mesh), Some(index)) = (&state.mesh, state.selected_triangle) { if let Some(face) = mesh.faces.get(index) { let group = mesh.face_groups.get(index).and_then(|id| mesh.groups.get(*id)).map(String::as_str).unwrap_or("Mặc định"); ui.label(format!("Đã chọn mặt #{} • nhóm OBJ: {} • đỉnh {:?}", index + 1, group, face)); if let Some(info) = state.catalog.get(group) { ui.label(format!("IFC GlobalId: {} • {} • {} • {}", info.global_id, info.ifc_type, info.storey, info.discipline)); } else { ui.small("Nhóm OBJ chưa có ánh xạ IFC GlobalId đã xác minh."); } } }
    if state.mesh.is_some() {
        ui.collapsing("Quản lý hiển thị nhóm OBJ", |ui| {
            let mesh = state.mesh.as_ref().expect("Mesh đã được kiểm tra");
            let mut unhide = None;
            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                for (group, name) in mesh.groups.iter().enumerate() {
                    let visible = group_is_visible(group, &state.hidden_groups, state.isolated_group);
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", if visible { "●" } else { "○" }, name));
                        if state.hidden_groups.contains(&group) && ui.small_button("Hiện").clicked() { unhide = Some(group); }
                        if state.isolated_group == Some(group) && ui.small_button("Thoát cô lập").clicked() { state.isolated_group = None; }
                    });
                }
            });
            if let Some(group) = unhide { state.hidden_groups.remove(&group); }
        });
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
    fn group_visibility_honors_hide_isolate_and_restore() {
        let mut hidden = BTreeSet::new();
        assert!(group_is_visible(1, &hidden, None));
        hidden.insert(1);
        assert!(!group_is_visible(1, &hidden, None));
        assert!(group_is_visible(2, &hidden, None));
        assert!(!group_is_visible(2, &hidden, Some(1)));
        hidden.remove(&1);
        assert!(group_is_visible(1, &hidden, Some(1)));
        hidden.clear();
        assert!(group_is_visible(2, &hidden, None));
    }

    #[test]
    fn viewport_culling_rejects_nonfinite_and_outside_faces() {
        let view = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0));
        assert!(triangle_visible_on_screen(egui::pos2(10.0, 10.0), egui::pos2(30.0, 10.0), egui::pos2(10.0, 30.0), view));
        assert!(!triangle_visible_on_screen(egui::pos2(120.0, 120.0), egui::pos2(140.0, 120.0), egui::pos2(120.0, 140.0), view));
        assert!(!triangle_visible_on_screen(egui::pos2(f32::NAN, 0.0), egui::pos2(10.0, 0.0), egui::pos2(0.0, 10.0), view));
    }

    #[test]
    fn barycentric_picking_is_stable_at_vertices_and_rejects_outside() {
        let a = egui::pos2(0.0,0.0); let b = egui::pos2(10.0,0.0); let c = egui::pos2(0.0,10.0);
        let w = triangle_barycentric(egui::pos2(2.0,3.0),a,b,c).unwrap();
        assert!((w[0]-0.5).abs() < 1e-6);
        assert!((w[1]-0.2).abs() < 1e-6);
        assert!((w[2]-0.3).abs() < 1e-6);
        assert!(triangle_barycentric(egui::pos2(8.0,8.0),a,b,c).is_none());
        assert!(triangle_barycentric(a,a,a,a).is_none());
    }
    #[test]
    fn triangle_pick_rejects_outside_and_degenerate_faces() {
        let a = egui::pos2(0.0,0.0); let b = egui::pos2(10.0,0.0); let c = egui::pos2(0.0,10.0);
        assert!(point_in_triangle(egui::pos2(2.0,2.0),a,b,c));
        assert!(!point_in_triangle(egui::pos2(9.0,9.0),a,b,c));
        assert!(!point_in_triangle(a,a,a,a));
    }

    #[test]
    fn parses_ifc_3d_point_and_rejects_2d() {
        assert_eq!(point_from_step("#42=IFCCARTESIANPOINT((1.5,-2,3.0));"), Some([1.5,-2.0,3.0]));
        assert!(point_from_step("#1=IFCCARTESIANPOINT((1,2));").is_none());
    }
}
