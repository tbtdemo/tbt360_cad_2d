# CAD 360 Việt Nam — Hướng dẫn AI Agent, MCP, CLI và chụp ảnh nghiệm thu Windows

> Trạng thái: quy trình có kiểm thử tự động với CLI/MCP headless; thao tác với cửa sổ desktop cần kiểm chứng trên máy Windows thật. Không nhầm **ảnh render từ dữ liệu CAD** với **ảnh chụp cửa sổ ứng dụng**. Ghi commit SHA tương ứng khi chụp.

## 1. Phạm vi có thật trong mã nguồn

| Khả năng | Thành phần / lệnh | Ghi chú |
|---|---|---|
| Agent gửi lệnh CAD | MCP `command_line`, `script`, `execute` | Có giao thức stdio JSON-RPC |
| Kiểm tra hình học và danh sách lệnh | `inspect_drawing`, `query_entities`, `list_commands` | Không cần UI |
| Render bản vẽ PNG | MCP `render`; CLI `run --export file.png` | Ảnh do renderer CAD tạo |
| Lưu bản vẽ | MCP `save`; CLI `run --save file.dxf` | Có thể mở lại để kiểm tra |
| AI điều khiển bản vẽ đang mở trên Windows | `cadcraft --control 7979` + `cadcraft-cli mcp --connect 127.0.0.1:7979` | Chỉ khi phần mềm thực sự được chạy với control port |
| Chụp đúng cửa sổ đang mở | MCP `screenshot` | Chỉ có khi MCP kết nối ứng dụng có UI |
| Tác nhân ngôn ngữ tự suy luận và tự chạy quy trình không giám sát | **Chưa được chứng minh đầy đủ** | MCP là lớp công cụ; cần client/agent ngoài, phê duyệt và chính sách an toàn |

**Lưu ý quan trọng:** Chạy `cadcraft-cli mcp` *không có* `--connect` tạo phiên headless riêng, không cập nhật cửa sổ CAD đang mở. Để tận mắt thấy đối tượng xuất hiện trong CAD 360, bắt buộc dùng `--connect`.

## 2. Cách A — Agent dùng CLI để tạo DXF và PNG không cần mở ứng dụng

### Bước 1. Lấy CLI đã build

1. Mở workflow **CAD360 Agent MCP CLI DXF PNG Windows Proof** trên GitHub Actions.
2. Chọn lượt chạy thành công tương ứng commit muốn kiểm chứng.
3. Mục **Artifacts** → tải `CAD360_Agent_CLI_MCP_Windows_DXF_PNG_Proof`.
4. Giải nén vào `C:\CAD360-Agent\`. Trong gói phải có `cadcraft-cli.exe`, `CAD360_AI_AGENT_DRAWING.dxf`, `CAD360_AI_AGENT_RENDER.png`, `CAD360_AI_AGENT_INSPECT.json`, `SHA256.txt`. Nếu gói chưa có, workflow chưa đạt.
5. Mở PowerShell trong thư mục trên. Kiểm tra `.\cadcraft-cli.exe --version`.

### Bước 2. Tự ra lệnh tạo bản vẽ mới

Chạy **PowerShell**:

```powershell
cd C:\CAD360-Agent
.\cadcraft-cli.exe run --metric `
  --cmd 'line {"points":[[0,0],[100,0]]}' `
  --cmd 'line {"points":[[100,0],[100,60]]}' `
  --cmd 'line {"points":[[100,60],[0,60]]}' `
  --cmd 'line {"points":[[0,60],[0,0]]}' `
  --cmd 'circle {"center":[10,10],"radius":4}' `
  --cmd 'circle {"center":[90,50],"radius":4}' `
  --save C:\CAD360-Agent\KT_AI_01.dxf `
  --export C:\CAD360-Agent\KT_AI_01.png --view extents
```

**Kết quả mong đợi:** Có tệp DXF gồm 4 đoạn LINE và 2 CIRCLE và PNG ảnh kỹ thuật của hình vừa tạo. Lệnh tạo bản vẽ, không dùng dữ liệu có sẵn trên màn hình.

### Bước 3. Mở lại và kiểm tra

```powershell
.\cadcraft-cli.exe info C:\CAD360-Agent\KT_AI_01.dxf
.\cadcraft-cli.exe convert C:\CAD360-Agent\KT_AI_01.dxf C:\CAD360-Agent\KT_AI_01_again.png --view extents
Get-FileHash C:\CAD360-Agent\KT_AI_01.dxf -Algorithm SHA256
Get-FileHash C:\CAD360-Agent\KT_AI_01.png -Algorithm SHA256
```

Kiểm tra `info` xác nhận có Line và Circle. Mở hai ảnh PNG và đối chiếu đường bao, hai lỗ tròn.

### Bước 4. Lấy ảnh minh chứng

- **Ảnh A01**: PowerShell hiển thị lệnh và kết quả tạo DXF/PNG, cùng đường dẫn các tệp.
- **Ảnh A02**: Mở `KT_AI_01.png` bằng Photos, toàn bộ bản vẽ rõ ràng.
- **Ảnh A03**: PowerShell `info KT_AI_01.dxf`, có danh sách/đếm Line và Circle.
- **Ảnh A04**: Mở DXF bằng CAD 360, chụp toàn cửa sổ có đầy đủ ribbon, bản vẽ và dòng lệnh.
- Chụp bằng **Win + Shift + S** → chọn vùng cửa sổ → lưu PNG, hoặc **Alt + PrtScn** rồi dán vào Paint và lưu PNG. Không cắt xén phần kiểm tra quan trọng.
- Dán A01–A04 vào báo cáo nghiệm thu kèm commit, phiên bản EXE, Windows và thời gian chụp.

## 3. Cách B — MCP điều khiển và vẽ trực tiếp lên cửa sổ CAD Windows

### Bước 1. Chạy ứng dụng có cổng điều khiển

Tìm đúng `CADTBT360_V1.023_RoadQTO_RC.exe` bên trong bản Portable RC hoặc thư mục cài đặt. Đường dẫn dưới đây **chỉ là ví dụ**, thay bằng đường dẫn thực tế của máy:

```powershell
& 'C:\Program Files\CAD TBT 360 Road QTO RC\CADTBT360_V1.023_RoadQTO_RC.exe' --control 7979
```

Hoặc đặt `$env:CADCRAFT_CONTROL_PORT='7979'` trước khi chạy ứng dụng. Chỉ sử dụng cổng trên `127.0.0.1`, không mở ra Internet. Cổng có mã xác thực phiên do ứng dụng tạo; không chia sẻ token.

### Bước 2. Kết nối MCP đúng cửa sổ đang chạy

Tạo tệp cấu hình MCP client (chỉ khi client hỗ trợ cấu trúc `mcpServers`), thay đường dẫn exe:

```json
{
  "mcpServers": {
    "cad360": {
      "command": "C:\\CAD360-Agent\\cadcraft-cli.exe",
      "args": ["mcp", "--connect", "127.0.0.1:7979"]
    }
  }
}
```

Khởi động lại MCP client để nó nạp cấu hình. Nếu báo không kết nối được, xác nhận CAD 360 đã chạy với `--control 7979`, không chuyển sang headless vì sẽ sửa một bản vẽ khác.

### Bước 3. Gửi lệnh vẽ theo đúng thứ tự từ AI agent

Đưa từng yêu cầu tool cho agent/client (đây là **minh họa lời gọi tool**, không phải lệnh PowerShell):

1. `list_commands {"filter":"line"}` → đọc thông số lệnh trước khi dùng.
2. `new_drawing {"metric":true}` → bản vẽ mét mới (chỉ thực hiện khi được phép tạo bản mới).
3. `execute {"command":"line","params":{"points":[[0,0],[100,0]]}}`
4. `execute {"command":"line","params":{"points":[[100,0],[100,60]]}}`
5. `execute {"command":"circle","params":{"center":[10,10],"radius":4}}`
6. `inspect_drawing {"entities":false}` → xác minh số đối tượng.
7. `query_entities {"type":"Circle"}` → đọc tâm, bán kính và handle.
8. `render {"width":1600,"height":1000,"fit":true}` → lấy ảnh **render từ hình học CAD**.
9. `screenshot {"path":"C:\\CAD360-Agent\\KT_AI_MCP_WINDOW.png"}` → chụp **cửa sổ CAD Windows đang mở**. Tool chỉ có khi MCP đã nối đúng app.
10. `save {"path":"C:\\CAD360-Agent\\KT_AI_MCP_DRAWING.dxf"}` → lưu bản vẽ để kiểm tra độc lập.

### Bước 4. Chụp ảnh và đối chiếu kết quả

- **Ảnh B01**: Toàn màn hình ứng dụng, có tên bản vẽ và vùng Model (sau lệnh `new_drawing`).
- **Ảnh B02**: Sau lệnh LINE/CIRCLE, nhìn rõ các đối tượng xuất hiện trên canvas, kèm thời gian/commit.
- **Ảnh B03**: Giao diện MCP client với `execute` thành công và `inspect_drawing` hiển thị đếm đối tượng.
- **Ảnh B04**: File PNG của `render` với kích thước đã yêu cầu.
- **Ảnh B05**: File PNG từ `screenshot`, phải thấy **menu, toolbar, canvas CAD và đối tượng**; không được lấy render thay screenshot.
- **Ảnh B06**: Mở lại DXF vừa lưu trong CAD 360, đối chiếu kích thước và số lượng đối tượng.
- Nếu các tool thất bại, chụp thêm **Ảnh B07** thông báo lỗi MCP/ứng dụng và lưu log; không ghi “đã đạt”.

## 4. Kiểm thử tự động trên GitHub — CI cho AI Agent

Workflow `.github/workflows/cadtbt360-agent-mcp-cli-render-windows.yml` thực hiện:

1. Lấy đúng commit và chuẩn bị Rust Windows.
2. Chạy kiểm thử thư viện MCP (bao gồm tool vẽ và render theo giao thức).
3. Chạy integration test CLI `agent_render` và kiểm thử kết nối MCP.
4. Build `cadcraft-cli.exe` thực trên Windows runner.
5. Tạo bản vẽ DXF có 4 LINE, 2 CIRCLE và render PNG.
6. Kiểm tra chữ ký PNG, kích thước DXF, mở lại DXF và lưu kết quả `info`.
7. Tạo `SHA256.txt` và tải các file làm artifact.

**Điều CI chưa chứng minh:** thao tác trực quan trong bản EXE có GPU, toàn bộ chức năng CAD của khách hàng, khả năng tự động cài MCP client, chụp UI thực khi chạy desktop trong môi trường người dùng. Các nội dung này yêu cầu nghiệm thu cách B.

## 5. Bảng kiểm tra nghiệm thu

| STT | Kiểm tra | Tiêu chí đạt | Ảnh |
|---|---|---|---|
| 1 | CLI khởi động trên Windows | `--version` trả về 0 | A01 |
| 2 | AI/CLI vẽ hình | DXF có đúng 4 LINE, 2 CIRCLE | A03 |
| 3 | Render bản vẽ | PNG mở được, hình học đúng | A02 |
| 4 | Mở lại DXF trên CAD 360 | Cửa sổ thực hiển thị đúng | A04 |
| 5 | MCP kết nối CAD đang chạy | Không ở headless, không lỗi token/cổng | B01/B03 |
| 6 | Lệnh MCP làm đổi canvas thật | LINE/CIRCLE xuất hiện | B02 |
| 7 | Render MCP | Có PNG hình học | B04 |
| 8 | Screenshot MCP | Có ảnh cửa sổ (toolbar, canvas) | B05 |
| 9 | Lưu và mở lại bản vẽ | Đối tượng và tỷ lệ không thay đổi | B06 |

**Chỉ kết luận nghiệm thu hoàn tất sau khi đủ minh chứng A01–A04 và B01–B06.**
