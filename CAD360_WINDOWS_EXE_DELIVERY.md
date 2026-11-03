# CAD360 Windows EXE — build thực tế, không phụ thuộc runner Windows

Mốc: B15, bản mã nguồn đã sửa Rust E0308 và bổ sung các thay đổi phục hồi/RAM/viewport gần nhất.

## Hai đường phát hành song song

1. **Windows Inno Setup gốc:** `.github/workflows/tbt-exe-direct.yml` chạy trên Windows runner, tạo Setup, Portable, Source và SHA256.
2. **Ubuntu dự phòng:** `.github/workflows/cad360-ubuntu-windows-exe.yml` dùng `cargo-xwin` biên dịch đúng `x86_64-pc-windows-msvc` cho cả `cadcraft.exe` và `cadcraft-cli.exe`, tạo bộ cài thật với NSIS, xuất Portable ZIP, Source ZIP và SHA256. Không chỉ đổi đuôi một file Linux thành EXE.

Hai job giữ SHA commit riêng. Không hủy các run đóng gói cũ vì phát triển tiếp.

## Điều kiện giao

Chỉ báo hoàn thành nếu GitHub Actions đã upload artifact có `CAD360_VietNam_TBT_Setup.exe`, `CAD360_VietNam_TBT_Portable.zip`, `CAD360_TBT_Source_R15.zip` và `SHA256SUMS.txt`. `TBT-Source-Inventory` không phải bộ cài. Chưa có xác nhận Windows GUI smoke-test trong sandbox thì `GUI_VERIFIED=false`.

Workflow sẽ fail nếu Rust compiler hoặc NSIS tạo file sai; không upload artifact giả.

## Theo dõi

- [Ubuntu cargo-xwin fallback](https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/cad360-ubuntu-windows-exe.yml)
- [Windows Inno Setup](https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/tbt-exe-direct.yml)
