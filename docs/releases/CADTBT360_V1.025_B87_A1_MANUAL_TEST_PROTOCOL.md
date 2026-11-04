# CAD 360 V1.025 — Đợt 130: Quy trình nghiệm thu thực tế Windows
Ngày lập: 2026-11-05. Đây là **kịch bản nghiệm thu chưa thực thi**, không phải biên bản xác nhận PASS.

## 0. Định danh đối tượng kiểm thử
- Repository: `tbtdemo/tbt360_cad_2d`.
- Tag phát hành: `CADTBT360_V1.025_B87_A1` (prerelease).
- Windows CI: `41882766983` (#87), kết quả: SUCCESS.
- **Commit tạo executable:** `0634a0ec56e47c2d61e659d7a3ff0d99ae0722ed`.
- Commit tài liệu #129 không được dùng để nhận diện executable.
- Asset: `CADTBT360_V1.025_Setup_Windows_x64.exe`, `CADTBT360_V1.025_Portable_Windows_x64.zip`.
- GUIAcceptance: NOT_VERIFIED; external-AutoCAD-roundtrip: NOT_VERIFIED.

## 1. Kiểm tra tính toàn vẹn trước cài
1. Tải 2 asset từ tag Release. Tránh dùng artifact/EXE của phiên bản khác.
2. Trên PowerShell, tính mã băm:
   ```powershell
   Get-FileHash '.\CADTBT360_V1.025_Setup_Windows_x64.exe' -Algorithm SHA256
   Get-FileHash '.\CADTBT360_V1.025_Portable_Windows_x64.zip' -Algorithm SHA256
   ```
3. Đối chiếu lần lượt với `72165e84f4266e49e0dd18397c77b8b528e94ecc40ccd8b3c67e526d8db1c7ae` và `9eedc1e568f4a739948716e87c794778102cae9fb3c97a419d07ccf3a63e1737` từ hồ sơ release #129. Nếu khác: **STOP/FAIL**, không chạy bộ cài.
4. Ghi tên máy, phiên bản Windows, CPU/GPU, độ phân giải, DPI, thời gian thử, người thử, tên tệp, SHA-256 vào phiếu bằng chứng.

## 2. Ma trận nghiệm thu (mọi ca đều cần ảnh/video và dữ liệu trước/sau)
| ID | Môi trường | Các bước thực hiện | Điều kiện PASS |
|---|---|---|---|
| WIN-01 | Windows x64, tài khoản thường | Cài Setup, khởi động từ Start Menu, đóng/mở lại | Không cần Admin khi chạy; khởi động không crash; đúng bản V1.025 |
| WIN-02 | Windows x64 | Giải nén Portable vào thư mục có dấu cách/tiếng Việt, chạy GUI | Không phụ thuộc Setup; tạo được file đầu ra |
| GUI-01 | DPI 100/125/150% | Mở/thu phóng cửa sổ, ribbon, menu, dialog | Không mất nút, tràn chữ, chồng control; font tiếng Việt đúng |
| CAD-01 | Offline, chưa đăng nhập | Tạo bản vẽ LINE/PLINE/CIRCLE/ARC, sửa, undo/redo, lưu/mở | CAD Free sử dụng được, không bắt đăng nhập lúc mở/biên tập |
| CAD-02 | Dự án mẫu 2 layer | Đổi Layer, màu, nét; OSNAP, PAN/ZOOM, lưu/mở lại | Hình học, layer, đơn vị, thuộc tính không đổi |
| IMG-01 | PNG/JPG đường dẫn có Unicode | Attach, move/scale, hide/show, IMAGECLIP, lưu/mở | Ảnh liên kết đúng, không lệch vị trí/kích thước; clip ổn định |
| XRF-01 | DWG/DXF master + tham chiếu | Attach xref relative; relocate master; reload/unload; thử broken path | Đường dẫn tương đối và hành vi reload/unload đúng, cảnh báo mất liên kết dễ hiểu |
| DWG-01 | CAD 360 + AutoCAD độc lập | Vẽ/lưu DWG, mở trong AutoCAD, chỉnh sửa/lưu, mở lại CAD 360 | DWG chuẩn, trao đổi hai chiều; hình học/layer/text/đơn vị/xref nhất quán |
| DWG-02 | Offline | Lưu DWG offline và kiểm tra bằng CAD độc lập | DWG mở/chỉnh sửa được, không mã hóa/chèn wrapper độc quyền |
| QTO-01 | Bản vẽ có nhiều vùng/tầng | Chọn vùng bóc, lọc đối tượng, nhóm & đổi đơn vị, xuất Excel (nếu có) | Bảng chỉ lấy phạm vi đã chọn; tổng lượng đúng; Excel có cột/đơn vị/định dạng rõ |
| LIC-01 | Free/Online/Offline | Mở, vẽ, Save và Export ở các trạng thái cấp phép | Chính sách login chỉ áp dụng đúng thao tác đã quy định; không chặn CAD Free |
| UPD-01 | Máy có bản trước và thiết lập mẫu | Cài đè, chạy, gỡ; kiểm tra settings, tài liệu người dùng | Không xóa/mất user file hoặc setting ngoài chính sách công bố |
| MCP-01 | Bản cài thực tế + agent/client MCP | Agent tạo entity, lưu DXF/DWG, khởi động lại, mở lại và chỉnh sửa | Không có lỗi thiếu IoHooks; entity vẫn sửa được sau roundtrip |

**Lưu ý:** Nếu chức năng chưa có trong đúng build #87, ghi `NOT_IMPLEMENTED` (không chấm PASS, không dựng ảnh giả). Nếu chưa thử, ghi `NOT_RUN`.

## 3. Bộ dữ liệu nghiệm thu tối thiểu
- `basic_geometry.dwg`: LINE/ARC/CIRCLE/LWPOLYLINE, layer, màu, nét, kích thước và text tiếng Việt; có số lượng và tọa độ chuẩn.
- `xref_master.dwg` + `refs/xref_child.dwg`: ít nhất một xref relative.
- `image_unicode.png`, `photo_perspective.jpg`: path có dấu và cách; so sánh clip & scale.
- `qto_two_zones.dwg`: 2 vùng độc lập có số lượng/chiều dài/diện tích có thể kiểm bằng tay.
- `external_roundtrip.dwg`: lưu bên AutoCAD và mở lại CAD 360; ghi phiên bản AutoCAD dùng thử.

## 4. Mẫu ghi kết quả
| Test ID | Kết quả (PASS/FAIL/NOT_RUN/NOT_IMPLEMENTED) | OS/DPI | File đầu vào | File đầu ra | Ảnh/video | Lỗi / issue |
|---|---|---|---|---|---|---|
| WIN-01 | NOT_RUN | — | — | — | — | — |

Bằng chứng nên lưu theo cấu trúc: `evidence/V1.025/B87/{TestID}/{yyyyMMdd-HHmm}/`; gồm `environment.txt`, `steps.md`, ảnh chụp trực tiếp từ ứng dụng, file thử trước/sau, `sha256.txt`. Không lấy ảnh AutoCAD hoặc bản GUI khác làm minh chứng cho GUI CAD 360.

## 5. Điều kiện GO / NO-GO
- **GO cho phát hành rộng** chỉ khi các ca P0 (`WIN-01`, `WIN-02`, `CAD-01`, `DWG-01`, `DWG-02`, `LIC-01`) PASS bằng chứng thực; các ca còn lại có kết quả rõ và không có lỗi blocking; hash đúng; người phụ trách ký xác nhận.
- **NO-GO** nếu GUI chưa chạy thử, DWG chưa mở/sửa/lưu bằng AutoCAD độc lập, hay tồn tại lỗi làm mất dữ liệu bản vẽ; không đổi prerelease thành stable.
- CI #87 PASS là bằng chứng kỹ thuật tự động, **không thay thế nghiệm thu thực tế**.

## 6. Kết quả đợt 130
Đã xác nhận qua GitHub connector: job #87 completed/success (preflight, regression, package, installed smoke và publish đều success); artifact ID `13352798404` chưa expired. **Chưa chạy GUI trên máy Windows người dùng**, chưa kiểm định AutoCAD độc lập, chưa chứng minh thực thi agent MCP bản cài. Giữ `GUIAcceptance=NOT_VERIFIED`.
