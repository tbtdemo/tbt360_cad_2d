# CAD 360 Việt Nam V1.028 — Hướng dẫn nhập lệnh song ngữ và thao tác chuột

## 1. Nguyên tắc tương thích CAD gốc

Mọi câu gợi ý phải trỏ đến một lệnh CAD có thực, **không thay đổi cú pháp của lệnh gốc**. Alias như `L` (LINE — Vẽ đường thẳng), `C` (CIRCLE — Vẽ đường tròn), `PL` (PLINE — Vẽ đa đoạn) vẫn nhận **Space hoặc Enter** theo hành vi truyền thống. Lệnh tiếng Anh được ưu tiên hiển thị và tiếng Việt đặt trong ngoặc.

## 2. Chọn lệnh bằng bàn phím

1. Nhập một phần tên lệnh, ví dụ `L`, `CEN`, `đường`, `đường thẳng`, `vẽ đường tròn`.
2. Khi AutoComplete xuất hiện, dùng **↓** để chọn dòng đầu tiên và xuống các dòng sau, **↑** để đi lên. Dòng chọn được đánh dấu nổi bật; nội dung đang gõ **không bị thay đổi** chỉ vì di chuyển lựa chọn.
3. Nhấn **Enter** để chạy lệnh CAD native tại dòng đang chọn.
4. Nếu **không** chọn dòng, Enter xử lý nội dung đang gõ theo hành vi gốc, trong đó có tên đầy đủ hoặc các cụm tiếng Việt đủ rõ nghĩa.

**Ví dụ:** Nhập `Đường Thẳng` rồi Enter → `LINE` (Vẽ đường thẳng). Nhập `đường` → nhiều gợi ý, chưa tự đoán thay người dùng. Nhập `L` rồi Space → `LINE` truyền thống.

## 3. Nhập tiếng Việt có dấu, khoảng trắng

Hệ thống giữ nguyên chuỗi người dùng gõ trong ô lệnh. Khi một từ hoặc cụm từ còn có khả năng được bổ sung thành gợi ý tiếng Việt, Space tiếp tục nhập dấu cách chứ không gửi lệnh; còn alias CAD gốc vẫn sử dụng Space như Enter. Hỗ trợ các ký tự tiếng Việt có dấu và Unicode khi IME giao sự kiện đã xác nhận. Gõ `Đường Thẳng`, không cần viết `duongthang`.

## 4. Tránh dính chuột khi đi từ thanh lệnh đến bản vẽ

- Thanh lệnh, lịch sử và danh sách gợi ý có vùng tương tác được xác định riêng. Khi con trỏ ở đó, không chọn đối tượng, không bắt điểm, không kéo vùng chọn hoặc zoom bản vẽ.
- Chỉ mép kéo thay đổi chiều cao lịch sử lệnh mới nhận động tác **drag**; phần thanh lệnh không còn chiếm kéo chuột trên toàn bộ vùng cửa sổ.
- Di chuyển ra vùng vẽ cho phép rê chuột, OSNAP, cuộn zoom, click chọn đúng hành vi CAD.
- **Esc** chỉ dùng để hủy lệnh/chọn hiện tại; không phải giải pháp bắt buộc để thoát trạng thái giữ chuột do thanh lệnh.

## 5. Phạm vi nghiệm thu

- Kiểm thử V1.028: tên lệnh tiếng Việt nhiều từ, dấu và khoảng trắng; ↑/↓/Enter; không thay chuỗi đang nhập; vùng click thanh lệnh tách biệt canvas.
- Kiểm thử bổ sung cần có: rà soát chuột thật trên Windows với DPI 100%/125%/150%, IME Telex/VNI, kéo giao diện, zoom, chọn đối tượng, Esc/Undo, đầy đủ lệnh CAD gốc và file DWG mở hai chiều.
- Việc đã commit mã hoặc chạy thử test **không đồng nghĩa** hoàn tất nghiệm thu cho tới khi Windows CI PASS và thử bản Setup/Portable trực tiếp.

## 6. Mục tiêu các bản

Phiên bản 1.028 tập trung khắc phục nhập lệnh và chuột; không tự nhận rằng AI đã đọc/dựng chính xác toàn bộ mặt bằng kiến trúc. Các thử nghiệm AI ảnh lớn thuộc module độc lập và vẫn phải xem trước, xác nhận trước khi tạo hình học CAD.
