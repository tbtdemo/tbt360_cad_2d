# CAD 360 Việt Nam — Tự nhận dạng nét từ PNG/JPEG để tạo CAD chỉnh sửa được (V1.025)

## Phạm vi hiện thực

Đã thêm công cụ MCP `raster_to_cad_lines` — tự đọc pixel từ ảnh PNG/JPEG trên máy cục bộ, dò các nét tối nằm ngang, thẳng đứng và đường chéo 45° theo hai hướng, hợp nhất nét đậm có bề rộng vài pixel, rồi tạo **đối tượng CAD LINE thực** khi được người dùng/Agent cho phép. Không cần tự cung cấp tọa độ từng đỉnh.

Đây là **vector hóa bằng thuật toán ảnh**, chưa sử dụng mô hình AI thị giác để hiểu tường, cửa, cột, ký hiệu, chữ, đường cong hoặc kích thước. Chưa tự động dựng Layer/kích thước hoặc hiệu chỉnh phối cảnh. Không được quảng cáo tương đương “chuyển mọi ảnh thành CAD hoàn chỉnh”.

## Kết nối với CAD đang mở

Chạy ứng dụng CAD với kênh điều khiển MCP được bảo vệ bằng token theo tài liệu `CADTBT360_V1.000/docs/control-protocol.md`. Trên MCP client, dùng kết nối `cadcraft-cli mcp --connect HOST:PORT` (có thể dùng chế độ headless nếu không cần GUI). Công cụ gửi lệnh `engine.execute` tới chính phiên CAD hiện tại khi bridge kết nối GUI.

### Bước 1 — Xem trước (không sửa bản vẽ)

Gọi MCP `raster_to_cad_lines` với tham số JSON:

```json
{
  "path": "C:/CAD360/Samples/mat-bang.png",
  "preview": true,
  "threshold": 160,
  "minLengthPixels": 20,
  "unitsPerPixel": 0.01,
  "origin": [0, 0],
  "maxLines": 250,
  "includeDiagonals": true,
  "includeArbitraryAngles": true
}
```

Kết quả trả về `imageSize`, `strokes` và `lines` (mỗi đoạn gồm hai tọa độ CAD). `editableEntitiesCreated=0` ở chế độ xem trước. Điểm ảnh được quy đổi theo trục Y hướng lên của CAD.

### Bước 2 — Ghi các LINE có thể chỉnh sửa

Sau khi kiểm tra đoạn nét và tỷ lệ, gọi lại với `preview:false` và các tham số giống nhau. Công cụ đưa từng đoạn vào engine bằng lệnh `line`, tạo đối tượng CAD thật, có thể chọn, sửa hoặc đo. Sau đó sử dụng công cụ MCP `save` để lưu DXF/DWG.

Lưu ý: hiện bước ghi nhiều đối tượng **chưa phải giao dịch tất cả-hoặc-không**. Nếu engine từ chối một đoạn giữa quá trình, các đoạn đã ghi trước đó có thể còn trong bản vẽ; nên thao tác trên bản sao và kiểm tra `editableEntitiesCreated` trước khi lưu.

## Giới hạn và an toàn

- Kiểm tra kích thước ảnh từ phần header trước khi giải mã toàn bộ PNG/JPEG để hạn chế cấp phát quá mức.

- Chỉ hỗ trợ tệp PNG/JPEG cục bộ, tối đa 16 MiB và ảnh tối đa 4.096 pixel mỗi chiều / 12 triệu pixel.
- Đã bổ sung thử nghiệm nhận dạng các nét thẳng đơn lẻ ở góc nghiêng bất kỳ bằng phân tích thành phần liên thông và PCA; bật bằng includeArbitraryAngles:true. Các điểm giao nhánh, chữ, khối đặc, cung tròn, đường cong và ảnh phối cảnh vẫn chưa được nhận dạng có ngữ nghĩa; bắt buộc kiểm duyệt preview.
- Có giới hạn `maxLines` để tránh lỡ tạo hàng nghìn đối tượng; nếu vượt giới hạn, công cụ trả lỗi trước khi sửa CAD.
- Mặc định `preview:true`; không tự chỉnh sửa bản vẽ chỉ vì Agent gửi yêu cầu xem trước.
- Chưa có giao diện nút bấm riêng trong CAD; thao tác thực hiện qua MCP hoặc Agent được kết nối với CAD.
- Cần chạy regression và Windows Build để xác nhận sẵn sàng phát hành; chưa khẳng định PASS khi chưa có log.

## Định hướng phát triển tiếp

1. Hoàn thiện nhận dạng góc nghiêng bất kỳ ở các điểm giao nhau, cung tròn, polyline đa hướng; loại nhiễu và ghép các đoạn.
2. Kết nối mô hình thị giác thực (local/remote) để phân loại tường, cửa, trục, ký hiệu và chữ.
3. Hiệu chỉnh kích thước/tỷ lệ bằng điểm kiểm soát; dựng Layer và đối tượng CAD theo ngữ nghĩa.
4. Tương tác chọn vùng ảnh ngay trên canvas, hiển thị lớp nét xem trước, xác nhận và Undo một lượt.
5. Kiểm thử CAD GUI và vòng mở/sửa/lưu DXF/DWG độc lập trong AutoCAD.

## Thao tác trực tiếp trên giao diện CAD 360 (bổ sung mã V1.025)

Bên cạnh MCP, desktop đã bổ sung nút **AI TỪ ẢNH → CAD** trên thanh điều khiển tab CAD.

1. Mở CAD 360, vào tab **CAD**, chọn **AI TỪ ẢNH → CAD**.
2. Chọn tệp PNG/JPG từ máy. Giữ `Nét tối thiểu` khoảng 16–20 pixel cho lần thử đầu, rồi nhập tỷ lệ `Đơn vị CAD/pixel` theo kích thước đo kiểm.
3. Chọn **Nét 45°** và khi cần, **Góc bất kỳ (thử nghiệm)**. Chế độ góc bất kỳ có thể bỏ qua nét giao nhánh/chồng lấn vì thuật toán cố tình không suy đoán cấu kiện.
4. Bấm **1. Nhận dạng và xem trước**. Các đoạn nét được vẽ minh họa trong hộp thoại theo tọa độ CAD và chưa làm thay đổi bản vẽ.
5. Kiểm tra các nét và tỷ lệ. Bấm **2. Tạo LINE chỉnh sửa trong CAD** để thực sự đưa nét vào tài liệu CAD đang mở.
6. Kiểm tra đối tượng LINE, chọn/sửa/lưu DWG hoặc DXF bằng chức năng CAD thông thường.

**Trạng thái nghiệm thu:** Các bước trên đã được lập trình, nhưng chưa có chứng cứ CI Windows PASS hoặc ảnh chụp xác nhận bản EXE mới chạy đạt yêu cầu. Không nhầm hình xem trước trong cửa sổ AI với lớp phủ trực tiếp lên canvas; phần overlay tương tác/chọn vùng ngay trên canvas vẫn cần triển khai.

**Bảo vệ bản vẽ:** Preview gắn với cả ID và số phiên bản sửa đổi (revision) của bản vẽ đang mở. Nếu người dùng chuyển tài liệu, chỉnh sửa bản vẽ sau khi Preview hoặc vẫn đang chạy lệnh CAD khác, giao diện từ chối ghi các nét AI; cần nhận dạng và xem trước lại trên phiên bản hiện hành.

**Hoàn tác:** Cửa sổ AI nhóm toàn bộ LINE của một lần xác nhận vào một bước UNDO. Yêu cầu UNDO Control All và không có nhóm UNDO khác đang mở. Nếu một nét bị engine từ chối giữa chừng, giao diện thử tự gọi UNDO để phục hồi bản vẽ trước lô AI; nếu hoàn tác cũng lỗi, hệ thống hiển thị cảnh báo để người dùng kiểm tra. Đã bổ sung kiểm thử lỗi giữa lô, nhưng chưa xác minh CI PASS.

**Giới hạn:** Quá trình tạo nhiều LINE hiện chưa bảo đảm tính nguyên tử toàn bộ. Hãy thực hiện trên bản sao bản vẽ và kiểm tra kết quả trước khi lưu. Không nhận diện ngữ nghĩa tường, cửa, cột, phòng hoặc kích thước như phần mềm AI thị giác chuyên dụng.
