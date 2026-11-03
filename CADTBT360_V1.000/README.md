# CADTBT360_V1.000 — mã nguồn ứng dụng

Thư mục này là workspace Rust chính duy nhất cho CAD TBT 360 phiên bản 1.000.

```powershell
cargo build --release --locked -p cadcraft -p cadcraft-cli
```

Tên crate nội bộ `cadcraft` được giữ để tránh thay đổi toàn bộ Rust package graph/Cargo.lock. EXE công bố được **đổi tên khi đóng gói** thành `CADTBT360.exe`; không mang theo bản CAD gốc dưới dạng executable.

Giữ nguyên `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` và `ATTRIBUTION.md` vì nghĩa vụ đối với mã nguồn mở. Tài liệu R47/R65 và script kiểm toán lịch sử không phải thành phần phát hành.
