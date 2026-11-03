# CAD360 TBT — Theo dõi đợt nâng cấp 15–30 lượt

## Quy ước
- Một lượt = một nhóm thay đổi chức năng đã được commit và có bằng chứng mã nguồn. Mỗi lượt không mặc định tương đương một commit.
- Mốc đóng gói định kỳ: khi đã hoàn thành ít nhất 15 lượt và không quá 30 lượt; một đợt đóng gói Windows Setup, Portable và mã nguồn sau khi vượt qua các gate.
- Mọi thay đổi vẫn có mã nguồn sao lưu theo commit. Không tính job `skipped` là EXE PASS.
- Chưa kích hoạt đóng gói cho đợt này.

## Phạm vi ưu tiên từ R17 — quyết định ngày 03/11/2026

**Tạm dừng phát triển mới**: đồng bộ online/offline, xử lý xung đột thay đổi giữa các client, clash/xung đột giữa các bộ môn. Không xóa chức năng cũ, không tự thay đổi hành vi nghiệp vụ đã có. Tests hồi quy hiện hữu vẫn phải chạy.

**Giữ lại**: đăng nhập và kiểm tra account, phân quyền xem/sửa theo user, tầng/bộ môn ở mức quyền truy cập, kiểm tra cấu hình quyền. Không mở rộng thành engine giải quyết xung đột giữa các chuyên ngành.

**Ưu tiên**: 1) Windows CI/EXE thực, 2) chuột/click/ESC, 3) mở file CAD kích thước lớn và giữ giao diện phản hồi, 4) RAM/render/zoom/pan, 5) autosave/recovery, 6) quyền tài khoản và thiết lập phân quyền.

**Cách tính tiến độ mới**: giữ 8 đầu việc hoạt động từ bảng cũ và loại hạng mục C2 Sync/Conflict đang tạm ngưng. Các tỷ lệ [45,45,45,70,15,50,35,10] có trung bình **39,4%** (ước lượng kỹ thuật, chưa nghiệm thu). Không đưa tiến độ các việc dừng vào mẫu số. Các mục chưa đạt Windows PASS không được tự tăng %.

## Tiến độ đợt hiện tại
| Lượt | Phạm vi | Commit mã nguồn | Test bổ sung | CI |
| --- | --- | --- | --- | --- |
| 01/15 | IFC/OBJ: giới hạn 1024 byte cho tên nhóm; chặn đỉnh trùng không liền kề | `39c4ea534bf801fd73184e7bf3876a6b52835ae4` | 3 Rust unit tests | Source/metadata PASS; Rust tests chưa chạy |
| 02/15 | CAD+BIM: tự lưu 5 phút, DXF+JSON cùng phiên, lưu khi thoát, checksum, 3 bản backup, phục hồi an toàn; tối ưu UI | `4afe448ec36ffca90e25c20c70ec3c25ece8b69b` | 3 Rust unit tests; Windows cargo check/test mới | Source/IFC PASS; Rust integration PASS (run 41563233121) |
| 03/15 | CAD+BIM: không xóa backup hợp lệ khi nhiều phiên bị hỏng; kiểm tra DXF giải mã được trước khi phục hồi | `7d76a4498c734da3a7dc2158f85e9eaf66971ff8` | 2 Rust regression tests | Chưa xác minh lại CI |
| 04/15 | CAD+BIM: định dạng snapshot v2 lưu mọi tab CAD, chỉ số tab đang hoạt động và workspace; đọc v1; kiểm tra từng DXF trước khi phục hồi | `900a7cde8cdc097268c190af079f482ae14f78c4` | 1 regression test 3 tab và corruption | Chưa xác nhận cargo check/test Windows |
| 05/15 | CAD+BIM: nếu phục hồi nhiều tab lỗi giữa chừng thì đóng tab mới, khôi phục chỉ số tab gốc; giới hạn 256 bản vẽ và 64 MiB workspace ngay trước ghi; kiểm tra active index sai | `df80c04c43043e4a979a5fe8f3739e1f0385f6db` | 1 regression test | Chưa xác nhận Windows CI |
| 06/15 | CAD+BIM: fingerprint gồm revision của mọi CAD tab + checksum workspace BIM; bỏ qua ghi tự lưu khi không thay đổi | `70a259938af27788f355b2ba43684d2efcae7feb` | Chưa bổ sung kiểm thử hành vi timer | Chưa xác nhận Windows CI |
| 07/15 | Canvas first click/drag; mở CAD theo khối có % đọc; bỏ ngưỡng mở file 512 MiB; hàng đợi mở tuần tự giảm áp lực RAM | `6cc13a944a756bd03b0016c3f75130a630404ed9` | 1 Rust test progressive bytes; GUI tests cần bổ sung | Chưa xác nhận Windows CI |
| 08/15 | Chặn selection drag tồn đọng khi chuột thả ngoài canvas / ESC / focus loss; ETA byte đọc; chuyển khởi động file tiếp sau khi finish_open | `f0cfc0fbf7d50f29f0840df0f89cfd3be457e249` | 2 unit tests bổ sung; test serial queue điều chỉnh | Windows CI/GUI chưa xác nhận PASS |
| 09/15 | Nhận sự kiện release thực tế trên canvas cho lệnh chọn điểm, không lấy nhấp từ toolbar/menu; tránh báo parse ảo khi file 0 byte | `0e97ab7c77072110037a658853deeaa2f8635383` | 2 unit tests bổ sung | Chưa có bằng chứng CI Windows PASS |
| 10/15 | Không lấy các release từ lớp overlay/menu thành điểm CAD; mở nhiều file đúng FIFO ngay cả khi thêm file mới trong lúc finish_open | `0b9297f2a4a093978d02f6e58de373c653cb40b5` | 1 unit test thứ tự FIFO, chưa chạy Windows | Chưa xác nhận Windows CI PASS |
| 11/15 | Giữ đúng tọa độ cho các click nhanh trong một UI frame; dừng nhập điểm khi lệnh đã hoàn tất; chặn click dư thành chọn đối tượng | `4f4ff825fa64b0e7894ee94b48e020a72b5b89cf` | 2 regression tests (tọa độ đa click và chặn click dư) | Chưa có Windows CI PASS |
| 12/15 | Dừng replay điểm khi lệnh chuyển sang prompt không nhận tọa độ; khi tắt PICKDRAG giữa chừng dọn pending window; thêm tests LINE/CIRCLE | `b0c704f75088f8b22d1638054f7a9c4a2ff452a7` | 2 Rust integration-style tests ở tầng engine | Chưa xác minh Windows cargo/GUI PASS |
| 13/15 | Báo giai đoạn decode sau khi đọc file thực sự hoàn thành; cấp phát read-ahead tối đa 8 MiB theo metadata; test trạng thái 100% bytes và file rỗng | `e4447bdb11a87069fc3fcdd23aeb11f7c5108188` | 2 regression tests bổ sung | Chưa chạy Cargo/Windows vì môi trường hiện tại không có Cargo và Git DNS lỗi |
| 14/15 | Kiểm tra quyền xem scope và chia sẻ trên lớp render chính; kiểm tra quyền sửa trước phiên thiết kế/chọn/lease; từ chối lease revision cũ | `9d94bbfe9c44edf502f86a06c16896b2209c7856` | 3 regression tests quyền xem/sửa và lease | Chưa xác minh Windows CI/2-user server PASS |
| 15/15 | Windows workflow kích hoạt trên nhánh active; kiểm thử targeted canvas/open/read; thử đóng gói Setup/Portable candidate sau các gates; sửa test LINE đếm model | `5d442914d5b2131c9820139467bc25357300ff92` | Bổ sung các lệnh Rust kiểm thử tập trung vào CI | Chưa có artifact hay CI Windows PASS được xác minh |
| 16/30 | Không hủy Windows build khi có push mới; ghi BUILD_INFO commit/run/ref; xác minh lại SHA256 cả Setup + Portable; in Actions step summary | `f5d95c61637c64eb448869ca76efca7f81bac99b` | Bổ sung kiểm tra manifest/artefact trong workflow; chưa kiểm tra Windows thực | CI Windows và artifact vẫn chưa có bằng chứng PASS |
| 17/30 | Tạm ngưng phát triển sync/conflict; giữ account permissions; sửa trạng thái hàng đợi OPEN và thêm gates LINE/CIRCLE/ESC trên Windows | `6c8a66fcecc9cdc808e2e616908662eb36d3ad5f` | 1 regression test trạng thái queued; 4 gates mục tiêu bổ sung | Windows test PASS + artifact chưa xác minh |
| 18/30 | Tách thời gian đọc/giải mã CAD, log mỗi file theo ms và hiển thị thời gian giải mã; đồng nhất mốc Instant của worker/UI | `75952a357f31a6dbd553d6e02872cfcdad262cf1` | 1 regression test đồng hồ khi decode bắt đầu; đã bổ sung CI opening::tests ở R15 | Chưa xác minh Cargo/Windows GUI/EXE PASS |
| 19/30 | Git archive nguồn đúng commit Windows; Setup/Portable/Source cùng SHA256; kiểm tra có Cargo.toml, Cargo.lock, installer và workflow | `dacacfd0b15a19110d821c7b55b399abea09fcf1` | Kiểm tra sự tồn tại file trong archive và 3 SHA256 trong CI; chưa chạy Windows | Windows run PASS, EXE và artifact vẫn chưa xác minh |
| 20/30 | Ghép cặp từng sự kiện nhấn/thả chuột trái; chặn toolbar click thành điểm CAD; dọn drag qua menu/palette; bổ sung Windows test | `1ce77e689fb866cca3958c11375d70f208e18245` | 3 tests press-release gesture; thêm CI targeted test | Chưa xác minh Windows CI hoặc GUI PASS |
| 21/30 | Không xử lý cú thả kéo chọn thành idle click; dùng press-origin một lần; ESC/focus ưu tiên hủy; Windows tests phân xử click/drag | `501ee9f06799035a2bab5ea93cafa989d37342bc` | 5 regression tests mới về release/drag/cancel | Chưa có Windows CI/GUI PASS được xác minh |
| 22/30 | Chuyển viewport chỉ với cặp nhấn/thả trong đúng vùng đích và không thuộc menu nổi; test chặn drag từ viewport/toolbar khác | `8c6028f5e2792f86e9a99a14237a7bb42ee01357` | 2 test bổ sung trong primary_gesture_tests (Windows CI đã gọi) | Chưa có Windows CI/GUI PASS được xác minh |
| 23/30 | Đọc file CAD trực tiếp vào Vec theo khối không sao chép qua scratch 1 MiB; kích thước block cuối theo metadata; kiểm thử đuôi file | `e1fad57607404d3a271f9ff9f9fa9946d2722c56` | 1 test phần đuôi và toàn vẹn dữ liệu; CI capped_read::tests đã cấu hình | Chưa có Cargo/Windows benchmark PASS được xác minh |

### Trạng thái
- Hoàn thành bằng chứng mã nguồn: **16 lượt mã nguồn (15 tối thiểu + 1 dự phòng)**.
- Đã đến mốc **15/15** lượt phát triển tối thiểu. Bộ cài Setup/Portable còn **chưa phát hành**, vì thiếu bằng chứng Windows CI PASS và artifact.
- Source inventory (run 41561782723) và IFC metadata test/source snapshot (run 41561782535) PASS. Windows Setup/Portable được skip đúng lịch 15–30. Rust tests CAD+BIM của lượt 02 đã PASS (run 41563233121). Hai regression tests lượt 03 đang chờ xác nhận. Chưa có EXE phát hành của đợt này.

### Rà soát khả năng phục hồi
- Lõi CAD đã có autosave và Drawing Recovery riêng trước đợt này. Lượt 02 thêm một cặp snapshot DXF (hình học bản vẽ đang hoạt động) + workspace BIM JSON + checksum, được lưu theo chu kỳ 5 phút và khi thoát.
- Phục hồi theo lệnh người dùng, mở bản vẽ CAD dạng bản sao; không tự ghi đè file thiết kế. Phần BIM phải được xác thực trước khi mở bản vẽ.
- Giới hạn hiện tại: chưa sao lưu đồng thời tất cả tab bản vẽ, chưa test khôi phục GUI end-to-end trên Windows và chưa kiểm thử mất điện phần cứng.
- Không đóng gói EXE trước mốc 15–30 lượt.

### Lượt 03 — phòng mất bản phục hồi và từ chối DXF lỗi
- `prune_generations` hiện chỉ đếm các snapshot xác minh hợp lệ vào hạn mức 3 bản; thư mục hỏng không được phép đẩy bản phục hồi tốt ra khỏi danh sách.
- Trước lệnh `recover`, đọc và giải mã thực tế DXF bằng `cadcraft_io::read`. Dữ liệu checksum đúng nhưng DXF sai định dạng sẽ bị từ chối trước khi chạm trạng thái CAD đang mở.
- Phần bổ sung: hai test về backup tốt bên cạnh nhiều bản lỗi, và giải mã DXF sai.
- Vẫn còn giới hạn nhiều bản vẽ, snapshot đồng bộ server và kiểm thử UI/mất điện thật.

### Lượt 04 — snapshot nhiều bản vẽ CAD+BIM
- Snapshot mới format v2 lưu riêng từng tab `drawing-0000.dxf`, `drawing-0001.dxf`... và danh sách UID/revision, chỉ số tab đang hoạt động, toàn bộ workspace BIM.
- Tổng hình học CAD tối đa 512 MiB và số bản vẽ tối đa 256 khi xác minh; mỗi file có checksum, kiểm tra độc lập. Snapshot cũ format v1 vẫn đọc được.
- Trước khi phục hồi, kiểm tra toàn bộ checksum, DXF và workspace; các tab mở dạng bản sao, tab active mở cuối. Nếu mở tab bị lỗi giữa chừng, vẫn có rủi ro phục hồi CAD một phần; cần xử lý rollback giao dịch và kiểm thử GUI thực tế ở lượt kế tiếp.
- Các commit lượt 04 đã đẩy; **chưa xác nhận Windows cargo check/test PASS và chưa có EXE**.

### Lượt 05 — hạn chế phục hồi dở và kiểm tra giới hạn
- Nếu lệnh phục hồi nhiều tab trả lỗi, đóng các tab CAD được thêm vào trong lượt phục hồi và trả lại chỉ số tab ban đầu. Workspace BIM chỉ thay sau khi tất cả lệnh `recover` trả thành công.
- Chặn lưu quá 256 tab hoặc workspace BIM vượt 64 MiB trước khi tạo thư mục snapshot.
- Bổ sung test cho active index vượt giới hạn; chưa có xác nhận build/test trên Windows. Rollback hiện chỉ bảo vệ danh sách tab và selection, chưa chứng minh hoàn nguyên toàn bộ side effect nội bộ của lệnh `recover`.
- **Tiến độ 5/15 = 33,3% là tỷ lệ lượt tối thiểu đến mốc đóng gói, không phải độ hoàn thiện sản phẩm**.

### Lượt 06 — phát hiện thay đổi đủ các tab và dữ liệu BIM
- Mỗi chu kỳ 5 phút, fingerprint gồm danh sách (uid, revision) của tất cả tab, chỉ số tab active và checksum JSON workspace BIM; nếu giống snapshot trước thì không tạo thêm snapshot chỉ vì hết thời gian.
- Sau khi phục hồi, fingerprint đặt lại để lần lưu tiếp theo bắt trạng thái mới. Sau lưu thủ công, fingerprint cập nhật.
- Bổ sung mã nguồn: `70a259938af27788f355b2ba43684d2efcae7feb`. Chưa xác nhận build/test trên Windows; không tính là hoàn thành nghiệm thu.
- Theo quy ước lượt, **6/15 = 40% quãng đường đến mốc đóng gói tối thiểu**. Không suy ra 40% hoàn thiện sản phẩm.

### Lượt 07 — input CAD & mở dữ liệu lớn
- Canvas: bỏ điều kiện `Response::hovered()` dễ bỏ sót click đầu tiên khi chuyển focus; vẫn kiểm tra lớp UI trên cùng. Không mở kéo cửa sổ chọn trong lúc lệnh đang nhận tọa độ điểm.
- Desktop OPEN: đường đọc file chuyển sang đọc theo khối 1 MiB, cấp phát `try_reserve`, kiểm tra regular file và phát tiến độ byte đã đọc. Bỏ hard cap 512 MiB ở đường OPEN này, nhưng máy vẫn cần RAM/VRAM; chưa có streaming geometry parser/render.
- UI: tiến độ **% đọc byte thực tế**, thời gian và giai đoạn giải mã riêng (không nói % giả cho giải mã).
- Đọc nhiều file: xếp hàng và chỉ giải mã một file đồng thời; tránh tăng áp lực RAM do nhiều decoder song song.
- Chưa xác minh cargo check, test UI đầu-cuối hay benchmark CAD 300–500 MiB; chưa coi là đạt nghiệm thu.
- **7/15 = 46,7% nhịp phát hành**, không đại diện độ hoàn thiện toàn bộ CAD360.

### Lượt 08 — chặn kẹt drag và ETA đọc file
- Canvas: khi drag-selection còn active, release ra khỏi canvas hoặc mất focus/Escape xóa pending_window và drag_window; release trong canvas tiếp tục xác nhận selection, không làm treo trạng thái.
- Thêm test thuần cho logic hủy drag. Đường OPEN hiển thị ETA cho **đọc byte từ ổ**, không diễn giải đó là ETA tổng cho parse/render.
- Commit: `150ba828c6f61155b00b10a8eaf1b1bce21b15b1`, `ff6ae5d485587a478cefbd74781df2a2ea1c7b2c`, `f8270491c1f49c7852507237633a321756995fc2`.
- Còn thiếu: test GUI click đầu tiên theo mọi lệnh, CI Windows của R07–R08, benchmark file 300–500 MiB, kiểm tra phạm vi user trên server, thiết kế streaming parser/render.
- Tiến độ nhịp: **8/15 = 53,3%**, không đồng nghĩa chất lượng hay độ hoàn thiện tổng thể.

### Cập nhật cuối lượt 08 — thứ tự bàn giao dữ liệu lớn
- Thay đổi `opening.poll()`: không khởi động decoder của file tiếp ngay trong lúc kết quả `ReadFile` của file trước còn đang được xử lý.
- `CadApp.finish_opening()` hoàn thành tích hợp kết quả file trước vào `Session` rồi mới gọi `opening.start_next_after_install()` để giảm đỉnh RAM khi mở hàng loạt.
- Commit bổ sung: `af9d308c9e527ec384ac9dbeece3132489e5a7d5`, `f0cfc0fbf7d50f29f0840df0f89cfd3be457e249`.

### Lượt 09 — click đầu tiên và trạng thái đọc file
- Với lệnh đang chờ tọa độ điểm, canvas bổ sung nhận sự kiện release chuột từ input thực ngay cả khi egui không phân loại thành click (di chuyển vài pixel, chuyển focus từ toolbar). Chỉ nhận nếu press bắt đầu trong canvas, tránh nuốt thao tác menu/toolbar. Bổ sung test kiểm tra điều kiện.
- File đang tải kích thước 0 byte không bị chuyển giai đoạn sang parse chỉ vì `bytes>=total`. Bổ sung regression test.
- Chưa kiểm thử GUI Windows trên LINE/PLINE/CIRCLE/MOVE, không được tuyên bố lỗi click đã xử lý triệt để.
- Repo commit: `2ec2ec4e1fc439c1e5231ce029ee2ad0e85a15ba`, `5c1ac6a235a958ffc522fccaf31c17d0b7f459ce`, `0e97ab7c77072110037a658853deeaa2f8635383`.
- Nhịp phát hành 9/15=60%; tỷ lệ này không đại diện nghiệm thu sản phẩm.

### Lượt 10 — lọc click lớp nổi và giữ thứ tự mở file
- Canvas loại các sự kiện thả chuột trong lớp giao diện nổi khỏi chuỗi nhập nhiều điểm CAD, đồng thời không cho menu tương tác đi vào điểm lệnh.
- Khi load CAD tuần tự, yêu cầu mở mới trong khoảng thời gian worker trước hoàn thành nhưng bản vẽ chưa tích hợp **không được vượt hàng đợi cũ**. Thêm test FIFO tái hiện trường hợp này.
- Mã nguồn: `3d6e8ce24b4cf5f0c15ed6fec53a789c5ce91693`, `0b9297f2a4a093978d02f6e58de373c653cb40b5`.
- Chưa chạy Rust tests GUI trên Windows, chưa có benchmark 300–500 MiB, chưa đóng gói EXE. **10/15 = 66,7% nhịp phát hành**, không phải mức độ hoàn thiện CAD360.

### Lượt 11 — xử lý lệnh CAD nhiều sự kiện nhấp trong một frame
- Khi nhiều sự kiện release đến trong một lần render, các sự kiện trước cuối cùng dùng tọa độ thực của riêng mình thay vì snap đang gắn với con trỏ cuối; tránh dồn các điểm LINE/PLINE về cùng vị trí.
- Khi một lệnh đã kết thúc ở release trước, ngừng nhập thêm và không để release cuối biến thành thao tác chọn khi idle.
- Đã thêm 2 unit tests để xác minh helper xử lý click dư và tọa độ màn hình khác nhau, **không phải GUI E2E**.
- Commits `d11a845170ce0ca9bac5211a357430f1cadc17c6`, `be5b3f6c2e9b43a992a73355a5465caa17252d07`, `cecce2a2d9e5cd5674daf1f97e8edf174945b762`, `4f4ff825fa64b0e7894ee94b48e020a72b5b89cf`.
- Nhịp phát hành **11/15 = 73,3%**. Chưa có bằng chứng thử GUI trên Windows và EXE mới.

### Lượt 12 — chống nhận điểm nhầm sau khi đổi prompt và hủy kéo chọn
- Khi lệnh tương tác chuyển prompt từ chọn tọa độ sang kiểu khác (lựa chọn đối tượng, keyword...), canvas ngừng phát các sự kiện thả chuột còn lại trong cùng frame; tránh nhận sai dữ liệu hoặc chọn đối tượng ngoài ý muốn.
- Khi cài đặt PICKDRAG bị tắt giữa thao tác kéo chọn, xóa đúng `pending_window` do drag tạo ra, không ảnh hưởng thao tác chọn cửa sổ click-click độc lập.
- Thêm test LINE và CIRCLE xác nhận prompt tọa độ đầu tiên được tiếp nhận ở lớp Session. Test đã được viết, **chưa chạy Windows PASS**.
- Commits `92003c26a1d9667cb9c819e5ba3893018543c0e6`, `84a852ffb93c38f72e52e06e5dba7d46d9bfeeae`, `b0c704f75088f8b22d1638054f7a9c4a2ff452a7`.
- Nhịp phát hành tối thiểu 12/15 = 80%, nhưng tiến độ hạng mục chức năng vẫn giữ 37,2% (ước lượng chưa nghiệm thu). Còn 3 lượt đến R15 và tối đa 18 lượt nếu dùng toàn bộ dự phòng đến R30.

### Lượt 13 — phân pha đọc/giải mã chính xác và cấp phát đầu vào
- Đường OPEN desktop có callback riêng cho thời điểm thật sự bắt đầu giải mã, thay vì UI suy ra từ số byte đọc = kích thước file. Giữ % chỉ ở giai đoạn đọc byte; parse/render vẫn chưa có % thật.
- Đường đọc theo khối cấp phát ban đầu bằng `try_reserve` với hint tối đa 8 MiB để giảm cấp phát lại nhưng không chiếm trước cả file 300–500 MiB. Không đặt trần kích thước đầu vào.
- Thêm regression test giai đoạn metadata boundary và file rỗng.
- Commits: `5670d1c36a88f31607ae0ecb1d9aaee5ab6d2813`, `7abb7f30516ab600142fbb116d8146f0315052d6`, `e4447bdb11a87069fc3fcdd23aeb11f7c5108188`.
- Giới hạn: môi trường shell hiện tại thiếu Cargo, git remote không thể resolve DNS GitHub, không có benchmark thực và không thể xác nhận Windows CI PASS.
- Nhịp phát hành: 13/15=86,7%, chỉ nói số lượt; còn 2 lượt đến mốc tối thiểu. Phát hành EXE chỉ khi có gate Windows thực tế.

### Lượt 14 — quyền BIM theo tài khoản, tầng/bộ môn và khóa đối tượng
- `can_edit_active_view`: điều kiện đúng cả phiên đăng nhập, `UserAccess.can_edit` và `AccessRule` theo scope (không chỉ rule). `enforce_selection_scope`: không cho chọn khi hết quyền đăng nhập hoặc bị tắt edit.
- `can_view` của chính chủ: không bỏ qua quyền `AccessRule.can_view`; `sync_render_filters` kiểm tra quyền ngay cả ở `primary_handles` và không đưa đối tượng người khác vào tab nếu `show_other_users=false`.
- `acquire_lease`: không cấp khóa cho người read-only; khóa có sẵn của cùng tài khoản phải khớp revision, tránh chấp nhận revision cũ.
- 3 bài test bổ sung (thu hồi quyền xem, tắt quyền sửa, read-only/stale lease). Đã commit, chưa có chứng cứ Cargo/Windows PASS.
- Chỉ bảo vệ desktop/local BIM, KHÔNG phải enforcement ở server cho mọi mutation; UI đồng bộ còn mô phỏng.
- Commits `9ba1c2f46dfeb7f8815dfb7e80d09374adabf28b`, `7ad3221ddd197949850aae90f4058f6da7762ceb`, `9d94bbfe9c44edf502f86a06c16896b2209c7856`.
- Lượt **14/15 = 93,3% nhịp phát triển tối thiểu**; còn 1 lượt đến ngưỡng cân nhắc đóng gói. Không coi là phần mềm hoàn thiện 93,3%.

### Lượt 15 — phát hành ứng viên trên nhánh riêng sau Windows CI gate
- Phát hiện `.github/workflows/main.yml` ban đầu chỉ chạy khi `push main`, do đó các commit trên `tbt/cad360-active-release` không tự khởi động Windows CI. Sửa branch triggers để nhánh active chạy build/test đầy đủ.
- Job `regression-tests`: thêm các phép kiểm tra `canvas::point_batch_tests`, `canvas::point_release_tests`, `opening::tests`, `capped_read::tests`. Lệnh PowerShell kiểm tra LASTEXITCODE để báo fail tức thì ở mỗi test.
- Job `release-windows`: nhánh active có thể tạo **Candidate** Setup/Portable chỉ sau khi toàn bộ build/test jobs `needs` PASS. Main vẫn đóng gói `Release`. Tên artifacts phân biệt Candidate/Release và kèm SHA256.
- Sửa test LINE dùng `session.doc().unwrap().model.iter().count()` thay `entity_count()` chưa được xác minh.
- Commit: `bae62df9683cadf7cdbf0cf6337221559e8210e3`, `5d442914d5b2131c9820139467bc25357300ff92`.
- **15/15 = 100% số lượt phát triển tối thiểu, KHÔNG phải 100% hoàn thiện.** Chưa có artifact được trả về; chưa xác minh build/cargo test Windows mới, GUI 2 người hay benchmark 300–500 MiB. Không gọi candidate là release chính thức.
- Nếu workflow PASS và có artifact: tải EXE Setup/Portable từ GitHub Actions rồi kiểm thử trên Windows thực. Nếu fail: R16 phân nhóm root cause, sửa từng nhóm cho đến khi PASS; mốc 15 lượt không phải lời hứa EXE đã sẵn sàng.

### Lượt 16 — giữ nguyên Windows CI run và chứng thực artifacts
- Sửa concurrency group theo `github.ref + github.sha`, `cancel-in-progress: false`, tránh job dài bị hủy khi commit mới lên nhánh active.
- Sau build và SHA256, workflow xuất `BUILD_INFO.txt` ghi `GITHUB_SHA`, ref, RunId, RunAttempt và trạng thái `UNVERIFIED_GUI_CANDIDATE` — không tuyên bố nghiệm thu giao diện.
- Kiểm tra 2 file Setup/Portable thực sự tồn tại và hash của từng file trùng đúng một dòng trong `SHA256SUMS.txt`; ghi Actions summary giúp kiểm tra kết quả theo run.
- SHA `f5d95c61637c64eb448869ca76efca7f81bac99b`. Chưa có EXE mới verified; chưa có Cargo/Windows GUI/benchmark PASS qua các công cụ hiện tại.
- **16/30 lượt kế hoạch tối đa; nhưng số lượt không thể thay thế build/test hay artifact thực**. Chuyển R17 sang phân tích lỗi CI từ log thật và loại bỏ blocker Windows.

### Lượt 17 — tập trung hoàn thiện các lỗi CAD và đóng gói, tạm dừng sync/conflict
- Chấp nhận chỉ đạo mới: dừng phát triển đồng bộ và xử lý xung đột; giữ scope đăng nhập/tài khoản/phân quyền. Không làm thêm clash theo bộ môn.
- Cập nhật `opening.rs`: khi file đầu đã giải mã nhưng còn các file xếp hàng chờ UI cài đặt, thanh hiển thị không biến mất; hiển thị file tiếp theo và số file còn hàng đợi. Thêm regression test cho khoảng trống này.
- Cập nhật Windows CI: test có tên chính xác cho LINE nhận điểm đầu tiên, CIRCLE nhận tâm, hủy kéo chọn ESC/focus, các test OPEN và file lớn. Các gates mới chưa có bằng chứng Windows PASS.
- Commits: `0224fa77d98b3f046bd8be3846189e0b609c3e2e`, `6c8a66fcecc9cdc808e2e616908662eb36d3ad5f`.
- Tiến độ hoạt động = 39,4% dự kiến, so với 37,2% cũ **chỉ do thay đổi phạm vi tính (bỏ C2)**; không phải 2,2 điểm phần trăm chức năng được hoàn thành mới.
- Trạng thái build: CI Windows và EXE Candidate chưa xác nhận có artifact. R18 ưu tiên log lỗi và khóa release gate, tiếp tục xử lý mouse/open/performance.

### Lượt 18 — thống kê thời gian đọc/giải mã CAD lớn (đang ưu tiên)
- Trong `opening.rs`, lưu mốc `read_done_ms` chính xác khi callback bắt đầu decode được gọi sau khi đọc xong thực sự; không đánh đồng 100% byte với 100% hoàn thành CAD.
- Thêm log `CAD OPEN` ghi đường dẫn, read_ms, decode_ms, total_ms và OK/ERROR cho mỗi file, giúp xác định bottleneck đọc IO hay giải mã. UI đang giải mã hiển thị cả thời gian đọc và giây decode đã qua.
- Dùng chung `Instant started` cho worker và `Job` để tránh lệch bộ đếm giữa hai luồng; thêm test timer.
- Commits `6c01c27b86fd231f0665f2a3d4b42553c5b1397c`, `75952a357f31a6dbd553d6e02872cfcdad262cf1`.
- R18 không tiếp tục phát triển đồng bộ/xung đột; vẫn giữ lại phân quyền tài khoản và kiểm tra quyền xem/sửa.
- **18/30 lượt tối đa; 12 lượt còn lại dự phòng**. Ước lượng 8 đầu việc vẫn 39,4% chưa nghiệm thu, do chưa có kết quả Windows PASS/benchmark/EXE thực.

### Lượt 19 — bộ 3 artifacts đi cùng một SHA nguồn (đang chờ CI thật)
- `.github/workflows/main.yml`: sau khi build Windows và kiểm thử, tạo `CAD360_VietNam_TBT_Source.zip` từ `git archive HEAD` đúng commit run, phân biệt Candidate/Release như Setup và Portable.
- Có kiểm tra `git rev-parse HEAD == GITHUB_SHA`, kiểm tra trong ZIP có `SOURCE_CAD_R65/Cargo.toml`, `SOURCE_CAD_R65/Cargo.lock`, `installer/CAD360_TBT_Windows.iss`, workflow build; bước bắt lỗi sẽ làm job thất bại.
- `SHA256SUMS.txt` giờ gồm ba dòng (Setup.exe, Portable.zip, Source.zip); job tái kiểm chứng ba hash trước khi publish. Artifact ZIP nguồn nằm cùng run với hai artifact Windows.
- Commits `13aa97201386390ff5662a4806127bdb5a2df0cf`, `dacacfd0b15a19110d821c7b55b399abea09fcf1`.
- Sự có mặt của source archive trong workflow **không phải bằng chứng đã có file**. Chưa có Windows CI PASS và file tải về được xác minh; không nâng % tính năng, vẫn 39,4% ước lượng trên 8 đầu việc đang hoạt động.
- Sync/conflict và clash bộ môn vẫn PAUSED. Phân quyền tài khoản tiếp tục ACTIVE.

### Lượt 20 — khắc phục sự kiện chuột đầu tiên và kéo chọn
- `canvas.rs`: thêm `primary_canvas_releases` ghép từng cặp nhấn và thả Primary riêng biệt, chỉ nhận sự kiện khi cả điểm nhấn và thả thuộc canvas; dùng `pointer.press_origin` dự phòng nếu nhấn ở frame trước. Tránh trường hợp kéo từ toolbar/menu vào canvas làm phát sinh điểm vẽ không đúng.
- Lệnh đang yêu cầu tọa độ chỉ lấy click canvas hợp lệ (raw paired releases hoặc `resp.clicked_by`) thay vì sử dụng click toàn cục trên UI; giữ đường thao tác menu Dynamic Input.
- Hủy kéo chọn khi thả trên vùng giao diện nổi/palette và không khởi tạo vùng chọn từ menu; thêm điều kiện ESC/focus.
- Viết 3 tests cho 2 cú click liên tiếp trong frame, press toolbar rồi thả lên canvas, và release khi press frame trước. CI Windows được bổ sung `canvas::primary_gesture_tests`.
- Commits `c699be8066b6597d47bc6633a4443be15fc0b23d`, `232b0e366fb77e287087485c533dcc6be82252e4`, `7c8d811e7a0330c0985bd1e35db6e302b4c832df`, `1ce77e689fb866cca3958c11375d70f208e18245`.
- **Đây là sửa mã nguồn và viết test; chưa xác minh Rust tests chạy PASS hoặc GUI trên Windows.** Giữ 39,4% ước lượng trên 8 nhóm active, không tự tăng vì chưa có QA.
- Lượt **20/30**, còn 10 lượt trong kế hoạch. Đồng bộ/xung đột bộ môn và giữa tài khoản đang tạm dừng; phân quyền tài khoản vẫn active.

### Lượt 21 — click-click, drag-stop và ESC/focus
- Sửa `canvas.rs`: khi `drag_window` còn hoạt động hoặc UI báo `drag_stopped`, không đưa sự kiện cuối vào `cycle_click` hay `idle_click` trước khi kết thúc vùng chọn. Nếu kết thúc kéo nhưng không có vị trí thả hợp lệ, xóa `pending_window` thay vì để điểm góc đầu tiên tồn tại sang thao tác kế.
- `primary_canvas_releases` dùng `press_origin` từ frame trước **tối đa một lần**; sự kiện press mới phủ nhận fallback cũ. 2 tests bổ sung.
- ESC hoặc mất focus hủy đường xử lý click và cycle trước khi tương tác trên canvas; vẫn để đường dọn kéo chọn xử lý trạng thái tồn đọng.
- Hàm `may_dispatch_canvas_click` cho 3 test phân biệt click-click hợp lệ, kết thúc kéo và ESC/focus. Các tests thêm vào Windows CI targeted gate.
- Commits `194e912a099d5c58e4d81b2fc2be60f226acf897`, `c7f83d794d86e83362c07231615b127f1fd6a497`, `e5ca33d26475bf5f9cd44774d2139f50e59a0811`, `84d48eec47ff4791d9f2c391c98e752fb3cb18b0`, `501ee9f06799035a2bab5ea93cafa989d37342bc`.
- Sync/conflict và clash vẫn PAUSED, tài khoản/phân quyền ACTIVE. Mốc 21/30 lượt, còn 9 lượt trong kế hoạch. Tiến độ kỹ thuật 8 nhóm vẫn 39,4% do chưa có Windows GUI, benchmark và EXE PASS thực.

### Lượt 22 — ngăn cú click ngoài viewport kích hoạt sai cửa sổ CAD
- `canvas.rs` chuyển từ điều kiện `pressed_primary` toàn cục + `hover_pos` sang xác minh một cặp sự kiện nhấn/thả của **đúng viewport đích** bằng `primary_canvas_releases(&input.events, target_rect, earlier_press)`.
- Điều kiện lớp vẽ trên cùng `layer_id_at` và `over_canvas_menu` ngăn click trên menu/palette bị tính là chuyển viewport. Không gửi click xuyên qua từ viewport 1 sang viewport 2.
- Thêm 2 unit tests: nhấn tile trái rồi thả tile phải không chuyển, nhấn toolbar rồi thả tile đích không chuyển. Vì dùng cùng module `primary_gesture_tests` nên Windows CI R20 đã gọi tự động.
- Commits `bde137eac01c4d3e91fa0cd7c24e8eecdaaebfa6`, `8c6028f5e2792f86e9a99a14237a7bb42ee01357`.
- Không làm thêm đồng bộ/xung đột; chỉ giữ quyền tài khoản/scope. Mốc 22/30 lượt phát triển (còn 8); 8 hạng mục hoạt động vẫn 39,4% **ước lượng chưa nghiệm thu**. Không xác nhận chạy Windows hay tạo EXE thành công.

### Lượt 23 — giảm thao tác sao chép khi đọc CAD lớn
- `capped_read.rs` thay thế bộ đệm trung gian `chunk = vec![0u8; 1MiB]` + `output.extend_from_slice(&chunk[..n])` bằng đường `Vec::try_reserve`, `resize`, đọc thẳng qua `file.read(&mut output[old_len..])` và `truncate` theo số byte thực. Bỏ một lần sao chép byte lớn ở mỗi khối, không khẳng định giảm toàn bộ RAM/giải mã.
- Khi metadata cho biết chỉ còn khối cuối vài byte, `read_size` theo số byte còn lại, không zero-init cả 1MiB; vẫn thăm dò sau metadata để không bỏ qua file tăng kích thước.
- Bảo toàn lỗi trả về khi không đủ RAM/IO, không áp trần 512MiB cho file OPEN; giữ nguyên callback tiến độ. Có test dữ liệu mẫu 2MiB+17 byte bảo đảm không thêm byte 0 hoặc mất tail.
- Commits `09a3cfa874ec2e376c7d734b2923c4826e806cf3`, `e1fad57607404d3a271f9ff9f9fa9946d2722c56`.
- Chưa có Cargo trên môi trường thao tác, chưa có CI Windows PASS hoặc benchmark file 300/500MiB. Tiến độ 8 nhóm vẫn 39,4% ước lượng chưa nghiệm thu. Lượt 23/30, còn 7 lượt dự kiến; nhóm sync/conflict/clash PAUSED, quyền tài khoản ACTIVE.
