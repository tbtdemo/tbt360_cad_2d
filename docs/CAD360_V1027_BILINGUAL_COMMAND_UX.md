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

## 10. Mọi lệnh CAD công khai đều có hướng dẫn nền theo metadata thực tế

- [x] Windows V1.027 trên commit `d081e360` PASS: https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/42345051483. Bản mới nhất cần kiểm thử lại.
- [x] `registered_active_command_help` tra `CommandSpec` của engine để hiện mã đầy đủ, alias ngắn được đăng ký thật, tên lệnh tiếng Anh và chú giải tiếng Việt (nếu có). Những lệnh chưa viết ví dụ riêng vẫn có chỉ dẫn tổng quát thay vì hiện gợi ý trống hoặc đánh lừa là đã hỗ trợ mọi chức năng AutoCAD.
- [x] `active_prompt_help` suy ra `WAITING (Đang chờ)` từ chính `Prompt.accept`: chọn đối tượng, điểm, số, văn bản, hoặc danh sách keyword; chỉ hiển thị Enter khi prompt thật cho phép. Không sửa dữ liệu lệnh, keyword hay xử lý tọa độ của engine.
- [x] Thêm test `every_registered_command_has_english_first_in_app_guide`, duyệt tất cả lệnh công khai trong registry và yêu cầu hướng dẫn luôn có mã lệnh gốc, không rỗng. Test nằm trong Windows Gate V1.027.
- [ ] Chưa đồng nghĩa mọi lệnh đều có bài hướng dẫn có ảnh, dữ liệu mẫu và kết quả thực nghiệm; điều này cần đánh giá từng lệnh.
- [ ] Phạm vi so sánh Autodesk AutoCAD 2027 vẫn là nhiệm vụ riêng: danh mục lệnh, biến hệ thống, phím tắt và workflow cần kiểm kê, chạy thử rồi báo sai khác theo từng nhóm.

**Chú ý:** Chỉ gọi đây là tầng trợ giúp metadata cho toàn bộ *lệnh CAD 360 có đăng ký*. Không phải khẳng định CAD 360 có hoặc hỗ trợ hoàn chỉnh 100% lệnh AutoCAD 2027.

## 11. Nâng cấp lớn: COMMAND HANDBOOK (Sổ tay lệnh) V1.027

Thay cửa sổ Command Reference cũ (chỉ danh sách ID, tên, alias) thành tra cứu + hướng dẫn hai cột, không chỉnh engine. Mở bằng **F1** hoặc Help → Command Reference.

- **Tìm Anh/Việt** có dấu/không dấu, lệnh đầy đủ, alias PGP, nhãn bản dịch, menu gốc và cụm ngôn ngữ tự nhiên đã được ánh xạ an toàn (`làm sạch bản vẽ` → `PURGE`, `vẽ đường thẳng` → `LINE`).
- **Cột trái:** hiển thị các mã lệnh thật kèm phím viết tắt nếu registry có, hỗ trợ chọn từng lệnh mà chưa sửa bản vẽ.
- **Cột phải:** hiển thị đúng ID gốc, tên tiếng Anh/Việt, tất cả alias, shortcut phím, nhóm menu, lệnh tương tác hay trực tiếp, tham số API nếu có.
- **Ví dụ thực hành cụ thể** cho nhóm lệnh thông dụng (`LINE`, `PLINE`, `CIRCLE`, `RECTANG`, `ARC`, `DIST`, `ID`, `AREA`, `DIMLINEAR`, `DIMALIGNED`, `TRIM`, `EXTEND`, `COPY`, `MOVE`, `OFFSET`, `PURGE`, `AUDIT`, `LAYER`, `ZOOM`). Các lệnh chưa có ví dụ riêng vẫn có **STEP 1/2/3** dựa trên metadata thật + prompt engine.
- **RUN (Thực hiện):** chỉ gọi `CadApp::start` gốc khi người dùng chủ động nhấn và lệnh đang được phép chạy; chọn dòng tra cứu không tự ghi đối tượng CAD.
- **Đo bao phủ thực tế:** test `audit_native_cad_command_handbook_coverage` in số lệnh công khai trong registry, số có bản dịch, alias, menu, giao diện tương tác, ví dụ riêng và số thiếu bài hướng dẫn. Không công bố 100% bản dịch khi chưa có số liệu test.

### Cổng nghiệm thu
- [x] Đã thêm test `entire_public_command_registry_remains_searchable_in_handbook`: kiểm tra toàn bộ ID và tất cả alias đăng ký trong registry.
- [x] Đã thêm test `vietnamese_search_and_examples_keep_native_identifiers`: tiếng Việt có dấu/không dấu, các cụm đầy đủ như `làm sạch bản vẽ` và ví dụ lệnh.
- [x] Đã thêm test thống kê bao phủ ngôn ngữ/hướng dẫn.
- [ ] Windows CI mới phải PASS. Cần ảnh chụp Windows EXE thật, thử gõ tiếng Việt qua bộ gõ IME, mở F1, nhấp/thoát, đối chiếu OSNAP và Undo.
- [ ] Ma trận tương đương mọi lệnh AutoCAD 2027 chưa đạt; không dùng số lệnh CAD 360 làm chứng cứ tương đương Autodesk.

## 12. Hệ thống lọc và xếp hạng Sổ tay lệnh — V1.027

- [x] Bộ lọc tùy chọn **VI TRANSLATED (Đã dịch)**, **EXAMPLES (Có ví dụ)**, **INTERACTIVE (Tương tác)**. Mặc định không chọn bất kỳ bộ lọc nào để hiển thị **toàn bộ CommandSpec công khai**; khi bật bộ lọc chỉ ẩn trên màn hình, không xóa/khóa lệnh.
- [x] Hiển thị công khai số lệnh có bản dịch tiếng Việt, có bài hướng dẫn riêng và tổng số lệnh đã đăng ký, thay vì tuyên bố 100% nhưng không chứng minh.
- [x] Xếp hạng kết quả: alias/mã lệnh **trùng chính xác** đứng trước ID bắt đầu bằng chuỗi tìm kiếm; tiếp đến cụm tiếng Việt tương ứng và nhãn Anh/Việt; tìm kiếm gần đúng cuối cùng. Quy tắc không thay đổi parser, không sửa PGP, không tự thực thi lệnh.
- [x] Test regression duyệt registry không bị lọc khi dùng mặc định và xác nhận **L→LINE, PL→PLINE, C→CIRCLE, REC→RECTANG, DI→DIST** được ưu tiên hạng nhất, đồng thời bảo toàn tìm `làm sạch bản vẽ` có dấu/không dấu.
- [ ] Chạy Windows CI trên commit mới nhất. Kiểm thử thực tế UI với tiếng Việt/IME, đánh dấu bộ lọc, mở F1, nhấp RUN và xem prompt. Chưa thể coi là kiểm chứng chức năng 100% AutoCAD 2027.

**Kỷ luật phát hành:** Sổ tay song ngữ chỉ được coi là đạt nghiệm thu khi kiểm thử Windows và thao tác trên EXE thực tế thành công; các kiểm thử unit chứng minh phạm vi hẹp theo đăng ký/metadata.

## 13. Gợi ý theo vùng chọn và mở rộng vốn từ tiếng Việt

- [x] `selection_aware_next_commands`: khi có đối tượng được chọn, ưu tiên các thao tác gốc đã đăng ký như MOVE, COPY, ROTATE, LIST, ERASE, PROPERTIES. Khi không chọn gì, ưu tiên LINE, PLINE, CIRCLE, RECTANG, DIST, ID. Lịch sử thao tác vẫn được xét; chỉ các `CommandSpec` thật mới được gợi ý. **Không tự thực thi khi click chọn đối tượng**.
- [x] Bổ sung hơn 50 cụm tìm lệnh tiếng Việt cho OFFSET, FILLET, CHAMFER, MIRROR, SCALE, STRETCH, HATCH, DIMRADIUS, DIMDIAMETER, DIMANGULAR, SAVEAS, PURGE/RECOVER, PLOT, LAYER, BLOCK/INSERT, OSNAP/DSETTINGS, FIND và nhiều công cụ khác. Tất cả được kiểm chứng `find_command` trước khi hiển thị; nếu engine chưa có lệnh, từ khóa không tạo được lệnh ảo.
- [x] Kiểm thử `selection_context_prioritizes_only_real_native_edit_commands` và `expanded_vietnamese_command_vocabulary_resolves_only_existing_engine_commands`; đưa vào Windows CI gate.
- [ ] Chưa có xác nhận build Windows cho commit cuối, chưa nghiệm thu thao tác chuột với vùng chọn có đối tượng thật, chưa đo hiệu quả dự đoán sử dụng trong thực tế.
- [ ] Cần bổ sung gợi ý theo **loại thực thể CAD** (Line, Arc, Circle, Polyline, Hatch, Block, Dimension...) với vòng đời chọn/không chọn/undo; giai đoạn hiện tại mới ưu tiên theo số lượng đối tượng được chọn.
- [ ] Cần mở rộng bao phủ bản dịch tiếng Việt qua thống kê CI của toàn bộ CommandSpec, dịch Properties/Settings, và đo bộ hồ sơ AutoCAD 2027 độc lập.

**Giới hạn nghiệm thu:** Tầng gợi ý là trợ lý thao tác theo quy tắc và lịch sử phiên; không phải mô hình AI tự học đã qua huấn luyện. Không thay đổi CAD engine và không bảo đảm chức năng AutoCAD 2027 chưa kiểm thử.

## 14. Đợt nâng cấp nhận diện thực thể được chọn — native CAD EntityKind

- [x] Không chỉ đếm vùng chọn: đọc trực tiếp `EntityKind` của các đối tượng thật trong tài liệu hiện hành, **không dùng AI phỏng đoán hình học**. Hỗ trợ phân nhóm: LINE/XLINE/RAY; CIRCLE/ARC/ELLIPSE/SPLINE; LWPOLYLINE/POLYLINE3D/MLINE; TEXT/MTEXT; INSERT/BLOCK; DIMENSION/LEADER; HATCH; MIXED/OTHER.
- [x] Không sao chép nội dung đối tượng khi tính gợi ý: đọc `&EntityKind` theo tham chiếu, giảm chi phí với bản vẽ có nhiều đối tượng được chọn.
- [x] Gợi ý lệnh theo hình học thật: LINE → OFFSET/TRIM/EXTEND/DIST; CIRCLE/ARC → TRIM/OFFSET/DIMRADIUS; POLYLINE → PEDIT/AREA/HATCH; BLOCK → BEDIT/EXPLODE; TEXT → TEXTEDIT/STYLE; DIMENSION → DIMEDIT/DIMSTYLE; HATCH → HATCHEDIT; MIXED → MOVE/COPY/ROTATE.
- [x] Tất cả lệnh đề xuất qua `find_command`, không gợi ý lệnh chưa đăng ký trong engine. Dòng trạng thái hiển thị số lượng + loại đối tượng theo **English (Tiếng Việt)**; không có hành vi tự chạy lệnh.
- [x] Regression `native_entity_kind_guides_related_commands_without_inventing_geometry` kiểm tra LINE, CIRCLE, vùng chọn nhiều loại, danh mục gợi ý hợp lệ; đã đưa vào Windows CI.
- [ ] Phải xác nhận Windows CI của commit mới và kiểm thử EXE với hình học thật, đặc biệt trường hợp chọn nhiều đối tượng bị ẩn/xóa hoặc đổi bản vẽ/Undo.
- [ ] Chưa có kiểm chứng 100% mọi lệnh AutoCAD 2027 hoặc tự học AI ngữ nghĩa. Đây là mô hình gợi ý dựa trên các kiểu đối tượng CAD đã có và lịch sử phiên.

**Ví dụ giao diện**: `1 LINE (Đường thẳng) · NEXT (Tiếp): O (OFFSET) | TR (TRIM) | EX (EXTEND)`. Các alias hiển thị thực tế được lấy từ `CommandSpec.aliases`, không gán cứng nếu registry không đăng ký.

## 15. Sửa khả năng nhập tiếng Việt có dấu từ trạng thái Command: (Windows IME)

- [x] Đã tìm và sửa nút thắt thật tại `cmdline::keyboard`: trước đây `enable_ime` chỉ được gọi khi `Prompt.accept.text` (chế độ TEXT/MTEXT). Người dùng nhập câu tiếng Việt khi **chưa có lệnh đang chạy** không được bật bộ gõ IME, nên dễ không nhập được tiếng Việt có dấu.
- [x] `command_line_needs_ime`: bật IME ở trạng thái rảnh `Command:` và khi engine hỏi nhập văn bản; không ép IME khi engine đang chờ chọn điểm, nhập số hoặc chọn đối tượng. IME chỉ ghi vào buffer sau sự kiện commit, không thay đổi luồng thực thi lệnh.
- [x] Bổ sung regression `vietnamese_ime_works_at_idle_command_but_not_inside_numeric_point_prompts` và đưa vào Windows Gate V1.027.
- [ ] Phải thử **Windows IME thật (Telex/VNI)** ở EXE và nhiều chế độ ngôn ngữ; unit test không mô phỏng toàn bộ nhập liệu hệ điều hành.
- [ ] Cần nghiệm thu tiếp các thao tác Space-to-Enter với lệnh L/PL/C/REC/DI, gõ cụm “làm sạch bản vẽ” có dấu và không dấu, nhập số/toạ độ vào Prompt, Esc/Enter/Undo.

**CI riêng V1.027 đã từng PASS** trên commit `03851999` ở https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/42348094604. Commit mới nhất chứa sửa IME cần chạy lại; không sử dụng PASS cũ làm nghiệm thu bản mới.

## 16. Khắc phục tràn chữ dòng lệnh và palette tùy chọn song ngữ

- [x] Đã bổ sung **OPTIONS (Tùy chọn lệnh gốc)** dạng `egui::Area` đặt phía trên dòng lệnh, tự xuống dòng khi nhiều tùy chọn. Không tiếp tục nhồi toàn bộ chữ English + (Tiếng Việt) lên một dòng lệnh khiến mất chỗ nhập tọa độ và chữ bị tràn sang bên phải.
- [x] Đọc `Prompt.keywords` **đầy đủ**; hiển thị từng mục theo `English (Tiếng Việt)` khi có bản dịch. Mỗi mục có thể nhấp và gửi đúng `Input::Keyword(keyword)` với **chuỗi tiếng Anh gốc** của engine, không dịch đầu vào, không thay cú pháp lệnh hoặc alias.
- [x] Dòng lệnh gốc chỉ hiển thị chỉ dẫn ngắn `[OPTIONS ↓]`; gợi ý trạng thái tương tác cũng tự **wrap** theo chiều rộng bản vẽ, chiều cao tự điều chỉnh thay vì vẽ chuỗi dài vượt khỏi vùng nhập lệnh.
- [x] Bổ sung 31 mục chưa có trong từ điển tiếng Việt, gồm `Width (Bề rộng)`, `Halfwidth (Nửa bề rộng)`, `Length (Chiều dài)`, `Join (Nối)`, `Previous (Đối tượng trước)`, `Next (Tiếp theo)`, và các lời nhắc chọn điểm/đối tượng.
- [x] Test mới `full_bilingual_option_palette_keeps_all_original_keywords` xác nhận 8 từ khóa được bảo toàn 100% và hiển thị đầy đủ, kể cả lệnh có nhiều tùy chọn; test `native_keyword_catalog_never_changes_when_rendered_bilingually` bảo vệ lệnh gốc.
- [ ] Cần Windows CI của commit cuối PASS và nghiệm thu EXE thật trên màn 1366×768, 1920×1080, HiDPI 125–150%; thử gõ, nhấp, OSNAP, Enter, Escape, chuột trên canvas để không bị panel đè.
- [ ] Bộ gợi ý này chưa phải mô hình AI hiểu kiến trúc toàn diện. V1.027 vẫn chưa được nghiệm thu và chưa thể tuyên bố tương đương 100% AutoCAD 2027.

**Ví dụ mong đợi trong lệnh PLINE:** `Undo (Hoàn tác)`, `Close (Đóng)`, `Arc (Cung tròn)`, `Width (Bề rộng)`, `Length (Chiều dài)`; chỉ hiển thị lựa chọn nào thuộc `Prompt.keywords` thực tế ở bước đó. Không bao giờ gợi ý `C` hoặc `A` như tùy chọn đang hoạt động nếu engine chưa cấp keyword đó.

## 17. Nâng cấp V1.027: tối ưu bảng OPTIONS trên màn hình nhỏ và viết tắt keyword có kiểm chứng

- [x] OPTIONS song ngữ tự xuống dòng **và giới hạn chiều cao** theo 28% chiều cao canvas (tối đa 180 điểm), cuộn danh sách nếu nhiều mục. Tránh bảng dài che khuất vùng bản vẽ/dòng lệnh trên laptop hoặc HiDPI; màn hình thật vẫn cần nghiệm thu.
- [x] Trạng thái lệnh gợi ý **tọa độ tuyệt đối** `1000,2000` khi engine nhận point; gợi ý thêm tọa độ tương đối `@1000,0` chỉ khi `Prompt.base` có điểm gốc. Không gợi ý tọa độ khi prompt yêu cầu chọn đối tượng.
- [x] Enter được hướng dẫn **dựa trên cờ `Prompt.accept.enter`**; không đánh đồng việc có keyword là có thể Enter kết thúc, hoặc ngược lại.
- [x] Danh sách keyword hiển thị `English (Tiếng Việt) · [Alias]` nếu alias **thực sự được `Prompt.match_keyword` chấp nhận**. Ví dụ `Undo (Hoàn tác) · [U]`, `Close (Đóng) · [C]`; không sử dụng alias suy đoán từ tiếng Việt và không gửi chuỗi đã dịch vào engine.
- [x] Test `prompt_guidance_respects_enter_and_real_point_input_context`, `keyword_abbreviation_is_verified_with_native_prompt_parser` và `full_bilingual_option_palette_keeps_all_original_keywords`; đã đưa vào Windows CI.
- [ ] Windows CI của commit cuối cần PASS; kiểm tra UI thực tế với F1, L/PL/TR, tọa độ, IME tiếng Việt và thu/phóng cửa sổ.

Lưu ý: bảng OPTIONS chỉ hiển thị tùy chọn mà Prompt đang cho phép, không phải quảng cáo thêm lệnh CAD chưa có. Không tuyên bố hoàn thiện 100% AutoCAD 2027 nếu chưa đối chiếu/kiểm thử tất cả.

## 18. V1.027: Dòng lệnh Unicode an toàn và kiểm thử IME thực tế (egui)

- [x] Đã sửa sai lệch ở test tích hợp IME cũ: khi hoàn thành lệnh TEXT/MTEXT, trạng thái `Command:` hiện phải **vẫn cho phép** IME để gõ tiếng Việt tự nhiên; chỉ điểm/số/đối tượng mới tắt IME. Trước đây test cũ còn yêu cầu IME tắt ở trạng thái rảnh.
- [x] `command_input_tail` cuộn **phần hiển thị** tới cuối input, trên ranh giới ký tự UTF-8 an toàn; `app.cmd.buffer` gốc không bị cắt, không thay đổi dữ liệu gửi tới CAD.
- [x] Khi prompt song ngữ dài, khu vực hiển thị lời nhắc được cắt theo chiều rộng và **dành riêng không gian bên phải** cho nhập tọa độ, nhập lệnh và caret; dòng IME preedit được cắt trong vùng nhập.
- [x] Giữ con trỏ và vị trí cửa sổ IME không lấn lên mũi tên lịch sử lệnh khi chuỗi dài.
- [x] Test `command_input_scroll_keeps_original_vietnamese_and_coordinates` bảo đảm không làm mất dữ liệu, Unicode hợp lệ; thêm cổng Windows chạy `cmdline::tests::ime_preedit_is_shown_and_its_commit_is_inserted_once` để kiểm tra IME preedit/commit trong egui thật (dạng test).
- [ ] Chưa thể coi test egui là kiểm thử Telex/VNI ngoài Windows thật. Cần chạy EXE trên Win10/11, gõ dấu, chuyển focus, mở bảng OPTIONS, gõ tọa độ rất dài, Zoom/OSNAP và thử cửa sổ 1366×768/HiDPI.
- [ ] Cổng Windows CI commit cuối phải PASS; chưa có xác nhận chấp nhận toàn bộ AutoCAD 2027 và chưa phát hành bộ cài V1.027.

**Nguyên tắc:** mọi thay đổi chỉ phục vụ render và IME; tên command, native PGP alias, keyword/parser, buffer, Enter/Esc, DWG và Undo/Redo không được thay đổi.

## 19. Đợt nâng cấp kiểm thử CAD engine thực tế và tăng tốc nhập liệu

- [x] Cổng Windows V1.027 được mở rộng để chạy **test hình học OSNAP thực tế** (`cadcraft-engine snap::tests::`), bao gồm endpoint, midpoint, intersection, center, perpendicular/tangent, các tình huống cung và deferred snapping; sử dụng mã engine hiện có chứ không thay bằng kiểm thử giao diện giả.
- [x] Thêm bài test `pointmod::tests::` cho cú pháp snap override/point modifier; đồng thời chạy integration suite `point_modifiers::` và `prompt_keywords::` của engine để kiểm tra LINE, tọa độ tương đối, FROM/M2P, lọc tọa độ, góc khóa và tương tác keyword gốc.
- [x] Dòng lệnh nhập dài sử dụng `command_input_tail` đo bề rộng bằng **binary search theo ranh giới ký tự Unicode**, tránh đo hàng nghìn hậu tố trong mỗi frame. Tối ưu phần trình bày; toàn bộ chuỗi lệnh/IME gốc vẫn giữ nguyên để engine xử lý.
- [x] Thêm kiểm thử `long_cad_input_binary_scroll_limits_layout_work_and_does_not_split_accents`, đảm bảo chuỗi hàng chục nghìn ký tự được hiển thị với số lượt đo chữ nhỏ và không bắt đầu bằng dấu tổ hợp tiếng Việt rời.
- [ ] Các test trên Windows CI commit cuối phải PASS. Unit/integration OSNAP chưa thay thế thao tác chuột OSNAP thật với chuỗi chọn điểm/đường tham chiếu trong EXE.
- [ ] Tiếp tục nghiệm thu loại snap END/MID/CEN/INT/PER/TAN với snap settings, nhiều đối tượng, zoom và HiDPI; phải đối chiếu vị trí marker với điểm engine thực nhận, cùng tính chính xác DWG lưu/mở lại.

**Mốc quản lý:** Đợt này chuyển từ thuần kiểm thử UI đến kiểm thử geometry/command engine thật. Không đánh đồng test engine PASS với bộ cài và mọi lệnh AutoCAD 2027 đã hoàn thành.

## 20. V1.027 — Kiểm chứng OSNAP → LINE → DXF lưu và mở lại

Lần nâng cấp này bổ sung **kiểm thử hành vi engine thực tế**; không thay thế OSNAP bằng mô phỏng UI hay quảng cáo tính năng chưa có:

1. Tạo bản vẽ nguồn chứa đường thẳng và đường tròn. Gọi hàm OSNAP native để tìm điểm **END** và **CEN** từ vị trí con trỏ gần cấu kiện.
2. Khởi động lệnh `LINE` thật qua `Session::start("line")`, truyền hai điểm qua `Input::Point` và kết thúc bằng `Input::Enter`. So sánh tọa độ đầu–cuối với điểm bắt được từ OSNAP.
3. Xuất bản vẽ có LINE mới qua `cadcraft_io::write_dxf`, đọc lại qua `read_dxf`; xác nhận đối tượng sau khi mở lại **vẫn có kiểu `EntityKind::Line`**, tọa độ hai đầu không sai lệch quá `1e-9`.
4. Kiểm thử negative: khi `OSMODE::OFF` bật hoặc con trỏ nằm ngoài aperture của END/MID/CEN/INT thì không tự nhận một điểm sai.

Các bài hồi quy đã bổ sung tại `crates/engine/src/snap.rs`:
- `snapped_endpoint_and_center_become_exact_native_line_geometry`
- `snapped_native_line_survives_dxf_save_and_reopen_editably`
- `osnap_never_invents_a_pick_outside_aperture_or_when_disabled`

Các bài test đã được khai báo là bắt buộc trong `cad360-v1027-command-ux.yml`.

**Giới hạn nghiệm thu:** Đây là kiểm chứng trên **engine + DXF**, không phải kiểm thử nhấp chuột OSNAP trên Windows GUI, không chứng minh định dạng DWG AutoCAD tương thích 100%. Để hoàn tất cần ghi hình/ảnh chụp marker đúng vị trí, chọn bằng chuột, so sánh tọa độ được gửi vào engine, kiểm thử DXF và DWG độc lập với AutoCAD, và Windows CI PASS.

## 21. Kiểm thử Undo/Redo và hồ sơ hình học kiến trúc PLINE khép kín

- [x] `osnap_line_undo_redo_restores_exact_geometry`: lấy tọa độ END/CEN của OSNAP native, dựng LINE bằng engine, xác nhận số lượng đối tượng tăng chính xác, `Undo` trở về như trước và `Redo` phục hồi đoạn thẳng **đúng tọa độ** trong sai số `1e-9`.
- [x] `native_closed_pline_preserves_editable_vertices_after_dxf_reopen`: chạy `PLINE 0,0 6000,0 6000,4000 0,4000 C` tạo hình học mặt bằng chữ nhật 6000×4000 khép kín, xác nhận `LwPolyline` có bốn đỉnh và cờ `closed`; lưu DXF và mở lại vẫn là thực thể CAD có thể chỉnh sửa, không chuyển thành ảnh.
- [x] Hai bài test đã được bổ sung vào **Windows Gate V1.027**, cùng các test đang có cho END/CEN, aperture/OFF, LINE → DXF → reopen, point modifiers và keyword parser.
- [ ] Chưa xác nhận Windows CI của commit cuối; cần chạy test và sửa nếu lỗi, trước khi nghiệm thu.
- [ ] Cần tiếp tục kiểm thử marker OSNAP với chuột/HiDPI trên EXE, Undo/Redo thực sự qua nút Ribbon và tổ hợp Ctrl+Z/Ctrl+Y, cùng kiểm tra DWG AutoCAD hai chiều thực tế.

**Giới hạn:** Bản vẽ hình học kiểm thử bằng DXF là một phần chuỗi nghiệm thu kỹ thuật, không đủ chứng minh DWG tương thích AutoCAD 2027 hoặc AI hiểu trọn vẹn mặt bằng kiến trúc.

## 22. Kiểm chứng chuỗi chỉnh sửa và nâng cấp nhãn OSNAP gần mép vùng vẽ

- [x] `offset_move::architectural_move_copy_offset_undo_redo_and_dxf_reopen`: lệnh **MOVE** di chuyển LINE từ (0,0)–(10,0) tới (5,2)–(15,2), **COPY** tạo bản sao tại y=6, **OFFSET** tại y=4, sau đó Undo xóa đúng lần offset và Redo khôi phục hình học. DXF lưu/mở lại phải có đúng ba thực thể `EntityKind::Line` với các cặp tọa độ như ban đầu, không bị raster hóa.
- [x] Bổ sung test integration trên Windows CI cho chuỗi chỉnh sửa, hoàn tác và DXF roundtrip.
- [x] `canvas/cursor.rs`: sửa **bố cục nhãn OSNAP** theo vùng clip của canvas. Khi điểm snap nằm sát góc/mép phải hoặc mép dưới, nhãn chuyển sang phía còn chỗ và luôn nằm trong canvas; marker vẫn đặt tại tọa độ snap do engine tính toán, **không dịch tọa độ điểm bắt**.
- [x] Regression `osnap_labels_fit_all_canvas_corners_without_changing_snap_location` kiểm tra bốn góc và tâm vùng vẽ; kiểm thử được gắn vào Windows CI.
- [ ] Chưa có nghiệm thu EXE Windows cho marker và thao tác chọn đối tượng bằng chuột. Cần đối chiếu vị trí marker hiển thị, `cursor` và tọa độ `Input::Point` engine nhận qua phóng to/thu nhỏ và HiDPI.
- [ ] Chưa kiểm thử DWG thực sự với AutoCAD. DXF roundtrip chỉ bảo đảm phần hình học trong test; chưa được dùng để tuyên bố tương thích 100% DWG.

**CI:** chỉ công bố đã PASS khi workflow V1.027 của commit cuối kết luận `success`.

## 23. OSNAP English-first (Tiếng Việt) và chuỗi chỉnh sửa hình học nghiệm thu

- [x] Snap marker tooltip đọc nhãn OSNAP **English (Tiếng Việt)** thay vì chỉ dịch/chỉ tiếng Anh. Các nhãn đã kiểm thử gồm `Endpoint (Điểm cuối)`, `Midpoint (Trung điểm)`, `Center (Tâm)`, `Intersection (Giao điểm)`, `Perpendicular (Vuông góc)`, `Tangent (Tiếp tuyến)`. Giữ nguyên `SnapHit.mode` và `SnapHit.point`.
- [x] Các nhãn được đặt trong vùng clip canvas tại bốn góc và tâm; không sửa hình học CAD để chiều theo hiển thị.
- [x] Integration test `architectural_move_copy_offset_undo_redo_and_dxf_reopen`: LINE (0,0)→(10,0), MOVE (+5,+2), COPY (+0,+4), OFFSET +2 theo phía chọn; kết quả chính xác tại y=2,4,6. Undo xóa bản offset vừa tạo, Redo khôi phục; DXF lưu/mở lại vẫn là các đường LINE CAD.
- [x] Bổ sung các test của mục này vào Windows CI V1.027 và cập nhật thông báo lỗi workflow.
- [ ] Xác minh Windows CI mới nhất PASS, chạy trực tiếp EXE thử chuột OSNAP, click marker, chọn điểm, Undo/Redo, so sánh tọa độ engine thực nhận.
- [ ] Đối chiếu DWG hai chiều với AutoCAD độc lập; DXF regression không phải bằng chứng tương đương DWG hoặc AutoCAD 2027 hoàn chỉnh.

**Trạng thái:** đã commit mã nguồn, yêu cầu kiểm thử Windows và thao tác thực tế vẫn là cổng nghiệm thu bắt buộc.
