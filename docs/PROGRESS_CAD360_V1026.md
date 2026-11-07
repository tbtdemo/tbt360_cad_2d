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

## Windows 15 MP — sửa lỗi profiler dựa trên log thật

- **Bằng chứng:** Job Windows `42325024481 / 128258852022` đã biên dịch được GUI, MCP/CLI; test `large_png_5000_by_3000_preserves_original_pixel_to_cad_calibration` báo **1 passed; 0 failed**. Tuy nhiên workflow FAIL tại bước kiểm tra `$LASTEXITCODE` sau script, **không phải do test 15 MP không đạt**.
- **Lỗi kế tiếp:** Script `scripts/profile_ai_raster.ps1` chạy `Start-Process cargo`, nhưng không cập nhật `$LASTEXITCODE` của PowerShell gọi. Giá trị lỗi cũ làm gate báo FAIL.
- [x] Khi test exit code = 0, cập nhật rõ `$global:LASTEXITCODE = 0` để workflow không đọc mã thoát lỗi cũ.
- [x] Chuyển `cargo test --no-run` ra ngoài stopwatch; số đo mới ghi `cargo_test_wall_ms_after_precompile`, **không gọi là thời gian suy luận AI thuần**.
- [x] Thêm regression đặt `$LASTEXITCODE=77` trước khi gọi profiler trong CI; test PASS bắt buộc trả về 0.
- **Không có kết quả RAM hợp lệ** ở lần chạy này: worker kết thúc nhanh hơn chu kỳ lấy mẫu; cần benchmark process sống lâu hơn/đo memory chuyên dụng.
- [ ] Lượt Windows mới phải PASS bài 15 MP và 36 MP đúng SHA. Chưa có Setup/Portable V1.026 nghiệm thu.
- **% tổng giữ 33%; phát hành nghiệm thu 0%.** Có bằng chứng PASS một test đơn lẻ trên commit trước, chưa thể áp dụng cho commit mới.

## Đợt tiếp theo — đo RSS đúng tiến trình và nhận dạng phòng chung tường

- [x] Nâng `scripts/profile_ai_raster.ps1`: `cargo test --no-run --message-format=json` để tìm **chính xác EXE thử nghiệm**; chạy trực tiếp EXE đó, đặt mặc định lấy mẫu 10 ms, đo `WorkingSet64` của chính PID đang chạy. Không lấy nhầm Cargo/rustc; không gọi thời gian bao gồm compile là inference.
- [x] Bắt buộc test phải có tên chính xác trong danh sách `raster_vector::tests::...`; `0 tests` không được coi là PASS.
- [x] Nếu worker chạy nhanh đến mức không lấy được mẫu, ghi `NOT_CAPTURED`; không tự sinh RAM. Nếu test thành công, xóa mã lỗi PowerShell cũ.
- [x] Cập nhật `closed_rectangles` để **tường ngăn thật được phép dùng chung** làm hai mép đóng phòng kề nhau, trong khi mỗi cặp top/bottom chỉ dùng một lần. Không thêm nét giả và không nhân đôi chỉ số nguồn.
- [x] Test `ai_adjacent_rooms_share_true_partition_wall_without_fake_geometry`: 2 phòng dùng chung một cạnh nguồn, có 2 ứng viên đường bao và cùng tham chiếu cạnh đó.
- [x] Yêu cầu bài test này trong cả AI Large Image Gate và Windows Setup/Portable Gate.
- [ ] CI Windows theo commit mới cần PASS trước khi coi hai tính năng trên đã nghiệm thu.
- [ ] Đo RAM có mẫu; lấy benchmark thực trên 15/36 MP; tường/cửa/cột/phòng trong bản vẽ thật; nghiệm thu DWG AutoCAD độc lập.

**Tiến độ triển khai giữ 33%, nghiệm thu phát hành 0%.** Các thay đổi mới là mã nguồn và bài thử đã commit; không phải bộ cài thành công.

## AI Studio V1.026 — phát hiện phòng trong Preview không phụ thuộc lệnh vẽ

- [x] Sửa `raster_vector.rs`: phân tích `closed_rectangles(&lines)` cho **bằng chứng phòng khép kín** ngay cả khi `closedRectangles=false`. Tách rõ **phát hiện tham khảo** khỏi **tạo PLINE/native CAD**; không tự thêm đường tường hoặc suy đoán công năng phòng.
- [x] Test PNG phòng khép kín thực tế với `closedRectangles=false`: yêu cầu `roomCandidateCount=1`, ứng viên cần duyệt `requiresHumanReview=true`, `editableEntitiesCreated=0`, `closedPolylines=[]`.
- [x] Đưa test vào AI Windows Gate và Windows Release Gate.
- [ ] Windows CI phải PASS trên SHA chứa mã mới; chưa được coi là nghiệm thu.
- [ ] Trên bản vẽ nhiễu, hở tường, phòng méo/không vuông góc hoặc có nhiều ngõ mở phải thực nghiệm và đo sai số độc lập; chức năng hiện chỉ phát hiện đường bao chữ nhật đủ bốn cạnh.

**% chung vẫn 33% triển khai, 0% nghiệm thu phát hành**, không tuyên bố đạt AI hiểu hoàn chỉnh mặt bằng.

## Bổ sung topology mặt bằng — cặp phòng cùng tường nguồn

- [x] `raster_vector.rs`: từ những ứng viên phòng đã khép kín, sinh `roomAdjacencies` và `roomAdjacencyCount` khi **hai phòng dùng đúng cùng một `sourceEdgeIndex`**. Không suy diễn cạnh mới, không ghép hai phòng chỉ vì ở gần nhau, không nhận một vách ngăn là cửa đi.
- [x] `ai_raster_panel.rs`: thể hiện từng cặp vùng tiếp giáp, chỉ số nét gốc làm bằng chứng, thông báo `chưa xác nhận lối thông`. Đây là dữ liệu Preview không thay đổi CAD.
- [x] Test `ai_room_topology_links_only_real_shared_source_edges_without_inventing_doors`: một cặp dùng chung cạnh phải được báo, vùng thứ ba không liên quan không được nối giả.
- [x] Thêm bài kiểm thử topology trong AI Gate và Windows Release Gate.
- [ ] Runner Windows trên SHA mới phải biên dịch và chạy hết kiểm thử, đo RAM 15/36 MP, nghiệm thu Setup/Portable.
- [ ] Cần dữ liệu thực tế có nhãn tường và cửa để nâng từ topology hình học lên AI ngữ nghĩa.

**% triển khai vẫn 33%; phát hành nghiệm thu 0%.** Sơ đồ kề phòng là chứng cứ hình học, chưa chứng minh AI hiểu toàn bộ mặt bằng.

## Tiếp tục V1.026 — đo tường chung CAD và giữ nguyên ranh giới ngữ nghĩa

- [x] Lấy tọa độ hai đầu và chiều dài từ **chính nét tường nguồn** của liên kết phòng `roomAdjacencies`, quy đổi sang CAD theo `origin/unitsPerPixel`; bổ sung `sharedWallCAD`, `sharedWallLengthCAD`, `canAutoCreateDoor=false`. Không sửa hình học đầu vào.
- [x] Test riêng đảm bảo một cặp phòng chung tường không phát sinh `opening` hoặc `doorBlockDraft` do suy đoán.
- [x] Gắn bài test vào CI ảnh lớn và đóng gói Windows.
- [ ] Ghép được **khoảng mở có đủ bằng chứng từ hai mép tường** với hai không gian phòng, có kiểm chứng tọa độ và review người dùng (chưa đạt).
- [ ] CI trên commit cuối phải PASS, RAM đo thực tế và Setup/Portable/Source/Word nghiệm thu đồng bộ (chưa đạt).
- [ ] Kiểm tra OSNAP trên các LINE/PLINE native và DWG hai chiều trong AutoCAD độc lập (chưa đạt).

**Tổng tiến độ kỹ thuật giữ 33%, nghiệm thu phát hành 0%**. Không biến một cạnh dùng chung thành cửa nếu chưa có bằng chứng.

## V1.026 — liên kết khoảng mở hai mép tường với phòng đã đo

- [x] Lõi `raster_vector.rs`: xây `observed_opening_room_contacts`, chỉ xét **paired opening** có hai mép tường quan sát được; dựa vào tọa độ tâm vùng mở và khoảng cách có giới hạn theo bề dày tường. Tối đa hai vùng có biên phù hợp; bỏ qua khoảng mở xa phòng, nằm ở góc hoặc dữ liệu thiếu.
- [x] Preview JSON thêm `openingRoomContactCount` và `openingRoomContacts` với `openingIndex`, `roomIndices`, `midpointPixels`, `isDoor=false`, `passageConfirmed=false`, `requiresHumanReview=true`.
- [x] AI Studio hiển thị khoảng mở gần phòng số mấy; nhấn mạnh không đồng nghĩa xác nhận cửa, không sinh hình học native.
- [x] Test `ai_paired_gap_near_two_room_boundaries_remains_unverified`: một khoảng mở nằm giữa hai phòng phải liên hệ được, một khoảng mở ở xa và gần góc không được nhận nhầm.
- [x] Thêm test vào AI Gate + Release Gate. **Chưa có Windows PASS cho commit mới.**
- [ ] Cần bản vẽ thực tế có tường gián đoạn, cửa gắn với đúng hai vùng phòng và mẫu không vuông góc để đánh giá Precision/Recall; cơ chế hiện tại mới đối chiếu vị trí hình học theo ngưỡng.
- [ ] Kiểm thử CI Windows / RAM 15–36MP / DWG độc lập / Setup–Portable–Source–Word theo đúng SHA.

**Tiến độ tổng vẫn 33% triển khai và 0% nghiệm thu phát hành**: chưa có bằng chứng kiểm thử tích hợp và chưa đạt nhận dạng AI ngữ nghĩa.

## Nâng cấp AI khoảng mở — đối chiếu đúng hướng tường

- [x] `observed_opening_room_contacts` kiểm tra **bốn góc thật của khung khoảng mở** và trục cạnh đúng hướng: tường đứng cần chiều rộng qua tường phù hợp `wallThicknessPixels`; tường ngang cần chiều cao qua tường phù hợp. Tâm gần phòng nhưng hướng cạnh sai không tạo liên kết giả.
- [x] Test `ai_room_contact_rejects_crosswise_and_skewed_opening_shapes`: từ chối khoảng mở xoay ngang khi tiếp giáp tường đứng; loại bỏ tứ giác xiên, bốn góc không vuông. Không tự sinh cửa hay tạo native CAD.
- [x] Thêm bài thử vào cả AI Large Image Gate và Windows Release Gate.
- [ ] Xác nhận Windows CI PASS trên SHA chứa thay đổi; chưa có dữ liệu Precision/Recall cửa thực tế hoặc báo cáo RAM hợp lệ.
- [ ] Khả năng nhận dạng phòng hình đa giác/phòng hở tường và tương tác AutoCAD độc lập vẫn chưa nghiệm thu.

**Tổng triển khai checklist giữ 33%; nghiệm thu phát hành 0%.** Đây là bộ lọc hình học an toàn, không phải kết quả AI đã hiểu cửa đi.

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
