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
        ("5.4 Tiền kiểm quyền AI Agent — ai_cad_preflight", "Trước khi gọi công cụ CAD, Agent có thể hỏi ai_cad_preflight về loại thao tác. Kết quả allowed = chỉ đọc; preview_only = chưa đủ xác nhận; denied = công cụ không được duyệt; requires_backend_checks = vẫn cần CAD Engine kiểm tra UID, epoch, revision và quyền người dùng.\nTiền kiểm chỉ cung cấp thông tin, KHÔNG cấp quyền, không thay xác thực host, không tự chạy lệnh, không yêu cầu cookie hay API key."),
        ("5.5 Chính sách MCP readonly / reviewed", "Thiết lập biến môi trường CAD360_MCP_ACCESS_MODE=readonly để giới hạn Agent chỉ đọc bản vẽ; dùng reviewed nếu muốn cho phép AI Preview và thao tác vẽ có xác nhận.\nTrong reviewed, các lệnh execute, script và command_line bị từ chối. Commit của AI phải có confirm=true và UID/epoch/revision. CAD Engine vẫn kiểm tra thực tế.\nChế độ legacy giữ tương thích các tích hợp cũ, không phải cấu hình bảo vệ dành cho Agent không tin cậy."),
        ("5.6 Nhật ký quyền và bảo vệ dữ liệu", "MCP Server ghi CAD360_MCP_AUDIT vào stderr gồm mode, tên công cụ, allow/deny, không ghi tài liệu, prompt, token, tham số. Host phải kiểm tra danh tính/role/tenant và bảo vệ log.\nMCP policy không phải hệ thống xác thực và không thay thế bước kiểm tra trên EXE thật."),
        ("5.7 Ràng buộc User/Role/Tenant trên MCP Host", "Host tin cậy sử dụng HostScope::new(tenantId,userId,role,documentUid), sau đó Server::with_host_scope. Viewer chỉ xem; Editor và Approver có thể Preview và đề xuất thao tác AI nhưng muốn commit vẫn phải có confirm và UID/epoch/revision.\nKhông nhận userId, tenantId hay role từ JSON-RPC do Agent tự khai báo. HostScope chỉ ràng buộc ngữ cảnh — cần OAuth, RBAC và xác thực tenant ở host; không phải cơ chế cách ly tenant trong CAD Engine."),
        ("5.8 Kiểm tra sai bản vẽ và dữ liệu MCP", "Khi mở tài nguyên cadcraft://drawing hoặc gọi tool trên phiên HostScope, MCP Server kiểm tra documentUid đang mở với documentUid đã được host tin cậy ràng buộc. Khác UID phải từ chối trước khi đọc hoặc ghi.\nTest Stage245 dùng viewer và tài liệu sai UID; vẫn cần Windows CI PASS và thử với host thực."),
        ("5.9 Giới hạn thời gian phiên MCP", "Phiên HostScope mặc định hết hạn sau 15 phút. Host đã kiểm chứng OIDC có thể đặt thời hạn ngắn hơn/không quá 1 giờ bằng with_verified_expiry. Quá hạn phải từ chối ngay, xác thực lại tại host; không tự gia hạn bằng dữ liệu Agent gửi. Nếu nhập sai cấu hình CAD360_MCP_ACCESS_MODE, hệ thống dùng readonly thay vì vô tình cấp quyền legacy."),
        ("5.10 Danh sách công cụ MCP theo quyền", "Host có thể dùng restrict_tools chỉ chọn danh sách công cụ từng phiên. Quyền bị hạn chế ở Server trước khi chạy CAD. tools/list chỉ hiển thị công cụ được cấp. Viewer không thể tăng quyền dù host gửi tên công cụ AI vẽ. Thử bằng inspect_drawing, ai_draw_plan_vietnamese và lệnh execute."),
        ("5.11 Giới hạn tài nguyên MCP theo quyền", "Phiên Agent được host cấp quyền inspect_drawing chỉ có thể xem cadcraft://drawing. Tài nguyên cadcraft://commands đòi quyền list_commands. MCP resources/list phải lọc danh sách theo quyền đã cấp, resources/read phải kiểm tra quyền trước khi đọc.\nNếu phiên hết hạn hoặc người dùng chuyển sang bản vẽ khác UID, Server phải từ chối ngay. Đây là khóa quyền trong MCP Server, KHÔNG phải tích hợp OIDC end-to-end hoặc RLS theo Tenant."),
        ("5.12 Kết nối OIDC từ host đã xác minh", "Host tích hợp với AuthServer OpenIddict phải xác minh chữ ký JWT/JWKS, issuer, audience, nbf, exp, thu hồi và quyền/tenant. Sau đó thực hiện VerifiedTokenProvider::verify_access_token và tạo OidcTrustPolicy cho đúng HTTPS issuer/audience.\nServer::with_verified_oidc chỉ nhận claims đã được host xác minh, tiếp tục giới hạn thời hạn ≤15 phút, vai trò và danh sách công cụ được cấp. Mã CAD 360 chưa tự xác minh chữ ký JWT, chưa có giao diện đăng nhập OIDC hoàn chỉnh. Không gửi token vào prompt/log, không sử dụng cookie phiên chat web."),
        ("5.13 Kiểm tra OIDC bằng host .NET 8", "Mẫu integrations/oidc-host-dotnet dùng ASP.NET Core JwtBearer, truy vấn JWKS tại issuer HTTPS để kiểm tra chữ ký, audience, issuer, exp, nbf và yêu cầu tenant_id / sub / role. Đặt CAD360_OIDC_AUTHORITY và CAD360_OIDC_AUDIENCE trong môi trường, chạy dotnet run tại máy thử.\nAPI /api/cad360/identity chỉ trả danh tính đã xác minh; mcpSessionCreated=false và không điều khiển CAD. Muốn kết nối AI thật phải có kênh đáng tin cậy từ host đã xác thực sang Rust MCP có kiểm soát quyền/UID. Không dùng cookie tài khoản chat và không lưu token vào bản vẽ."),
        ("5.14 OIDC host đọc CAD qua MCP readonly", "Host .NET 8 đã có GET /api/cad360/mcp/read-drawing. Middleware xác minh Bearer token từ issuer HTTPS/JWKS, host đối chiếu tenant_id với CAD360_LOCAL_TENANT_ID và role viewer/editor/approver. Chỉ cho phép đọc inspect_drawing từ tiến trình CAD CLI chạy trên loopback có mã bảo vệ riêng.\nCấu hình CAD360_MCP_CLI_PATH trỏ tuyệt đối đến cadcraft-cli.exe, CAD360_MCP_CONNECT_ADDRESS=127.0.0.1:7979, chạy host và CAD cùng tài khoản hệ điều hành. Host không nhận lệnh shell/tool tùy ý từ Agent, không được phép sửa CAD. Đây là bridge đọc thử, chưa có nghiệm thu EXE/host thật và chưa có kênh quyền ghi."),
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
fn lesson_diagram(ui:&mut egui::Ui,lesson:&str) {
    let width=ui.available_width().min(540.0).max(220.0);
    let (rect,_)=ui.allocate_exact_size(egui::vec2(width,180.0),egui::Sense::hover());
    let painter=ui.painter_at(rect);
    painter.rect_filled(rect,5.0,egui::Color32::from_rgb(17,36,56));
    let mid=rect.center();
    let white=egui::Stroke::new(2.5,egui::Color32::from_rgb(98,205,239));
    let yellow=egui::Stroke::new(2.5,egui::Color32::from_rgb(246,192,98));
    let green=egui::Stroke::new(2.0,egui::Color32::from_rgb(91,229,165));
    if lesson.contains("Đường thẳng")||lesson.contains("LINE") {
        painter.line_segment([egui::pos2(rect.left()+38.0,mid.y+47.0),egui::pos2(rect.right()-36.0,mid.y-46.0)],white);
    }else if lesson.contains("vuông")||lesson.contains("chữ nhật")||lesson.contains("RECTANG") {
        painter.rect_stroke(egui::Rect::from_center_size(mid,egui::vec2(154.0,124.0)),0.0,white,egui::StrokeKind::Outside);
    }else if lesson.contains("tròn")||lesson.contains("CIRCLE") {
        painter.circle_stroke(mid,58.0,white);
        painter.circle_filled(mid,4.0,egui::Color32::WHITE);
    }else if lesson.contains("ARC") {
        for i in 0..24 {
            let a=i as f32*std::f32::consts::FRAC_PI_2/24.0;
            let b=(i+1) as f32*std::f32::consts::FRAC_PI_2/24.0;
            painter.line_segment([mid+egui::vec2(a.cos()*65.0,-a.sin()*65.0),
                mid+egui::vec2(b.cos()*65.0,-b.sin()*65.0)],yellow);
        }
    }else {
        let outline=egui::Rect::from_center_size(mid,egui::vec2(248.0,125.0));
        painter.rect_stroke(outline,0.0,white,egui::StrokeKind::Inside);
        let x=mid.x-10.0;
        painter.line_segment([egui::pos2(x,outline.top()),egui::pos2(x,mid.y-20.0)],yellow);
        painter.line_segment([egui::pos2(x,mid.y+18.0),egui::pos2(x,outline.bottom())],yellow);
        painter.line_segment([egui::pos2(x,mid.y-20.0),egui::pos2(x+40.0,mid.y-20.0)],green);
    }
    painter.text(egui::pos2(rect.left()+12.0,rect.bottom()-17.0),egui::Align2::LEFT_CENTER,
        "SƠ ĐỒ MINH HỌA — KHÔNG PHẢI ẢNH CHỤP EXE",
        egui::FontId::proportional(10.0),egui::Color32::from_rgb(182,201,216));
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
            lesson_diagram(ui,&state.selected);
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
