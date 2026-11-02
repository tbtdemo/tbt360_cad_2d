# CAD360 — IFC Geometry Runtime (Windows)

## Trạng thái
CAD360 có module `model3d.rs` đọc STEP IFC để thống kê và xem trước tọa độ. Khi có `IfcConvert.exe` hợp lệ, tab **3D / IFC** có thể chuyển tệp IFC sang Wavefront OBJ và hiển thị tam giác. Bản tích hợp này chưa phải BIM Viewer hoàn chỉnh (chưa chọn GUID theo mặt, chưa ẩn/cô lập, chưa xử lý độ sâu chính xác).

## Công cụ hình học
Thư mục phát hành dự kiến:

```text
CAD360/
  CAD360.exe
  tools/
    IfcConvert.exe
    <các DLL runtime phụ thuộc do build IfcOpenShell tạo ra>
```

Gói IfcOpenShell-0.9.0.zip người dùng cung cấp là mã nguồn, **không chứa IfcConvert.exe hay DLL Windows**. Cần tạo bản Windows x64 theo tài liệu build gốc của IfcOpenShell, kiểm tra license và phân phối đầy đủ runtime. Không đổi tên mã nguồn ZIP thành DLL, không tự coi là dependency sẵn có.

## Các mốc nghiệm thu
1. Build CAD360 Rust bằng Cargo và chạy unit tests trên Windows.
2. Kiểm tra IfcConvert.exe hoạt động với IFC2x3/IFC4 mẫu và xuất OBJ.
3. Kiểm tra OBJ được nạp và vẽ các tam giác.
4. So sánh hình học tường/cột/dầm/sàn và GUID với BIM viewer tham chiếu.
5. Thêm kiểm tra bounding box, thứ tự chiều sâu, chọn đối tượng, lớp/tầng, bộ môn và QTO.
6. Chạy EXE thực tế trên máy Windows mới, không có bộ công cụ phát triển.

## Cảnh báo kỹ thuật
- Chuyển đổi hiện chạy đồng bộ, có thể chặn giao diện với IFC lớn. Cần đổi sang worker bất đồng bộ có timeout và khả năng huỷ.
- OBJ không bảo toàn chắc chắn GUID/properties và quan hệ IFC; lưu mapping theo GlobalId nếu phát triển phối hợp BIM và QTO.
- Mesh renderer hiện dùng egui painter, chưa có z-buffer và chưa đủ tối ưu cho tệp lớn; nên chuyển sang pipeline GPU/wgpu.
- Chỉ phân phối EXE portable sau khi xác nhận tất cả các điều kiện runtime và test thực tế.
