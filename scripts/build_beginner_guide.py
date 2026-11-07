#!/usr/bin/env python3
"""Print-ready novice CAD workbook. Schematics are not actual UI screenshots.
Requires python-docx, Pillow. Usage: script.py lesson_source.md output.docx
"""
import re,sys,tempfile
from pathlib import Path
from zipfile import ZipFile
from PIL import Image,ImageDraw,ImageFont
from docx import Document
from docx.shared import Cm,Pt
from docx.enum.text import WD_ALIGN_PARAGRAPH

def font(size):
    for path in ["C:/Windows/Fonts/arial.ttf","/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]:
        if Path(path).exists(): return ImageFont.truetype(path,size)
    return ImageFont.load_default()

def parse(path):
    text=Path(path).read_text(encoding="utf-8")
    heads=list(re.finditer(r"(?m)^### Bài (\d\d) — (.+)$",text))
    lessons=[]
    for i,m in enumerate(heads):
        body=text[m.end():heads[i+1].start() if i+1<len(heads) else len(text)]
        command=re.search(r"(?m)^\*\*Lệnh/trường hợp:\*\*\s*(.+)$",body)
        steps=re.findall(r"(?m)^[1-6]\.\s+(.+)$",body)
        if command is None or len(steps)!=6: raise ValueError("Incomplete lesson "+m.group(1))
        lessons.append((m.group(1),m.group(2),command.group(1).replace(chr(96),""),steps))
    if [x[0] for x in lessons]!=[f"{i:02}" for i in range(1,35)]:
        raise ValueError("Expected 34 unique lessons")
    return lessons

def picture(path,num,title):
    img=Image.new("RGB",(1200,440),"white")
    d=ImageDraw.Draw(img)
    d.rectangle((8,8,1192,430),outline="#B0C6D9",width=4)
    d.rectangle((28,28,1172,85),fill="#173B5E")
    d.text((55,44),"CAD 360 / BAI "+num+" / SO DO GIANG DAY",font=font(24),fill="white")
    for x in range(120,1120,80):d.line((x,100,x,395),fill="#EDF2F7",width=2)
    title=title.upper()
    if "TRIM" in title or "EXTEND" in title or "OFFSET" in title:
        d.line((240,180,970,180),fill="#2079B6",width=8)
        d.line((240,265,970,265),fill="#2079B6",width=8)
        d.line((590,120,590,365),fill="#DA852D",width=6)
        if "TRIM" in title:d.line((820,180,980,180),fill="white",width=15)
    elif any(t in title for t in ["CIRCLE","TRÒN","CUNG","ARC"]):
        d.ellipse((460,115,755,390),outline="#2079B6",width=8)
        d.line((608,252,750,252),fill="#DA852D",width=6)
    elif any(t in title for t in ["AI","ẢNH","PDF"]):
        for j in range(6):d.line((170,135+j*40,490,145+j*40),fill="#95B4C8",width=4)
        d.polygon([(555,220),(650,178),(650,203),(720,203),(720,252),(650,252)],fill="#DA852D")
        d.rectangle((810,140,1050,345),outline="#2079B6",width=8)
    elif any(t in title for t in ["LINE","PLINE","OSNAP","COPY","MOVE"]):
        d.line([(210,340),(425,175),(740,240),(1020,140)],fill="#2079B6",width=8)
        for x,y in [(210,340),(425,175),(740,240),(1020,140)]:
            d.ellipse((x-9,y-9,x+9,y+9),fill="#DA852D")
    else:
        d.rectangle((330,145,920,350),outline="#2079B6",width=8)
        d.line((330,377,920,377),fill="#DA852D",width=5)
    d.text((55,397),"HINH MINH HOA NGUYEN LY - KHONG PHAI ANH CHUP EXE",
           font=font(18),fill="#53738B")
    img.save(path)

def build(source,output):
    lessons=parse(source)
    output=Path(output);output.parent.mkdir(parents=True,exist_ok=True)
    doc=Document();sec=doc.sections[0]
    sec.page_width=Cm(21);sec.page_height=Cm(29.7)
    sec.left_margin=Cm(2.2);sec.right_margin=Cm(1.7)
    sec.top_margin=Cm(1.8);sec.bottom_margin=Cm(1.7)
    doc.styles["Normal"].font.name="Arial";doc.styles["Normal"].font.size=Pt(10)
    sec.header.paragraphs[0].text="TBT VIỆT NAM | CAD 360 | GIÁO TRÌNH NGƯỜI MỚI"
    sec.footer.paragraphs[0].text="CAD 360 V1.026 - BẢN PHÁT TRIỂN / CHƯA NGHIỆM THU"
    p=doc.add_paragraph();p.alignment=WD_ALIGN_PARAGRAPH.CENTER;p.space_before=Pt(130)
    run=p.add_run("CAD 360 VIỆT NAM");run.font.size=Pt(30);run.bold=True
    for line in ["GIÁO TRÌNH CAD 2D CHO NGƯỜI MỚI",
                 "34 BÀI THỰC HÀNH - CÓ HÌNH MINH HỌA",
                 "CÔNG TY CỔ PHẦN CÔNG NGHỆ TBT VIỆT NAM",
                 "BẢN PHÁT TRIỂN - CHƯA NGHIỆM THU EXE WINDOWS"]:
        p=doc.add_paragraph(line);p.alignment=WD_ALIGN_PARAGRAPH.CENTER
    doc.add_page_break()
    doc.add_heading("LỜI NÓI ĐẦU",level=1)
    doc.add_paragraph("Sách dành cho người chưa từng học CAD. Đọc từng bước, thực hành trên bản sao, "
                      "dùng DIST kiểm tra kích thước và Undo khi thao tác sai.")
    doc.add_paragraph("Các hình là sơ đồ tự vẽ để giảng dạy, không phải ảnh chụp phần mềm. "
                      "Tên lệnh đầy đủ được ưu tiên; alias có thể tùy cấu hình.")
    doc.add_heading("MỤC LỤC",level=1)
    toc=doc.add_table(rows=1,cols=2);toc.style="Light Shading Accent 1"
    toc.rows[0].cells[0].text="BÀI";toc.rows[0].cells[1].text="NỘI DUNG"
    for no,title,_,_ in lessons:
        r=toc.add_row();r.cells[0].text=no;r.cells[1].text=title
    doc.add_page_break();doc.add_heading("BẢNG TRA LỆNH TẮT",level=1)
    doc.add_paragraph("Đặt ở đầu sách. Lệnh đầy đủ là lựa chọn chính; alias được đối chiếu trên máy.")
    table=doc.add_table(rows=1,cols=2);table.style="Light Shading Accent 1"
    table.rows[0].cells[0].text="LỆNH";table.rows[0].cells[1].text="CÔNG DỤNG"
    for a,b in [("LINE / L","Đoạn thẳng"),("PLINE / PL","Polyline"),
        ("RECTANG / REC","Hình chữ nhật"),("CIRCLE / C","Hình tròn"),
        ("MOVE / M","Di chuyển"),("COPY / CO","Sao chép"),
        ("OFFSET / O","Song song"),("TRIM / TR","Cắt nét"),
        ("EXTEND / EX","Kéo tới cạnh"),("LAYER / LA","Quản lý lớp"),
        ("HATCH / H","Tô vùng"),("DIM / DIST","Ghi/đo kích thước"),
        ("BLOCK / INSERT","Tạo/chèn Block"),("F3","OSNAP"),
        ("F8","ORTHO"),("F12","Dynamic Input"),
        ("ESC","Hủy"),("ENTER","Xác nhận")]:
        r=table.add_row();r.cells[0].text=a;r.cells[1].text=b
    with tempfile.TemporaryDirectory() as temp:
        for no,title,command,steps in lessons:
            doc.add_page_break()
            doc.add_heading("BÀI "+no+" — "+title,level=1)
            doc.add_paragraph("LỆNH/THÔNG SỐ: "+command)
            doc.add_heading("A. THỰC HÀNH TỪNG BƯỚC",level=2)
            for i,step in enumerate(steps,1):
                p=doc.add_paragraph();p.add_run("Bước "+str(i)+". ").bold=True;p.add_run(step)
            doc.add_heading("B. HÌNH MINH HỌA",level=2)
            path=Path(temp)/("bai_"+no+".png");picture(path,no,title)
            p=doc.add_paragraph();p.alignment=WD_ALIGN_PARAGRAPH.CENTER
            p.add_run().add_picture(str(path),width=Cm(15.5))
            p=doc.add_paragraph("Hình "+no+". Sơ đồ minh họa, không phải ảnh chụp phần mềm.")
            p.style="Caption";p.alignment=WD_ALIGN_PARAGRAPH.CENTER
            doc.add_heading("C. KIỂM TRA KẾT QUẢ",level=2)
            doc.add_paragraph("☐ Lệnh nhận và hiển thị đúng prompt.")
            doc.add_paragraph("☐ Kiểm tra đơn vị, kích thước và dữ liệu gốc.")
            doc.add_paragraph("☐ Đã lưu bản sao, có thể Undo để làm lại.")
    doc.add_page_break();doc.add_heading("PHIẾU ĐÁNH GIÁ CUỐI KHÓA",level=1)
    doc.add_paragraph("Học viên: _____________________   Ngày: __________")
    doc.add_paragraph("☐ Vẽ/sửa CAD, bắt điểm, Layer, ghi kích thước.")
    doc.add_paragraph("☐ AI Preview/Approve, DWG/DXF, bản in PDF.")
    doc.core_properties.title="CAD 360 V1.026 - Giáo trình 34 bài cho người mới"
    doc.core_properties.author="TBT Việt Nam"
    doc.save(output)
    with ZipFile(output) as z:
        figures=[name for name in z.namelist() if name.startswith("word/media/")]
        if len(figures)<34: raise RuntimeError("Missing 34 actual illustrations")
    print("PASS",len(lessons),"lessons",len(figures),"illustrations",output)

if __name__=="__main__":
    if len(sys.argv)!=3:sys.exit("Usage: INPUT.md OUTPUT.docx")
    build(sys.argv[1],sys.argv[2])
