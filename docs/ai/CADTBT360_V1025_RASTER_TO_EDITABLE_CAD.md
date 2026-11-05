# CAD 360 Việt Nam — Tự nhận dạng nét từ PNG/JPEG để tạo CAD chỉnh sửa được (V1.025)

## Phạm vi hiện thực

Đã thêm công cụ MCP `raster_to_cad_lines` — tự đọc pixel từ ảnh PNG/JPEG trên máy cục bộ, dò các nét tối nằm ngang và thẳng đứng, hợp nhất nét đậm có bề rộng vài pixel, rồi tạo **đối tượng CAD LINE thực** khi được người dùng/Agent cho phép. Không cần tự cung cấp tọa độ từng đỉnh.

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
  "maxLines": 250
}
```

Kết quả trả về `imageSize`, `strokes` và `lines` (mỗi đoạn gồm hai tọa độ CAD). `editableEntitiesCreated=0` ở chế độ xem trước. Điểm ảnh được quy đổi theo trục Y hướng lên của CAD.

### Bước 2 — Ghi các LINE có thể chỉnh sửa

Sau khi kiểm tra đoạn nét và tỷ lệ, gọi lại với `preview:false` và các tham số giống nhau. Công cụ đưa từng đoạn vào engine bằng lệnh `line`, tạo đối tượng CAD thật, có thể chọn, sửa hoặc đo. Sau đó sử dụng công cụ MCP `save` để lưu DXF/DWG.

Lưu ý: hiện bước ghi nhiều đối tượng **chưa phải giao dịch tất cả-hoặc-không**. Nếu engine từ chối một đoạn giữa quá trình, các đoạn đã ghi trước đó có thể còn trong bản vẽ; nên thao tác trên bản sao và kiểm tra `editableEntitiesCreated` trước khi lưu.

## Giới hạn và an toàn

- Chỉ hỗ trợ tệp PNG/JPEG cục bộ, tối đa 16 MiB và ảnh tối đa 4.096 pixel mỗi chiều / 12 triệu pixel.
- Chỉ nhận dạng nét thẳng ngang/dọc tương phản rõ; ảnh phối cảnh, nét chéo, bản vẽ có nhiễu hoặc đứt nét sẽ cần thuật toán nâng cao.
- Có giới hạn `maxLines` để tránh lỡ tạo hàng nghìn đối tượng; nếu vượt giới hạn, công cụ trả lỗi trước khi sửa CAD.
- Mặc định `preview:true`; không tự chỉnh sửa bản vẽ chỉ vì Agent gửi yêu cầu xem trước.
- Chưa có giao diện nút bấm riêng trong CAD; thao tác thực hiện qua MCP hoặc Agent được kết nối với CAD.
- Cần chạy regression và Windows Build để xác nhận sẵn sàng phát hành; chưa khẳng định PASS khi chưa có log.

## Định hướng phát triển tiếp

1. Nhận dạng nét chéo, cung tròn, polyline đa hướng; loại nhiễu và ghép các đoạn.
2. Kết nối mô hình thị giác thực (local/remote) để phân loại tường, cửa, trục, ký hiệu và chữ.
3. Hiệu chỉnh kích thước/tỷ lệ bằng điểm kiểm soát; dựng Layer và đối tượng CAD theo ngữ nghĩa.
4. Tương tác chọn vùng ảnh ngay trên canvas, hiển thị lớp nét xem trước, xác nhận và Undo một lượt.
5. Kiểm thử CAD GUI và vòng mở/sửa/lưu DXF/DWG độc lập trong AutoCAD.
