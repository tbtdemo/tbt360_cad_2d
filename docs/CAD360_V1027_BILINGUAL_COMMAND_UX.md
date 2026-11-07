# CAD 360 Việt Nam V1.027 — Lệnh song ngữ và hướng dẫn thao tác tức thời

## 1. Phạm vi bản đầu

Đã triển khai trong nhánh V1.027, chưa đồng nghĩa đã biên dịch và nghiệm thu:
- Nhập câu lệnh tiếng Việt **có dấu hoặc không dấu**, khớp chính xác từ điển lệnh nội bộ đã ánh xạ đến engine. Không phải chatbot hiểu mọi câu tự nhiên.
- Tìm lệnh theo gõ dở câu tiếng Việt, gợi ý hiển thị nhãn Việt + mã lệnh CAD, nhấn chọn để chạy. Alias AutoCAD thông dụng **L, PL, C, REC, DI...** phải được ưu tiên và không thay đổi.
- Khi chạy lệnh tương tác, dòng lệnh hiển thị **ĐANG CHỜ** nhập điểm, số, văn bản hoặc chọn đối tượng từ chính metadata `Prompt.accept`, cùng ví dụ riêng cho các lệnh hay dùng.
- Esc hủy lệnh, Enter chấp nhận mặc định/kết thúc theo giai đoạn, Undo/Redo vẫn do engine quản lý. Không tự ghi bản vẽ từ một gợi ý tìm kiếm.

## 2. Hướng dẫn thao tác thử trên CAD 360

| Gõ vào dòng lệnh | Lệnh engine | Người mới cần làm gì tiếp |
|---|---|---|
| làm sạch bản vẽ / lam sach ban ve | PURGE | Xem các mục không dùng, chọn loại cần dọn và xác nhận trong lệnh |
| vẽ đường thẳng / ve duong thang | LINE / L | Nhấp điểm đầu, nhấp điểm cuối, Enter để kết thúc |
| vẽ đa đoạn / ve da doan | PLINE / PL | Chọn các đỉnh liên tiếp, Enter để kết thúc |
| vẽ đường tròn / ve duong tron | CIRCLE / C | Chọn tâm, nhập bán kính và Enter |
| vẽ hình chữ nhật / ve hinh chu nhat | RECTANG / REC | Chọn góc thứ nhất, góc đối diện |
| đo khoảng cách / do khoang cach | DIST / DI | Chọn điểm đầu và điểm thứ hai, xem kết quả trong lịch sử lệnh |
| tìm tọa độ / tim toa do | ID | Nhấp một điểm, đọc tọa độ kết quả |
| đo diện tích / do dien tich | AREA | Chọn hình hoặc lần lượt chọn các đỉnh theo prompt |
| ghi kích thước / ghi kich thuoc | DIMLINEAR | Chọn hai điểm đo và vị trí đặt đường kích thước |
| ghi kích thước xiên / ghi kich thuoc xien | DIMALIGNED / DAL | Chọn hai điểm và vị trí đặt kích thước |
| di chuyển đối tượng | MOVE / M | Chọn đối tượng, điểm gốc, điểm đích |
| sao chép đối tượng | COPY / CO | Chọn đối tượng, điểm gốc, điểm bản sao |
| cắt nét | TRIM / TR | Chọn cạnh cắt và nét cần cắt theo hướng dẫn |
| kéo dài nét | EXTEND / EX | Chọn biên và đối tượng cần kéo dài |
| kiểm tra lỗi bản vẽ | AUDIT | Thực hiện rà soát bản vẽ theo các xác nhận trong engine |

**Cách sử dụng:** Nhập đúng cụm tiếng Việt, có thể không dấu. Khi gõ tới một phần cụm như `lam sach`, các gợi ý tương ứng được hiển thị. Sau khi nhập **đầy đủ** cụm, nhấn Enter để chạy lệnh tương ứng. Nhấp một dòng gợi ý thì chạy mã lệnh CAD đã được xác nhận có tồn tại.

## 3. Cần kiểm tra trong Windows CI / EXE thật

- [ ] Biên dịch `cadcraft-ui-egui` và GUI Windows; không chỉ xác nhận nguồn tĩnh.
- [ ] Alias `L, PL, C, REC, DI` nhận đúng khi Space/Enter.
- [ ] Nhập tiếng Việt có dấu qua IME Windows, gõ không dấu và chọn gợi ý bằng chuột hoặc Tab.
- [ ] Lệnh tương tác hiện gợi ý **ĐANG CHỜ**, không biến mất giữa chừng khi nhấn chuột.
- [ ] Kiểm tra OSNAP End/Mid/Center/Intersection, bắt điểm thực tế cho đường vẽ và kích thước.
- [ ] Kiểm tra trường hợp lệnh có bước chọn tập đối tượng, số, điểm, tọa độ tương đối.
- [ ] Nghiệm thu UI tiếng Việt ở Ribbon, menu, Properties, Settings, OSNAP và toàn bộ danh mục lệnh, không chỉ nhóm ví dụ.
- [ ] Chụp ảnh EXE thật và bổ sung vào tài liệu hướng dẫn đi kèm.
- [ ] Tích hợp bảng trợ giúp AI Review và AI Studio thành hướng dẫn thống nhất; phần này chưa hoàn tất.

## 4. Giới hạn cần nêu rõ

Đợt này là nền tảng **tìm/nhập lệnh song ngữ + hướng dẫn trạng thái** chứ chưa phải hoàn thiện AI hiểu mọi câu tiếng Việt, tự xử lý mọi bản vẽ kiến trúc hoặc đã dịch 100% UI. Không được coi V1.027 là sẵn sàng phát hành trước CI và kiểm thử thao tác trên Windows.

## 5. Tiến độ và định hướng

- Đã có commit cho từ điển lệnh, gợi ý, nhập câu nhiều từ và hiển thị trạng thái tương tác.
- Chưa có số đo độc lập về tỉ lệ dịch toàn bộ giao diện; không công bố phần trăm thành công thiếu kiểm chứng.
- Tiếp theo: dữ liệu song ngữ có mô tả, quy trình, ví dụ cho **mọi CommandSpec**; hướng dẫn theo từng trạng thái `Prompt`, trạng thái chạy/đợi/có lỗi; đo bằng bộ UI audit và test qua Windows.

## 6. Bộ trợ giúp OSNAP trong Settings và AI Studio

Bản V1.027 bổ sung chú giải khi rê chuột lên từng tùy chọn **14 chế độ OSNAP** (Endpoint, Midpoint, Center, Geometric Center, Node, Quadrant, Intersection, Extension, Insertion, Perpendicular, Tangent, Nearest, Apparent Intersection, Parallel). Trong hộp **Drafting Settings** có mục mở rộng **HƯỚNG DẪN OSNAP / OBJECT SNAP — VÍ DỤ**, chỉ cụ thể cách bật F3, tạo đường từ END đến MID, dùng DIST/ID kiểm tra số đo và phân biệt F9 (Snap Grid) với F3 (OSNAP).

**Tình huống học:**
1. Mở bản vẽ có hai đoạn LINE; bật F3 và chọn Endpoint, Midpoint.
2. Gõ `L` → chọn đầu mút LINE thứ nhất bằng chuột (END) → chọn trung điểm LINE thứ hai (MID) → Enter.
3. Gõ `DI` → chọn lại hai điểm bằng OSNAP để đọc khoảng cách; gõ `ID` → nhấp điểm để đọc tọa độ.
4. Nếu không hiển thị/không bắt đúng điểm, kiểm tra chế độ OSNAP, APERTURE, zoom, và loại đối tượng; việc thêm văn bản trợ giúp **không phải** bằng chứng OSNAP đã đạt kiểm thử thực tế.

## 7. Hướng dẫn trực tiếp trong AI Studio

Tab **04 Kết nối & Hướng dẫn** nay có các khối hướng dẫn: bắt đầu từ ảnh mặt bằng không chứa UI/nhiễu, nhập câu tiếng Việt để tìm lệnh CAD, phân biệt AI Studio/AI Review, và thao tác OSNAP. Các hướng dẫn mô tả đúng giới hạn hiện tại (bộ dò raster hiện chưa đọc ngữ nghĩa hoàn chỉnh và nhánh AI Review chưa tự đồng bộ với hình học AI Studio).

**Chưa hoàn tất:** dịch toàn bộ Ribbon/menu/Properties và tất cả lỗi engine; còn cần đo tỷ lệ bao phủ và nghiệm thu ảnh Windows GUI thực tế. Không gọi trạng thái này là V1.027 đã phát hành.

## 8. Nguyên tắc khóa V1.027 — bảo vệ 100% lệnh CAD nguyên bản

**Không sửa engine CAD để thêm lớp dịch.** Lệnh gốc, alias PGP, cú pháp tọa độ tuyệt đối/tương đối, keyword, Enter, Space, Esc, Undo, script, LISP và các lệnh transparent vẫn qua đường thực thi cũ. Bản Việt hóa chỉ nằm ở giao diện dòng lệnh, gợi ý và thông tin bước đang chờ. Từ điển Việt là tầng phụ trợ, ánh xạ đến ID lệnh đã được `find_command` xác nhận có thật.

- Danh mục tìm kiếm không còn chỉ dựa vào các lệnh mẫu: các lệnh công khai trong `cadcraft_engine::command_specs()` có thể được tìm bằng mã lệnh đầy đủ. Test `every_public_native_command_remains_searchable_by_full_id` duyệt toàn bộ registry động thay vì kiểm 10–20 ví dụ.
- Alias đã đăng ký có độ dài ngắn nhất được tìm từ **chính `CommandSpec.aliases`**, không tự đặt phím tắt. Cụm `L`, `PL`, `C`, `REC`, `CO`, `DI` không được mất cách dùng Space/Enter ban đầu.
- Lời nhắc engine và tên lệnh hiển thị theo dạng `English (Tiếng Việt)`; văn bản đầu vào keyword gửi về engine **vẫn nguyên bản**.
- Khi không còn lệnh chạy, gợi ý lệnh tiếp theo từ các cặp thao tác thực sự đã xuất hiện trong lịch sử **phiên hiện tại**, sau đó mới đến các đề xuất ngữ cảnh có lệnh thật trong registry. Không tự chạy, không truyền dữ liệu ra máy chủ, không tuyên bố đây là mô hình AI học liên tục.
- Không thêm lệnh không tồn tại vào GUI chỉ để làm đẹp menu. Không xem `CommandSpec` đã đăng ký là bằng chứng thao tác chuột, OSNAP, Undo hay DWG đã chạy đúng.

### Mẫu hiển thị thống nhất

| Trạng thái | Mẫu hướng dẫn |
|---|---|
| Tìm lệnh | `L — LINE (Vẽ đường thẳng)`; `PL — PLINE (Vẽ đa đoạn)`; `DI — DIST (Đo khoảng cách)` |
| Đang vẽ | `LINE / L (Vẽ đường thẳng) — Specify next point (Chọn điểm tiếp theo)` |
| Chờ thao tác | `ĐANG CHỜ: chọn điểm bằng chuột hoặc nhập X,Y` |
| Lệnh kết thúc | `NEXT (Lệnh tiếp): DI (DIST · Đo khoảng cách) · L (LINE · Vẽ đường thẳng) · Enter (Lặp lệnh trước)` |
| Hủy | `ESC (Cancel / Hủy lệnh)` |
| Đo đạc | `DI / DIST (Đo khoảng cách) → hai điểm`; `ID (Tọa độ điểm) → chọn điểm` |
| Sửa bản vẽ | `PURGE (Làm sạch bản vẽ)`; người dùng phải xem và xác nhận kết quả |

### Hai phạm vi “100%” phải kiểm toán độc lập

1. **100% lệnh nguyên bản của chính CAD 360:** bảo toàn và kiểm thử toàn bộ lệnh CAD đã đăng ký (không thay đổi engine), aliases, trạng thái tương tác, kết quả ghi và hoàn tác; mã nguồn có test registry, nhưng **Windows CI và thao tác thật chưa đạt nghiệm thu**.
2. **100% lệnh tương ứng AutoCAD 2027:** đây là mục tiêu riêng; phải đối chiếu từng command và system variable với tài liệu chính thức của Autodesk AutoCAD 2027, phân loại **Có/Đầy đủ**, **Có/Một phần**, **Chưa có**, **Không áp dụng**, cùng test hành vi. Không được công bố 100% tương đương AutoCAD 2027 chỉ nhờ registry nội bộ. Autodesk còn có các lệnh riêng như `GEOMETRYCLEANUPOPEN`, `BDETECT`, `BSEARCH` và `CHECKOUT` cần đánh giá từng khả năng tương đương/giới hạn. Tham chiếu: https://help.autodesk.com/view/ACD/2027/ENU/

**Trạng thái nghiệm thu V1.027: CHƯA ĐẠT**. Phần tìm kiếm song ngữ và gợi ý theo lịch sử mới có mã nguồn, chờ test Windows; không gộp thành tỉ lệ hoàn thiện toàn bộ AutoCAD 2027.

## 9. Nâng cấp đợt tiếp — gợi ý có thể nhấp và keyword song ngữ

- [x] Windows V1.027 **đã PASS** trên commit `3b9fe408`, bao gồm `cargo check` engine + GUI + desktop, discover named tests và unit tests command/alias; run: https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/42343109856.
- [x] Sau khi chạy lệnh, dòng `NEXT (Lệnh tiếp)` hiển thị **gợi ý có thể nhấp**. Lệnh đề xuất lấy từ lịch sử chuỗi thao tác trong phiên hiện tại, rồi mới dùng các nhóm lệnh liên quan. Từng gợi ý kiểm tra có `CommandSpec` thật, hiển thị alias ngắn đã đăng ký, và chỉ khởi động khi người dùng chủ động nhấp.
- [x] Keyword của prompt như `Undo (Hoàn tác)`, `Close (Đóng)` được trình bày song ngữ; giao diện gửi **keyword tiếng Anh nguyên bản** để bảo vệ hành vi của lệnh cũ, không sửa matcher hay parser.
- [x] Bổ sung regression đảm bảo gợi ý có phím tắt native thật và không chứa lệnh không tồn tại; regression keyword chứng minh hình thức song ngữ không làm đổi keyword engine. Cổng CI V1.027 chạy các test này trên Windows.
- [ ] **Chưa có Windows PASS** của commit mới nhất chứa gợi ý nhấp/keyword; cần kiểm tra thao tác chuột thật và ảnh EXE, bao gồm Esc/Enter/F3/OSNAP, tránh UI tooltip đè vùng bản vẽ.
- [ ] Yêu cầu 100% lệnh AutoCAD 2027 chưa được chứng minh; cần ma trận so sánh với danh mục lệnh chính thức, không được đồng nhất `CommandSpec` nội bộ với danh mục Autodesk.
