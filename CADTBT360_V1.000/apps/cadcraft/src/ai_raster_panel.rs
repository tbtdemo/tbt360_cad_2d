//! Native desktop workflow for automatic PNG/JPEG-to-editable-CAD draft lines.
//! Uses the same preview function as the MCP agent: no second vision implementation.

use cadcraft_ui_egui::CadApp;
use serde_json::{Value, json};

/// Preview is computed in a worker. Snapshot prevents applying stale AI geometry to a changed drawing.
struct LocalModelTask {
    receiver: std::sync::mpsc::Receiver<Result<String,String>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    request: String,
    model: String,
}

struct PdfRenderTask {
    receiver: std::sync::mpsc::Receiver<Result<std::path::PathBuf,String>>,
    source: std::path::PathBuf,
    page: u32,
    source_fingerprint: (u64,u64),
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

struct PreviewTask {
    receiver: std::sync::mpsc::Receiver<Result<Value,String>>,
    params: Value,
    uid: Option<u64>,
    revision: Option<u64>,
    epoch: u64,
    source_fingerprint: (u64,u64),
}

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub show_help: bool,
    assistant_instruction: String,
    native_command_input: String,
    local_model: String,
    local_pending: Option<LocalModelTask>,
    local_command: Option<String>,
    local_message: String,
    pdf_path: String,
    pdf_page: u32,
    pdf_pending: Option<PdfRenderTask>,
    pdf_owned_image: Option<std::path::PathBuf>,
    pdf_render_fingerprint: Option<(u64,u64)>,
    path: String,
    preview: Option<Value>,
    preview_task: Option<PreviewTask>,
    preview_source_fingerprint: Option<(u64,u64)>,
    preview_params: Option<Value>,
    preview_document_uid: Option<u64>,
    preview_revision: Option<u64>,
    preview_document_epoch: Option<u64>,
    error: Option<String>,
    min_length: u32,
    max_lines: u32,
    studio_tab: usize,
    advanced_tab: usize,
    scale: f64,
    calibration_pixels: f64,
    calibration_length: f64,
    include_diagonals: bool,
    arbitrary_angles: bool,
    closed_rectangles: bool,
    column_hatch_hints: bool,
    create_column_guides: bool,
    confirm_column_guides: bool,
    create_column_blocks: bool,
    confirm_column_blocks: bool,
    wall_pair_hints: bool,
    wall_panel_hints: bool,
    create_wall_panel_guides: bool,
    confirm_wall_panel_guides: bool,
    create_wall_blocks: bool,
    confirm_wall_blocks: bool,
    min_wall_spacing: f64,
    max_wall_spacing: f64,
    create_wall_centerlines: bool,
    opening_gap_hints: bool,
    paired_wall_opening_hints: bool,
    window_sash_hints: bool,
    create_window_guides: bool,
    confirm_window_guides: bool,
    create_window_blocks: bool,
    confirm_window_blocks: bool,
    door_leaf_hints: bool,
    create_door_swing_arcs: bool,
    confirm_door_swing_arcs: bool,
    create_door_blocks: bool,
    confirm_door_blocks: bool,
    override_door_width: bool,
    door_width_cad: f64,
    selected_door_number: usize,
    selected_door_width_cad: f64,
    door_width_overrides: std::collections::BTreeMap<usize,f64>,
    door_hinge_flips: std::collections::BTreeSet<usize>,
    door_swing_flips: std::collections::BTreeSet<usize>,
    last_door_source_keys: Vec<String>,
    selected_wall_junction: Option<usize>,
    reviewed_wall_junctions: std::collections::BTreeSet<usize>,
    create_opening_guides: bool,
    confirm_opening_guides: bool,
    min_opening_gap: f64,
    max_opening_gap: f64,
}

/// Hit-test a reviewed door's native leaf LINE in the image preview.
/// A close tie is ambiguous and must not change the selected door.
fn preview_door_hit(drafts: &[Value], click: [f64;2], project: impl Fn(f64,f64)->[f64;2], tolerance: f64) -> Option<usize> {
    if !tolerance.is_finite() || tolerance<=0.0 {return None;}
    let mut best: Option<(usize,f64)>=None;
    let mut second=f64::INFINITY;
    for (index,door) in drafts.iter().enumerate() {
        let leaf=&door["leaf"];
        let (Some(x0),Some(y0),Some(x1),Some(y1))=
            (leaf[0][0].as_f64(),leaf[0][1].as_f64(),leaf[1][0].as_f64(),leaf[1][1].as_f64()) else {continue};
        let a=project(x0,y0);let b=project(x1,y1);
        let ab=[b[0]-a[0],b[1]-a[1]];
        let len2=ab[0]*ab[0]+ab[1]*ab[1];
        if !len2.is_finite() || len2<=1e-12 {continue;}
        let t=(((click[0]-a[0])*ab[0]+(click[1]-a[1])*ab[1])/len2).clamp(0.0,1.0);
        let d=(click[0]-a[0]-t*ab[0]).hypot(click[1]-a[1]-t*ab[1]);
        if !d.is_finite() {continue;}
        if best.is_none_or(|(_,old)|d<old) {
            second=best.map_or(f64::INFINITY,|(_,old)|old);
            best=Some((index,d));
        } else if d<second {second=d;}
    }
    best.filter(|(_,distance)|*distance<=tolerance && second-*distance>4.0).map(|(index,_)|index)
}

/// Pick only a unique junction whose non-mutating CAD preview point is close.
fn preview_wall_junction_hit(items: &[Value], click: [f64;2], project: impl Fn(f64,f64)->[f64;2], tolerance: f64) -> Option<usize> {
    if !tolerance.is_finite() || tolerance<=0.0 {return None;}
    let mut nearest=None;
    let mut second=f64::INFINITY;
    for (index,item) in items.iter().enumerate() {
        let (Some(x),Some(y))=(item["pointCAD"][0].as_f64(),item["pointCAD"][1].as_f64()) else {continue;};
        if !x.is_finite()||!y.is_finite(){continue;}
        let p=project(x,y);let d=(p[0]-click[0]).hypot(p[1]-click[1]);
        if !d.is_finite(){continue;}
        if nearest.is_none_or(|(_,old)|d<old) {second=nearest.map_or(f64::INFINITY,|(_,old)|old);nearest=Some((index,d));}
        else if d<second {second=d;}
    }
    nearest.filter(|(_,d)|*d<=tolerance&&second-*d>4.0).map(|(index,_)|index)
}

/// Compute world units per pixel from a known measured dimension.
fn calibrated_units_per_pixel(pixel_distance: f64, cad_length: f64) -> Result<f64, &'static str> {
    if !pixel_distance.is_finite() || !cad_length.is_finite() || pixel_distance <= 0.0 || cad_length <= 0.0 {
        return Err("Khoảng pixel và chiều dài CAD phải là các số hữu hạn lớn hơn 0.");
    }
    let value = cad_length / pixel_distance;
    if !value.is_finite() || !(0.000001..=1_000_000.0).contains(&value) {
        return Err("Tỷ lệ CAD/pixel vượt phạm vi cho phép (0.000001 đến 1000000).");
    }
    Ok(value)
}

impl State {
    pub fn open(&mut self) {
        self.open = true;
        if self.min_length == 0 {
            self.min_length = 16;
            self.max_lines = 500;
            self.local_model = "qwen2.5:7b".into();
            self.native_command_input="vẽ phòng rộng 6000 dài 4000 tại 0,0".into();
            self.scale = 1.0;
            self.include_diagonals = true;
            self.min_wall_spacing = 2.0;
            self.max_wall_spacing = 40.0;
            self.min_opening_gap = 3.0;
            self.max_opening_gap = 40.0;
            self.door_width_cad = 900.0;
            self.selected_door_number = 1;
            self.selected_door_width_cad = 900.0;
        }
    }

    pub fn open_help(&mut self) {
        self.open();
        self.show_help = true;
    }

    fn has_individual_door_review(&self) -> bool {
        !self.door_width_overrides.is_empty() || !self.door_hinge_flips.is_empty() || !self.door_swing_flips.is_empty()
    }

    fn reset_door_reviews_for_new_source(&mut self) {
        self.door_width_overrides.clear();
        self.door_hinge_flips.clear();
        self.door_swing_flips.clear();
        self.last_door_source_keys.clear();
        self.selected_door_number=1;
        self.reset_wall_junction_review();
    }

    fn reset_wall_junction_review(&mut self) {
        self.selected_wall_junction=None;
        self.reviewed_wall_junctions.clear();
    }

    fn matches_document(&self, uid: u64, revision: u64, epoch: u64) -> bool {
        self.preview_document_uid == Some(uid) && self.preview_revision == Some(revision)
            && self.preview_document_epoch == Some(epoch)
    }

    fn params(&self) -> Value {
        let mut params=json!({"path":self.path,"preview":true,"minLengthPixels":self.min_length,
            "unitsPerPixel":self.scale,"includeDiagonals":self.include_diagonals,
            "includeArbitraryAngles":self.arbitrary_angles,"closedRectangles":self.closed_rectangles,
            "columnHatchHints":self.column_hatch_hints,
            "createColumnGuidePolylines":self.create_column_guides,
            "confirmColumnGuides":self.confirm_column_guides,
            "createColumnBlocks":self.create_column_blocks,
            "confirmColumnBlocks":self.confirm_column_blocks});
        let group1=json!({"wallPairHints":self.wall_pair_hints,"createWallCenterlines":self.create_wall_centerlines,
            "wallPanelHints":self.wall_panel_hints,
            "createWallPanelPolylines":self.create_wall_panel_guides,
            "confirmWallPanels":self.confirm_wall_panel_guides,
            "createWallBlocks":self.create_wall_blocks,
            "confirmWallBlocks":self.confirm_wall_blocks,
            "minWallSpacingPixels":self.min_wall_spacing,"maxWallSpacingPixels":self.max_wall_spacing});
        let group2=json!({"openingGapHints":self.opening_gap_hints,
            "pairedWallOpeningHints":self.paired_wall_opening_hints,
            "windowSashHints":self.window_sash_hints,
            "createWindowGuidePolylines":self.create_window_guides,
            "confirmWindowGuides":self.confirm_window_guides,
            "createWindowBlocks":self.create_window_blocks,
            "confirmWindowBlocks":self.confirm_window_blocks});
        let group3=json!({"doorLeafHints":self.door_leaf_hints,
            "createDoorSwingArcs":self.create_door_swing_arcs,
            "confirmDoorSwingArcs":self.confirm_door_swing_arcs,
            "createDoorBlocks":self.create_door_blocks,
            "confirmDoorBlocks":self.confirm_door_blocks});
        let group4=json!({"doorWidthOverrideCAD":if self.create_door_blocks && self.override_door_width {Some(self.door_width_cad)} else {None},
            "doorWidthOverridesCAD":if self.create_door_blocks && !self.door_width_overrides.is_empty() {Some(&self.door_width_overrides)} else {None},
            "doorHingeFlips":if self.create_door_blocks && !self.door_hinge_flips.is_empty() {Some(&self.door_hinge_flips)} else {None},
            "doorSwingFlips":if self.create_door_blocks && !self.door_swing_flips.is_empty() {Some(&self.door_swing_flips)} else {None},
            "expectedDoorSourceKeys":if self.create_door_blocks && self.has_individual_door_review() {Some(&self.last_door_source_keys)} else {None}});
        let group5=json!({"createOpeningGuidePolylines":self.create_opening_guides,
            "confirmOpeningGuides":self.confirm_opening_guides,
            "minOpeningGapPixels":self.min_opening_gap,
            "maxOpeningGapPixels":self.max_opening_gap,"maxLines":self.max_lines});
        for group in [group1,group2,group3,group4,group5] {
            let Value::Object(fields)=group else {unreachable!("AI settings JSON groups are objects")};
            params.as_object_mut().expect("AI settings JSON object").extend(fields);
        }
        if !(self.create_door_blocks && self.override_door_width) {
            params.as_object_mut().expect("AI settings JSON object").remove("doorWidthOverrideCAD");
        }
        if !(self.create_door_blocks && !self.door_width_overrides.is_empty()) {
            params.as_object_mut().expect("AI settings JSON object").remove("doorWidthOverridesCAD");
        }
        for (key,enabled) in [("doorHingeFlips",self.create_door_blocks && !self.door_hinge_flips.is_empty()),
                               ("doorSwingFlips",self.create_door_blocks && !self.door_swing_flips.is_empty()),
                               ("expectedDoorSourceKeys",self.create_door_blocks && self.has_individual_door_review())] {
            if !enabled {params.as_object_mut().expect("AI settings JSON object").remove(key);}
        }
        params
    }
}

/// Prefer the renderer packaged next to CADTBT360.exe; fall back to the user's PATH.
/// A separately configured absolute path is also supported for managed deployments.
fn pdf_renderer() -> std::path::PathBuf {
    if let Some(configured)=std::env::var_os("CAD360_PDF_RENDERER") {
        let custom=std::path::PathBuf::from(configured);
        if custom.is_file() && custom.is_absolute() {return custom;}
    }
    if let Ok(exe)=std::env::current_exe() {
        if let Some(dir)=exe.parent() {
            for relative in ["Tools/Poppler/bin/pdftoppm.exe","Tools/Poppler/pdftoppm.exe","pdftoppm.exe"] {
                let candidate=dir.join(relative);
                if candidate.is_file() {return candidate;}
            }
        }
    }
    std::path::PathBuf::from("pdftoppm")
}

/// Convert one PDF page to PNG with Poppler. Never edits the PDF and does not launch a shell.
/// Fingerprint the PDF bytes, not just its path or last-modified time.
/// A changed page source must never be accepted as the same AI raster job.
fn fingerprint_pdf(pdf:&std::path::Path)->Result<(u64,u64),String> {
    use std::hash::{Hash,Hasher};
    use std::io::Read as _;
    let file=std::fs::File::open(pdf).map_err(|e|format!("Không mở được PDF AI: {e}"))?;
    let meta=file.metadata().map_err(|e|e.to_string())?;
    if !meta.is_file() || meta.len()==0 || meta.len()>100*1024*1024 {
        return Err("PDF đầu vào phải là tệp thường và có dung lượng tối đa 100 MB.".into());
    }
    let mut reader=std::io::BufReader::new(file);
    let mut hash=std::collections::hash_map::DefaultHasher::new();
    let mut size=0u64;
    let mut chunk=[0u8;32768];
    loop {
        let read=reader.read(&mut chunk).map_err(|e|format!("Không đọc được PDF AI: {e}"))?;
        if read==0 {break;}
        size+=read as u64;
        if size>100*1024*1024 {return Err("PDF thay đổi kích thước quá 100 MB.".into());}
        chunk[..read].hash(&mut hash);
    }
    Ok((size,hash.finish()))
}

fn render_pdf_page(pdf: &std::path::Path, page: u32) -> Result<std::path::PathBuf,String> {
    render_pdf_page_cancellable(pdf,page,&std::sync::atomic::AtomicBool::new(false))
}

/// Poppler runs in a worker, but cancellation must actually stop its child process.
fn render_pdf_page_cancellable(pdf:&std::path::Path,page:u32,cancel:&std::sync::atomic::AtomicBool)->Result<std::path::PathBuf,String> {
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Đã hủy chuyển trang PDF trước khi khởi chạy Poppler.".into());
    }
    if !(1..=500).contains(&page) { return Err("Số trang PDF phải từ 1 đến 500.".into()); }
    let metadata=std::fs::metadata(pdf).map_err(|e|format!("Không truy cập được PDF: {e}"))?;
    if !metadata.is_file() || metadata.len()>100*1024*1024 { return Err("PDF phải là tệp thường và không vượt quá 100 MB.".into()); }
    let mut file=std::fs::File::open(pdf).map_err(|e|format!("Không mở được PDF: {e}"))?;
    let mut header=[0u8;5];
    use std::io::Read as _;
    file.read_exact(&mut header).map_err(|e|format!("Không đọc được đầu tệp PDF: {e}"))?;
    if &header!=b"%PDF-" { return Err("Tệp được chọn không có chữ ký PDF hợp lệ.".into()); }
    let unique=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos();
    let prefix=std::env::temp_dir().join(format!("cad360_ai_pdf_{}_{}",std::process::id(),unique));
    let png=prefix.with_extension("png");
    let mut child=std::process::Command::new(pdf_renderer())
        .arg("-f").arg(page.to_string()).arg("-l").arg(page.to_string())
        .arg("-singlefile").arg("-scale-to").arg("2400").arg("-png")
        .arg(pdf).arg(&prefix)
        .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e|format!("Không tìm thấy Poppler pdftoppm ({e}). Cài trong Tools/Poppler/bin cạnh CADTBT360.exe, thiết lập CAD360_PDF_RENDERER hoặc thêm vào PATH."))?;
    let start=std::time::Instant::now();
    let status=loop {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let _=child.kill();let _=child.wait();let _=std::fs::remove_file(&png);
            return Err("Đã hủy chuyển trang PDF và dừng Poppler.".into());
        }
        match child.try_wait() {
            Ok(Some(status))=>break status,
            Ok(None) if start.elapsed()<std::time::Duration::from_secs(60)=>std::thread::sleep(std::time::Duration::from_millis(100)),
            Ok(None)=>{let _=child.kill();let _=child.wait();let _=std::fs::remove_file(&png);
                return Err("Chuyển PDF quá 60 giây; đã dừng tiến trình để tránh treo AI Studio.".into());},
            Err(e)=>{let _=child.kill();let _=child.wait();let _=std::fs::remove_file(&png);
                return Err(format!("Không theo dõi được tiến trình chuyển PDF: {e}"));}
        }
    };
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        let _=std::fs::remove_file(&png);
        return Err("Đã hủy chuyển trang PDF.".into());
    }
    if !status.success() {
        let _=std::fs::remove_file(&png);
        return Err(format!("Không thể chuyển trang PDF {page}. Kiểm tra PDF có mật khẩu, bị hỏng hoặc trang không tồn tại (mã thoát: {status})."));
    }
    let size=std::fs::metadata(&png).map_err(|e|format!("Không tìm thấy ảnh trang PDF: {e}"))?.len();
    if size==0 || size>64*1024*1024 { let _=std::fs::remove_file(&png); return Err("Ảnh PDF tạo ra rỗng hoặc vượt 64 MiB.".into()); }
    Ok(png)
}

/// Rendering can take seconds on large PDFs. Keep Poppler off the egui UI thread.
fn start_pdf_render(state: &mut State, ctx: &egui::Context) {
    if state.pdf_pending.is_some() { return; }
    let path=std::path::PathBuf::from(&state.pdf_path);
    let page=state.pdf_page;
    let source_fingerprint=match fingerprint_pdf(&path) {
        Ok(fingerprint)=>fingerprint,
        Err(error)=>{state.error=Some(error);return;}
    };
    let (sender,receiver)=std::sync::mpsc::channel();
    let ctx=ctx.clone();
    let cancel=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state.pdf_pending=Some(PdfRenderTask{receiver,source:path.clone(),page,source_fingerprint,cancel:cancel.clone()});
    state.path.clear(); // Never reuse an earlier image if a different PDF page fails to render.
    state.pdf_render_fingerprint=None;
    state.preview=None;state.preview_document_uid=None;state.preview_revision=None;state.preview_document_epoch=None;
    state.error=None;
    std::thread::spawn(move || {
        let result=render_pdf_page_cancellable(&path,page,&cancel).and_then(|png| {
            match fingerprint_pdf(&path) {
                Ok(fingerprint) if fingerprint==source_fingerprint && !cancel.load(std::sync::atomic::Ordering::Relaxed)=>Ok(png),
                Ok(_)=>{
                    let _=std::fs::remove_file(&png);
                    Err("Nội dung PDF đã bị sửa trong khi kết xuất. Hãy nạp lại trang.".into())
                },
                Err(error)=>{
                    let _=std::fs::remove_file(&png);
                    Err(error)
                },
            }
        });
        // A closed dialog drops the receiver; never leak an unclaimed temporary raster file.
        if let Err(undelivered)=sender.send(result) {
            if let Ok(png)=undelivered.0 {let _=std::fs::remove_file(png);}
        }
        ctx.request_repaint();
    });
}

fn poll_pdf_render(state: &mut State, ctx: &egui::Context) {
    let Some(task)=state.pdf_pending.as_ref() else {return};
    let result=match task.receiver.try_recv() {
        Ok(result)=>Some(result),
        Err(std::sync::mpsc::TryRecvError::Empty)=>{ctx.request_repaint_after(std::time::Duration::from_millis(200));None},
        Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Luồng chuyển PDF đã kết thúc mà không trả về kết quả.".into())),
    };
    if let Some(result)=result {
        let Some(task)=state.pdf_pending.take() else {return};
        if task.source!=std::path::PathBuf::from(&state.pdf_path) || task.page!=state.pdf_page
            || fingerprint_pdf(&task.source).ok()!=Some(task.source_fingerprint) {
            if let Ok(png)=result {let _=std::fs::remove_file(png);}
            state.error=Some("Trang PDF hoặc tệp nguồn đã thay đổi; cần nạp lại trang đã chọn.".into());
            return;
        }
        match result {
            Ok(png)=>{
                if let Some(old)=state.pdf_owned_image.replace(png.clone()) {let _=std::fs::remove_file(old);}
                state.path=png.to_string_lossy().into_owned();
                state.pdf_render_fingerprint=Some(task.source_fingerprint);
                state.reset_door_reviews_for_new_source();
                state.error=None;
            },
            Err(e)=>{state.error=Some(e);}
        }
    }
}

/// Applies a safe, deterministic AI workflow preset. A text instruction never writes CAD directly.
/// Conservative one-click recovery when a large screenshot produces too
/// many short raster strokes. This only changes detection thresholds; it
/// NEVER accepts a result, issues a CAD command or alters the source image.
fn next_dense_image_filter_min_length(current:u32)->u32 {
    current.max(16).saturating_mul(4).min(4096)
}

fn apply_ai_instruction(state: &mut State, instruction: &str) -> Result<&'static str, &'static str> {
    let q=instruction.trim().to_lowercase();
    if q.is_empty() { return Err("Hãy nhập yêu cầu, ví dụ: nhận dạng toàn bộ mặt bằng, tường, cửa, cột hoặc nét."); }
    let all=["toàn bộ","mặt bằng","floor plan"].iter().any(|word|q.contains(word)) || q.split_whitespace().any(|word|word=="all");
    let wall=all || ["tường","wall"].iter().any(|word|q.contains(word));
    let door=all || ["cửa","door","window"].iter().any(|word|q.contains(word));
    let column=all || ["cột","column"].iter().any(|word|q.contains(word));
    let lines=all || ["nét","line","vector","vẽ lại"].iter().any(|word|q.contains(word));
    if !(wall||door||column||lines) {
        return Err("Yêu cầu chưa được hỗ trợ nội bộ. Hãy chọn toàn bộ mặt bằng, tường, cửa, cột hoặc nét.");
    }
    // Preview-only preset: never enables a destructive commit or bypasses CAD review.
    state.wall_pair_hints=wall||door;
    state.wall_panel_hints=wall||door;
    state.opening_gap_hints=wall||door;
    state.paired_wall_opening_hints=wall||door;
    state.door_leaf_hints=door;
    state.window_sash_hints=door;
    state.column_hatch_hints=column;
    if lines { state.include_diagonals=true; state.arbitrary_angles=true; }
    state.create_column_guides=false; state.confirm_column_guides=false; state.create_column_blocks=false; state.confirm_column_blocks=false;
    state.create_wall_panel_guides=false; state.confirm_wall_panel_guides=false; state.create_wall_blocks=false; state.confirm_wall_blocks=false;
    state.create_door_blocks=false; state.confirm_door_blocks=false; state.create_door_swing_arcs=false; state.confirm_door_swing_arcs=false;
    state.create_window_guides=false; state.confirm_window_guides=false; state.create_window_blocks=false; state.confirm_window_blocks=false;
    state.create_opening_guides=false; state.confirm_opening_guides=false;
    state.closed_rectangles=false;
    state.preview=None; state.preview_document_uid=None; state.preview_revision=None; state.preview_document_epoch=None; state.error=None;
    if all { Ok("Đã chọn phân tích tổng hợp mặt bằng: nét, tường, cửa và cột. Kiểm tra Preview trước khi tạo CAD.") }
    else { Ok("Đã áp dụng những nhóm nhận dạng được nêu trong yêu cầu. Chỉ Preview, không tự sửa bản vẽ.") }
}

/// Streams the source image without holding an additional full copy in RAM.
/// Detects accidental source mutation even when path, length and modified timestamp stay the same.
fn fingerprint_image(path:&str)->Result<(u64,u64),String> {
    use std::hash::{Hash,Hasher};
    use std::io::Read;
    let file=std::fs::File::open(path).map_err(|e|format!("Không đọc được ảnh nguồn AI: {e}"))?;
    let size=file.metadata().map_err(|e|e.to_string())?.len();
    if size==0 || size>64*1024*1024 {return Err("Ảnh nguồn AI phải từ 1 byte đến 64 MiB.".into());}
    let mut reader=std::io::BufReader::new(file);
    let mut hasher=std::collections::hash_map::DefaultHasher::new();
    let mut total=0u64;
    let mut buffer=[0u8;32768];
    loop {
        let n=reader.read(&mut buffer).map_err(|e|format!("Lỗi đọc ảnh nguồn AI: {e}"))?;
        if n==0 {break;}
        total+=n as u64;
        if total>64*1024*1024 {return Err("Ảnh nguồn AI vượt giới hạn 64 MiB.".into());}
        buffer[..n].hash(&mut hasher);
    }
    Ok((total,hasher.finish()))
}

/// The sole approval gate for the source and parameters behind raster-to-CAD output.
/// This rejects a same-name source overwrite, a changed unit scale, or changed detector options.
fn ensure_raster_preview_input_unchanged(state:&State)->Result<(),String> {
    let source=state.preview_source_fingerprint.ok_or("AI chưa có bản xem trước gắn với ảnh nguồn.")?;
    if state.preview_params.as_ref()!=Some(&state.params()) {
        return Err("Thông số AI đã thay đổi sau Preview; hãy nhận dạng lại.".into());
    }
    if let Some(approved_pdf)=state.pdf_render_fingerprint {
        if fingerprint_pdf(std::path::Path::new(&state.pdf_path))?!=approved_pdf {
            return Err("PDF gốc đã bị sửa kể từ khi tạo ảnh Preview; cần chuyển trang PDF và nhận dạng lại.".into());
        }
    }
    let current=fingerprint_image(&state.path)?;
    if current!=source {
        return Err("Nội dung ảnh nguồn đã thay đổi sau Preview; không ghi CAD.".into());
    }
    Ok(())
}

/// Spawn image inference off the UI thread, retaining the source and exact drawing revision.
fn start_image_preview(state:&mut State,cad:&CadApp,ctx:&egui::Context) {
    if state.preview_task.is_some() || state.pdf_pending.is_some() {return;}
    let params=state.params();
    let source=match fingerprint_image(&state.path) {
        Ok(source)=>source,
        Err(error)=>{state.error=Some(error);return;}
    };
    let snapshot=cad.session.state().ok();
    let uid=snapshot.map(|s|s.uid);
    let revision=snapshot.map(|s|s.revision);
    let (tx,rx)=std::sync::mpsc::channel();
    let ctx=ctx.clone();
    let worker_params=params.clone();
    state.preview_task=Some(PreviewTask{receiver:rx,params,uid,revision,epoch:cad.session.doc_epoch,source_fingerprint:source});
    state.preview=None;state.preview_document_uid=None;state.preview_revision=None;state.preview_document_epoch=None;state.error=None;
    state.preview_source_fingerprint=None;state.preview_params=None;
    std::thread::spawn(move || {
        let result=cadcraft_mcp::raster_vector::preview(&worker_params).and_then(|proposal| {
            let path=worker_params["path"].as_str().ok_or("AI thiếu đường dẫn ảnh")?;
            if fingerprint_image(path)?!=source {
                return Err("Ảnh nguồn thay đổi trong khi AI đang nhận dạng; vui lòng Preview lại.".into());
            }
            Ok(proposal)
        });
        let _=tx.send(result);
        ctx.request_repaint();
    });
}

/// Discard inference when parameters, source, or the live CAD document changed during execution.
fn poll_image_preview(state:&mut State,cad:&CadApp,ctx:&egui::Context) {
    let Some(task)=state.preview_task.as_ref() else {return;};
    let result=match task.receiver.try_recv() {
        Ok(result)=>Some(result),
        Err(std::sync::mpsc::TryRecvError::Empty)=>{ctx.request_repaint_after(std::time::Duration::from_millis(150));None},
        Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Bộ nhận dạng AI không trả về kết quả.".into())),
    };
    let Some(result)=result else {return};
    let Some(task)=state.preview_task.take() else {return};
    let current=cad.session.state().ok();
    let same_document=current.map(|s|s.uid)==task.uid && current.map(|s|s.revision)==task.revision
        && cad.session.doc_epoch==task.epoch;
    if !same_document || state.params()!=task.params || fingerprint_image(&state.path).ok()!=Some(task.source_fingerprint) {
        state.preview=None;state.error=Some("Nguồn ảnh, tham số AI hoặc bản vẽ đã thay đổi trong khi nhận dạng. Hãy xem trước lại.".into());
        return;
    }
    match result {
        Ok(value)=>{
            state.last_door_source_keys=value["doorBlockDrafts"].as_array().map(|items|items.iter()
                .filter_map(|door|door["sourceDoorKey"].as_str().map(str::to_owned)).collect()).unwrap_or_default();
            state.preview=Some(value);
            state.preview_source_fingerprint=Some(task.source_fingerprint);
            state.preview_params=Some(task.params);
            state.preview_document_uid=task.uid;
            state.preview_revision=task.revision;
            state.preview_document_epoch=Some(task.epoch);
            state.error=None;
        }
        Err(error)=>{state.preview=None;state.error=Some(error);}
    }
}

/// Optional local model: returns only a proposed command, which is checked by native grammar.
fn start_local_model(state:&mut State,ctx:&egui::Context) {
    if state.local_pending.is_some() {return;}
    let request=state.assistant_instruction.trim().to_owned();
    let model=state.local_model.trim().to_owned();
    if let Err(error)=crate::ai_local_provider::validate_model(&model) {state.local_message=error;return;}
    if request.is_empty() {state.local_message="Hãy nhập yêu cầu trước khi gửi cho AI cục bộ.".into();return;}
    let (sender,receiver)=std::sync::mpsc::channel();
    let context=ctx.clone();
    let cancel=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state.local_pending=Some(LocalModelTask{receiver,cancel:cancel.clone(),request:request.clone(),model:model.clone()});
    state.local_command=None;
    state.local_message="Đang hỏi mô hình Ollama cục bộ, không gửi dữ liệu CAD lên dịch vụ đám mây.".into();
    std::thread::spawn(move || {
        let answer=crate::ai_local_provider::generate(&model,&request,&cancel);
        let _=sender.send(answer);
        context.request_repaint();
    });
}

fn poll_local_model(state:&mut State,ctx:&egui::Context) {
    let Some(task)=state.local_pending.as_ref() else {return;};
    let response=match task.receiver.try_recv() {
        Ok(response)=>Some(response),
        Err(std::sync::mpsc::TryRecvError::Empty)=>{ctx.request_repaint_after(std::time::Duration::from_millis(200));None},
        Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Trình kết nối Ollama không trả kết quả.".into())),
    };
    let Some(response)=response else {return;};
    let Some(task)=state.local_pending.take() else {return;};
    if state.assistant_instruction.trim()!=task.request || state.local_model.trim()!=task.model {
        state.local_command=None;
        state.local_message="Yêu cầu hoặc mô hình đã thay đổi; kết quả AI cũ bị từ chối.".into();
        return;
    }
    match response {
        Ok(command) if cadcraft_ui_egui::ai_review::AiReview::validates_native_command(&command)=>{
            state.local_message="AI đề xuất lệnh CAD hợp lệ. Kiểm tra lệnh rồi mở AI Review; chưa thay đổi bản vẽ.".into();
            state.local_command=Some(command);
        },
        Ok(_)=>{state.local_command=None;state.local_message="AI trả lệnh không thuộc danh sách CAD được phép; đã chặn để bảo vệ bản vẽ.".into();},
        Err(error)=>{state.local_command=None;state.local_message=error;},
    }
}

pub fn window(ctx: &egui::Context, cad: &mut CadApp, state: &mut State) {
    poll_pdf_render(state,ctx);
    poll_image_preview(state,cad,ctx);
    poll_local_model(state,ctx);
    if !state.open { return; }
    let mut open = state.open;
    egui::Window::new("CAD 360  |  AI Studio")
        .open(&mut open).resizable(true).default_width(960.0).min_width(690.0).default_height(660.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("cad360.ai.studio.scroll").auto_shrink([false,false]).show(ui, |ui| {
            ui.heading("AI STUDIO  |  CHUYỂN ẢNH & VẼ CAD");
            ui.small("Nguồn ảnh/PDF → Xem trước → Xác nhận CAD. Mọi thay đổi đều cần phê duyệt.");
            ui.horizontal_wrapped(|ui| {
                for (index,label) in ["01  Ảnh / PDF","02  Vẽ bằng AI","03  Thiết lập nhận dạng","04  Kết nối & Hướng dẫn"].iter().enumerate() {
                    if ui.selectable_label(state.studio_tab==index,*label).clicked() {state.studio_tab=index;}
                }
            });
            ui.separator();

            if state.studio_tab==1 {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.strong("Trợ lý thao tác AI");
                ui.small("Ra lệnh theo các chức năng đang được hỗ trợ; không tự ý ghi hoặc sửa bản vẽ.");
                if ui.add(egui::TextEdit::multiline(&mut state.assistant_instruction).desired_rows(2)
                    .hint_text("Ví dụ: Nhận dạng tường từ ảnh hoặc nhờ AI vẽ phòng 6x4m...")).changed() {
                    state.local_command=None;
                }
                ui.horizontal_wrapped(|ui| {
                    for (label,query) in [("Toàn bộ mặt bằng","nhận dạng toàn bộ mặt bằng"),("Vẽ lại nét","vẽ lại nét"),("Tìm tường","nhận dạng tường"),("Tìm cửa","nhận dạng cửa"),("Tìm cột","nhận dạng cột")] {
                        if ui.button(label).clicked() { state.assistant_instruction=query.into(); state.local_command=None; state.local_message.clear(); }
                    }
                    if ui.button("Áp dụng yêu cầu").clicked() {
                        state.local_command=None;
                        let instruction=state.assistant_instruction.clone();
                        match apply_ai_instruction(state,&instruction) {
                            Ok(message)=>cad.set_status(message.to_owned()),
                            Err(message)=>state.error=Some(message.to_owned()),
                        }
                    }
                });
                ui.small("Yêu cầu chọn chế độ phân tích. Nhấn Nhận dạng và xem trước để thực thi; kiểm tra kết quả trước khi tạo CAD.");
                ui.separator();
                ui.collapsing("AI hội thoại cục bộ — Ollama (không cần API key)",|ui| {
                    ui.small("Mô hình AI Ollama chạy trên máy riêng; không bắt buộc nếu chỉ dùng CAD hoặc vector hóa ảnh.");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Chưa cài Ollama?");
                        ui.hyperlink_to("Tải Ollama chính thức","https://ollama.com/download");
                        ui.label("Sau khi cài, mở Terminal chạy:");
                        ui.code("ollama pull qwen2.5:7b");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mô hình:");
                        if ui.text_edit_singleline(&mut state.local_model).changed() {state.local_command=None;}
                        if ui.add_enabled(state.local_pending.is_none(),egui::Button::new("Hỏi AI cục bộ")).clicked() {
                            start_local_model(state,ctx);
                        }
                    });
                    if state.local_pending.is_some() {
                        ui.horizontal(|ui|{
                            ui.spinner();
                            ui.label("Đang tạo đề xuất lệnh CAD...");
                            if ui.button("Dừng AI").clicked() {
                                if let Some(task)=state.local_pending.take() {
                                    task.cancel.store(true,std::sync::atomic::Ordering::Relaxed);
                                }
                                state.local_command=None;
                                state.local_message="Đã yêu cầu dừng Ollama; lệnh đang chạy sẽ bị kết thúc và không áp dụng vào CAD.".into();
                            }
                        });
                    }
                    if !state.local_message.is_empty() {ui.small(&state.local_message);}
                    if let Some(command)=state.local_command.clone() {
                        ui.label("Lệnh đề xuất — phải kiểm tra trước khi chấp nhận:");
                        match cadcraft_ui_egui::ai_review::AiReview::describe_native_plan(&command) {
                            Ok((instructions,entities,description))=>{
                                ui.strong(format!("{instructions} lệnh → {entities} đối tượng hình học CAD dự kiến"));
                                ui.small(description);
                            },
                            Err(error)=>{ui.colored_label(egui::Color32::RED,format!("Không thể lập kế hoạch CAD: {error}"));}
                        }
                        ui.code(&command);
                        if ui.button("Chuyển lệnh sang AI Review để xem trước").clicked() {
                            cad.ai_review.open_with_prompt(&command);
                            cad.set_status("Đã chuyển lệnh mô hình AI sang AI Review; chưa thực thi.");
                            state.local_command=None;
                            state.open=false; // One focused review window, not two overlapping dialogs.
                        }
                    }
                });
                ui.separator();
                ui.collapsing("Vẽ CAD trực tiếp bằng lệnh (không cần ảnh hoặc mô hình AI)",|ui| {
                    ui.small("Nhập các lệnh hình học có tọa độ và kích thước rõ ràng. Khác với yêu cầu nhận dạng ảnh ở phía trên.");
                    ui.add(egui::TextEdit::multiline(&mut state.native_command_input).desired_rows(2)
                        .desired_width(f32::INFINITY).hint_text("vẽ phòng rộng 6000 dài 4000 tại 0,0 | vẽ hình tròn tâm 3000,2000 bán kính 250"));
                    ui.horizontal_wrapped(|ui| {
                        for (name,example) in [
                            ("Phòng 6 × 4 m","vẽ phòng rộng 6000 dài 4000 tại 0,0"),
                            ("Tường 200","vẽ tường từ 0,0 đến 6000,0 dày 200"),
                            ("Tường và cửa","vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở trái"),
                            ("Phòng + cột tròn","vẽ phòng rộng 6000 dài 4000 tại 0,0 | vẽ hình tròn tâm 3000,2000 bán kính 250"),
                        ] {
                            if ui.button(name).clicked() {state.native_command_input=example.into();}
                        }
                    });
                    match cadcraft_ui_egui::ai_review::AiReview::describe_native_plan(&state.native_command_input) {
                        Ok((commands,entities,_))=>{
                            ui.small(format!("Hợp lệ: {commands} lệnh → {entities} đối tượng CAD. Chưa ghi bản vẽ."));
                        },
                        Err(reason)=>{ui.small(format!("Cần sửa lệnh trước khi Preview: {reason}"));}
                    }
                    if ui.add_enabled(cadcraft_ui_egui::ai_review::AiReview::validates_native_command(&state.native_command_input),
                        egui::Button::new("Xem trước lệnh CAD trong AI Review")).clicked() {
                        cad.ai_review.open_with_prompt(&state.native_command_input);
                        cad.set_status("Đã mở AI Review. Kiểm tra và xác nhận Preview trước khi tạo CAD.");
                        state.open=false; // Give the entire canvas to review instead of overlaying AI Studio.
                    }
                });

            });
            }
            if state.studio_tab==3 {
            ui.heading("Kết nối AI & Tài liệu");
            ui.collapsing("BẮT ĐẦU TỪ SỐ 0 — AI ẢNH → CAD", |ui| {
                ui.label("1. Chỉ dùng ảnh hoặc PDF có MẶT BẰNG, không chụp cả ribbon CAD, bảng Layer, chữ menu; cắt vùng tầng hoặc phân khu nếu nhiều nét.");
                ui.label("2. Mở tab 01 Ảnh/PDF → Chọn ảnh → đặt tỷ lệ CAD/pixel theo kích thước đã biết → Nhận dạng và xem trước.");
                ui.label("3. Nếu báo quá 1.000 nét kiến trúc hoặc 10.000 nét hình học: tăng Nét tối thiểu hoặc cắt ảnh, KHÔNG tăng tùy tiện tỷ lệ CAD để né lỗi.");
                ui.label("4. Đối chiếu các nét, phòng P1/P2, tường, khoảng cửa nghi vấn với ảnh gốc. Khoảng mở chưa được coi là cửa thực tế.");
                ui.label("5. Chỉ khi đã xem trước và xác nhận, mới cho phép tạo LINE/PLINE/BLOCK; lưu bản vẽ mới và dùng UNDO khi cần.");
                ui.small("GIỚI HẠN: đây là dò hình học raster; chưa phải AI ngữ nghĩa đọc hoàn chỉnh mặt bằng kiến trúc.");
            });
            ui.collapsing("LỆNH TIẾNG VIỆT — VÍ DỤ THỰC HIỆN", |ui| {
                ui.label("Gõ ở dòng lệnh dưới bản vẽ (có dấu hoặc không dấu), chọn gợi ý hoặc gõ đủ câu rồi Enter:");
                for (vi,cmd,guide) in [
                    ("làm sạch bản vẽ","PURGE","Chọn các thành phần không dùng để dọn"),
                    ("vẽ đường thẳng","LINE / L","Chọn điểm đầu và điểm tiếp theo"),
                    ("vẽ đa đoạn","PLINE / PL","Chọn các đỉnh, Enter kết thúc"),
                    ("vẽ hình chữ nhật","RECTANG / REC","Chọn hai góc đối diện"),
                    ("đo khoảng cách","DIST / DI","Chọn hai điểm để đọc kết quả"),
                    ("tìm tọa độ","ID","Chọn một điểm đọc X,Y,Z"),
                    ("ghi kích thước","DIMLINEAR / DLI","Chọn hai điểm và vị trí đặt số đo"),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(vi);ui.small(format!("→ {cmd} · {guide}"));
                    });
                }
                ui.small("ĐANG CHỜ hiển thị loại thao tác lệnh đang đợi. F3 bật OSNAP; Esc hủy; Enter kết thúc hoặc xác nhận tùy lệnh.");
            });
            ui.collapsing("AI REVIEW KHÁC AI STUDIO NHƯ THẾ NÀO?", |ui| {
                ui.label("AI Studio (01) đọc ảnh để tạo bản xem trước nhiều nét. AI Review (02) là trình kiểm duyệt câu lệnh CAD đã mô tả rõ tọa độ.");
                ui.label("Ví dụ bản nháp AI Review: vẽ phòng rộng 6000 dài 4000 tại 0,0; xem trước RECTANG trước khi xác nhận.");
                ui.label("Không dùng lời tự do chưa có trong từ điển để trực tiếp tạo hình học; AI Review chưa tự đồng bộ với kết quả nhận dạng ảnh.");
                ui.label("Nếu cần tường + cửa: chọn ví dụ 'Tường và cửa' trong tab 02 và xem tọa độ, độ dày, hướng mở trước khi ghi.");
            });
            ui.collapsing("OSNAP / BẮT ĐIỂM — KHÔNG PHẢI TỌA ĐỘ ƯỚC LƯỢNG", |ui| {
                ui.label("F3: bật bắt điểm thật trên đối tượng; END: đầu mút LINE; MID: trung điểm LINE; CEN: tâm CIRCLE; INT: giao điểm.");
                ui.label("Ví dụ: L → nhấp END của tường A → nhấp MID của tường B → Enter. DIST → nhấp lại hai điểm để đo khoảng cách.");
                ui.label("Mở Options → Drafting → Drafting Settings để xem từng OSNAP có ví dụ. F8 trực giao, F10 dò góc, F12 nhập động.");
            });
            ui.collapsing("Kết nối trợ lý AI ngoài qua MCP", |ui| {
                ui.label("CAD 360 có MCP server kết nối với ứng dụng AI hỗ trợ Model Context Protocol.");
                ui.label("Để AI thao tác trên bản vẽ đang mở, chạy CAD 360 với --control 7979 và kết nối:");
                ui.code("cadcraft-cli mcp --connect 127.0.0.1:7979");
                ui.small("Chỉ cấp quyền cho AI đáng tin cậy; kiểm tra từng thay đổi và sao lưu trước khi cho phép ghi.");
                ui.small("Đây là hướng dẫn sử dụng cầu nối MCP hiện có, không phải kết nối tài khoản ChatGPT/Gemini hoặc đăng nhập OAuth trực tiếp.");
            });
            }
            if state.studio_tab==0 {
            ui.add_space(8.0);
            ui.heading("Nguồn bản vẽ");
            ui.small("Bước 1: chọn ảnh hoặc PDF. Bước 2: kiểm tra tỷ lệ, giới hạn nét. Bước 3: nhận dạng để xem trước, rồi mới xác nhận ghi CAD.");
            ui.small("PNG/JPG hoặc PDF nhiều trang. PDF được đưa về PNG để AI nhận dạng; hỗ trợ Poppler đi kèm bộ cài hoặc công cụ trong PATH.");
            if ui.toggle_value(&mut state.show_help, "📖 Hướng dẫn từng bước").clicked() {
                state.error = None;
            }
            if state.show_help {
                ui.group(|ui| {
                    ui.heading("Cách sử dụng AI từ ảnh → CAD");
                    ui.label("1. Mở một bản sao bản vẽ, chuẩn bị ảnh PNG/JPEG nét đậm và nền sáng.");
                    ui.label("2. Chọn ảnh. Thiết lập Nét tối thiểu (thử 16 px) và tỷ lệ Đơn vị CAD/pixel.");
                    ui.label("3. Chọn Nét 45° nếu cần; Góc bất kỳ là tính năng thử nghiệm.");
                    ui.label("4. Nhấn 'Nhận dạng và xem trước', kiểm tra số lượng nét và kích thước dự kiến.");
                    ui.label("5. Nhấn 'Tạo LINE chỉnh sửa trong CAD', chọn từng nét để kiểm tra và lưu bản vẽ mới.");
                    ui.label("6. Dùng UNDO một lần để hoàn tác cả nhóm AI khi cần.");
                    ui.separator();
                    ui.small("Tài liệu Word đầy đủ: Setup → HuongDan\\HUONG_DAN_AI_TU_ANH_THANH_CAD.docx; Portable → HUONG_DAN_AI_TU_ANH_THANH_CAD.docx.");
                    ui.small("Giới hạn: Không tự hiểu tường, cửa, cột, kích thước. Chưa có xác nhận kiểm thử GUI Windows thủ công.");
                });
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(state.pdf_pending.is_none(),egui::Button::new("Chọn ảnh PNG/JPG…")).clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png","jpg","jpeg"]).pick_file()
                {
                    state.path = path.to_string_lossy().into_owned();
                    state.pdf_path.clear();
                    state.pdf_render_fingerprint=None;
                    if let Some(old)=state.pdf_owned_image.take() {let _=std::fs::remove_file(old);}
                    // Per-door indices refer only to the OLD image's candidate order.
                    // Never transfer reviewed geometry to a newly selected image.
                    state.reset_door_reviews_for_new_source();
                    state.reset_wall_junction_review();
                    state.preview = None;
                    state.preview_document_uid = None;
                    state.preview_revision = None;
                    state.preview_document_epoch = None;
                    state.error = None;
                }
                if ui.add_enabled(state.pdf_pending.is_none(),egui::Button::new("Chọn PDF…")).clicked()
                    && let Some(pdf)=rfd::FileDialog::new().add_filter("PDF",&["pdf"]).pick_file()
                {
                    state.pdf_path=pdf.to_string_lossy().into_owned();state.pdf_page=1;
                    start_pdf_render(state,ctx);
                }
                ui.label(state.path.rsplit(['/', '\\']).next().unwrap_or("(Chưa chọn ảnh)"));
            });
            if !state.pdf_path.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(format!("PDF: {}",std::path::Path::new(&state.pdf_path).file_name().unwrap_or_default().to_string_lossy()));
                    ui.label("Trang:");
                    ui.add_enabled(state.pdf_pending.is_none(),egui::DragValue::new(&mut state.pdf_page).range(1..=500));
                    if ui.add_enabled(state.pdf_pending.is_none(),egui::Button::new("Nạp trang PDF")).clicked() {start_pdf_render(state,ctx);}
                });
            }
            if state.pdf_pending.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();ui.label("Đang chuyển trang PDF ngoài luồng giao diện...");
                    if ui.button("Dừng chuyển PDF").clicked() {
                        if let Some(task)=state.pdf_pending.take() {
                            task.cancel.store(true,std::sync::atomic::Ordering::Relaxed);
                        }
                        state.path.clear();state.preview=None;state.pdf_render_fingerprint=None;
                        state.error=Some("Đã yêu cầu dừng Poppler. Trang PDF chưa được chấp nhận để tạo CAD.".into());
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label("Giới hạn nét tối đa");
                if ui.add(egui::DragValue::new(&mut state.max_lines).speed(10).range(50..=10_000)).changed() { state.preview=None; }
                ui.label("Nét tối thiểu (px)");
                if ui.add(egui::DragValue::new(&mut state.min_length).range(2..=4096)).changed() { state.preview = None; }
                ui.label("Đơn vị CAD/pixel");
                if ui.add(egui::DragValue::new(&mut state.scale).speed(0.01).range(0.000001..=1_000_000.0)).changed() { state.preview = None; }
            });
            if state.max_lines > 1000 {
                ui.small("Chế độ dữ liệu lớn: hỗ trợ tối đa 10.000 nét hình học; một số chức năng nhận dạng phòng/cửa/cột vẫn phải phân vùng để tránh sai và treo máy.");
            }
            }
            if state.studio_tab==2 {
            ui.heading("Thiết lập nhận dạng nâng cao");
            ui.small("Chọn nhóm cài đặt theo hàng ngang. Mỗi thay đổi cần xem trước lại bản vẽ.");
            ui.horizontal_wrapped(|ui|{
                for (index,label) in ["Tỷ lệ & nét","Tường","Cửa / cửa sổ","Cột / PLINE"].iter().enumerate() {
                    if ui.selectable_label(state.advanced_tab==index,*label).clicked() {state.advanced_tab=index;}
                }
            });
            ui.group(|ui| {
            if state.advanced_tab==0 {
            ui.group(|ui| {
                ui.label("Hiệu chuẩn tỷ lệ theo chiều dài đã biết (không tự nhận dạng kích thước):");
                ui.horizontal(|ui| {
                    ui.label("Khoảng cách trên ảnh (pixel)");
                    ui.add(egui::DragValue::new(&mut state.calibration_pixels).speed(1.0).range(0.0..=1_000_000.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Chiều dài thực (đơn vị CAD)");
                    ui.add(egui::DragValue::new(&mut state.calibration_length).speed(1.0).range(0.0..=1_000_000_000.0));
                });
                if ui.button("Tính tỷ lệ từ đoạn chuẩn").clicked() {
                    match calibrated_units_per_pixel(state.calibration_pixels, state.calibration_length) {
                        Ok(value) => {
                            state.scale = value;
                            state.preview = None;
                            state.preview_document_uid = None;
                            state.preview_revision = None;
                            state.preview_document_epoch = None;
                            state.error = None;
                            cad.set_status(format!("Đã hiệu chuẩn AI: 1 pixel = {value:.6} đơn vị CAD. Nhấn Preview lại."));
                        }
                        Err(error) => state.error = Some(error.to_string()),
                    }
                }
                ui.small("Đo khoảng pixel giữa hai điểm tham chiếu trong ảnh; nhập chiều dài thực cùng đơn vị bản vẽ. Tỷ lệ = chiều dài thực / số pixel.");
            });
            }
            ui.vertical(|ui| {
                if state.advanced_tab==0 {
                if ui.checkbox(&mut state.include_diagonals, "Nét 45°").changed() { state.preview = None; }
                if ui.checkbox(&mut state.arbitrary_angles, "Góc bất kỳ (thử nghiệm)").changed() { state.preview = None; }
                }
                if state.advanced_tab==1 {
                if ui.checkbox(&mut state.wall_pair_hints, "Gợi ý cặp nét tường song song (chỉ Preview)").changed() {
                    if !state.wall_pair_hints {
                        state.create_wall_centerlines=false;
                        state.wall_panel_hints=false;
                        state.create_wall_panel_guides=false;
                        state.confirm_wall_panel_guides=false;
                        state.create_wall_blocks=false;
                        state.confirm_wall_blocks=false;
                    }
                    state.preview = None;
                }
                if state.wall_pair_hints {
                    if ui.checkbox(&mut state.wall_panel_hints,"Gợi ý dải tường từ hai mép quan sát được (không tự nối qua cửa)").changed() {
                        if !state.wall_panel_hints {
                            state.create_wall_panel_guides=false;state.confirm_wall_panel_guides=false;
                            state.create_wall_blocks=false;state.confirm_wall_blocks=false;
                        }
                        state.preview=None;
                    }
                    if state.wall_panel_hints {
                        ui.small("Chỉ tìm vùng tường từ hai mép có bằng chứng, bỏ qua vùng trùng khoảng mở/cột.");
                        if ui.checkbox(&mut state.create_wall_panel_guides,"Vẽ PLINE kín dải tường sau khi kiểm tra").changed() {
                            if state.create_wall_panel_guides {state.create_wall_blocks=false;state.confirm_wall_blocks=false;}
                            if !state.create_wall_panel_guides {state.confirm_wall_panel_guides=false;}
                            state.preview=None;
                        }
                        if state.create_wall_panel_guides && ui.checkbox(&mut state.confirm_wall_panel_guides,
                            "Tôi xác nhận tạo PLINE dải tường").changed() {state.preview=None;}
                        if ui.checkbox(&mut state.create_wall_blocks,"Tạo BLOCK CAD từ dải tường được xác nhận").changed() {
                            if state.create_wall_blocks {
                                state.create_wall_panel_guides=false;state.confirm_wall_panel_guides=false;
                                state.closed_rectangles=false;
                            } else {state.confirm_wall_blocks=false;}
                            state.preview=None;
                        }
                        if state.create_wall_blocks {
                            if ui.checkbox(&mut state.confirm_wall_blocks,"Tôi đồng ý tạo BLOCK dải tường sau khi xem Preview").changed() {state.preview=None;}
                            ui.small("BLOCK là bản nháp PLINE có kích thước đo từ ảnh; không phải tường BIM hoặc BLOCK động.");
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.label("Mép tường (px): từ");
                        if ui.add(egui::DragValue::new(&mut state.min_wall_spacing).speed(1.0).range(1.0..=80.0)).changed() { state.preview = None; }
                        ui.label("đến");
                        if ui.add(egui::DragValue::new(&mut state.max_wall_spacing).speed(1.0).range(1.0..=80.0)).changed() { state.preview = None; }
                    });
                    if state.min_wall_spacing > state.max_wall_spacing {
                        ui.colored_label(egui::Color32::RED,"Khoảng cách mép tường: giá trị từ phải nhỏ hơn hoặc bằng giá trị đến.");
                    }
                }
                if ui.add_enabled(state.wall_pair_hints, egui::Checkbox::new(&mut state.create_wall_centerlines, "Tạo tim tường từ cặp nét duy nhất (tùy chọn)")).changed() {
                    state.preview = None;
                }
                }
                if state.advanced_tab==2 {
                if ui.checkbox(&mut state.opening_gap_hints, "Gợi ý khoảng hở có thể là cửa (chỉ Preview)").changed() {
                    if !state.opening_gap_hints {
                        state.paired_wall_opening_hints=false;
                        state.window_sash_hints=false;
                        state.create_window_guides=false;
                        state.confirm_window_guides=false;
                        state.create_window_blocks=false;
                        state.confirm_window_blocks=false;
                        state.door_leaf_hints=false;
                        state.create_door_swing_arcs=false;
                        state.confirm_door_swing_arcs=false;
                        state.create_door_blocks=false;
                        state.confirm_door_blocks=false;
                        state.override_door_width=false;
                        state.door_width_overrides.clear();
                        state.door_hinge_flips.clear();
                        state.door_swing_flips.clear();
                        state.create_opening_guides=false;
                        state.confirm_opening_guides=false;
                    }
                    state.preview=None;
                }
                if state.opening_gap_hints {
                    ui.horizontal(|ui| {
                        ui.label("Khoảng mở (px): từ");
                        if ui.add(egui::DragValue::new(&mut state.min_opening_gap).speed(1.0).range(1.0..=120.0)).changed() { state.preview=None; }
                        ui.label("đến");
                        if ui.add(egui::DragValue::new(&mut state.max_opening_gap).speed(1.0).range(1.0..=120.0)).changed() { state.preview=None; }
                    });
                    if state.min_opening_gap>state.max_opening_gap {
                        ui.colored_label(egui::Color32::RED,"Khoảng mở: ngưỡng từ không được lớn hơn ngưỡng đến.");
                    }
                    if ui.checkbox(&mut state.paired_wall_opening_hints, "AI đối chiếu khoảng mở trên hai mép tường (gợi ý)").changed() {
                        if !state.paired_wall_opening_hints {
                            state.create_opening_guides=false;
                            state.confirm_opening_guides=false;
                            state.window_sash_hints=false;
                            state.create_window_guides=false;
                            state.confirm_window_guides=false;
                            state.create_window_blocks=false;
                            state.confirm_window_blocks=false;
                            state.door_leaf_hints=false;
                            state.create_door_swing_arcs=false;
                            state.confirm_door_swing_arcs=false;
                            state.create_door_blocks=false;
                            state.confirm_door_blocks=false;
                            state.override_door_width=false;
                            state.door_width_overrides.clear();
                            state.door_hinge_flips.clear();
                            state.door_swing_flips.clear();
                        }
                        state.preview=None;
                    }
                    if state.paired_wall_opening_hints {
                        if ui.checkbox(&mut state.window_sash_hints, "AI gợi ý khung cửa sổ (2 nét song song trong khoảng mở)").changed() {
                            if !state.window_sash_hints {state.create_window_guides=false;state.confirm_window_guides=false;state.create_window_blocks=false;state.confirm_window_blocks=false;}
                            state.preview=None;
                        }
                        if state.window_sash_hints {
                            ui.small("Chỉ nhận biết cặp nét khung cửa sổ. Đây là ứng viên hình học, chưa phải Window Block/BIM.");
                            if ui.checkbox(&mut state.create_window_guides,"Vẽ PLINE bao khung cửa sổ được gợi ý").changed() {
                                if state.create_window_guides {state.create_window_blocks=false;state.confirm_window_blocks=false;}
                                if !state.create_window_guides {state.confirm_window_guides=false;}
                                state.preview=None;
                            }
                            if state.create_window_guides && ui.checkbox(&mut state.confirm_window_guides,
                                "Tôi đã xem Preview và đồng ý tạo PLINE khung cửa sổ").changed() {state.preview=None;}
                            if ui.checkbox(&mut state.create_window_blocks,"Tạo Block CAD cửa sổ bản nháp (PLINE khung + 2 LINE)").changed() {
                                if state.create_window_blocks {state.create_window_guides=false;state.confirm_window_guides=false;}
                                else {state.confirm_window_blocks=false;}
                                state.preview=None;
                            }
                            if state.create_window_blocks {
                                if ui.checkbox(&mut state.confirm_window_blocks,"Tôi đồng ý tạo Block cửa sổ sau khi xem Preview").changed() {state.preview=None;}
                                ui.small("Block chỉ là bản nháp hình học CAD; không tự gán loại cửa sổ hay BIM.");
                            }
                        }
                        if ui.checkbox(&mut state.door_leaf_hints, "Gợi ý nét cánh cửa tại góc khoảng mở (chỉ Preview)").changed() {
                            if !state.door_leaf_hints {state.create_door_swing_arcs=false; state.confirm_door_swing_arcs=false; state.create_door_blocks=false; state.confirm_door_blocks=false;}
                            state.preview=None;
                        }
                        ui.small("Tìm đoạn nét vuông góc từ góc mở ra ngoài bề dày tường. Có thể là nét cánh cửa, chưa phải nhận dạng cửa xác thực.");
                        if state.door_leaf_hints {
                            if ui.checkbox(&mut state.create_door_swing_arcs, "Tạo cung quay ARC cho nét cửa đã nhận dạng (tùy chọn)").changed() {
                                if !state.create_door_swing_arcs {state.confirm_door_swing_arcs=false; state.create_door_blocks=false; state.confirm_door_blocks=false;}
                                state.preview=None;
                            }
                            if state.create_door_swing_arcs {
                                if ui.checkbox(&mut state.confirm_door_swing_arcs, "Tôi xác nhận các cung quay ARC này sau khi xem trước").changed() {state.preview=None;}
                                ui.small("Cung quay chỉ là hình học có bằng chứng; chưa được xác nhận là cửa BIM.");
                                if ui.checkbox(&mut state.create_door_blocks, "Gộp cánh LINE + cung ARC thành Block CAD bản nháp").changed() {
                                    if !state.create_door_blocks {
                                        state.confirm_door_blocks=false;state.override_door_width=false;
                                        state.door_width_overrides.clear();state.door_hinge_flips.clear();state.door_swing_flips.clear();
                                    }
                                    state.preview=None;
                                }
                                if state.create_door_blocks {
                                    if ui.checkbox(&mut state.confirm_door_blocks, "Xác nhận tạo Block CAD bản nháp, giữ lại nét ảnh gốc").changed() {state.preview=None;}
                                    if ui.checkbox(&mut state.override_door_width, "Đặt chiều rộng cửa thủ công (CAD units)").changed() {state.preview=None;}
                                    if state.override_door_width {
                                        ui.horizontal(|ui| {
                                            ui.label("Chiều rộng cửa:");
                                            if ui.add(egui::DragValue::new(&mut state.door_width_cad).speed(10.0).range(0.000001..=1_000_000_000.0)).changed() {state.preview=None;}
                                        });
                                        ui.small("Chỉ thay đổi LINE/ARC mới của Block, giữ nguyên các nét tường đã trích từ ảnh.");
                                    }
                                    ui.group(|ui| {
                                        ui.label("Hiệu chỉnh riêng từng cửa trong danh sách AI (số thứ tự bắt đầu từ 1):");
                                        ui.horizontal(|ui| {
                                            ui.label("Cửa số");
                                            ui.add(egui::DragValue::new(&mut state.selected_door_number).range(1..=64).speed(1.0));
                                            ui.label("Chiều rộng (đơn vị CAD)");
                                            ui.add(egui::DragValue::new(&mut state.selected_door_width_cad).speed(10.0).range(0.000001..=1_000_000_000.0));
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.button("Áp dụng cho cửa đã chọn").clicked() {
                                                state.door_width_overrides.insert(state.selected_door_number.saturating_sub(1),state.selected_door_width_cad);
                                                state.preview=None;
                                            }
                                            if ui.button("Xóa hiệu chỉnh cửa này").clicked() {
                                                state.door_width_overrides.remove(&state.selected_door_number.saturating_sub(1));
                                                state.preview=None;
                                            }
                                        });
                                        for (index,width) in state.door_width_overrides.iter().take(12) {
                                            ui.small(format!("Cửa số {}: {:.3} đơn vị CAD (đã hiệu chỉnh)",index+1,width));
                                        }
                                        ui.small("Sau khi áp dụng, nhấn Preview lại. Cửa không chọn giữ chiều rộng gốc hoặc chiều rộng chung nếu đã bật.");
                                        let idx=state.selected_door_number.saturating_sub(1);
                                        ui.horizontal(|ui| {
                                            let mut flip_hinge=state.door_hinge_flips.contains(&idx);
                                            if ui.checkbox(&mut flip_hinge,"Đảo bản lề cửa đã chọn").changed() {
                                                if flip_hinge {state.door_hinge_flips.insert(idx);}
                                                else {state.door_hinge_flips.remove(&idx);}
                                                state.preview=None;
                                            }
                                            let mut flip_swing=state.door_swing_flips.contains(&idx);
                                            if ui.checkbox(&mut flip_swing,"Đảo phía mở cửa đã chọn").changed() {
                                                if flip_swing {state.door_swing_flips.insert(idx);}
                                                else {state.door_swing_flips.remove(&idx);}
                                                state.preview=None;
                                            }
                                        });
                                        ui.small("Đảo bản lề hoặc hướng mở chỉ thay đổi LINE/ARC trong Block bản nháp, không sửa nét từ ảnh.");
                                    });
                                    ui.small("Block tạo bằng LINE và ARC CAD gốc, không phải Block cửa kiến trúc đã kiểm chứng. MCP chưa hỗ trợ commit Block.");
                                }
                            }
                        }
                        if ui.checkbox(&mut state.create_opening_guides, "Cho phép thêm PLINE bao vùng mở đã nhận dạng").changed() {
                            if !state.create_opening_guides { state.confirm_opening_guides=false; }
                            state.preview=None;
                        }
                        if state.create_opening_guides {
                            if ui.checkbox(&mut state.confirm_opening_guides, "Tôi đã kiểm tra và đồng ý tạo PLINE hướng dẫn").changed() {
                                state.preview=None;
                            }
                            ui.small("PLINE chỉ là đường bao kiểm tra, KHÔNG phải Block cửa đã được nhận dạng.");
                        }
                    }
                }
                }
                if state.advanced_tab==3 {
                if ui.checkbox(&mut state.column_hatch_hints,"Gợi ý cột: hình chữ nhật kín + 2 nét giao nhau bên trong").changed() {
                    if !state.column_hatch_hints {state.create_column_guides=false;state.confirm_column_guides=false;state.create_column_blocks=false;state.confirm_column_blocks=false;}
                    state.preview=None;
                }
                if state.column_hatch_hints {
                    ui.small("Chỉ là dấu hiệu hình học; không tự phân loại cột kết cấu hoặc thay thế nét ảnh.");
                    if ui.checkbox(&mut state.create_column_guides,"Thêm PLINE hướng dẫn tại ứng viên cột").changed() {
                        if state.create_column_guides {state.closed_rectangles=false;state.create_column_blocks=false;state.confirm_column_blocks=false;}
                        else {state.confirm_column_guides=false;}
                        state.preview=None;
                    }
                    if state.create_column_guides && ui.checkbox(&mut state.confirm_column_guides,
                        "Tôi đã xem Preview và xác nhận vẽ PLINE cột").changed() {state.preview=None;}
                    if ui.checkbox(&mut state.create_column_blocks,"Tạo BLOCK cột CAD bản nháp (khung + 2 hatch)").changed() {
                        if state.create_column_blocks {state.closed_rectangles=false;state.create_column_guides=false;state.confirm_column_guides=false;}
                        else {state.confirm_column_blocks=false;}
                        state.preview=None;
                    }
                    if state.create_column_blocks && ui.checkbox(&mut state.confirm_column_blocks,
                        "Tôi đã xem Preview và đồng ý tạo BLOCK cột").changed() {state.preview=None;}
                }
                if ui.checkbox(&mut state.closed_rectangles, "Đường bao chữ nhật kín → PLINE").changed() {
                    if state.closed_rectangles {
                        state.create_column_guides=false;state.confirm_column_guides=false;
                        state.create_column_blocks=false;state.confirm_column_blocks=false;
                        state.create_wall_blocks=false;state.confirm_wall_blocks=false;
                    }
                    state.preview = None; state.preview_document_uid = None;
                    state.preview_revision = None; state.preview_document_epoch = None;
                }
                }
            });
            });
            }
            if state.studio_tab==0 {
            ui.separator();
            ui.heading("Nhận dạng và kiểm tra kết quả");
            if ui.add_enabled(state.pdf_pending.is_none() && state.preview_task.is_none() && !state.path.is_empty(),
                egui::Button::new("PHÂN TÍCH MẶT BẰNG KIẾN TRÚC - Tường / Cửa / Cột / Phòng")).clicked() {
                let _=apply_ai_instruction(state,"nhận dạng toàn bộ mặt bằng");
                // Architectural candidates still have a stricter bounded workload.
                state.max_lines=state.max_lines.max(500);
                state.reset_wall_junction_review();
                start_image_preview(state,cad,ctx);
            }
            ui.small("Một lần nhấn chạy đầy đủ nhóm dò hình học. Hệ thống chỉ đề xuất; mọi cấu kiện và phép ghi CAD phải qua Preview.");

            if ui.add_enabled(state.pdf_pending.is_none() && state.preview_task.is_none() && !state.path.is_empty(),
                egui::Button::new("1. Nhận dạng và xem trước")).clicked() {
                state.reset_wall_junction_review();
                start_image_preview(state,cad,ctx);
            }
            if state.preview_task.is_some() {
                ui.horizontal(|ui| {ui.spinner();ui.label("AI đang phân tích ảnh và dựng bản nháp CAD. Bạn vẫn có thể thao tác trên giao diện.");});
            }
            if let Some(error)=state.error.clone() {
                ui.colored_label(egui::Color32::RED,&error);
                let dense_architecture=error.contains("architectural candidate detection is bounded to 1000 strokes");
                let dense_geometry=error.contains("exceeding maxLines");
                if dense_architecture || dense_geometry {
                    ui.label("Ảnh quá nhiều nét cho bộ phân tích hiện tại. Đây KHÔNG phải kết quả AI đã vẽ hoặc hiểu mặt bằng; Preview chưa được tạo.");
                    ui.small("Ảnh chụp cả phần mềm CAD (ribbon, bảng Layer, chữ, icon) tạo rất nhiều nét nhiễu. Hãy dùng ảnh/PDF chỉ chứa vùng mặt bằng, không chụp cả giao diện. Tốt nhất cắt vùng từng tầng hoặc từng khu có đúng tỷ lệ.");
                    ui.small("Để thử giảm nhiễu, có thể tăng độ dài nét tối thiểu. Các chi tiết cửa, tường ngắn có thể bị mất; phải kiểm duyệt lại, không tự ghi CAD.");
                    ui.horizontal_wrapped(|ui| {
                        if ui.add_enabled(state.min_length<4096 && state.preview_task.is_none(),
                            egui::Button::new("Lọc mạnh nét ngắn (x4) và thử lại")).clicked() {
                            state.min_length=next_dense_image_filter_min_length(state.min_length);
                            state.max_lines=10_000;
                            state.reset_wall_junction_review();
                            start_image_preview(state,cad,ctx);
                        }
                        if dense_geometry && state.max_lines<10_000
                            && ui.button("Tăng giới hạn nét hình học đến 10.000").clicked() {
                            state.max_lines=10_000;
                            state.reset_wall_junction_review();
                            start_image_preview(state,cad,ctx);
                        }
                    });
                    ui.small(format!("Đang thiết lập: nét tối thiểu {} px; giới hạn hiển thị {} nét. Phân tích kiến trúc vẫn giới hạn 1.000 nét và chưa có mô hình AI ngữ nghĩa.",state.min_length,state.max_lines));
                }
            }
            let Some(proposal) = &state.preview else { return; };
            let count = proposal["strokes"].as_u64().unwrap_or(0);
            ui.separator();
            let rectangle_count = proposal["closedPolylines"].as_array().map_or(0,Vec::len);
            let room_count=proposal["roomCandidateCount"].as_u64().unwrap_or(0);
            let column_hint_count=proposal["columnHatchCandidateCount"].as_u64().unwrap_or(0);
            let column_block_count=proposal["columnBlockDraftCount"].as_u64().unwrap_or(0);
            let wall_pairs = proposal["wallPairCandidateCount"].as_u64().unwrap_or(0);
            let wall_panel_count=proposal["wallPanelCandidateCount"].as_u64().unwrap_or(0);
            let wall_block_count=proposal["wallBlockDraftCount"].as_u64().unwrap_or(0);
            let wall_boundary_contacts=proposal["wallPanelContactCount"].as_u64().unwrap_or(0);
            let wall_junctions=proposal["wallPanelJunctionCount"].as_u64().unwrap_or(0);
            let opening_count = proposal["openingGapCandidateCount"].as_u64().unwrap_or(0);
            let paired_opening_count = proposal["pairedWallOpeningCandidateCount"].as_u64().unwrap_or(0);
            let door_leaf_count = proposal["doorLeafCandidateCount"].as_u64().unwrap_or(0);
            let window_sash_count = proposal["windowSashCandidateCount"].as_u64().unwrap_or(0);
            let window_block_count=proposal["windowBlockDraftCount"].as_u64().unwrap_or(0);
            let door_arcs_count = proposal["doorSwingArcsProposed"].as_u64().unwrap_or(0);
            let door_blocks_count = proposal["doorBlockDraftCount"].as_u64().unwrap_or(0);
            ui.label(format!("Đã nhận dạng: {count} nét nguồn, {rectangle_count} PLINE kín, {wall_pairs} cặp nét song song nghi là mép tường."));
            ui.strong(format!("PHÂN TÍCH MẶT BẰNG: {room_count} vùng khép kín nghi là phòng · {wall_panel_count} dải tường · {paired_opening_count} khoảng mở · {column_hint_count} ứng viên cột"));
            if let Some(tiles)=proposal["rasterTiles"].as_array() {
                let populated=tiles.iter().filter(|tile|
                    tile["vectorCount"].as_u64().unwrap_or(0)>0).count();
                let overfull=tiles.iter().filter(|tile|
                    tile["indicesTruncated"]==true).count();
                ui.small(format!("Phân vùng ảnh 1024 px: {} vùng, {} vùng có nét · tọa độ toàn cục giữ nguyên",
                    tiles.len(),populated));
                if overfull>0 {
                    ui.colored_label(egui::Color32::YELLOW,
                        format!("{overfull} vùng có >256 nét giao biên; danh sách chỉ số chi tiết bị giới hạn để tránh nặng giao diện."));
                }
                ui.small("Phân vùng là số liệu kiểm soát nguồn; CHƯA chứng minh AI đã hiểu cửa, cột và phòng phức tạp.");
            }
            if room_count>0 {
                ui.small("Phòng chỉ là giả thuyết từ 4 cạnh khép kín quan sát được; chưa tự xác định tên phòng hoặc công năng.");
                let adjacent=proposal["roomAdjacencyCount"].as_u64().unwrap_or(0);
                ui.small(format!("Có {adjacent} cặp vùng chung cạnh tường NGUỒN thực tế; không suy diễn cửa ra vào hoặc lối thông."));
                let contact_count=proposal["openingRoomContactCount"].as_u64().unwrap_or(0);
                ui.small(format!("{contact_count} khoảng mở có hai mép tường gần biên phòng: chỉ là ứng viên cần duyệt, KHÔNG phải cửa đi đã xác nhận."));
                if let Some(contacts)=proposal["openingRoomContacts"].as_array() {
                    for contact in contacts.iter().take(12) {
                        let rooms=contact["roomIndices"].as_array()
                            .map(|v|v.iter().filter_map(Value::as_u64)
                                .map(|i|(i+1).to_string()).collect::<Vec<_>>().join(", "))
                            .unwrap_or_default();
                        let opening=contact["openingIndex"].as_u64().unwrap_or(0)+1;
                        ui.small(format!("Khoảng mở #{opening} gần vùng [{rooms}] · chưa xác nhận loại cửa/lối thông"));
                    }
                }
                if let Some(links)=proposal["roomAdjacencies"].as_array() {
                    for link in links.iter().take(12) {
                        let a=link["roomIndices"][0].as_u64().unwrap_or(0)+1;
                        let b=link["roomIndices"][1].as_u64().unwrap_or(0)+1;
                        let edge=link["sourceEdgeIndex"].as_u64().unwrap_or(0);
                        ui.small(format!("Vùng {a} ↔ {b}: cùng nét nguồn #{edge} · chưa xác nhận lối thông"));
                    }
                }
                if let Some(rooms)=proposal["roomCandidates"].as_array() {
                    for (index,room) in rooms.iter().take(12).enumerate() {
                        let w=room["widthCAD"].as_f64().unwrap_or(0.0);
                        let h=room["depthCAD"].as_f64().unwrap_or(0.0);
                        ui.small(format!("Vùng {} · {:.1} × {:.1} đơn vị CAD · chờ xác nhận",index+1,w,h));
                    }
                }
            }
            if state.column_hatch_hints {
                ui.small(format!("Có {column_hint_count} ứng viên hình chữ nhật có dấu gạch chéo chữ thập bên trong; chưa xác nhận là cột."));
                ui.small("Ứng viên cột hiện viền vàng trên Preview. Chỉ tạo PLINE/BLOCK sau khi xác nhận.");
                if state.create_column_blocks {ui.small(format!("Dự kiến {column_block_count} BLOCK CAD cột bản nháp."));}
            }
            if state.wall_pair_hints {
                ui.small("Gợi ý hình học, không phải tường BIM. Nét nguồn luôn được giữ để đối chiếu.");
                if state.wall_panel_hints {
                    ui.small(format!("Có {wall_panel_count} dải tường quan sát được; đường bao Preview màu cam, không tự lấp khoảng cửa."));
                    ui.small(format!("QA đầu mút: {wall_boundary_contacts} lần chạm ứng viên khoảng mở/cột; CHỈ cảnh báo, không nối hoặc cắt tường."));
                    ui.small(format!("QA nút giao: {wall_junctions} vị trí nghi giao T/X; KHÔNG nối, cắt hay tự kéo dài tường."));
                    if let Some(junctions)=proposal["wallPanelJunctionCandidates"].as_array() {
                        ui.small("Chọn nút giao từ danh sách hoặc nhấn trực tiếp vào dấu tròn trên Preview. Chọn và đánh dấu đã kiểm tra KHÔNG chỉnh sửa CAD.");
                        for (index,j) in junctions.iter().take(30).enumerate() {
                            let a=j["panelIndices"][0].as_u64().unwrap_or(0)+1;
                            let b=j["panelIndices"][1].as_u64().unwrap_or(0)+1;
                            let reviewed=state.reviewed_wall_junctions.contains(&index);
                            let selected=state.selected_wall_junction==Some(index);
                            let title=format!("{} Nút {}: {} · tường {} và {}",if reviewed {"✓"} else if selected {"●"} else {"○"},index+1,j["kind"].as_str().unwrap_or("?"),a,b);
                            if ui.selectable_label(selected,title).clicked() {state.selected_wall_junction=Some(index);}
                        }
                        if let Some(index)=state.selected_wall_junction.filter(|&i|i<junctions.len()) {
                            ui.horizontal(|ui| {
                                if ui.button("Xác nhận đã kiểm tra hình học (không vẽ)").clicked() {state.reviewed_wall_junctions.insert(index);}
                                if ui.button("Bỏ dấu đã kiểm tra").clicked() {state.reviewed_wall_junctions.remove(&index);}
                            });
                            let candidate=&junctions[index];
                            let distances=candidate["endpointDistancesPixels"].as_array().map(|values|values.iter().filter_map(Value::as_f64).map(|v|format!("{v:.2}")).collect::<Vec<_>>().join(" / ")).unwrap_or_else(||"chưa xác định".to_string());
                            let advice=match candidate["proposedEditAction"].as_str().unwrap_or("") {
                                "review-endpoint-snap-without-edit"=>"Đầu tường sát nút giao; kiểm tra sai lệch và phương án Snap thủ công",
                                "review-crossing-overlap-without-edit"=>"Hai tim tường cắt nhau; kiểm tra phần giao và ưu tiên tường chính trước khi Trim",
                                _=>"Chưa đủ dữ liệu để đề xuất chỉnh sửa, cần kiểm tra thủ công"
                            };
                            ui.small(format!("Khoảng cách gần nhất từ các đầu tường đến nút: {distances} px."));
                            ui.small(advice);
                            let edits=candidate["editPreview"].as_array();
                            let edits_count=edits.map_or(0,Vec::len);
                            ui.small(format!("Phương án trước/sau: {edits_count} đầu tường đủ điều kiện xem thử, không áp dụng tự động."));
                            if let Some(edits)=edits {
                                for edit in edits.iter().take(4) {
                                    let panel=edit["panelIndex"].as_u64().unwrap_or(0)+1;
                                    let distance=edit["distanceCAD"].as_f64().unwrap_or(0.0);
                                    ui.small(format!("Tường {panel}: dự kiến dịch đầu gần nút {distance:.4} đơn vị CAD; không ảnh hưởng bản vẽ."));
                                }
                            }
                            ui.small(format!("Nút {}: chỉ đề xuất kiểm tra, canAutoApply=false; không tự Trim/Extend hoặc gộp BLOCK.",index+1));
                        }
                    }
                    if let Some(rows)=proposal["wallPanelBoundaryContacts"].as_array() {
                        for item in rows.iter().take(12) {
                            let index=item["panelIndex"].as_u64().unwrap_or(0)+1;
                            let start=item["startContacts"].as_array().map_or(0,Vec::len);
                            let end=item["endContacts"].as_array().map_or(0,Vec::len);
                            if start+end>0 {
                                ui.small(format!("Tường {index}: đầu 1 có {start} tiếp xúc; đầu 2 có {end} tiếp xúc. Cần đối chiếu ảnh gốc."));
                            }
                        }
                    }
                }
                if state.create_wall_blocks {
                    ui.small(format!("Dự kiến {wall_block_count} BLOCK dải tường CAD; mỗi Block là một PLINE kín chỉnh sửa được."));
                    if let Some(drafts)=proposal["wallBlockDrafts"].as_array() {
                        for (i,draft) in drafts.iter().take(12).enumerate() {
                            ui.small(format!("Tường {}: dài {:.2}, dày {:.2} đơn vị CAD (chưa xác minh).",
                                i+1,draft["lengthCAD"].as_f64().unwrap_or(0.0),draft["thicknessCAD"].as_f64().unwrap_or(0.0)));
                        }
                    }
                }
            }
            if state.opening_gap_hints {
                ui.small(format!("{opening_count} khoảng hở tiềm năng. Chưa xác nhận là cửa đi/cửa sổ; không tự đóng đoạn."));
                if let Some(gaps)=proposal["openingGapCandidates"].as_array() {
                    for (index,gap) in gaps.iter().take(12).enumerate() {
                        let px=gap["gapPixels"].as_f64().unwrap_or(0.0);
                        let cad_width=gap["widthCAD"].as_f64().unwrap_or(0.0);
                        ui.small(format!("Khoảng {}: {:.2} px = {:.3} đơn vị CAD (chưa xác minh).",index+1,px,cad_width));
                    }
                    if gaps.len()>12 {ui.small(format!("Còn {} khoảng khác; xem kết quả MCP chi tiết.",gaps.len()-12));}
                }
            }
            if state.paired_wall_opening_hints {
                ui.label(format!("Đối chiếu hai mép tường: {paired_opening_count} vùng mở hình học cần xác nhận (không tự nhận là cửa)."));
                if state.window_sash_hints {ui.small(format!("{window_sash_count} vùng có hai nét khung giống cửa sổ (chưa phân loại chắc chắn)."));}
                if state.create_window_blocks {ui.small(format!("Dự kiến {window_block_count} Block CAD cửa sổ (khung PLINE + 2 thanh LINE) sau xác nhận."));}
                if state.door_leaf_hints {
                    ui.small(format!("{door_leaf_count} vùng có nét giống cánh cửa vuông góc. Chỉ là dấu hiệu ảnh; không tự vẽ Block cửa."));
                    if state.create_door_swing_arcs {ui.small(format!("Dự kiến {door_arcs_count} cung quay ARC chỉnh sửa được, sau xác nhận."));}
                    if state.create_door_blocks {
                        ui.small(format!("Dự kiến {door_blocks_count} Block cửa dạng hình học nháp, không nhận dạng ngữ nghĩa."));
                        if state.override_door_width {ui.small(format!("Chiều rộng chung: {:.3} đơn vị CAD.",state.door_width_cad));}
                        ui.small(format!("Có {} cửa được hiệu chỉnh chiều rộng riêng.",state.door_width_overrides.len()));
                        ui.small("Có thể NHẤP TRỰC TIẾP vào nét cánh cửa màu tím trên hình Preview để chọn cửa. Nhấp vào vị trí giữa hai cửa quá gần sẽ không tự chọn.");
                        if let Some(drafts)=proposal["doorBlockDrafts"].as_array() {
                            for (index,draft) in drafts.iter().take(24).enumerate() {
                                let width=draft["widthCAD"].as_f64().unwrap_or(0.0);
                                ui.horizontal(|ui| {
                                    if ui.small_button(format!("Chọn cửa {}",index+1)).clicked() {
                                        state.selected_door_number=index+1;
                                        state.selected_door_width_cad=width;
                                    }
                                    ui.small(format!("Rộng {:.3} CAD; gốc {:.3} CAD{}",
                                        width,draft["sourceOpeningWidthCAD"].as_f64().unwrap_or(width),
                                        if draft["widthOverridden"]==true {" (đã sửa)"} else {""}));
                                });
                            }
                        }
                    }
                }
                if let Some(items)=proposal["pairedWallOpeningCandidates"].as_array() {
                    for (index,item) in items.iter().take(8).enumerate() {
                        let t=item["wallThicknessCAD"].as_f64().unwrap_or(0.0);
                        ui.small(format!("Vùng {}: bề dày {:.3} đơn vị CAD; hai mép có khoảng hở tương ứng.{}",index+1,t,
                            if item["doorLeafCandidate"]==true {" Có thêm nét giống cánh cửa."} else {""}));
                    }
                }
            }
            let centerline_count = proposal["wallCenterlinesProposed"].as_u64().unwrap_or(0);
            let junction_count = proposal["wallCenterlineJunctionCount"].as_u64().unwrap_or(0);
            if state.create_wall_centerlines {
                ui.label(format!("Dự kiến tạo thêm {centerline_count} LINE tim tường, không xóa hai mép tường. Cặp 3 nét nhập nhằng bị loại."));
                ui.small(format!("Phát hiện {junction_count} ứng viên giao tim tường T/chữ thập. Chỉ kiểm tra, không tự kéo dài hoặc cắt tường."));
            }
            if state.closed_rectangles {
                ui.small("Chỉ nhận dạng hình chữ nhật có đủ 4 cạnh; không tự lấp cửa hoặc khoảng hở. Cạnh thuộc PLINE không tạo LINE trùng.");
            }
            if let Some(lines) = proposal["lines"].as_array() {
                let polygons = proposal["closedPolylines"].as_array();
                let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width().max(200.0), 220.0), egui::Sense::click());
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
                for polygon in polygons.into_iter().flatten() {
                    for point in polygon.as_array().into_iter().flatten() {
                        if let (Some(x), Some(y)) = (point[0].as_f64(), point[1].as_f64()) {
                            min_x = min_x.min(x); max_x = max_x.max(x);
                            min_y = min_y.min(y); max_y = max_y.max(y);
                        }
                    }
                }
                // A reviewed door may be wider or hinge-flipped relative to the PNG:
                // include both its leaf and full swing radius in the Preview viewport.
                if state.create_door_blocks {
                    if let Some(doors)=proposal["doorBlockDrafts"].as_array() {
                        for door in doors {
                            let arc=&door["arc"];
                            if let (Some(cx),Some(cy),Some(radius))=(arc["center"][0].as_f64(),arc["center"][1].as_f64(),arc["radius"].as_f64()) {
                                min_x=min_x.min(cx-radius);max_x=max_x.max(cx+radius);
                                min_y=min_y.min(cy-radius);max_y=max_y.max(cy+radius);
                            }
                        }
                    }
                }
                if min_x.is_finite() && min_y.is_finite() {
                    let span_x = (max_x - min_x).max(1.0e-9);
                    let span_y = (max_y - min_y).max(1.0e-9);
                    let factor = (f64::from(rect.width()) * 0.90 / span_x)
                        .min(f64::from(rect.height()) * 0.90 / span_y);
                    let to_screen = |x: f64, y: f64| egui::pos2(
                        (f64::from(rect.center().x) + ((x - (min_x + max_x) * 0.5) * factor)) as f32,
                        (f64::from(rect.center().y) - ((y - (min_y + max_y) * 0.5) * factor)) as f32);
                    if state.wall_panel_hints && response.clicked() && !state.create_door_blocks {
                        if let (Some(click),Some(junctions))=(response.interact_pointer_pos(),proposal["wallPanelJunctionCandidates"].as_array()) {
                            let project=|x:f64,y:f64|{let p=to_screen(x,y);[f64::from(p.x),f64::from(p.y)]};
                            if let Some(index)=preview_wall_junction_hit(junctions,[f64::from(click.x),f64::from(click.y)],project,12.0) {
                                state.selected_wall_junction=Some(index);
                                cad.set_status(format!("AI Preview: đã chọn nút T/X số {} (chỉ QA, chưa sửa hình học).",index+1));
                            }
                        }
                    }
                    if state.create_door_blocks && response.clicked() {
                        if let (Some(click),Some(drafts))=(response.interact_pointer_pos(),proposal["doorBlockDrafts"].as_array()) {
                            let project=|x:f64,y:f64|{let p=to_screen(x,y);[f64::from(p.x),f64::from(p.y)]};
                            if let Some(index)=preview_door_hit(drafts,[f64::from(click.x),f64::from(click.y)],project,13.0) {
                                state.selected_door_number=index+1;
                                state.selected_door_width_cad=drafts[index]["widthCAD"].as_f64().unwrap_or(state.door_width_cad);
                                cad.set_status(format!("AI Preview: đã chọn cửa số {}. Có thể chỉnh chiều rộng, bản lề và phía mở.",index+1));
                            }
                        }
                    }
                    for segment in lines {
                        let (Some(a), Some(b)) = (segment.get(0), segment.get(1)) else { continue };
                        let (Some(x0), Some(y0), Some(x1), Some(y1)) =
                            (a[0].as_f64(), a[1].as_f64(), b[0].as_f64(), b[1].as_f64()) else { continue };
                        painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                            egui::Stroke::new(1.5, egui::Color32::LIGHT_GREEN));
                    }
                    for polygon in polygons.into_iter().flatten() {
                        let Some(points) = polygon.as_array() else { continue };
                        if points.len() < 3 { continue; }
                        for idx in 0..points.len() {
                            let a = &points[idx]; let b = &points[(idx+1)%points.len()];
                            let (Some(x0),Some(y0),Some(x1),Some(y1)) =
                                (a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) else { continue };
                            painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                egui::Stroke::new(2.5, egui::Color32::from_rgb(100,190,255)));
                        }
                    }
                    // Lime room boundaries are READ-ONLY candidates detected
                    // from four observed original walls. Distinguish these from
                    // blue closed PLINEs that the operator opted to create.
                    if let Some(rooms)=proposal["roomCandidates"].as_array() {
                        for (room_idx,room) in rooms.iter().take(128).enumerate() {
                            let Some(vertices)=room["outlineCAD"].as_array() else {continue};
                            if vertices.len()!=4 {continue;}
                            let Some(points)=vertices.iter().map(|p|{
                                let x=p[0].as_f64()?;let y=p[1].as_f64()?;
                                (x.is_finite() && y.is_finite()).then_some(to_screen(x,y))
                            }).collect::<Option<Vec<_>>>() else {continue};
                            for i in 0..4 {
                                painter.line_segment([points[i],points[(i+1)%4]],
                                    egui::Stroke::new(2.0,egui::Color32::from_rgb(160,230,110)));
                            }
                            let center=egui::pos2(
                                (points[0].x+points[2].x)*0.5,
                                (points[0].y+points[2].y)*0.5);
                            painter.text(center,egui::Align2::CENTER_CENTER,
                                format!("P{}",room_idx+1),
                                egui::FontId::proportional(12.0),
                                egui::Color32::from_rgb(190,245,155));
                        }
                    }
                    // Orange outlines show paired-wall opening candidates in real CAD
                    // coordinates, including the chosen image origin and scale.
                    if state.paired_wall_opening_hints {
                        if let Some(openings)=proposal["pairedWallOpeningCandidates"].as_array() {
                            for opening in openings {
                                let Some(vertices)=opening["outlineCAD"].as_array() else {continue};
                                for index in 0..vertices.len() {
                                    let a=&vertices[index]; let b=&vertices[(index+1)%vertices.len()];
                                    if let (Some(x0),Some(y0),Some(x1),Some(y1))=
                                        (a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) {
                                        painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                            egui::Stroke::new(2.0,egui::Color32::from_rgb(255,187,82)));
                                    }
                                }
                            }
                        }
                    }
                    // Teal dashed-like rectangular guides only for window-sash candidates,
                    // without claiming a verified architectural Window Block.
                    if state.window_sash_hints {
                        if let Some(openings)=proposal["pairedWallOpeningCandidates"].as_array() {
                            for opening in openings {
                                if opening["windowSashCandidate"]!=true {continue;}
                                let Some(points)=opening["outlineCAD"].as_array() else {continue};
                                for i in 0..points.len() {
                                    let a=&points[i];let b=&points[(i+1)%points.len()];
                                    if let (Some(x0),Some(y0),Some(x1),Some(y1))=
                                        (a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) {
                                        painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                            egui::Stroke::new(3.0,egui::Color32::from_rgb(70,225,207)));
                                    }
                                }
                            }
                        }
                    }
                    if state.column_hatch_hints {
                        if let Some(columns)=proposal["columnHatchCandidates"].as_array() {
                            for col in columns {
                                let Some(points)=col["outlineCAD"].as_array() else {continue};
                                if points.len()!=4 {continue;}
                                for i in 0..4 {
                                    let a=&points[i];let b=&points[(i+1)%4];
                                    if let (Some(x0),Some(y0),Some(x1),Some(y1))=
                                        (a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) {
                                        painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                            egui::Stroke::new(3.0,egui::Color32::from_rgb(251,210,70)));
                                    }
                                }
                            }
                        }
                    }
                    // Orange outlines are observed two-edge wall panels only.
                    // A visible gap remains a gap; there is no automatic closure.
                    if state.wall_panel_hints {
                        if let Some(panels)=proposal["wallPanelCandidates"].as_array() {
                            for panel in panels {
                                let Some(points)=panel["outlineCAD"].as_array() else {continue};
                                if points.len()!=4 {continue;}
                                for i in 0..4 {
                                    let a=&points[i];let b=&points[(i+1)%4];
                                    if let (Some(x0),Some(y0),Some(x1),Some(y1))=
                                        (a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) {
                                        painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                            egui::Stroke::new(2.0,egui::Color32::from_rgb(240,152,66)));
                                    }
                                }
                            }
                        }
                    }
                    // T/X nodes are evidence markers only; no edit or topology command is dispatched.
                    if state.wall_panel_hints {
                        if let Some(nodes)=proposal["wallPanelJunctionCandidates"].as_array() {
                            for (index,node) in nodes.iter().enumerate() {
                                let (Some(x),Some(y))=(node["pointCAD"][0].as_f64(),node["pointCAD"][1].as_f64()) else {continue;};
                                let point=to_screen(x,y);
                                let selected=state.selected_wall_junction==Some(index);
                                let color=if selected {egui::Color32::WHITE} else if state.reviewed_wall_junctions.contains(&index) {egui::Color32::LIGHT_GREEN} else {egui::Color32::from_rgb(245,105,212)};
                                painter.circle_stroke(point,if selected {8.0} else {5.0},egui::Stroke::new(if selected {3.0} else {2.0},color));
                            }
                        }
                    }
                    // Stage 226: show the selected short endpoint correction as an overlay.
                    // A preview line is not submitted to the editor, engine or undo stack.
                    if let Some(index)=state.selected_wall_junction {
                        if let Some(edits)=proposal["wallPanelJunctionCandidates"][index]["editPreview"].as_array() {
                            for edit in edits {
                                let (Some(ax),Some(ay),Some(bx),Some(by))=(edit["beforeCAD"][0].as_f64(),edit["beforeCAD"][1].as_f64(),
                                    edit["afterCAD"][0].as_f64(),edit["afterCAD"][1].as_f64()) else {continue;};
                                let from=to_screen(ax,ay);let to=to_screen(bx,by);
                                painter.line_segment([from,to],egui::Stroke::new(3.0,egui::Color32::from_rgb(255,171,43)));
                                painter.circle_filled(from,3.0,egui::Color32::from_rgb(245,90,90));
                                painter.circle_filled(to,3.0,egui::Color32::from_rgb(90,240,170));
                            }
                        }
                    }
                    // Draw the exact native ARC proposal in purple before committing.
                    // CAD uses CCW degrees; preview samples its short CCW sweep.
                    if state.create_door_swing_arcs {
                        let arcs=if state.create_door_blocks {None} else {proposal["doorSwingArcs"].as_array()};
                        if let Some(arcs)=arcs {
                            for arc in arcs {
                                let (Some(cx),Some(cy),Some(radius),Some(start),Some(end))=
                                    (arc["center"][0].as_f64(),arc["center"][1].as_f64(),
                                     arc["radius"].as_f64(),arc["start"].as_f64(),arc["end"].as_f64()) else {continue};
                                let sweep=(end-start).rem_euclid(360.0);
                                if sweep<1.0 || sweep>180.0 {continue;}
                                let mut points=Vec::new();
                                for n in 0..=24 {
                                    let angle=(start+sweep*f64::from(n)/24.0).to_radians();
                                    points.push(to_screen(cx+radius*angle.cos(),cy+radius*angle.sin()));
                                }
                                for segment in points.windows(2) {
                                    painter.line_segment([segment[0],segment[1]],
                                        egui::Stroke::new(2.0,egui::Color32::from_rgb(216,125,255)));
                                }
                            }
                        }
                    }
                    // Human-reviewed door Block drafts, including EACH selected
                    // door's hinge, opening direction and width, are drawn in
                    // magenta here. This matches the native LINE/ARC submitted.
                    if state.create_door_blocks {
                        if let Some(doors)=proposal["doorBlockDrafts"].as_array() {
                            for door in doors {
                                let leaf=&door["leaf"];
                                if let (Some(x0),Some(y0),Some(x1),Some(y1))=(leaf[0][0].as_f64(),leaf[0][1].as_f64(),leaf[1][0].as_f64(),leaf[1][1].as_f64()) {
                                    painter.line_segment([to_screen(x0,y0),to_screen(x1,y1)],
                                        egui::Stroke::new(3.0,egui::Color32::from_rgb(241,130,255)));
                                }
                                let arc=&door["arc"];
                                let (Some(cx),Some(cy),Some(radius),Some(start),Some(end))=
                                    (arc["center"][0].as_f64(),arc["center"][1].as_f64(),arc["radius"].as_f64(),
                                     arc["start"].as_f64(),arc["end"].as_f64()) else {continue};
                                let sweep=(end-start).rem_euclid(360.0);
                                if !(1.0..=180.0).contains(&sweep) {continue;}
                                let mut points=Vec::new();
                                for i in 0..=24 {
                                    let angle=(start+sweep*f64::from(i)/24.0).to_radians();
                                    points.push(to_screen(cx+radius*angle.cos(),cy+radius*angle.sin()));
                                }
                                for edge in points.windows(2) {
                                    painter.line_segment([edge[0],edge[1]],
                                        egui::Stroke::new(2.0,egui::Color32::from_rgb(241,130,255)));
                                }
                            }
                        }
                    }
                    // Two amber endpoint markers identify a *possible* opening.
                    // Never draw a connecting segment: that might look like a
                    // closed doorway, while the image itself is unverified.
                    if state.opening_gap_hints {
                        if let Some(gaps)=proposal["openingGapCandidates"].as_array() {
                            for gap in gaps {
                                for key in ["fromCAD","toCAD"] {
                                    if let (Some(x),Some(y))=(gap[key][0].as_f64(),gap[key][1].as_f64()) {
                                        let point=to_screen(x,y);
                                        painter.circle_stroke(point,4.0,egui::Stroke::new(2.0,egui::Color32::from_rgb(255,187,82)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(rectangles) = proposal["closedPolylines"].as_array() {
                ui.label(format!("Có {} PLINE kín (màu xanh lam), các LINE còn lại màu xanh lá.", rectangles.len()));
            }
            if let Some(axes) = proposal["wallCenterlines"].as_array() {
                if !axes.is_empty() { ui.small(format!("Có {} đường tim tường dự kiến. Nét nguồn được giữ nguyên.", axes.len())); }
            }
            ui.label("Bản xem trước chưa sửa dữ liệu CAD. Hãy lưu bản vẽ trước khi thực hiện một lô nét lớn.");
            if ui.add_enabled(count > 0, egui::Button::new("2. Tạo LINE / PLINE chỉnh sửa trong CAD")).clicked() {
                // Keep the whole AI batch in a single user-visible Undo step. Do not
                // unexpectedly close somebody else's active undo group.
                let undo_ready = cad.session.state().map(|d| {
                    d.undo_ctl.control == cadcraft_engine::undoctl::Control::All && d.undo_ctl.group.is_none()
                        && state.matches_document(d.uid, d.revision, cad.session.doc_epoch) && cad.session.running.is_none()
                }).unwrap_or(false);
                if let Err(reason)=ensure_raster_preview_input_unchanged(state) {
                    state.error=Some(reason);
                    state.preview=None;
                    state.preview_source_fingerprint=None;state.preview_params=None;
                    return;
                }
                if !undo_ready {
                    state.error = Some("Không thể ghi nét AI: bản vẽ đã chuyển hoặc thay đổi sau Preview, lệnh CAD khác đang chạy, hoặc UNDO Control chưa bật All. Hãy nhận dạng và xem trước lại.".to_string());
                    return;
                }
                if let Err(e) = cad.session.state_mut().map(|d| d.begin_undo_group()) {
                    state.error = Some(format!("Không thể bắt đầu nhóm UNDO: {e}"));
                    return;
                }
                let mut created = 0usize;
                let mut failure = None;
                if let Some(rectangles) = proposal["closedPolylines"].as_array() {
                    for corners in rectangles {
                        match cad.run("pline", json!({"vertices":corners,"closed":true})) {
                            Ok(_) => created += 1,
                            Err(e) => { failure = Some(e); break; }
                        }
                    }
                }
                if failure.is_none() && let Some(lines) = proposal["lines"].as_array() {
                    for line in lines {
                        match cad.run("line", json!({"points":line})) {
                            Ok(_) => created += 1,
                            Err(e) => { failure = Some(e); break; }
                        }
                    }
                }
                if failure.is_none() && let Some(drafts)=proposal["wallBlockDrafts"].as_array() {
                    let uid=cad.session.state().map(|d|d.uid).unwrap_or(0);
                    let rev=cad.session.state().map(|d|d.revision).unwrap_or(0);
                    for (index,draft) in drafts.iter().enumerate() {
                        let outline=cad.run("pline",json!({"vertices":draft["outline"],"closed":true}));
                        let handle=match outline {
                            Ok(v)=>{created+=1;v["handle"].as_str().map(str::to_string)},
                            Err(e)=>{failure=Some(e);break;}
                        };
                        let Some(handle)=handle else {failure=Some("AI Wall BLOCK: PLINE không trả Handle".into());break;};
                        let name=format!("TBT_AI_WALL_{}_{}_{}",uid,rev,index+1);
                        match cad.run("block",json!({"name":name,"base":draft["base"],"handles":[handle],
                            "keep":"convert","description":format!("AI wall panel drafting Block, lengthCAD={:.3}, thicknessCAD={:.3}; not semantic/BIM",
                                draft["lengthCAD"].as_f64().unwrap_or(0.0),draft["thicknessCAD"].as_f64().unwrap_or(0.0))})) {
                            Ok(v) if v["insert"].as_str().is_some()=>created+=1,
                            Ok(_)=>{failure=Some("AI Wall BLOCK: không tạo được INSERT".into());break;},
                            Err(e)=>{failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(panels) = proposal["wallPanelPolylines"].as_array() {
                    for polygon in panels {
                        match cad.run("pline",json!({"vertices":polygon,"closed":true})) {
                            Ok(_) => created+=1,
                            Err(e)=>{failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(centerlines) = proposal["wallCenterlines"].as_array() {
                    for line in centerlines {
                        match cad.run("line", json!({"points":line})) {
                            Ok(_) => created += 1,
                            Err(e) => { failure = Some(e); break; }
                        }
                    }
                }
                if !state.create_door_blocks && failure.is_none() && let Some(door_arcs) = proposal["doorSwingArcs"].as_array() {
                    for arc in door_arcs {
                        match cad.run("arc",json!({"center":arc["center"],"radius":arc["radius"],"start":arc["start"],"end":arc["end"]})) {
                            Ok(_) => created+=1,
                            Err(e) => {failure=Some(e);break;}
                        }
                    }
                }
                // Create a separate native LINE + ARC and convert only those two
                // new entities to a Block INSERT; keep traced source lines intact.
                if failure.is_none() && let Some(drafts) = proposal["doorBlockDrafts"].as_array() {
                    let doc_uid=cad.session.state().map(|d|d.uid).unwrap_or(0);
                    let doc_revision=cad.session.state().map(|d|d.revision).unwrap_or(0);
                    for (index,draft) in drafts.iter().enumerate() {
                        let l=cad.run("line",json!({"points":draft["leaf"]}));
                        let leaf_handle=match l {
                            Ok(value) => {created+=1; value["handles"][0].as_str().map(str::to_string)},
                            Err(e) => {failure=Some(e);break;}
                        };
                        let Some(leaf_handle)=leaf_handle else {failure=Some("AI Door BLOCK: lệnh LINE không trả Handle hợp lệ".into());break;};
                        let arc=&draft["arc"];
                        let ar=cad.run("arc",json!({"center":arc["center"],"radius":arc["radius"],"start":arc["start"],"end":arc["end"]}));
                        let arc_handle=match ar {
                            Ok(value) => {created+=1;value["handle"].as_str().map(str::to_string)},
                            Err(e) => {failure=Some(e);break;}
                        };
                        let Some(arc_handle)=arc_handle else {failure=Some("AI Door BLOCK: lệnh ARC không trả Handle hợp lệ".into());break;};
                        let name=format!("TBT_AI_DOOR_{}_{}_{}",doc_uid,doc_revision,index+1);
                        match cad.run("block",json!({"name":name,"base":draft["base"],"handles":[leaf_handle,arc_handle],
                            "keep":"convert","description":"AI door drafting guide; manually verified; not a semantic door"})) {
                            Ok(value) if value["insert"].as_str().is_some() => created+=1,
                            Ok(_) => {failure=Some("AI Door BLOCK: không tạo được đối tượng INSERT".into());break;},
                            Err(e) => {failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(drafts)=proposal["windowBlockDrafts"].as_array() {
                    let doc_uid=cad.session.state().map(|d|d.uid).unwrap_or(0);
                    let doc_revision=cad.session.state().map(|d|d.revision).unwrap_or(0);
                    for (index,draft) in drafts.iter().enumerate() {
                        let mut handles=Vec::new();
                        match cad.run("pline",json!({"vertices":draft["frame"],"closed":true})) {
                            Ok(v) => {created+=1;if let Some(handle)=v["handle"].as_str(){handles.push(handle.to_string());}
                                else {failure=Some("AI Window BLOCK: PLINE không trả Handle hợp lệ".into());break;}},
                            Err(e)=>{failure=Some(e);break;}
                        }
                        if let Some(rails)=draft["rails"].as_array() {
                            for rail in rails {
                                match cad.run("line",json!({"points":rail})) {
                                    Ok(v)=>{created+=1;if let Some(handle)=v["handles"][0].as_str(){handles.push(handle.to_string());}
                                        else {failure=Some("AI Window BLOCK: LINE không trả Handle hợp lệ".into());break;}},
                                    Err(e)=>{failure=Some(e);break;}
                                }
                            }
                        }
                        if failure.is_some(){break;}
                        if handles.len()!=3 {failure=Some("AI Window BLOCK: thiếu khung hoặc hai thanh cửa sổ".into());break;}
                        let name=format!("TBT_AI_WINDOW_{}_{}_{}",doc_uid,doc_revision,index+1);
                        match cad.run("block",json!({"name":name,"base":draft["base"],"handles":handles,
                            "keep":"convert","description":"AI window drafting guide, human reviewed, not semantic BIM"})) {
                            Ok(v) if v["insert"].as_str().is_some()=>created+=1,
                            Ok(_)=>{failure=Some("AI Window BLOCK: không tạo được INSERT".into());break;},
                            Err(e)=>{failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(drafts)=proposal["columnBlockDrafts"].as_array() {
                    let uid=cad.session.state().map(|d|d.uid).unwrap_or(0);
                    let revision=cad.session.state().map(|d|d.revision).unwrap_or(0);
                    for (index,draft) in drafts.iter().enumerate() {
                        let mut handles=Vec::new();
                        match cad.run("pline",json!({"vertices":draft["frame"],"closed":true})) {
                            Ok(v)=>{created+=1;if let Some(h)=v["handle"].as_str(){handles.push(h.to_string());}
                                else {failure=Some("AI Column BLOCK: thiếu PLINE handle".into());break;}},
                            Err(e)=>{failure=Some(e);break;}
                        }
                        if let Some(rails)=draft["hatches"].as_array() {
                            for rail in rails {
                                match cad.run("line",json!({"points":rail})) {
                                    Ok(v)=>{created+=1;if let Some(h)=v["handle"].as_str(){handles.push(h.to_string());}
                                        else {failure=Some("AI Column BLOCK: thiếu LINE handle".into());break;}},
                                    Err(e)=>{failure=Some(e);break;}
                                }
                            }
                        }
                        if failure.is_some(){break;}
                        if handles.len()!=3 {failure=Some("AI Column BLOCK: cần đúng 2 nét hatch và khung".into());break;}
                        let name=format!("TBT_AI_COLUMN_{}_{}_{}",uid,revision,index+1);
                        match cad.run("block",json!({"name":name,"base":draft["base"],"handles":handles,
                            "keep":"convert","description":"Reviewed column-like drawing block; not structural BIM"})) {
                            Ok(v) if v["insert"].as_str().is_some()=>created+=1,
                            Ok(_)=>{failure=Some("AI Column BLOCK: thiếu INSERT".into());break;},
                            Err(e)=>{failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(guides) = proposal["columnGuidePolylines"].as_array() {
                    for polygon in guides {
                        match cad.run("pline",json!({"vertices":polygon,"closed":true})) {
                            Ok(_) => created+=1,
                            Err(e) => {failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(guides) = proposal["windowGuidePolylines"].as_array() {
                    for polygon in guides {
                        match cad.run("pline",json!({"vertices":polygon,"closed":true})) {
                            Ok(_) => created+=1,
                            Err(e) => {failure=Some(e);break;}
                        }
                    }
                }
                if failure.is_none() && let Some(guides) = proposal["openingGuidePolylines"].as_array() {
                    for polygon in guides {
                        match cad.run("pline", json!({"vertices":polygon,"closed":true})) {
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
                    cad.set_status(format!("AI ảnh → CAD: đã xử lý {created} thao tác LINE/PLINE/ARC/BLOCK trong một nhóm UNDO. Kiểm tra trước khi lưu DWG."));
                    state.preview = None;
                    state.preview_document_uid = None;
                    state.preview_revision = None;
                    state.preview_document_epoch = None;
                    state.open = false;
                }
            }
            }
            });
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
    fn ai_raster_preview_never_commits_to_another_open_drawing() {
        let mut state = State::default();
        let original = cadcraft_engine::Session::new();
        let other = cadcraft_engine::Session::new();
        let original_uid = original.state().unwrap().uid;
        let other_uid = other.state().unwrap().uid;
        assert_ne!(original_uid, other_uid);
        let revision = original.state().unwrap().revision;
        let epoch = original.doc_epoch;
        assert!(!state.matches_document(original_uid, revision, epoch), "no preview means no commit");
        state.preview_document_uid = Some(original_uid);
        state.preview_revision = Some(revision);
        state.preview_document_epoch = Some(epoch);
        assert!(state.matches_document(original_uid, revision, epoch));
        assert!(!state.matches_document(other_uid, revision, epoch), "switching documents invalidates the AI preview binding");
        assert!(!state.matches_document(original_uid, revision + 1, epoch), "editing the same drawing after preview also invalidates it");
        assert!(!state.matches_document(original_uid, revision, epoch + 1), "switch-away-and-back invalidates preview even without edits");
    }

    #[test]
    fn ai_studio_primary_tab_is_image_workflow_and_advanced_settings_are_separate() {
        let mut state=State::default();state.open();
        assert_eq!(state.studio_tab,0);
        assert!(state.max_lines>=274);
        assert!(state.max_lines<=10_000);
    }

    #[test]
    fn ai_architecture_one_click_preset_enables_walls_openings_columns_and_rooms_evidence() {
        let mut state=State::default();state.open();
        apply_ai_instruction(&mut state,"nhận dạng toàn bộ mặt bằng").unwrap();
        let args=state.params();
        for key in ["wallPairHints","wallPanelHints","openingGapHints","pairedWallOpeningHints","doorLeafHints","windowSashHints","columnHatchHints"] {
            assert_eq!(args[key],true,"architecture preset should enable {key}");
        }
        assert_eq!(args["preview"],true);
        assert_eq!(args["createDoorBlocks"],false);
        assert_eq!(args["createWallBlocks"],false);
        assert_eq!(args["createColumnBlocks"],false);
    }

    #[test]
    fn ai_studio_default_handles_274_strokes_without_silent_truncation() {
        let mut state=State::default();state.open();
        assert_eq!(state.max_lines,500);
        assert_eq!(state.params()["maxLines"],500);
        state.max_lines=850;
        assert_eq!(state.params()["maxLines"],850);
    }

    #[test]
    fn ai_raster_preview_refuses_switch_away_and_back_with_unchanged_revision() {
        let mut state = State::default();
        state.preview_document_uid = Some(12);
        state.preview_revision = Some(30);
        state.preview_document_epoch = Some(7);
        assert!(state.matches_document(12, 30, 7));
        assert!(!state.matches_document(12, 30, 9),
            "switching to another drawing and back must invalidate the old AI preview");
        assert!(!state.matches_document(12, 31, 7));
        assert!(!state.matches_document(13, 30, 7));
    }

    #[test]
    fn ai_raster_help_is_accessible_offline_without_external_links() {
        let mut state = State::default();
        state.open_help();
        assert!(state.open);
        assert!(state.show_help);
        assert_eq!(state.params()["preview"], true);
    }

    #[test]
    fn ai_raster_calibration_uses_known_length_and_pixel_distance() {
        let scale = calibrated_units_per_pixel(50.0, 500.0).unwrap();
        assert!((scale - 10.0).abs() < 1.0e-12);
        let mut state = State::default();
        state.open();
        state.scale = scale;
        assert_eq!(state.params()["unitsPerPixel"], 10.0);
        assert_eq!(state.params()["preview"], true);
    }

    #[test]
    fn ai_raster_calibration_rejects_zero_nan_and_oversize_ratio() {
        assert!(calibrated_units_per_pixel(0.0, 500.0).is_err());
        assert!(calibrated_units_per_pixel(-2.0, 500.0).is_err());
        assert!(calibrated_units_per_pixel(50.0, f64::NAN).is_err());
        assert!(calibrated_units_per_pixel(f64::INFINITY, 500.0).is_err());
        assert!(calibrated_units_per_pixel(0.0000001, 500.0).is_err());
        assert!(calibrated_units_per_pixel(100000000.0, 0.1).is_err());
    }

    #[test]
    fn ai_new_source_requires_review_of_per_door_candidate_indices_again() {
        let mut state=State::default();state.open();
        state.door_width_overrides.insert(1,900.0);
        state.door_hinge_flips.insert(1);
        state.door_swing_flips.insert(0);
        // Test the SAME production reset routine invoked by the image picker.
        state.reset_door_reviews_for_new_source();
        assert!(state.door_width_overrides.is_empty());
        assert!(state.door_hinge_flips.is_empty());
        assert!(state.door_swing_flips.is_empty());
        assert_eq!(state.selected_door_number,1);
    }

    #[test]
    fn ai_individual_door_review_must_include_expected_source_keys_on_preview() {
        let mut state=State::default();state.open();
        state.create_door_blocks=true;
        state.last_door_source_keys=vec!["original-door-A".to_string(),"original-door-B".to_string()];
        assert!(state.params().get("expectedDoorSourceKeys").is_none());
        state.door_hinge_flips.insert(1);
        assert_eq!(state.params()["expectedDoorSourceKeys"],json!(["original-door-A","original-door-B"]));
        state.door_width_overrides.insert(0,900.0);
        assert_eq!(state.params()["expectedDoorSourceKeys"][0],"original-door-A");
        state.reset_door_reviews_for_new_source();
        assert!(state.params().get("expectedDoorSourceKeys").is_none());
    }

    #[test]
    fn ai_native_window_block_contains_one_frame_and_two_rails_and_undoes() {
        let mut engine=cadcraft_engine::Session::new();
        let before=engine.doc().unwrap().entity_count();
        engine.state_mut().unwrap().begin_undo_group();
        let frame=engine.execute("pline",&json!({"vertices":[[10.0,10.0],[30.0,10.0],[30.0,16.0],[10.0,16.0]],"closed":true})).unwrap();
        let rail_a=engine.execute("line",&json!({"points":[[10.0,12.0],[30.0,12.0]]})).unwrap();
        let rail_b=engine.execute("line",&json!({"points":[[10.0,14.0],[30.0,14.0]]})).unwrap();
        let (Some(hf),Some(ha),Some(hb))=(frame["handle"].as_str(),rail_a["handles"][0].as_str(),rail_b["handles"][0].as_str()) else {panic!("Missing native window entities")};
        let result=engine.execute("block",&json!({"name":"TBT_AI_WINDOW_TEST","base":[10.0,10.0],
            "handles":[hf,ha,hb],"keep":"convert"})).unwrap();
        assert!(result["insert"].as_str().is_some());
        assert_eq!(engine.doc().unwrap().entity_count(),before+1);
        assert_eq!(engine.doc().unwrap().block("TBT_AI_WINDOW_TEST").unwrap().entities.len(),3);
        engine.state_mut().unwrap().end_undo_group();
        engine.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(engine.doc().unwrap().entity_count(),before);
        assert!(engine.doc().unwrap().block("TBT_AI_WINDOW_TEST").is_none());
    }

    #[test]
    fn ai_window_block_gui_requires_separate_human_confirmation() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["createWindowBlocks"],false);
        assert_eq!(state.params()["confirmWindowBlocks"],false);
        state.window_sash_hints=true;
        state.create_window_blocks=true;
        assert_eq!(state.params()["confirmWindowBlocks"],false);
        state.confirm_window_blocks=true;
        assert_eq!(state.params()["createWindowBlocks"],true);
        assert_eq!(state.params()["confirmWindowBlocks"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_wall_junction_click_selects_unique_node_without_committing_geometry() {
        let nodes=vec![json!({"pointCAD":[10.0,20.0]}),json!({"pointCAD":[40.0,20.0]})];
        let identity=|x:f64,y:f64|[x,y];
        assert_eq!(preview_wall_junction_hit(&nodes,[11.0,20.0],identity,12.0),Some(0));
        assert_eq!(preview_wall_junction_hit(&nodes,[40.0,21.0],identity,12.0),Some(1));
        assert_eq!(preview_wall_junction_hit(&nodes,[25.0,20.0],identity,20.0),None);
        assert_eq!(preview_wall_junction_hit(&nodes,[90.0,20.0],identity,12.0),None);
        let mut state=State::default();state.open();state.selected_wall_junction=Some(1);state.reviewed_wall_junctions.insert(1);
        assert_eq!(state.params()["preview"],true);
        assert_eq!(state.params()["createWallBlocks"],false);
        assert_eq!(state.params()["createWallPanelPolylines"],false);
        state.reset_wall_junction_review();
        assert_eq!(state.selected_wall_junction,None);
        assert!(state.reviewed_wall_junctions.is_empty());
    }

    #[test]
    fn ai_wall_junction_preview_is_never_a_mutating_request() {
        let mut state=State::default();state.open();
        state.wall_pair_hints=true;state.wall_panel_hints=true;
        let params=state.params();
        assert_eq!(params["wallPanelHints"],true);
        assert_eq!(params["createWallPanelPolylines"],false);
        assert_eq!(params["createWallBlocks"],false);
        assert_eq!(params["confirmWallPanels"],false);
        assert_eq!(params["confirmWallBlocks"],false);
        assert_eq!(params["preview"],true);
    }

    #[test]
    fn ai_wall_block_gui_is_opt_in_and_requires_separate_review() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["createWallBlocks"],false);
        assert_eq!(state.params()["confirmWallBlocks"],false);
        state.wall_pair_hints=true;state.wall_panel_hints=true;
        state.create_wall_blocks=true;
        assert_eq!(state.params()["confirmWallBlocks"],false);
        state.confirm_wall_blocks=true;
        assert_eq!(state.params()["createWallBlocks"],true);
        assert_eq!(state.params()["confirmWallBlocks"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_wall_block_native_insert_contains_one_closed_panel_and_undoes() {
        let mut engine=cadcraft_engine::Session::new();
        let before=engine.doc().unwrap().entity_count();
        engine.state_mut().unwrap().begin_undo_group();
        let poly=engine.execute("pline",&json!({"vertices":[[10.0,10.0],[80.0,10.0],[80.0,24.0],[10.0,24.0]],"closed":true})).unwrap();
        let Some(handle)=poly["handle"].as_str() else {panic!("PLINE must return native CAD handle")};
        let result=engine.execute("block",&json!({"name":"TBT_AI_WALL_TEST","base":[10.0,10.0],
            "handles":[handle],"keep":"convert"})).unwrap();
        assert!(result["insert"].as_str().is_some());
        assert_eq!(engine.doc().unwrap().entity_count(),before+1);
        assert_eq!(engine.doc().unwrap().block("TBT_AI_WALL_TEST").unwrap().entities.len(),1);
        engine.state_mut().unwrap().end_undo_group();
        engine.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(engine.doc().unwrap().entity_count(),before);
        assert!(engine.doc().unwrap().block("TBT_AI_WALL_TEST").is_none());
    }

    #[test]
    fn ai_wall_panel_gui_defaults_to_preview_with_separate_confirmation() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["wallPanelHints"],false);
        assert_eq!(state.params()["createWallPanelPolylines"],false);
        assert_eq!(state.params()["confirmWallPanels"],false);
        state.wall_pair_hints=true;state.wall_panel_hints=true;
        assert_eq!(state.params()["wallPanelHints"],true);
        state.create_wall_panel_guides=true;
        assert_eq!(state.params()["confirmWallPanels"],false);
        state.confirm_wall_panel_guides=true;
        assert_eq!(state.params()["createWallPanelPolylines"],true);
        assert_eq!(state.params()["confirmWallPanels"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_column_block_gui_is_opt_in_and_preserves_preview_mode() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["createColumnBlocks"],false);
        assert_eq!(state.params()["confirmColumnBlocks"],false);
        state.column_hatch_hints=true;
        state.create_column_blocks=true;
        assert_eq!(state.params()["confirmColumnBlocks"],false);
        state.confirm_column_blocks=true;
        assert_eq!(state.params()["createColumnBlocks"],true);
        assert_eq!(state.params()["confirmColumnBlocks"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_column_block_engine_convert_has_one_insert_and_one_undo() {
        let mut engine=cadcraft_engine::Session::new();
        let before=engine.doc().unwrap().entity_count();
        engine.state_mut().unwrap().begin_undo_group();
        let frame=engine.execute("pline",&json!({"vertices":[[10.0,10.0],[42.0,10.0],[42.0,42.0],[10.0,42.0]],"closed":true})).unwrap();
        let x=engine.execute("line",&json!({"points":[[21.0,26.0],[31.0,26.0]]})).unwrap();
        let y=engine.execute("line",&json!({"points":[[26.0,21.0],[26.0,31.0]]})).unwrap();
        let (Some(hf),Some(hx),Some(hy))=(frame["handle"].as_str(),x["handles"][0].as_str(),y["handles"][0].as_str())
            else {panic!("column draft must return handles");};
        let block=engine.execute("block",&json!({"name":"TBT_AI_COLUMN_TEST","base":[10.0,10.0],
            "handles":[hf,hx,hy],"keep":"convert"})).unwrap();
        assert!(block["insert"].as_str().is_some());
        assert_eq!(engine.doc().unwrap().entity_count(),before+1);
        assert_eq!(engine.doc().unwrap().block("TBT_AI_COLUMN_TEST").unwrap().entities.len(),3);
        engine.state_mut().unwrap().end_undo_group();
        engine.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(engine.doc().unwrap().entity_count(),before);
        assert!(engine.doc().unwrap().block("TBT_AI_COLUMN_TEST").is_none());
    }

    #[test]
    fn ai_column_hatch_gui_defaults_to_preview_and_separate_confirmation() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["columnHatchHints"],false);
        assert_eq!(state.params()["createColumnGuidePolylines"],false);
        assert_eq!(state.params()["confirmColumnGuides"],false);
        state.column_hatch_hints=true;
        assert_eq!(state.params()["columnHatchHints"],true);
        state.create_column_guides=true;
        assert_eq!(state.params()["confirmColumnGuides"],false);
        state.confirm_column_guides=true;
        assert_eq!(state.params()["createColumnGuidePolylines"],true);
        assert_eq!(state.params()["confirmColumnGuides"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_window_sash_gui_requires_separate_hint_and_confirmed_guide_opt_in() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["windowSashHints"],false);
        assert_eq!(state.params()["createWindowGuidePolylines"],false);
        state.opening_gap_hints=true;
        state.paired_wall_opening_hints=true;
        state.window_sash_hints=true;
        assert_eq!(state.params()["windowSashHints"],true);
        state.create_window_guides=true;
        assert_eq!(state.params()["confirmWindowGuides"],false);
        state.confirm_window_guides=true;
        assert_eq!(state.params()["createWindowGuidePolylines"],true);
        assert_eq!(state.params()["confirmWindowGuides"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_preview_door_click_selects_nearest_door_and_refuses_ambiguous_click() {
        let doors=vec![
            json!({"leaf":[[10.0,10.0],[10.0,30.0]]}),
            json!({"leaf":[[50.0,10.0],[50.0,30.0]]})
        ];
        let identity=|x:f64,y:f64|[x,y];
        assert_eq!(preview_door_hit(&doors,[11.0,20.0],identity,13.0),Some(0));
        assert_eq!(preview_door_hit(&doors,[49.0,20.0],identity,13.0),Some(1));
        assert_eq!(preview_door_hit(&doors,[30.0,20.0],identity,30.0),None,
            "equal distance to two door leaves must not select either");
        assert_eq!(preview_door_hit(&doors,[90.0,20.0],identity,13.0),None);
        let overlap=vec![json!({"leaf":[[10.0,10.0],[10.0,30.0]]}),json!({"leaf":[[12.0,10.0],[12.0,30.0]]})];
        assert_eq!(preview_door_hit(&overlap,[11.0,20.0],identity,13.0),None,
            "overlapping leaves require explicit list selection");
    }

    #[test]
    fn ai_door_hinge_swing_gui_requires_door_block_and_preserves_selected_indices() {
        let mut state=State::default();state.open();
        assert!(state.params().get("doorHingeFlips").is_none());
        assert!(state.params().get("doorSwingFlips").is_none());
        state.create_door_blocks=true;
        state.door_hinge_flips.insert(1);
        state.door_swing_flips.insert(0);
        assert_eq!(state.params()["doorHingeFlips"],json!([1]));
        assert_eq!(state.params()["doorSwingFlips"],json!([0]));
        state.create_door_blocks=false;
        assert!(state.params().get("doorHingeFlips").is_none());
        assert!(state.params().get("doorSwingFlips").is_none());
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_individual_door_width_override_gui_serializes_selected_indices_only() {
        let mut state=State::default();state.open();
        assert!(state.params().get("doorWidthOverridesCAD").is_none());
        state.create_door_blocks=true;
        state.door_width_overrides.insert(1,850.0);
        state.door_width_overrides.insert(3,1200.0);
        let p=state.params();
        assert_eq!(p["doorWidthOverridesCAD"]["1"],850.0);
        assert_eq!(p["doorWidthOverridesCAD"]["3"],1200.0);
        assert!(p["doorWidthOverridesCAD"].as_object().unwrap().get("0").is_none());
        state.create_door_blocks=false;
        assert!(state.params().get("doorWidthOverridesCAD").is_none());
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_door_width_override_gui_requires_block_opt_in_and_defaults_to_unmodified_draft() {
        let mut state=State::default();
        state.open();
        assert!(state.params()["doorWidthOverrideCAD"].is_null());
        state.create_door_blocks=true;
        state.override_door_width=true;
        state.door_width_cad=850.0;
        assert_eq!(state.params()["doorWidthOverrideCAD"],850.0);
        state.create_door_blocks=false;
        assert!(state.params()["doorWidthOverrideCAD"].is_null(),"must not change geometry when Block is disabled");
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_native_door_draft_block_contains_line_and_arc_and_can_undo_as_one_step() {
        let mut engine=cadcraft_engine::Session::new();
        let original=engine.doc().unwrap().entity_count();
        engine.state_mut().unwrap().begin_undo_group();
        let l=engine.execute("line",&json!({"points":[[10.0,10.0],[10.0,22.0]]})).unwrap();
        let a=engine.execute("arc",&json!({"center":[10.0,10.0],"radius":12.0,"start":0.0,"end":90.0})).unwrap();
        let (Some(lh),Some(ah))=(l["handles"][0].as_str(),a["handle"].as_str()) else {panic!("CAD engine did not return entity handles")};
        let block=engine.execute("block",&json!({"name":"TBT_AI_DOOR_TEST","base":[10.0,10.0],"handles":[lh,ah],"keep":"convert"})).unwrap();
        assert!(block["insert"].as_str().is_some(),"native CAD BLOCK must have one INSERT");
        assert_eq!(engine.doc().unwrap().entity_count(),original+1);
        assert_eq!(engine.doc().unwrap().block("TBT_AI_DOOR_TEST").unwrap().entities.len(),2);
        engine.state_mut().unwrap().end_undo_group();
        engine.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(engine.doc().unwrap().entity_count(),original,"one Undo must clear the drafted door insert");
        assert!(engine.doc().unwrap().block("TBT_AI_DOOR_TEST").is_none(),
            "one Undo must also remove the drafted Block definition, not just the INSERT");
    }

    #[test]
    fn ai_door_swing_arc_gui_requires_explicit_two_step_confirmation() {
        let mut state=State::default();
        state.open();
        assert_eq!(state.params()["createDoorSwingArcs"],false);
        assert_eq!(state.params()["confirmDoorSwingArcs"],false);
        state.opening_gap_hints=true;
        state.paired_wall_opening_hints=true;
        state.door_leaf_hints=true;
        state.create_door_swing_arcs=true;
        assert_eq!(state.params()["createDoorSwingArcs"],true);
        assert_eq!(state.params()["confirmDoorSwingArcs"],false);
        state.confirm_door_swing_arcs=true;
        assert_eq!(state.params()["confirmDoorSwingArcs"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_combined_floor_plan_prompt_enables_all_detectors_but_no_commits() {
        let mut state=State::default();state.open();
        assert!(apply_ai_instruction(&mut state,"Vẽ lại toàn bộ mặt bằng có tường, cửa và cột").is_ok());
        assert!(state.wall_pair_hints && state.wall_panel_hints && state.door_leaf_hints && state.window_sash_hints && state.column_hatch_hints);
        assert!(state.arbitrary_angles);
        assert!(!state.confirm_door_blocks && !state.confirm_column_blocks && !state.confirm_wall_blocks);
        assert_eq!(state.params()["preview"],true);
        assert!(apply_ai_instruction(&mut state,"không rõ lệnh").is_err());
        assert!(apply_ai_instruction(&mut state,"wall").is_ok());
        assert!(state.wall_pair_hints && !state.door_leaf_hints && !state.column_hatch_hints);
    }

    #[test]
    fn ai_pdf_renderer_resolution_has_safe_executable_fallback() {
        let renderer=pdf_renderer();
        assert!(!renderer.as_os_str().is_empty());
    }

    #[test]
    fn ai_pdf_rejects_non_pdf_file_before_launching_external_renderer() {
        let path=std::env::temp_dir().join(format!("cad360_ai_not_pdf_{}_{}.pdf",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::write(&path,b"not actually a PDF").unwrap();
        let err=render_pdf_page(&path,1).unwrap_err();
        let _=std::fs::remove_file(&path);
        assert!(err.contains("chữ ký PDF"),"unexpected validation error: {err}");
    }

    #[test]
    fn ai_raster_commit_guard_rejects_changed_image_and_calibration_after_preview() {
        let path=std::env::temp_dir().join(format!("cad360-ai-commit-guard-{}.png",std::process::id()));
        std::fs::write(&path,b"PNG-plan-111111").unwrap();
        let mut state=State::default();
        state.open();
        state.path=path.to_string_lossy().into_owned();
        state.preview_source_fingerprint=Some(fingerprint_image(&state.path).unwrap());
        state.preview_params=Some(state.params());
        assert!(ensure_raster_preview_input_unchanged(&state).is_ok());
        state.scale=2.0;
        assert!(ensure_raster_preview_input_unchanged(&state).is_err(),"unit scale change invalidates approval");
        state.scale=1.0;
        assert!(ensure_raster_preview_input_unchanged(&state).is_ok());
        std::fs::write(&path,b"PNG-plan-222222").unwrap();
        assert!(ensure_raster_preview_input_unchanged(&state).is_err(),"source overwrite invalidates approval");
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_image_fingerprint_detects_same_size_source_mutation_without_timestamp_check() {
        let path=std::env::temp_dir().join(format!("cad360-ai-image-fingerprint-{}.png",std::process::id()));
        std::fs::write(&path,b"PNG-demo-contents-A").unwrap();
        let original=fingerprint_image(&path.to_string_lossy()).unwrap();
        std::fs::write(&path,b"PNG-demo-contents-B").unwrap();
        let modified=fingerprint_image(&path.to_string_lossy()).unwrap();
        assert_eq!(original.0,modified.0);
        assert_ne!(original.1,modified.1,"same-length changed image must invalidate approval");
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn ai_dense_image_filter_increases_only_detection_threshold_and_stays_bounded() {
        assert_eq!(next_dense_image_filter_min_length(16),64);
        assert_eq!(next_dense_image_filter_min_length(64),256);
        assert_eq!(next_dense_image_filter_min_length(500),2000);
        assert_eq!(next_dense_image_filter_min_length(4096),4096);
        assert_eq!(next_dense_image_filter_min_length(u32::MAX),4096);
    }

    #[test]
    fn ai_preview_task_starts_empty_and_never_commits_geometry_on_open() {
        let mut state=State::default();state.open();
        assert!(state.preview_task.is_none());
        assert!(state.preview.is_none());
        assert!(state.params()["preview"].as_bool().unwrap());
    }

    #[test]
    fn ai_studio_separates_image_detection_instruction_from_native_cad_commands() {
        let mut state=State::default();
        state.open();
        state.assistant_instruction="nhận dạng toàn bộ mặt bằng".into();
        assert!(cadcraft_ui_egui::ai_review::AiReview::validates_native_command(&state.native_command_input));
        assert!(!cadcraft_ui_egui::ai_review::AiReview::validates_native_command(&state.assistant_instruction));
        assert_ne!(state.native_command_input,state.assistant_instruction);
    }

    #[test]
    fn ai_local_cancel_flag_never_changes_document_or_approval_state() {
        let mut state=State::default();state.open();
        let (sender,receiver)=std::sync::mpsc::channel::<Result<String,String>>();
        let cancel=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        state.local_pending=Some(LocalModelTask {receiver,cancel:cancel.clone(),request:"vẽ phòng".into(),model:"qwen2.5:7b".into()});
        if let Some(task)=state.local_pending.take() {task.cancel.store(true,std::sync::atomic::Ordering::Relaxed);}
        drop(sender);
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
        assert!(state.local_pending.is_none() && state.local_command.is_none() && state.preview.is_none());
    }

    #[test]
    fn ai_local_llm_is_optional_and_does_not_commit_native_cad_on_open() {
        let mut state=State::default();state.open();
        assert!(state.local_pending.is_none() && state.local_command.is_none());
        assert_eq!(state.local_model,"qwen2.5:7b");
        assert!(cadcraft_ui_egui::ai_review::AiReview::validates_native_command("vẽ phòng rộng 6000 dài 4000 tại 0,0"));
        assert!(!cadcraft_ui_egui::ai_review::AiReview::validates_native_command("xóa toàn bộ bản vẽ"));
    }

    #[test]
    fn ai_pdf_commit_guard_refuses_modified_original_after_successful_page_preview() {
        let dir=std::env::temp_dir();
        let pdf=dir.join(format!("cad360-ai-pdf-approval-{}.pdf",std::process::id()));
        let png=dir.join(format!("cad360-ai-pdf-approval-{}.png",std::process::id()));
        std::fs::write(&pdf,b"%PDF-1.4\nframe-one").unwrap();
        std::fs::write(&png,b"sample raster content").unwrap();
        let mut state=State::default();state.open();
        state.path=png.to_string_lossy().into_owned();
        state.pdf_path=pdf.to_string_lossy().into_owned();
        state.pdf_render_fingerprint=Some(fingerprint_pdf(&pdf).unwrap());
        state.preview_source_fingerprint=Some(fingerprint_image(&state.path).unwrap());
        state.preview_params=Some(state.params());
        assert!(ensure_raster_preview_input_unchanged(&state).is_ok());
        std::fs::write(&pdf,b"%PDF-1.4\nframe-two").unwrap();
        assert!(ensure_raster_preview_input_unchanged(&state).is_err());
        let _=std::fs::remove_file(pdf);
        let _=std::fs::remove_file(png);
    }

    #[test]
    fn ai_pdf_cancel_refuses_before_poppler_spawn_and_keeps_no_raster() {
        let flag=std::sync::atomic::AtomicBool::new(true);
        let nonexistent=std::path::Path::new("pdf_missing_should_never_be_opened.pdf");
        let result=render_pdf_page_cancellable(nonexistent,1,&flag);
        assert!(result.unwrap_err().contains("Đã hủy"));
    }

    #[test]
    fn ai_pdf_source_fingerprint_rejects_same_length_mutation() {
        let file=std::env::temp_dir().join(format!("cad360_pdf_fingerprint_{}.pdf",std::process::id()));
        std::fs::write(&file,b"%PDF-1.4\nsource-aa").unwrap();
        let before=fingerprint_pdf(&file).unwrap();
        std::fs::write(&file,b"%PDF-1.4\nsource-bb").unwrap();
        let after=fingerprint_pdf(&file).unwrap();
        assert_eq!(before.0,after.0);
        assert_ne!(before.1,after.1);
        let _=std::fs::remove_file(file);
    }

    #[test]
    fn ai_pdf_page_gate_rejects_invalid_page_without_external_process() {
        assert!(render_pdf_page(std::path::Path::new("unused.pdf"),0).is_err());
        assert!(render_pdf_page(std::path::Path::new("unused.pdf"),501).is_err());
    }

    #[test]
    fn ai_instruction_presets_are_preview_only_and_never_auto_confirm_cad() {
        let mut state=State::default(); state.open();
        assert!(apply_ai_instruction(&mut state,"nhận dạng tường").is_ok());
        assert!(state.wall_pair_hints && state.wall_panel_hints);
        assert!(!state.confirm_wall_blocks && !state.create_wall_blocks);
        assert!(apply_ai_instruction(&mut state,"tìm cửa").is_ok());
        assert!(state.door_leaf_hints && state.window_sash_hints);
        assert!(!state.confirm_door_blocks);
        assert_eq!(state.params()["preview"],true);
        assert!(apply_ai_instruction(&mut state,"đặt mật khẩu").is_err());
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
    fn ai_raster_wall_spacing_can_be_set_by_user_without_mutating_drawing() {
        let mut state=State::default();
        state.open();
        assert_eq!(state.params()["minWallSpacingPixels"],2.0);
        assert_eq!(state.params()["maxWallSpacingPixels"],40.0);
        state.min_wall_spacing=8.0;
        state.max_wall_spacing=18.0;
        assert_eq!(state.params()["minWallSpacingPixels"],8.0);
        assert_eq!(state.params()["maxWallSpacingPixels"],18.0);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_raster_wall_centerline_creation_requires_explicit_user_opt_in() {
        let mut state=State::default();
        state.open();
        assert_eq!(state.params()["createWallCenterlines"], false);
        state.wall_pair_hints=true;
        state.create_wall_centerlines=true;
        assert_eq!(state.params()["wallPairHints"], true);
        assert_eq!(state.params()["createWallCenterlines"], true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_opening_gap_thresholds_are_editable_and_preview_only() {
        let mut state=State::default();state.open();
        assert_eq!(state.params()["minOpeningGapPixels"],3.0);
        assert_eq!(state.params()["maxOpeningGapPixels"],40.0);
        state.opening_gap_hints=true;
        state.min_opening_gap=10.0;state.max_opening_gap=20.0;
        assert_eq!(state.params()["openingGapHints"],true);
        assert_eq!(state.params()["minOpeningGapPixels"],10.0);
        assert_eq!(state.params()["maxOpeningGapPixels"],20.0);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_paired_wall_opening_guides_require_explicit_preview_and_confirmation() {
        let mut state=State::default();
        state.open();
        assert_eq!(state.params()["pairedWallOpeningHints"],false);
        assert_eq!(state.params()["createOpeningGuidePolylines"],false);
        assert_eq!(state.params()["confirmOpeningGuides"],false);
        state.opening_gap_hints=true;
        state.paired_wall_opening_hints=true;
        state.create_opening_guides=true;
        state.confirm_opening_guides=true;
        assert_eq!(state.params()["pairedWallOpeningHints"],true);
        assert_eq!(state.params()["createOpeningGuidePolylines"],true);
        assert_eq!(state.params()["confirmOpeningGuides"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_opening_gap_hints_are_disabled_by_default_and_preview_only() {
        let mut state=State::default(); state.open();
        assert_eq!(state.params()["openingGapHints"],false);
        state.opening_gap_hints=true;
        assert_eq!(state.params()["openingGapHints"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_raster_wall_pair_hints_are_opt_in_and_non_mutating() {
        let mut state=State::default();
        state.open();
        assert_eq!(state.params()["wallPairHints"],false);
        state.wall_pair_hints=true;
        assert_eq!(state.params()["wallPairHints"],true);
        assert_eq!(state.params()["preview"],true);
    }

    #[test]
    fn ai_raster_closed_rectangle_mode_is_opt_in_and_preserves_preview_safety() {
        let mut state = State::default();
        state.open();
        assert_eq!(state.params()["closedRectangles"], false);
        state.closed_rectangles = true;
        assert_eq!(state.params()["closedRectangles"], true);
        assert_eq!(state.params()["preview"], true);
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
