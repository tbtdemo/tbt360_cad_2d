# CAD360 TBT - mốc bàn giao R05

## Quy định đóng gói
- Mọi lượt sửa mã nguồn: commit lên nhánh `tbt/cad360-active-release`, cung cấp link mã nguồn đúng commit.
- Các mốc R05, R10, R15, R20...: bắt buộc build Windows Setup x64, chạy test BIM, kiểm tra EXE và checksum, kèm ZIP mã nguồn; chỉ được phát hành GitHub Release khi mọi bước PASS.
- Tag: `cad360-r05`, `cad360-r10`, ... . Khi mốc bị lỗi CI, giữ trạng thái blocked, tuyệt đối không nhận là đã có bộ cài.
- Mốc phát hành được kích hoạt bằng commit `release(cad360): R05`, `release(cad360): R10`, ... ; không dùng ZIP mã nguồn để thay cho EXE.
- Gói EXE: `CAD360_VietNam_TBT_Setup.exe`; checksum: `SHA256SUMS.txt`; nguồn: `CAD360_TBT_Source_RXX.zip`.
- Source snapshot sau mỗi commit vẫn có tại GitHub Actions `CAD360 IFC Windows Portable`, artifact `CAD360-TBT-Active-Source`.

## Trạng thái R05
- Đã kích hoạt workflow để build bản R05; chỉ chuyển thành PASS khi GitHub Release có file EXE tải được.
- Môi trường test: GitHub-hosted Windows x64 (MSVC, Rust 1.95, Inno Setup 6).
- Mã nguồn: nhánh release đang hoạt động. Không sử dụng `CAD_ORIGINAL_014_REFERENCE` hoặc `QTO_R134_WEB_WORKSPACE` làm binary đầu vào.

## R05 - build retry after YAML repair
- Rebuild with corrected workflow expression. Only a release containing the Windows installer counts as delivered.

## R05 - sửa lỗi biên dịch thực tế
- Sửa 6 lỗi đã được xác nhận trong Windows Cargo log: chrono Local, egui scroll, cấm unsafe FFI, borrow/move suffix.
- Bắt buộc chạy lại các gate trước khi phát hành EXE.
