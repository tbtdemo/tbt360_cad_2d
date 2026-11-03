# CAD360 TBT — Theo dõi đợt nâng cấp 15–30 lượt

## Quy ước
- Một lượt = một nhóm thay đổi chức năng đã được commit và có bằng chứng mã nguồn. Mỗi lượt không mặc định tương đương một commit.
- Mốc đóng gói định kỳ: khi đã hoàn thành ít nhất 15 lượt và không quá 30 lượt; một đợt đóng gói Windows Setup, Portable và mã nguồn sau khi vượt qua các gate.
- Mọi thay đổi vẫn có mã nguồn sao lưu theo commit. Không tính job `skipped` là EXE PASS.
- Chưa kích hoạt đóng gói cho đợt này.

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

### Trạng thái
- Hoàn thành bằng chứng mã nguồn: **12/15** lượt tối thiểu.
- Còn ít nhất **3** lượt trước khi đóng gói định kỳ.
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
