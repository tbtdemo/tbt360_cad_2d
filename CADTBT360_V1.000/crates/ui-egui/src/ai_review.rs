//! CAD 360 V1.025 stage259: local human-reviewed native drawing commands.
//! Deliberately deterministic: no image inference, arbitrary LLM commands, or OIDC impersonation.
use cadcraft_engine::Session;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Preview {
    uid: u64,
    epoch: u64,
    revision: u64,
    text: String,
    steps: Vec<(&'static str, Value)>,
    description: String,
    /// For raster plans: exact preview snapshot for source revalidation before any CAD mutation.
    image_evidence: Option<(Value, Value, Vec<u8>)>,
}

#[derive(Default)]
pub struct AiReview {
    pub open: bool,
    prompt: String,
    image_path: String,
    image_scale: String,
    preview: Option<Preview>,
    message: String,
}

fn number(raw:&str)->Result<f64,String> {
    let x=raw.parse::<f64>().map_err(|_|format!("Số không hợp lệ: {raw}"))?;
    if !x.is_finite() || x.abs()>1e9 {return Err("Tọa độ ngoài giới hạn".into());}
    Ok(x)
}

fn pair(raw:&str)->Result<[f64;2],String> {
    let mut parts=raw.split(',');
    let x=number(parts.next().ok_or("Thiếu X")?)?;
    let y=number(parts.next().ok_or("Thiếu Y")?)?;
    if parts.next().is_some() {return Err("Tọa độ cần dạng X,Y".into());}
    Ok([x,y])
}

fn nonzero(a:[f64;2],b:[f64;2])->Result<(),String> {
    if a==b {return Err("Hai điểm trùng nhau".into());}
    Ok(())
}

fn parse_draft(text:&str)->Result<(&'static str,Value,String),String> {
    let normalized=text.trim().to_lowercase();
    let words:Vec<&str>=normalized.split_whitespace().collect();
    match words.as_slice() {
        ["vẽ","phòng","rộng",w,"dài",h,"tại",origin] => {
            let width=number(w)?;let height=number(h)?;let a=pair(origin)?;
            if width<=0.0||height<=0.0 {return Err("Kích thước phải lớn hơn 0".into());}
            let b=[a[0]+width,a[1]+height];
            if !b.iter().all(|n|n.is_finite()&&n.abs()<=1e9) {return Err("Kích thước vượt giới hạn".into());}
            Ok(("rectang",json!({"p1":a,"p2":b}),format!("Phòng RECTANG: ({},{}) đến ({},{})",a[0],a[1],b[0],b[1])))
        }
        ["vẽ","đường","thẳng","từ",from,"đến",to] | ["vẽ","line","từ",from,"đến",to] => {
            let a=pair(from)?;let b=pair(to)?;nonzero(a,b)?;
            Ok(("line",json!({"points":[a,b]}),format!("LINE: ({},{}) đến ({},{})",a[0],a[1],b[0],b[1])))
        }
        ["vẽ","hình","chữ","nhật","từ",from,"đến",to] => {
            let a=pair(from)?;let b=pair(to)?;
            if a[0]==b[0]||a[1]==b[1] {return Err("Hình chữ nhật cần chiều rộng và cao khác 0".into());}
            Ok(("rectang",json!({"p1":a,"p2":b}),format!("RECTANG: ({},{}) đến ({},{})",a[0],a[1],b[0],b[1])))
        }
        ["vẽ","hình","tròn","tâm",center,"bán","kính",radius] => {
            let p=pair(center)?;let r=number(radius)?;
            if r<=0.0 {return Err("Bán kính phải lớn hơn 0".into());}
            Ok(("circle",json!({"center":p,"radius":r}),format!("CIRCLE: tâm ({},{}) bán kính {}",p[0],p[1],r)))
        }
        ["vẽ","polyline","qua",vertices] => {
            let raw:Vec<&str>=vertices.split(';').collect();
            if !(2..=64).contains(&raw.len()) {return Err("POLYLINE yêu cầu từ 2 đến 64 điểm".into());}
            let pts:Vec<[f64;2]>=raw.iter().map(|p|pair(p)).collect::<Result<_,_>>()?;
            if pts.windows(2).any(|p|p[0]==p[1]) {return Err("POLYLINE có đoạn dài bằng 0".into());}
            Ok(("pline",json!({"vertices":pts,"closed":false}),
                format!("POLYLINE: {} đỉnh, {} đoạn",pts.len(),pts.len()-1)))
        }
        ["vẽ","cung","tâm",center,"bán","kính",radius,"từ",start,"đến",end] => {
            let p=pair(center)?;let r=number(radius)?;let a=number(start)?;let b=number(end)?;
            if r<=1e-9 || r>1e8 || a<0.0 || a>=360.0 || b<=a || b>360.0 {
                return Err("Cung yêu cầu bán kính > 0 và góc 0 <= bắt đầu < kết thúc <= 360".into());
            }
            if p.iter().any(|v|v.abs()+r>1e9) {return Err("Cung vượt giới hạn tọa độ".into());}
            Ok(("arc",json!({"center":p,"radius":r,"start":a,"end":b}),
                format!("ARC tâm ({},{}) R={} từ {}° đến {}°",p[0],p[1],r,a,b)))
        }
        ["vẽ","tường","từ",from,"đến",to,"dày",thick] => {
            let a=pair(from)?;let b=pair(to)?;nonzero(a,b)?;
            let width=number(thick)?;
            let dx=b[0]-a[0];let dy=b[1]-a[1];let len=dx.hypot(dy);
            if !len.is_finite() || len<=1e-9 || width<=0.0 || width>len*100.0 {
                return Err("Trục tường hoặc chiều dày không hợp lệ".into());
            }
            let nx=-dy/len*width/2.0;let ny=dx/len*width/2.0;
            let pts=[[a[0]+nx,a[1]+ny],[b[0]+nx,b[1]+ny],
                [b[0]-nx,b[1]-ny],[a[0]-nx,a[1]-ny]];
            if pts.iter().flatten().any(|v|!v.is_finite()||v.abs()>1e9) {
                return Err("Tường vượt giới hạn tọa độ".into());
            }
            Ok(("pline",json!({"vertices":pts,"closed":true}),
                format!("TƯỜNG HÌNH HỌC: trục ({},{}) đến ({},{}), dày {} (PLINE kín)",a[0],a[1],b[0],b[1],width)))
        }
        _ => Err("Lệnh hỗ trợ: vẽ phòng rộng 6000 dài 4000 tại 0,0; vẽ đường thẳng từ 0,0 đến 100,0; vẽ hình chữ nhật từ 0,0 đến 100,50; vẽ hình tròn tâm 0,0 bán kính 50".into())
    }
}

/// Stage261: two separate wall pieces (an opening), a native leaf LINE and a native swing ARC.
/// Door dimensions and the opening side are *explicit*; no image/semantic inference occurs.
fn parse_door_wall(text:&str)->Result<Option<(Vec<(&'static str,Value)>,String)>,String> {
    let normalized=text.trim().to_lowercase();
    let w:Vec<&str>=normalized.split_whitespace().collect();
    let ["vẽ","tường","có","cửa","từ",from,"đến",to,"dày",thick,"cửa",opening,"tại",at,"mở",direction]=w.as_slice() else {
        return Ok(None);
    };
    if *direction!="trái" && *direction!="phải" {
        return Err("Hướng mở chỉ hỗ trợ trái hoặc phải".into());
    }
    let a=pair(from)?;let b=pair(to)?;nonzero(a,b)?;
    let thickness=number(thick)?;let door=number(opening)?;let center=number(at)?;
    let dx=b[0]-a[0];let dy=b[1]-a[1];let length=dx.hypot(dy);
    if !length.is_finite() || length<1e-6 || thickness<=0.0 || door<=0.0
        || door>=length || thickness>=length || center-door/2.0<=1e-6
        || center+door/2.0>=length-1e-6 || thickness>door {
        return Err("Cửa phải nằm hoàn toàn trong tường, chiều rộng lớn hơn chiều dày và không chạm mép".into());
    }
    let ux=dx/length;let uy=dy/length;
    let nx=-uy*thickness/2.0;let ny=ux*thickness/2.0;
    let start=[a[0]+ux*(center-door/2.0),a[1]+uy*(center-door/2.0)];
    let end=[a[0]+ux*(center+door/2.0),a[1]+uy*(center+door/2.0)];
    let rect=|p:[f64;2],q:[f64;2]| -> Vec<[f64;2]> {
        vec![[p[0]+nx,p[1]+ny],[q[0]+nx,q[1]+ny],
            [q[0]-nx,q[1]-ny],[p[0]-nx,p[1]-ny]]
    };
    let (hinge,angle,leaf)=if *direction=="trái" {
        (start,uy.atan2(ux).to_degrees().rem_euclid(360.0),
            [start[0]-uy*door,start[1]+ux*door])
    } else {
        (end,(-uy).atan2(-ux).to_degrees().rem_euclid(360.0),
            [end[0]+uy*door,end[1]-ux*door])
    };
    // Both swing variants go CCW from their own wall-axis ray toward the native leaf.
    // For the right hinge, the ray points backward and CCW reaches its right-hand side.
    let arc_start=angle;
    let arc_end=angle+90.0;
    let points=[a,b,start,end,hinge,leaf];
    if points.iter().flatten().any(|v|!v.is_finite()||v.abs()>1e9)
        || [a,b,start,end].iter().any(|p|p.iter().any(|v|v.abs()+door>1e9)) {
        return Err("Hình học tường và cửa vượt giới hạn tọa độ".into());
    }
    let steps=vec![
        ("pline",json!({"vertices":rect(a,start),"closed":true})),
        ("pline",json!({"vertices":rect(end,b),"closed":true})),
        ("line",json!({"points":[hinge,leaf]})),
        ("arc",json!({"center":hinge,"radius":door,"start":arc_start,"end":arc_end}))
    ];
    Ok(Some((steps,format!("Tường 2 đoạn + cửa {} tại khoảng cách {} theo trục; bản lề phía {}. Cửa có khoảng mở thực, một LINE cánh và một ARC cung quét (sơ đồ 2D).",door,center,direction))))
}

fn plan_from_text(text:&str)->Result<(Vec<(&'static str,Value)>,String),String> {
    if let Some(door)=parse_door_wall(text)? {return Ok(door);}
    let (command,params,description)=parse_draft(text)?;
    Ok((vec![(command,params)],description))
}

fn commit_steps(session:&mut Session,preview:&Preview)->Result<(),String> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some((args,approved_proposal,source_bytes))=&preview.image_evidence {
        let path=args["path"].as_str().ok_or("Ảnh Preview thiếu đường dẫn")?;
        let bytes=std::fs::read(path).map_err(|e|format!("Không thể đọc lại ảnh trước xác nhận: {e}"))?;
        if &bytes!=source_bytes {return Err("Nội dung ảnh đã đổi sau khi xem trước; cần Preview lại".into());}
        let latest=cadcraft_mcp::raster_vector::preview(args)?;
        if std::fs::read(path).map_err(|e|format!("Không thể kiểm tra ảnh sau vector hóa: {e}"))?!=*source_bytes {
            return Err("Ảnh thay đổi ngay trong lúc vector hóa; không vẽ CAD".into());
        }
        if &latest!=approved_proposal {
            return Err("Ảnh hoặc kết quả phát hiện đã đổi sau Preview; phải đọc ảnh lại".into());
        }
    }
    if !drawing_ready(session) {return Err("CAD đang bận hoặc Undo không khả dụng".into());}
    if preview.steps.len()==1 {
        let (command,params)=&preview.steps[0];
        session.execute(command,params).map_err(|e|e.to_string())?;
        return Ok(());
    }
    let same=|s:&Session,rev:u64,group:bool| {
        s.doc_epoch==preview.epoch && s.state().is_ok_and(|st|
            st.uid==preview.uid && st.revision==rev && st.undo_ctl.group.is_some()==group)
    };
    session.execute("undo",&json!({"mode":"begin"})).map_err(|e|e.to_string())?;
    let mut succeeded=0_u64;
    let mut failure=None;
    for (command,params) in &preview.steps {
        let Some(expected)=preview.revision.checked_add(succeeded) else {
            failure=Some("Revision vượt giới hạn".to_string());break;
        };
        if !same(session,expected,true) {failure=Some("Bản vẽ thay đổi trong lúc AI vẽ".into());break;}
        match session.execute(command,params) {
            Ok(_)=>succeeded+=1,
            Err(e)=>{failure=Some(format!("{command}: {e}"));break;}
        }
    }
    let expected=preview.revision.checked_add(succeeded).ok_or("Revision vượt giới hạn")?;
    if !same(session,expected,true) {
        return Err("Mất quyền sở hữu giao dịch; không tự Undo để tránh xóa dữ liệu khác".into());
    }
    session.execute("undo",&json!({"mode":"end"})).map_err(|e|e.to_string())?;
    if let Some(error)=failure {
        if succeeded>0 && same(session,expected,false) {
            session.execute("undo",&json!({"count":1})).map_err(|e|format!("{error}; rollback lỗi: {e}"))?;
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn preview_raster(session:&Session,path:&str,scale_text:&str)->Result<Preview,String> {
    if !drawing_ready(session) {return Err("CAD đang bận; hãy kết thúc lệnh hiện tại trước khi đọc ảnh".into());}
    if path.trim().is_empty() {return Err("Nhập đường dẫn PNG/JPEG trên máy tính".into());}
    let scale=number(scale_text.trim())?;
    if !(scale>0.0&&scale<=1000000.0) {return Err("Đơn vị CAD / pixel phải nằm trong (0, 1000000]".into());}
    let metadata=std::fs::metadata(path.trim()).map_err(|e|format!("Không đọc được thông tin ảnh: {e}"))?;
    if !metadata.is_file() || metadata.len()>16*1024*1024 {return Err("Ảnh phải là file tối đa 16 MiB".into());}
    let source_bytes=std::fs::read(path.trim()).map_err(|e|format!("Không đọc được ảnh: {e}"))?;
    let args=json!({"path":path.trim(),"preview":true,"unitsPerPixel":scale,
        "openingGapHints":true,"pairedWallOpeningHints":true,"doorLeafHints":true,"maxGapPixels":0,
        "minLengthPixels":12,"maxLines":128,"includeDiagonals":true});
    let proposal=cadcraft_mcp::raster_vector::preview(&args)?;
    if std::fs::read(path.trim()).map_err(|e|format!("Không kiểm tra lại được file ảnh: {e}"))?!=source_bytes {
        return Err("Ảnh đã bị sửa trong khi tạo Preview; hãy đọc lại".into());
    }
    let lines=proposal["lines"].as_array().ok_or("Ảnh không trả về mảng LINE hợp lệ")?;
    if lines.is_empty() {return Err("Không phát hiện đường nét vector đủ dài; ảnh chưa được vẽ".into());}
    if lines.len()>128 {return Err("Ảnh có quá 128 nét; cần lọc vùng ảnh nhỏ hơn".into());}
    let mut steps=Vec::new();
    for line in lines {
        let endpoints=line.as_array().filter(|v|v.len()==2).ok_or("LINE ảnh sai số điểm")?;
        let a:Vec<f64>=endpoints[0].as_array().filter(|v|v.len()==2)
            .ok_or("Tọa độ ảnh A sai")?.iter().map(|v|v.as_f64().ok_or("Tọa độ A không phải số"))
            .collect::<Result<_,_>>()?;
        let b:Vec<f64>=endpoints[1].as_array().filter(|v|v.len()==2)
            .ok_or("Tọa độ ảnh B sai")?.iter().map(|v|v.as_f64().ok_or("Tọa độ B không phải số"))
            .collect::<Result<_,_>>()?;
        if a.iter().chain(b.iter()).any(|v| !v.is_finite()||v.abs()>1e9) || a==b {
            return Err("Ảnh trả về đoạn LINE rỗng hoặc vượt giới hạn CAD".into());
        }
        steps.push(("line",json!({"points":[a,b]})));
    }
    let s=session.state().map_err(|e|e.to_string())?;
    let gap_count=proposal["openingGapCandidateCount"].as_u64().unwrap_or(0);
    let door_count=proposal["doorLeafCandidateCount"].as_u64().unwrap_or(0);
    let summary=format!("Ảnh: {} LINE chỉnh sửa được; {} khoảng mở nghi vấn; {} nét giống cánh cửa. Không tự suy đoán/cắt cửa.",steps.len(),gap_count,door_count);
    Ok(Preview {uid:s.uid,epoch:session.doc_epoch,revision:s.revision,
        text:format!("{}|{}",path,scale_text),steps,description:summary,
        image_evidence:Some((args,proposal,source_bytes))})
}

fn drawing_ready(session:&Session)->bool {
    if session.prompting() {return false;}
    session.state().is_ok_and(|s|s.block_edit.is_none()&&s.undo_ctl.group.is_none()&&(s.undo_ctl.bits()&11)==1)
}

impl AiReview {
    pub fn show(&mut self,ctx:&egui::Context,session:&mut Session) {
        if !self.open {return;}
        let mut open=self.open;
        egui::Window::new("CAD 360 - AI Review (vẽ cơ bản)")
            .open(&mut open).resizable(true).default_width(520.0).show(ctx,|ui| {
            ui.label("Chọn 1 câu lệnh tiếng Việt, kiểm tra Preview trước khi vẽ:");
            ui.label("vẽ phòng rộng 6000 dài 4000 tại 0,0");
            ui.label("vẽ đường thẳng từ 0,0 đến 100,0");
            ui.label("vẽ hình chữ nhật từ 0,0 đến 100,50");
            ui.label("vẽ hình tròn tâm 0,0 bán kính 50");
            ui.label("vẽ polyline qua 0,0;100,0;100,50");
            ui.label("vẽ cung tâm 0,0 bán kính 100 từ 0 đến 90");
            ui.label("vẽ tường từ 0,0 đến 6000,0 dày 200");
            ui.label("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở trái");
            ui.text_edit_singleline(&mut self.prompt);
            ui.separator();
            ui.label("AI ảnh → nét CAD (PNG/JPEG, tập trung các nét đường; chưa tự xác nhận cửa):");
            ui.label("Đường dẫn ảnh cục bộ:");
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Chọn ảnh PNG/JPEG...").clicked() {
                if let Some(path)=rfd::FileDialog::new().add_filter("Ảnh bản vẽ", &["png","jpg","jpeg"]).pick_file() {
                    self.image_path=path.to_string_lossy().to_string();
                    self.preview=None; // Choosing a new image invalidates any earlier approval.
                    self.message="Đã chọn ảnh; hãy Xem trước trước khi vẽ".into();
                }
            }
            ui.text_edit_singleline(&mut self.image_path);
            ui.label("Đơn vị CAD trên mỗi pixel (cần hiệu chuẩn thực tế):");
            if self.image_scale.is_empty() {self.image_scale="1".into();}
            ui.text_edit_singleline(&mut self.image_scale);
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Đọc ảnh → Xem trước các nét CAD").clicked() {
                self.preview=None;
                match preview_raster(session,&self.image_path,&self.image_scale) {
                    Ok(p)=>{self.message=p.description.clone();self.preview=Some(p);}
                    Err(e)=>self.message=format!("Không thể xem trước ảnh: {e}")
                }
            }
            #[cfg(target_arch = "wasm32")]
            ui.label("Đọc ảnh cục bộ hiện hỗ trợ ứng dụng Windows/macOS/Linux, chưa hỗ trợ trình duyệt.");
            if ui.button("1. Xem trước").clicked() {
                self.preview=None;
                let next=(|| {
                    let (steps,description)=plan_from_text(&self.prompt)?;
                    if !drawing_ready(session) {return Err("CAD đang có lệnh/Undo Group hoặc Block Editor: chưa thể Preview".into());}
                    let st=session.state().map_err(|e|e.to_string())?;
                    Ok(Preview {uid:st.uid,epoch:session.doc_epoch,revision:st.revision,
                        text:self.prompt.clone(),steps,description,image_evidence:None})
                })();
                match next {Ok(p)=>{self.preview=Some(p);self.message="Đã xem trước; chưa ghi đối tượng.".into();},Err(e)=>self.message=e}
            }
            if let Some(p)=self.preview.as_ref() {
                ui.separator();
                ui.label(&p.description);
                if p.image_evidence.is_some() {
                    let segments:Vec<([f32;2],[f32;2])>=p.steps.iter().filter_map(|(_,params)| {
                        let points=params.get("points")?.as_array()?;
                        let xy=|k:usize| -> Option<[f32;2]> {
                            let src=points.get(k)?.as_array()?;
                            Some([src.first()?.as_f64()? as f32,src.get(1)?.as_f64()? as f32])
                        };
                        Some((xy(0)?,xy(1)?))
                    }).collect();
                    if !segments.is_empty() {
                        let xs=segments.iter().flat_map(|(a,b)|[a[0],b[0]]);
                        let ys=segments.iter().flat_map(|(a,b)|[a[1],b[1]]);
                        let min_x=xs.clone().fold(f32::INFINITY,f32::min);
                        let max_x=xs.fold(f32::NEG_INFINITY,f32::max);
                        let min_y=ys.clone().fold(f32::INFINITY,f32::min);
                        let max_y=ys.fold(f32::NEG_INFINITY,f32::max);
                        let (response,painter)=ui.allocate_painter(egui::vec2(470.0,240.0),egui::Sense::hover());
                        let bounds=response.rect.shrink(12.0);
                        let sx=bounds.width()/(max_x-min_x).max(0.01);
                        let sy=bounds.height()/(max_y-min_y).max(0.01);
                        let k=sx.min(sy);
                        let position=|point:[f32;2]|egui::pos2(bounds.left()+(point[0]-min_x)*k,
                            bounds.bottom()-(point[1]-min_y)*k);
                        for (a,b) in segments {
                            painter.line_segment([position(a),position(b)],
                                egui::Stroke::new(1.5,egui::Color32::LIGHT_BLUE));
                        }
                    }
                }
                ui.label(format!("UID: {} | Epoch: {} | Revision: {}",p.uid,p.epoch,p.revision));
                ui.label("Kiểm tra tọa độ, đơn vị, đúng bản vẽ trước khi xác nhận.");
            }
            if ui.add_enabled(self.preview.is_some(),egui::Button::new("2. Tôi đồng ý - Vẽ vào CAD")).clicked() {
                if let Some(p)=self.preview.take() {
                    let permitted=drawing_ready(session)&&session.state().is_ok_and(|s|
                        s.uid==p.uid&&s.revision==p.revision&&session.doc_epoch==p.epoch);
                    let input_unchanged=if p.image_evidence.is_some() {
                        format!("{}|{}",self.image_path,self.image_scale)==p.text
                    } else {self.prompt==p.text};
                    if !permitted||!input_unchanged {
                        self.message="Preview cũ hoặc bản vẽ thay đổi; vui lòng Preview lại.".into();
                    } else {
                        match commit_steps(session,&p) {
                            Ok(())=>self.message=format!("Đã thực hiện {} bước CAD. Kiểm tra Undo/Redo.",p.steps.len()),
                            Err(e)=>self.message=format!("CAD từ chối giao dịch: {e}")
                        }
                    }
                }
            }
            if ui.button("Hủy").clicked() {self.preview=None;self.message="Đã hủy".into();}
            if !self.message.is_empty() {ui.separator();ui.label(&self.message);}
            ui.small("Ảnh được vector hóa theo pixel, có thể sai nét; chỉ vẽ sau xác nhận. Chưa phải AI hiểu tường/cửa ngữ nghĩa hoặc phê duyệt OIDC.");
        });
        self.open=open;
        if !open {self.preview=None;}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stage258_room_parser_accepts_vietnamese_and_rejects_invalid_geometry() {
        assert_eq!(parse_draft("vẽ phòng rộng 6000 dài 4000 tại 0,0").unwrap().0,"rectang");
        assert!(parse_draft("vẽ phòng rộng -1 dài 4000 tại 0,0").is_err());
        assert!(parse_draft("vẽ phòng rộng NaN dài 4000 tại 0,0").is_err());
        assert!(parse_draft("xóa tất cả").is_err());
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage262_png_to_native_cad_preview_is_readonly_and_source_changes_are_detected() {
        use image::{GrayImage,Luma};
        let mut img=GrayImage::from_pixel(96,64,Luma([255]));
        for x in 8..=80 {img.put_pixel(x,16,Luma([0]));}
        let filename=format!("stage262-raster-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let path=std::env::temp_dir().join(filename);
        img.save(&path).unwrap();
        let source=path.to_string_lossy().to_string();
        let mut session=Session::default();
        let p=preview_raster(&session,&source,"2").expect("PNG preview vectorization");
        assert!(p.steps.len()>=1);
        assert!(p.steps.iter().all(|(command,_)|*command=="line"),
            "image hints must never silently fabricate CAD doors");
        assert!(p.image_evidence.is_some());
        let (args,accepted,bytes)=p.image_evidence.as_ref().unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(cadcraft_mcp::raster_vector::preview(args).unwrap(),*accepted);
        assert_eq!(session.doc().unwrap().entity_count(),0,
            "image Preview alone must not mutate CAD");
        commit_steps(&mut session,&p).expect("confirmed image vectors inserted in native CAD Engine");
        let inserted=session.doc().unwrap().entity_count();
        assert_eq!(inserted,p.steps.len(),"each detected stroke must be native CAD LINE");
        session.execute("undo",&json!({"count":1})).expect("one grouped native Undo");
        assert_eq!(session.doc().unwrap().entity_count(),0,"Undo must remove all image vectors");
        session.execute("redo",&json!({})).expect("native Redo");
        assert_eq!(session.doc().unwrap().entity_count(),inserted,"Redo must restore every LINE");
        for x in 8..=80 {img.put_pixel(x,16,Luma([255]));}
        img.save(&path).unwrap();
        let changed=cadcraft_mcp::raster_vector::preview(args).unwrap();
        assert_ne!(changed,*accepted,"modified source must force a new preview");
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn stage261_door_wall_keeps_a_real_gap_and_four_editable_native_entities() {
        let (steps,_) = plan_from_text("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở trái").unwrap();
        assert_eq!(steps.len(),4);
        assert_eq!(steps.iter().map(|s|s.0).collect::<Vec<_>>(),vec!["pline","pline","line","arc"]);
        assert_eq!(steps[0].1["vertices"],json!([[0.0,100.0],[2550.0,100.0],[2550.0,-100.0],[0.0,-100.0]]));
        assert_eq!(steps[1].1["vertices"],json!([[3450.0,100.0],[6000.0,100.0],[6000.0,-100.0],[3450.0,-100.0]]));
        assert_eq!(steps[2].1["points"],json!([[2550.0,0.0],[2550.0,900.0]]));
        assert_eq!(steps[3].1["radius"],900.0);
        assert_eq!(steps[3].1["start"],0.0);assert_eq!(steps[3].1["end"],90.0);
        let right=plan_from_text("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở phải").unwrap().0;
        assert_eq!(right.len(),4);
        assert_eq!(right[2].1["points"],json!([[3450.0,0.0],[3450.0,-900.0]]));
        assert_eq!(right[3].1["start"],180.0);assert_eq!(right[3].1["end"],270.0);
        assert!(plan_from_text("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 7000 tại 3000 mở trái").is_err());
        assert!(plan_from_text("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 100 mở trái").is_err());
        assert!(plan_from_text("vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở lên").is_err());
        let tilted=plan_from_text("vẽ tường có cửa từ 0,0 đến 3000,4000 dày 200 cửa 900 tại 2500 mở trái").unwrap().0;
        assert_eq!(tilted.len(),4);
    }

    #[test]
    fn stage260_polyline_arc_and_wall_are_editable_native_entities() {
        let (cmd,p,_) = parse_draft("vẽ polyline qua 0,0;100,0;100,50").unwrap();
        assert_eq!(cmd,"pline");assert_eq!(p["vertices"].as_array().unwrap().len(),3);
        assert_eq!(p["closed"],false);
        let (cmd,p,_) = parse_draft("vẽ cung tâm 0,0 bán kính 100 từ 0 đến 90").unwrap();
        assert_eq!(cmd,"arc");assert_eq!(p["start"],0.0);assert_eq!(p["end"],90.0);
        let (cmd,p,_) = parse_draft("vẽ tường từ 0,0 đến 6000,0 dày 200").unwrap();
        assert_eq!(cmd,"pline");assert_eq!(p["closed"],true);
        assert_eq!(p["vertices"],json!([[0.0,100.0],[6000.0,100.0],[6000.0,-100.0],[0.0,-100.0]]));
        assert!(parse_draft("vẽ polyline qua 0,0;0,0").is_err());
        assert!(parse_draft("vẽ cung tâm 0,0 bán kính 100 từ 90 đến 0").is_err());
        assert!(parse_draft("vẽ tường từ 0,0 đến 0,0 dày 200").is_err());
        assert!(parse_draft("vẽ tường từ 0,0 đến 6000,0 dày 0").is_err());
        assert!(parse_draft("vẽ polyline qua 0,0;NaN,20").is_err());
    }
    #[test]
    fn stage259_multiple_native_shapes_are_validated_before_preview() {
        assert_eq!(parse_draft("vẽ đường thẳng từ 0,0 đến 100,50").unwrap().0,"line");
        assert_eq!(parse_draft("vẽ hình chữ nhật từ 0,0 đến 100,50").unwrap().0,"rectang");
        assert_eq!(parse_draft("vẽ hình tròn tâm 0,0 bán kính 50").unwrap().0,"circle");
        assert!(parse_draft("vẽ đường thẳng từ 0,0 đến 0,0").is_err());
        assert!(parse_draft("vẽ hình tròn tâm 0,0 bán kính 0").is_err());
        assert!(parse_draft("vẽ hình chữ nhật từ 0,0 đến 0,50").is_err());
        assert!(parse_draft("vẽ đường thẳng từ 0,0 đến NaN,0").is_err());
    }
}
