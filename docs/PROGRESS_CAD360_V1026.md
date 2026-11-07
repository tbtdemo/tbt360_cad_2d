# CAD 360 Việt Nam — Tiến độ có kiểm chứng V1.026

**Mục tiêu**: hoàn thiện AI cho dữ liệu kiến trúc lớn, hiển thị lệnh/OSNAP rõ ràng, DWG chuẩn và một gói phát hành có bộ cài + mã nguồn + Word đầy đủ.

**Nhánh làm việc:** `feature/CADTBT360_V1.026_AI_LARGE_IMAGE`  
**PR:** https://github.com/tbtdemo/tbt360_cad_2d/pull/13  
**CI:** https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/cad360-v1026-ai-large-image-gate.yml  
**Trạng thái:** PHÁT TRIỂN, **không được coi là bản phát hành PASS**.

## Cách tính phần trăm (bắt buộc cho các lần nâng cấp)

`% triển khai = số mốc kỹ thuật có bằng chứng commit / tổng số mốc kỹ thuật theo danh mục bên dưới × 100`. Mốc PASS Windows/AutoCAD chỉ được tính khi có log của **đúng commit**. Dấu [x] ở đầu việc 'đã viết code/test' **không** tương đương bài test PASS. Các nhóm có trọng số bằng nhau để tránh che lấp nhóm AI chưa đạt.

**% triển khai tổng hợp hiện tại: 33%** (làm tròn trung bình của 6 nhóm: 40%, 67%, 13%, 13%, 50%, 13%). **% nghiệm thu phát hành V1.026 = 0%** cho đến khi toàn bộ cổng phát hành bắt buộc PASS. Đây là thang mốc kỹ thuật do nhóm phát triển đặt ra, **không phải độ chính xác AI**.

| Nhóm bắt buộc | Mốc mã nguồn | % triển khai | Bằng chứng hiện có | Chưa đạt / bước tiếp theo |
|---|---:|---:|---|---|
| 1. PNG 15 MP trên Windows | 2/5 | **40%** | Test thật 5000×3000 + workflow Windows riêng | Chưa có PASS đúng SHA, log CPU/RAM, UI Preview |
| 2. 36 MP / nhiều nghìn nét | 4/6 | **67%** | 64 MiB/8192 px/36 MP, quét cặp tường 2500 nét, đường bao phòng 2400 nét, vùng 1024 px có chỉ số vector toàn cục | Test 36 MP, đo thời gian/RAM Windows chưa PASS; phân vùng suy luận/ghép cấu kiện semantic chưa hoàn chỉnh |
| 3. Hiểu tường/cửa/cột/phòng phức tạp | 1/8 | **13%** | Hiện có bộ tạo ứng viên hình học, không phải model semantic | Bộ ground truth, kiểm thử mở tường/cửa, phòng không kín, độ chính xác Precision/Recall, phát hiện chữ/kích thước, nhiều tầng/vùng |
| 4. Tự dựng mặt bằng chuẩn xác | 1/8 | **13%** | Các lệnh native và Preview+Approve đã có | Plan tổng thể, kết nối đối tượng, gán layer/block và kích thước, Undo nhóm, sai số/vòng đời CAD, AutoCAD đối chiếu |
| 5. Setup/Portable/ZIP nguồn/Word V1.026 | 3/6 | **50%** | GUI đổi nhãn V1.026, installer Inno Setup riêng, Windows release workflow tự kiểm tra Setup/Portable/Source SHA + 3 Word sau PASS | CI chưa PASS, chưa có EXE tải về; Word chỉ là sơ đồ, chưa có screenshot EXE; AutoCAD độc lập chưa nghiệm thu |
| 6. DWG hai chiều với AutoCAD | 1/8 | **13%** | CI lịch sử có DWG roundtrip nội bộ, chưa tính là nghiệm thu V1.026 | Bộ mẫu thật, độc lập AutoCAD, chỉnh sửa/lưu lại từ AutoCAD, compare hình học/layer/Xref, scan tương thích, báo cáo sai số |

Các tỷ lệ trên cố ý **không cộng điểm cho build đang Queued**. Nếu CI FAIL thì giữ mốc code, tuyệt đối không chuyển trạng thái nghiệm thu sang PASS.

## Kết quả thay đổi đợt hiện tại

- [x] Tách nhánh V1.026 thay vì chồng lên V1.025.
- [x] Bổ sung chỉ báo TRIM/EXTEND và gợi ý cạnh con trỏ khi Dynamic Prompt bị tắt.
- [x] Loại bỏ việc vẽ đúp chữ tên OSNAP.
- [x] AI Review hỗ trợ mỗi dòng một lệnh và kiểm tra toàn bộ kế hoạch trước Preview.
- [x] Tăng ngưỡng đọc ảnh PNG/JPEG có kiểm soát kích thước trước decode.
- [x] Test PNG 15 MP giữ nguyên tỷ lệ chuyển pixel → CAD.
- [x] Quét cặp nét tường theo vị trí thay vì O(n²) cho mọi nét; test 2500 nét với chỉ số nguồn gốc.
- [x] Đưa các bộ suy luận kiến trúc còn giới hạn vào luồng báo lỗi rõ ràng thay vì trả ứng viên rỗng.
- [x] Thêm Windows gate độc lập và điều kiện lưu nguồn + Word + SHA256 khi PASS.
- [ ] Windows gate đúng commit đã PASS.
- [x] Viết test ảnh thật 36 MP, log thời gian và kiểm tra giữ nguyên kích thước nguồn (chưa PASS).\n- [ ] Benchmark 36 MP nhiều trang và dữ liệu >10.000 nét có giới hạn RAM rõ ràng.
- [ ] Nhận dạng phòng mở/tường đứt/cửa+cột trên dữ liệu ground truth.
- [ ] Dựng mặt bằng hoàn chỉnh, sai số kích thước được thống kê.
- [ ] V1.026 Setup/Portable/ZIP nguồn/HDSD Word mới đồng nhất version.
- [ ] Kiểm chứng DWG AutoCAD hai chiều trên Windows có công cụ thật.

## Cấu trúc và nơi lưu code/hướng dẫn

- Lõi AI ảnh: `CADTBT360_V1.000/crates/mcp/src/raster_vector.rs` (**tên thư mục workspace cũ, không phải nhãn phiên bản phát hành**).
- Lõi giao diện TRIM/OSNAP: `CADTBT360_V1.000/crates/ui-egui/src/canvas.rs` và `canvas/cursor.rs`.
- AI Review: `CADTBT360_V1.000/crates/ui-egui/src/ai_review.rs`.
- CI AI V1.026: `.github/workflows/cad360-v1026-ai-large-image-gate.yml`.
- Hướng dẫn AI V1.026: `docs/user-guide/CAD360_V1.026_AI_LARGE_IMAGE.md`.
- Tiến độ mỗi đợt: **file này**.
- Gói pre-release thành công: GitHub Actions → workflow **CAD 360 V1.026 - AI Large Image Gate** → Artifacts `CADTBT360_V1.026_AI_Gate_<SHA>`, gồm ZIP nguồn, Word AI, Word tổng hợp nền, BUILD_INFO và SHA256SUMS.
- Chưa có **Setup/Portable V1.026** trong gói pre-release; không được giới thiệu đây là bản cài phát hành.

## Hướng dẫn cho người mới — đợt nâng cấp tài liệu 07/11/2026

- [x] Tạo giáo trình Word **34 bài / 39 trang A4**, thiết kế bìa, mục lục có số trang, bảng lệnh tắt trước bài 01, header/footer và số trang.
- [x] Mỗi bài có 6 bước hướng dẫn, mục tiêu, công cụ/lệnh, 1 sơ đồ minh họa có caption, tự kiểm tra và xử lý lỗi.
- [x] Render toàn bộ Word thành PNG, kiểm tra bố cục tổng thể và không tràn trang.
- [x] Lưu khung giáo trình chi tiết tại `docs/user-guide/CAD360_V1.026_GIAO_TRINH_NGUOI_MOI_34_BAI.md`.
- [ ] Hình chụp thật từng thao tác từ Windows EXE theo đúng SourceCommit; bản hiện tại dùng sơ đồ kỹ thuật giảng dạy, không giả làm screenshot.
- [ ] Gắn nguồn Word và hình vào workflow đóng gói Setup/Portable V1.026, nghiệm thu hash trước phát hành.
- **Tài liệu không làm tăng điểm AI lõi/nhận dạng. Mốc chung hiện 33% nhờ hạ tầng ảnh lớn và workflow phát hành; nghiệm thu phát hành vẫn 0%.**

## Đợt hiện tại — nhận dạng ảnh nhiều nét và đo tải thực tế

- [x] Dò cặp nét tường bằng quét tọa độ + tìm nhị phân; bổ sung test 2.500 nét và 10.000 nét đồng trục.
- [x] Dò đường bao phòng chữ nhật bằng chỉ mục tọa độ, không tự trả trống trên 1.000 nét; kiểm thử phòng có chỉ số nguồn ở cuối danh sách 2.400 nét.
- [x] Bộ dữ liệu **3 phòng hình học đã gán nhãn + 1.800 nét nhiễu** và ca kiểm thử số phòng/đường nguồn. Chỉ là đánh giá đường bao chữ nhật, không phải độ chính xác ngữ nghĩa tường/cửa/cột.
- [x] Bảo vệ RAM cho nét xiên PCA trước vùng hatch đặc (giới hạn số điểm giữ lại, vẫn duyệt vùng để đánh dấu đã thăm).
- [x] Tạo bảng 1.024 px/vùng từ ảnh gốc, đếm mật độ pixel và tham chiếu **một nét toàn cục** khi nó cắt qua nhiều vùng; AI Studio hiển thị số vùng có nét, cảnh báo khi vùng quá dày. Chưa có thuật toán nhận dạng semantic phân vùng hợp nhất.
- [x] Tạo script Windows `scripts/profile_ai_raster.ps1` đo thời gian và RAM mẫu của process kiểm thử 15/36 MP, báo `NOT_CAPTURED` khi không thu được mẫu. Đã thêm vào CI nhưng chưa có số liệu PASS.
- [ ] Kiểm thử V1.026 đúng SHA trên GitHub Actions: các lượt trước có trạng thái `cancelled` hoặc `pending` mà chưa cung cấp số liệu.
- [ ] Đo ngưỡng tài nguyên thực, chạy benchmark đa tập bản vẽ, so sánh precision/recall có ground truth từ dự án thật, và tích hợp Setup/Portable V1.026.

**Thay đổi % trong đợt hiện tại:** 27% → 33% (**+6 điểm phần trăm**) theo checklist mã nguồn: nhóm ảnh lớn 3/6 → 4/6 và nhóm chuẩn bị gói phát hành 2/6 → 3/6.  Năng lực AI tự hiểu kiến trúc vẫn 13% theo checklist; phát hành nghiệm thu vẫn 0%.

## V1.026 — Chuẩn bị phát hành có thể tái tạo

- [x] Đồng bộ nhãn GUI/badge sang V1.026 trong `CADTBT360_V1.000/apps/cadcraft/src/main.rs`.
- [x] Inno Setup riêng: `installer/CADTBT360_V1.026_Windows.iss`.
- [x] Release workflow riêng: `.github/workflows/cad360-v1026-release.yml`, dùng đúng nhánh `feature/CADTBT360_V1.026_AI_LARGE_IMAGE`, gồm biên dịch, kiểm thử AI 15/36MP, đóng Setup/Portable/Source, kiểm tra cài–gỡ và SHA.
- [x] Bộ tạo Word người mới 34 bài với **34 sơ đồ thực**, bìa, mục lục và bảng lệnh tắt tại `scripts/build_beginner_guide.py`; phụ thuộc `python-docx==1.2.0` và `Pillow==11.3.0` được pin trong CI.
- [x] Setup/Portable phải chứa ba giáo trình Word (AI, tổng hợp, người mới) và cùng hash từng tệp.
- [x] `BUILD_INFO.txt` ghi rõ các mục chưa được xác minh: GUI thủ công, AutoCAD độc lập, độ chính xác nhận dạng kiến trúc, ảnh chụp EXE.
- [ ] V1.026 Windows AI Gate và Windows Release CI PASS trên đúng commit.
- [ ] Kiểm định Word in thật và thay sơ đồ bằng ảnh EXE thật theo từng thao tác.
- [ ] Đối chiếu DWG hai chiều bằng AutoCAD thật; kiểm tra không sai Layer/Block/Xref/scale.
- [ ] Xuất và xác nhận bộ cài thực tế, Word/ZIP mã nguồn và release tag trên GitHub.

**Chú ý:** GitHub đã nhận workflow V1.026 nhưng các lượt trước có trạng thái `pending` hoặc `cancelled`; phần trăm triển khai phản ánh *code đã commit*, không xác nhận chương trình đã chạy thành công.

## Đợt tiếp theo — 07/11/2026: kiểm thử diện tích phòng và rà phát hành

- [x] Bổ sung `referenceRoomRectsCAD` để so sánh ứng viên đường bao phòng với hình chữ nhật tham chiếu CAD `[xmin,ymin,xmax,ymax]`.
- [x] Xuất `roomGeometryBenchmark` gồm `referenceRoomCount`, `candidateRoomCount`, `matchedRoomCount`, Precision, Recall, F1 và `toleranceCAD`. Không ghép lặp một ứng viên vào nhiều phòng chuẩn.
- [x] Kiểm thử phòng có ứng viên giả, phòng thiếu, dữ liệu tham chiếu không hợp lệ; gắn kiểm thử vào cả AI Gate và Windows Release Gate.
- [x] Kiểm tra API workflow các lượt CI cũ: V1.026 `pending` khi chưa có jobs; các lượt `cancelled` không thể coi là PASS.
- [ ] Đưa benchmark này chạy trên >=3 bản vẽ kiến trúc thật có ground truth; ghi precision/recall phòng, tường, cửa, cột riêng.
- [ ] Chạy hết CI Windows trên đúng SHA, lấy log compile/error/RAM/15–36 MP và chứng nhận bộ cài.
- [ ] Dùng AutoCAD độc lập kiểm tra trao đổi DWG hai chiều; nộp biên bản đối chiếu.
- [ ] Ghi hình chụp EXE V1.026 cho từng bước bài giảng, thay sơ đồ trong Word trước nghiệm thu phát hành.

**Báo cáo % đợt này:** **Giữ 33% triển khai, 0% nghiệm thu phát hành**. Benchmark QA bổ sung chất lượng nhưng chưa hoàn thành mốc định lượng trên dữ liệu thật; không tăng % thiếu bằng chứng.

## Đợt sửa lỗi CI V1.026 — Windows run 42323719615

- **Kết quả thực tế:** Windows Setup/Portable #42323719615 **FAIL** tại step `Build and validate both Word manuals before Rust compile`. Log xác định `ValueError: User guide is incomplete: missing ['Bước 1', 'Bước 2', 'Bước 3']` khi tạo Word AI V1.026; đây **không phải** lỗi Rust. Các bước biên dịch, đóng gói, cài–gỡ phía sau bị bỏ qua.
- [x] Bổ sung mục giáo trình ba bước thực hành thật cho người mới: mở CAD, LINE + DIST 5.000 mm, SAVEAS/mở lại, rồi mới vào AI Preview.
- [x] Sửa metadata Word từ nhãn V1.025 hardcoded sang ưu tiên phiên bản **tệp DOCX đầu ra**; dự phòng đọc nội dung nguồn nếu cần.
- [ ] Kiểm tra kết quả workflow Windows trên commit chứa sửa lỗi; **chưa được công nhận PASS** nếu vẫn pending.
- [ ] Kiểm tra in Word 34 bài có ảnh thật và mục lục số trang, không tính sơ đồ thay ảnh chụp EXE đã nghiệm thu.
- **% kỹ thuật giữ 33%, nghiệm thu phát hành 0%.** Hoàn tất sửa một lỗi làm dừng pipeline không đồng nghĩa đạt các mốc nghiệm thu độc lập.

## CI V1.026 — lỗi Word số 2 và bản sửa có đối chiếu log

- **Windows run #42325024498:** `FixtureValidation=success`, `FullManuals=failure` do `ValueError: Markdown contains table cells but DOCX contains no actual Word tables`. Tình huống phát sinh khi tài liệu AI chứa dấu `|` dùng để ngăn cách lệnh; không phải bảng Markdown.
- [x] Sửa `docs/user-guide/build_word_guide.py`: chỉ yêu cầu Word table khi nguồn có dòng bảng `| cột 1 | cột 2 |` thực sự; dấu `|` nằm giữa câu lệnh CAD không bị hiểu nhầm.
- [x] Thêm regression trong workflow V1.026 phát hành: dùng `LINE | CIRCLE` không có bảng Markdown, tạo DOCX phải thành công.
- **AI Gate #42325024481:** đã bắt đầu công đoạn biên dịch `Compile CAD GUI, MCP and CLI`, chưa công bố PASS.
- [ ] Windows Setup/Portable mới phải chạy lại qua bước Word, Rust, kiểm thử, tạo bộ cài và lưu/mở DWG; chưa được nghiệm thu.
- **Tiến độ tổng giữ 33%, nghiệm thu phát hành 0%** vì pipeline chưa vượt hết các cổng.

## Đợt hiện tại — kiểm chứng tính trung thực của benchmark phòng

- [x] Siết `reference_room_geometry_benchmark`: phải có đủ **bốn đỉnh hợp lệ**, số hữu hạn trong giới hạn CAD và tạo thành hình chữ nhật song song trục. Không cho phép chỉ ba đỉnh hợp lệ hoặc góc bị xiên được tính là một phòng đúng.
- [x] Thêm test `ai_room_geometry_benchmark_excludes_missing_corner_and_skew_false_positive` cho hai ứng viên sai và một hình đúng. Yêu cầu Precision, Recall không tăng giả từ dữ liệu lỗi.
- [x] Thêm test này vào `cad360-v1026-ai-large-image-gate.yml` và `cad360-v1026-release.yml`.
- [ ] CI Windows trên đúng SHA có PASS hay không vẫn phải kiểm tra; các workflow trước có `pending` chưa khởi chạy job.
- [ ] Kết quả kiểm nghiệm tường/cửa/cột/phòng bằng dữ liệu thực tế vẫn còn thiếu.

**Tiến độ:** giữ **33% theo checklist code** và **0% nghiệm thu phát hành**. Kiểm thử benchmark là bảo đảm chất lượng số liệu, không là bằng chứng thêm về AI semantic.

## Định hướng lần nâng cấp tiếp theo (theo thứ tự)

1. **P0 – Gỡ lỗi CI Windows đúng SHA**: fix compile/test, chỉ ghi PASS khi kết quả hiện hữu. Không push liên tục làm nhiễu lượt kiểm thử.
2. **P0 – Tile + global coordinates**: đọc ảnh lớn theo vùng giao nhau, xử lý mỗi vùng với ngân sách memory, gộp đường trùng biên theo tọa độ pixel gốc, phục hồi toàn cục; test synthetic 36 MP và ground truth.
3. **P0 – Semantic QA**: tường đôi, nút T/X, cửa/ô mở, cột có hatch, phòng chưa kín; xuất confidence/evidence và bảng đo sai số trên ≥3 mặt bằng độc lập (không tự ý xác nhận semantic).
4. **P0 – Native DWG authoring**: chuyển ứng viên đã xác nhận thành LINE/PLINE/ARC/BLOCK/LAYER chỉnh sửa được, Undo/Redo nhóm, đóng/mở lại DWG giữ vị trí, đơn vị và metadata.
5. **P1 – UX chuyên nghiệp**: xử lý theo tiến trình có báo %, Stop/Cancel, thanh tình trạng AI, không khóa vùng vẽ; các tab con nằm ngang, hỗ trợ file lớn, chuẩn màu/chữ.
6. **P0 – Version + release**: sau khi CI đạt và kiểm thử EXE thật, đồng bộ tên/phiên bản GUI, installer, manual, SHA, Git tag; xuất Setup/Portable/Source/Word. Sau PASS tạo nhánh V1.027.

## Chuẩn báo cáo sau MỖI commit

Ghi rõ: (1) % từng nhóm và % tổng; (2) commit SHA, PR, vị trí file; (3) bài test mới, CI run + trạng thái; (4) sự khác biệt giữa mã đã viết và chức năng đã chạy; (5) danh sách blocker; (6) bước nâng cấp tiếp theo; (7) đường dẫn tải gói khi CI đã PASS. Không tuyên bố hoàn thành dựa trên số lượng commit.

**Nguyên tắc**: CAD bản địa và DWG phải tương thích trao đổi hai chiều. AI không được sửa bản gốc hay tự ghi hình học dựa trên suy luận thiếu bằng chứng; Preview/Approve là bắt buộc trước mutation.
