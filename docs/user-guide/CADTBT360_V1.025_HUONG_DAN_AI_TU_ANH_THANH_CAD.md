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
