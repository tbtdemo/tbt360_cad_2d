# HƯỚNG DẪN SỬ DỤNG CAD 360 V1.025 — AI TỪ ẢNH THÀNH CAD

**Đơn vị:** Công ty CP Công nghệ TBT Việt Nam  
**Áp dụng:** CADTBT360 V1.025, Windows x64 (khi build có chức năng này)  
**Tài liệu:** UG-AI-001 | Ngày cập nhật: 05/11/2026  
**Phạm vi:** Chuyển PNG/JPG thành nét thẳng LINE có thể chỉnh sửa trong bản vẽ CAD. Đây là công cụ vector hóa pixel, **chưa phải AI hiểu tường, cửa, cột, kích thước**.

> **Lưu ý nghiệm thu:** Tài liệu mô tả chức năng trong mã nguồn V1.025. Tại thời điểm soạn, chưa có bằng chứng CI Windows PASS hoặc xác nhận bản EXE mới chạy đạt; không được hiểu đây là biên bản nghiệm thu hay ảnh chụp thực tế.

## 1. Trước khi sử dụng

1. Mở CAD 360 V1.025 trên Windows, mở **một bản sao** của bản vẽ cần làm việc.
2. Chuẩn bị ảnh PNG/JPG/JPEG có nét tối, nền sáng và tương phản rõ. Tệp tối đa **16 MiB**; kích thước tối đa **4096 px mỗi cạnh**, tổng tối đa **12 triệu pixel**.
3. Nếu ảnh chụp nghiêng/phối cảnh, nên hiệu chỉnh thẳng ảnh trước khi vector hóa; công cụ này **không tự hiệu chỉnh phối cảnh**.
4. Xác định tỷ lệ đo thực tế, ví dụ một khoảng **100 pixel** trên ảnh tương ứng **2000 mm** thì nhập **20 đơn vị CAD/pixel**, nếu bản vẽ dùng đơn vị mm.

## 2. Hướng dẫn từng bước bằng giao diện

### Bước 1 — Mở chức năng
Chọn tab **CAD** rồi nhấp **AI TỪ ẢNH → CAD** ở thanh phía trên. Cửa sổ **AI từ ảnh → CAD | TBT 360** mở ra.

### Bước 2 — Chọn ảnh nguồn
Nhấn **Chọn ảnh PNG/JPG…**, chọn đúng ảnh. Tên tệp được hiện cạnh nút. Chọn ảnh khác sẽ xóa bản xem trước cũ.

### Bước 3 — Thiết lập cách nhận dạng
- **Nét tối thiểu (px)**: mặc định trong giao diện **16**; tăng giá trị nếu ảnh nhiều nhiễu, giảm khi đường nét ngắn.
- **Đơn vị CAD/pixel**: mặc định **1,0**. Phải nhập theo tỷ lệ thiết kế; không coi 1 pixel mặc nhiên là 1 mm.
- **Nét 45°**: mặc định bật, bổ sung nhận dạng hai hướng chéo 45°.
- **Góc bất kỳ (thử nghiệm)**: mặc định tắt; bật để thử nhận dạng các nét xiên đơn lẻ, ít giao nhau. Chế độ này có thể bỏ sót hoặc nhận sai những nét phức tạp.
- Công cụ chỉ vector hóa nét tối; chưa đọc chữ, kích thước ghi trên bản vẽ hoặc nhận dạng đối tượng xây dựng theo ngữ nghĩa.

### Bước 4 — Xem trước không thay đổi bản vẽ
Nhấn **1. Nhận dạng và xem trước**. Phần mềm hiển thị số nét phát hiện và vẽ bản xem trước trên nền tối trong cửa sổ AI.
Kiểm tra số nét, hình dạng, chiều hướng và tỷ lệ. Nếu sai, chỉnh cấu hình và nhấn xem trước lại. Đây là **hình xem trước trong hộp thoại**, chưa phải lớp phủ tương tác trên canvas.

### Bước 5 — Tạo LINE chỉnh sửa được
Nhấn **2. Tạo LINE chỉnh sửa trong CAD**. Lệnh chỉ được thực hiện khi:
- Đang mở đúng tài liệu đã tạo Preview và chưa có sửa đổi bản vẽ kể từ khi Preview;
- Không còn lệnh CAD khác đang thực hiện;
- UNDO ở chế độ **Control All** và không có nhóm UNDO khác đang mở.
Khi thành công, các LINE được tạo trong bản vẽ và gom vào một nhóm UNDO. Khi có lỗi, phần mềm thử hoàn tác cả nhóm; hãy đọc cảnh báo và kiểm tra bản vẽ nếu không hoàn tác được.

### Bước 6 — Kiểm tra và lưu
Chuyển về canvas, chọn từng đối tượng để kiểm tra chúng là **LINE** thật, không phải ảnh phủ. Kiểm tra vị trí, tỷ lệ, chiều dài, Layer hiện hành và điểm giao. Lưu một bản **DXF/DWG mới**, đóng và mở lại để xác nhận các đối tượng vẫn chỉnh sửa được.

## 3. Ví dụ thực hành có kiểm chứng

**Ảnh mẫu:** nét ngang từ pixel (10, 10) đến (90, 10), chiều dài 80 pixel.  
**Tỷ lệ:** 10 mm/pixel.  
**Kỳ vọng:** đối tượng LINE có chiều dài xấp xỉ **800 mm** sau vector hóa (tùy vị trí tâm pixel và độ dày nét). Đo lại bằng DIST/Properties; không dùng kích thước dự đoán thay cho kiểm tra CAD thực.

**Bài tập:** Tạo ảnh gồm hai nét ngang tách nhau; chạy Preview; quan sát số lượng, tạo LINE; dùng UNDO một lần để thử hoàn tác cả nhóm; làm lại và lưu DXF, mở lại.

## 4. Xử lý lỗi thường gặp

| Hiện tượng | Nguyên nhân thường gặp | Cách xử lý |
|---|---|---|
| Không phát hiện nét | Ảnh nhạt, nét ngắn, ngưỡng/lọc quá chặt | Chọn ảnh rõ hơn, giảm Nét tối thiểu; xem lại ảnh |
| Phát hiện quá nhiều nét | Chữ, nhiễu, gạch nền bị nhận nhầm | Tăng Nét tối thiểu, cắt vùng ảnh, tắt chế độ góc thử nghiệm |
| Sai kích thước CAD | Sai Đơn vị CAD/pixel | Hiệu chuẩn theo một kích thước đo chuẩn và xem trước lại |
| Báo quá số nét | Quá giới hạn **250 nét** trong giao diện | Cắt nhỏ vùng ảnh, tăng Nét tối thiểu |
| Không tạo được LINE sau Preview | Đã chuyển/chỉnh sửa tài liệu, UNDO không ở All, đang chạy lệnh khác | Kết thúc lệnh, mở đúng bản vẽ, chạy Preview lại |
| Không mở được ảnh | Sai định dạng, ảnh quá lớn hoặc tệp lỗi | Chuyển thành PNG/JPG hợp lệ, giảm kích thước |
| Có LINE nhưng sai ngữ nghĩa | Công cụ chưa phân loại tường/cột/cửa | Kiểm tra thủ công; chưa nghiệm thu nhận dạng cấu kiện |

## 5. Dành cho AI Agent/MCP (nâng cao)

Tên công cụ: `raster_to_cad_lines`. Ví dụ JSON tham số:

```json
{
  "path": "C:\\BanVe\\mat-bang.png",
  "preview": true,
  "minLengthPixels": 16,
  "threshold": 160,
  "unitsPerPixel": 10,
  "maxLines": 250,
  "includeDiagonals": true,
  "includeArbitraryAngles": false
}
```

Đầu tiên dùng `preview: true` để lấy tọa độ dự kiến mà không sửa CAD. Sau khi kiểm tra mới đổi `preview: false`. Chỉ cho AI truy cập file ảnh được phép; không cho phép Agent tự động lưu hoặc ghi đè bản vẽ gốc khi chưa được người dùng xác nhận.

## 6. Checklist nghiệm thu khi phát hành

- [ ] Windows GUI và MCP `cargo check --locked` PASS
- [ ] Test PNG → LINE → DXF → mở lại PASS
- [ ] Test UNDO/rollback và chống ghi nhầm tài liệu PASS
- [ ] Setup EXE cài, mở, gỡ thành công
- [ ] Portable ZIP chứa tệp **HUONG_DAN_AI_TU_ANH_THANH_CAD.md**
- [ ] Setup EXE cài file hướng dẫn vào thư mục `HuongDan`
- [ ] Tệp hướng dẫn có mặt trong danh sách release assets
- [ ] Thử nghiệm ảnh mặt bằng thực tế và kiểm tra thủ công trước khi phát hành ổn định

## 7. Phạm vi chưa hoàn thiện

Chưa tự nhận dạng tường, cửa, cột, dầm, kích thước, Layer theo ngữ nghĩa; chưa tự vector hóa đường cong hoặc các đường giao nhau phức tạp đáng tin cậy; chưa có kiểm thử trực quan Windows GUI được xác nhận PASS. Không sử dụng kết quả vector hóa để thi công hoặc nghiệm thu mà không có người chuyên môn kiểm tra.

---
**Quy tắc phát hành tài liệu TBT:** Mỗi phiên bản/đợt nâng cấp phải cập nhật hướng dẫn thao tác, bảng tính năng đã làm/chưa làm, các trường hợp lỗi và checklist nghiệm thu; file hướng dẫn phải được đóng gói cùng Setup, Portable và công bố cùng phiên bản trên GitHub.


## 8. Bổ sung cho đợt nâng cấp 169 — File Word đi kèm bản phát hành

Từ đợt này, Windows Release tạo tự động **hai định dạng hướng dẫn** từ đúng mã nguồn tài liệu của commit:

- **Word:** `CADTBT360_V1.025_HUONG_DAN_AI_TU_ANH_THANH_CAD.docx`, có thể tải riêng ở GitHub Release.
- **Markdown:** `HUONG_DAN_AI_TU_ANH_THANH_CAD.md`, phiên bản văn bản gốc.

Sau khi cài **Setup EXE**, mở `HuongDan/HUONG_DAN_AI_TU_ANH_THANH_CAD.docx` trong thư mục cài đặt. Với **Portable ZIP**, mở `HUONG_DAN_AI_TU_ANH_THANH_CAD.docx` nằm cùng thư mục với tệp `CADTBT360.exe`. Không dùng tài liệu phiên bản cũ để đối chiếu giao diện mới.

CI có bước kiểm tra tệp Word là gói DOCX hợp lệ, bản Word trong Portable giống bản phát hành theo SHA256, bản Word được cài bởi Setup giống tệp Word phát hành theo SHA256 và cả hai định dạng có trong tài nguyên công bố.

**Chưa xác minh thực tế:** Windows CI PASS, EXE phát hành thành công và ảnh chụp trực tiếp từ GUI Windows. Tài liệu hiện hướng dẫn đúng cách sử dụng theo mã nguồn; sau khi có bản EXE được nghiệm thu cần bổ sung ảnh chụp thực tế từng bước, không sử dụng ảnh mô phỏng thay thế.

**Quy ước lâu dài:** Mỗi đợt nâng cấp chức năng phải cập nhật tài liệu từng bước, ví dụ, cách kiểm tra kết quả, lỗi thường gặp và trạng thái nghiệm thu; khi công bố phát hành phải mang theo bản Word và bản Markdown đồng bộ.


## 9. Nâng cấp đợt 170 — Hướng dẫn ngay trong giao diện CAD 360

**Điểm mới:** Trên tab CAD có thêm nút **HƯỚNG DẪN AI**. Khi nhấn, phần mềm mở cửa sổ **AI từ ảnh → CAD** và bật khung **Hướng dẫn từng bước** mà không cần truy cập Internet.

1. Mở **CAD 360 V1.025** và chuyển đến tab **CAD**.
2. Chọn **HƯỚNG DẪN AI** ở thanh công cụ. Nội dung hướng dẫn 6 bước được hiển thị ngay trong hộp thoại.
3. Đọc quy trình chọn ảnh, tỷ lệ CAD/pixel, Preview và tạo LINE.
4. Khi cần đọc tài liệu dài, với bản cài Setup: vào **Start Menu → CAD 360 Viet Nam - TBT - Huong dan AI** hoặc thư mục cài đặt `HuongDan\HUONG_DAN_AI_TU_ANH_THANH_CAD.docx`.
5. Với bản **Portable**: giải nén ZIP và mở `HUONG_DAN_AI_TU_ANH_THANH_CAD.docx` nằm cạnh `CADTBT360.exe`.

**Kiểm thử nguồn bổ sung:** `ai_raster_help_is_accessible_offline_without_external_links`; Windows CI kiểm tra tồn tại và yêu cầu thực thi bài kiểm thử này trước khi phát hành.

**Trạng thái nghiệm thu:** Đã có mã nút và nội dung hướng dẫn trong GUI, cùng shortcut hướng dẫn của Inno Setup. Chưa xác minh Windows CI PASS, bản EXE hoạt động thực tế hay ảnh chụp màn hình từ chính bản cài. Cần kiểm tra và bổ sung ảnh chụp chuẩn từ phần mềm sau khi build đạt.


## 10. Nâng cấp đợt 171 — Kiểm tra AI Agent/MCP từ bộ cài thật

**Mục tiêu:** Bản cài Windows và Portable ZIP phải thực sự chạy được `CADTBT360_CLI.exe mcp`, không chỉ biên dịch mã MCP.

### Bước 1 — Kiểm tra MCP trên máy Windows

Với bản Setup: mở **PowerShell** tại thư mục cài CAD 360 (thường ở `%LOCALAPPDATA%\Programs\CADTBT360`). Với Portable: mở PowerShell ngay trong thư mục đã giải nén.

Chạy lệnh:

```powershell
.\CADTBT360_CLI.exe mcp
```

MCP sử dụng giao thức JSON-RPC dạng từng dòng qua **stdin/stdout**. Khi chạy trực tiếp trong cửa sổ PowerShell, chương trình sẽ chờ yêu cầu từ client MCP; đây không phải lệnh vẽ tự động và không có giao diện riêng. Nhấn **Ctrl+C** để kết thúc.

### Bước 2 — Kết nối bằng ứng dụng MCP client

Trong cấu hình MCP client hỗ trợ công cụ qua stdio, chọn **Command** là đường dẫn đầy đủ tới `CADTBT360_CLI.exe` và **Arguments** là `["mcp"]`. Sau khi kết nối, kiểm tra trong danh sách công cụ có `raster_to_cad_lines`.

Để xử lý ảnh trong một phiên CAD đang mở, cần cấu hình đúng **Remote** hoặc kênh điều khiển kết nối đến phiên GUI. Nếu chạy `mcp` không có tham số kết nối, công cụ hoạt động ở chế độ **Headless** với bản vẽ riêng, không tự động sửa bản vẽ đang hiển thị trên GUI.

### Bước 3 — Nhận dạng ảnh an toàn

Đầu tiên gọi công cụ `raster_to_cad_lines` với `preview: true`; cung cấp đường dẫn ảnh hợp lệ và tỷ lệ CAD/pixel. Kiểm tra danh sách tọa độ `lines` và số lượng `strokes`. Chỉ dùng `preview: false` khi xác nhận kết quả và đúng bản vẽ cần chỉnh sửa. Sau đó kiểm tra LINE đã tạo, Undo và lưu bản vẽ mới.

### Bước 4 — Đọc kết quả tự kiểm tra phát hành

Trên GitHub Actions, workflow **CAD 360 V1.025 - Windows Setup Portable** dùng script `scripts/verify_mcp_executable.ps1` chạy trên cả EXE đã cài và EXE giải nén từ Portable ZIP. Script phải xác minh:

1. MCP `initialize` trả phiên bản giao thức tương thích.
2. `tools/list` chứa công cụ `raster_to_cad_lines`.
3. Lời gọi công cụ đối với tệp ảnh không tồn tại trả lỗi có kiểm soát.
4. Tiến trình MCP thoát sạch, không bị treo hoặc thất bại.

**Phân biệt:** Smoke test không chứng minh AI nhận dạng mọi bản vẽ thực tế hay Windows GUI đã được người dùng nghiệm thu. Chỉ khi workflow PASS và có biên bản thử GUI/ảnh xây dựng thực mới được ghi trạng thái đạt sử dụng.


## 11. Nâng cấp đợt 172 — Kiểm thử MCP với ảnh PNG thật từ chính bản EXE phát hành

Trước đây, bài smoke test MCP chỉ kiểm tra khởi động máy chủ và việc từ chối một đường dẫn ảnh không tồn tại. Từ đợt 172, CI sẽ **tự tạo ảnh PNG thử nghiệm ngay trên Windows runner**, gọi `CADTBT360_CLI.exe mcp` từ đúng bộ cài và kiểm tra chuỗi thao tác thực tế.

### Cách tự thực hiện với phần mềm đã cài

1. Mở PowerShell trong thư mục cài đặt hoặc Portable và khởi động `.\CADTBT360_CLI.exe mcp` thông qua một MCP client hỗ trợ giao thức stdio.
2. Tạo hoặc chọn ảnh PNG 64 × 32 pixel, nền trắng có **một đoạn thẳng đen dài khoảng 50 pixel**. Không để ảnh chứa chữ hoặc các đường chéo.
3. Gọi công cụ `raster_to_cad_lines` với `preview: true`, `minLengthPixels: 16`, `includeDiagonals: false`. Kết quả kỳ vọng: **1 nét**, `editableEntitiesCreated=0`.
4. Gọi lại cùng công cụ với `preview: false`. Kết quả kỳ vọng: **1 LINE CAD được tạo**.
5. Gọi `query_entities` với `type: "Line"`, xác nhận số lượng bằng **1**.
6. Gọi công cụ `execute` với `command: "undo"`, `params: {"count":1}`. Gọi `query_entities` lại, số LINE kỳ vọng là **0**.

**Đường dẫn mã kiểm thử:** `scripts/verify_mcp_executable.ps1`. Windows CI gọi script với CLI trong thư mục cài Setup và lần thứ hai với CLI giải nén từ Portable ZIP. Nếu bất kỳ bước nào sai, job dừng và **không được ghi kết quả PASS**.

**Lưu ý:** Thử nghiệm này kiểm tra đường đi PNG → LINE → Undo của bộ cài Headless/MCP. Nó **không thay thế** việc mở GUI trên Windows, kiểm tra ảnh thực tế, đo tỷ lệ và kiểm chứng DWG bằng phần mềm CAD độc lập. Chưa có bằng chứng Windows CI PASS tại thời điểm cập nhật tài liệu.


## 12. Nâng cấp đợt 173 — Kiểm chứng bản vẽ do AI tạo được lưu và mở lại

**Điểm mới:** Bài kiểm thử bộ cài đã mở rộng chuỗi **ảnh PNG → Preview → tạo LINE → Undo → tạo lại LINE → lưu DXF → chuyển sang DWG R2018 → mở lại DXF và DWG**. Cả **Setup** và **Portable** đều phải vượt qua các bước tương tự trước khi công bố release.

### Thao tác kiểm tra từng bước

1. Mở CAD 360 V1.025, vào **CAD → HƯỚNG DẪN AI** nếu cần xem các bước; chọn **AI TỪ ẢNH → CAD** để thao tác.
2. Mở một bản vẽ thử độc lập. Chọn PNG có một nét ngang trên nền trắng. Nhập `Nét tối thiểu = 16 px`, bỏ chọn chế độ góc bất kỳ nếu ảnh chỉ có nét ngang.
3. Nhấn **Nhận dạng và xem trước**, kiểm tra một nét và tỷ lệ. Sau đó nhấn **Tạo LINE chỉnh sửa trong CAD**.
4. Dùng **Properties** hoặc lựa chọn trực tiếp để xác nhận kết quả là đối tượng **LINE**, không phải raster IMAGE.
5. Thử **UNDO** để hoàn tác cả nhóm AI. Thực hiện lại nhận dạng và tạo LINE để kiểm tra đường tạo lần hai.
6. Lưu bản vẽ dưới dạng **DXF** mới; mở lại DXF và kiểm tra LINE còn tồn tại, có thể kéo điểm hoặc sửa vị trí.
7. Dùng công cụ chuyển đổi của CAD 360 để xuất sang **DWG**, mở lại và kiểm tra số lượng đối tượng. Với DWG R2018, đầu tệp nhị phân phải mang mã `AC1032`; riêng mã đầu tệp **không chứng minh** tương thích hoàn toàn với AutoCAD.
8. Kiểm tra độc lập bằng AutoCAD hoặc phần mềm CAD bên thứ ba khi có điều kiện; không đánh dấu tương thích hoàn chỉnh nếu chưa thực hiện.

### Kiểm thử tự động trên EXE đã cài

Script `scripts/verify_mcp_executable.ps1` hiện tạo ảnh PNG 64 × 32 pixel có một nét ngang dài 51 pixel, gọi các công cụ MCP để tạo và hoàn tác LINE, sau đó dùng **chính CADTBT360_CLI.exe trong bộ cài** để chuyển DXF thành DWG, kiểm tra mã `AC1032` và đếm đối tượng khi mở lại cả hai tệp.

**Giới hạn nghiệm thu:** Kết quả tự động chỉ xác nhận pipeline CAD nội bộ trên dữ liệu mẫu đơn giản. Chưa phải kết quả nghiệm thu mô hình AI hiểu bản vẽ xây dựng, chưa có ảnh chụp GUI từ bộ cài mới, chưa chứng minh khả năng mở/chỉnh sửa hai chiều với AutoCAD độc lập. Chỉ thông báo Windows CI PASS khi có workflow thành công kèm binary được phát hành.


## 13. Đợt nâng cấp 174 — Sửa ảnh PNG thử nghiệm, chặn CI khi dữ liệu thử bị lỗi

### Lỗi đã phát hiện và khắc phục
Đường dẫn Base64 chứa ảnh PNG thử nghiệm trong script Windows MCP của các đợt trước bị lỗi định dạng (giải mã Base64 thất bại). Đây là **lỗi dữ liệu kiểm thử**, không đủ cơ sở kết luận lỗi thuật toán CAD. Đợt 174 thay dữ liệu bằng PNG đã được kiểm tra: **64 × 32 pixel RGB8**, nền trắng, một nét ngang **51 pixel đen** tại hàng y=15.

### Cách kiểm tra thủ công theo từng bước
1. Vào nhánh V1.025 trên GitHub và mở `scripts/verify_mcp_executable.ps1`.
2. Xác nhận ảnh mẫu được định nghĩa bằng `$pngBase64`. Không chỉnh tay chuỗi Base64 nếu chưa kiểm tra ảnh nguồn.
3. Chạy `python scripts/validate_mcp_smoke_fixture.py scripts/verify_mcp_executable.ps1` từ thư mục gốc mã nguồn. Kết quả kỳ vọng là thông báo PASS cho PNG **64×32, RGB8, có một nét đen 51 pixel**.
4. Khi Windows CI chạy, script xác nhận Base64 hợp lệ, chữ ký PNG, CRC từng chunk, dữ liệu điểm ảnh đã giải nén và số pixel đen. Nếu một điều kiện sai, dừng trước bước đóng gói.
5. Sau bước kiểm tra ảnh, bài `scripts/verify_mcp_executable.ps1` tiếp tục thử AI/MCP trên bản **Setup** và **Portable**: Preview → LINE → Undo → DXF → DWG → mở lại.

### Thay đổi quy trình phát hành
Workflow `.github/workflows/cad360-v1025-release.yml` được cấu hình tự chạy khi thay đổi `scripts/verify_mcp_executable.ps1`, `scripts/validate_mcp_smoke_fixture.py`, hoặc bất kỳ tài liệu nào trong `docs/user-guide/`. Điều này giúp tránh phát hành tài liệu không đồng bộ với mã nguồn đang nghiệm thu.

**Giới hạn nghiệm thu:** Kiểm tra PNG chính xác và cập nhật script/CI **không đồng nghĩa** Windows build đã PASS. Cần có job thành công, bộ cài phát hành và thử GUI/AutoCAD độc lập trước khi ghi trạng thái đạt nghiệm thu.


## 14. Đợt nâng cấp 175 — Kiểm tra ảnh mẫu ngay đầu Windows CI

**Thay đổi mới:** Workflow chạy `scripts/validate_mcp_smoke_fixture.py` **trước khi tải/biên dịch Rust**. Nếu ảnh PNG Base64 bị sửa nhầm, checksum chunk không hợp lệ hoặc nét đen không đúng dữ liệu kiểm thử, bước này báo lỗi sớm mà không tạo bộ cài giả đạt.

### Hướng dẫn kiểm tra từng bước

1. Mở PowerShell tại thư mục gốc của mã nguồn CAD 360 V1.025.
2. Kiểm tra Python: `python --version` (workflow sử dụng Python 3.12).
3. Chạy lệnh `python scripts/validate_mcp_smoke_fixture.py scripts/verify_mcp_executable.ps1`.
4. Kết quả kỳ vọng: `PASS: valid PNG 64x32 RGB8; one horizontal 51-pixel black stroke`.
5. Nếu lỗi **Base64**: kiểm tra chuỗi `$pngBase64` trong `scripts/verify_mcp_executable.ps1`; không tùy tiện sửa nếu chưa tạo và kiểm tra lại ảnh.
6. Nếu lỗi **CRC**: tệp PNG đã hỏng; tái tạo ảnh gốc rồi chạy lại script xác thực.
7. Nếu lỗi **pixel**: ảnh mẫu không còn đúng một nét ngang đen 51 pixel; điều chỉnh ảnh thử hoặc thay đổi kỳ vọng kiểm thử có kiểm soát.
8. Chỉ khi bước kiểm tra trên đạt, Windows workflow mới tiếp tục Cargo preflight, các bài test AI/MCP, build Release, cài Setup và kiểm chứng Portable.

### Cách theo dõi nghiệm thu

Trong GitHub Actions, mở workflow **CAD 360 V1.025 - Windows Setup Portable**. Kiểm tra bước `Validate embedded image fixture before Rust compilation`, sau đó các bước **Preflight**, **Require image-to-CAD regression**, **Build**, **Smoke test installed application** và **Verify portable archive**. Các trạng thái chỉ có ý nghĩa khi toàn bộ lượt chạy hoàn thành thành công; dấu tích từng bước không thay thế kiểm thử giao diện thực tế.

**Giới hạn:** Chưa xác minh có lượt Windows CI hoàn tất PASS cho mã mới; chưa có ảnh chụp GUI thật hoặc kiểm chứng DWG bằng AutoCAD độc lập. Đợt 175 mới cập nhật quy trình kiểm thử sớm và hướng dẫn xử lý lỗi.


## 15. Đợt nâng cấp 176 — Bàn giao đủ CODE + HƯỚNG DẪN

Đáp ứng yêu cầu bàn giao mỗi lần nâng cấp: một bản phát hành Windows được nghiệm thu thành công phải cung cấp đồng thời **mã nguồn đầy đủ từ đúng commit**, **hướng dẫn Word**, **bộ cài Setup**, **bản Portable** và **bảng kiểm tra SHA256**.

### Tải và đối chiếu theo từng bước

1. Mở **GitHub → Actions → CAD 360 V1.025 - Windows Setup Portable**, chọn lượt chạy có trạng thái **Success** và xác nhận mã commit hiển thị trong run.
2. Ở mục **Artifacts**, tải gói `CADTBT360_V1.025_Windows_x64_Setup_Portable`. Nếu workflow là push/dispatch thành công và đủ quyền công bố, có thể tải từng file từ **GitHub Releases**.
3. Trong gói phải có **`CADTBT360_V1.025_Full_Source.zip`** chứa toàn bộ mã nguồn được quản lý bằng Git tại commit đã build, **`CADTBT360_V1.025_Setup_Windows_x64.exe`**, **`CADTBT360_V1.025_Portable_Windows_x64.zip`**, **file hướng dẫn Word `CADTBT360_V1.025_HUONG_DAN_AI_TU_ANH_THANH_CAD.docx`**, `HUONG_DAN_AI_TU_ANH_THANH_CAD.md`, `BUILD_INFO.txt`, và `SHA256SUMS.txt`.
4. Kiểm tra SHA256 trong PowerShell bằng `Get-FileHash -Algorithm SHA256 .\CADTBT360_V1.025_Full_Source.zip` và so sánh giá trị với `SHA256SUMS.txt`. Thực hiện tương tự đối với Setup và Portable.
5. Giải nén `Full_Source.zip`; xác nhận có `CADTBT360_V1.000/Cargo.toml`, `CADTBT360_V1.000/Cargo.lock`, `CADTBT360_V1.000/apps/cadcraft/src/main.rs`, `CADTBT360_V1.000/crates/mcp/src/raster_vector.rs`, `installer/CADTBT360_V1.025_Windows.iss`, `scripts/verify_mcp_executable.ps1` và thư mục `docs/user-guide`.
6. Đọc `BUILD_INFO.txt` để xác nhận `SourceCommit`. Mã nguồn ZIP được tạo bằng `git archive HEAD` trên chính commit đó; không dùng bản tải từ một nhánh tiếp tục thay đổi làm căn cứ nghiệm thu phiên bản cũ.
7. Mở hướng dẫn Word trước khi vận hành tính năng AI. Với Setup, mở bản Word trong thư mục `HuongDan` của ứng dụng; với Portable, mở tệp Word cạnh `CADTBT360.exe`.

### Quy tắc kiểm tra tự động

Windows CI kiểm tra bản ZIP mã nguồn bằng `scripts/verify_source_bundle.py`: so khớp nội dung các file trọng yếu trong ZIP với nội dung Git của commit `HEAD`, kiểm tra có tài liệu hướng dẫn và không vô tình đưa các tệp EXE/DLL build vào gói mã nguồn. Thiếu một tệp bắt buộc hoặc sai mã nguồn thì **dừng phát hành**.

**Phạm vi giới hạn:** `Full_Source.zip` là toàn bộ **các file được Git theo dõi tại commit phát hành**, không bao gồm thư mục `.git`, bộ thư viện tải về bởi Cargo, hoặc tệp untracked chưa được commit. Word là asset phát hành riêng và đồng thời nằm trong bộ cài/Portable. Chỉ coi bộ bàn giao đầy đủ sau khi Windows CI PASS và các asset xuất hiện thật; tại thời điểm soạn hướng dẫn chưa có xác nhận điều kiện đó.


## 16. Đợt nâng cấp 177 — Chống dùng Preview AI lỗi thời khi chuyển qua lại bản vẽ

**Lỗi cần phòng ngừa:** Trước đây cửa sổ **AI từ ảnh → CAD** chỉ kiểm tra `documentUid` và `revision`. Khi người dùng đã Preview trên bản vẽ A, chuyển sang bản vẽ B rồi quay lại A mà không sửa nội dung, cả hai giá trị có thể giữ nguyên. Preview cũ có thể bị nhầm là còn hợp lệ.

**Đã cập nhật mã nguồn:** GUI ghi thêm `documentEpoch` tại thời điểm Preview. Trước khi tạo đối tượng `LINE`, hệ thống chỉ chấp nhận khi cả **ID tài liệu + revision + epoch** trùng khớp và không có lệnh khác/nhóm Undo xung đột. Đổi ảnh nguồn, Preview lỗi hoặc hoàn thành tạo nét sẽ xóa thông tin epoch của Preview trước đó.

### Hướng dẫn kiểm thử từng bước

1. Mở CAD 360 V1.025 và tạo bản vẽ A. Mở chức năng **AI TỪ ẢNH → CAD**.
2. Chọn ảnh PNG có nét ngang, thiết lập tỷ lệ rồi nhấn **1. Nhận dạng và xem trước**.
3. Chưa nhấn tạo LINE. Chuyển sang bản vẽ B, sau đó quay lại bản vẽ A, **không chỉnh sửa A**.
4. Quay lại cửa sổ AI, nhấn **2. Tạo LINE chỉnh sửa trong CAD**. Kết quả mong đợi: phần mềm từ chối Preview cũ, yêu cầu nhận dạng và xem trước lại. Không tạo thêm LINE.
5. Nhấn **Nhận dạng và xem trước** lần nữa trên đúng bản vẽ A, rồi nhấn tạo LINE. Lúc này việc ghi chỉ được phép nếu toàn bộ điều kiện an toàn vẫn phù hợp.
6. Kiểm tra số đối tượng vừa tạo và UNDO; lưu một bản DXF/DWG mới để thử mở lại.

### Kiểm thử tự động đi kèm

Bài kiểm thử `ai_raster_preview_refuses_switch_away_and_back_with_unchanged_revision` xác nhận rằng tăng `documentEpoch` phải khiến Preview cũ bị từ chối, ngay cả khi `documentUid` và `revision` không đổi. Windows CI được yêu cầu khám phá và chạy bài kiểm thử này trước khi đóng gói.

### Bàn giao đủ mã nguồn và hướng dẫn

Mỗi lượt phát hành Windows thành công được cấu hình để kèm `CADTBT360_V1.025_Full_Source.zip`, Setup, Portable, bản Word hướng dẫn và bảng SHA256. Khi chưa có GitHub Actions Success, có thể tải **ZIP mã nguồn tại commit cố định** và bản Word hướng dẫn được cung cấp riêng; đây là mã nguồn và tài liệu, **không phải** xác nhận EXE đã đạt.

**Chưa nghiệm thu:** Windows CI PASS, ảnh chụp thật từ GUI phát hành, tính đúng đắn với mặt bằng thực tế và khả năng tương thích DWG hai chiều với AutoCAD độc lập.
