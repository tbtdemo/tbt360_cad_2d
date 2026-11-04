# CAD 360 Việt Nam V1.025 — Biên bản rà soát kỹ thuật và checklist nghiệm thu

**Mốc kiểm chứng:** 2026-11-05 (giờ Việt Nam)  
**Nhánh nguồn:** `feature/CADTBT360_V1.025_xref-regression`  
**Commit chính xác của bộ cài:** `0634a0ec56e47c2d61e659d7a3ff0d99ae0722ed`  
**Windows CI:** [#87](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41882766983) — `success`  
**GitHub prerelease:** [CADTBT360_V1.025_B87_A1](https://github.com/tbtdemo/tbt360_cad_2d/releases/tag/CADTBT360_V1.025_B87_A1)

> Đây là checklist nghiệm thu, không phải biên bản nghiệm thu GUI đã ký. `BUILD_INFO.txt` của bản này ghi `GUIAcceptance=NOT_VERIFIED`. Không thay đổi nhãn đó khi chưa có bằng chứng chạy GUI trên máy Windows thực tế.

## 1. Tệp đã phát hành và định danh

| Asset | Kích thước (byte) | SHA-256 (từ metadata GitHub Release) |
|---|---:|---|
| `CADTBT360_V1.025_Setup_Windows_x64.exe` | 25,544,920 | `72165e84f4266e49e0dd18397c77b8b528e94ecc40ccd8b3c67e526d8db1c7ae` |
| `CADTBT360_V1.025_Portable_Windows_x64.zip` | 37,348,126 | `9eedc1e568f4a739948716e87c794778102cae9fb3c97a419d07ccf3a63e1737` |
| `BUILD_INFO.txt` | 153 | `c3fe53f5b513a147af7d4f4a2d613666ecfd9dc26068ba84408528d58f323cfb` |
| `SHA256SUMS.txt` | 215 | `b323596817e55a8fb3a153607a7ee54810b2e1661ab4c1322c50e1e4b063a329` |

GitHub Release đang ở trạng thái **prerelease**. Dữ liệu trên là thông tin về asset đã tải lên; người nghiệm thu nên tính SHA-256 của file tải về để đối chiếu.

## 2. Những gì Windows CI #87 đã xác minh — PASS

- [x] Phát hiện đủ các regression test bắt buộc trước khi chạy.
- [x] Kiểm thử hợp đồng JSON IMAGE/LWPOLYLINE tại CLI.
- [x] Kiểm thử image-to-CAD, MCP và bộ dựng ảnh raster trong mã nguồn.
- [x] Kiểm thử Xref và DWG trên mã nguồn.
- [x] Build native Windows GUI + CLI release.
- [x] Tạo bộ cài bằng Inno Setup và gói Portable ZIP.
- [x] Cài đặt im lặng trên Windows runner; kiểm tra sự tồn tại và SHA-256 các EXE sau cài đặt.
- [x] Kiểm tra PE header: GUI x64 Windows subsystem và CLI x64 console subsystem.
- [x] Kiểm thử CLI đã cài: tạo DXF, lưu DWG R2018 (`AC1032`), mở lại và đối chiếu entity.
- [x] Kiểm thử ảnh PNG liên kết và đường polyline có thể chỉnh sửa sau khi lưu/mở lại DXF.
- [x] Kiểm thử gỡ cài đặt bằng uninstaller.
- [x] Giải nén Portable; đối chiếu binary với bản build, lưu/mở lại DXF và DWG.
- [x] Upload artifact, publish GitHub prerelease và xác minh tên + kích thước 4 asset.

**Giới hạn chứng cứ:** Những kiểm thử trên không đủ để kết luận giao diện đã được chạy/nghiệm thu thủ công, AI tự suy luận ảnh thành hình học một cách tự động, hoặc tương thích hai chiều với mọi phiên bản AutoCAD.

## 3. Kiểm thử GUI thủ công trên Windows — CHƯA XÁC MINH

Mỗi mục phải có **kết quả**, **máy/OS**, **ngày thử**, **ảnh/video chứng cứ**, **file DXF/DWG test**, **người nghiệm thu**. Không đánh dấu PASS khi chỉ có kiểm thử CLI.

- [ ] Mở `CADTBT360.exe` từ Setup bằng người dùng Windows thông thường, không cần quyền quản trị khi vận hành.
- [ ] Mở `CADTBT360.exe` từ Portable ở thư mục có dấu cách và tên tiếng Việt.
- [ ] CAD Free: mở ứng dụng không đăng nhập; tạo bản vẽ; LINE, PLINE, CIRCLE, ARC; chọn, xóa, Undo/Redo; lưu bản vẽ.
- [ ] Giao diện Windows: ribbon/menu, biểu tượng, font tiếng Việt, DPI 100% / 125% / 150%, resize/maximize/minimize, dialog mở/lưu.
- [ ] PAN/ZOOM, OSNAP, layer và thuộc tính đối tượng; thay đổi và lưu lại.
- [ ] Đính kèm PNG/JPG, di chuyển/đổi tỷ lệ/ẩn hiện ảnh; IMAGECLIP và lưu/mở lại.
- [ ] Xref có đường dẫn tương đối, broken link, reload/unload; bảo toàn đường dẫn sau đóng/mở tệp.
- [ ] Lưu `.DWG` và mở/chỉnh sửa/lưu lại trong AutoCAD thực tế; sau đó mở lại bằng CAD 360. Đối chiếu hình học, layer, đơn vị, đường nét, text và vị trí ảnh/xref.
- [ ] Đo bóc: chọn theo vùng, bộ lọc đối tượng, đơn vị, nhóm hiển thị và xuất Excel (nếu tính năng có trong bản này).
- [ ] Chế độ Offline không đồng bộ Cloud; xác minh `.DWG` vẫn là DWG chuẩn, không mã hóa định dạng.
- [ ] Kiểm tra chính sách đăng nhập/giấy phép khi Export theo đúng yêu cầu sản phẩm; tách rõ Save và Export nếu phần mềm có quy ước riêng.
- [ ] Khởi động/gỡ cài đặt sạch, cài đè bản trước, giữ file người dùng và setting theo chính sách được duyệt.
- [ ] Chụp màn hình đúng giao diện *của executable phát hành này*, ghi rõ commit và cách tái hiện.

## 4. Những khả năng chưa được CI chứng minh

- **Automatic image recognition / AI auto tracing:** smoke test sử dụng các điểm truy vết đã cung cấp; chưa chứng minh mô hình AI tự phát hiện mọi cạnh/cấu kiện từ ảnh.
- **Tương thích AutoCAD thực tế:** kiểm thử DXF↔DWG và tự mở lại của CAD 360 là cần thiết nhưng chưa đủ. Cần kiểm thử với phần mềm CAD bên ngoài.
- **GUI:** kiểm tra PE header và kiểm thử CLI không thay thế click/run trên giao diện.
- **Hệ sinh thái MCP/CLI qua máy cài đặt thực:** có test source-level MCP và test installed CLI; cần thêm bài test end-to-end của agent trên Windows khi nghiệm thu chức năng đó.

## 5. Tiêu chí chốt phát hành chính thức

Chỉ chuyển prerelease sang phát hành rộng sau khi: (1) thực thi checklist GUI với bằng chứng; (2) kiểm tra DWG qua ứng dụng CAD độc lập; (3) đối chiếu SHA-256 của EXE/ZIP tải xuống; (4) xử lý hoặc chấp thuận bằng văn bản mọi lỗi còn tồn tại; (5) ký xác nhận nghiệm thu.

| Người nghiệm thu | Máy / Windows | Ngày giờ | Kết quả | Bằng chứng |
|---|---|---|---|---|
| Chưa ghi nhận | Chưa ghi nhận | Chưa ghi nhận | CHƯA NGHIỆM THU GUI | Chưa có |

**Trạng thái hiện tại:** CI PASS / BỘ CÀI ĐÃ CÓ / GUI CHƯA NGHIỆM THU / PRERELEASE.
