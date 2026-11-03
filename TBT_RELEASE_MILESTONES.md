# CAD360 TBT — Quy trình bàn giao 15–30 lượt / 1 bộ cài

## Quy tắc mới (03/11/2026)
- Sau mỗi lượt phát triển: commit đầy đủ mã nguồn, báo cáo chức năng và link tải source theo commit. Không tuyên bố hoàn tất nếu chưa có bằng chứng.
- **Chỉ đóng gói một lần sau khi tích lũy từ 15 đến 30 lượt/bộ thay đổi**, tùy quy mô đợt. Không đóng gói tự động tại mốc 5 hoặc 10 nữa.
- Mốc đóng gói dùng định dạng `B15`, `B20`, `B25` hoặc `B30` (số lượt phải được ghi trong báo cáo tiến độ thực tế). Ví dụ commit kích hoạt: `release(cad360): B15`.
- Workflow Direct EXE mỗi commit chỉ chạy kiểm kê nguồn. Windows build chỉ chạy ở commit mốc B15–B30 hoặc khi dispatch với số lượt tương ứng.
- Workflow IFC Portable tiếp tục tạo source snapshot trên các commit có thay đổi code; chỉ xây dựng EXE Portable khi dispatch với đủ 15–30 lượt.
- **Một lần đóng gói gồm** `CAD360_VietNam_TBT_Setup.exe`, `SHA256SUMS.txt`, `CAD360_TBT_Source_RXX.zip`. Phát hành lên GitHub Releases nếu đủ test PASS, build Release PASS và file cài đặt tồn tại.
- Nếu một gate thất bại, trạng thái là `BLOCKED`: phải sửa và chạy lại; không được lấy file cũ hoặc ZIP nguồn thay EXE.
- GitHub Releases: https://github.com/tbtdemo/tbt360_cad_2d/releases
- Nhánh phát triển: `tbt/cad360-active-release`. Không dùng thư mục CAD/QTO lịch sử làm binary đầu vào.

## Đợt hiện tại
- Quy tắc 5/10 cũ đã được thay thế bởi đợt 15–30. Đếm số lượt thực hiện trong báo cáo, không giả vờ mỗi commit tương đương một lượt hoàn chỉnh.
- Lần build R05 trước khi đổi quy tắc là build kiểm tra tồn đọng; không được tính là bản phát hành của đợt 15–30 mới.
- Chưa có bản Windows Setup phát hành chính thức được xác nhận.
