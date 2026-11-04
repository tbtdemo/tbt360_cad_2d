# CAD 360 Việt Nam — quy tắc phiên bản phát hành

**Từ 04/11/2026:** Không được giữ nguyên số phiên bản khi phát hành một bộ Setup/Portable mới.

## Phiên bản

- **V1.023**: bộ cài RC đã PASS, GitHub Actions run 41791185215.
- **V1.024**: bản RC kế tiếp, workflow `.github/workflows/cadtbt360-v1024-rc-windows.yml`. Chỉ công bố là có thể tải khi workflow SUCCESS và có cả Setup/Portable artifact.
- **V1.025**: tên dự kiến cho bộ cài *tiếp theo*, không dùng lại V1.024.

Mỗi **lần phát hành bộ cài mới** tăng ít nhất một đơn vị patch: V1.024 → V1.025 → V1.026… Thay đổi source hoặc thêm kiểm thử mà **chưa biên dịch và phát hành EXE mới** không được tính thành phiên bản phần mềm mới. Chạy lại cùng workflow do lỗi CI không tự tạo phiên bản mới; nhưng bản phát hành kế tiếp phải có mã mới.

## Kiểm tra bắt buộc trước khi công bố

1. Manifest Inno Setup: `AppVersion=1.0.NNN`, `AppVerName`, `OutputBaseFilename` đúng phiên bản.
2. Build Rust Windows release với `CADTBT360_RELEASE_VERSION=CADTBT360_V1.NNN`.
3. Kiểm tra Windows FileDescription ghi đúng V1.NNN, kiểm tra MZ và công ty phát hành.
4. Setup và Portable phải chứa `cadcraft.exe` phiên bản mới và `cadcraft-cli.exe`.
5. Kiểm thử smoke cài đặt/gỡ cài đặt, CLI `--version`, checksum, artifact upload và provenance phải PASS.
6. Không liên kết artifact hoặc báo thành công trước khi workflow đã SUCCESS.
7. Các số phiên bản trong hướng dẫn, link download, manifest và báo cáo phải đồng nhất.

**Lưu ý:** `cadcraft-cli --version` dùng phiên bản crate Rust `0.13.0` tại thời điểm V1.024, đây là **phiên bản CLI/engine riêng**, không phải phiên bản thương mại CADTBT360. Không dùng số crate Rust để thay cho phiên bản Setup/EXE.

## Quy trình tạo lần phát hành kế tiếp

Sao chép manifest .iss và workflow của lần trước, tăng số ở **toàn bộ** phần metadata/đường dẫn/kiểm thử/zip/artifact, giữ branch trigger trỏ đúng branch chứa code và kiểm tra chính xác SHA. Không tạo thêm CI test sử dụng nhầm EXE cũ nếu mục tiêu là nghiệm thu tính năng thuộc bản mới. Tránh commit có thể hủy workflow phát hành đang chạy (concurrency `cancel-in-progress`).
