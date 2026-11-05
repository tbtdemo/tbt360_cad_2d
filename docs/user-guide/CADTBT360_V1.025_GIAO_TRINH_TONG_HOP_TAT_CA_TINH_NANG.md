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


## 23. Đợt 183 — Kiểm chứng tỷ lệ và tọa độ AI trên EXE trong bộ cài

### Vì sao phải kiểm tra tọa độ thay vì chỉ đếm số LINE?

Việc ảnh PNG tạo ra một đối tượng LINE chưa đủ để kết luận đúng. Một nét có thể bị lệch vị trí, ngược chiều trục Y hoặc sai tỷ lệ CAD/pixel. Từ đợt 183, bài smoke test Windows dùng **ảnh thực có 51 pixel đen**, nằm ngang từ **x=6 đến x=56**, tại **y=15**, trong ảnh **64 × 32 pixel**, với `unitsPerPixel=10`. Đoạn thẳng nối tâm pixel đầu và cuối có chiều dài hình học **50 khoảng pixel**; kết quả đúng là **500 đơn vị CAD**, không phải 510.

### Bài 23 — Tự thực hành kiểm tra ảnh, tỷ lệ và tọa độ trong CAD 360

1. Tạo bản vẽ mới `BT23_KiemTraTyLeAI.dxf`, dùng đơn vị CAD phù hợp với ví dụ, không mở một file sản xuất đang có đối tượng khác.
2. Chuẩn bị ảnh PNG **64 × 32 px**, nền trắng, một đường đen ngang từ pixel x=6 đến x=56 tại y=15. Không vẽ thêm chữ, viền hay điểm đen thừa.
3. Trong tab **CAD** chọn **AI TỪ ẢNH → CAD**, nạp ảnh PNG. Đặt **Nét tối thiểu = 16 px** và **Đơn vị CAD/pixel = 10**.
4. Nhấn **Nhận dạng và xem trước**. Chờ kết quả chỉ có **01 nét**. Nếu nhận dạng ra nhiều hơn hoặc không có nét, kiểm tra ảnh và ngưỡng; không tạo ngay.
5. Đối chiếu hệ tọa độ của nét khi dùng gốc (0,0): **điểm đầu (65,165)** và **điểm cuối (565,165)**. Công thức gốc sử dụng tâm pixel: `(x+0.5)×10` cho X; `(32-y-0.5)×10` cho Y.
6. Chọn **Tạo LINE chỉnh sửa trong CAD**. Dùng Properties hoặc công cụ `DIST` đo lại đúng **500 đơn vị CAD**, nằm ngang.
7. Thử **UNDO**: đoạn AI vừa tạo phải biến mất. Nhận dạng lại, tạo lại, lưu DXF rồi đóng/mở lại kiểm tra.
8. Thực hiện thử xuất DWG từ bản sao và mở bằng CAD 360; muốn kết luận DWG chuẩn tương thích AutoCAD phải mở/chỉnh sửa/lưu bằng **AutoCAD độc lập**.

### Bài 24 — Kiểm tra Setup và Portable có cùng kết quả MCP

1. Vào GitHub Actions của workflow **CAD 360 V1.025 - Windows Setup Portable**, chọn run có trạng thái **Success** thuộc đúng commit.
2. Tải **Setup EXE** và **Portable ZIP** cùng run. Không trộn file từ run khác.
3. Trên bản Setup, chạy `scripts/verify_mcp_executable.ps1` với `-Exe` trỏ tới **CADTBT360_CLI.exe** của thư mục cài đặt. Bài thử tự tạo ảnh và kiểm chứng Preview, LINE, tọa độ, chiều dài, UNDO, lưu DXF/DWG.
4. Giải nén Portable, lặp lại lệnh cùng script nhưng trỏ `-Exe` vào **CADTBT360_CLI.exe** ở thư mục Portable.
5. Cả hai phải trả về `MCP smoke PASS` và có kết quả **500 CAD units**. Nếu một bên sai, không được công bố hai gói cài đặt tương đương.
6. Đối chiếu `BUILD_INFO.txt`, `SHA256SUMS.txt` và ZIP mã nguồn đầy đủ. Trạng thái lệnh lỗi hoặc thiếu file hướng dẫn đều phải ghi trong phiếu nghiệm thu.

### Kiểm soát dữ liệu kiểm thử

Script `scripts/validate_mcp_smoke_fixture.py` kiểm tra PNG tới **từng tọa độ pixel x=6…56, y=15**. Script `scripts/verify_mcp_executable.ps1` kiểm tra LINE đã quy đổi là **(65,165) → (565,165)**, chiều dài 500. Windows CI tiếp tục yêu cầu giáo trình Word tổng hợp có **đủ 21 bài cơ bản/nâng cao**, bảng Word thật và tài liệu chuyên đề. Đây là **điều kiện kiểm thử được bổ sung trong mã nguồn**, chưa phải xác nhận lượt Windows CI đã PASS.

**Giới hạn:** Bộ thử ảnh này là một nét đơn, chưa chứng minh AI nhận dạng tường/cửa/cột, chữ kích thước hoặc mặt bằng kiến trúc thực tế. Không tự nâng trạng thái chức năng AI ngữ nghĩa lên Đạt.


## 24. Đợt 184 — Bảng tra cứu nhanh và kiểm tra chất lượng hướng dẫn

### 24.1. Bảng tra cứu lệnh CAD 2D dùng trong giáo trình

Bảng này là **bản đồ học lệnh**, không thay thế bảng **Help → Command Reference** hiển thị thực tế trên từng phiên bản. Gõ đúng tên lệnh trong Command Line trên tab CAD và dùng bản vẽ thử.

| Lệnh / nhóm lệnh | Công việc sử dụng | Điều kiện kiểm tra |
|---|---|---|
| NEW, OPEN, SAVEAS | Tạo/mở/lưu bản vẽ | Mở lại DXF không thay đổi nét |
| LINE, PLINE | Đường thẳng và đường bao | LINE chỉnh sửa được; PLINE khép kín khi cần |
| CIRCLE, ARC | Tròn và cung tròn | Bán kính/tâm/điểm đúng |
| RECTANG, POLYGON | Hình chữ nhật/đa giác | Đối tượng ở đúng Layer |
| MOVE, COPY, ROTATE | Dời/sao chép/xoay | Đo lại tọa độ và góc |
| OFFSET, TRIM, EXTEND | Tường song song, cắt/nối nét | Không có nét thừa hoặc bị mất |
| SCALE, MIRROR, ARRAY | Thay tỷ lệ, đối xứng, dãy | Không thay tỷ lệ bản vẽ ngoài ý muốn |
| FILLET, CHAMFER | Bo góc, vát góc | Bán kính/khoảng vát đúng |
| LAYER | Tổ chức bản vẽ theo lớp | Bật/tắt/khóa không làm mất entity |
| BLOCK, INSERT | Ký hiệu sử dụng lặp lại | Kiểm tra Block Definition và số lần Insert |
| XREF | Tham chiếu bản vẽ bên ngoài | Đường dẫn nguồn hợp lệ |
| IMAGEATTACH, IMAGECLIP | Ảnh nền và cắt ảnh | Không nhầm Image với LINE |
| TEXT, MTEXT, STYLE | Ghi chú và kiểu chữ | Tiếng Việt hiển thị, không lỗi font |
| DIMLINEAR, DIMSTYLE | Ghi kích thước | Giá trị khớp DIST |
| DIST, AREA | Kiểm tra chiều dài/diện tích | Quy đổi mm/m và mm²/m² đúng |
| ZOOM, PAN | Điều hướng trong CAD | Không thay đổi tọa độ entity |
| HATCH | Tô vật liệu/vùng kín | Hatch không tràn khỏi boundary |
| UNDO, REDO | Hoàn tác và làm lại | Không ảnh hưởng đối tượng khác |
| PLOT | In và xuất bản vẽ | Kiểm tra bản PDF bằng trình đọc độc lập |

### 24.2. Bài 25 — Kiểm tra bộ giáo trình đi kèm Setup/Portable

1. Lấy **Setup EXE** và **Portable ZIP** từ cùng một lượt GitHub Actions được xác nhận **Success**; không trộn tệp phát hành khác commit.
2. Cài Setup vào thư mục thử và kiểm tra `HuongDan/GIAO_TRINH_TONG_HOP_CAD360.docx`. Mở tệp bằng Microsoft Word hoặc trình đọc DOCX tương thích.
3. Giải nén Portable, kiểm tra `GIAO_TRINH_TONG_HOP_CAD360.docx` nằm cùng thư mục `CADTBT360.exe`. Mở và đối chiếu nội dung với bản Setup.
4. Dùng tìm kiếm trong Word để tra các tên **`CADTBT360_CLI.exe`**, **`BT01_CAD_CoBan.dxf`**, **`GIAO_TRINH_TONG_HOP_CAD360.docx`**. Ba tên phải giữ nguyên dấu gạch dưới; nếu bị thành `CADTBT360CLI.exe` hoặc `BT01CADCoBan.dxf` là lỗi bộ sinh tài liệu.
5. Chuyển đến chương có bảng tra cứu lệnh và ma trận nghiệm thu; xác nhận là **bảng Word thật** có thể chọn ô, không phải một dòng văn bản có dấu `|`.
6. Kiểm tra 21 bài thực hành liên tục theo từng nhóm CAD/QTO/BIM/IFC/AI/MCP. Bản Word cũng có phần nâng cao và quy trình kiểm thử.
7. Tại tab CAD, nhấn **HƯỚNG DẪN CAD 360** để mở file tổng hợp, đối chiếu có thể tra cứu đủ chương. Kiểm tra **HƯỚNG DẪN AI** chỉ là hướng dẫn chuyên đề.
8. Kiểm tra `BUILD_INFO.txt` và bảng `SHA256SUMS.txt`; nếu chưa có bộ cài đã chạy thật, đánh dấu **Chưa kiểm chứng**, không ghi Đạt.

**Thay đổi trong mã nguồn đợt 184:** Bộ sinh `docs/user-guide/build_word_guide.py` không còn xóa ký tự gạch dưới `_` trong tên file khi chuyển Markdown thành DOCX. `scripts/validate_word_guide.py` bắt buộc tìm được các tên tệp chính xác trước khi cho phép đóng gói Windows.

**Giới hạn nghiệm thu:** Chưa có xác nhận Windows CI PASS cho các commit mới và chưa có ảnh chụp thao tác thực tế từ EXE đã phát hành. Phải tiếp tục kiểm tra DWG hai chiều bằng AutoCAD độc lập trước khi tuyên bố tương thích hoàn toàn.


## 25. Đợt 185 — Chẩn đoán lỗi ảnh thành nét CAD và xác minh độc lập bằng hai tầng kiểm thử

**Mục đích:** Không còn kết luận “AI đúng” chỉ vì lệnh MCP trả về một LINE. Tách riêng (a) kiểm tra thuật toán Rust giải mã ảnh và quy đổi hình học, (b) kiểm tra CLI/MCP của chính file EXE trong Setup và Portable.

### 25.1. Bài 26 — Kiểm tra hình học ảnh PNG ngay ở lõi Rust

1. Mở thư mục mã nguồn ZIP của đúng commit đang thử, vào thư mục `CADTBT360_V1.000`.
2. Kiểm tra có Rust toolchain phù hợp Cargo.lock, sau đó chạy:
   `cargo test --locked -p cadcraft-mcp --lib raster_vector_preview_real_png_has_exact_scaled_world_coordinates -- --nocapture`
3. Bài kiểm thử tự tạo **ảnh RGB PNG 64×32** với một nét đen ngang từ `x=6` đến `x=56` tại `y=15`. Ảnh không có khung hoặc nét phụ.
4. Luồng thực hiện đi qua hàm `preview` thật: giải mã PNG, tìm nét đậm, quy đổi `unitsPerPixel=10`, đảo trục Y theo chiều cao ảnh.
5. Đối chiếu kết quả kỳ vọng: `imageSize=[64,32]`, `strokes=1`, `editableEntitiesCreated=0`, `lines=[[[65,165],[565,165]]]`.
6. Độ dài là **500 đơn vị CAD** do 51 pixel tạo thành 50 khoảng giữa tâm pixel. Nếu khác, lỗi thuộc nhận dạng/tọa độ trước khi xét đến đóng gói Windows.
7. Lưu toàn bộ log lệnh. Không thay giá trị kỳ vọng để ép PASS nếu chưa kiểm tra lại ảnh nguồn và thuật toán.

**Đạt ở mức lõi:** Rust test PASS đúng trị số. Điều này **chưa** chứng minh DLL/EXE Windows và giao diện dùng tốt.

### 25.2. Bài 27 — Kiểm tra MCP trên EXE đã cài và Portable

1. Sau khi GitHub Actions **Success**, tải đúng Setup và Portable của cùng run/commit.
2. Xác nhận `CADTBT360_CLI.exe` tồn tại trong cả thư mục Setup và thư mục Portable giải nén.
3. Trên Windows PowerShell, tại gốc mã nguồn chạy `./scripts/verify_mcp_executable.ps1 -Exe "ĐƯỜNG_DẪN_ĐẾN_CADTBT360_CLI.exe"` cho **từng** bản.
4. Script tự chạy JSON-RPC `initialize`, `tools/list`, kiểm thử ảnh không tồn tại được từ chối an toàn, Preview ảnh thật, tạo LINE, UNDO, tạo lại LINE, lưu DXF, chuyển sang DWG và mở lại cả hai qua CLI.
5. Kiểm tra Preview của cả hai bản đều là `(65,165) → (565,165)`, chiều dài 500, số entity sau tạo 1, sau UNDO 0, sau tạo lại 1.
6. Đối chiếu `BUILD_INFO.txt`, `SHA256SUMS.txt`, file mã nguồn và tài liệu Word. Nếu hai bản có kết quả khác nhau, không thể ghi nghiệm thu Setup = Portable.
7. Kiểm tra tiếp trên **GUI**: mở DXF, chọn LINE và điều chỉnh được; thử một bản DWG bằng AutoCAD độc lập để xác nhận tương thích hai chiều.

### 25.3. Bảng xác định nhóm lỗi

| Bước lỗi | Nhóm nguyên nhân cần điều tra | Không nên kết luận |
|---|---|---|
| Xác thực PNG thất bại | Base64, CRC, kích thước hoặc pixel | Thuật toán CAD sai |
| Rust preview sai tọa độ | Tỷ lệ, dấu trục Y, tâm pixel, trích nét | Setup/Portable hỏng |
| Rust PASS nhưng MCP thất bại | JSON-RPC, CLI, đường dẫn/phiên bản chạy | AI nhận dạng tường/cửa/cột sai |
| MCP PASS nhưng DXF/DWG mở lại lỗi | Writer, parser, bảo toàn entities | Đạt DWG tương thích AutoCAD |
| CLI PASS nhưng GUI không mở được | Giao diện, driver, tài nguyên, cài đặt | Toàn bộ phần mềm đã đạt |
| Bản Word thiếu bảng/tên file | Bộ sinh DOCX, kiểm tra định dạng | Thiếu tính năng CAD |

**Giới hạn hiện tại:** Kiểm thử ảnh đơn sắc chỉ xác nhận vector hóa một nét. AI hiểu tường/cửa/cột, chuẩn DWG tương thích AutoCAD, đầy đủ nội dung PDF và chức năng Cloud BIM đều cần bài kiểm thử riêng. Chưa có bằng chứng Windows CI PASS hoặc ảnh chụp giao diện từ EXE mới cho đợt 185.


## 26. Đợt 186 — Hiệu chuẩn AI từ ảnh thành CAD theo kích thước đã biết

### Bài 28 — Người mới hiệu chuẩn AI từ khoảng pixel và chiều dài thực

1. Mở bản vẽ thực hành `BT28_AI_HieuChuan.dxf` trong tab **CAD**, chọn **AI TỪ ẢNH → CAD** và nạp ảnh PNG/JPEG đường nét rõ.
2. Xác định hai điểm chuẩn trên ảnh và **đo khoảng cách giữa hai điểm theo pixel** bằng một công cụ xem ảnh có hiển thị tọa độ; đó là khoảng cách giữa hai vị trí, không phải số pixel tô đen tính cả hai đầu.
3. Xác định chiều dài thực của cùng đoạn chuẩn theo đơn vị dùng trong bản CAD. Ví dụ hai điểm cách **50 px**, chiều dài thật là **500 mm**.
4. Trong nhóm **Hiệu chuẩn tỷ lệ theo chiều dài đã biết**, nhập **Khoảng cách trên ảnh (pixel) = 50**, **Chiều dài thực (đơn vị CAD) = 500**, nhấn **Tính tỷ lệ từ đoạn chuẩn**.
5. CAD 360 tính tỷ lệ **10 CAD unit/pixel** và yêu cầu **Nhận dạng và xem trước** lại. Tỷ lệ được sử dụng cho lần Preview tiếp theo; Preview cũ bị loại để tránh ghi vào sai kích thước.
6. Quan sát kích thước một nét chuẩn trong Preview; chỉ nhấn **Tạo LINE chỉnh sửa trong CAD** sau khi xác nhận chiều dài và vị trí hợp lý.
7. Dùng **DIST** hoặc Properties kiểm tra chiều dài đoạn vừa vẽ; ví dụ hai tâm pixel cách nhau 50 px phải cho **500 đơn vị CAD**.
8. Thử nhập **0 pixel**, **chiều dài 0**, giá trị quá nhỏ/quá lớn hoặc vô hạn bằng dữ liệu thử nghiệm. Phần mềm phải báo từ chối tỷ lệ không hợp lệ và không tự thay đổi nét đã tạo.
9. Lưu bản sao DXF/DWG và mở lại. Khi kiểm chứng DWG tương thích đầy đủ phải thử bằng AutoCAD độc lập, không chỉ dùng CLI của CAD 360.

**Phạm vi chính xác:** Phương pháp hiệu chuẩn này nhận **hai con số do người dùng đo và nhập**. Đây là bổ sung tương tác để đặt tỷ lệ hình học; **không phải** OCR đọc chữ kích thước và **không phải** AI tự nhận dạng tường, cửa, cột hoặc bản vẽ kiến trúc.

### Bài 29 — Kiểm tra tự động hiệu chuẩn và bộ cài

1. Trong ZIP mã nguồn tại commit đợt 186, vào `CADTBT360_V1.000`, dùng Cargo và Rust toolchain tương thích.
2. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_raster_calibration -- --nocapture`.
3. Kiểm tra bài `ai_raster_calibration_uses_known_length_and_pixel_distance` xác minh **50 px × 10 CAD/pixel = 500 CAD units**, và Preview vẫn không chỉnh sửa bản vẽ.
4. Kiểm tra bài `ai_raster_calibration_rejects_zero_nan_and_oversize_ratio` từ chối chia cho 0, NaN, Infinity, tỷ lệ quá nhỏ/quá lớn.
5. Sau khi Rust test đạt, kiểm tra bước tương ứng trong Windows GitHub Actions **CAD 360 V1.025 - Windows Setup Portable**.
6. Chạy **Setup EXE** và **Portable** từ cùng một lượt build Success, mở nút AI trên GUI thật, nhập số hiệu chuẩn và chụp màn hình kiểm chứng.
7. So sánh kết quả đo trên DXF/DWG, ghi rõ mã commit và dùng giáo trình Word tổng hợp để đối chiếu.

**Giới hạn nghiệm thu:** Chưa có kết quả Windows CI PASS cho commit đợt 186. Code hiệu chuẩn đã đưa vào giao diện và CI nhưng chưa có ảnh chụp GUI Windows thực tế, chưa xác nhận tất cả thiết bị chạy được.


## 27. Đợt 187 — Từ ảnh PNG/JPEG thành đường bao PLINE khép kín

**Nâng cấp chức năng AI theo hướng có cấu trúc:** Từ các nét ảnh đã vector hóa, CAD 360 có chế độ **tùy chọn** phát hiện một đường bao hình chữ nhật có đủ bốn cạnh, tạo **một đối tượng PLINE đóng** thay cho bốn LINE chồng lên nhau. Đây là xử lý hình học có điều kiện, **không phải** tự suy luận kiến trúc, tên phòng, tường, cửa hay cột.

### Bài 30 — Tạo đường bao hình chữ nhật khép kín từ ảnh

1. Mở bản vẽ thực hành `BT30_AI_DuongBaoKin.dxf` trong **CAD**, chọn **AI TỪ ẢNH → CAD**.
2. Chuẩn bị PNG nền trắng có một hình chữ nhật mảnh, bốn cạnh thẳng ngang/dọc nối liền nhau, không có chữ/kích thước chồng lên cạnh.
3. Nhấn **Chọn ảnh PNG/JPG…** và chọn ảnh. Khai báo ngưỡng nét, **Đơn vị CAD/pixel** hoặc dùng công cụ **Tính tỷ lệ từ đoạn chuẩn** trước khi Preview.
4. Tích chọn **Đường bao chữ nhật kín → PLINE**. Chức năng mặc định **tắt** để bảo vệ tương thích các bản vẽ dùng LINE trước đây.
5. Nhấn **1. Nhận dạng và xem trước**; kiểm tra màn hình hiển thị số đường bao PLINE kín. Khi khung có đủ bốn cạnh, kết quả dự kiến là **1 PLINE đóng**, không nhân thêm bốn LINE trùng cùng ranh giới.
6. Nhấn **2. Tạo LINE / PLINE chỉnh sửa trong CAD**. Chọn PLINE trên canvas, kiểm tra thuộc tính **Closed** và các đỉnh của hình chữ nhật.
7. Dùng `AREA` hoặc công cụ QTO để kiểm tra diện tích hình học. Không tự xem đường bao là một cấu kiện tường BIM hay diện tích sàn khi chưa gán nghiệp vụ.
8. Nhấn **UNDO** một lần để hoàn tác nhóm AI, rồi làm lại và lưu DXF hoặc DWG trong thư mục bài tập.
9. Đóng/mở lại file để xác nhận PLINE không biến thành raster. Để nghiệm thu DWG tương thích AutoCAD hai chiều, phải thử độc lập với AutoCAD.

**Cần đạt:** Một hình chữ nhật bốn cạnh hợp lệ tạo **một native closed POLYLINE**, chỉnh sửa và đo diện tích được, không còn LINE trùng cạnh do cùng một nguồn ảnh.

### Bài 31 — Không tự lấp khoảng hở, cửa đi và cạnh thiếu

1. Sao chép ảnh bài 30, xóa toàn bộ một cạnh của hình chữ nhật để giả lập bản vẽ chưa đầy đủ hoặc cửa mở.
2. Nạp ảnh mới, vẫn bật **Đường bao chữ nhật kín → PLINE** và Preview lại.
3. Kiểm tra hệ thống **không** phát hiện PLINE đóng từ ba cạnh; các nét hợp lệ còn lại tiếp tục là LINE để người dùng chỉnh sửa thủ công.
4. Nếu hình có một khoảng hở trên cạnh, không tự động gọi đó là tường/cửa. Người dùng cần quyết định có nối đoạn bằng lệnh CAD sau khi xem xét bản vẽ thật.
5. Dùng **UNDO** hoặc xóa đối tượng thử; lưu kết quả nghiệm thu "Không đóng đường bao thiếu cạnh" kèm ảnh chụp giao diện thực tế.

**Cần đạt:** Không tự thêm cấu kiện hoặc cạnh bị thiếu; ưu tiên an toàn bản vẽ hơn việc ép tạo hình kín.

### Bài 32 — Kiểm thử Rust và giao diện cho PLINE từ ảnh

1. Trong `CADTBT360_V1.000` chạy `cargo test --locked -p cadcraft-mcp --lib raster_rectangular_png -- --nocapture`; kiểm tra hai bài test với ảnh PNG thật: bốn cạnh tạo một PLINE, thiếu cạnh không tự đóng.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib raster_closed_rectangles_recognizes_four_edges_and_refuses_missing_side -- --nocapture` để kiểm tra detector trên nét vector.
3. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_raster_closed_rectangle_mode_is_opt_in_and_preserves_preview_safety -- --nocapture` để kiểm tra chế độ tùy chọn và Preview trên GUI.
4. Với MCP, công cụ `raster_to_cad_lines` chấp nhận tham số **`closedRectangles: true`** và trả về `rectanglesDetected`, `closedPolylines`, `closedPolylinesCreated` khi phù hợp.
5. Khi **`closedRectangles: false`**, hành vi LINE như cũ; không tự chuyển hình chữ nhật thành PLINE.
6. Chỉ kết luận **đạt** khi có Windows CI Success, kết quả chạy trên chính Setup/Portable, file DXF/DWG mở lại được và có ảnh chụp GUI thật. Chưa có xác nhận này trong đợt 187.

**Giới hạn cần giữ rõ:** Chức năng chỉ phát hiện hình chữ nhật theo trục ngang/dọc với đủ bốn cạnh. Hình nghiêng, nét gãy, nhiều phòng liên thông hoặc tường dày giao cắt cần các bộ nhận dạng chuyên sâu khác. Vẫn chưa có mô hình AI ngữ nghĩa đọc bản vẽ tường, cửa, cột.


## 28. Đợt 188 — Gợi ý mép tường song song từ ảnh mà không làm sai bản CAD

Chế độ **Gợi ý cặp nét tường song song (chỉ Preview)** bổ sung thông tin hình học từ các nét ảnh PNG/JPEG. Thuật toán tìm các cặp LINE ngang hoặc dọc tương đối dài, có đoạn chồng lấn đủ lớn, khoảng cách trong ảnh từ **2 đến 40 pixel**. Kết quả là **ứng viên** để kỹ sư kiểm tra, không phải nhận dạng chắc chắn tường, không có đối tượng BIM Wall.

### Bài 33 — Xem trước các cặp nét song song nghi là tường

1. Chuẩn bị ảnh PNG mặt bằng thử có hai nét dài nằm ngang, gần song song, ví dụ nét thứ nhất từ x=10 đến x=110 tại y=10, nét thứ hai từ x=10 đến x=110 tại y=20.
2. Trong tab CAD chọn **AI TỪ ẢNH → CAD**, mở ảnh và chọn đơn vị CAD/pixel hoặc hiệu chuẩn theo một đoạn có chiều dài đã biết.
3. Tích **Gợi ý cặp nét tường song song (chỉ Preview)**. Mặc định chế độ này tắt, không ảnh hưởng các file cũ.
4. Nhấn **Nhận dạng và xem trước**; đọc số lượng **cặp nét song song nghi là mép tường**.
5. Kiểm tra hình học: hai nét cách nhau 10 pixel, đoạn chồng lấn đủ dài; phần mềm báo một ứng viên. Người dùng quyết định hai nét này có thực sự là hai mép tường hay chỉ là ký hiệu/hai đường khác.
6. Không có tường tự động được dựng. Nếu nhấn **Tạo LINE / PLINE chỉnh sửa trong CAD**, các nét vẫn là đối tượng CAD hình học để chỉnh tay.
7. Dùng `DIST` để đo khoảng cách hai mép và đối chiếu bề dày tường sau khi hiệu chuẩn. Không gán bề dày tường nếu chưa hiểu đúng tỷ lệ và đơn vị bản vẽ.

**Nghiệm thu:** Có gợi ý đúng cặp song song trên ảnh thử, không tự thêm hoặc xóa bất kỳ hình học nào chỉ vì người dùng bật chế độ gợi ý.

### Bài 34 — Kiểm tra chống nhận nhầm tường

1. Tạo hai đoạn LINE ngang dài nhưng **không chồng lấn** theo trục X; đây không phải hai mép cùng một dải tường.
2. Tạo thêm hai LINE thẳng đứng cách nhau trên 40 pixel; khoảng cách ngoài ngưỡng, không được ghép thành một ứng viên.
3. Thêm một đường giao cắt vuông góc; chỉ những cặp **cùng phương, có chồng lấn đủ lớn và nằm gần nhau** mới được gợi ý.
4. Chạy lại Preview và kiểm tra số ứng viên. Nếu nhận diện bất thường, lưu ảnh nguồn, kích thước ảnh, thông số threshold/minLength và log.
5. Tắt chế độ **Gợi ý cặp nét tường song song** và Preview lại. Kết quả số gợi ý phải trở về 0; nét LINE của bản vẽ không bị thay đổi.

**Nghiệm thu:** Không nhận nhầm nét giao nhau, nét rời, hoặc nét song song quá xa thành tường.

### Bài 35 — Tự động kiểm thử bằng Rust và Windows CI

1. Tại thư mục `CADTBT360_V1.000`, dùng Rust/Cargo của dự án.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_parallel_wall_ -- --nocapture` để kiểm tra ghép cặp và từ chối các trường hợp hình học không hợp lệ.
3. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_raster_wall_pair_hints_are_opt_in_and_non_mutating -- --nocapture` để kiểm tra tùy chọn tắt mặc định và Preview không có quyền ghi.
4. Xác nhận workflow Windows V1.025 có hai bộ kiểm thử trên, sau đó mới đánh giá trạng thái **Success/Failure** của đúng commit.
5. Nếu CI thành công và có Setup/Portable, mở GUI thật để chụp ảnh nút, màn hình Preview và các nét CAD. Không lấy ảnh minh họa trên Internet làm bằng chứng phần mềm.
6. Thực hiện thêm kiểm thử ảnh mặt bằng thật nhiều lớp nét, số ứng viên đúng/sai, thời gian xử lý, RAM và tính ổn định trước khi dùng trong sản xuất.

**Giới hạn quan trọng:** Chỉ là phát hiện **ứng viên cặp mép tường từ nét song song**. Chưa có semantic AI phân loại vật liệu, cửa, cửa sổ, cầu thang, chưa gán Layer tường, và chưa sinh mô hình BIM. Không được dùng số ứng viên như khối lượng tường để quyết toán nếu chưa kiểm tra bằng kỹ sư.

**Trạng thái đợt 188:** Đã đưa mã nguồn phát hiện ứng viên, giao diện xem trước, mô tả MCP và kiểm thử CI vào repository. Chưa có xác nhận Windows Build PASS và ảnh GUI từ EXE được nghiệm thu.


## 29. Đợt 189 — Tạo đường tim tường CAD từ hai mép tường song song đã kiểm tra

Nâng cấp trực tiếp trên giao diện AI ảnh → CAD và MCP: từ **hai LINE ứng viên mép tường** có khoảng cách và chiều dài chồng lấn phù hợp, phần mềm có thể đề xuất **LINE tim tường**. Người dùng phải bật riêng `Gợi ý cặp nét tường song song (chỉ Preview)` rồi mới được bật `Tạo tim tường từ cặp nét duy nhất (tùy chọn)`. Chế độ mặc định tắt; hệ thống **giữ nguyên các LINE mép tường gốc** và **không sinh BIM Wall**.

### Bài 36 — Thực hành từ ảnh tường đôi thành LINE tim tường

1. Trong tab **CAD**, tạo bản vẽ `BT36_TimTuongTuAnh.dxf`; chọn **AI TỪ ẢNH → CAD**, tải PNG có hai nét đen ngang dài và song song.
2. Dùng ảnh mẫu có hai đoạn ngang cùng chạy từ **x=10 đến x=110**, tại **y=12** và **y=22**, trong ảnh **128×64 pixel**. Mỗi đoạn có 101 điểm đen, khoảng cách hai tâm đường là 10 pixel.
3. Đặt ngưỡng nhận dạng và **Đơn vị CAD/pixel = 10** sau hiệu chuẩn; tắt chế độ nhận dạng nét góc nếu ảnh chỉ có hai đoạn ngang.
4. Bật **Gợi ý cặp nét tường song song (chỉ Preview)**. Kiểm tra có 01 cặp ứng viên và khoảng cách 10 pixel; người dùng xác nhận đây thực sự là hai mép tường.
5. Tích thêm **Tạo tim tường từ cặp nét duy nhất (tùy chọn)**; bấm **Nhận dạng và xem trước** để xem đề xuất.
6. Đối chiếu ba LINE sẽ xuất hiện khi tạo: **hai LINE mép** và **một LINE tim**. Với gốc (0,0), tâm nét tim theo ảnh mẫu phải có tọa độ CAD **(105,465) → (1105,465)**, dài **1000 đơn vị CAD**.
7. Chọn **Tạo LINE / PLINE chỉnh sửa trong CAD**. Chọn từng LINE trong bản vẽ, kiểm tra tim là hình học native, đo `DIST` và lưu bản DXF.
8. Nhấn **UNDO một lần** để hoàn tác cả ba LINE của nhóm AI. Sau đó tạo lại và lưu DXF/DWG bản sao; kiểm tra độc lập trong AutoCAD trước khi nghiệm thu tương thích DWG hai chiều.

**Điều kiện đạt trên EXE:** Chỉ tạo LINE tim khi có hai mép hợp lệ và tùy chọn được bật. Giữ hai LINE gốc và Undo hoàn tác tất cả đối tượng AI vừa tạo. Đây là tim tường hình học, chưa có vật liệu, độ dày BIM, tên phòng hay quan hệ không gian.

### Bài 37 — Tự từ chối ghép nhầm khi nhiều mép tường

1. Tạo ảnh/nét với ba đoạn thẳng ngang song song gần nhau, ví dụ y=10, y=20 và y=30, chồng lấn dọc theo cùng khoảng x.
2. Mở AI, bật gợi ý cặp mép và tùy chọn tim tường. Trường hợp này có **nhiều cách bắt cặp**, vì một nét có thể tham gia hơn một ứng viên.
3. Kiểm tra thuật toán **không tạo LINE tim tường** từ cặp nhập nhằng; chỉ giữ các LINE gốc cho người dùng chỉnh sửa.
4. Kiểm tra thêm hai đoạn rời không chồng lấn và hai đoạn vuông góc: chúng không được hiểu là hai mép của cùng một tường.
5. Tạo một đường bao chữ nhật đã chuyển sang PLINE; các cạnh đã đưa vào PLINE không được đồng thời dùng làm tim tường từ các LINE nguồn đã loại.
6. Lưu màn hình Preview và bản vẽ kiểm tra, ghi tỷ lệ, số cặp ứng viên và số tim tường tạo ra.

**Điều kiện đạt:** Không phán đoán từ ba hoặc nhiều nét song song nhập nhằng; tránh tự tạo tường giả trên mặt bằng xây dựng.

### Bài 38 — Nghiệm thu Rust, MCP và GUI của chức năng tim tường

1. Tại thư mục `CADTBT360_V1.000`, chạy `cargo test --locked -p cadcraft-mcp --lib ai_wall_centerline_ -- --nocapture`.
2. Xem bài thử **`ai_wall_centerline_real_png_keeps_both_edges_and_adds_editable_centerline`**: ảnh PNG RGB thật, 2 mép nguồn + 1 tim tường, sau Undo còn 0 entity.
3. Xem bài thử **`ai_wall_centerline_is_only_created_for_unique_parallel_pair`**: một cặp duy nhất được tạo tim; ba nét song song nhập nhằng và cạnh đã chuyển PLINE không được tạo.
4. Xem bài thử **`ai_wall_centerline_rejects_creation_without_hint_review`**: từ chối bật `createWallCenterlines=true` nếu `wallPairHints=false`.
5. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_raster_wall_centerline_creation_requires_explicit_user_opt_in -- --nocapture`.
6. Với MCP `raster_to_cad_lines`: `wallPairHints=true` trả thông tin cặp ứng viên, `createWallCenterlines=true` đề xuất `wallCenterlines` và khi `preview=false` tạo LINE thực tế trong một Undo group.
7. Chạy Windows CI với đúng commit và thử lại **Setup/Portable**. Nếu không có build Success hoặc ảnh chụp GUI thực tế, trạng thái vẫn là **Chưa kiểm chứng**.

**Trạng thái:** Mã nguồn, tùy chọn GUI/MCP và bài kiểm thử Rust/CI đã được bổ sung trong đợt 189, chưa xác minh Windows CI PASS và chưa nghiệm thu bộ cài mới. Chức năng AI hiểu tường/cửa/cột, OCR kích thước, tạo BIM Wall vẫn chưa hoàn thiện.


## 30. Đợt 190 — Hiệu chỉnh khoảng cách hai mép tường theo độ phân giải ảnh

Ở các ảnh scan độ phân giải khác nhau, cùng một bề dày tường thực tế có thể chiếm số pixel khác nhau. Đợt 190 bổ sung **Mép tường (px): từ ... đến ...** để kỹ sư chủ động giới hạn cặp đường song song mà AI được phép gợi ý. Tính năng có trong **CAD → AI TỪ ẢNH → CAD**, đồng thời cung cấp tham số MCP `minWallSpacingPixels` và `maxWallSpacingPixels`. Mặc định từ **2–40 px**; giới hạn hợp lệ **1 ≤ min ≤ max ≤ 80 px**. Thay thông số làm hủy Preview cũ và bắt buộc xem trước lại.

### Bài 39 — Tự điều chỉnh ngưỡng nhận dạng hai mép tường

1. Tạo bản vẽ thử `BT39_AIMepTuongTheoTyLe.dxf`, giữ nguyên bản sản xuất.
2. Nạp PNG có hai nét ngang cách nhau 10 pixel; dùng chức năng hiệu chuẩn theo chiều dài đã biết để nhập đơn vị CAD/pixel.
3. Bật **Gợi ý cặp nét tường song song (chỉ Preview)**; phần mềm hiển thị hai trường khoảng cách **từ** và **đến**, mặc định 2–40 px.
4. Giữ 2–40 px và Preview: hai nét đủ dài, có chồng lấn phù hợp sẽ được gợi ý là ứng viên.
5. Đổi thành **11–40 px** rồi Preview lại: cặp cách nhau 10 pixel phải bị loại.
6. Đổi thành **1–9 px** rồi Preview lại: cặp vẫn bị loại do vượt giới hạn 9 pixel.
7. Đặt **8–18 px**, Preview lại: cặp 10 pixel có thể được gợi ý trở lại.
8. Nếu muốn sinh LINE tim tường, tích thêm **Tạo tim tường từ cặp nét duy nhất (tùy chọn)**; không kích hoạt khi nhiều mép tường nhập nhằng.
9. Kiểm tra **hai mép LINE gốc vẫn được giữ nguyên** và Undo có thể hoàn tác nhóm AI.

### Bài 40 — Ngăn nhập ngưỡng sai và bảo vệ bản vẽ

1. Nhập **từ 30 px đến 10 px**. Phần mềm phải báo khoảng min/max không hợp lệ; không được xác nhận Preview tạo tim với dữ liệu sai.
2. Nếu gọi qua MCP, thử `minWallSpacingPixels=0`, `maxWallSpacingPixels=81`, giá trị âm, ký tự thay cho số hoặc `min > max`. Kết quả phải trả về lỗi rõ ràng và không ghi CAD.
3. Thử bật tính năng tim tường nhưng không bật **Gợi ý cặp nét tường song song**: hệ thống phải từ chối đề xuất tim để người dùng xem xét ứng viên trước.
4. Thay đổi ngưỡng sau một lần Preview và kiểm tra Preview cũ đã mất hiệu lực; phải nhấn **Nhận dạng và xem trước** lại.
5. Thử ảnh có ba nét gần nhau. Không được tạo LINE tim tường ở vùng nhập nhằng, dù ngưỡng cho phép.
6. Thử lưu file, hoàn tác một lần, và so sánh đối tượng CAD trước/sau; giữ bản sao khi thử Undo.

### Bài 41 — Kiểm thử Rust và phát hành Windows

1. Mở thư mục `CADTBT360_V1.000` ở ZIP mã nguồn tương ứng commit.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_wall_spacing_ -- --nocapture` và kiểm tra cả bộ lọc 2–40/11–40/1–9 và các trường hợp min/max không hợp lệ.
3. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_raster_wall_spacing_can_be_set_by_user_without_mutating_drawing -- --nocapture`.
4. Chạy các kiểm thử tim tường đã bổ sung ở đợt 189 để xác nhận không làm hỏng khả năng tạo LINE tim từ cặp duy nhất và Undo.
5. Trên GitHub Actions kiểm tra run Success theo **đúng commit**, sau đó cài Setup/giải nén Portable và thao tác ngưỡng trên giao diện Windows thật.
6. Ghi kết quả, đường dẫn bản DXF/DWG, SHA256, ảnh chụp giao diện thực tế; nếu chưa thực hiện bước này thì để trạng thái **Chưa kiểm chứng**.

**Giới hạn:** Đây là điều chỉnh bộ lọc **hình học pixel**, không phải AI ngữ nghĩa nhận diện tường, cửa, cột hay vật liệu; không thay thế kiểm tra kích thước thực bằng kỹ sư và không tự tạo BIM Wall. Chưa có bằng chứng Windows CI PASS hoặc bộ cài mới đã nghiệm thu.


## 31. Đợt 191 — AI đánh dấu giao tim tường và khoảng hở nghi là cửa

Nội dung nâng cấp kết nối hai phân tích **chỉ xem trước** trên ảnh PNG/JPEG: (a) giao các đoạn tim tường tại điểm giao dạng **T/chữ thập**, (b) những đoạn thẳng hàng có khoảng hở **3–40 pixel** có thể là khoảng mở. Không tự tạo cửa, không tự kéo dài tim tường và không tự nối các cạnh ảnh. Đây là gợi ý hình học, không phải AI nhận dạng kiến trúc chắc chắn.

### Bài 42 — Kiểm tra giao tim tường theo hình học chữ T và giao chữ thập

1. Mở bản vẽ thử `BT42_AI_GiaoTuong.dxf`, không làm trên hồ sơ sản xuất.
2. Chuẩn bị ảnh có hai cặp nét tường song song: một cặp ngang và một cặp đứng cắt nhau. Chọn tỉ lệ CAD/pixel và lọc ngưỡng khoảng cách cặp mép phù hợp.
3. Bật **Gợi ý cặp nét tường song song**, sau đó bật **Tạo tim tường từ cặp nét duy nhất (tùy chọn)**.
4. Nhấn **Nhận dạng và xem trước**, kiểm tra dòng `ứng viên giao tim tường T/chữ thập` và các đường tim dự kiến.
5. Trên hình giao chữ thập, hai tim phải cắt nhau ở phần giữa của từng đoạn. Trên hình chữ T, một tim kết thúc gần đoạn khác.
6. Trường hợp hai đầu đoạn chỉ chạm nhau hoặc nằm gần nhưng không cắt qua, phần mềm không được tự xác nhận là một giao tường.
7. Lưu ảnh giao diện của EXE thực tế, vị trí giao điểm, số LINE và mã commit. Không nhầm số giao tim với số giao cấu kiện BIM được nghiệm thu.
8. Chỉ bấm **Tạo LINE / PLINE** sau khi kiểm tra Preview. Giao điểm chỉ là metadata xem trước, không sinh thêm đối tượng hình học.

### Bài 43 — Nhận biết khoảng hở mà không tự bịt cửa

1. Chuẩn bị ảnh PNG gồm hai nét cùng đường thẳng hàng ngang: đoạn thứ nhất từ x=5.5 đến x=35.5, đoạn tiếp từ x=47.5 đến x=87.5; khoảng trống giữa hai đầu nét là 12 pixel.
2. Mở **AI TỪ ẢNH → CAD**, tích **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**.
3. Nhấn Preview; trường kết quả `openingGapCandidateCount` cần báo 1 cho ảnh mẫu. Trường `classification` chỉ mang ý nghĩa `unverified-gap-not-door`.
4. Kiểm tra hai nét nguồn vẫn còn độc lập, không được nối vào nhau, và không có Block cửa nào được tạo tự động.
5. Thử hai nét song song lệch hàng hoặc có đầu mút chạm nhau: không được gắn nhãn khoảng hở hợp lệ.
6. Nếu chủ động bật chức năng `maxGapPixels` ở bài học khác, cần hiểu đó là thao tác ghép nét và có thể làm mất khoảng hở. Để bảo vệ cửa, giữ **maxGapPixels=0** khi chưa xác minh.
7. Nếu cần dựng cửa đi, kỹ sư tiếp tục xác nhận trên ảnh và sử dụng lệnh CAD/Block theo quy trình kiểm tra riêng.

### Bài 44 — Kiểm thử Rust, MCP và Windows Setup

1. Vào thư mục `CADTBT360_V1.000` của ZIP mã nguồn đúng commit.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_wall_junction_ -- --nocapture`.
3. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_hints_ -- --nocapture`.
4. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_opening_gap_hints_are_disabled_by_default_and_preview_only -- --nocapture`.
5. Với MCP công cụ `raster_to_cad_lines`, dùng `openingGapHints=true` để xem các khoảng hở chưa xác minh; dùng `createWallCenterlines=true` cùng `wallPairHints=true` để nhận thêm số giao tim tường dự kiến.
6. Chạy Windows CI trên đúng commit; chỉ xác nhận thành công sau khi có **Success**, bộ cài và Portable. Chụp ảnh thực tế trên EXE và mở lại DXF/DWG trước khi đánh giá.
7. Đối chiếu hành vi khi tắt cả hai chức năng mới: bản vẽ phải giữ đúng LINE/PLINE theo phương thức cũ, không tự thêm hay xóa geometry.

**Phạm vi:** Đã đưa mã nguồn, GUI, MCP và CI cho ứng viên giao tường và khoảng mở vào repository. Chưa có bằng chứng Windows CI PASS. Việc nhận dạng chính xác cửa/cửa sổ, OCR kích thước, phân tách tường/cột và tạo mô hình BIM là công việc chưa hoàn thiện.


## 32. Đợt 192 — Bảo vệ khoảng mở cửa khi AI vector hóa bản vẽ

Đợt 192 tập trung vào việc tránh tự nối nhầm khoảng mở cửa trên mặt bằng scan. Trong công cụ MCP `raster_to_cad_lines`, chế độ **`openingGapHints=true`** gợi ý các khoảng trống chưa xác minh, còn **`maxGapPixels>0`** có chức năng nối các nét ngắt ngắn. Hai tác vụ này có mục đích trái nhau và từ đợt 192 được **chặn dùng đồng thời**. Muốn xem vị trí nghi là cửa phải để **`maxGapPixels=0`**.

### Bài 45 — Giữ nguyên khoảng hở cửa từ ảnh PNG

1. Tạo một bản vẽ mới `BT45_AI_GiuKhoangMo.dxf` để tránh ảnh hưởng hồ sơ công trình.
2. Chuẩn bị ảnh PNG nền trắng kích thước 110 × 40 pixel; vẽ nét thứ nhất từ x=5 đến x=39, nét thứ hai từ x=51 đến x=95, cùng cao độ y=17. Có một khoảng hở theo tâm nét dài **12 pixel**.
3. Chọn AI TỪ ẢNH → CAD. Đặt `minLengthPixels=16`, bỏ chọn nét góc nếu không có, và bật **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**.
4. Trong cấu hình MCP, dùng `openingGapHints=true` và `maxGapPixels=0`.
5. Nhấn **Nhận dạng và xem trước**, kiểm tra **2 LINE** còn tách biệt và **1 khoảng hở chưa xác minh**.
6. Chỉ tạo đối tượng CAD sau khi người dùng kiểm tra hình học. Không tự sinh Block cửa hoặc nối hai đoạn thành một tường liên tục.
7. Dùng DIST đo chiều rộng khoảng trống theo tỷ lệ CAD/pixel đã hiệu chuẩn. Ghi kết quả là *khoảng mở nghi vấn*, không ghi *cửa đi chính xác*.

### Bài 46 — Ngăn xung đột hai chế độ của AI

1. Giữ nguyên ảnh bài 45 và yêu cầu `openingGapHints=true`, `maxGapPixels=12`.
2. Phần mềm phải **trả lỗi trước khi ghi bản vẽ** với thông báo chế độ nhận biết khoảng mở yêu cầu `maxGapPixels=0`.
3. Nếu muốn nối các nét bị đứt do scan, tắt chế độ gợi ý khoảng mở và chỉ thực hiện trên bản vẽ thử đã biết rõ không có cửa đi/cửa sổ.
4. Nếu cần tìm cửa, giữ `maxGapPixels=0`, dùng thao tác kiểm tra thủ công để xác định vị trí mở. Không đánh đồng đoạn nét bị mờ với cửa thật.
5. Kiểm tra **Undo**, số lượng LINE/PLINE và độ mở trên DXF sau khi tạo. Việc chuyển DWG phải được kiểm chứng độc lập.

### Cách đọc dấu khoảng hở trên màn hình Preview

Khi bật **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**, màn hình Preview đánh dấu **hai đầu khoảng hở bằng vòng tròn màu vàng cam**. Hai dấu mốc này chỉ giúp nhận biết đoạn thiếu nét trên ảnh; chương trình **không vẽ đường nối màu cam** và không biến khoảng hở thành cửa đi đã xác định. Để xác minh cần mở ảnh nguồn, đo khoảng cách và xem tiếp đối tượng CAD sau khi tạo.

### Bài 47 — Kiểm thử Rust, MCP và nghiệm thu bộ cài

1. Từ ZIP mã nguồn đúng commit, tại `CADTBT360_V1.000`, chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_real_png_refuses_to_bridge_possible_door -- --nocapture`.
2. Bài test tự tạo ảnh PNG RGB với 2 nét ngang, xác nhận Preview có **2 LINE**, **1 khoảng hở**, rồi bắt buộc từ chối khi bật đồng thời `maxGapPixels=12`.
3. Chạy các test `ai_opening_gap_hints_`, `ai_wall_junction_` và `ai_wall_centerline_` để kiểm tra không làm hỏng chức năng đã có.
4. Xác minh Windows CI thực sự ở trạng thái **Success** trên commit phù hợp. Chỉ sau đó chạy Setup EXE và Portable, nạp cùng ảnh, chụp màn hình thực tế và thử lưu/mở lại DXF/DWG.
5. Nếu CI chưa PASS hoặc chưa có bằng chứng GUI thực tế, ghi **Chưa nghiệm thu**, không coi đây là chức năng sản xuất hoàn thiện.

### Bảng tình huống an toàn AI

| Trạng thái nhập | Cách xử lý bắt buộc | Có được tự nối cửa? |
|---|---|---|
| openingGapHints=true, maxGapPixels=0 | Preview khoảng hở và giữ LINE gốc | Không |
| openingGapHints=true, maxGapPixels=12 | Từ chối thao tác, báo lỗi rõ ràng | Không |
| openingGapHints=false, maxGapPixels=12 | Có thể ghép nét ngắt ngắn theo điều kiện | Chỉ thử nghiệm, cần kỹ sư duyệt |
| Hai nét khác hàng hoặc chỉ chạm đầu | Không nhận là khoảng mở hợp lệ | Không |

**Giới hạn:** Đây là cải tiến bảo vệ hình học trong công cụ vector hóa, không phải AI OCR hoặc mô hình ngữ nghĩa xác định chắc chắn tường, cửa, cột. Chưa xác minh Windows CI PASS hoặc bộ cài EXE mới.


## 33. Đợt 193 — Đo kích thước khoảng mở từ ảnh theo đúng tỷ lệ CAD

Đợt 193 bổ sung khả năng **lọc và đo khoảng hở chưa xác minh** trên ảnh PNG/JPEG. Trong **CAD → AI TỪ ẢNH → CAD**, chọn **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**, nhập **Khoảng mở (px): từ … đến …**. Mặc định 3–40 pixel; giới hạn hợp lệ từ **1 đến 120 pixel**, và giá trị tối thiểu không được vượt tối đa. Khi thay ngưỡng, phải Preview lại.

AI trả các trường `gapPixels`, `widthCAD`, `fromCAD` và `toCAD`, sử dụng tỷ lệ `unitsPerPixel` cùng gốc `origin` để quy đổi. Kết quả là **thông tin đo**, không phải kích thước cửa thật, vì một khoảng mờ trên ảnh cũng có thể là nét đứt do scan.

### Bài 48 — Đo độ rộng khoảng mở với hiệu chuẩn 10 đơn vị CAD/pixel

1. Trong CAD 360 tạo bản vẽ thử `BT48_AI_DoKhoangMo.dxf`. Mở AI TỪ ẢNH → CAD và chọn PNG RGB kích thước 110 × 40 pixel.
2. Ảnh mẫu có hai nét ngang: nét trái từ x=5 đến x=39, nét phải từ x=51 đến x=95, cùng tại y=17; khoảng cách giữa **tâm hai đầu mút** là 12 pixel.
3. Đặt ngưỡng `minLengthPixels=16` và `unitsPerPixel=10` sau khi hiệu chuẩn chiều dài thực. Bật **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**.
4. Chọn khoảng mở **từ 10 px đến 20 px**, nhấn **Nhận dạng và xem trước**.
5. Kiểm tra màn hình thông báo 01 khoảng hở có **12 px = 120 đơn vị CAD**; hai nét vẫn là LINE tách biệt.
6. Thay ngưỡng dưới thành 13 px rồi Preview lại: khoảng hở 12 px không được xuất hiện trong danh sách.
7. Khi dùng MCP với `origin=[100,200]`, điểm bắt đầu và kết thúc khoảng hở là `fromCAD=[495,425]` và `toCAD=[615,425]`. Giá trị này đã bao gồm phép đảo chiều trục Y của ảnh.
8. Nếu khoảng hở thật là cửa, phải kiểm tra ảnh gốc, bản thiết kế, kích thước ghi chú và thông tin kiến trúc trước khi dựng Block cửa.

### Bài 49 — Kiểm thử độ rộng và giới hạn an toàn

1. Trong thư mục `CADTBT360_V1.000` của mã nguồn đúng commit, chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_bounds_and_world_width_are_calibrated -- --nocapture`.
2. Bài test yêu cầu khoảng hở 12 pixel với tỷ lệ 10 cho `widthCAD=120`; khi `minOpeningGapPixels=13`, số ứng viên phải về 0.
3. Khi nhập ngưỡng 0, ngưỡng tối thiểu lớn hơn tối đa hoặc giá trị không phải số, chương trình phải từ chối và không ghi bản vẽ.
4. Chạy `cargo test --locked -p cadcraft --bin cadcraft ai_opening_gap_thresholds_are_editable_and_preview_only -- --nocapture` để kiểm tra GUI.
5. Chạy lại bài `ai_opening_gap_real_png_refuses_to_bridge_possible_door`: chế độ tìm khoảng mở không được bật đồng thời với `maxGapPixels>0`.
6. Kiểm tra chính Windows CI thuộc đúng SHA commit trước khi coi kiểm thử đã đạt; thử cả Setup và Portable bằng EXE thực tế, lấy ảnh chụp GUI có thông số khoảng hở.
7. Lưu DXF/DWG bản thử, đối chiếu bằng trình CAD độc lập; không nghiệm thu DWG chỉ dựa vào header hoặc thông báo CLI.

**Ranh giới chức năng:** Đây là một bước AI hỗ trợ phân tích và đo kích thước hình học từ ảnh, vẫn chưa tự nhận diện cửa, tường hoặc cột theo ngữ nghĩa. Chưa có xác nhận Windows CI PASS cho đợt 193.


## 34. Đợt 194 — Chống AI nhận nhầm khoảng mở khi có tường giao cắt

Đợt 194 tối ưu bộ phát hiện khoảng hở trong ảnh PNG/JPEG. Trước đây hai nét thẳng hàng cách nhau một khoảng phù hợp có thể được gợi ý là *khoảng mở*. Nhưng nếu **một nét thứ ba hoặc tường giao cắt đi xuyên qua đoạn trống**, thì đoạn này không còn là khoảng trống rõ ràng. Bộ lọc mới loại trường hợp đó trước khi trả gợi ý; không tự tạo cửa, tự vẽ đường nối hoặc xóa LINE.

### Bài 50 — Kiểm tra khoảng mở có tường giao cắt

1. Mở bản vẽ thử `BT50_AI_KhoangMoGiaoTuong.dxf`; không thao tác trực tiếp trên hồ sơ dự án.
2. Chuẩn bị ảnh nền trắng có hai nét ngang cùng tại y=20, nét trái từ x=5 đến 35, nét phải từ x=55 đến 95. Ban đầu khoảng cách giữa hai đầu nét là 20 pixel.
3. Bật **Gợi ý khoảng hở có thể là cửa (chỉ Preview)** và giữ `maxGapPixels=0`, đặt ngưỡng khoảng mở 3–40 pixel.
4. Preview khi chưa có nét đi qua khoảng trống: phần mềm có thể trả 01 khoảng hở *chưa xác minh*.
5. Thêm một nét đứng tại x=45, kéo từ y=4 đến y=38, xuyên qua đúng khoảng trống. Preview lại.
6. Xác minh khoảng mở 20 pixel không còn được gợi ý vì có hình học cắt qua. Các LINE nguồn vẫn tồn tại; không tạo BLOCK cửa hoặc tường BIM.
7. Kiểm tra tương tự với trường hợp hai nét đứng bị ngắt nhưng một nét ngang đi xuyên qua khoảng trống.
8. Chỉ khi đã đối chiếu bản vẽ thực tế mới sử dụng các ứng viên khoảng mở để kiểm tra vị trí cửa hoặc triển khai nghiệp vụ CAD tiếp theo.

**Cần đạt:** Không đánh dấu một khoảng trống là ứng viên cửa nếu nó đang bị đường ngang/dọc khác giao cắt. Đây là bộ lọc hình học bảo thủ, chưa xác định ngữ nghĩa cửa đi.

### Bài 51 — Chống gợi ý nhầm khi khoảng trống chứa nét thứ ba

1. Dùng hai nét ngang tại y=20: nét 1 kết thúc x=35, nét 2 bắt đầu x=55.
2. Thêm đoạn nằm cùng hàng từ x=40 đến x=50 ở giữa hai nét.
3. Nhấn Preview với **Gợi ý khoảng hở** đã bật; thuật toán không được coi toàn bộ đoạn x=35 đến x=55 là một khoảng mở trống 20 pixel.
4. Bộ lọc có thể đưa ra hai khoảng trống nhỏ x=35 đến x=40 và x=50 đến x=55; người dùng phải kiểm tra xem chúng là nét scan đứt hay cửa thực.
5. Không bật nối tự động `maxGapPixels>0` trong khi đang tìm khoảng mở; công cụ phải chặn cấu hình mâu thuẫn này.
6. Ghi lại số lượng ứng viên, ảnh đầu vào, thông số `unitsPerPixel`, và bản DXF kết quả. Không dùng hình mô phỏng thay cho ảnh GUI từ bản EXE nghiệm thu.

### Bài 52 — Kiểm thử Rust/CI và danh mục nghiệm thu

1. Mở `CADTBT360_V1.000` trong ZIP của **đúng commit** và chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_refuses_crossing_wall_or_third_fragment -- --nocapture`.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_vertical_crossing_is_not_a_clear_doorway -- --nocapture`.
3. Chạy lại `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_bounds_and_world_width_are_calibrated -- --nocapture` để bảo đảm không làm hỏng quy đổi chiều rộng sang CAD.
4. Chạy kiểm thử bảo vệ khoảng mở và Undo ở các bài 45–49, sau đó kiểm tra **Windows CI Success** trên cùng commit.
5. Cài Setup/Portable từ run thành công, thử chức năng trên GUI thực tế, lưu ảnh trước/sau và DXF/DWG để đối chiếu.
6. Nếu chưa có xác nhận chạy Windows CI hoặc kiểm tra EXE, chỉ ghi **Mã nguồn đã cập nhật — Chưa nghiệm thu**.

**Giới hạn:** Tính năng hiện tại xử lý nét ngang/dọc theo hình học; chưa phân loại được tường/cửa/cột bằng mô hình AI ngữ nghĩa. Không được suy luận một khoảng mở là cửa thực tế chỉ dựa trên số liệu pixel.


## 35. Đợt 195 — Không báo trùng khoảng mở khi ảnh scan có nhiều nét chồng

Ảnh scan thực tế có thể làm nét tường bị nhận dạng hai lần (nét nhòe, ảnh lớp trùng, nét LINE lặp). Mục tiêu đợt 195 là không trả **hai ứng viên cho cùng một vị trí khoảng mở** nhưng vẫn giữ các khoảng mở thật ở vị trí khác nhau. Dữ liệu nguồn `sourceEdges` vẫn được giữ cho ứng viên đầu tiên, và chỉ danh sách gợi ý được lọc trùng; không tự xóa nét CAD hoặc tạo BLOCK cửa.

### Bài 53 — Kiểm tra không đếm trùng một khoảng mở

1. Trong tab CAD tạo bản vẽ thử `BT53_AI_LocTrungKhoangMo.dxf`. Không chỉnh sửa hồ sơ dự án đang sản xuất.
2. Dựng ảnh thử có đoạn ngang trái từ (5.5,20.5) đến (35.5,20.5), và đoạn phải từ (47.5,20.5) đến (87.5,20.5).
3. Chèn mỗi đoạn thành **hai nét trùng tọa độ**, mô phỏng trường hợp công cụ vector hóa nhận trùng cùng một nét.
4. Bật **Gợi ý khoảng hở có thể là cửa (chỉ Preview)**; để `maxGapPixels=0` và ngưỡng khoảng mở 3–40 px.
5. Kết quả đúng là **một ứng viên khoảng hở 12 pixel**, không phải bốn ứng viên từ tổ hợp các nét trùng.
6. Xác minh các nét nguồn vẫn còn, bộ lọc chỉ loại gợi ý vị trí trùng. Đừng coi một ứng viên là một cửa thật trước khi kỹ sư xác minh trên hồ sơ.

### Bài 54 — Giữ đúng hai khoảng mở thật trên hai tuyến

1. Tạo cặp nét trái/phải theo bài 53 tại y=10.5, rồi tạo thêm một cặp giống vậy tại y=30.5.
2. Preview với `openingGapHints=true`; kết quả phải là **hai ứng viên** tại hai tuyến khác nhau.
3. Kiểm tra khoảng cách giữa hai tuyến lớn hơn dung sai lọc trùng; không được gom hai khoảng mở khác vị trí thành một.
4. Thử một nét thứ ba đi xuyên qua khoảng hở ở một tuyến; chỉ khoảng hở còn thông thoáng được gợi ý.
5. Chụp màn hình EXE thực tế, giữ ảnh đầu vào và đối chiếu DXF để nghiệm thu, không dùng ảnh minh họa thay bằng chứng.

### Bài 55 — Kiểm thử Rust và điều kiện phát hành

1. Mở thư mục `CADTBT360_V1.000` của ZIP mã nguồn theo đúng commit.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_duplicate_raster_strokes_do_not_duplicate_one_gap -- --nocapture`.
3. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_opening_gap_different_openings_are_not_deduplicated -- --nocapture`.
4. Chạy lại các test `ai_opening_gap_refuses_crossing_wall_or_third_fragment` và `ai_opening_gap_vertical_crossing_is_not_a_clear_doorway`.
5. Kiểm tra workflow Windows V1.025 chỉ đóng gói khi những bài test trên thực sự PASS. Sau khi có Setup và Portable, thử trên GUI thật, lưu ảnh màn hình và mở lại DXF/DWG trong phần mềm độc lập.
6. Nếu chưa có GitHub Actions Success hoặc kết quả EXE, ghi **Đã thêm mã nguồn/bài kiểm thử, chưa nghiệm thu**, không suy luận PASS.

**Giới hạn:** Chức năng mới là lọc dữ liệu gợi ý hình học từ các nét ảnh bị trùng, chưa phải AI phân loại ngữ nghĩa cửa đi, cửa sổ, tường, cột hoặc vật liệu. Các bản vẽ DWG tiếp tục cần thử nghiệm tương thích với AutoCAD hai chiều.


## 36. Đợt 196 — Kiểm tra Windows CI thật trước khi phát hành Setup và Portable

Mục tiêu quan trọng nhất của đợt này là chuyển từ việc **chỉ có mã nguồn, test và hướng dẫn** sang khả năng xác nhận một bộ cài Windows chạy ổn định. Các lần chạy cũ đã có lỗi thực tế. Không đánh dấu **PASS** cho phiên bản V1.025 khi chưa thấy kết quả GitHub Actions của đúng SHA.

### 36.1. Những lỗi đã đọc từ GitHub Actions

| Ngày, run | Lỗi đã ghi trong log | Trạng thái đối chiếu nguồn hiện tại |
|---|---|---|
| 04/11/2026, run 41744862778 | Test `zero_padded_hex_marker_is_considered_the_same_entity` của module QTO giao thông thất bại, giá trị `(0,1,1)` khác dự kiến | Run của nhánh/phát hành cũ; kiểm thử này không thấy trong `apps/cadcraft/src/takeoff.rs` trên nhánh V1.025 hiện tại, không thể kết luận lỗi còn tồn tại |
| 03/11/2026, run 41608879566 | Năm bài `bim_collab::tests` liên quan lưu và phục hồi workspace đều báo `Access is denied (os error 5)` | Nguồn hiện tại dùng quyền đọc/ghi khi đồng bộ bản backup; đã bổ sung gate kiểm thử Windows nhưng chưa có kết quả PASS |
| 30/10/2026, run 41064962499 | XREF test không biên dịch do gọi `EntityStore.first()` | Nguồn hiện tại dùng `model.iter().next()`; đã bổ sung kiểm thử toàn bộ `cadcraft-engine --test xref` trước khi đóng gói |

### Bài 56 — Kiểm thử XREF và BIM Workspace trên Windows

1. Mở nguồn tại SHA commit đợt 196, vào `CADTBT360_V1.000`.
2. Chạy `cargo test --locked -p cadcraft-engine --test xref -- --test-threads=2`; đây là các bài kiểm thử Xref thực, không chỉ test tên lệnh.
3. Chạy `cargo test --locked -p cadcraft-io --test it xref_roundtrip -- --nocapture`, sau đó `cargo test --locked -p cadcraft-io --test it large_dwg_v1025 -- --nocapture`.
4. Chạy `cargo test --locked -p cadcraft --bin cadcraft bim_collab::tests::safe_workspace_save_preserves_previous_file_as_backup -- --nocapture`.
5. Chạy tương tự bốn bài test BIM: `corrupted_workspace_recovers_previous_valid_backup`, `repeated_workspace_saves_replace_existing_main_and_backup`, `save_after_corrupted_main_does_not_destroy_last_valid_backup`, `workspace_recovery_does_not_discard_unsynced_in_memory_changes`.
6. Quan sát workflow `.github/workflows/cad360-v1025-release.yml`; bước **Verify Xref, DWG, and Windows BIM workspace safety before Setup** phải đạt Success trước bước tạo bộ cài.
7. Nếu một trong các bài test thất bại, đọc stacktrace và log tại chính job thất bại; không đổi tên hoặc bỏ qua test để ép có Setup.

### Bài 57 — Nghiệm thu đúng bộ cài EXE từ GitHub Actions

1. Vào repository `tbtdemo/tbt360_cad_2d` → Actions → **CAD 360 V1.025 - Windows Setup Portable**.
2. Chọn đúng nhánh `feature/CADTBT360_V1.025_xref-regression`. So sánh SHA trong Actions với SHA ZIP mã nguồn.
3. Chỉ tải Setup và Portable từ run có kết quả **Success**; không nhầm artifact chẩn đoán hoặc bản `UNVERIFIED_GUI_CANDIDATE` với bộ cài đã nghiệm thu.
4. Trên Windows sạch: chạy Setup, mở CAD 360, thử mở bản vẽ DXF và DWG, tạo LINE, PLINE, OFFSET và Undo.
5. Thử Xref: attach, reload, detach, save/mở lại. Bảo đảm file gốc tham chiếu không bị đổi khi chỉ lưu bản vẽ chủ.
6. Thử BIM Workspace: tạo workspace, lưu, lưu lại, phá nội dung file chính trên bản sao rồi kiểm tra khả năng khôi phục bản sao lưu.
7. Thử AI: PNG/JPEG → Preview → tạo LINE hoặc PLINE kín → Undo → save DXF/DWG. Dùng ảnh có khoảng hở giả lập cửa và ảnh có tường giao cắt để xác minh AI không tự nối.
8. Thử Portable từ cùng lượt run; ghi mã SHA, đường dẫn EXE, SHA256 của bộ cài, màn hình ứng dụng và kết quả nghiệm thu.
9. Muốn khẳng định DWG tương thích AutoCAD hai chiều phải thử mở/chỉnh sửa/lưu lại bằng AutoCAD độc lập, không chỉ kiểm tra header `AC1032`.

**Điều kiện kết luận:** Nguồn V1.025 hiện đã có thêm gate cho các lỗi Xref và BIM cũ, nhưng chưa có xác nhận Windows CI Success và GUI Acceptance. Giữ trạng thái **Chưa nghiệm thu EXE** cho tới khi có bằng chứng.


## 37. Đợt 197 — Bắt lỗi Windows Build bằng log riêng, không phát hành bản chưa qua kiểm thử

Đợt 197 tiếp tục ưu tiên **Setup/Portable thực sự chạy được** hơn là bổ sung thêm nút AI. Workflow `.github/workflows/cad360-v1025-release.yml` đã bổ sung hàm `Invoke-ReleaseGate` cho kiểm thử Xref, DWG và BIM Workspace. Từng nhóm kiểm thử được lưu vào log riêng trong `dist/v1025/diagnostics/` và có thể tải về qua artifact **CADTBT360_V1.025_Windows_x64_Failure_Diagnostics** khi một bước thất bại.

### Bài 58 — Phân tích log lỗi Windows chính xác

1. Vào GitHub repository `tbtdemo/tbt360_cad_2d` → **Actions** → `CAD 360 V1.025 - Windows Setup Portable`.
2. Tìm lượt chạy tương ứng SHA nguồn; **không lấy log của run cũ rồi kết luận là lỗi hiện tại**.
3. Khi bước `Verify Xref, DWG, and Windows BIM workspace safety before Setup` thất bại, mở trang run và tải artifact `CADTBT360_V1.025_Windows_x64_Failure_Diagnostics`.
4. Kiểm tra các file log `XREF_ENGINE_WINDOWS.log`, `XREF_IO_ROUNDTRIP.log`, `DWG_LARGE_V1025.log`, và các file `BIM_*.log` có sẵn trong artifact.
5. Nếu thấy `Access is denied (os error 5)` ở BIM, truy nguồn vào `save_workspace_safely`, thao tác `backup_temp`, `sync_all` và quy trình thay tệp trên Windows; không xóa bài kiểm thử.
6. Nếu Xref fail, kiểm tra đường dẫn file tham chiếu, test `--test xref`, vấn đề race giữa các test và phiên bản nguồn. Đợt 197 chuyển nhóm Xref sang `--test-threads=1` để tránh tranh chấp bộ hook I/O dùng chung.
7. Khi test thành công, trong GitHub Step Summary sẽ có dòng `XREF/DWG/BIM persisted-storage gates: all passed on Windows`. Dòng này chỉ xác nhận **nhóm kiểm thử tương ứng**, chưa đủ để coi toàn bộ Setup/Portable PASS.

### Bài 59 — Quy trình xác nhận Setup/Portable đã thực sự sẵn sàng

1. Kiểm tra toàn bộ lượt run được đánh dấu **Success**, bao gồm Rust preflight, AI regression, Xref/DWG/BIM regression, release build, Inno Setup, cài/gỡ và kiểm tra Portable.
2. Chỉ tải artifact Setup/Portable của **cùng SHA**; kiểm tra `BUILD_INFO.txt` và SHA256.
3. Cài trên Windows sạch, mở `CADTBT360.exe`; chạy lệnh CAD cơ bản, thử Layer/Block/Xref, thao tác Undo và nạp PNG/JPEG vào AI Preview.
4. Thử `CADTBT360_CLI.exe` tạo, đọc lại, sửa và lưu DXF/DWG; kiểm tra độc lập bằng AutoCAD đối với yêu cầu trao đổi DWG hai chiều.
5. Thử phần BIM Workspace lưu/khôi phục từ `.bak` và xác nhận không mất dữ liệu đang chỉnh sửa.
6. Thử AI chuyển ảnh thành LINE/PLINE, hiển thị tim tường/khoảng mở chưa xác minh, bảo vệ khoảng hở và hoàn tác nhóm đúng cách.
7. Lưu ảnh chụp GUI thật, biên bản từng ca nghiệm thu và mã SHA. Nếu chưa có bằng chứng này, ghi **Chưa nghiệm thu GUI**, ngay cả khi workflow Success.

**Trạng thái đợt 197:** Mã nguồn workflow đã sửa để tăng khả năng chẩn đoán và xác nhận hồi quy Xref/DWG/BIM. Chưa có bằng chứng GitHub Actions Success mới, chưa có Setup/Portable được xác nhận chạy trên Windows sạch. Không gọi bản nguồn này là bản EXE đã hoàn thiện.


## 38. Đợt 198 — Sửa lỗi Rust E0425 trong bài kiểm thử hướng dẫn

**Bài 60 — Kiểm tra import trong module `#[cfg(test)]`**

1. Mở `CADTBT360_V1.000/apps/cadcraft/src/main.rs`, tìm bài test `complete_user_manual_is_found_in_setup_and_portable_paths`.
2. Bản trước có câu lệnh `let paths = full_user_guide_candidates(root);`, khiến biên dịch test thất bại `E0425` vì hàm ở module cha không nằm trong phạm vi hiện tại.
3. Bản sửa: `let paths = super::full_user_guide_candidates(root);`. Giữ các assertion kiểm tra vị trí giáo trình trong thư mục Setup/Portable.
4. Chạy tại `CADTBT360_V1.000`: `cargo check --locked -p cadcraft --tests`. Sau đó chạy `cargo test --locked -p cadcraft --bin cadcraft complete_user_manual_is_found_in_setup_and_portable_paths -- --nocapture`.
5. Đối chiếu GitHub run 42031101855 (SHA 32ec8c80) để nhận diện lỗi gốc. Bản sửa đã được đẩy ở commit `7ada49a7832974154ab700f644bedaa398eee3eb`. Việc đẩy mã không đồng nghĩa Windows test đã PASS.

## 39. Đợt 199 — Bắt lỗi biên dịch bài test từ Preflight Windows

**Bài 61 — Đọc và khắc phục lỗi Cargo test-target trước khi đóng gói**

1. Workflow: `.github/workflows/cad360-v1025-release.yml`, nhánh `feature/CADTBT360_V1.025_xref-regression`.
2. Ở Preflight, sau khi `cargo check --locked -p cadcraft --bin cadcraft` thành công, chạy thêm `cargo check --locked -p cadcraft --tests`.
3. Kết quả được ghi riêng vào `dist/v1025/diagnostics/CARGO_CHECK_GUI_TESTS.log`; nếu mã kiểm thử không biên dịch, workflow phải dừng ngay với thông báo chỉ tới tệp log.
4. Khi có lỗi `E0425`, đọc đường dẫn và số dòng, kiểm tra `use super::...` hoặc tham chiếu `super::function_name(...)`. Không vô hiệu hóa bài kiểm thử để qua CI.
5. Khi các bài test biên dịch được, tiếp tục kiểm tra AI ảnh→CAD, DWG/Xref, BIM Workspace và bộ cài; Preflight PASS chỉ chứng minh biên dịch mã, **không** chứng minh GUI, AI hay DWG đã nghiệm thu.
6. Mỗi lượt phát hành phải giữ nguyên số phiên bản V1.025 cho đến khi có quyết định tăng phiên bản; số đợt 199 là số đợt công việc, không phải phiên bản sản phẩm mới.

**Trạng thái nghiệm thu đợt 199:** Đã cập nhật mã kiểm tra CI. Chưa ghi nhận Windows Setup/Portable và AI GUI PASS trước khi có kết quả đầy đủ trên đúng SHA và kiểm thử Windows độc lập.


## 40. Đợt 200 — Xác nhận giáo trình trong Setup trùng bản nguồn phát hành

### Bài 62 — Đối chiếu tài liệu đã cài với tài liệu tạo cùng SHA

1. Mở workflow `.github/workflows/cad360-v1025-release.yml` của nhánh V1.025 và xác định bước `Smoke test installed application, uninstall and binary identity`.
2. Sau khi chạy Inno Setup ở Windows, kiểm tra tệp giáo trình tổng hợp tại `HuongDan/GIAO_TRINH_TONG_HOP_CAD360.docx`, giáo trình AI tại `HuongDan/HUONG_DAN_AI_TU_ANH_THANH_CAD.docx` và hướng dẫn Markdown AI.
3. Dùng `Get-FileHash -Algorithm SHA256` so sánh các file đã cài với tài liệu tạo trong `dist/v1025/` cùng lượt build; không nhận một tệp Word cũ dù tệp còn mở được.
4. Chạy `python scripts/validate_word_guide.py` trên **chính giáo trình tổng hợp vừa cài**; nếu nội dung hoặc bảng biểu bị thiếu, không cho quy trình phát hành vượt qua smoke test.
5. Kiểm tra workflow commit `e4e795aa09914cc796e29cc854be3e0a2c0c2c70`; thay đổi này chỉ bổ sung chốt kiểm thử, chưa chứng minh GUI/AI/DWG đã PASS.

### Bài 63 — Kiểm soát thống nhất Setup và Portable

1. Dùng artifact đúng SHA nguồn để so sánh `BUILD_INFO.txt`, giá trị SHA256 của ứng dụng và các tệp giáo trình.
2. Bản Portable có bài kiểm tra riêng cho giáo trình tổng hợp; cả Setup và Portable phải có giáo trình CAD toàn phần, không được chỉ kèm phần AI.
3. Thực hành kiểm tra GUI, Xref, đo bóc, IFC, đường nét từ ảnh và DWG trong phần mềm đã cài; chỉ nghiệm thu nếu tất cả kiểm thử thực tế đạt.
4. Đối với DWG, phải kiểm tra mở–chỉnh sửa–lưu bằng AutoCAD độc lập. Kiểm tra header `AC1032` không đủ để xác nhận tương thích hai chiều.

**Trạng thái đợt 200:** Đã sửa workflow kiểm tra toàn vẹn giáo trình sau khi cài Setup. Chưa khẳng định Windows CI PASS, bản Setup/Portable chạy ổn định hoặc AI tự dựng toàn bộ tường, cột, cửa. Giữ nguyên số phiên bản V1.025 đến khi quyết định phát hành mới.


## 41. Đợt 201 — Kiểm tra giáo trình Portable như bản cài Setup

### Bài 64 — Xác thực giáo trình Word sau giải nén Portable

1. Mở `.github/workflows/cad360-v1025-release.yml` trên nhánh V1.025; tìm bước `Verify portable archive, create SHA256 checksums`.
2. Giải nén `CADTBT360_V1.025_Portable_Windows_x64.zip` vào thư mục thử nghiệm và xác nhận tệp `GIAO_TRINH_TONG_HOP_CAD360.docx` có tồn tại.
3. Kiểm tra SHA256 của Word trong Portable với `dist/v1025/CADTBT360_V1.025_GIAO_TRINH_TONG_HOP.docx` vừa dựng từ đúng commit.
4. Chạy `python scripts/validate_word_guide.py $fullGuidePortable` trên tài liệu **sau giải nén**, không chỉ kiểm tra `ZipFile.testzip()` và XML hợp lệ.
5. Nếu giáo trình thiếu phần CAD 2D, QTO, BIM, IFC, Xref, AI, MCP, bài hướng dẫn hoặc bảng, bộ xác thực phải từ chối và dừng phát hành.
6. Xem commit `a1df508e060c2d7d37089ffbb58d70e1abdcd4e0` và nhật ký run GitHub tương ứng để biết kết quả; mã workflow được cập nhật chưa có nghĩa đã nghiệm thu bộ cài.

### Bài 65 — Phân biệt bốn mức xác nhận

- **Mức 1 — Source:** các thay đổi đã có trên GitHub; không đủ khẳng định sản phẩm hoạt động.
- **Mức 2 — CI:** toàn bộ job Windows kết luận Success trên đúng SHA; bảo đảm các bước CI thực sự đã chạy.
- **Mức 3 — Setup/Portable:** thử giải nén/cài/gỡ, các tệp và giáo trình khớp nguồn, các lệnh kiểm thử bắt buộc thành công.
- **Mức 4 — Người dùng:** kiểm tra GUI trên Windows sạch, đối tượng AI chỉnh sửa được, DWG mở/sửa/lưu hai chiều qua AutoCAD độc lập và các chức năng CAD/QTO/BIM thực tế.

**Trạng thái đợt 201:** Workflow đã bổ sung kiểm tra cấu trúc giáo trình Word trong Portable. Chưa xác nhận đạt Mức 2–4 khi không có bằng chứng Windows CI Success và nghiệm thu người dùng.


## 42. Đợt 202 — Sửa ba lỗi E0599 của bài kiểm thử Xref trên Windows

### Bài 66 — Sửa API `Vec<Handle>` và `EntityStore` đúng kiểu Rust

1. Mở run 42034172599 và bước `Verify Xref, DWG, and Windows BIM workspace safety before Setup`; xem log `XREF_ENGINE_WINDOWS.log`.
2. Lỗi thứ nhất: `crates/engine/tests/xref.rs:107` gọi `model.handles().next()`, nhưng `handles()` trả về `Vec<Handle>`, không phải iterator. Dùng `model.handles().into_iter().next()`.
3. Lỗi thứ hai và ba: `loaded.entities.first()` và `reloaded.entities.first()` không có trên `EntityStore`. Dùng `loaded.entities.iter().next()` và `reloaded.entities.iter().next()`.
4. Giữ nguyên assertion của các ca `xclip_rectangular_boundary_is_stored_on_xref_reference` và `xref_reload_preserves_bim_xdata_repairs_member_identity_and_invalidates_root_revision`; tuyệt đối không bỏ kiểm thử để có build xanh.
5. Bản sửa nằm tại commit `0fb28a9146a11ca4bbacf4436be3ce7281327119`.
6. Chạy `cargo check --locked -p cadcraft-engine --test xref` rồi `cargo test --locked -p cadcraft-engine --test xref -- --test-threads=1 --nocapture` trên Windows. Chỉ đánh dấu PASS nếu thấy kết quả thật trên đúng mã nguồn.

### Bài 67 — Phát hiện lỗi Xref ngay từ Preflight

1. Mở `.github/workflows/cad360-v1025-release.yml` tại nhánh V1.025.
2. Sau `cargo check --locked -p cadcraft --tests`, chạy thêm `cargo check --locked -p cadcraft-engine --test xref`.
3. Khi có lỗi biên dịch tích hợp Xref, log `dist/v1025/diagnostics/CARGO_CHECK_XREF_TEST.log` phải được ghi và bước Preflight dừng ngay; không đợi đến vòng thử AI và đóng gói Setup.
4. Chạy tiếp các bài kiểm thử Xref/DWG/BIM chỉ khi đã qua chốt biên dịch. Bản bổ sung workflow nằm tại commit `4ea7dce1ec7813074fb1eb10b0c71ed59de02c0c`.
5. Sau đó đối chiếu cùng commit giữa Setup, Portable, giáo trình, SHA256SUMS và hình ảnh nghiệm thu Windows. Không coi kiểm tra biên dịch là chứng minh tương thích AutoCAD hai chiều.

**Trạng thái đợt 202:** Đã sửa ba vị trí gây lỗi biên dịch Xref đã được chứng minh trong run 42034172599 và bổ sung gate test-target sớm. Chưa có bằng chứng toàn bộ Windows CI thành công, Setup EXE hoạt động hoặc AI dựng hoàn chỉnh tường, cửa, cột từ ảnh thực tế.


## 43. Đợt 203 — Lưu log khi bài kiểm thử Windows không được phát hiện

### Bài 68 — Kiểm tra nguồn lỗi ở bước `Verify required named regression tests exist before running them`

1. Mở `.github/workflows/cad360-v1025-release.yml` trên nhánh `feature/CADTBT360_V1.025_xref-regression`.
2. Bước `discover` dùng `cargo test --locked ... -- --list` để bảo đảm bài kiểm thử có tồn tại và biên dịch được; việc tìm thấy test **không** chứng minh test đã PASS.
3. Khi Cargo trả mã lỗi hoặc không phát hiện dòng `: test`, workflow tạo `dist/v1025/diagnostics/DISCOVERY_<package>_<target>_<test>.log` (tên được làm sạch ký tự đặc biệt). Log giữ nguyên thông báo Cargo để phân biệt lỗi compile, sai tên bài test và lỗi phụ thuộc.
4. Mở artifact `CADTBT360_V1.025_Windows_x64_Failure_Diagnostics` của **đúng run/SHA**, đọc tệp `DISCOVERY_*.log`, sửa lỗi ở nguồn rồi chạy lại. Không bỏ `--list` hoặc giảm số bài test bắt buộc để có build xanh.
5. Khi tất cả bài test được phát hiện, CI tiếp tục chạy kiểm thử AI, Xref/DWG, BIM, đóng gói và cài/gỡ. Kiểm tra trạng thái của từng bước để biết test nào đã chạy thực sự.
6. Bản bổ sung này bắt đầu ở commit `9a339a7b18e357dd45b9a77feee7d3755b0f0d41`. Trạng thái CI chưa được công bố PASS khi chưa có bằng chứng run thành công.

### Bài 69 — Phân biệt lỗi phát hiện, lỗi kiểm thử và lỗi nghiệm thu GUI

1. **Discovery**: chỉ xác nhận tên test có mặt và binary kiểm thử biên dịch được. Nếu lỗi, xem `DISCOVERY_*.log`.
2. **Regression**: bài test thực thi và kiểm tra các khẳng định, bao gồm ảnh→CAD, Xref/DWG và BIM. Nếu lỗi, xem tên test và các artifact chẩn đoán tương ứng.
3. **Packaging**: Windows tạo Setup và Portable từ cùng SHA; kiểm tra `BUILD_INFO.txt`, SHA256 và giáo trình toàn bộ phần mềm.
4. **GUI Acceptance**: người dùng kiểm tra CAD Free không bắt buộc đăng nhập; thử Layer/Block/Xref, QTO, giao thông, IFC, AI trên ảnh thực tế và thao tác Undo.
5. **DWG**: kiểm thử trao đổi hai chiều với AutoCAD độc lập trước khi tuyên bố tương thích hoàn toàn.

**Trạng thái đợt 203:** Đã bổ sung lưu log chẩn đoán cho bước khám phá bài kiểm thử. Chưa có xác nhận Setup EXE, Portable hoặc AI dựng kiến trúc hoàn chỉnh đã nghiệm thu.


## 44. Đợt 204 — AI ảnh→CAD từ chối cấu hình nối khoảng hở sai kiểu

### Bài 70 — Kiểm soát tham số `maxGapPixels` trong AI Preview và MCP

1. Mở `CADTBT360_V1.000/crates/mcp/src/raster_vector.rs`, tìm hàm `preview`.
2. Mặc định không khai báo `maxGapPixels` sẽ không nối khoảng hở (giá trị 0). Khai báo số nguyên từ 0 đến 12 pixel để giới hạn độ dài được phép nối khi người dùng chủ động yêu cầu.
3. Trước đây `as_u64().unwrap_or(0)` nhận giá trị sai kiểu như `-1`, `2.5`, `"4"`, `null` hoặc `true` rồi âm thầm đổi thành 0. Hiện tại mọi giá trị được khai báo sai kiểu đều trả lỗi rõ ràng, không làm thay đổi CAD.
4. Khi `openingGapHints=true`, vẫn bắt buộc `maxGapPixels=0`, bảo toàn khoảng hở nghi là cửa và không tự tạo Block cửa.
5. Đầu vào hợp lệ `maxGapPixels=0` vẫn hoạt động như cũ; không khai báo tham số cũng mặc định là 0. Với số lớn hơn 12, hệ thống tiếp tục từ chối vì có thể làm kín khoảng mở thật.

### Bài 71 — Kiểm thử tự động ngăn dữ liệu sai làm thay đổi hành vi AI

1. Tạo ảnh PNG trắng 64×32 trong test `raster_vector_rejects_malformed_gap_bridging_options_before_cad_creation`.
2. Lần lượt gửi `-1`, `2.5`, `"4"`, `null`, `true` và xác nhận `preview` trả lỗi chứa `maxGapPixels must be an integer`.
3. Gửi `maxGapPixels=0` để chứng minh hành vi mặc định hợp lệ vẫn được chấp nhận.
4. Chạy `cargo test --locked -p cadcraft-mcp --lib raster_vector_rejects_malformed_gap_bridging_options_before_cad_creation -- --nocapture` trong thư mục `CADTBT360_V1.000`.
5. Windows CI có thêm bước phát hiện tên test và bắt buộc chạy test trước khi đóng gói. Cần kiểm tra log đúng SHA, không đồng nhất "đã viết test" với "test đã PASS".

**Trạng thái đợt 204:** Đã sửa xử lý tham số và bổ sung kiểm thử tự động trong mã nguồn/CI. Đây là xác thực tham số vector hóa, **không phải** chức năng AI nhận dạng ngữ nghĩa tường, cửa hoặc cột hoàn chỉnh. Setup EXE và tương thích DWG AutoCAD hai chiều vẫn cần nghiệm thu thực tế.


## 45. Đợt 205 — Chống sai tỷ lệ CAD khi AI đọc ảnh PNG/JPEG

### Bài 72 — Kiểm soát đơn vị CAD trên mỗi pixel

1. Trong công cụ AI raster-to-CAD hoặc MCP, đặt `unitsPerPixel` theo tỷ lệ hiệu chuẩn bản vẽ. Ví dụ `0.5` tạo một nửa đơn vị CAD cho mỗi pixel; `2.0` tạo hai đơn vị CAD cho mỗi pixel.
2. Nếu không khai báo tham số, hệ thống vẫn mặc định `1.0` để duy trì tương thích cũ. Trước khi Commit phải xác nhận tỷ lệ dự kiến.
3. Nếu khai báo `"2"` (chuỗi), `null`, `false`, `0` hoặc số âm, AI từ chối với thông báo `unitsPerPixel`, không tự biến thành `1.0` vì có thể tạo các đối tượng CAD sai kích thước.
4. Nhận dạng khoảng mở, kích thước theo đơn vị CAD và kết quả LINE/PLINE đều phải dùng cùng hệ số hiệu chuẩn hợp lệ; kiểm tra kích thước thực tế trước khi lưu DWG.

### Bài 73 — Kiểm thử dữ liệu tỷ lệ sai trước khi tạo CAD

1. Tạo PNG thử nghiệm 64×32, dùng bài test `raster_vector_rejects_invalid_explicit_cad_scale_before_geometry` trong `crates/mcp/src/raster_vector.rs`.
2. Thử `"2"`, `null`, `false`, `0` và `-1`: tất cả phải bị từ chối trước khi tạo bản vẽ.
3. Thử số hợp lệ `0.5`, `2.0` và xác nhận AI Preview tiếp tục hoạt động.
4. Chạy `cargo test --locked -p cadcraft-mcp --lib raster_vector_rejects_invalid_explicit_cad_scale_before_geometry -- --nocapture`.
5. Windows CI đã bổ sung phát hiện tên test và chạy bắt buộc. Chỉ kết luận PASS khi GitHub Actions của đúng SHA thành công.

**Trạng thái đợt 205:** Đã cập nhật kiểm tra tham số hiệu chuẩn ảnh→CAD cùng test hồi quy và workflow Windows. Chưa xác nhận khả năng dựng tường/cửa/cột theo ngữ nghĩa, GUI chạy thực tế hoặc tương thích DWG AutoCAD hai chiều.


## 46. Đợt 206 — Giữ đúng phiên bản BIM Xdata khi Reload Xref

### Bài 74 — Phân biệt tạo bản sao CAD và nạp phần tử tham chiếu BIM

1. Mở tệp `CADTBT360_V1.000/crates/engine/src/cmd/blocks/xref/load.rs`, tìm đoạn `adopt` trong hàm `import`.
2. Trước đợt 206, Xref gọi `refresh_tbt360_bim_copy_identity`, làm lại `revision=1` ngay cả khi phần tử gốc trong `site.dxf` có `revision=7` hoặc `revision=9`.
3. Cơ chế mới `rebind_xref_bim_identity` chỉ đổi `entity=<handle mới>`, giữ nguyên `revision`, `floor`, `discipline` và `owner`; đồng thời không làm thay đổi dữ liệu Xdata của ứng dụng khác.
4. Quy tắc CAD COPY thông thường không đổi: một bản sao CAD mới có thể bắt đầu revision riêng. Không sử dụng quy tắc COPY cho Xref vì tham chiếu BIM phải phản ánh đúng trạng thái nguồn.
5. Sau khi Reload Xref, kiểm tra phần tử thuộc bản tham chiếu mang revision mới của nguồn, không mặc định trở lại 1. Revision của bản vẽ chủ chỉ được cập nhật theo quy tắc thay đổi định nghĩa Block.
6. Mã nguồn sửa được đẩy ở commit `a3b16bd2e04582c6a9c21162601ba136a14ecab7`.

### Bài 75 — Kiểm thử dữ liệu BIM khi nạp và cập nhật Xref

1. Mở bài kiểm thử `xref_reload_preserves_bim_xdata_repairs_member_identity_and_invalidates_root_revision` trong `crates/engine/tests/xref.rs`.
2. Tạo file `site.dxf` có phần tử mang `floor=2;discipline=ELEC;owner=u1;revision=7`. Mở file chủ có Xref trỏ tới file đó và kiểm tra phần tử giữ revision 7 sau khi nạp.
3. Sửa file nguồn với `revision=9`, Reload Xref, đối chiếu revision 9, mã `entity` khớp handle mới và bản vẽ chủ được tăng revision theo quy tắc có sẵn.
4. Chạy hai bài test đơn vị `xref_bim_identity_rebind_preserves_source_revision_and_other_apps` và `xref_bim_identity_rebind_adds_missing_entity_without_resetting_revision`, để xác nhận dữ liệu Xdata của ứng dụng khác và trường hợp thiếu entity không bị thay đổi sai.
5. Windows Preflight chạy `cargo test --locked -p cadcraft-engine --test xref xref_reload_preserves_bim_xdata_repairs_member_identity_and_invalidates_root_revision -- --test-threads=1 --nocapture`, lưu kết quả ở `dist/v1025/diagnostics/XREF_BIM_REVISION_PREFLIGHT.log`.
6. Khi test thành công, tiếp tục toàn bộ kiểm thử Xref, DWG, BIM, AI, cài/gỡ Setup và Portable; chỉ kết luận đạt khi đúng lượt CI của bản nguồn đó thành công.

**Trạng thái đợt 206:** Đã sửa cơ chế Xref bảo toàn revision và bổ sung unit test/CI preflight. Chưa có kết quả Windows CI PASS của đúng SHA mới, chưa nghiệm thu bộ cài EXE hoặc chức năng AI dựng kiến trúc hoàn chỉnh từ ảnh.


## 47. Đợt 207 — AI đối chiếu hai mép tường để gợi ý khoảng mở và vẽ đường bao CAD

### Bài 76 — Từ ảnh PNG/JPG nhận biết khoảng mở xuyên hai mép tường

1. Chuẩn bị ảnh mặt bằng rõ nét có hai đường tường song song, cả hai bị ngắt cùng vị trí tại một khoảng mở. Đây là đầu vào kiểm chứng hình học, **chưa phải** mô hình học sâu nhận dạng cửa.
2. Trong AI Raster/MCP chọn ảnh, hiệu chuẩn `unitsPerPixel`, bật `openingGapHints=true` và `pairedWallOpeningHints=true`; giữ `maxGapPixels=0` để không tự lấp lỗ mở.
3. Chạy Preview để xem `openingGapCandidates` theo từng mép, sau đó `pairedWallOpeningCandidates` chỉ khi hai khoảng hở thẳng hàng và cách nhau theo chiều dày tường hợp lệ (hiện 2–40 pixel).
4. Kiểm tra `outlinePixels`, `outlineCAD`, `wallThicknessPixels`, `wallThicknessCAD`, `sourceGapIndices`. Kết quả `paired-through-wall-gap-unverified` chỉ là ứng viên và cần người dùng xác nhận.
5. Nếu chỉ có một mép đứt, hai khoảng hở lệch vị trí hoặc có nhiều mép có thể ghép thì thuật toán từ chối tự suy ra cửa. Không tạo cửa đi, cửa sổ hay Block kiến trúc từ giả thuyết chưa đủ chứng cứ.

### Bài 77 — Cho AI tạo PLINE hướng dẫn trong CAD sau xác nhận

1. Mở bản vẽ CAD làm việc; bảo đảm không còn lệnh đang chạy, chế độ Undo là All và không có nhóm Undo khác.
2. Sau khi xem Preview và xác nhận vị trí, bật cả ba tùy chọn: `openingGapHints=true`, `pairedWallOpeningHints=true`, `createOpeningGuidePolylines=true`. Đặt `confirmOpeningGuides=true` để đồng ý tạo hình học hướng dẫn.
3. Chuyển `preview=false` để Commit. AI tạo các LINE nét gốc và thêm PLINE khép kín bao quanh khoảng mở đã đối chiếu, nằm trong một nhóm Undo; có thể chọn, sửa và xóa như đối tượng CAD thông thường.
4. Đường bao này chỉ để kiểm tra, không phải Block cửa hay đối tượng BIM; không được tự dùng làm kết quả đo bóc kiến trúc khi chưa nghiệm thu ngữ nghĩa.
5. Thực hành Undo để hoàn tác cả nhóm, mở lại bản vẽ và kiểm tra kích thước theo đơn vị CAD. Việc lưu DWG chuẩn và trao đổi AutoCAD cần kiểm thử độc lập.

### Bài 78 — Kiểm thử ảnh PNG thật và trường hợp không đủ chứng cứ

1. Trong `CADTBT360_V1.000/crates/mcp/src/raster_vector.rs` có kiểm thử `ai_real_png_paired_wall_opening_preview_and_editable_guide`: ảnh PNG tự dựng có bốn đoạn tường tạo hai mép ngắt đồng vị trí.
2. Preview phải xác nhận hai khoảng hở đơn, một khoảng mở ghép đôi, bảo toàn bốn LINE gốc; `wallThicknessCAD` phải đúng theo tỷ lệ hiệu chuẩn.
3. Khi chưa có `confirmOpeningGuides=true`, yêu cầu tạo guide phải bị từ chối. Sau xác nhận, Headless CAD phải có bốn LINE và một PLINE chỉnh sửa được.
4. Kiểm thử `ai_single_or_ambiguous_opening_gap_must_not_invent_a_door` đảm bảo không suy ra cửa từ một khoảng hở đơn lẻ hoặc tổ hợp mơ hồ.
5. Windows CI bắt buộc phát hiện và chạy các kiểm thử AI này trước bước đóng gói. Chỉ coi thành công khi đúng run/SHA xác nhận PASS.

**Giới hạn được công bố:** Đã bổ sung thuật toán hình học và vẽ đường bao có xác nhận, nhưng chưa đạt AI nhận diện ngữ nghĩa cửa/cột/tường trên tập ảnh thực tế đa dạng. Không thay thế nghiệm thu ngoài thực tế bằng kết quả test tổng hợp.


## 48. Đợt 208 — AI tìm dấu hiệu nét cánh cửa trên ảnh mặt bằng, không suy diễn cửa giả

### Bài 79 — Phát hiện hình học nét giống cánh cửa đi

1. Mở cửa sổ **AI từ ảnh → CAD**, chọn PNG/JPG mặt bằng có hai mép tường song song bị ngắt tại cùng vị trí. Hiệu chuẩn `unitsPerPixel` theo kích thước đã biết.
2. Bật `openingGapHints=true` và `pairedWallOpeningHints=true`, giữ `maxGapPixels=0`. Bật tùy chọn mới **Gợi ý nét cánh cửa tại góc khoảng mở (chỉ Preview)** tương ứng tham số `doorLeafHints=true`.
3. Nhấn **Nhận dạng và xem trước**. AI xem các LINE đã vector hóa, tìm một đoạn vuông góc gắn với đúng góc khoảng mở, kéo dài ra ngoài bề dày tường và có độ dài tương đối phù hợp với bề rộng khoảng mở.
4. Kết quả trả về `doorLeafCandidateCount` và trường `doorLeafEvidence` của `pairedWallOpeningCandidates`. Thông tin kèm theo gồm `sourceLineIndex`, `hingePixels`, `leafLengthPixels`; phân loại `possible-door-leaf-stroke-unverified`.
5. Bản chất kết quả chỉ là dấu hiệu hình học trên ảnh, **không phải nhận dạng chắc chắn cửa đi**. Một đoạn jamb nối hai mép tường hoặc các nét trùng/nhập nhằng không được dùng làm bằng chứng cửa.
6. Chưa có thao tác nào tự tạo Block cửa, ký hiệu cửa, cung quay cánh hoặc đối tượng BIM. Nếu muốn tạo đường bao tham chiếu PLINE của khoảng mở, phải bật các tùy chọn xác nhận của Bài 77.

### Bài 80 — Kiểm thử PNG và trường hợp không đủ bằng chứng

1. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_door_leaf_evidence_requires_a_perpendicular_outward_stroke_at_opening_corner -- --nocapture` để kiểm tra cách đối chiếu góc bản lề, bề dày tường và chiều dài nét.
2. Kiểm tra ba tình huống: một nét vuông góc có bằng chứng → một ứng viên; chỉ có jamb ngắn qua chiều dày tường → không có ứng viên; hai nét giống cánh cửa cạnh tranh → báo mơ hồ và không chọn.
3. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_real_png_door_leaf_hint_is_preview_only_and_preserves_native_geometry -- --nocapture` với PNG tổng hợp có bốn đoạn tường và một nét cánh cửa vuông góc.
4. Khi Preview, `editableEntitiesCreated` phải bằng 0, số guide mặc định bằng 0. Tắt `doorLeafHints` thì số ứng viên nét cánh cửa phải trở về 0; bốn mép nguồn vẫn được bảo toàn.
5. Windows CI có chốt phát hiện và chạy hai bài test mới trước đóng gói. Chỉ xác nhận đạt nếu bài test thực sự PASS trên đúng SHA; không coi commit GitHub là bằng chứng nghiệm thu.
6. Tiếp tục kiểm thử ảnh chụp thực tế bị nhiễu, tường nhiều lớp, cửa mở quay, cửa lùa; mô hình hiện tại là nhận dạng hình học quy tắc, chưa phải AI thị giác có đào tạo nhận thức kiến trúc.

**Trạng thái đợt 208:** Đã bổ sung nhận biết hình học nét giống cánh cửa, dữ liệu Preview, điều khiển trong GUI và unit/regression tests. Chưa xác nhận Windows CI PASS, chưa tự tạo Block cửa đi, chưa hoàn thành AI hiểu và vẽ kiến trúc từ ảnh thực tế.


## 49. Đợt 209 — AI vẽ cung quay cửa ARC trực tiếp trong CAD sau xác nhận

### Bài 81 — Vẽ hình học cung quay cửa từ bằng chứng ảnh

1. Mở CAD 360, khởi tạo bản vẽ hoặc bản sao để kiểm tra; chọn PNG/JPEG mặt bằng và hiệu chuẩn `unitsPerPixel`.
2. Bật **Gợi ý khoảng hở**, **Đối chiếu khoảng mở hai mép tường**, **Gợi ý nét cánh cửa**. Giữ `maxGapPixels=0` để không tự lấp khoảng mở.
3. Bật tùy chọn **Tạo cung quay ARC cho nét cửa đã nhận dạng** và tích **Tôi xác nhận các cung quay ARC này sau khi xem trước**. Tương ứng MCP: `createDoorSwingArcs=true`, `confirmDoorSwingArcs=true`, `doorLeafHints=true`, `pairedWallOpeningHints=true`.
4. Chạy **Nhận dạng và xem trước**: các cung đề xuất xuất hiện màu tím trong hình xem trước, số lượng được thống kê ở `doorSwingArcsProposed`. Kết quả chứa `center`, `radius`, `start`, `end` theo tọa độ và độ góc CAD; chiều dài quy đổi dùng đúng tỷ lệ `unitsPerPixel`.
5. AI chỉ tạo ARC nếu phát hiện một nét giống cánh cửa duy nhất gắn đúng góc khoảng mở, chiều dài phù hợp chiều rộng và hai phương gần vuông góc. Nếu thiếu mép tường, nhiều nét mơ hồ hoặc chưa xác nhận thì **không tạo cung**.
6. Nhấn **Tạo LINE / PLINE chỉnh sửa trong CAD** để ghi hình học vào bản vẽ. Lúc này nét LINE gốc và cung ARC được tạo dưới một nhóm Undo; có thể chọn, chỉnh sửa, xóa như đối tượng CAD. **Không tự tạo Block cửa/BIM**, chưa tự xác định loại cửa đi, cửa sổ hay quy tắc kiến trúc.
7. Kiểm tra tâm bản lề, bán kính, góc quay và tỷ lệ bằng lệnh đo; kiểm thử Undo/Redo, mở lại và thử trao đổi DWG chuẩn với AutoCAD độc lập trước nghiệm thu.

### Bài 82 — Kiểm thử tự động từ PNG thành ARC CAD và chính sách xác nhận

1. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_confirmed_door_swing_arc_is_editable_and_refuses_missing_confirmation -- --nocapture`: ca thử xác nhận hình học cung có tâm, bán kính và góc đúng; không tạo khi thiếu nét hoặc có nhiều nét cạnh tranh.
2. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_real_png_door_arc_commit_creates_native_arc_after_confirmation -- --nocapture`: PNG thử có hai mép tường hở và một nét tương ứng cánh cửa; không xác nhận sẽ báo lỗi; khi xác nhận thì tạo thêm ARC CAD cùng các LINE nguồn.
3. Chạy kiểm thử GUI `ai_door_swing_arc_gui_requires_explicit_two_step_confirmation` để bảo đảm giao diện mặc định chưa bật hai công tắc và tham số Preview luôn ở trạng thái chưa sửa bản vẽ.
4. Windows CI bắt buộc phát hiện và chạy các bài test trên trước đóng gói; xem đúng commit SHA và kết quả run để biết trạng thái thực tế.
5. Cung quay ARC là hình học dựng theo quy tắc từ ảnh, chưa phải nhận dạng cửa bằng mô hình học sâu. Kiểm thử trên ảnh thực tế bị nhiễu, nhiều cửa cạnh nhau, ảnh nghiêng và trùng nét vẫn là công việc bắt buộc.

**Trạng thái đợt 209:** Đã bổ sung thuật toán tạo ARC, tích hợp trực tiếp vào cửa sổ AI và các ca test PNG/GUI. Chưa xác minh Windows CI PASS và chưa tuyên bố AI nhận dạng kiến trúc hoàn chỉnh.


## 50. Đợt 210 — AI gộp cánh LINE và cung quay ARC thành BLOCK CAD có thể chỉnh sửa

### Bài 83 — Dựng một Block cửa dạng bản nháp từ ảnh mặt bằng

1. Mở CAD 360, vào **AI từ ảnh → CAD**, chọn PNG/JPG có khoảng mở hai mép tường và dấu hiệu cánh cửa. Đặt tỷ lệ `unitsPerPixel` đã hiệu chuẩn.
2. Bật lần lượt `openingGapHints`, `pairedWallOpeningHints`, `doorLeafHints`, `createDoorSwingArcs` và xác nhận `confirmDoorSwingArcs`. Giữ `maxGapPixels=0`.
3. Bật tùy chọn mới **Gộp cánh LINE + cung ARC thành Block CAD bản nháp**, sau đó tích **Xác nhận tạo Block CAD bản nháp, giữ lại nét ảnh gốc**. Các tham số tương ứng là `createDoorBlocks=true` và `confirmDoorBlocks=true`.
4. Nhấn Preview. Đọc `doorBlockDraftCount` và `doorBlockDrafts`; mỗi bản nháp chứa `base` (tọa độ bản lề), `leaf` (LINE cánh cửa CAD) và `arc` (ARC cung quay). Mặc định không có Block nào được tạo khi chưa xác nhận.
5. Nhấn **Tạo LINE / PLINE chỉnh sửa trong CAD**. Với mỗi ứng viên đã xác nhận, GUI tạo LINE cánh cửa mới và ARC mới; chuyển **chỉ hai đối tượng mới** thành một `BLOCK` với `keep=convert`, kết quả là `INSERT` có thể chọn, di chuyển, sao chép và chỉnh sửa theo lệnh Block thông thường. Mã tên gợi ý: `TBT_AI_DOOR_<documentUid>_<revision>_<index>`.
6. Các LINE gốc truy xuất từ ảnh vẫn được giữ độc lập để đối chiếu. Block này là **bản nháp hình học**, chưa nhận dạng chắc chắn loại cửa, vật liệu, hệ cửa hoặc thuộc tính BIM.
7. Sau Commit, thử chọn Block INSERT, dùng Properties, các thao tác Block, Undo/Redo; kiểm tra bằng bản vẽ thử trước khi Save DWG và trao đổi AutoCAD.

### Bài 84 — An toàn giao dịch, Undo và giới hạn MCP

1. Bài test `ai_native_door_draft_block_contains_line_and_arc_and_can_undo_as_one_step` tạo một LINE và một ARC bằng engine CAD thật, sau đó chuyển thành Block với `keep=convert`.
2. Kiểm tra bản vẽ còn một INSERT thay cho hai đối tượng mới, định nghĩa Block chứa đúng hai thành phần và một lần Undo phải loại bỏ cả INSERT lẫn định nghĩa Block.
3. Bài test `ai_door_block_drafts_require_review_and_keep_traced_lines_separate` kiểm tra PNG, bộ khóa xác nhận và danh sách bản nháp. Dữ liệu nguồn phải còn nguyên sau Preview.
4. **Phạm vi hiện tại:** Chỉ GUI desktop được phép Commit Block; MCP chỉ Preview Block. Nếu gửi `createDoorBlocks=true` và `preview=false` qua MCP, công cụ từ chối rõ ràng thay vì tự động thực hiện một chuỗi thay đổi thiếu cơ chế xác nhận và hoàn tác an toàn.
5. Windows CI bổ sung chốt kiểm thử tạo Block/Undo và kiểm tra an toàn Preview. Chưa có xác nhận PASS toàn bộ Windows CI hay kết quả kiểm thử trong AutoCAD độc lập.
6. Khi xem kết quả từ ảnh thật, người dùng phải xác minh bản lề, hướng mở, chiều rộng và bề dày tường. AI hiện chưa tạo Block cửa động có thông số BIM/kiến trúc hoàn chỉnh.

**Trạng thái đợt 210:** Đã bổ sung mã tạo `BLOCK/INSERT` từ LINE+ARC có xác nhận trong GUI, Preview Block trong shared AI và kiểm thử Undo/CI. Chưa thể coi là hoàn thành AI kiến trúc hoặc Setup EXE đã nghiệm thu khi chưa có kết quả test thực tế thành công.


## 51. Đợt 211 — AI hiệu chỉnh chiều rộng cửa trước khi dựng Block CAD

### Bài 85 — Nhập chiều rộng cửa thủ công, giữ nguyên nét ảnh gốc

1. Trong **AI từ ảnh → CAD**, chọn ảnh mặt bằng, xác lập `unitsPerPixel`, bật nhận diện khoảng mở hai mép tường, nét giống cánh cửa, tạo cung ARC và hai xác nhận cần thiết.
2. Bật **Gộp cánh LINE + cung ARC thành Block CAD bản nháp**, xác nhận tạo Block, sau đó bật **Đặt chiều rộng cửa thủ công (CAD units)**.
3. Nhập chiều rộng đã kiểm chứng, ví dụ `850` theo đơn vị của bản vẽ; kiểm tra đơn vị CAD là mm hay đơn vị khác trước khi đặt kích thước. Giá trị được gửi qua `doorWidthOverrideCAD`.
4. Nhấn Preview lại. Bộ dựng Block thay đổi **chỉ LINE cánh và bán kính ARC của Block mới**, giữ nguyên `base` bản lề, hướng quay và tất cả LINE trích xuất từ ảnh. Xem `widthCAD`, `sourceOpeningWidthCAD`, `widthOverridden` trong `doorBlockDrafts`.
5. Nếu không bật điều chỉnh chiều rộng, GUI không gửi `doorWidthOverrideCAD`; AI sử dụng chiều rộng suy ra từ ảnh như trước. Đây là cải tiến quan trọng để không gửi `null` cho đầu vào kiểm tra kiểu chặt chẽ.
6. Hệ thống từ chối chiều rộng không phải số, không hữu hạn, không dương hoặc ngoài `0.000001..1000000000` CAD units. Việc dùng độ rộng khác khoảng mở ảnh có thể tạo hình học không khớp tường: người dùng phải xác nhận vị trí trước khi Commit.
7. Nhấn Commit để tạo Block CAD bản nháp dạng INSERT, thử đo kích thước cánh và cung quay. Thực hiện Undo nếu hình học không chính xác; kiểm thử độc lập việc lưu DWG và trao đổi AutoCAD.

### Bài 86 — Kiểm thử bảo toàn nét nguồn và kiểm tra cài đặt GUI

1. Kiểm thử MCP Preview `ai_reviewed_door_width_changes_only_new_block_leaf_and_arc_not_image_strokes` tạo PNG mẫu, so sánh Block trước/sau khi đặt `doorWidthOverrideCAD=30` và xác minh `widthCAD=30`, `sourceOpeningWidthCAD=24`.
2. Đối chiếu `lines` và `openingGapCandidates` giữa hai kết quả: phải giống hệt nhau vì không được phép kéo giãn nét ảnh nguồn.
3. Thử `"30"`, `null`, `0`, `-1`, giá trị quá lớn: tất cả phải trả lỗi trước khi thay đổi CAD.
4. Kiểm thử desktop `ai_door_width_override_gui_requires_block_opt_in_and_defaults_to_unmodified_draft`: mặc định không gửi tham số hiệu chỉnh, chỉ gửi sau khi bật tạo Block và hiệu chỉnh.
5. Các bài kiểm thử được gắn vào Windows CI; cần xem trạng thái PASS của đúng commit trước khi khẳng định dùng được trong EXE.
6. Block ở đây vẫn là bản nháp CAD sau khi người dùng xác nhận, chưa phải đối tượng cửa kiến trúc thông minh hoặc nhận dạng ngữ nghĩa bằng học sâu.

**Trạng thái đợt 211:** Đã bổ sung hiệu chỉnh chiều rộng Block cửa cùng kiểm thử bảo toàn nét gốc và GUI. Chưa xác nhận CI Windows PASS, chưa hoàn thiện chỉnh chiều bản lề/hướng mở bằng AI.


## 52. Đợt 212 — AI hiệu chỉnh từng cửa riêng biệt trước khi vẽ BLOCK CAD

### Bài 87 — Chọn cửa số 1, số 2… và thay đổi chiều rộng riêng từng cửa

1. Mở **AI từ ảnh → CAD** và ảnh mặt bằng có từ hai cửa trở lên. Hiệu chuẩn `unitsPerPixel` và kích hoạt gợi ý khoảng mở, nhận dạng nét cánh, tạo ARC, tạo Block; xác nhận các công tắc an toàn như trong Bài 83–86.
2. Nhấn **Nhận dạng và xem trước**. Khi có các `doorBlockDrafts` độc lập, giao diện liệt kê **Chọn cửa 1**, **Chọn cửa 2**… kèm `widthCAD` và `sourceOpeningWidthCAD`.
3. Chọn đúng cửa cần thay đổi. Ở khung **Hiệu chỉnh riêng từng cửa trong danh sách AI**, nhập chiều rộng mới theo đơn vị bản vẽ rồi bấm **Áp dụng cho cửa đã chọn**. Ví dụ, cửa 1 giữ 800 mm, cửa 2 chỉnh thành 900 mm; hai cửa phải được xử lý độc lập.
4. Nhấn Preview lại để đối chiếu danh sách Block. Cửa được chọn có `widthOverridden=true`, bán kính ARC và chiều dài LINE cánh được đổi theo chiều rộng mới. Các cửa không chọn tiếp tục dùng kích thước suy ra từ ảnh hoặc kích thước chung nếu đã bật.
5. Để trả lại kích thước nhận dạng, chọn cửa tương ứng và bấm **Xóa hiệu chỉnh cửa này**, sau đó Preview lại.
6. Người dùng phải đối chiếu lại thứ tự cửa sau mỗi lần thay đổi nguồn ảnh, vùng nhận dạng hoặc tham số có thể làm thứ tự ứng viên thay đổi; phiên bản hiện tại dùng chỉ số cửa, chưa có ID hình học bất biến xuyên mọi lần nhận dạng.
7. Chỉ Commit sau khi đã kiểm tra từng cửa trong Preview. Block vẫn là bản nháp CAD được xác nhận thủ công, chưa phải đối tượng cửa kiến trúc thông minh.

### Bài 88 — Kiểm thử độc lập hai cửa từ PNG và an toàn cấu hình cũ

1. Tham số MCP/GUI mới: `doorWidthOverridesCAD` là object gồm chỉ số **bắt đầu từ 0** và chiều rộng CAD, ví dụ `{"1":900.0}` điều chỉnh cửa số 2 trên giao diện. GUI hiển thị số bắt đầu từ 1.
2. Giữ tương thích với `doorWidthOverrideCAD` của đợt 211: nếu cả chiều rộng chung và riêng tồn tại, chiều rộng riêng của mỗi cửa được ưu tiên; các cửa còn lại nhận chiều rộng chung.
3. Chạy `cargo test --locked -p cadcraft-mcp --lib ai_two_png_doors_have_independent_widths_and_validate_stale_review -- --nocapture` để thử ảnh PNG có hai cửa. Kiểm tra cửa 1 giữ rộng 24 CAD units, cửa 2 đổi 36, toàn bộ `lines` từ ảnh không thay đổi.
4. Thử chỉ số ngoài danh sách, số âm, khóa không đúng định dạng, null, chiều rộng âm hoặc sai kiểu: Preview phải từ chối trước khi ghi CAD, không tự áp dụng vào cửa khác.
5. Chạy kiểm thử giao diện `ai_individual_door_width_override_gui_serializes_selected_indices_only` để xác nhận GUI không gửi hiệu chỉnh khi chưa chọn tạo Block.
6. Kiểm tra đúng SHA của Windows CI. Các bài test là bằng chứng chỉ khi chạy PASS; kết quả chưa thay thế nghiệm thu EXE, ảnh thực tế nhiều cửa hoặc DWG mở/lưu hai chiều trong AutoCAD.

**Trạng thái đợt 212:** Đã bổ sung kiểm soát chiều rộng riêng từng cửa trong thuật toán, giao diện lựa chọn từ Preview và bài test hai cửa PNG. Chưa có xác nhận CI Windows PASS, chưa tự nhận dạng hướng mở/bản lề mọi kiểu cửa hoặc Block động kiến trúc hoàn chỉnh.
