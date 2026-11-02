# CAD360 IFC — Thực trạng tích hợp Windows và dữ liệu BIM

## IFC GlobalId, tầng và bộ môn
Tab 3D/IFC hỗ trợ nạp ánh xạ JSON bằng nút **Nạp ánh xạ IFC...**, lọc GlobalId, tầng, bộ môn và đối tượng OBJ chưa xác định danh tính. Đầu vào JSON dạng:

```json
{"objects":[{"obj_group":"Beam_3h7B1JG5D4QQ4W8gcZQ0Xa","global_id":"3h7B1JG5D4QQ4W8gcZQ0Xa","storey":"Tầng 2","discipline":"Kết cấu","ifc_type":"IFCBEAM"}]}
```

Công cụ Python `tools/export_ifc_obj_mapping.py` trích IFC STEP GlobalId và quan hệ `IfcRelContainedInSpatialStructure`, khớp nhóm OBJ chỉ khi tên nhóm chứa đúng GlobalId duy nhất có trong IFC. **Không suy đoán** ánh xạ theo hình học hoặc loại đối tượng. Bộ môn là nhãn nguồn được truyền qua `--discipline`, không được suy đoán từ tên IFC class.

Ví dụ từ thư mục `SOURCE_CAD_R65`:

```powershell
python apps/cadcraft/tools/export_ifc_obj_mapping.py --ifc D:\Models\house.ifc --obj D:\Models\house.obj --out D:\Models\house.mapping.json --discipline "Kết cấu"
python -m unittest discover -s apps/cadcraft/tools -p "test_export_ifc_obj_mapping.py" -v
```

Cần cung cấp OBJ có tên `o/g` chứa đúng 22 ký tự GlobalId. Nếu IfcConvert hoặc trình xuất OBJ loại bỏ GlobalId, không tạo được ánh xạ đáng tin cậy: phải thay thế pipeline tessellation xuất trực tiếp GUID-to-face theo chuẩn riêng hoặc dùng API geometry IFC hỗ trợ định danh.

## Renderer 3D
Phần hiển thị đang dùng `egui::Painter` và sắp xếp chiều sâu phía CPU; dù eframe đã bật backend wgpu, **chưa có custom wgpu render pipeline/depth buffer**. Phải bổ sung `egui_wgpu` render callback, vertex/index buffer, WGSL shader, depth texture, camera origin rebasing, picking theo element và kiểm thử GPU.

## EXE Portable
Workflow `.github/workflows/cad360-ifc-portable.yml` chạy cargo check/test/release build trên `windows-latest`, sau đó tạo ZIP artifact chứa `CAD360.exe`, công cụ metadata Python và cảnh báo thiếu IfcConvert. **Workflow chạy PASS mới xác nhận ZIP đã được tạo**; chưa xác nhận chạy trên máy sạch. Bản ZIP không bao gồm IfcConvert.exe hoặc DLL chưa được build/kiểm định.

Nguồn IfcOpenShell-0.9.0.zip được cung cấp là mã nguồn, không phải Windows runtime. Để hoàn thiện chuyển đổi IFC phải build IfcConvert x64, xác minh DLL phụ thuộc, kiểm tra IFC2x3/IFC4, đóng gói và chạy smoke test trên Windows sạch.

## Các điều kiện nghiệm thu
- CI Windows cargo check/test/build PASS và artifact ZIP tồn tại.
- Chạy CAD360.exe trên Windows mới; ảnh chụp đúng giao diện thực tế, kiểm tra menu/tab.
- Mở IFC2x3/IFC4 mẫu, chọn/ẩn/cô lập cấu kiện có GlobalId xác nhận đối chiếu với IFC.
- Lọc tầng/bộ môn với dữ liệu IFC thực và kiểm tra các đối tượng không có ánh xạ.
- Renderer custom wgpu phải có depth buffer và picking chính xác trước khi báo đạt yêu cầu GPU.
