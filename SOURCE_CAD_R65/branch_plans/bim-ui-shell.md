# BIM CỘNG TÁC | bim-ui-shell

Branch: `feature/bim-ui-shell`. Base: main (2026-11-02).

## Phạm vi riêng
Bố cục workspace 5 tab; đối chiếu ảnh thực tế; không thay shell CAD ngoài vùng BIM.

## File sở hữu chính
`SOURCE_CAD_R65/apps/cadcraft/src/bim_collab.rs`

## Điều kiện merge
- Không làm sai giao diện shell CAD đã có; không sử dụng ảnh mockup làm ảnh EXE.
- Có kiểm thử phù hợp: UI screenshot Windows + kiểm tra chuyển tab.
- Không sửa trực tiếp module của nhánh khác; mọi điều chỉnh `bim_collab.rs` tích hợp qua PR sau khi module chức năng đã được kiểm tra.
- Chỉ báo PASS khi CI trả kết quả thực tế.

## Trạng thái ban đầu
Nhánh đã được tạo; phạm vi công việc được cố định. Chưa có bằng chứng build và nghiệm thu Windows.
