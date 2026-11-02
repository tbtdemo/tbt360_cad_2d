# BIM CỘNG TÁC | bim-windows-release

Branch: `feature/bim-windows-release`. Base: main (2026-11-02).

## Phạm vi riêng
Build, test, release EXE portable, screenshot đối chiếu bản thật.

## File sở hữu chính
`SOURCE_CAD_R65/.github/workflows/main.yml`

## Điều kiện merge
- Không làm sai giao diện shell CAD đã có; không sử dụng ảnh mockup làm ảnh EXE.
- Có kiểm thử phù hợp: CI Windows PASS + EXE chạy đúng ảnh.
- Không sửa trực tiếp module của nhánh khác; mọi điều chỉnh `bim_collab.rs` tích hợp qua PR sau khi module chức năng đã được kiểm tra.
- Chỉ báo PASS khi CI trả kết quả thực tế.

## Trạng thái ban đầu
Nhánh đã được tạo; phạm vi công việc được cố định. Chưa có bằng chứng build và nghiệm thu Windows.
