# CAD360 TBT — R29/30: Quyền chỉnh sửa đối tượng CAD/BIM theo tài khoản, tầng và bộ môn

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Phạm vi
Tiếp tục đăng nhập, phân quyền xem/sửa, CAD mouse, file lớn, autosave và Windows CI/Setup/Portable/Source. Tạm dừng đồng bộ giữa các client, conflict reconciliation và clash giữa bộ môn; không thay đổi các module này.

## Lỗi quyền truy cập phát hiện
`enforce_selection_scope` đã kiểm tra đăng nhập, user can_edit, quyền edit trên tầng/bộ môn và danh sách `primary_handles` của viewport. Tuy nhiên `primary_handles` là tập đối tượng **được phép hiển thị** (bao gồm đối tượng của chủ sở hữu khác có chia sẻ can_view), chứ không phải tập được phép sửa. Một tài khoản có thể nhìn thấy đối tượng được chia sẻ chỉ đọc nhưng vẫn giữ handle đó trong selection để thực hiện một số thao tác chỉnh sửa CAD.

## Sửa thực tế R29
1. Thêm `can_edit_scoped_entity(state, slot, meta)`: yêu cầu viewport Edit, đúng tầng/bộ môn, giới hạn owner ở slot nếu khai báo, AuthSession hợp lệ, active_user.can_edit, scope-level edit và owner-level share grant can_edit. Nếu owner chính là current account thì dùng scope edit của owner.
2. `enforce_selection_scope` chỉ giữ các selected handles có `BimEntityMeta` và đủ quyền edit theo hàm trên; không dùng `primary_handles` đơn thuần làm thẩm quyền sửa.
3. Tối ưu cho file lớn: thay full `doc.model.iter()` mỗi frame bằng `doc.model.get(handle)` cho các selected handles, giữ binary_search của `primary_handles`.
4. Thêm test `selection_requires_foreign_owner_edit_grant_not_just_view_grant`: chỉ nhìn thấy không đủ để sửa; grant can_edit mới cho phép.
5. Thêm test `selection_rejects_wrong_floor_discipline_or_reference_view`: sai tầng, sai bộ môn hoặc viewport Reference đều không hợp lệ.
6. Workflow Windows chạy hai tests cụ thể với `--exact` và sau đó full workspace tests.

## Giới hạn an ninh và cần nghiệm thu
- Guard này chỉ ngăn đưa đối tượng ngoài quyền vào **selection CAD UI**; không thể thay thế kiểm tra cấp quyền ở engine mutation, command-line/control server hoặc REST API. Trước phát hành phải chặn server-side trong mọi mutation (Create, Update, Delete, Publish) và kiểm thử privilege escalation trực tiếp API.
- Không có bằng chứng Cargo/Windows CI PASS hoặc artifact EXE/Portable/Source đã chạy được; chưa thử GUI quyền xem/sửa hoặc can thiệp từ control server.
- Cần rà trường hợp entity không có BIM metadata: hiện không được chấp nhận vào selection khi render filter BIM bật. Đối tượng CAD thuần vẫn phải qua chính sách phù hợp theo loại file/project, tránh khóa nhầm CAD thuần hoặc cho phép sửa không được xác thực.
- Các bài test mới là đơn vị và đường chọn giao diện, không phải security audit hoàn chỉnh.

## Tiến độ 8 hạng mục active
| Nhóm | Ước lượng kỹ thuật R29 | Gate còn thiếu |
| --- | ---: | --- |
| LINE/PLINE/CIRCLE/MOVE first click | 45% | GUI E2E |
| ESC/drag/focus/multi-viewport | 45% | GUI E2E |
| OPEN CAD >512MiB | 45% | DXF/DWG thực |
| Progress/ETA | 70% | UI acceptance |
| CAD 300–500MiB mượt | 15% | FPS, RAM, P95 |
| FIFO nhiều file/RAM | 50% | benchmark |
| Tài khoản/cài đặt phân quyền | 35% | server mutation/API tests |
| Windows CI/Setup/Portable/Source | 10% | jobs PASS/artifacts SHA256 và cài thử |

Trung bình **39,4%**, ước lượng chưa nghiệm thu. R29/30 là số lượt thay đổi mã nguồn, không phải % phần mềm hoàn thiện.

## Commit
- `f324fde1ccb2d2d5d88f026ebb00b7992d3f7726`: guard per-owner scope.
- `5f7921492fe8a3d0db6dd50856b2915da36a8bb0`: two authorization regression tests.
- `2fbdaab6de18d9e5bee0ffb0ef9ced43867175b7`: indexed handle lookup performance.
- `088506c8e821d1ca85b10bb476a116aab0e2de5e`: Windows CI targeted gate.
- `3ff540f871838c9e19a634322883753cab5a3e17`: progress tracker.

## R30
Prioritize verified Windows CI and job logs; fix actual build/test failures. Do not claim runnable EXE without Setup/Portable/Source artifacts from the same SHA with matching checksums, and GUI testing. Continue remaining runtime corrections if release gates fail, regardless of the 30-lượt planning cap.
