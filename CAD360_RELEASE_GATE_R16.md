# CAD360 — R16/30 Windows Candidate: Build Attestation and Release Runbook

Ngày: 03/11/2026. Nhánh: `tbt/cad360-active-release`.

## Phạm vi đã triển khai
- Workflow `.github/workflows/main.yml` dùng concurrency riêng `workflow + ref + SHA`, tắt cancel-in-progress để có thể hoàn tất đầy đủ một lần build của từng commit.
- Nhánh active tiếp tục chạy toàn bộ Windows build/test và tạo Setup/Portable **ứng viên** sau khi đạt các job `needs`; nhánh main được phân biệt là Release.
- Sau bước tạo SHA256, thêm bước xác thực cả file Setup/Portable, ghi `BUILD_INFO.txt` với commit, ref, RunId, RunAttempt và `Status=UNVERIFIED_GUI_CANDIDATE`.
- Artifact Portable upload cùng `SHA256SUMS.txt` và `BUILD_INFO.txt`, artifact Setup upload riêng, cả hai phải tải về rồi đối chiếu hash và `BUILD_INFO` cùng run/commit.
- GitHub Actions step summary ghi rõ mã commit và lưu ý **GUI & benchmark chưa chứng nhận**.

## Trạng thái nghiệm thu
| Điều kiện | Trạng thái R16 |
|---|---|
| Cấu hình push active branch và không tự hủy | Đã commit và rà chuỗi cấu hình |
| Cấu hình target Rust tests và cả workspace | Đã commit, chưa chạy PASS được kiểm chứng |
| Cấu hình cargo build Windows, Setup.exe, Portable.zip | Đã commit |
| Cấu hình manifest, SHA256 verify | Đã commit và rà tĩnh |
| Có GitHub Actions Windows run mới với kết quả success | Chưa xác minh |
| Có artifact ZIP/Setup thực tế | Chưa xác minh |
| Test khởi chạy GUI Windows trên bản cài mới | Chưa xác minh |
| Benchmark CAD 100/300/500/>512 MiB | Chưa thực hiện |
| Cộng tác hai client, server authoritative permissions | Chưa thực hiện |

Connector cho commit workflow runs chỉ truy vấn PR-triggered runs, trả `[]` đối với SHA R16; combined status cũng trả `[]`. Không suy ra workflow đã pass, fail hoặc chưa chạy.

## Cách nghiệm thu từng bản ứng viên có run thành công
1. Mở `https://github.com/tbtdemo/tbt360_cad_2d/actions`, chọn workflow **CAD 360 Viet Nam - TBT Windows CI**, chọn run của nhánh `tbt/cad360-active-release`.
2. Xác minh các nhóm build-core, unified-ui, vi-tbt-branding, regression-tests, bim-module-regressions, qto-xref-smoke đều SUCCESS; Windows Setup/Portable candidate job SUCCESS (không `skipped`).
3. Tải hai artifacts có hậu tố `Candidate` trong cùng một run: `CAD360_VietNam_TBT_Windows_Setup_Candidate`, `CAD360_VietNam_TBT_Windows_Portable_And_SHA256_Candidate`.
4. Giải nén ZIP artifacts, tìm `CAD360_VietNam_TBT_Setup.exe`, `CAD360_VietNam_TBT_Portable.zip`, `SHA256SUMS.txt`, `BUILD_INFO.txt`. Đối chiếu `BUILD_INFO.Commit` với SHA commit của run.
5. Trên Windows PowerShell, chạy `Get-FileHash -Algorithm SHA256 .\\CAD360_VietNam_TBT_Setup.exe` và `Get-FileHash -Algorithm SHA256 .\\CAD360_VietNam_TBT_Portable.zip`; đối chiếu SHA256SUMS.
6. Cài trên máy thử Windows và mở giao diện thực tế: thao tác LINE/PLINE/CIRCLE/MOVE, ESC, undo/redo, file lớn 100/300/500 MiB và bộ dữ liệu BIM nhiều tầng.
7. Chưa đạt bất kỳ điều kiện nào thì ghi blocker, xử lý ở R17+; tuyệt đối không tự đổi Candidate thành Release.

## Định lượng tiến độ
| Đầu việc | Ước lượng kỹ thuật (%) | Chứng cứ còn thiếu |
|---|---:|---|
| I1 Click đầu tiên | 45 | Windows GUI smoke |
| I2 Kẹt chọn/ESC | 45 | Windows GUI |
| L1 Đọc file không hard-cap 512 MiB | 45 | File thật >512 MiB |
| L2 Progress byte/ETA | 70 | Progress parse/render |
| L3 Mượt 300–500 MiB | 15 | RAM/VRAM/pan/zoom benchmark |
| L4 Queue FIFO mở file lớn | 50 | Peak RAM nhiều file |
| C1 Quyền account/floor/discipline/workset | 35 | Quyền server |
| C2 Lock/conflict/offline hai client | 20 | End-to-end |
| Q1 Windows CI/Setup/Portable | 10 | Windows PASS + artifact thực |

Trung bình (45+45+45+70+15+50+35+20+10)/9 = **37,2% ước lượng**, chưa được nghiệm thu. **16 lượt mã nguồn / tối đa 30 lượt kế hoạch**; không đồng nghĩa phần mềm hoàn thành 53,3%.

## R17+ và ngày công dự kiến
- R17–R19: giải quyết compiler/test/runtime failures dựa trên log Windows thật; xác minh artifact và ZIP checksum. Dự kiến kỹ thuật 3–6 ngày công nếu chỉ là lỗi build/test.
- R20–R24: benchmark file lớn 300–500 MiB, theo dõi RAM & render, cải tiến streaming/LOD nếu cần. Khối lượng chưa thể xác định chính xác khi thiếu dữ liệu thật.
- R25–R30: server-side scope/lease, collaboration test hai user và đóng gói sau nghiệm thu. Các nhánh có thể cần hơn 30 lượt nếu phải thay đổi kiến trúc.
- Tổng thời gian còn lại chưa có căn cứ định lượng đáng tin nếu thiếu kết quả CI & benchmark; không hứa lịch phát hành.

## GitHub commits
- `f5d95c61637c64eb448869ca76efca7f81bac99b` workflow/manifest/attestation
- `a5035705dbc767b077118badfabce15fa757e068` batch tracker
