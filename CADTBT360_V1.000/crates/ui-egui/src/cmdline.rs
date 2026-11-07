//! The floating command line: history, the active prompt with clickable keywords, typed input
//! (keyboard focus stays on the drawing — typing anywhere goes here) and AutoComplete.

use std::collections::HashSet;

use cadcraft_engine::Input;
use egui::{Key, Pos2, Rect, Sense, Stroke, pos2, vec2};

use crate::CadApp;
use crate::theme::Tokens;

#[derive(Default)]
pub struct CmdLine {
    pub buffer: String,
    /// The selection as of the last frame, to notice when it changes.
    seen_selection: Vec<cadcraft_doc::Handle>,
    /// Something was typed at the command line since the selection was made.
    typed_since_selection: bool,
    pub history: Vec<String>,
    pub history_pos: Option<usize>,
    /// What was typed at command prompts (not command names), oldest first: the in-command
    /// shortcut menu's Recent Input.
    pub recent_input: Vec<String>,
    /// The expanded history window is open (F2, the chevron, or a click on the history lines).
    pub expanded: bool,
    /// Dragging the history's top edge: (pointer y, line count) when the drag started.
    resize_from: Option<(f32, usize)>,
    /// The Dynamic Input options menu at the cursor (Down Arrow at a prompt with keywords).
    pub options: Option<crate::dyninput::OptionsMenu>,
    /// The text whose second Dynamic Input value Tab locked; the lock holds while it is unchanged.
    locked_second: Option<String>,
    /// Where the options menu's rows were drawn last frame (screen), for clicks on them.
    pub option_rows: Vec<(Rect, String)>,
    /// Where the Dynamic Input tooltip stays while the options menu is open.
    pub options_anchor: Option<Pos2>,
    /// The IME's composition (pre-edit) text: shown after the buffer while composing, never saved.
    pub preedit: String,
    /// The command line's editing rectangle and caret (screen) from the last frame, for the IME's
    /// composition window (see `enable_ime`).
    pub ime_rect: Option<(Rect, Rect)>,
    /// The command line's IME context generation: a new one each time it starts collecting text,
    /// as egui asks of an IME owner that (re)gains focus.
    ime_generation: u64,
    /// Whether the command line published an IME context last frame.
    ime_on: bool,
}

/// The most history lines the command line shows above its bar.
pub const MAX_LINES: usize = 12;
const LINE_H: f32 = 16.0;

/// The Dynamic Input frame for the prompt on screen, or `None` when the value boxes are not live
/// (Dynamic Input off, no point prompt or a pending selection window).
pub fn dyn_frame(app: &CadApp) -> Option<crate::dyninput::Frame> {
    let st = &app.session.settings;
    if !st.dynmode || app.session.pending_window.is_some() {
        return None;
    }
    let p = app.session.current_prompt().filter(cadcraft_engine::pointmod::accepts)?;
    Some(crate::dyninput::Frame { base: p.base, cartesian: st.dynpi_cartesian, absolute: st.dynpi_absolute })
}

/// Take Tab away from egui before it moves keyboard focus to the next widget (which would leave
/// the command line deaf to typing). While no widget has focus, a pressed Tab becomes a `"\t"`
/// text event, which keeps its place among the other keystrokes; [`keyboard`] handles it.
pub fn capture_tab(ctx: &egui::Context, raw: &mut egui::RawInput) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    raw.events.retain_mut(|e| {
        let tab = match e {
            egui::Event::Key { key: Key::Tab, pressed, .. } => Some(*pressed),
            _ => None,
        };
        match tab {
            Some(true) => {
                *e = egui::Event::Text("\t".into());
                true
            }
            Some(false) => false,
            None => true,
        }
    });
}

/// Tab on the command line: cycle the object snaps under the cursor, type the Dynamic Input
/// separator (`,` or `<`) after a value, or complete a command name.
fn tab(app: &mut CadApp) {
    // Grip dimensional input: Tab locks the typed value and moves to the next field.
    if crate::grip_dyn::tab(app) {
        return;
    }
    // Over a snap marker with nothing typed, Tab cycles the candidate snaps.
    if app.cmd.buffer.is_empty() && app.canvas.snap.is_some() {
        app.canvas.snap_tab = true;
    } else if let Some(f) = dyn_frame(app)
        && let Some(sep) = crate::dyninput::tab_separator(&app.cmd.buffer, &f)
    {
        app.cmd.buffer.push(sep);
    } else if let Some(f) = dyn_frame(app)
        && crate::dyninput::can_lock_second(&app.cmd.buffer, &f)
    {
        // Both values typed: Tab locks the second one as well.
        app.cmd.locked_second = Some(app.cmd.buffer.clone());
    } else if let Some(s) = suggestions(&app.cmd.buffer).first() {
        app.cmd.buffer = s.id.clone();
    }
}

/// An AutoComplete suggestion for the command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    /// Command identifier (uppercase, e.g. "RECTANG"), used for buffer completion and execution.
    pub id: String,
    /// Display text shown in the AutoComplete list (e.g. "REC (RECTANG)" or "RECOVER").
    pub display: String,
    /// Human-readable command label (e.g. "Rectangle").
    pub label: &'static str,
}

/// Vietnamese command search uses a small verified dictionary of *real* engine
/// commands. Suggestions never advertise a nonexistent command or bypass prompts.
const VI_COMMANDS: &[(&str, &str, &str)] = &[
    ("lam sach ban ve", "purge", "Làm sạch bản vẽ"),
    ("don dep ban ve", "purge", "Dọn dẹp bản vẽ"),
    ("kiem tra loi ban ve", "audit", "Kiểm tra lỗi bản vẽ"),
    ("ve duong thang", "line", "Vẽ đường thẳng"),
    ("ve da doan", "pline", "Vẽ đa đoạn"),
    ("ve duong tron", "circle", "Vẽ đường tròn"),
    ("ve hinh chu nhat", "rectang", "Vẽ hình chữ nhật"),
    ("ve cung tron", "arc", "Vẽ cung tròn"),
    ("do khoang cach", "dist", "Đo khoảng cách"),
    ("do chieu dai", "dist", "Đo chiều dài giữa hai điểm"),
    ("do dien tich", "area", "Đo diện tích"),
    ("tim toa do", "id", "Tìm tọa độ điểm"),
    ("lay toa do", "id", "Lấy tọa độ điểm"),
    ("ghi kich thuoc", "dimlinear", "Ghi kích thước thẳng"),
    ("do kich thuoc", "dimlinear", "Đo và ghi kích thước thẳng"),
    ("ghi kich thuoc xien", "dimaligned", "Ghi kích thước xiên"),
    ("sao chep doi tuong", "copy", "Sao chép đối tượng"),
    ("di chuyen doi tuong", "move", "Di chuyển đối tượng"),
    ("xoay doi tuong", "rotate", "Xoay đối tượng"),
    ("xoa doi tuong", "erase", "Xóa đối tượng"),
    ("cat net", "trim", "Cắt nét"),
    ("keo dai net", "extend", "Kéo dài nét"),
    ("thu phong ban ve", "zoom", "Thu phóng bản vẽ"),
    ("xem toan bo ban ve", "zoom", "Xem toàn bộ bản vẽ"),
    ("quan ly lop", "layer", "Quản lý lớp"),
    ("ve duong song song", "offset", "Vẽ đường song song"),
    ("dich chuyen song song", "offset", "Dịch chuyển song song"),
    ("bo tron goc", "fillet", "Bo tròn góc"),
    ("vat mep", "chamfer", "Vát mép"),
    ("lay doi xung", "mirror", "Lấy đối xứng"),
    ("doi ty le", "scale", "Đổi tỷ lệ"),
    ("keo gian doi tuong", "stretch", "Kéo giãn đối tượng"),
    ("nhan ban theo mang", "array", "Nhân bản theo mảng"),
    ("pha khoi", "explode", "Phá khối"),
    ("noi doan thang", "join", "Nối đoạn thẳng"),
    ("ngat doan thang", "break", "Ngắt đoạn thẳng"),
    ("to mat cat", "hatch", "Tô mặt cắt"),
    ("to hatch", "hatch", "Tô mẫu vật liệu"),
    ("ve chu", "text", "Vẽ chữ"),
    ("ghi chu nhieu dong", "mtext", "Ghi chú nhiều dòng"),
    ("ghi ban kinh", "dimradius", "Ghi kích thước bán kính"),
    ("ghi duong kinh", "dimdiameter", "Ghi kích thước đường kính"),
    ("ghi kich thuoc goc", "dimangular", "Ghi kích thước góc"),
    ("luu ban ve", "qsave", "Lưu bản vẽ"),
    ("luu ban ve thanh", "saveas", "Lưu bản vẽ thành tệp mới"),
    ("mo ban ve", "open", "Mở bản vẽ"),
    ("tao ban ve moi", "new", "Tạo bản vẽ mới"),
    ("hoan tac", "undo", "Hoàn tác"),
    ("lam lai thao tac", "redo", "Làm lại thao tác"),
    ("bat diem doi tuong", "osnap", "Bắt điểm đối tượng"),
    ("cai dat bat diem", "dsettings", "Thiết lập bắt điểm"),
    ("cai dat ve", "dsettings", "Thiết lập vẽ"),
    ("xem thuoc tinh", "properties", "Xem thuộc tính"),
    ("sao chep thuoc tinh", "matchprop", "Sao chép thuộc tính"),
    ("tao khoi", "block", "Tạo khối"),
    ("chen khoi", "insert", "Chèn khối"),
    ("chen tham chieu", "xref", "Chèn tham chiếu"),
    ("in ban ve", "plot", "In bản vẽ"),
    ("di chuyen khung nhin", "pan", "Di chuyển khung nhìn"),
    ("tai tao hien thi", "regen", "Tái tạo hiển thị"),
    ("cai dat don vi", "units", "Cài đặt đơn vị"),
    ("xem danh sach doi tuong", "list", "Xem danh sách đối tượng"),
    ("thong ke doi tuong", "count", "Thống kê đối tượng"),
    ("do ban kinh", "dimradius", "Đo bán kính và ghi kích thước"),
    ("do duong kinh", "dimdiameter", "Đo đường kính và ghi kích thước"),
    ("ve duong cong", "spline", "Vẽ đường cong spline"),
    ("ve duong xuyen tam", "xline", "Vẽ đường thẳng vô hạn"),
    ("ghi chu thich", "mleader", "Ghi chú dẫn hướng"),
    ("sap xep thu tu ve", "draworder", "Sắp xếp thứ tự hiển thị"),
    ("doi ten lop", "layer", "Đổi tên lớp"),
    ("ve elip", "ellipse", "Vẽ elip"),
    ("ve diem", "point", "Vẽ điểm"),
    ("can chinh doi tuong", "align", "Căn chỉnh đối tượng"),
    ("can chinh chu", "textalign", "Căn chỉnh chữ"),
    ("chon nhanh doi tuong", "qselect", "Chọn nhanh đối tượng"),
    ("chon cung thuoc tinh", "selectsimilar", "Chọn đối tượng cùng thuộc tính"),
    ("lay thong tin ban ve", "status", "Lấy thông tin bản vẽ"),
    ("tim va thay the chu", "find", "Tìm và thay thế chữ"),
    ("don net trung", "overkill", "Dọn nét trùng"),
    ("khoi phuc tep ve", "recover", "Khôi phục bản vẽ"),
    ("quan ly kieu chu", "style", "Quản lý kiểu chữ"),
    ("tao bang", "table", "Tạo bảng"),
    ("bao ve ban ve", "lock", "Khóa đối tượng"),
    ("xuat du lieu", "export", "Xuất dữ liệu"),
];

pub(crate) fn fold_vi(text: &str) -> String {
    text.to_lowercase().chars().filter_map(|ch| {
        let plain = match ch {
            'à'|'á'|'ả'|'ã'|'ạ'|'ă'|'ằ'|'ắ'|'ẳ'|'ẵ'|'ặ'|'â'|'ầ'|'ấ'|'ẩ'|'ẫ'|'ậ' => 'a',
            'è'|'é'|'ẻ'|'ẽ'|'ẹ'|'ê'|'ề'|'ế'|'ể'|'ễ'|'ệ' => 'e',
            'ì'|'í'|'ỉ'|'ĩ'|'ị' => 'i',
            'ò'|'ó'|'ỏ'|'õ'|'ọ'|'ô'|'ồ'|'ố'|'ổ'|'ỗ'|'ộ'|'ơ'|'ờ'|'ớ'|'ở'|'ỡ'|'ợ' => 'o',
            'ù'|'ú'|'ủ'|'ũ'|'ụ'|'ư'|'ừ'|'ứ'|'ử'|'ữ'|'ự' => 'u',
            'ỳ'|'ý'|'ỷ'|'ỹ'|'ỵ' => 'y',
            'đ' => 'd',
            '\u{0300}'..='\u{036f}' => return None,
            _ => ch,
        };
        Some(plain)
    }).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Reuse verified native-command Vietnamese phrases in F1 Command Handbook.
/// No new alias or executable command is registered by this search metadata.
pub(crate) fn matches_vietnamese_command_phrase(id:&str,query:&str)->bool {
    let term=fold_vi(query);
    !term.is_empty() && VI_COMMANDS.iter().any(|(phrase,command,_)|
        command.eq_ignore_ascii_case(id) && phrase.contains(&term))
}

fn vietnamese_matches(text: &str) -> Vec<Suggestion> {
    let folded = fold_vi(text);
    if folded.is_empty() {return Vec::new();}
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    for (phrase, command, label) in VI_COMMANDS {
        if !phrase.starts_with(&folded) {continue;}
        let Some(spec)=cadcraft_engine::cmd::find_command(command) else {continue};
        if seen.insert(spec.id) {
            results.push(Suggestion {
                id:spec.id.to_ascii_uppercase(),
                display:format!("{} ({label})",spec.id.to_ascii_uppercase()),
                label:spec.label
            });
        }
        if results.len()>=8 {break;}
    }
    results
}

/// Search the existing Vietnamese translation catalog across registered
/// commands. This scales to hundreds of commands without a second hardcoded
/// command registry; only exposed engine IDs are allowed.
fn translated_command_matches(text:&str)->Vec<Suggestion> {
    let folded=fold_vi(text);
    if folded.chars().count()<2 {return Vec::new();}
    let mut found=Vec::new();
    for spec in cadcraft_engine::command_specs() {
        if spec.id.contains('.') {continue;}
        let translated=crate::i18n::tr("vi",spec.label);
        if translated==spec.label {continue;}
        let normalized=fold_vi(translated);
        if normalized.starts_with(&folded) || normalized.contains(&folded) {
            found.push((!normalized.starts_with(&folded),normalized.len(),spec.id,
                Suggestion {id:spec.id.to_ascii_uppercase(),
                    display:format!("{} ({translated})",spec.id.to_ascii_uppercase()),
                    label:spec.label}));
        }
    }
    found.sort_by_key(|item|(item.0,item.1,item.2));
    found.into_iter().take(8).map(|(_,_,_,suggestion)|suggestion).collect()
}

/// Read the *registered* command aliases, rather than inventing abbreviations.
/// A command can have many aliases; shortest is preferred for everyday typing.
fn shortest_registered_alias(id:&str)->Option<&'static str> {
    cadcraft_engine::cmd::find_command(&id.to_ascii_lowercase())
        .and_then(|spec| spec.aliases.iter().copied()
            .filter(|a| !a.is_empty() && !a.contains('.'))
            .min_by_key(|a| (a.len(),*a)))
}

/// English first, then Vietnamese in parentheses. Every suggestion keeps
/// its original executable ID, label and exact alias resolution.
fn bilingual_command_caption(s:&Suggestion)->String {
    let english=s.label;
    let vietnamese=bilingual_label(&s.id)
        .or_else(|| {
            let vi=crate::i18n::tr("vi",s.label);
            (vi!=s.label).then_some(vi)
        });
    let shortcut=shortest_registered_alias(&s.id)
        .map(|alias|format!(" · {}",alias.to_ascii_uppercase()))
        .unwrap_or_default();
    match vietnamese {
        Some(vi)=>format!("{english} ({vi}){shortcut}"),
        None=>format!("{english}{shortcut}")
    }
}

fn natural_language_command(id:&str)->Option<&'static str> {
    VI_COMMANDS.iter().find(|(_,cmd,_)|cmd.eq_ignore_ascii_case(id))
        .map(|(_,_,vi)|*vi)
}

/// Suggestions are tied to the command just used. Nothing is executed
/// automatically; they are only displayed after a command finishes.
fn suggested_next_commands(id:&str)->&'static [&'static str] {
    match id {
        "line"|"pline"|"arc"|"rectang" => &["line","dist","dimlinear","offset"],
        "circle" => &["line","dimdiameter","dist","move"],
        "trim"|"extend" => &["fillet","dist","dimlinear","line"],
        "move"|"copy"|"rotate" => &["dist","line","copy","undo"],
        "dist"|"id"|"area" => &["dimlinear","dimaligned","line","id"],
        "dimlinear"|"dimaligned" => &["dimcontinue","dist","line","move"],
        "purge"|"audit" => &["audit","qsave","zoom"],
        _ => &["line","pline","dist","id"],
    }
}
/// Resolve only known native CAD commands. Session history may contain
/// Vietnamese phrases or literal CAD aliases; never turn arbitrary user text
/// into executable advice or silently create missing command aliases.
fn command_from_history(text:&str)->Option<String> {
    let query=fold_vi(text);
    if let Some((_,command,_))=VI_COMMANDS.iter().find(|(phrase,_,_)|*phrase==query)
        && cadcraft_engine::cmd::find_command(command).is_some() {
        return Some((*command).to_owned());
    }
    let command=cadcraft_engine::cmd::resolve_alias(&query);
    cadcraft_engine::cmd::find_command(&command).map(|spec|spec.id.to_owned())
}

/// A small session-only next-step model: frequently repeated command
/// transitions are ranked higher, then safe context defaults. This does
/// not change engine state, store private history or auto-dispatch actions.
fn next_command_habits(current:&str, history:&[String])->Vec<String> {
    let mut choices=Vec::new();
    let mut scored=Vec::<(String,usize)>::new();
    let tail=history.iter().rev().take(80).rev()
        .filter_map(|command|command_from_history(command)).collect::<Vec<_>>();
    for pair in tail.windows(2) {
        if pair[0]!=current || pair[1]==current {continue;}
        if let Some((_,count))=scored.iter_mut().find(|(name,_)|name==&pair[1]) {
            *count+=1;
        } else {
            scored.push((pair[1].clone(),1));
        }
    }
    scored.sort_by(|a,b|b.1.cmp(&a.1).then_with(||a.0.cmp(&b.0)));
    for (cmd,_) in scored.into_iter().take(3) { choices.push(cmd); }
    for cmd in suggested_next_commands(current) {
        if cadcraft_engine::cmd::find_command(cmd).is_some()
            && !choices.iter().any(|used|used==cmd) {
            choices.push((*cmd).to_owned());
        }
        if choices.len()>=4 {break;}
    }
    choices.truncate(4);
    choices
}

/// Offers native editing commands for the actual current selection. These
/// suggestions are context-specific but intentionally do NOT inspect, mutate
/// or predict the geometry. They only make existing CAD actions discoverable.
/// Classify the native selected EntityKind; no raster inference or guessed
/// geometry. Mixed selections use general edit commands rather than pretending
/// the whole selection consists of a line, circle or text.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum SelectionKind {None,Line,Curve,Polyline,Text,Block,Dimension,Hatch,Mixed,Other}

impl SelectionKind {
    fn bilingual_caption(self)->&'static str {
        match self {
            Self::None=>"NONE (Chưa chọn)",
            Self::Line=>"LINE (Đường thẳng)",
            Self::Curve=>"CURVE (Đường cong / Tròn / Cung)",
            Self::Polyline=>"POLYLINE (Đa đoạn)",
            Self::Text=>"TEXT (Văn bản)",
            Self::Block=>"BLOCK (Khối)",
            Self::Dimension=>"DIMENSION (Kích thước)",
            Self::Hatch=>"HATCH (Mặt cắt)",
            Self::Mixed=>"MIXED (Nhiều loại đối tượng)",
            Self::Other=>"OBJECT (Đối tượng khác)",
        }
    }
}

fn native_selection_kind(kinds:&[&cadcraft_doc::EntityKind])->SelectionKind {
    use cadcraft_doc::EntityKind as E;
    let classify=|kind:&E|match kind {
        E::Line(_) | E::Ray(_) | E::XLine(_)=>SelectionKind::Line,
        E::Circle(_) | E::Arc(_) | E::Ellipse(_) | E::Spline(_)=>SelectionKind::Curve,
        E::LwPolyline(_) | E::Polyline3d(_) | E::MLine(_)=>SelectionKind::Polyline,
        E::Text(_) | E::MText(_)|E::AttDef(_)=>SelectionKind::Text,
        E::Insert(_)=>SelectionKind::Block,
        E::Dimension(_) | E::Leader(_) | E::MLeader(_)=>SelectionKind::Dimension,
        E::Hatch(_)=>SelectionKind::Hatch,
        _=>SelectionKind::Other,
    };
    let Some(first)=kinds.first().map(|kind|classify(kind)) else{return SelectionKind::None;};
    if kinds.iter().skip(1).any(|kind|classify(kind)!=first){SelectionKind::Mixed}
    else{first}
}

fn selected_native_context(app:&CadApp)->SelectionKind {
    let handles=app.session.selection();
    if handles.is_empty(){return SelectionKind::None;}
    let Ok(st)=app.session.state() else {return SelectionKind::Other};
    let kinds=handles.iter().filter_map(|handle|
        st.doc.entity(*handle).map(|entity|&entity.kind)).collect::<Vec<_>>();
    if kinds.len()!=handles.len() {return SelectionKind::Other;}
    native_selection_kind(&kinds)
}

fn contextual_native_candidates(kind:SelectionKind)->&'static [&'static str] {
    match kind {
        SelectionKind::None=>&["line","pline","circle","rectang","dist","id"],
        SelectionKind::Line=>&["offset","trim","extend","fillet","dist","dimlinear","move"],
        SelectionKind::Curve=>&["trim","offset","dimradius","dimdiameter","dist","move"],
        SelectionKind::Polyline=>&["pedit","offset","area","hatch","dist","move"],
        SelectionKind::Text=>&["ddedit","textedit","style","move","copy","properties"],
        SelectionKind::Block=>&["bedit","explode","insert","move","copy","properties"],
        SelectionKind::Dimension=>&["dimedit","dimstyle","dimcontinue","move","properties"],
        SelectionKind::Hatch=>&["hatchedit","hatch","area","move","properties"],
        SelectionKind::Mixed|SelectionKind::Other=>&["move","copy","rotate","list","erase","properties"],
    }
}

/// Never suggest a command that is missing in the real engine registry.
fn selection_kind_next_commands(kind:SelectionKind,previous:Option<&str>,history:&[String])->Vec<String> {
    let mut names=Vec::<String>::new();
    for candidate in contextual_native_candidates(kind) {
        if cadcraft_engine::cmd::find_command(candidate).is_some() {
            names.push((*candidate).to_owned());
        }
    }
    if let Some(previous)=previous {
        for id in next_command_habits(previous,history) {
            if !names.contains(&id) {names.push(id);}
        }
    }
    names.truncate(5);
    names
}

fn selection_aware_next_commands(selection_count:usize,previous:Option<&str>,history:&[String])->Vec<String> {
    let mut names=Vec::<String>::new();
    if selection_count>0 {
        let candidate_ids: &[&str] = if selection_count==1 {
            &["move","copy","rotate","list","erase","properties","dimlinear"]
        } else {
            &["move","copy","rotate","erase","list","properties","group"]
        };
        for id in candidate_ids {
            if cadcraft_engine::cmd::find_command(id).is_some() {
                names.push((*id).to_owned());
            }
        }
    }
    if let Some(previous)=previous {
        for id in next_command_habits(previous,history) {
            if !names.contains(&id) {names.push(id);}
        }
    } else if selection_count==0 {
        for id in ["line","pline","circle","rectang","dist","id"] {
            if cadcraft_engine::cmd::find_command(id).is_some() {names.push(id.to_owned());}
        }
    }
    names.truncate(5);
    names
}

fn related_command_caption(id:&str)->Option<String> {
    let spec=cadcraft_engine::cmd::find_command(id)?;
    let vi=natural_language_command(id)
        .or_else(|| {
            let val=crate::i18n::tr("vi",spec.label);
            (val!=spec.label).then_some(val)
        });
    let alias=shortest_registered_alias(id).unwrap_or(id).to_ascii_uppercase();
    Some(match vi {
        Some(vi)=>format!("{alias} ({} · {vi})",spec.id.to_ascii_uppercase()),
        None=>format!("{alias} ({})",spec.id.to_ascii_uppercase()),
    })
}

fn bilingual_label(command: &str) -> Option<&'static str> {
    VI_COMMANDS.iter().find(|(_,id,_)|id.eq_ignore_ascii_case(command))
        .map(|(_,_,label)|*label)
}

/// A space after a Vietnamese natural-language verb is part of the query.
/// A normal CAD alias (L, PL, C, REC, DI...) retains its traditional
/// Space-to-Enter behaviour, and command prompts always preserve text input.
fn vietnamese_query_continues(buffer:&str)->bool {
    let normalized=fold_vi(buffer);
    // Native exact names and PGP aliases take precedence even if one happens
    // to spell the beginning of a Vietnamese word.
    let native=cadcraft_engine::cmd::resolve_alias(&normalized);
    if cadcraft_engine::cmd::find_command(&native).is_some() {return false;}
    !normalized.is_empty() && VI_COMMANDS.iter()
        .any(|(phrase,_,_)|phrase.starts_with(&(normalized.clone()+" ")))
}

fn active_command_guide(id:&str)->&'static str {
    match id {
        "line" => "LINE / L (Vẽ đường thẳng): chọn điểm đầu → điểm tiếp theo; Enter kết thúc · Esc hủy",
        "pline" => "PLINE / PL (Vẽ đa đoạn): chọn các đỉnh → Enter kết thúc; Close (Khép kín)",
        "circle" => "CIRCLE / C (Vẽ đường tròn): chọn tâm → nhập bán kính hoặc chọn điểm trên đường tròn",
        "rectang" => "RECTANG / REC (Vẽ chữ nhật): chọn góc thứ nhất → chọn góc đối diện",
        "dist" => "DIST / DI (Đo khoảng cách): chọn hai điểm; xem kết quả đo trong lịch sử lệnh",
        "id" => "ID (Tìm tọa độ): chọn một điểm → xem X, Y, Z",
        "dimlinear" => "DIMLINEAR / DLI (Ghi kích thước): chọn hai điểm đo → nơi đặt kích thước",
        "dimaligned" => "DIMALIGNED / DAL (Kích thước xiên): chọn hai điểm → vị trí đặt số đo",
        "purge" => "PURGE (Làm sạch bản vẽ): chọn nội dung không dùng; kiểm tra rồi xác nhận",
        "trim" => "TRIM / TR (Cắt nét): chọn cạnh cắt → đối tượng; Enter/Esc tùy bước",
        "extend" => "EXTEND / EX (Kéo dài nét): chọn biên → đối tượng cần kéo dài",
        "move" => "MOVE / M (Di chuyển): chọn đối tượng → điểm gốc → điểm đích",
        "copy" => "COPY / CO (Sao chép): chọn đối tượng → điểm gốc → điểm bản sao",
        _ => "ACTIVE COMMAND (Đang thực hiện): xem prompt; chọn điểm/đối tượng hoặc nhập giá trị; Esc hủy",
    }
}

/// The user sees the real registered command and its *actual* shortest
/// alias even if this command has no hand-authored tutorial yet. All command
/// IDs remain owned by the native engine; this never changes CAD semantics.
fn registered_active_command_help(id:&str)->String {
    let named=active_command_guide(id);
    if !named.starts_with("ACTIVE COMMAND") {return named.to_owned();}
    let Some(spec)=cadcraft_engine::cmd::find_command(id) else {
        return format!("{} (Lệnh chưa có trong danh mục): Esc (Hủy)",id.to_ascii_uppercase());
    };
    let vi=natural_language_command(id).or_else(||{
        let translated=crate::i18n::tr("vi",spec.label);
        (translated!=spec.label).then_some(translated)
    });
    let abbrev=shortest_registered_alias(id)
        .filter(|alias| !alias.eq_ignore_ascii_case(id))
        .map(|alias|format!(" / {}",alias.to_ascii_uppercase()))
        .unwrap_or_default();
    let label=vi.map(|vn|format!("{} ({vn})",spec.label))
        .unwrap_or_else(||spec.label.to_owned());
    format!("{}{} — {} · Follow prompt (Theo bước đang chờ) · Esc (Hủy)",
        spec.id.to_ascii_uppercase(),abbrev,label)
}

/// Generate the assistance from the *actual active Prompt*, not guessed
/// command-specific steps. Object picks, point/number/text prompts, keyword
/// options and Enter semantics remain controlled by the engine.
fn active_prompt_help(prompt:&cadcraft_engine::Prompt)->String {
    let action=if prompt.picks_objects() {
        "Select object (Chọn đối tượng) · Click on drawing (Nhấp đối tượng)"
    } else if prompt.accept.point && prompt.accept.number {
        "Pick point / Enter value (Chọn điểm hoặc nhập số)"
    } else if prompt.accept.point {
        "Pick point / X,Y (Chọn điểm trên bản vẽ hoặc nhập tọa độ)"
    } else if prompt.accept.number {
        "Enter number (Nhập giá trị số)"
    } else if prompt.accept.text {
        "Enter text (Nhập văn bản hoặc tên tùy chọn)"
    } else {
        "Follow command prompt (Xem yêu cầu ở dòng lệnh)"
    };
    let mut result=format!("WAITING (Đang chờ): {action}");
    if !prompt.keywords.is_empty() {
        let options=prompt.keywords.iter().take(3)
            .map(|k|bilingual_keyword_caption(k)).collect::<Vec<_>>().join(" / ");
        result.push_str(&format!(" · Options (Tùy chọn): {options}"));
    } else if prompt.accept.enter {
        result.push_str(" · Enter (Mặc định/kết thúc theo bước)");
    }
    result.push_str(" · Esc (Hủy)");
    result
}

/// Keep the English engine prompt intact and only add a Vietnamese visual
/// explanation. Interactive keyword values and script parser remain original.
fn bilingual_prompt_caption(english:&str)->String {
    let vietnamese=crate::i18n::tr("vi",english);
    if vietnamese!=english {
        format!("{english} ({vietnamese})")
    } else {
        english.to_owned()
    }
}

/// UI-only bilingual keyword caption. The original English keyword is
/// still submitted to Prompt::match_keyword / Input::Keyword unchanged.
fn bilingual_keyword_caption(keyword:&str)->String {
    let vietnamese=crate::i18n::tr("vi",keyword);
    if vietnamese!=keyword {
        format!("{keyword} ({vietnamese})")
    } else {
        keyword.to_owned()
    }
}

/// Every native prompt option gets a bilingual *display*, but retains its
/// exact engine keyword for mouse dispatch; no truncation to the first 3.
fn native_bilingual_options(prompt:&cadcraft_engine::Prompt)->Vec<(String,String)> {
    prompt.keywords.iter().map(|keyword|
        (keyword.clone(),bilingual_keyword_caption(keyword))).collect()
}

/// Matching command names/aliases for AutoComplete.
fn suggestions(prefix: &str) -> Vec<Suggestion> {
    if prefix.is_empty() {return Vec::new();}
    if prefix.chars().any(char::is_whitespace) {
        let phrases=vietnamese_matches(prefix);
        return if phrases.is_empty() {translated_command_matches(prefix)} else {phrases};
    }
    let p = prefix.to_ascii_lowercase();

    let mut out: Vec<Suggestion> = Vec::new();
    let mut seen_ids: HashSet<&'static str> = HashSet::new();

    // 1. Exact match: alias resolution or exact command name match.
    let resolved = cadcraft_engine::cmd::resolve_alias(&p);
    if let Some(spec) = cadcraft_engine::cmd::find_command(&resolved)
        && !spec.id.contains('.')
    {
        let display =
            if spec.id == p { spec.id.to_ascii_uppercase() } else { format!("{} ({})", p.to_ascii_uppercase(), spec.id.to_ascii_uppercase()) };
        out.push(Suggestion { id: spec.id.to_ascii_uppercase(), display, label: spec.label });
        seen_ids.insert(spec.id);
    }

    // Preserve exact native command aliases L/PL/C/REC/DI... as highest
    // priority. Vietnamese prefix matches take over only if no exact alias.
    if out.is_empty() {
        let vi=vietnamese_matches(prefix);
        if !vi.is_empty() {return vi;}
        let translated=translated_command_matches(prefix);
        if !translated.is_empty() {return translated;}
    }

    // 2. Command ID prefix matches (excluding already matched exact command).
    let mut id_matches: Vec<&'static cadcraft_engine::CommandSpec> =
        cadcraft_engine::command_specs().iter().filter(|c| !c.id.contains('.') && !seen_ids.contains(c.id) && c.id.starts_with(&p)).collect();
    id_matches.sort_by_key(|c| (c.id.len(), c.id));
    for c in id_matches {
        out.push(Suggestion { id: c.id.to_ascii_uppercase(), display: c.id.to_ascii_uppercase(), label: c.label });
        seen_ids.insert(c.id);
    }

    // 3. Command alias prefix matches (for commands not yet added).
    let mut alias_matches: Vec<(&'static str, &'static cadcraft_engine::CommandSpec)> = Vec::new();
    for c in cadcraft_engine::command_specs() {
        if c.id.contains('.') || seen_ids.contains(c.id) {
            continue;
        }
        let matching_alias = c.aliases.iter().copied().filter(|a| a.starts_with(&p)).min_by_key(|a| (a.len(), *a));
        if let Some(alias) = matching_alias {
            alias_matches.push((alias, c));
        }
    }
    alias_matches.sort_by_key(|(alias, c)| (alias.len(), *alias, c.id));
    for (alias, c) in alias_matches {
        if seen_ids.insert(c.id) {
            out.push(Suggestion {
                id: c.id.to_ascii_uppercase(),
                display: format!("{} ({})", alias.to_ascii_uppercase(), c.id.to_ascii_uppercase()),
                label: c.label,
            });
        }
    }

    out.truncate(8);
    out
}

impl CmdLine {
    /// Track the selection: a new selection starts with nothing typed since.
    pub fn watch_selection(&mut self, sel: &[cadcraft_doc::Handle]) {
        if self.seen_selection != sel {
            self.seen_selection = sel.to_vec();
            self.typed_since_selection = false;
        }
    }

    /// Note that the command line holds typed text.
    pub fn note_typing(&mut self) {
        if !self.buffer.is_empty() {
            self.typed_since_selection = true;
        }
    }

    /// Tab locked the second Dynamic Input value and nothing was typed since.
    pub fn second_locked(&self) -> bool {
        self.locked_second.as_deref() == Some(self.buffer.as_str())
    }

    /// The options menu row under `at` (screen), if the menu is open.
    pub fn option_at(&self, at: Pos2) -> Option<String> {
        self.options.as_ref()?;
        self.option_rows.iter().find(|(r, _)| r.contains(at)).map(|(_, k)| k.clone())
    }
}

/// Keep the options menu only while its prompt is on screen with Dynamic Input and DYNPROMPT on.
fn options_still_fit(app: &mut CadApp) {
    let st = &app.session.settings;
    let on = st.dynmode && st.dynprompt && app.canvas.hot_grips.is_empty();
    let kws = app.session.current_prompt().map(|p| p.keywords).unwrap_or_default();
    if app.cmd.options.as_ref().is_some_and(|m| !on || !m.fits(&kws)) {
        app.cmd.options = None;
    }
}

/// Down Arrow at a prompt with keywords (Dynamic Input on, DYNPROMPT on, not browsing the
/// history): open the options menu at the cursor, or move its highlight. `false` when the key is
/// left for the history.
fn options_key(app: &mut CadApp, step: isize) -> bool {
    if let Some(m) = app.cmd.options.as_mut() {
        m.step(step);
        return true;
    }
    let st = &app.session.settings;
    if step < 0 || !st.dynmode || !st.dynprompt || app.cmd.history_pos.is_some() || !app.canvas.hot_grips.is_empty() {
        return false;
    }
    let kws = app.session.current_prompt().map(|p| p.keywords).unwrap_or_default();
    app.cmd.options = crate::dyninput::OptionsMenu::open(&kws);
    app.cmd.options.is_some()
}

/// Send a keyword picked from the options menu, as a click on it at the command line does.
pub fn pick_option(app: &mut CadApp, keyword: String) {
    app.cmd.options = None;
    app.cmd.buffer.clear();
    if let Err(e) = app.session.input(Input::Keyword(keyword)) {
        app.session.echo_error(e.to_string());
    }
}

/// Whether Backspace should erase the selection instead of editing the command line. The key
/// labelled "delete" on a Mac sends Backspace, so on macOS it erases selected objects, but only
/// when it can't be meant for the command line: a fresh, plain press (never an auto-repeat, so
/// holding it to clear typing can't run on into the drawing, and not Option/Cmd+delete, which
/// delete a word or line of text on macOS), no command running, the command line empty, and
/// nothing typed there since the selection was made.
pub fn backspace_erases(mac: bool, press: KeyPress, buffer: &str, command_running: bool, has_selection: bool, typed_since_selection: bool) -> bool {
    mac && press.plain && !press.repeat && buffer.is_empty() && !command_running && has_selection && !typed_since_selection
}

/// How a key was pressed.
#[derive(Clone, Copy, Debug)]
pub struct KeyPress {
    /// The operating system's auto-repeat while the key is held.
    pub repeat: bool,
    /// No Option, Control or Command held (Shift doesn't matter).
    pub plain: bool,
}

/// Let the operating system's IME compose into the command line while a prompt asks for text.
/// Nothing owns keyboard focus there (typing goes to the command line from anywhere), so egui
/// never enables the IME by itself: publishing an IME output does it, with the command line's
/// rectangle and caret for the composition window. Without this a Windows IME (Pinyin, Japanese,
/// Korean...) cannot even switch on (#1761).
fn enable_ime(app: &mut CadApp, ctx: &egui::Context) {
    if !app.cmd.ime_on {
        app.cmd.ime_on = true;
        app.cmd.ime_generation = app.cmd.ime_generation.wrapping_add(1);
    }
    let (rect, cursor_rect) = app.cmd.ime_rect.unwrap_or_else(|| {
        let r = ctx.content_rect();
        (r, Rect::from_min_size(pos2(r.left() + 30.0, r.bottom() - 34.0), vec2(1.0, 18.0)))
    });
    ctx.output_mut(|o| {
        o.ime = Some(egui::output::IMEOutput::new(
            egui::Id::unique("cc_cmdline_ime"),
            app.cmd.ime_generation,
            egui::IMEPurpose::Normal,
            rect,
            cursor_rect,
        ));
    });
}

/// IME needs to work at idle Command: for Vietnamese natural-language
/// commands, not only inside TEXT/MTEXT prompts. Do not force an IME during
/// point/number/object selection because native CAD key handling remains active.
fn command_line_needs_ime(command_running:bool,text_prompt:bool)->bool {
    text_prompt || !command_running
}

/// Handle keyboard input destined for the command line (when no text field has focus).
pub fn keyboard(app: &mut CadApp, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() {
        app.cmd.preedit.clear();
        app.cmd.ime_on = false;
        return;
    }
    let events = ctx.input(|i| i.events.clone());
    let selection = app.session.selection();
    app.cmd.watch_selection(&selection);
    let mac = ctx.os() == egui::os::OperatingSystem::Mac;
    let text_prompt = app.session.current_prompt().is_some_and(|p| p.accept.text && !p.accept.point && !p.accept.number);
    if command_line_needs_ime(app.session.running.is_some()||app.session.prompting(),text_prompt) {
        // Enable Vietnamese Telex/VNI composition at idle Command: as well as
        // native free-text prompts. egui keeps focusless CAD keyboard routing.
        enable_ime(app, ctx);
    } else {
        // Numeric/point/selection prompts keep original CAD keyboard semantics.
        app.cmd.preedit.clear();
        app.cmd.ime_on = false;
    }
    options_still_fit(app);
    for ev in events {
        // The options menu takes the arrows, Enter and Escape; typing closes it.
        if app.cmd.options.is_some() {
            let space = matches!(&ev, egui::Event::Text(t) if t == " ") && !text_prompt;
            match &ev {
                _ if space => {
                    if let Some(k) = app.cmd.options.as_ref().and_then(|m| m.current()).map(str::to_owned) {
                        pick_option(app, k);
                    }
                    continue;
                }
                egui::Event::Key { key: Key::Enter, pressed: true, .. } => {
                    if let Some(k) = app.cmd.options.as_ref().and_then(|m| m.current()).map(str::to_owned) {
                        pick_option(app, k);
                    }
                    continue;
                }
                egui::Event::Key { key: Key::Escape, pressed: true, .. } => {
                    app.cmd.options = None;
                    continue;
                }
                egui::Event::Key { key: Key::ArrowUp, pressed: true, modifiers, .. } if !modifiers.any() => {
                    options_key(app, -1);
                    continue;
                }
                egui::Event::Key { key: Key::ArrowDown | Key::ArrowUp, pressed: true, .. } => {}
                // Space arrives as a key and as text; the text picks.
                egui::Event::Key { key: Key::Space, .. } => {}
                egui::Event::Text(_) | egui::Event::Key { pressed: true, .. } => app.cmd.options = None,
                _ => {}
            }
        }
        match ev {
            egui::Event::Text(t) => {
                if t == "\t" {
                    tab(app);
                } else if t == " " && !text_prompt {
                    if !app.session.prompting() && vietnamese_query_continues(&app.cmd.buffer) {
                        app.cmd.buffer.push(' ');
                    } else {
                        submit(app);
                    }
                } else {
                    app.cmd.preedit.clear();
                    app.cmd.buffer.push_str(&t);
                }
            }
            // IME composition: the pre-edit is shown but stays out of the buffer; the commit is
            // the only text that is inserted. Enter/Return commits arrive separately as keys, so
            // a newline commit is ignored (as egui's own text fields do) instead of submitting.
            egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => app.cmd.preedit = text,
            egui::Event::Ime(egui::ImeEvent::Commit(text)) => {
                app.cmd.preedit.clear();
                if text != "\n" && text != "\r" {
                    app.cmd.buffer.push_str(&text);
                }
            }
            egui::Event::Ime(_) => {}
            egui::Event::Key { key, pressed: true, repeat, modifiers, .. } => match key {
                Key::Enter => submit(app),
                Key::Escape => {
                    app.cmd.buffer.clear();
                    app.cmd.preedit.clear();
                    app.session.cancel();
                }
                Key::Backspace => {
                    let press = KeyPress { repeat, plain: !(modifiers.alt || modifiers.ctrl || modifiers.command) };
                    let erase = backspace_erases(
                        mac,
                        press,
                        &app.cmd.buffer,
                        app.session.prompting(),
                        !app.session.selection().is_empty(),
                        app.cmd.typed_since_selection,
                    );
                    if erase {
                        let _ = app.run("erase.selection", serde_json::json!({}));
                    } else {
                        app.cmd.buffer.pop();
                    }
                }
                Key::ArrowUp if !modifiers.any() => {
                    if !app.cmd.history.is_empty() {
                        let n = app.cmd.history.len();
                        let pos = app.cmd.history_pos.map(|p| p.saturating_sub(1)).unwrap_or(n - 1);
                        app.cmd.history_pos = Some(pos);
                        app.cmd.buffer = app.cmd.history.get(pos).cloned().unwrap_or_default();
                    }
                }
                Key::ArrowDown if !modifiers.any() => {
                    if !options_key(app, 1)
                        && let Some(p) = app.cmd.history_pos
                    {
                        let np = p + 1;
                        if np >= app.cmd.history.len() {
                            app.cmd.history_pos = None;
                            app.cmd.buffer.clear();
                        } else {
                            app.cmd.history_pos = Some(np);
                            app.cmd.buffer = app.cmd.history.get(np).cloned().unwrap_or_default();
                        }
                    }
                }
                Key::Tab => tab(app),
                Key::Delete if app.cmd.buffer.is_empty() && !app.session.prompting() => {
                    let _ = app.run("erase.selection", serde_json::json!({}));
                }
                _ => {}
            },
            _ => {}
        }
        app.cmd.note_typing();
    }
}

pub fn submit(app: &mut CadApp) {
    app.cmd.preedit.clear();
    // Grip dimensional input: the typed lengths and angles place the grip.
    if crate::grip_dyn::submit(app) {
        return;
    }
    // Dynamic Input: a split entry (`40,30`, `40<90`, `40,`) is completed from the cursor and
    // measured from the last point.
    if let Some(f) = dyn_frame(app) {
        let cursor = app.canvas.cursor.unwrap_or(app.session.cursor);
        let last = app.session.last_point;
        let done = crate::dyninput::parse(&app.cmd.buffer, &f).and_then(|e| crate::dyninput::complete(&e, &f, cursor, last));
        if let Some(text) = done {
            app.cmd.buffer = text;
        }
    }
    let text = std::mem::take(&mut app.cmd.buffer);
    app.cmd.locked_second = None;
    app.cmd.history_pos = None;
    if !text.trim().is_empty() && !app.session.prompting() {
        app.cmd.history.push(text.clone());
        if app.cmd.history.len() > 200 {
            app.cmd.history.remove(0);
        }
    }
    crate::context_menu::note_input(app, &text);
    if text.is_empty() {
        if let Err(e) = app.session.input(Input::Enter) {
            app.session.echo_error(e.to_string());
        }
        return;
    }
    // Translate only a full, exact Vietnamese command phrase. Partial words
    // never trigger a command, preserving user review of suggestions.
    let query=fold_vi(&text);
    if !app.session.prompting() {
        if let Some((_,command,_))=VI_COMMANDS.iter().find(|(phrase,_,_)|*phrase==query)
            && cadcraft_engine::cmd::find_command(command).is_some()
        {
            app.cmdline(command);
            return;
        }
    }
    app.cmdline(&text);
}

fn visible_lines(app: &CadApp, canvas: Rect, bar: Rect) -> usize {
    let available = ((bar.top() - canvas.top() - 8.0).max(0.0) / LINE_H) as usize;
    app.ui.history_lines.min(MAX_LINES).min(available)
}

/// Draw the floating command bar and its history over the bottom-centre of `canvas`; the bar is 48 % of the canvas width, clamped to 360–760 points and never wider than the canvas minus 8 points.
pub fn show(app: &mut CadApp, ui: &mut egui::Ui, canvas: Rect) {
    let t = Tokens::get();
    let w = (canvas.width() * 0.48).clamp(360.0, 760.0).min((canvas.width() - 8.0).max(0.0));
    let h = 24.0;
    let bar = Rect::from_min_size(pos2(canvas.center().x - w / 2.0, canvas.bottom() - h - 10.0), vec2(w, h));
    let p = ui.painter_at(canvas);
    history(app, ui, canvas, bar);
    // History lines above the bar (the expanded window replaces them).
    let n = if app.cmd.expanded { 0 } else { visible_lines(app, canvas, bar) };
    let lines: Vec<String> = app.session.log.iter().rev().take(n).rev().cloned().collect();
    if !lines.is_empty() {
        let lh = 16.0;
        let hr = Rect::from_min_size(pos2(bar.left(), bar.top() - lh * lines.len() as f32 - 4.0), vec2(w, lh * lines.len() as f32 + 2.0));
        p.rect_filled(hr, 3.0, t.cmd_history);
        for (i, l) in lines.iter().enumerate() {
            p.text(pos2(hr.left() + 8.0, hr.top() + 1.0 + lh * i as f32), egui::Align2::LEFT_TOP, l, crate::theme::small(), t.text_dim);
        }
    }
    // Explicit command-state help remains visible while the user is in
    // LINE/PLINE/CIRCLE/RECTANG/DIST/ID/DIMLINEAR, not only after clicking.
    if let Some(running)=app.session.running.as_ref() {
        let guide=registered_active_command_help(&running.id);
        let expected=app.session.current_prompt()
            .map(|prompt|active_prompt_help(&prompt))
            .unwrap_or_else(||"ACTIVE (Đang thực hiện lệnh) · Esc (Hủy)".to_owned());
        // Long bilingual instructions must wrap instead of drawing past the
        // right edge of the canvas or covering the caret / command history.
        let text_width=(w-14.0).max(20.0);
        let expected_galley=p.layout(expected,egui::FontId::proportional(11.0),t.accent,text_width);
        let guide_galley=p.layout(guide,egui::FontId::proportional(11.0),t.text,text_width);
        let expected_height=expected_galley.size().y;
        let height=expected_height+guide_galley.size().y+12.0;
        let y=(bar.top()-height-5.0-n as f32*16.0
            -if lines.is_empty(){0.0}else{6.0}).max(canvas.top()+3.0);
        let hint=Rect::from_min_size(pos2(bar.left(),y),vec2(w,height));
        p.rect_filled(hint,3.0,t.cmd_history);
        p.galley(pos2(hint.left()+7.0,hint.top()+4.0),expected_galley,t.accent);
        p.galley(pos2(hint.left()+7.0,hint.top()+7.0+expected_height),
            guide_galley,t.text);
    }
    // Contextual next-command hints appear after the previous command, not
    // while entering geometry. Existing CAD command input is never hijacked.
    if !app.session.prompting() && app.cmd.buffer.is_empty()
        && app.session.running.is_none() {
        let previous=app.cmd.history.last()
            .and_then(|text|command_from_history(text));
        let selected_count=app.session.selection().len();
        let kind=selected_native_context(app);
        let related=if selected_count>0 {
            selection_kind_next_commands(kind,previous.as_deref(),&app.cmd.history)
        } else {
            selection_aware_next_commands(0,previous.as_deref(),&app.cmd.history)
        };
        if !related.is_empty() {
                let hint=Rect::from_min_size(
                    pos2(bar.left(),(bar.top()-23.0-n as f32*16.0
                        -if lines.is_empty(){0.0}else{6.0}).max(canvas.top()+3.0)),
                    vec2(w,20.0));
                p.rect_filled(hint,3.0,t.cmd_history);
                let title=if selected_count>0 {
                    format!("{} {} · NEXT (Tiếp): ",selected_count,kind.bilingual_caption())
                } else {
                    "NEXT (Lệnh tiếp): ".to_owned()
                };
                let tx=hint.left()+7.0;
                let font=egui::FontId::proportional(10.0);
                let prefix=p.layout_no_wrap(title,font.clone(),t.text);
                let mut x=tx+prefix.size().x;
                p.galley(pos2(tx,hint.center().y-prefix.size().y/2.0),prefix,t.text);
                for (i,id) in related.iter().take(3).enumerate() {
                    let Some(caption)=related_command_caption(id) else {continue};
                    let short=shortest_registered_alias(id).unwrap_or(id);
                    let label=natural_language_command(id)
                        .or_else(||cadcraft_engine::cmd::find_command(id).and_then(|spec|{
                            let translated=crate::i18n::tr("vi",spec.label);
                            (translated!=spec.label).then_some(translated)
                        }));
                    let button_text=label.map(|vi|format!("{} ({vi})",short.to_ascii_uppercase()))
                        .unwrap_or_else(||short.to_ascii_uppercase());
                    let galley=p.layout_no_wrap(button_text,font.clone(),t.text);
                    let width=galley.size().x+10.0;
                    if x+width>hint.right()-6.0 {break;}
                    let r=Rect::from_min_size(pos2(x,hint.top()+1.0),vec2(width,18.0));
                    let response=ui.interact(r,ui.scope_id().with(("next_command",i)),Sense::click())
                        .on_hover_text(format!("{caption} · Click to start (Nhấn để chạy)"));
                    if response.hovered() {p.rect_filled(r,3.0,t.cmd_keyword_hover);}
                    p.galley(pos2(r.left()+5.0,r.center().y-galley.size().y/2.0),
                        galley,t.text);
                    if response.clicked() && !app.session.prompting() {
                        // A deliberate click uses the same native dispatch path
                        // as command line selection. Suggestions never auto-run.
                        app.start(id);
                    }
                    x+=width+4.0;
                }
        }
    }
    p.rect_filled(bar, 3.0, t.cmd_bg);
    p.rect_stroke(bar, 3.0, Stroke::new(1.0, t.cmd_border), egui::StrokeKind::Inside);
    // Prompt glyph.
    p.text(pos2(bar.left() + 8.0, bar.center().y), egui::Align2::LEFT_CENTER, ">_", crate::theme::mono(), t.text_dim);
    let mut x = bar.left() + 30.0;
    let prompt = app.session.current_prompt();
    let font = crate::theme::body();
    match &prompt {
        Some(pr) => {
            // The grip modes name themselves ("** STRETCH **…").
            let name = app.session.running.as_ref().filter(|r| r.id != "grip").map(|r| r.id.to_ascii_uppercase() + " ").unwrap_or_default();
            let g = p.layout_no_wrap(name, font.clone(), t.text_faint);
            let gw = g.size().x;
            p.galley(pos2(x, bar.center().y - g.size().y / 2.0), g, t.text_faint);
            x += gw;
            // A LISP `get…` prompt shows as the routine wrote it (no keywords, default or colon added).
            let message = if pr.raw { pr.display() } else { bilingual_prompt_caption(&pr.message) };
            let g = p.layout_no_wrap(message, font.clone(), t.text);
            let gw = g.size().x;
            p.galley(pos2(x, bar.center().y - g.size().y / 2.0), g, t.text);
            x += gw;
            // Keyword captions can be very long when bilingual. The main
            // command bar must reserve room for typed coordinates and the IME.
            // All clickable native keywords are shown in the wrapped overlay
            // below instead of overflowing the bar off-screen.
            if !pr.raw && !pr.keywords.is_empty() {
                let g=p.layout_no_wrap("  [OPTIONS ↓] ".into(),font.clone(),t.cmd_keyword);
                let gw=g.size().x;
                if x+gw<bar.right()-90.0 {
                    p.galley(pos2(x,bar.center().y-g.size().y/2.0),g,t.cmd_keyword);
                    x+=gw;
                }
            }
            if let Some(d) = pr.default.as_ref().filter(|_| !pr.raw) {
                let g = p.layout_no_wrap(format!(" <{d}>"), font.clone(), t.text);
                let gw = g.size().x;
                p.galley(pos2(x, bar.center().y - g.size().y / 2.0), g, t.text);
                x += gw;
            }
            if !pr.raw {
                let g = p.layout_no_wrap(": ".into(), font.clone(), t.text);
                let gw = g.size().x;
                p.galley(pos2(x, bar.center().y - g.size().y / 2.0), g, t.text);
                x += gw;
            } else if !pr.message.ends_with(' ') {
                x += 4.0;
            }
        }
        None => {
            if app.cmd.buffer.is_empty() {
                p.text(
                    pos2(x, bar.center().y),
                    egui::Align2::LEFT_CENTER,
                    crate::tl!("Type a command"),
                    egui::FontId::proportional(12.5),
                    t.text_faint,
                );
            }
        }
    }
    let g = p.layout_no_wrap(app.cmd.buffer.clone(), font.clone(), t.text);
    let gw = g.size().x;
    p.galley(pos2(x, bar.center().y - g.size().y / 2.0), g, t.text);
    let mut caret_x = x + gw;
    // The IME's composition text sits after the buffer (underlined) until it is committed.
    if !app.cmd.preedit.is_empty() {
        let pg = p.layout_no_wrap(app.cmd.preedit.clone(), font, t.text);
        let pw = pg.size().x;
        p.galley(pos2(caret_x, bar.center().y - pg.size().y / 2.0), pg, t.text);
        p.line_segment([pos2(caret_x, bar.bottom() - 7.0), pos2(caret_x + pw, bar.bottom() - 7.0)], Stroke::new(1.0, t.accent));
        caret_x += pw;
    }
    // Caret.
    let blink = (ui.input(|i| i.time) * 2.0).floor() as i64 % 2 == 0;
    if blink {
        p.line_segment([pos2(caret_x + 1.0, bar.top() + 5.0), pos2(caret_x + 1.0, bar.bottom() - 5.0)], Stroke::new(1.0, t.text));
    }
    // Where the IME's composition window follows: the whole command line and the caret in it.
    let caret = Rect::from_min_size(pos2(caret_x + 1.0, bar.top() + 5.0), vec2(1.0, h - 10.0));
    app.cmd.ime_rect = Some((bar, caret));
    // History chevron.
    crate::icons::paint(
        &p,
        Rect::from_center_size(pos2(bar.right() - 14.0, bar.center().y), vec2(14.0, 14.0)),
        if app.cmd.expanded { crate::icons::Icon::ChevronDown } else { crate::icons::Icon::ChevronUp },
        false,
    );
    // Bilingual command keywords must be visible even on narrow windows.
    // This is a foreground, wrapped presentation of *actual* Prompt.keywords:
    // never a translated keyword sent to the engine, never a new CAD command.
    if let Some(pr)=app.session.current_prompt()
        && !pr.raw && !pr.keywords.is_empty()
    {
        let keywords=native_bilingual_options(&pr);
        let mut chosen:Option<String>=None;
        let available_w=(bar.width()-10.0).max(180.0);
        let pos=pos2(bar.left(),
            (bar.top()-115.0-(n as f32*16.0)
                -if lines.is_empty(){0.0}else{6.0}).max(canvas.top()+4.0));
        egui::Area::new(egui::Id::new("cad360_bilingual_prompt_options"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(),|ui|{
                egui::Frame::popup(ui.style()).show(ui,|ui|{
                    ui.set_max_width(available_w);
                    ui.label(egui::RichText::new("OPTIONS (Tùy chọn lệnh gốc) · Click or type (Nhấp hoặc nhập)")
                        .small().strong());
                    ui.horizontal_wrapped(|ui|{
                        for (index,(keyword,caption)) in keywords.iter().enumerate() {
                            if ui.push_id(index,|ui|{
                                ui.small_button(caption)
                                    .on_hover_text(format!("Send {} to CAD (Gửi nguyên văn {})",keyword,keyword))
                                    .clicked()
                            }).inner {
                                chosen=Some(keyword.clone());
                            }
                        }
                    });
                });
            });
        if let Some(keyword)=chosen {
            if let Err(err)=app.session.input(Input::Keyword(keyword)) {
                app.session.echo_error(err.to_string());
            }
        }
    }
    // AutoComplete list.
    if !app.session.prompting() {
        let sug = suggestions(&app.cmd.buffer);
        if !sug.is_empty() {
            let rh = 20.0;
            let lr = Rect::from_min_size(
                pos2(
                    bar.left() + 24.0,
                    bar.top() - 6.0 - rh * sug.len() as f32 - (if lines.is_empty() { 0.0 } else { 16.0 * lines.len() as f32 + 6.0 }),
                ),
                vec2((bar.width() - 24.0).max(0.0), rh * sug.len() as f32),
            );
            p.rect_filled(lr, 3.0, t.list_bg);
            p.rect_stroke(lr, 3.0, Stroke::new(1.0, t.cmd_border), egui::StrokeKind::Inside);
            for (i, s) in sug.iter().enumerate() {
                let r = Rect::from_min_size(pos2(lr.left(), lr.top() + rh * i as f32), vec2(lr.width(), rh));
                let resp = ui.interact(r, ui.scope_id().with(("sug", i)), Sense::click());
                if resp.hovered() || i == 0 {
                    p.rect_filled(r.shrink(1.0), 2.0, if resp.hovered() { t.cmd_keyword_hover } else { t.list_row });
                }
                p.text(Pos2::new(r.left() + 8.0, r.center().y), egui::Align2::LEFT_CENTER, &s.display, crate::theme::body(), t.text);
                p.text(
                    Pos2::new(r.right() - 8.0, r.center().y),
                    egui::Align2::RIGHT_CENTER,
                    bilingual_command_caption(s),
                    crate::theme::small(),
                    t.text_faint,
                );
                if resp.clicked() {
                    app.cmd.buffer.clear();
                    app.start(&s.id.to_ascii_lowercase());
                }
            }
        }
    }
}

/// The command line's areas, laid out like [`show`] draws them.
struct Zones {
    /// The inline history lines (when there are any and the window is closed).
    lines: Option<Rect>,
    /// The expanded history window.
    window: Option<Rect>,
    chevron: Rect,
    /// The top edge that resizes the inline history.
    grip: Rect,
    /// Everything, AutoComplete list included: kept away from the drawing underneath.
    all: Rect,
}

fn zones(app: &CadApp, canvas: Rect, bar: Rect) -> Zones {
    let n = if app.cmd.expanded { 0 } else { visible_lines(app, canvas, bar).min(app.session.log.len()) };
    let lines =
        (n > 0).then(|| Rect::from_min_size(pos2(bar.left(), bar.top() - LINE_H * n as f32 - 4.0), vec2(bar.width(), LINE_H * n as f32 + 2.0)));
    let window = app.cmd.expanded.then(|| {
        // As tall as the log (at least three lines), at most half the drawing area.
        let full = LINE_H * app.session.log.len().max(3) as f32 + 4.0;
        let h = full.min((canvas.height() * 0.5).max(3.0 * LINE_H));
        Rect::from_min_size(pos2(bar.left(), bar.top() - h - 4.0), vec2(bar.width(), h + 2.0))
    });
    let top = window.or(lines).map_or(bar.top(), |r| r.top());
    let grip = Rect::from_min_max(pos2(bar.left(), top - 3.0), pos2(bar.right(), top + 3.0));
    let mut all = bar.union(grip);
    if !app.session.prompting() {
        let k = suggestions(&app.cmd.buffer).len();
        if k > 0 {
            let above = if n == 0 { 0.0 } else { LINE_H * n as f32 + 6.0 };
            all = all.union(Rect::from_min_size(pos2(bar.left(), bar.top() - 6.0 - 20.0 * k as f32 - above), vec2(bar.width(), 1.0)));
        }
    }
    Zones { lines, window, chevron: Rect::from_center_size(pos2(bar.right() - 14.0, bar.center().y), vec2(24.0, 22.0)), grip, all }
}

/// The pointer on the command line: one widget under all of it keeps clicks, drags and the wheel
/// off the drawing (the keywords and AutoComplete rows sit on top of it). Clicking the history
/// lines or the chevron opens/closes the history window (`ui.toggle.history`, also F2); dragging
/// the top edge sets how many lines show (`ui.cmdline.lines`).
fn history(app: &mut CadApp, ui: &mut egui::Ui, canvas: Rect, bar: Rect) {
    let z = zones(app, canvas, bar);
    let resp = ui.interact(z.all, ui.scope_id().with("cmdline"), Sense::click_and_drag());
    let at = resp.hover_pos().or(resp.interact_pointer_pos());
    let on_grip = |p: Pos2| z.grip.contains(p) && z.window.is_none();
    let on_toggle = |p: Pos2| z.chevron.contains(p) || z.lines.is_some_and(|r| r.contains(p));
    if app.cmd.resize_from.is_some() || at.is_some_and(on_grip) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    } else if at.is_some_and(on_toggle) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.drag_started()
        && let Some(o) = ui.input(|i| i.pointer.press_origin())
        && on_grip(o)
    {
        app.cmd.resize_from = Some((o.y, visible_lines(app, canvas, bar)));
    }
    if let Some((y0, n0)) = app.cmd.resize_from {
        if let (true, Some(p)) = (resp.dragged(), resp.interact_pointer_pos()) {
            let n = (n0 as f32 + (y0 - p.y) / LINE_H).round().clamp(0.0, MAX_LINES as f32) as usize;
            if n != app.ui.history_lines {
                let _ = app.run("ui.cmdline.lines", serde_json::json!({ "lines": n }));
            }
        } else {
            app.cmd.resize_from = None;
        }
    }
    if resp.clicked() && resp.interact_pointer_pos().is_some_and(on_toggle) {
        let _ = app.run("ui.toggle.history", serde_json::json!({}));
    }
    crate::shortcut_menus::attach(app, &resp, crate::shortcut_menus::command_line_entries);
    if let Some(r) = zones(app, canvas, bar).window {
        history_window(app, ui, r);
    }
}

/// The expanded history: the whole command log, scrollable, newest at the bottom.
fn history_window(app: &CadApp, ui: &mut egui::Ui, r: Rect) {
    let t = Tokens::get();
    ui.painter().rect_filled(r, 3.0, t.cmd_bg);
    ui.painter().rect_stroke(r, 3.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let inner = r.shrink2(vec2(8.0, 3.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
    child.spacing_mut().item_spacing.y = 0.0;
    let log = &app.session.log;
    egui::ScrollArea::vertical().id_salt("cmd_history").stick_to_bottom(true).auto_shrink([false, false]).show_rows(
        &mut child,
        LINE_H,
        log.len(),
        |ui, rows| {
            for l in log.get(rows).into_iter().flatten() {
                let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), LINE_H), Sense::hover());
                let p = ui.painter().with_clip_rect(row.intersect(ui.clip_rect()));
                p.text(pos2(row.left(), row.center().y), egui::Align2::LEFT_CENTER, l, crate::theme::small(), t.text_dim);
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_engine::Session;
    use egui::{Event, PointerButton};

    #[test]
    fn compact_canvas_contains_history_controls_without_changing_line_preference() {
        let mut app = CadApp::new(Session::empty(), crate::Services::default());
        app.ui.history_lines = 12;
        for i in 0..20 {
            app.session.echo(format!("line {i}"));
        }
        let canvas = Rect::from_min_size(Pos2::ZERO, vec2(300.0, 180.0));
        let w = (canvas.width() * 0.48).clamp(360.0, 760.0).min((canvas.width() - 8.0).max(0.0));
        let bar = Rect::from_min_size(pos2(canvas.center().x - w / 2.0, canvas.bottom() - 34.0), vec2(w, 24.0));
        let z = zones(&app, canvas, bar);
        assert!(canvas.contains_rect(bar));
        assert!(canvas.contains_rect(z.chevron));
        assert!(z.lines.is_some_and(|r| canvas.contains_rect(r)));
        assert!(canvas.contains_rect(z.grip));
        assert_eq!(app.ui.history_lines, 12);
    }

    fn frame(app: &mut CadApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
        let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))), events, ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
        out
    }

    fn press(app: &mut CadApp, ctx: &egui::Context, at: Pos2, pressed: bool) {
        frame(app, ctx, vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: egui::Modifiers::default() }]);
    }

    #[test]
    fn ime_preedit_is_shown_and_its_commit_is_inserted_once() {
        use cadcraft_engine::geom::Vec2;
        let mut app = CadApp::new(Session::new(), crate::Services::default());
        app.run("text", serde_json::json!({"at": [0, 0], "text": "HELLO", "height": 2.5})).unwrap();
        app.start("textedit");
        app.session.input(Input::Point(Vec2::new(1.0, 1.0))).unwrap();
        assert_eq!(app.session.current_prompt().map(|p| p.message.clone()).as_deref(), Some("Enter new text"));
        let ctx = egui::Context::default();
        let out = frame(&mut app, &ctx, Vec::new());
        // The IME is enabled for the command line while it is collecting text.
        assert!(out.platform_output.ime.is_some());
        // Composition is shown at the command line but is not part of the text being edited.
        frame(&mut app, &ctx, vec![Event::Ime(egui::ImeEvent::Preedit { text: "zhong".into(), active_range_chars: None })]);
        assert_eq!(app.cmd.preedit, "zhong");
        assert!(app.cmd.buffer.is_empty());
        // The committed text replaces the pre-edit, exactly once.
        frame(&mut app, &ctx, vec![Event::Ime(egui::ImeEvent::Commit("中文".into()))]);
        assert!(app.cmd.preedit.is_empty());
        assert_eq!(app.cmd.buffer, "中文");
        let enter = |pressed| Event::Key { key: Key::Enter, physical_key: None, pressed, repeat: false, modifiers: egui::Modifiers::NONE };
        frame(&mut app, &ctx, vec![enter(true), enter(false)]);
        let text = app.session.doc().unwrap().model.iter().next().map(|e| match &e.kind {
            cadcraft_engine::doc::EntityKind::Text(t) => t.value.clone(),
            _ => String::new(),
        });
        assert_eq!(text.as_deref(), Some("中文"));
        // Back to selecting objects: the next frame lets the IME go, so it can't swallow ASCII typing.
        let out = frame(&mut app, &ctx, Vec::new());
        assert!(out.platform_output.ime.is_none());
    }

    #[test]
    fn history_expands_by_click_chevron_f2_and_resizes_by_drag_without_touching_the_drawing() {
        let mut app = CadApp::new(Session::empty(), crate::Services::default());
        app.start("ui.sample");
        for i in 0..20 {
            app.session.echo(format!("line {i}"));
        }
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, Vec::new());
        frame(&mut app, &ctx, Vec::new());
        let canvas = app.canvas.rect.unwrap_or(Rect::NOTHING);
        let w = (canvas.width() * 0.48).clamp(360.0, 760.0).min((canvas.width() - 8.0).max(0.0));
        let bar = Rect::from_min_size(pos2(canvas.center().x - w / 2.0, canvas.bottom() - 34.0), vec2(w, 24.0));
        let z = zones(&app, canvas, bar);
        let Some(lines) = z.lines else { panic!("three history lines show by default") };
        // A click on the history lines opens the history window, and doesn't start a selection.
        for (at, pressed) in [(lines.center(), true), (lines.center(), false)] {
            frame(&mut app, &ctx, vec![Event::PointerMoved(at)]);
            press(&mut app, &ctx, at, pressed);
        }
        frame(&mut app, &ctx, Vec::new());
        assert!(app.cmd.expanded);
        assert!(app.session.pending_window.is_none(), "the click stayed on the command line");
        // The chevron closes it; F2 and TEXTSCR open it.
        let chevron = z.chevron.center();
        press(&mut app, &ctx, chevron, true);
        press(&mut app, &ctx, chevron, false);
        frame(&mut app, &ctx, Vec::new());
        assert!(!app.cmd.expanded);
        let f2 = |pressed| Event::Key { key: Key::F2, physical_key: None, pressed, repeat: false, modifiers: egui::Modifiers::NONE };
        frame(&mut app, &ctx, vec![f2(true), f2(false)]);
        assert!(app.cmd.expanded);
        assert_eq!(app.run("ui.toggle.history", serde_json::json!({"on": false})), Ok(serde_json::json!({"on": false})));
        app.cmdline("textscr");
        assert!(app.cmd.expanded);
        let _ = app.run("ui.toggle.history", serde_json::json!({"on": false}));
        frame(&mut app, &ctx, Vec::new());
        // Dragging the top edge up by two lines shows two more lines (capped).
        let top = pos2(lines.center().x, lines.top() - 1.0);
        frame(&mut app, &ctx, vec![Event::PointerMoved(top)]);
        press(&mut app, &ctx, top, true);
        for dy in [8.0, 20.0, 2.0 * LINE_H] {
            frame(&mut app, &ctx, vec![Event::PointerMoved(top - vec2(0.0, dy))]);
        }
        press(&mut app, &ctx, top - vec2(0.0, 2.0 * LINE_H), false);
        frame(&mut app, &ctx, Vec::new());
        assert_eq!(app.ui.history_lines, 5);
        assert!(!app.cmd.expanded, "a drag isn't a click");
        assert!(app.session.pending_window.is_none());
        assert_eq!(app.run("ui.cmdline.lines", serde_json::json!({"lines": 99})), Ok(serde_json::json!({"lines": MAX_LINES})));
    }
}

#[cfg(test)]
mod delete_key_tests {
    use super::*;
    use cadcraft_doc::Handle;

    const PRESS: KeyPress = KeyPress { repeat: false, plain: true };

    #[test]
    fn mac_delete_erases_a_fresh_selection() {
        assert!(backspace_erases(true, PRESS, "", false, true, false));
        // Not on Windows/Linux, where Backspace never deletes objects.
        assert!(!backspace_erases(false, PRESS, "", false, true, false));
        // Nothing selected, a command running, or text on the command line: just edit text.
        assert!(!backspace_erases(true, PRESS, "", false, false, false));
        assert!(!backspace_erases(true, PRESS, "", true, true, false));
        assert!(!backspace_erases(true, PRESS, "m", false, true, false));
        // Option/Cmd+delete are text-editing gestures on macOS.
        assert!(!backspace_erases(true, KeyPress { plain: false, ..PRESS }, "", false, true, false));
    }

    #[test]
    fn holding_delete_to_clear_typing_never_erases() {
        // Auto-repeat after the last character is gone.
        assert!(!backspace_erases(true, KeyPress { repeat: true, ..PRESS }, "", false, true, false));
        // Typed, then cleared with separate presses: still not erased.
        assert!(!backspace_erases(true, PRESS, "", false, true, true));
    }

    #[test]
    fn typing_is_forgotten_when_the_selection_changes() {
        let mut c = CmdLine::default();
        c.watch_selection(&[Handle(1)]);
        c.buffer.push('m');
        c.note_typing();
        c.buffer.clear();
        c.watch_selection(&[Handle(1)]);
        assert!(c.typed_since_selection);
        c.watch_selection(&[Handle(1), Handle(2)]);
        assert!(!c.typed_since_selection);
    }
}

#[cfg(test)]
mod autocomplete_tests {
    use super::*;

    #[test]
    fn vietnamese_command_suggestions_accept_accents_and_plain_text() {
        let accented=suggestions("làm sạch bản vẽ");
        assert_eq!(accented[0].id,"PURGE");
        assert!(accented[0].display.contains("Làm sạch bản vẽ"));
        assert_eq!(suggestions("lam sach ban ve")[0].id,"PURGE");
        assert_eq!(suggestions("ve duong thang")[0].id,"LINE");
        assert_eq!(suggestions("đo khoảng cách")[0].id,"DIST");
        assert_eq!(suggestions("tim toa do")[0].id,"ID");
        assert_eq!(suggestions("ghi kich thuoc")[0].id,"DIMLINEAR");
        assert_eq!(suggestions("duong thang")[0].id,"LINE");
        assert_eq!(suggestions("hinh chu nhat")[0].id,"RECTANG");
        assert!(suggestions("lam sach xyz").is_empty());
    }

    #[test]
    fn vietnamese_ime_works_at_idle_command_but_not_inside_numeric_point_prompts() {
        assert!(command_line_needs_ime(false,false));
        assert!(command_line_needs_ime(true,true));
        assert!(!command_line_needs_ime(true,false));
        assert_eq!(fold_vi("Làm sạch bản vẽ"),"lam sach ban ve");
    }

    #[test]
    fn expanded_vietnamese_command_vocabulary_resolves_only_existing_engine_commands() {
        let mut unique=std::collections::HashSet::new();
        let mut known=0usize;
        for (phrase,id,_) in VI_COMMANDS {
            assert!(unique.insert(*phrase),"duplicate Vietnamese phrase {phrase}");
            if cadcraft_engine::cmd::find_command(id).is_some() {known+=1;}
        }
        assert!(known>=25,"expected real native commands in Vietnamese dictionary");
        for (phrase,id) in [
            ("bo tron goc","fillet"),
            ("ve duong song song","offset"),
            ("ghi ban kinh","dimradius"),
            ("luu ban ve","qsave"),
            ("ve duong cong","spline"),
        ] {
            let expected=cadcraft_engine::cmd::find_command(id).is_some();
            let found=suggestions(phrase).iter().any(|s|s.id.eq_ignore_ascii_case(id));
            assert_eq!(found,expected,"incorrect Vietnamese phrase mapping: {phrase}");
        }
    }

    #[test]
    fn space_keeps_vietnamese_phrase_but_not_native_cad_alias() {
        assert!(vietnamese_query_continues("làm"));
        assert!(vietnamese_query_continues("lam sach"));
        assert!(vietnamese_query_continues("vẽ đường"));
        assert!(!vietnamese_query_continues("l"));
        assert!(!vietnamese_query_continues("pl"));
        assert!(!vietnamese_query_continues("rec"));
        assert!(!vietnamese_query_continues("lam sach ban ve"));
        assert_eq!(fold_vi("Làm Sạch BẢN VẼ"),"lam sach ban ve");
        assert!(active_command_guide("pline").contains("đỉnh"));
    }

    #[test]
    fn every_registered_command_has_english_first_in_app_guide() {
        let mut checked=0usize;
        for spec in cadcraft_engine::command_specs() {
            if spec.id.contains('.') {continue;}
            checked+=1;
            let guide=registered_active_command_help(spec.id);
            assert!(guide.contains(&spec.id.to_ascii_uppercase()),
                "Missing original command ID in help: {}",spec.id);
            assert!(!guide.trim().is_empty());
        }
        assert!(checked>100);
        let trim=registered_active_command_help("trim");
        assert!(trim.contains("TRIM / TR"));
        let number=active_prompt_help(&cadcraft_engine::Prompt::new("Radius",cadcraft_engine::Accept::NUMBER));
        assert!(number.contains("Enter number"));
    }

    #[test]
    fn native_entity_kind_guides_related_commands_without_inventing_geometry() {
        use cadcraft_doc::{EntityKind as E,Line,Circle};
        let line=E::Line(Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::ZERO});
        let circle=E::Circle(Circle{center:cadcraft_geom::Vec3::ZERO,radius:1.0});
        assert_eq!(native_selection_kind(&[]),SelectionKind::None);
        assert_eq!(native_selection_kind(&[&line]),SelectionKind::Line);
        assert_eq!(native_selection_kind(&[&circle]),SelectionKind::Curve);
        assert_eq!(native_selection_kind(&[&line,&circle]),SelectionKind::Mixed);
        assert!(SelectionKind::Mixed.bilingual_caption().contains("Nhiều loại"));
        let line_commands=selection_kind_next_commands(SelectionKind::Line,None,&[]);
        assert!(line_commands.iter().any(|id|id=="offset"||id=="trim"));
        let circle_commands=selection_kind_next_commands(SelectionKind::Curve,None,&[]);
        assert!(circle_commands.iter().any(|id|id=="trim"||id=="offset"));
        let mixed=selection_kind_next_commands(SelectionKind::Mixed,None,&[]);
        assert_eq!(mixed.first().map(String::as_str),Some("move"));
        for kind in [SelectionKind::None,SelectionKind::Line,SelectionKind::Curve,
                     SelectionKind::Polyline,SelectionKind::Text,SelectionKind::Block,
                     SelectionKind::Dimension,SelectionKind::Hatch,SelectionKind::Mixed] {
            let candidates=selection_kind_next_commands(kind,None,&[]);
            assert!(candidates.iter().all(|id|
                cadcraft_engine::cmd::find_command(id).is_some()));
        }
    }

    #[test]
    fn selection_context_prioritizes_only_real_native_edit_commands() {
        let history=vec!["l".into(),"di".into()];
        let selected=selection_aware_next_commands(2,Some("line"),&history);
        assert_eq!(selected.first().map(String::as_str),Some("move"));
        assert!(selected.contains(&"copy".to_owned()));
        assert!(selected.iter().all(|id|cadcraft_engine::cmd::find_command(id).is_some()));
        let nothing=selection_aware_next_commands(0,None,&[]);
        assert!(!nothing.is_empty());
        assert_eq!(nothing.first().map(String::as_str),Some("line"));
        assert!(!nothing.contains(&"erase".to_owned()));
    }

    #[test]
    fn next_command_suggestions_are_registry_backed_and_have_actual_shortcuts() {
        let history=vec!["l".to_string(),"di".to_string(),"l".to_string()];
        for id in next_command_habits("line",&history) {
            assert!(cadcraft_engine::cmd::find_command(&id).is_some(),
                "cannot suggest nonexistent CAD command: {id}");
            let caption=related_command_caption(&id).expect("registered command caption");
            assert!(!caption.is_empty());
        }
        assert_eq!(shortest_registered_alias("line"),Some("l"));
        assert_eq!(shortest_registered_alias("pline"),Some("pl"));
        assert_eq!(shortest_registered_alias("dist"),Some("di"));
    }

    #[test]
    fn next_command_habits_use_only_observed_verified_session_commands() {
        let session=vec!["l","di","l","di","l","pl","l"].into_iter()
            .map(String::from).collect::<Vec<_>>();
        let suggested=next_command_habits("line",&session);
        assert_eq!(suggested[0],"dist");
        assert_eq!(suggested[1],"pline");
        assert_eq!(command_from_history("làm sạch bản vẽ"),Some("purge".into()));
        assert_eq!(command_from_history("lam sach ban ve"),Some("purge".into()));
        assert!(command_from_history("invented_ai_command").is_none());
        assert!(suggested.iter().all(|id|cadcraft_engine::cmd::find_command(id).is_some()));
    }

    #[test]
    fn every_public_native_command_remains_searchable_by_full_id() {
        // This covers the engine's entire public registered command catalog,
        // not an arbitrary handpicked list. Internal dotted RPCs are excluded.
        let mut count=0usize;
        for spec in cadcraft_engine::command_specs() {
            if spec.id.contains('.') {continue;}
            count+=1;
            let matched=suggestions(spec.id);
            assert!(matched.iter().any(|suggestion|
                suggestion.id.eq_ignore_ascii_case(spec.id)),
                "native CAD command disappeared from search: {}",spec.id);
        }
        assert!(count>100,"engine command registry was unexpectedly empty");
    }

    #[test]
    fn full_bilingual_option_palette_keeps_all_original_keywords() {
        let source=["Undo","Close","Arc","Width","Length","Radius","Multiple","Exit"];
        let prompt=cadcraft_engine::Prompt::new("Specify next point",cadcraft_engine::Accept::POINT)
            .kw(&source);
        let options=native_bilingual_options(&prompt);
        assert_eq!(options.len(),source.len());
        for (i,(native,caption)) in options.iter().enumerate() {
            assert_eq!(native,source[i]);
            assert!(caption.starts_with(native));
            assert!(!caption.is_empty());
        }
        assert_eq!(options[0].1,"Undo (Hoàn tác)");
        assert_eq!(options[1].1,"Close (Đóng)");
        assert_eq!(options[3].1,"Width (Bề rộng)");
    }

    #[test]
    fn native_keyword_catalog_never_changes_when_rendered_bilingually() {
        let words=["Undo","Close","Arc","Width","Length"];
        let before=words.iter().map(ToString::to_string).collect::<Vec<_>>();
        let captions=before.iter().map(|k|bilingual_keyword_caption(k)).collect::<Vec<_>>();
        assert_eq!(before[0],"Undo");
        assert_eq!(before[1],"Close");
        assert_eq!(captions[0],"Undo (Hoàn tác)");
        assert_eq!(captions[1],"Close (Đóng)");
        assert_eq!(before.len(),captions.len());
        assert!(captions.iter().all(|s|!s.is_empty()));
    }

    #[test]
    fn native_prompt_keyword_input_stays_english_when_caption_is_bilingual() {
        assert_eq!(bilingual_keyword_caption("Undo"),"Undo (Hoàn tác)");
        assert_eq!(bilingual_keyword_caption("Close"),"Close (Đóng)");
        assert_eq!(bilingual_keyword_caption("UnknownKeyword"),"UnknownKeyword");
    }

    #[test]
    fn original_english_prompts_preserved_with_vietnamese_in_parentheses() {
        assert_eq!(bilingual_prompt_caption("Specify first point"),
            "Specify first point (Chọn điểm thứ nhất (nhấp chuột hoặc nhập X,Y))");
        assert_eq!(bilingual_prompt_caption("No translation registered"),
            "No translation registered");
    }

    #[test]
    fn native_alias_space_dispatch_is_never_replaced_by_translation() {
        for alias in ["l","pl","c","rec","co","di"] {
            assert!(!vietnamese_query_continues(alias),
                "must preserve native space-to-enter alias: {alias}");
            assert!(!suggestions(alias).is_empty(),
                "native alias must remain searchable: {alias}");
        }
        assert_eq!(suggestions("pl")[0].id,"PLINE");
        assert_eq!(suggestions("rec")[0].id,"RECTANG");
        assert_eq!(suggestions("di")[0].id,"DIST");
    }

    #[test]
    fn bilingual_ui_keeps_english_first_and_actual_aliases() {
        let line=suggestions("l");
        assert_eq!(line[0].id,"LINE");
        let caption=bilingual_command_caption(&line[0]);
        assert!(caption.starts_with("Line ("));
        assert!(caption.contains("L"));
        assert!(caption.contains("Đường thẳng")||caption.contains("Vẽ đường thẳng"));
        let purge=suggestions("lam sach ban ve");
        assert_eq!(purge[0].id,"PURGE");
        let label=bilingual_command_caption(&purge[0]);
        assert!(label.starts_with("Purge ("));
        assert!(suggested_next_commands("line").contains(&"dist"));
        assert!(related_command_caption("dist").is_some());
        assert!(related_command_caption("this_command_does_not_exist").is_none());
    }

    #[test]
    fn exact_alias_is_first_suggestion_with_alias_display() {
        let rec = suggestions("rec");
        assert!(!rec.is_empty());
        assert_eq!(rec[0].id, "RECTANG");
        assert_eq!(rec[0].display, "REC (RECTANG)");
        assert_eq!(rec[0].label, "Rectangle");
        // RECOVER should follow RECTANG, not precede it
        assert!(rec.iter().any(|s| s.id == "RECOVER"));
        assert_eq!(rec[1].id, "RECOVER");
        assert_eq!(rec[1].display, "RECOVER");

        let l = suggestions("l");
        assert_eq!(l[0].id, "LINE");
        assert_eq!(l[0].display, "L (LINE)");

        let c = suggestions("c");
        assert_eq!(c[0].id, "CIRCLE");
        assert_eq!(c[0].display, "C (CIRCLE)");

        let co = suggestions("co");
        assert_eq!(co[0].id, "COPY");
        assert_eq!(co[0].display, "CO (COPY)");
    }

    #[test]
    fn prefix_without_alias_shows_command_id() {
        let rect = suggestions("rect");
        assert_eq!(rect[0].id, "RECTANG");
        assert_eq!(rect[0].display, "RECTANG");

        let line = suggestions("line");
        assert_eq!(line[0].id, "LINE");
        assert_eq!(line[0].display, "LINE");
    }

    #[test]
    fn tab_autocompletes_alias_to_resolved_command() {
        let mut app = CadApp::new(cadcraft_engine::Session::empty(), crate::Services::default());
        app.cmd.buffer = "rec".to_string();
        tab(&mut app);
        assert_eq!(app.cmd.buffer, "RECTANG");
    }
}
