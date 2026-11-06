# CAD 360 V1.025 – Đợt 253: kiểm thử AI ảnh → CAD

## Cấu trúc hiện thực
- Công cụ độc lập: `tools/ai_benchmark/stage253_line_benchmark.py`
- Windows CI: `.github/workflows/cad360-stage253-benchmark.yml`
- Mã nguồn công cụ đánh giá đã bổ sung nhưng **chưa tích hợp vào giao diện EXE**; không quảng cáo đây là AI thị giác mới.
- Công cụ đánh giá chỉ nhận tọa độ LINE CAD đã được công cụ nhận dạng tạo ra và bộ LINE tham chiếu của người kiểm tra, không trực tiếp đọc PNG/JPG.

## Bài 196 – Chuẩn bị bộ ảnh và bộ LINE tham chiếu
1. Chọn ảnh bản vẽ được phép sử dụng và đặt mã `id` cho từng ảnh.
2. Đo đối chiếu các đường LINE trên bản CAD chuẩn; ghi mỗi đường thành `[[x1,y1],[x2,y2]]` trong `referenceLinesCAD`.
3. Xuất các LINE đã được AI vector hóa ra `detectedLinesCAD`. Dùng đúng cùng đơn vị, cùng gốc tọa độ, cùng tỷ lệ.
4. Thiết lập `referenceToleranceCAD` dựa trên yêu cầu sai số thực tế của loại bản vẽ; không mặc định hiểu giá trị 10 là 10 mm.
5. Lưu JSON với `cases`, mỗi ảnh là một phần tử.

Mẫu `benchmark.json`:
```json
{
  "cases": [
    {
      "id": "test-duong-thang",
      "referenceLinesCAD": [[[0,0],[100,0]], [[0,0],[0,100]]],
      "detectedLinesCAD": [[[100,0],[0,0]], [[0,0],[0,100]]],
      "referenceToleranceCAD": 1
    }
  ]
}
```

## Bài 197 – Chạy đánh giá và đọc các chỉ số
1. Kiểm tra bài thử nội bộ: `python tools/ai_benchmark/stage253_line_benchmark.py --self-test`.
2. Chạy bộ ảnh đã chuẩn hóa: `python tools/ai_benchmark/stage253_line_benchmark.py benchmark.json > benchmark-result.json`.
3. `truePositives`: số LINE khớp tối đa 1:1 trong dung sai.
4. `falsePositives`: số LINE AI vẽ dư/chưa ghép.
5. `falseNegatives`: số LINE chuẩn bị bỏ sót.
6. `precision`, `recall`, `f1`: mức khớp hình học, không phải độ đúng về chức năng tường/cửa.
7. `matchedEndpointRmseCAD`: RMSE đầu mút **chỉ của các cặp khớp**; nếu không có cặp khớp là `null`. Luôn đọc kèm số bỏ sót, tránh hiểu RMSE thấp là hoàn hảo.

## Bài 198 – Kiểm chứng và tích hợp tiếp
1. Vào GitHub Actions, chọn workflow `CAD 360 Stage253 Image-to-CAD Benchmark`, kiểm tra status PASS và SHA chính xác.
2. Đối chiếu riêng khả năng mở/chỉnh sửa/Undo/Save/Reopen DXF và DWG trên CAD 360, AutoCAD; các phép thử độc lập này không chứng minh tương thích DWG.
3. Khi tích hợp UI, hiển thị kết quả đánh giá ở tab **HƯỚNG DẪN → 2. SỬ DỤNG AI → 2.10 Đánh giá Precision/Recall/F1 từ nhiều ảnh**.
4. Ưu tiên kế tiếp: đo sai số theo hướng/độ dài LINE, độ đúng topology; kiểm thử nhận dạng cửa/tường trên ảnh thực và liên kết Preview–Confirm–Undo với CAD Engine.

## Tình trạng nghiệm thu
Mới bổ sung code benchmark + workflow; chưa có bằng chứng PASS ở đúng SHA cuối, chưa nghiệm thu Windows EXE/AI thị giác/AutoCAD thực tế. Không cộng tỷ lệ hoàn thiện AI chỉ dựa trên số lượng bài học hay workflow.
