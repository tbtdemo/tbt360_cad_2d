# CAD360 TBT — R26/30: GPU mesh memory and frame-allocation corrections

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Scope
Active: ổn định thao tác CAD (LINE/PLINE/CIRCLE/MOVE, ESC, viewport), file 100–500/>512MiB, RAM/VRAM/decoder/renderer, autosave/recovery, quyền tài khoản, Windows CI và Setup/Portable/Source. Paused: đồng bộ online/offline, xử lý conflict nhiều client và clash bộ môn. Không tự gỡ bỏ test cũ.

## Sửa đổi R26 — mã nguồn đã commit
1. `SOURCE_CAD_R65/crates/ui-egui/src/gpu.rs`: tạo `mesh_initial_capacity` với ngưỡng **1MiB cấp phát ban đầu trên mỗi lines/tris vector**, thay vì cấp trước toàn bộ dự báo đỉnh và tam giác; các vector tăng theo nhu cầu, không giới hạn tổng số hình được vẽ. Đặc biệt có lợi khi nhiều polyline dùng lineweight tạo quad thay vì hairline.
2. `CanvasCallback::prepare`: bỏ `Vec<u8>` tạm cho 20 `f32` uniform mỗi frame, thay bằng mảng `[u8; 80]` trên stack, vẫn ghi đúng thứ tự các thành phần `a,b,c,d,e` vào GPU buffer.
3. `MeshData::upload_now`: nếu `uploaded` đã có thì return, không làm trống GPU buffers bằng dữ liệu CPU đã được giải phóng trong lần upload trước.
4. Bổ sung tests `gpu::gpu_memory_budget_tests` cho estimate 500MiB và 1024 segments (2048 line vertices); CI Windows targeted gate chạy cùng các bài regression cũ.
5. `TBT_BATCH_PROGRESS.md` cập nhật R26 theo phạm vi ưu tiên.

## Chưa nghiệm thu, tuyệt đối không coi là PASS
- Chưa xác minh cargo check/test Windows thành công; không có con số peak RAM/VRAM, FPS, P95 latency hoặc benchmark 100/300/500/>512MiB thật.
- Chưa xác minh artifact Setup EXE, Portable ZIP, Source ZIP hoặc chạy GUI thật. Workflow đã được cấu hình đóng gói và SHA256, nhưng chỉ artifacts cùng một Windows Actions run SUCCESS mới là bằng chứng.
- Đây không phải streaming GPU hay deferred upload trên mọi đường. GPU upload vẫn có thể cấp phát đủ toàn bộ mesh theo số đỉnh hình học, và cần kiểm soát theo số đo thực.

## Bảng tiến độ 8 đầu việc đang ưu tiên
| Mục | % kỹ thuật ước lượng | Chứng cứ thiếu |
| --- | ---: | --- |
| Click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45 | Windows GUI 100 lần/lệnh |
| ESC/drag/focus/viewport | 45 | GUI E2E |
| OPEN CAD >512MiB | 45 | File CAD lớn thật |
| Đọc file progress/ETA | 70 | UI/phase acceptance |
| CAD 300–500MiB mượt | 15 | FPS, P95, RAM/VRAM |
| FIFO nhiều file/RAM | 50 | File thực 5–10 bản |
| Account permissions | 35 | Quyền server-side |
| Windows Setup/Portable/Source | 10 | CI SUCCESS, files, hashes và chạy thử |

Trung bình 8 đầu việc **39,4% ước lượng kỹ thuật, chưa nghiệm thu**. Lượt **26/30**, còn 4 lượt kế hoạch dự kiến. Sync/conflict/clash không được tính vào mẫu số.

## Commit
- `4f029daf104c1d70aff601eeb06e033abc5da935`: bound mesh initial reservation + stack uniforms.
- `04a42d2eea75557ecc2fb36f229d0ed0a8d769ee`: tests.
- `6639f2e1fd88460a5a28dd340a7cd8fba8e4f860`: fixture index type.
- `afb0661d84b163449072d6aaaa28747c44d25131`: Windows CI targeted gate.
- `dc8c785167c5909caf9bd1db3ccf5899e0bf6ed7`: repeat-upload guard.
- `368c9720e76a84530303df5cd899e73f5bcf8f5b`: tracker.

## Tiếp R27
Ưu tiên kiểm tra tự động lưu và phục hồi toàn bộ geometry/multi-tab; kiểm tra đủ trường hợp file corrupt và dữ liệu mới hơn. Khi Windows CI log/artifacts khả dụng, giải quyết lỗi compiler và cài chạy thực tế thay vì chỉ tăng số lượt.
