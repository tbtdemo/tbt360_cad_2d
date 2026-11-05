# CAD 360 VIỆT NAM — TBT V1.025
## Giáo trình tổng hợp từ cài đặt đến các tính năng CAD, QTO, BIM và AI — Đợt 178

**Ngày:** 05/11/2026. **Đơn vị:** Công ty CP Công nghệ TBT Việt Nam.  
**Tình trạng:** tài liệu đối chiếu mã nguồn; Windows CI PASS, EXE hiện hành và ảnh chụp giao diện thật vẫn phải kiểm chứng. Không mô tả tính năng thử nghiệm như đã nghiệm thu.

**Tài liệu Word học viên:** Bản phát hành Windows tạo `CADTBT360_V1.025_GIAO_TRINH_TONG_HOP.docx` từ chính Markdown này và cài với tên `GIAO_TRINH_TONG_HOP_CAD360.docx`; bản trao đổi đợt 180 có tên `CADTBT360_V1.025_GiaoTrinh_TOAN_BO_TINH_NANG_Dot180.docx` (32 trang, 21 bài thực hành). Tài liệu AI chỉ là hướng dẫn chuyên đề, không thay thế giáo trình toàn phần mềm.

### 15 chương của giáo trình

| Chương | Các nội dung chính | Bài kiểm tra |
|---|---|---|
| 1. Bắt đầu | Cài Setup/Portable, 5 tab, NEW/OPEN/SAVEAS, lưu file | Tạo DXF và mở lại |
| 2. Môi trường CAD | Tọa độ, mm/m, ZOOM/PAN, chọn window/crossing, OSNAP/ORTHO | Vẽ và đo đoạn 5000 mm |
| 3. Vẽ 2D | LINE/PLINE, RECTANG/POLYGON, CIRCLE/ARC/ELLIPSE, HATCH, SPLINE, XLINE | Mặt bằng hình học |
| 4. Sửa CAD | MOVE/COPY/ROTATE, OFFSET/TRIM/EXTEND, FILLET/CHAMFER, SCALE/MIRROR/ARRAY, UNDO/REDO | So sánh trước/sau |
| 5. Tổ chức hồ sơ | Layer, Block/INSERT/Attributes, Xref, IMAGEATTACH/IMAGECLIP, DXF/DWG | Bản vẽ nhiều lớp |
| 6. Chữ và in | TEXT/MTEXT/STYLE, Dimension/DIMSTYLE, Model/Layout, PDF/PNG/SVG | Bản in A3 |
| 7. QTO cơ bản | Length/Area/Perimeter/Count, vùng chọn, mã công tác, review/confirm, nested Block/Array | Bảng đo có nguồn |
| 8. QTO chuyên sâu | Mặt bằng × mặt cắt, bóc đào/đắp theo cọc, Average End Area | A×H; V=(A1+A2)×L/2 |
| 9. QTO báo cáo | Chọn 12 cột, phân nhóm, đơn vị, Excel XLSX, CSV, truy nguyên handle | Báo cáo Excel in được |
| 10. IFC/3D | Đọc IFC STEP, entity counts, mesh runtime, orbit/wireframe, GlobalId | Mở IFC mẫu |
| 11. BIM cộng tác | Workspace, tầng/bộ môn, Issue/RFI/BCF, clash sơ bộ, ChangeSet, sync mô phỏng, viewport | Tạo issue thử |
| 12. AI ảnh → CAD | PNG/JPEG, hiệu chỉnh tỷ lệ, Preview, LINE chỉnh sửa được, Undo và epoch | PNG → LINE → DXF |
| 13. CLI/MCP | commands/info/convert/run/sample, MCP stdio, Headless vs Remote, Agent tool | CLI chuyển DXF/DWG |
| 14. Phục hồi | Tự lưu CAD+BIM, phục hồi, xử lý lỗi, kiểm tra CI, mã commit | Kiểm tra file backup |
| 15. Dự án tổng hợp | Mặt bằng phòng 6000×4000 mm, Layer, Block, kích thước, QTO, BIM, xuất báo cáo | Bộ hồ sơ thực hành |

### Hướng dẫn bắt đầu từ số không

**Bước 1 — Cài đặt.** Sau khi tải bộ phát hành **đã xác nhận Success**, chạy `CADTBT360.exe`; xem `BUILD_INFO.txt` để đối chiếu `SourceCommit`.
**Bước 2 — Tạo bản vẽ.** Chọn tab **CAD**, dùng **NEW**, xác định đơn vị bản vẽ và vẽ `LINE` từ `0,0` đến `5000,0`. Kiểm tra bằng `DIST`.
**Bước 3 — Lưu và mở lại.** Lưu thành `BT01.dxf`, đóng rồi mở lại. Không ghi đè bản vẽ người dùng.
4. Vào tab **ĐO BÓC** để thử đo chiều dài; chọn đúng phạm vi cửa sổ và quy đổi mm → m.
5. Thực hành tiếp từng chương theo bảng; chỉ xác nhận QTO sau khi kiểm tra nguồn và đơn vị.

### Các ví dụ định lượng không được làm sai

- Phòng 5×4 m có diện tích **20 m²**, cao 3 m → thể tích **60 m³** khi mặt bằng và mặt cắt có cùng hệ quy đổi.
- Đất đào giữa hai cọc cách 20 m, diện tích mặt cắt 8 m² và 12 m² → theo AEA: **200 m³**.
- Một nét ảnh dài 80 pixel với tỷ lệ 10 mm/pixel → chiều dài CAD **xấp xỉ 800 mm** (đo lại trên LINE).
- Nếu bản vẽ dùng mm, không được coi 1 CAD unit là 1 m khi xuất báo cáo.

### Phân biệt chức năng thật với phần đang hoàn thiện

- **Tab PDF / BẢN VẼ:** GUI ghi rõ đây là khu vực đang được tích hợp. Không coi là PDF editor đầy đủ.
- **IFC/3D:** đọc IFC và có thành phần hiển thị; mesh đầy đủ có thể cần **IfcConvert runtime** riêng.
- **BIM cộng tác:** có workspace offline, metadata và Issue; các tính năng đăng nhập/đồng bộ server có thể đang **mô phỏng**.
- **Clash BIM:** nguồn hiện mô tả lọc ứng viên bằng **bounding box 2D**, không được gọi là clash 3D chính thức.
- **AI từ ảnh:** hiện chủ yếu vector hóa nét pixel thành LINE, chưa tự hiểu tường, cửa, cột, kích thước thiết kế.
- **DWG:** phải kiểm chứng đọc/chỉnh sửa/lưu hai chiều bằng AutoCAD độc lập trước khi tuyên bố đạt tương thích.
- **Ảnh hướng dẫn:** chỉ dùng ảnh chụp thật từ GUI Windows đã được kiểm chứng; không thay bằng ảnh dựng rồi ghi như ảnh phần mềm.

### Mẫu kiểm tra một bài học

Ghi **tên bài → phiên bản/commit → file nguồn → bước thao tác → kết quả đo được → ảnh chụp thực tế → lỗi phát sinh → kết luận Đạt/Không đạt/Chưa kiểm chứng**. Bài chưa xác nhận trên EXE vẫn được phép dùng để học cấu trúc lệnh, nhưng không đánh dấu đã nghiệm thu.

### Bàn giao sau mỗi đợt

Lượt phát hành Windows đạt yêu cầu phải có **Full_Source.zip + Setup.exe + Portable.zip + hướng dẫn Word + SHA256SUMS + BUILD_INFO** cùng mã nguồn. File Word tổng hợp sẽ tiếp tục được cập nhật các bài học mới; không chỉ bổ sung riêng chương AI và không tạo nhiều bài lặp lại.

### Nguồn mã đã đối chiếu

- `CADTBT360_V1.000/apps/cadcraft/src/main.rs` — 5 tab và luồng giao diện.
- `CADTBT360_V1.000/crates/ui-egui/src/menus.rs` — menu File/Edit/View/Insert/Format/Tools/Draw/Dimension/Modify/3D/Help.
- `CADTBT360_V1.000/apps/cadcraft/src/takeoff.rs` — QTO live, chọn nguồn, confirm/review, nested.
- `CADTBT360_V1.000/apps/cadcraft/src/takeoff_composite.rs` — diện tích × chiều cao mặt cắt.
- `CADTBT360_V1.000/apps/cadcraft/src/takeoff_road.rs` — khối lượng đào/đắp theo AEA.
- `CADTBT360_V1.000/apps/cadcraft/src/takeoff_report.rs` — báo cáo Excel 12 cột và nhóm.
- `CADTBT360_V1.000/apps/cadcraft/src/model3d.rs` — xem IFC/3D.
- `CADTBT360_V1.000/apps/cadcraft/src/bim_collab.rs` — cộng tác BIM.
- `CADTBT360_V1.000/apps/cadcraft/src/ai_raster_panel.rs` — AI ảnh → LINE.
- `CADTBT360_V1.000/apps/cadcraft-cli/src/main.rs` và `crates/mcp/src/tools.rs` — CLI/MCP.


## 16. Sổ tay thao tác chi tiết theo tính năng (bổ sung đợt 179)

Các bài thực hành sau dùng **bản vẽ thử** và chỉ dẫn bám theo lệnh/giao diện trong mã nguồn V1.025. Giữ bản vẽ gốc độc lập, lưu file mới sau mỗi bài. Đường dẫn menu có thể thay đổi theo cấu hình hiển thị; nếu không thấy bảng lệnh, bật **Window → Command Line** hoặc vào **Help → Command Reference**. Mỗi bài đều có kết quả cần tự kiểm tra.

### Bài 01 — Khởi động, lưu bản vẽ và mở lại (File)

**Mục tiêu:** Tạo dự án đầu tiên mà không ghi đè tệp người dùng.

1. Cài từ **Setup EXE** hoặc giải nén **Portable ZIP** đã qua kiểm thử; chạy `CADTBT360.exe`.
2. Nhìn phiên bản `CADTBT360_V1.025` trên giao diện; đối chiếu `BUILD_INFO.txt` khi có.
3. Chọn **File → New** hoặc gọi `NEW`; giữ Model space. Chọn đơn vị mm cho bài này.
4. Từ Command Line, gọi `LINE`, nhập điểm **0,0** và **5000,0**, kết thúc bằng Enter.
5. Gọi `ZOOM` → `E` nếu chưa nhìn thấy đoạn vừa vẽ.
6. Chọn **File → Save As**, nhập `BT01_CAD_CoBan.dxf` vào thư mục bài tập mới.
7. Đóng/mở lại bằng **File → Open**; chọn nét và dùng lệnh `DIST` để kiểm tra chiều dài **5000 CAD units**.
8. Nếu cần DWG, lưu bản sao `BT01_CAD_CoBan.dwg`. Không coi việc phần mềm tự đọc lại là đủ bằng chứng tương thích AutoCAD.

**Cần đạt:** File DXF tồn tại, mở lại được, chứa LINE chỉnh sửa và kích thước không thay đổi. Nếu phần mềm thiếu nút trên thanh công cụ, dùng cùng lệnh qua Command Line.

### Bài 02 — Tùy chỉnh giao diện và truy cập công cụ (Window / View)

1. Vào **Window → Command Line** để bật lại cửa sổ nhập lệnh khi bị ẩn.
2. Vào **Window → Layers Palette** hoặc **Layer Properties Manager** để mở quản lý Layer.
3. Vào **Window → Properties Inspector** để kiểm tra thông số của LINE đang chọn.
4. Thử **Window → Reset Panel Layout** khi mất bảng; ghi nhận bố cục trở về mặc định.
5. Vào **View → Interface Theme → Light/Dark** nếu muốn thay đổi chế độ sáng/tối.
6. Thử chuyển qua 5 tab: **CAD**, **ĐO BÓC**, **PDF / BẢN VẼ**, **3D / IFC**, **BIM CỘNG TÁC**. Không nhập liệu trên tab PDF vì giao diện nguồn hiện thông báo tính năng đang tích hợp.

**Cần đạt:** Người mới biết chỗ bật Command Line, Layers, Properties và xác định các tab chính; không nhầm tab PDF đang phát triển thành chức năng hoàn chỉnh.

### Bài 03 — Vẽ mặt bằng phòng 6 × 4 m bằng LINE và OFFSET

1. Tạo file `BT03_Phong_6000x4000.dxf`; đặt bản vẽ sử dụng mm.
2. Lệnh `LINE`: lần lượt nhập **0,0 → 6000,0 → 6000,4000 → 0,4000 → 0,0**; kết thúc.
3. Gọi `DIST` trên hai đầu cạnh đáy: kỳ vọng **6000**; cạnh đứng: **4000**.
4. Dùng `OFFSET` nhập **200**, chọn các cạnh bao tường và chọn phía trong phòng. Kiểm tra các đoạn sau OFFSET nằm đúng phía cần thiết.
5. Dùng `TRIM` để xử lý phần thừa tại giao nhau nếu có; không tự cho rằng OFFSET đã tạo bốn góc kín.
6. Vào **Window → Properties Inspector**, chọn từng đoạn; ghi lại chiều dài và tọa độ.
7. Lưu lại rồi đóng/mở. Dùng `UNDO` hoặc `REDO` trên bản nháp để hiểu thao tác hoàn tác trước khi tạo bản chính thức.

**Cần đạt:** Khung ngoài 6000 × 4000 mm, tường offset 200 mm theo phía đã chọn, không có nét rác hoặc trùng đối tượng gây sai đo bóc.

### Bài 04 — Layer theo nhóm công việc và kiểm soát hiện hành

1. Mở **Window → Layers Palette**. Tạo các Layer ví dụ `A-TUONG`, `A-CUA`, `A-KICH_THUOC`, `S-COT`, `M-DIEN`.
2. Chỉnh màu/linetype từng Layer; đặt `A-TUONG` là **Current Layer**.
3. Vẽ một LINE làm tường và kiểm tra Properties thể hiện đúng Layer.
4. Đổi sang `A-CUA`, vẽ đường tượng trưng vị trí cửa; bảo đảm không nằm lẫn trên Layer tường.
5. Tạm **Off/Freeze** Layer `A-CUA` và kiểm tra hình biến mất; bật lại để chắc chắn đối tượng chưa bị xóa.
6. Dùng **Lock** để bảo vệ Layer tường trước khi quét chọn phần khác; bỏ khóa khi chỉnh sửa.
7. Lưu và mở lại để kiểm tra trạng thái Layer và hình học.

**Cần đạt:** Có tối thiểu ba lớp riêng biệt, thao tác ẩn/khóa không làm mất đối tượng. Tránh đưa mọi cấu kiện lên Layer 0.

### Bài 05 — Block và tái sử dụng hình học

1. Trên bản vẽ mẫu, tạo hình tượng trưng cửa bằng LINE/ARC.
2. Dùng lệnh `BLOCK`, đặt tên `CUA_D1`, chọn điểm gốc và các nét thuộc cửa.
3. Dùng `INSERT`, chèn block `CUA_D1` tại hai vị trí khác nhau.
4. Chọn một block, mở Properties để kiểm tra tên/tham chiếu; kiểm tra các đối tượng bên trong không bị nhân đôi sai chủ ý.
5. Thực hành với `EXPLODE` chỉ trên một bản sao block; đối chiếu tác dụng trước/sau và **UNDO** để khôi phục.
6. Chuyển sang **ĐO BÓC**, dùng lựa chọn **Thêm QTO nested từ Block/Array đang chọn** nếu đang cần bóc thành phần lồng nhau.

**Cần đạt:** Có ít nhất hai tham chiếu Block dùng chung định nghĩa. Kiểm tra khác biệt giữa số lượng Insert và số đối tượng con.

### Bài 06 — Xref và ảnh tham chiếu

1. Chuẩn bị hai tệp DXF trong cùng thư mục: `mat-bang.dxf` và `ket-cau.dxf`; tạo một bản vẽ tổng `tong-hop.dxf`.
2. Trong bản vẽ tổng, sử dụng lệnh **XREF** để gắn `ket-cau.dxf` làm tham chiếu; chọn đường dẫn tương đối khi phù hợp.
3. Thử ẩn/xuất hiện tham chiếu; đối chiếu rằng nguồn `ket-cau.dxf` không bị chỉnh khi sửa bản vẽ tổng.
4. Thử mở bản vẽ tổng sau khi đổi tên/thay vị trí file nguồn trên bản **sao** để nhận biết lỗi đường dẫn Xref.
5. Với ảnh tham chiếu, dùng **IMAGEATTACH** chọn PNG/JPG và **IMAGECLIP** để giới hạn vùng nhìn. IMAGEATTACH chỉ gắn ảnh, không chuyển thành LINE.
6. Dùng công cụ AI từ ảnh thành CAD nếu muốn tạo nét chỉnh sửa; **Preview trước** rồi mới tạo LINE và kiểm tra tỷ lệ.

**Cần đạt:** Không nhầm Xref với Block, không nhầm hình ảnh tham chiếu với đối tượng LINE.

### Bài 07 — Ghi chữ, kích thước và kiểm tra tỷ lệ

1. Mở bản vẽ phòng 6000 × 4000; đặt Layer `A-KICH_THUOC` làm hiện hành.
2. Dùng `STYLE` để xem/chọn kiểu chữ; dùng `TEXT` hoặc `MTEXT` để ghi “PHÒNG 01”.
3. Dùng `DIMLINEAR` trên hai đầu cạnh ngang để đo **6000**; dùng `DIMALIGNED` khi cần kích thước theo phương xiên.
4. Dùng `DIMSTYLE` để kiểm tra chiều cao chữ, kích thước mũi tên và đơn vị.
5. Mở Properties trên đối tượng Dimension để kiểm tra nội dung giá trị. Nếu ghi kích thước thủ công không khớp hình học, phải sửa hình hoặc Dim; không dùng chữ ghi đè để che sai số.
6. Thử **PLOT** và xuất PDF ở bản vẽ thử; mở tệp PDF bằng trình đọc ngoài để xem đủ nét và không mất font.

**Cần đạt:** Kích thước hiển thị khớp số đo thực, font đọc được và PDF không bị mất nét.

### Bài 08 — Đo bóc chiều dài và diện tích có nguồn

1. Vào tab **ĐO BÓC**, giữ bản vẽ phòng đang mở ở cột CAD bên trái.
2. Trong giao diện QTO, xem phần **Phạm vi bóc**; quét một vùng cửa sổ trên canvas. Trái → phải là chọn đối tượng nằm trong; phải → trái là chọn đối tượng bị quét cắt qua.
3. Chọn một LINE hợp lệ rồi dùng **Thêm phép đo từ CAD đang chọn**; chọn phép đo **Length**.
4. Đặt **Nhóm công việc**, **Mã công tác**, **Diễn giải** và ghi chú rõ đối tượng nguồn.
5. Đối chiếu đơn vị bản vẽ: nếu LINE dài 6000 mm thì khối lượng dài theo m là **6,000 m**, không phải 6000 m.
6. Dùng trạng thái **Review → Đã xác nhận** sau khi kiểm tra nguồn; chỉ record xác nhận mới được tính vào tổng hợp theo cấu hình báo cáo.
7. Thử xóa/thay hình nguồn trên bản sao để kiểm tra cảnh báo nguồn đo bóc cũ hoặc cần rà soát.

**Cần đạt:** Bảng có mã công tác, trạng thái, đơn vị, khối lượng và nguồn CAD truy xuất được; không chỉ đếm số Entity.

### Bài 09 — Bóc thể tích qua hai vùng mặt bằng và mặt cắt

1. Vào tab **ĐO BÓC**; chuẩn bị mặt bằng có **một đường bao kín** diện tích 20 m² và mặt cắt có **một LINE thẳng đứng** cao 3 m.
2. Nhấn **1. LẤY DIỆN TÍCH MẶT BẰNG**, chọn vùng/đối tượng CAD làm diện tích nguồn.
3. Nhấn **2. LẤY CHIỀU CAO MẶT CẮT**, chọn LINE đứng biểu diễn chiều cao; phần mềm có kiểm tra hướng đứng của LINE, không tự suy từ kích thước chữ.
4. Nhập **Nhóm**, **Mã CT**, **Diễn giải**; xác nhận cả hai nguồn hình học.
5. Đối chiếu công thức **V = A × H = 20 × 3 = 60 m³** sau khi quy đổi đồng nhất từ CAD unit sang đơn vị m.
6. Thử thay đổi chiều cao trên bản thử; kiểm tra rằng bản ghi đã xác nhận phải được rà soát lại nếu nguồn hình học thay đổi.

**Cần đạt:** Bản ghi có **hai nguồn hình học riêng** (mặt bằng + mặt cắt). Không sử dụng phép đếm đối tượng để suy ra m³.

### Bài 10 — Đo bóc đất đào/đắp theo lý trình đường

1. Vào tab **ĐO BÓC**, tìm chức năng bóc giao thông theo **Lý trình**, **Tên cọc**, **Mã đào**, **Mã đắp**.
2. Tạo cọc A ở lý trình **0 m**, diện tích đào **8 m²**, cọc B ở **20 m**, diện tích đào **12 m²**. Chọn đúng các đối tượng vùng mặt cắt đã đo.
3. Chọn **Khoảng cọc cần bóc** từ A đến B, đảm bảo hai lý trình tăng dần và không bị trùng.
4. Kiểm tra thể tích đào theo phương pháp Average End Area: **V = (8+12) × 20 / 2 = 200 m³**.
5. Nếu có diện tích đắp, nhập bộ nguồn khác; không gộp đào và đắp thành một giá trị dương.
6. Xuất bảng theo cọc và kiểm tra số liệu từng khoảng trước khi cộng tổng.

**Cần đạt:** Khối lượng phụ thuộc dữ liệu diện tích mặt cắt và khoảng cách cọc thực, không dựa trên giả định từ bình đồ đơn thuần.

### Bài 11 — Tạo bảng QTO Excel được in trực tiếp

1. Sau khi có ít nhất ba bản ghi đo bóc, chọn chức năng xuất **Excel/XLSX** từ tab **ĐO BÓC**.
2. Nhập **Tên dự án**, **Chủ đầu tư**, **Số hiệu biểu mẫu**; chọn nhóm theo **Nhóm công việc**, **Mã công tác**, **Layer**, **Tầng** hoặc **Bộ môn**.
3. Chọn các cột muốn xuất trong 12 trường hiện có, ví dụ STT, Mã công tác, Nội dung đo bóc, Nhóm, Layer, Tầng, Loại phép đo, Khối lượng, Đơn vị.
4. Cấu hình **Đơn vị chiều dài** và **Số chữ số lẻ** (ví dụ 3); nếu chỉ xuất bản ghi đã xác nhận, bật lọc tương ứng.
5. Xuất XLSX và mở bằng Excel. Kiểm tra tiêu đề, nhóm, tổng cộng, số thập phân, độ rộng cột, ngắt trang và handle nguồn.
6. So sánh số liệu XLSX với bảng QTO trong CAD. Không sửa tay một con số Excel để che chênh lệch chưa giải quyết.

**Cần đạt:** Mở và in XLSX được, cột dữ liệu dễ đọc và tổng hợp đúng trạng thái bản ghi.

### Bài 12 — Xem và phân loại IFC/3D

1. Chuyển tab **3D / IFC**, chọn tệp `.ifc` chuẩn STEP (`ISO-10303-21`).
2. Kiểm tra bảng **Thống kê IFC Entities**; đối chiếu số lượng cấu kiện và tên file.
3. Khi có mesh hiển thị, kéo chuột xoay và cuộn để phóng to; có thể **Đặt lại góc nhìn**.
4. Sử dụng ô lọc **IFC GlobalId**, **Tầng**, **Bộ môn**; ghi lại số phần tử còn lại sau lọc.
5. Thử **Ẩn nhóm đã chọn**, **Cô lập nhóm đã chọn**, **Hiện tất cả** trên dữ liệu mẫu.
6. Nếu IFC chỉ hiển thị danh mục/điểm mà chưa có lưới 3D, kiểm tra yêu cầu **IfcConvert** hoặc tệp OBJ hỗ trợ. Không ghi lỗi “mất cấu kiện” khi chưa có runtime chuyển mesh.

**Cần đạt:** Xem đúng metadata và phạm vi mô hình; ghi rõ khác biệt giữa IFC xem sơ bộ và mesh 3D đầy đủ.

### Bài 13 — BIM cộng tác theo tầng, bộ môn và Issue

1. Chuyển tab **BIM CỘNG TÁC**, nhập **Mã dự án** và **Tên dự án**, chọn **Dùng profile offline** để thực hành.
2. Dùng **+ Thêm tầng** tạo TẦNG 1, TẦNG 2. Gán bộ môn Kiến trúc/Kết cấu cho các đối tượng CAD được chọn.
3. Vào **Project Navigator BIM** để chuyển tầng; kiểm tra bộ lọc không che mất nhầm dữ liệu.
4. Trong mục **Issue phối hợp / Coordination**, tạo Issue thử cho bộ môn Kết cấu; nhập mô tả và người nhận.
5. Xem **LỊCH SỬ THAY ĐỔI ISSUE**; chỉ đóng Issue sau khi có kết quả kiểm tra và xác nhận.
6. Thử **Quét tầng hiện tại** để xem giao cắt sơ bộ: kết quả dựa vào bounding box 2D, **không phải clash 3D chính thức**.
7. Thử **Lưu workspace BIM** rồi mở lại. Nút đồng bộ server hiện có chế độ **mô phỏng**; không coi đó là bằng chứng upload cloud thành công.

**Cần đạt:** Workspace có tầng/bộ môn/Issue và dữ liệu lưu được. Phải phân biệt phối hợp offline, mô phỏng và dịch vụ online thực tế.

### Bài 14 — AI từ ảnh thành LINE và chống ghi nhầm bản vẽ

1. Vào tab **CAD**, nhấn **AI TỪ ẢNH → CAD**.
2. Nhấn **Chọn ảnh PNG/JPG…**, chọn ảnh nét đậm có tỷ lệ đã biết.
3. Thiết lập **Nét tối thiểu** (ví dụ 16 px) và **Đơn vị CAD/pixel** (ví dụ 10 mm/pixel); có thể bật nét 45°.
4. Nhấn **1. Nhận dạng và xem trước**; kiểm tra số nét, hướng và tỷ lệ.
5. Nếu chuyển sang bản vẽ khác rồi quay lại, **Preview cũ phải bị từ chối** theo document UID + revision + epoch. Nhấn Preview lại trước khi ghi.
6. Nhấn **2. Tạo LINE chỉnh sửa trong CAD**, chọn LINE trên canvas và kiểm tra điểm đầu/cuối.
7. Thử UNDO một lần, kiểm tra toàn nhóm AI bị hoàn tác. Làm lại và lưu bản DXF/DWG mới để mở lại.
8. **Không coi chức năng này là AI đọc được tường/cửa/cột** hoặc kích thước ghi chữ; đó vẫn là năng lực chưa hoàn thiện.

**Cần đạt:** Đối tượng LINE thật có thể chỉnh sửa; không ảnh hưởng bản vẽ ngoài phiên Preview.

### Bài 15 — CLI, MCP và quy trình bàn giao phiên bản

1. Mở PowerShell tại thư mục cài đặt, chạy `.\CADTBT360_CLI.exe commands` để xem lệnh.
2. Thử `.\CADTBT360_CLI.exe sample floorplan BT15_floorplan.dxf`, rồi `.\CADTBT360_CLI.exe info BT15_floorplan.dxf`.
3. Chuyển sang DWG: `.\CADTBT360_CLI.exe convert BT15_floorplan.dxf BT15_floorplan.dwg`. Dùng `info` mở lại và kiểm tra số đối tượng.
4. Với MCP, cấu hình client stdio dùng `CADTBT360_CLI.exe` và đối số `mcp`. **Không có --connect** nghĩa là Headless riêng; muốn điều khiển GUI đang mở phải cấu hình kênh Remote hợp lệ.
5. Thử công cụ `raster_to_cad_lines` với `preview:true`, kiểm tra rồi mới `preview:false`.
6. Vào GitHub Actions để kiểm tra workflow đã **Success** rồi tải **Setup + Portable + Full_Source.zip + Word + SHA256SUMS**. Tệp mã nguồn có thể yêu cầu đăng nhập GitHub.
7. Đối chiếu SHA256 và `SourceCommit` với lượt phát hành. Nếu chưa có workflow PASS hoặc EXE chưa được kiểm thử GUI, ghi **Chưa nghiệm thu**, không ghi Đạt.

**Cần đạt:** Học viên nắm quy trình CLI/MCP và biết chứng minh nguồn gốc một bộ cài, không nhầm commit thành bản EXE đã nghiệm thu.

## 17. Phiếu nghiệm thu thực hành toàn phần mềm

Ghi lại kết quả theo từng chương, không chỉ tick “đã xem”:

| Nhóm | Bằng chứng cần có | Đạt khi |
|---|---|---|
| CAD 2D | DXF có LINE, POLYLINE, Layer, Block | Lưu/mở lại, đo đúng |
| Trình bày | Bản in PDF và DIM đo được | Đúng font, kích thước |
| QTO cơ bản | Bảng có handle và khối lượng | Quy đổi đơn vị đúng, có trạng thái |
| QTO mặt cắt | Hai nguồn A và H | V=60 m³ theo mẫu |
| QTO giao thông | Hai diện tích + khoảng cọc | V=200 m³ theo AEA |
| Excel | XLSX có các cột chọn và tổng | Mở/in được, dữ liệu khớp |
| IFC/3D | Metadata IFC và ảnh chụp thật | Bộ lọc tầng/bộ môn đúng |
| BIM | Workspace và Issue mẫu | Lưu/mở lại được |
| AI | PNG→LINE→Undo→DXF | Không ghi nhầm tài liệu |
| MCP | JSON-RPC và CLI output | Phân biệt Headless/Remote |
| Bàn giao | Build, source ZIP, Word, SHA256 | Chỉ ghi Đạt khi CI PASS |

**Chú ý:** Hình chụp giao diện phải lấy từ chương trình Windows thực tế sau khi build và chạy thành công. Không sử dụng hình giả lập thay cho bằng chứng nghiệm thu.


## 18. Mở giáo trình tổng hợp trực tiếp trong CAD 360 — Đợt 179

Sau khi cài **Setup** hoặc giải nén **Portable** từ một Windows Release đã thành công, mở CAD 360 và nhấn nút **HƯỚNG DẪN CAD 360** ở thanh công cụ chính (nút này khác **HƯỚNG DẪN AI** trong tab CAD).

- Với **Setup**, phần mềm mở `HuongDan\GIAO_TRINH_TONG_HOP_CAD360.docx` nằm trong thư mục cài đặt.
- Với **Portable**, phần mềm mở `GIAO_TRINH_TONG_HOP_CAD360.docx` nằm cạnh `CADTBT360.exe`.
- Nếu thiếu file Word, phần mềm hiện thông báo đường dẫn cần kiểm tra, không âm thầm dùng giáo trình AI thay thế.
- Người dùng cần có chương trình mở được định dạng Word DOCX trên máy.
- Tính năng gọi tài liệu sử dụng trình xử lý tệp mặc định của Windows; chưa có kết quả xác nhận GUI Windows thực tế sau khi bổ sung.

CI dùng bài kiểm thử `complete_user_manual_is_found_in_setup_and_portable_paths` để đối chiếu hai đường dẫn tài liệu, ngoài bước kiểm tra file trong bộ cài và Portable ZIP.

**Quy tắc từ đợt 179:** Giáo trình tổng hợp là tài liệu chính bao gồm toàn bộ chương CAD 2D/QTO/IFC/BIM/AI/CLI; hướng dẫn AI chỉ là chuyên đề. Mã nguồn ZIP phải giữ cả hai tài liệu, bản phát hành phải có Word tổng hợp và kiểm tra SHA256.


## 18. Bài tập nâng cao chuyên đề theo tính năng — Đợt 180

Các bài dưới đây tiếp tục từ bài 01–15. Mỗi bài có tình huống riêng và tiêu chí đầu ra; không lặp lại quy trình AI.

### Bài 16 — Bắt điểm chính xác, ORTHO và tọa độ tương đối

1. Mở bản vẽ thực hành mới; bật **OSNAP** và kiểm tra các kiểu bắt điểm Endpoint, Midpoint, Center trong giao diện công cụ.
2. Dùng `LINE` từ `0,0` đến `1000,0`, kết thúc lệnh. Bắt điểm **Endpoint** bên phải để vẽ đoạn nối tiếp dài 500 đơn vị.
3. Bật **ORTHO** để ràng buộc đoạn theo phương ngang/đứng; kiểm tra hướng đường khi di chuột.
4. Thử nhập tọa độ tương đối `@0,500` từ điểm cuối hiện hành, sau đó `DIST` kiểm tra đoạn đứng bằng 500 đơn vị.
5. Dùng **OSNAP Midpoint** để xác định trung điểm của đoạn đầu tiên; đọc tọa độ để kiểm tra giá trị x = 500.
6. Tắt/bật ORTHO và OSNAP để nhận biết khác biệt; lưu thành `BT16_BatDiem.dxf`.

**Nghiệm thu:** Có thể đo độ dài các đoạn đúng giá trị hình học, không dùng thao tác vẽ gần đúng bằng chuột thay cho bắt điểm.

### Bài 17 — POLYLINE, HATCH và kiểm tra đường bao

1. Tạo đường bao phòng bằng `PLINE`, nhập `0,0`, `5000,0`, `5000,4000`, `0,4000`, rồi dùng tùy chọn **Close** để khép kín.
2. Chọn đường bao, kiểm tra là **Polyline** chứ không phải nhiều LINE tách rời.
3. Sử dụng `HATCH` trên hình kín; quan sát vùng tô nằm trong đường bao và không tràn sang hình khác.
4. Trên bản sao, tạo một khe hở nhỏ trong đường bao và thử HATCH; ghi lại cảnh báo hoặc khác biệt.
5. Dùng `AREA` để đo: kỳ vọng **20.000.000 mm² = 20 m²**; không đọc nhầm mm² thành m².
6. Lưu `BT17_Polyline_Hatch.dxf` và mở lại kiểm tra các thực thể vẫn tồn tại.

**Nghiệm thu:** Polyline khép kín, Hatch chỉ phủ vùng mong muốn, diện tích quy đổi đúng.

### Bài 18 — BLOCK có thuộc tính và quản lý tham chiếu

1. Tạo một ký hiệu thiết bị điện gồm hình tròn, nét chỉ dẫn và tên ký hiệu.
2. Khai báo thuộc tính bằng `ATTDEF` (nếu lệnh hiển thị trong Command Reference của bản đang kiểm thử), ví dụ `TAG=MA_TB`.
3. Dùng `BLOCK` tạo block `DEN_T1`, chọn điểm chèn và các phần tử.
4. Chèn hai lần bằng `INSERT`, đặt mã thử `D01` và `D02` nếu công cụ thuộc tính hỗ trợ.
5. Chọn từng block trong **Properties Inspector**, kiểm tra dữ liệu tên và thuộc tính có thể phân biệt.
6. Thử **COPY** một tham chiếu, **UNDO**, sau đó so sánh số Insert trong bản vẽ và kết quả đếm trong tab ĐO BÓC.

**Nghiệm thu:** Không đánh đồng cùng một Block Definition với nhiều vị trí Insert. Nếu bản EXE không có ATTDEF, ghi **Chưa kiểm chứng** thay vì khẳng định tính năng có.

### Bài 19 — Layout, Viewport và bản in A3

1. Mở mặt bằng đã ghi kích thước; kiểm tra Model space với đơn vị đúng.
2. Vào **Layout** và tạo/điều chỉnh một viewport nếu chức năng có trong Command Reference.
3. Đặt tỷ lệ nhìn phù hợp, ví dụ **1:100** cho bản vẽ mm; tránh tự phóng hình gốc để “đúng giấy”.
4. Tạo khung bản vẽ và vùng ghi chú ở Paper space; kiểm tra dòng kích thước không chồng lên khung.
5. Thử `PLOT` ra PDF khổ A3 ngang; mở bản PDF và đọc kiểm tra tên bản vẽ, tỷ lệ, chữ và kích thước.
6. Nếu không xuất được PDF, lưu log/ảnh từ GUI thật và đánh dấu **Chưa đạt**, không thay bằng ảnh PDF mô phỏng.

**Nghiệm thu:** Bản PDF có thể in, không vỡ bố cục; tỷ lệ được kiểm tra theo một đoạn chuẩn đã biết.

### Bài 20 — Kiểm soát dữ liệu đo bóc nguồn đã thay đổi

1. Mở dự án đo bóc có ít nhất 3 bản ghi nguồn hình học; đảm bảo có nhóm công việc và mã công tác.
2. Trong tab **ĐO BÓC**, kiểm tra trạng thái `Review`, `Đã xác nhận`, `Loại` và lọc theo từng trạng thái.
3. Sửa một LINE nguồn trên bản vẽ thử, chuyển lại bảng để kiểm tra cập nhật trạng thái hoặc cảnh báo nguồn đã thay đổi.
4. Nếu nguồn bị mất, dùng **Tự relink nguồn bị mất** chỉ khi kết quả tìm được là duy nhất; không nối tự động vào đối tượng khác vì giống tên Layer.
5. Sử dụng **Xác nhận tất cả phép đo nguồn hợp lệ**, sau đó xem lại bản ghi không hợp lệ vẫn cần rà soát.
6. Chọn cột và nhóm xuất XLSX; kiểm tra **chỉ bản ghi xác nhận** được tính vào tổng nếu bật bộ lọc tương ứng.

**Nghiệm thu:** Các phép đo đều có nguồn truy nguyên, không tự xuất số liệu chưa xác nhận như đã nghiệm thu khối lượng.

### Bài 21 — Phục hồi CAD+BIM và đối chiếu dữ liệu sau sự cố

1. Trên bản vẽ thử, tạo một Layer, vẽ 2 LINE và mở workspace BIM có Issue mẫu.
2. Trong giao diện CAD, tìm tùy chọn **Tự lưu CAD+BIM: 5 phút** và xác định trạng thái tự lưu.
3. Chọn **Lưu CAD+BIM** để tạo bản phục hồi cặp dữ liệu; không tắt ứng dụng đột ngột trong khi chưa xác nhận file đã được ghi.
4. Trên bản sao thực hành, thay đổi hình học/Issue rồi thử **Phục hồi CAD+BIM**; đọc thông báo trước khi xác nhận.
5. Đối chiếu lại số LINE, Layer, tên tầng/bộ môn, Issue và đường dẫn bản vẽ.
6. Kiểm tra không mất bản gốc, không nhầm file phục hồi với file DWG chuẩn trao đổi bên ngoài.

**Nghiệm thu:** Phục hồi được hai phần CAD và BIM về cùng thời điểm dữ liệu đã lưu; các tình huống thực tế vẫn cần chạy thử trên EXE Windows.

## 19. Chuẩn bị hình ảnh thật cho phiên bản giáo trình phát hành

- **Ảnh 01:** màn hình CAD 360 V1.025 sau khởi động, đủ 5 tab và build SHA.
- **Ảnh 02:** thanh Command Line, bảng Layer, Properties và vùng vẽ.
- **Ảnh 03:** mặt bằng hình học trước/sau OFFSET và HATCH.
- **Ảnh 04:** ví dụ Dimension và màn hình thiết lập in.
- **Ảnh 05:** tab ĐO BÓC với lựa chọn theo vùng và mã công tác.
- **Ảnh 06:** giao diện bóc khối lượng mặt bằng × mặt cắt.
- **Ảnh 07:** màn hình cọc đường đào/đắp và báo cáo Excel đã mở.
- **Ảnh 08:** IFC/3D đang hiển thị danh mục/mesh thật.
- **Ảnh 09:** BIM cộng tác có Issue, tầng và bộ môn.
- **Ảnh 10:** AI Preview và LINE CAD đã tạo.
- **Ảnh 11:** màn hình CLI/MCP, kiểm tra đầu ra DXF/DWG.
- **Ảnh 12:** cửa sổ Phục hồi CAD+BIM và trạng thái file nguồn.

Chỉ chèn ảnh khi được chụp từ **đúng EXE Windows tại commit tương ứng**. Đến khi có hình xác thực, để mục “Chưa có ảnh kiểm chứng”, không sử dụng ảnh từ AutoCAD/phần mềm bên ngoài thay cho CAD 360.


## 20. Đợt 181 — Xử lý sự cố toàn phần mềm cho người mới

Các tình huống dưới đây áp dụng theo đúng thứ tự: **kiểm tra bản vẽ thử → đối chiếu nguồn/tỷ lệ → kiểm tra trạng thái lệnh → lưu bản sao → đánh giá lại**. Không sửa trực tiếp dữ liệu sản xuất khi đang tìm nguyên nhân.

### Sự cố 01 — Không thấy thanh lệnh hoặc không gõ được lệnh CAD

#### Dấu hiệu và bước xử lý
1. Chọn tab **CAD**, không nhập lệnh vào các trường của tab ĐO BÓC hay BIM.
2. Mở **Window → Command Line** để bật bảng nhập lệnh bị ẩn.
3. Kiểm tra có lệnh đang chờ thao tác. Dùng **Esc** để thoát lệnh hiện tại trước khi nhập lệnh mới.
4. Nhập `LINE` và thực hành hai điểm `0,0`, `1000,0`; sau đó kiểm tra Properties.
5. Nếu không thấy bảng công cụ, dùng **Window → Reset Panel Layout**; lưu lại ảnh màn hình thật và tên commit.

### Sự cố 02 — Đo bóc chiều dài, diện tích hoặc thể tích sai số

#### Dấu hiệu và bước xử lý
1. Kiểm tra đơn vị bản vẽ trước khi bóc: 6000 CAD unit theo hệ **mm** chỉ bằng **6 m**.
2. Đối chiếu `DIST`, `AREA` hoặc Properties trước khi tạo bản ghi QTO.
3. Với bóc khối lượng mặt bằng × mặt cắt, kiểm tra **cả hai nguồn hình học** thuộc cùng hệ đơn vị; chiều cao phải là LINE gần thẳng đứng.
4. Với đào đắp đường, kiểm tra **lý trình tăng dần**, mặt cắt A₁/A₂ và nhãn đào/đắp riêng biệt.
5. Chưa xác nhận bản ghi nếu handle nguồn đã thay đổi; chỉ bật bộ lọc “Đã xác nhận” khi mọi đối tượng đã được rà soát.
6. Xuất XLSX, so sánh ít nhất ba số đo trực tiếp với bản vẽ nguồn.

### Sự cố 03 — Xref mất tham chiếu hoặc ảnh PNG chỉ hiện như ảnh

#### Dấu hiệu và bước xử lý
1. Mở tệp nguồn DXF/DWG độc lập, xác nhận tồn tại và không bị đổi tên.
2. Kiểm tra đường dẫn Xref và thư mục chứa tệp tổng hợp; ưu tiên thử đường dẫn tương đối với bản sao cùng thư mục.
3. **IMAGEATTACH** chỉ đính kèm raster: không thể dùng grips của LINE để sửa nét trong ảnh.
4. Nếu muốn chuyển ảnh thành nét CAD, mở **AI TỪ ẢNH → CAD**, khai báo tỷ lệ, kiểm tra Preview rồi mới tạo LINE.
5. Nếu chuyển bản vẽ sau Preview, phải chạy Preview mới để tránh áp dụng nét vào tài liệu không đúng.

### Sự cố 04 — IFC chỉ hiện danh mục/điểm hoặc BIM không lên Cloud

#### Dấu hiệu và bước xử lý
1. Vào tab **3D / IFC**, kiểm tra file có header **ISO-10303-21** và không vượt giới hạn đọc.
2. Nếu thiếu mesh, xác nhận có công cụ chuyển **IfcConvert** hoặc OBJ hợp lệ; metadata/điểm không thay thế mesh tam giác.
3. Vào tab **BIM CỘNG TÁC**, xác nhận workspace đã được lưu; dùng profile **offline** khi tập huấn chưa kết nối server.
4. Các nút “Mô phỏng đăng nhập” và “Đồng bộ với server (mô phỏng)” không phải bằng chứng dữ liệu đã lên Cloud.
5. Quét clash trong workspace có thể chỉ dựa trên **bounding box 2D**; không báo là clash BIM 3D chính thức nếu chưa có thuật toán/cấu hình tương ứng.

### Sự cố 05 — Không có giáo trình Word hoặc không biết bản đang chạy

#### Dấu hiệu và bước xử lý
1. Tại tab CAD, nhấn **HƯỚNG DẪN CAD 360** để mở giáo trình tổng hợp, không nhầm với **HƯỚNG DẪN AI**.
2. Với Setup, kiểm tra `HuongDan/GIAO_TRINH_TONG_HOP_CAD360.docx` trong thư mục cài.
3. Với Portable, kiểm tra `GIAO_TRINH_TONG_HOP_CAD360.docx` nằm cạnh `CADTBT360.exe`.
4. Kiểm tra `BUILD_INFO.txt`, mã commit, file SHA256SUMS và trạng thái Windows CI cho đúng phiên bản.
5. Nếu file Word không mở, kiểm tra ứng dụng đọc DOCX trên Windows; không tự kết luận CAD mất chức năng vì máy thiếu ứng dụng văn phòng.
6. Nếu chưa có EXE thực tế hoặc chưa có ảnh chụp GUI thật, kết quả nghiệm thu phải ghi **Chưa kiểm chứng**.

## 21. Phiếu kiểm thử bàn giao theo nhóm tính năng — Đợt 181

| Nhóm nghiệm thu | Mẫu đầu vào | Đầu ra bắt buộc | Trạng thái mặc định |
|---|---|---|---|
| CAD 2D cơ bản | Phòng 6000 × 4000 mm | DXF/DWG đọc lại được, LINE và Layer đúng | Chưa kiểm chứng EXE |
| Chữ, kích thước, in | Bản vẽ A3 có Dimension | PDF chữ và đường nét đọc được | Chưa kiểm chứng EXE |
| Block, Xref | 2 DXF cùng thư mục | Xref không gãy, số Block hợp lệ | Chưa kiểm chứng EXE |
| Đo bóc cơ bản | LINE 6000 mm | 6 m có handle và trạng thái | Chưa kiểm chứng EXE |
| Mặt bằng + mặt cắt | Diện tích 20 m², cao 3 m | 60 m³ và 2 nguồn CAD | Chưa kiểm chứng EXE |
| Giao thông | A1=8, A2=12 m², L=20 m | 200 m³ đào theo AEA | Chưa kiểm chứng EXE |
| Excel QTO | Các bản ghi đã xác nhận | XLSX nhóm/cột/tổng đúng | Chưa kiểm chứng EXE |
| IFC | IFC STEP mẫu | Danh mục GlobalId, tầng, trạng thái mesh rõ | Chưa kiểm chứng EXE |
| BIM | Workspace với tầng và issue | Lưu/mở lại Issue và metadata | Chưa kiểm chứng EXE |
| AI | PNG nét ngang và tỷ lệ chuẩn | LINE chỉnh sửa + Undo đúng nhóm | Chưa kiểm chứng EXE |
| CLI/MCP | DXF và JSON-RPC | Info, convert, tools/list đúng | Chưa kiểm chứng EXE |
| Hướng dẫn | Setup và Portable | DOCX đầy đủ 21 bài, bảng thực, mở từ GUI | Chưa kiểm chứng EXE |

Sau khi Windows CI thực tế PASS, ghi **run ID, commit SHA, Setup/Portable SHA256, ảnh chụp GUI thật, người kiểm thử và kết quả** trong biên bản. Không ghi “Đạt” chỉ vì script có trong GitHub.


## 22. Đợt 182 — Quy trình nghiệm thu 21 bài học trên EXE thật

Giáo trình **không chỉ hướng dẫn đọc**, mà cần phục vụ kiểm tra thực tế từng nhóm tính năng. Người dùng thực hành trong một thư mục riêng, không dùng bản vẽ sản xuất. Sau mỗi bài, ghi **tên file – kết quả đo – phiên bản EXE – commit SHA – ảnh chụp màn hình thật – Đạt/Không đạt/Chưa kiểm chứng**.

### Phiếu theo dõi từ bài 01 đến bài 21

| Bài | Nhóm chức năng | Dữ liệu/kết quả cần lưu | Cần kiểm tra trên EXE |
|---|---|---|---|
| 01 | Cài đặt, New/Open/Save | BT01_CAD_CoBan.dxf, LINE 5000 | Có |
| 02 | Giao diện, Layers/Properties | Ảnh giao diện 5 tab, Command Line | Có |
| 03 | Vẽ, Offset, Trim | Phòng 6000×4000, tường 200 mm | Có |
| 04 | Layer và chế độ khóa/ẩn | A-TUONG, A-CUA, A-KICH_THUOC | Có |
| 05 | Block, Insert, Explode | CUA_D1 và hai INSERT | Có |
| 06 | Xref và IMAGEATTACH | Tổng hợp DXF và ảnh tham chiếu | Có |
| 07 | Text, Dimension, Plot | DIM 6000 và bản in PDF | Có |
| 08 | QTO cơ bản, phân vùng chọn | LINE 6000 mm → 6 m | Có |
| 09 | QTO mặt bằng × mặt cắt | 20 m² × 3 m = 60 m³ | Có |
| 10 | Đào/đắp giao thông | AEA 8/12 m², L=20 m → 200 m³ | Có |
| 11 | Báo cáo đo bóc XLSX | Báo cáo có mã, nguồn, nhóm và tổng | Có |
| 12 | IFC/3D và bộ lọc | Danh sách GlobalId/tầng/bộ môn | Có |
| 13 | BIM cộng tác | Workspace có tầng, bộ môn và Issue | Có |
| 14 | AI từ ảnh thành CAD | PNG → LINE, Preview, Undo, DXF | Có |
| 15 | CLI/MCP | tools/list, info, convert DXF/DWG | Có |
| 16 | OSNAP/ORTHO | LINE chính xác 1000 và 500 đơn vị | Có |
| 17 | Polyline và Hatch | Vùng kín 20 m² và Hatch | Có |
| 18 | Attribute của Block | Hai mã D01/D02 nếu ATTDEF có trong EXE | Có |
| 19 | Layout và Viewport | PDF A3, tỷ lệ phù hợp | Có |
| 20 | Truy nguyên nguồn QTO | Trạng thái Review/Confirm và XLSX | Có |
| 21 | Sao lưu/phục hồi CAD+BIM | Cặp file CAD và workspace BIM khớp | Có |

### Bài 22 — Cách tạo bộ bằng chứng nghiệm thu thực tế (quy trình QA)

1. Cài đúng **CADTBT360_V1.025_Setup_Windows_x64.exe** hoặc giải nén **Portable** thuộc cùng **GitHub Actions Success run**; không dùng file EXE cũ có tên giống nhau.
2. Kiểm tra `BUILD_INFO.txt` và lưu `SourceCommit` vào bảng kiểm tra.
3. Mở giao diện CAD 360, chụp màn hình toàn cửa sổ bằng **Win + Shift + S** hoặc Snipping Tool. Ảnh phải hiển thị giao diện thật và không cắt mất phần thông báo lỗi.
4. Trong mỗi bài, chỉ lấy ảnh ở **trạng thái cuối thao tác** hoặc trạng thái lỗi cần tái hiện; không thay bằng ảnh AutoCAD hoặc hình thiết kế giả lập.
5. Tạo thư mục `CAD360_V1025_NghiemThu/`, có `01_CAD/`, `02_QTO/`, `03_IFC_BIM/`, `04_AI_MCP/`, `05_Screenshots/` và `06_BaoCao/`.
6. Với file DXF/DWG, phải mở và chỉnh sửa lại bằng chính CAD 360, sau đó **kiểm tra độc lập bằng AutoCAD** trước khi ký đạt tiêu chí tương thích DWG hai chiều.
7. Với Excel QTO, mở tệp bằng Excel và đối chiếu tổng với nguồn, đơn vị và trạng thái nghiệm thu; không chỉ xem số lượng dòng.
8. Ghi rõ tính năng **PDF đang tích hợp**, **BIM đồng bộ mô phỏng**, **Clash 2D sơ bộ**, **AI vector hóa pixel**, và **IFC mesh phụ thuộc runtime** — các giới hạn này không được đánh dấu hoàn thiện sai thực tế.
9. Chỉ kết luận bản phát hành **Đạt** khi có Windows CI Success, bộ cài/Portable tải được, mã nguồn ZIP cùng commit, tài liệu Word mở được và bảng kiểm thử có bằng chứng.

### Tăng cường chất lượng đóng gói đợt 182

Trong CI, ảnh PNG test được kiểm tra trước; tiếp đó hệ thống **xác minh đủ 21 bài trong Markdown**, **tạo cả Word tổng hợp lẫn Word chuyên đề**, rồi **kiểm tra Word tổng hợp có bảng thật, tiêu đề đúng cấu trúc và đủ 21 bài**. Nếu sai, dừng trước khi biên dịch Rust và trước bước tạo Setup/Portable.

**Lưu ý về phiên bản:** V1.025 vẫn là định danh nhánh phát triển hiện tại; các đợt 179–182 là các lần cập nhật mã nguồn/tài liệu, chưa phải bằng chứng phiên bản EXE phát hành mới. Chưa có kết quả Windows CI PASS được xác minh cho commit này và chưa có bộ ảnh chụp thực tế từ EXE để đưa vào giáo trình.
