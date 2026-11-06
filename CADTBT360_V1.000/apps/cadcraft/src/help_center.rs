//! Embedded offline CAD 360 lessons: hierarchical browser backed by the full
//! repository manual.  The content is compiled into the EXE; no network required.
use egui::{self, RichText};

const FULL_GUIDE: &str = include_str!("../../../../docs/user-guide/CADTBT360_V1.025_GIAO_TRINH_TONG_HOP_TAT_CA_TINH_NANG.md");

const GROUPS: &[(&str, &[(&str, &str)])] = &[
    ("1. NHÓM LỆNH VẼ", &[
        ("1.1 Đường thẳng — LINE", "LINE: nhập hai điểm 0,0 và 5000,0; Enter kết thúc. Kiểm tra bằng DIST.\nLệnh LINE sinh entity có thể chỉnh sửa, không phải ảnh."),
        ("1.2 Hình vuông / chữ nhật — RECTANG", "RECTANG: nhập góc thứ nhất 0,0; góc thứ hai 4000,4000 để được hình vuông. Nhập 6000,4000 để được hình chữ nhật.\nKiểm tra tọa độ trong Properties."),
        ("1.3 Đường tròn — CIRCLE", "CIRCLE: chọn tâm 0,0, nhập bán kính 500. Dùng DIST để kiểm tra bán kính và thử Undo trên bản vẽ sao."),
        ("1.4 Cung tròn — ARC", "ARC: khai báo tâm, bán kính, góc bắt đầu và kết thúc. Trên cửa AI, ARC minh họa cung mở và được đưa vào BLOCK khi được xác nhận."),
        ("1.5 Đa tuyến — PLINE", "PLINE: nhập chuỗi đỉnh, dùng Close khi muốn hình khép kín. Kiểm tra độ kín trước khi tính diện tích."),
        ("1.6 Tường, Layer, BLOCK", "Dùng OFFSET/TRIM để vẽ tường và khoảng mở. Phân lớp A-TUONG/A-CUA, tạo BLOCK và thử INSERT."),
    ]),
    ("2. SỬ DỤNG AI", &[
        ("2.1 Đọc ảnh vào", "Trên tab CAD chọn AI TỪ ẢNH → CAD; chọn PNG/JPEG từ ổ đĩa. Không tự thay thế ảnh bằng đường CAD; cần bước vector hóa."),
        ("2.2 Xử lý ảnh", "Hiệu chỉnh tỷ lệ mm/pixel, vùng chọn, xoay/biến dạng phối cảnh nếu công cụ hiện hành hỗ trợ; kiểm tra độ tương phản trước khi dựng LINE.\nKhông cam kết nhận dạng tường/cửa tự động khi chưa kiểm thử."),
        ("2.3 AI vẽ từ ảnh", "Bắt đầu bằng Preview, kiểm tra điểm đầu/cuối và độ dài; xác nhận thì sinh LINE chỉnh sửa được. Lưu DXF/DWG thử và mở lại."),
        ("2.4 AI vẽ bằng tiếng Việt", "MCP: ai_draw_plan_vietnamese. Ví dụ: vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0 cửa 900 giữa bản lề dưới mở phải.\nPreview mặc định; commit yêu cầu confirm=true cùng UID/epoch/revision."),
        ("2.5 Cửa BLOCK + Layer", "Bật createDoorBlock=true và applyLayers=true khi vẽ hai phòng có mô tả bản lề. AI tạo LINE/ARC cửa, chuyển thành BLOCK/INSERT, gán TBT_AI_ROOM, TBT_AI_WALL, TBT_AI_DOOR."),
        ("2.6 AI chỉnh sửa an toàn", "Nếu revision thay đổi sau Preview, phải tạo Preview mới. Chỉ ghi vào bản vẽ có xác nhận; giữ nhóm Undo và không ghi đè DWG gốc."),
        ("2.7 MCP / CLI / kết nối ngoài", "CAD 360 hỗ trợ MCP và CLI để Agent gọi lệnh CAD qua công cụ có cấu trúc. Kết nối ChatGPT/Gemini qua ứng dụng/bridge hợp lệ: không giả định tài khoản chat là một API key.\nChỉ cấp quyền trong phạm vi người dùng cho phép; token cần được giữ ngoài bản vẽ và giáo trình."),
        ("2.8 Cải thiện AI từ phản hồi", "Lưu tập tình huống đã được người dùng đồng ý, kết quả preview/confirm và các bài kiểm thử. Dùng để đánh giá/rút kinh nghiệm hoặc huấn luyện mô hình bên ngoài khi có quy trình rõ ràng.\nPhần mềm không thể tự sửa trọng số của mô hình ChatGPT/Gemini hoặc tự cập nhật mã không qua duyệt và kiểm thử."),
    ]),
    ("3. QUẢN LÝ BẢN VẼ", &[
        ("3.1 Layer", "Tạo Layer từ Layer Palette; đặt current, đổi màu/linetype, Lock/Freeze. Kiểm tra Properties trên entity."),
        ("3.2 BLOCK / INSERT", "Chọn entity, dùng BLOCK với tên và điểm gốc; INSERT vào vị trí mới. Giữ đối tượng chỉnh sửa được."),
        ("3.3 DWG / DXF", "SAVEAS vào file thử; đóng rồi mở lại. Kiểm tra Layer, BLOCK, tọa độ. Kiểm thử hai chiều AutoCAD phải thực hiện độc lập."),
        ("3.4 Undo và Recovery", "Dùng UNDO/REDO và bản tự lưu. Trước thao tác AI, kiểm tra revision; bản vẽ bị thay đổi cần Preview mới."),
    ]),
    ("4. ĐO BÓC VÀ BIM", &[
        ("4.1 Đo chiều dài / diện tích", "Chọn nguồn CAD thật, kiểm tra đơn vị và số đo. Giữ nguyên handle để truy nguyên khối lượng."),
        ("4.2 QTO và xuất báo cáo", "Đặt mã công tác, xác nhận kết quả đo, xuất Excel và kiểm tra công thức."),
        ("4.3 IFC / BIM", "Kiểm tra cấu kiện và thông tin IFC; phân biệt IFC model với hình học CAD 2D."),
    ]),
    ("5. KẾT NỐI & AN TOÀN AI", &[
        ("5.1 MCP kết nối ứng dụng bên ngoài", "Ưu tiên kết nối MCP stdio/Remote localhost, kiểm tra danh sách công cụ, quyền và phiên CAD. Không cho phép Agent gọi lệnh xóa hoặc lưu đè không xác nhận."),
        ("5.2 API và tài khoản ChatGPT / Gemini", "Gói chat trên web không mặc định cấp quyền API tự động. Chỉ dùng kết nối ngoài theo phương thức chính thức được nhà cung cấp và ứng dụng chấp nhận. Không thu thập cookie/phiên đăng nhập của trình duyệt."),
        ("5.3 Kho kiến thức và nâng cấp kiểm thử", "Quản lý giáo trình, các mẫu DWG/DXF được phép dùng, bộ câu lệnh và phản hồi lỗi. Phiên bản nâng cấp cần code review, kiểm thử CI và khả năng rollback."),
    ]),
];

#[derive(Default)]
pub struct State { search: String, selected: String, show_full_manual: bool }
fn matches(text:&str,query:&str)->bool {
    query.is_empty() || text.to_lowercase().contains(&query.to_lowercase())
}
fn full_lessons()->Vec<(String,String)> {
    let mut entries=Vec::new();
    let mut head:String=String::new();
    let mut body=String::new();
    for line in FULL_GUIDE.lines() {
        if line.trim().starts_with("### Bài ") {
            if !head.is_empty(){entries.push((std::mem::take(&mut head),std::mem::take(&mut body)));}
            head=line.trim().trim_start_matches("### ").to_string();
        } else if !head.is_empty() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if !head.is_empty() {entries.push((head,body));}
    entries
}
pub fn panel(ui:&mut egui::Ui,state:&mut State) {
    ui.heading("HƯỚNG DẪN CAD 360 — CÂY BÀI HỌC OFFLINE");
    ui.label("Nội dung nhúng trong EXE. Chọn một bài bên trái, xem thao tác chi tiết bên phải. Giáo trình tổng hợp luôn có sẵn để tra cứu.");
    ui.horizontal(|ui|{
        ui.label("Tìm bài:");
        ui.add(egui::TextEdit::singleline(&mut state.search).hint_text("LINE, AI ảnh, BLOCK, MCP, DWG...").desired_width(390.0));
        if ui.button("Xóa tìm kiếm").clicked() {state.search.clear();}
        ui.checkbox(&mut state.show_full_manual,"Hiện toàn bộ giáo trình");
    });
    ui.separator();
    let query=state.search.trim().to_lowercase();
    let full=if state.show_full_manual {full_lessons()}else{Vec::new()};
    ui.columns(2,|columns|{
        columns[0].set_min_width(260.0);
        egui::ScrollArea::vertical().id_salt("cad360_help_tree").show(&mut columns[0],|ui|{
            ui.heading("HƯỚNG DẪN");
            for &(group,items) in GROUPS {
                if !query.is_empty()&&!matches(group,&query)&&!items.iter().any(|(name,description)|matches(name,&query)||matches(description,&query)){continue;}
                egui::CollapsingHeader::new(group).default_open(true).show(ui,|ui|{
                    for &(name,description) in items {
                        if !query.is_empty()&&!matches(group,&query)&&!matches(name,&query)&&!matches(description,&query){continue;}
                        if ui.selectable_label(state.selected==name,name).clicked(){state.selected=name.to_string();}
                    }
                });
            }
            if state.show_full_manual {
                egui::CollapsingHeader::new(format!("6. GIÁO TRÌNH FULL — {} bài có nội dung",full.len())).default_open(false).show(ui,|ui|{
                    for (idx,(name,body)) in full.iter().enumerate(){
                        if !query.is_empty()&&!matches(name,&query)&&!matches(body,&query){continue;}
                        let key=format!("6.{} {}",idx+1,name);
                        if ui.selectable_label(state.selected==key,&key).clicked(){state.selected=key;}
                    }
                });
            }
        });
        egui::ScrollArea::vertical().id_salt("cad360_help_reader").show(&mut columns[1],|ui|{
            if state.selected.is_empty() {ui.heading("Chọn bài học từ cây bên trái");return;}
            ui.heading(&state.selected);
            let basic=GROUPS.iter().flat_map(|(_,items)|items.iter()).find(|(name,_)|*name==state.selected);
            if let Some((_,description))=basic {
                for line in description.lines(){ui.label(line);}
            }else if let Some((_,(_,body)))=full.iter().enumerate().find(|(idx,(name,_))|state.selected==format!("6.{} {}",idx+1,name)){
                for line in body.lines(){ui.label(line);}
            } else {ui.label("Chọn lại bài học trong cây hướng dẫn.");}
            ui.separator();
            ui.label(RichText::new("Lưu ý: hướng dẫn mô tả khả năng trong mã nguồn; các chức năng chưa có CI/EXE PASS không được coi là đã nghiệm thu.").small().italics());
        });
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn cad360_help_has_expected_hierarchy_and_offline_full_manual() {
        assert!(GROUPS.iter().any(|(g,items)|g.starts_with("1.")&&items.iter().any(|(x,_)|x.starts_with("1.1 Đường thẳng"))));
        assert!(GROUPS.iter().any(|(g,items)|g.starts_with("2.")&&items.iter().any(|(x,_)|x.starts_with("2.1 Đọc ảnh"))));
        assert!(GROUPS.iter().any(|(_,items)|items.iter().any(|(x,_)|x.contains("MCP"))));
        assert!(FULL_GUIDE.len()>30000);
        assert!(full_lessons().len()>=100);
    }
}
