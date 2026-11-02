# BIM CỘNG TÁC | bim-issue-workflow

Branch: `feature/bim-issue-workflow`. Base: main (2026-11-02).

## Phạm vi riêng
Trạng thái Issue và phê duyệt độc lập; cấm sửa sau đóng; validate deadline.

## File sở hữu chính
`SOURCE_CAD_R65/apps/cadcraft/src/bim_issue_workflow.rs`

## Điều kiện merge
- Không làm sai giao diện shell CAD đã có; không sử dụng ảnh mockup làm ảnh EXE.
- Có kiểm thử phù hợp: unit tests chuyển trạng thái + quyền duyệt.
- Không sửa trực tiếp module của nhánh khác; mọi điều chỉnh `bim_collab.rs` tích hợp qua PR sau khi module chức năng đã được kiểm tra.
- Chỉ báo PASS khi CI trả kết quả thực tế.

## Trạng thái ban đầu
Nhánh đã được tạo; phạm vi công việc được cố định. Chưa có bằng chứng build và nghiệm thu Windows.
