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
    command: &'static str,
    params: Value,
    description: String,
}

#[derive(Default)]
pub struct AiReview {
    pub open: bool,
    prompt: String,
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
        _ => Err("Lệnh hỗ trợ: vẽ phòng rộng 6000 dài 4000 tại 0,0; vẽ đường thẳng từ 0,0 đến 100,0; vẽ hình chữ nhật từ 0,0 đến 100,50; vẽ hình tròn tâm 0,0 bán kính 50".into())
    }
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
            ui.text_edit_singleline(&mut self.prompt);
            if ui.button("1. Xem trước").clicked() {
                self.preview=None;
                let next=(|| {
                    let (command,params,description)=parse_draft(&self.prompt)?;
                    if !drawing_ready(session) {return Err("CAD đang có lệnh/Undo Group hoặc Block Editor: chưa thể Preview".into());}
                    let st=session.state().map_err(|e|e.to_string())?;
                    Ok(Preview {uid:st.uid,epoch:session.doc_epoch,revision:st.revision,
                        text:self.prompt.clone(),command,params,description})
                })();
                match next {Ok(p)=>{self.preview=Some(p);self.message="Đã xem trước; chưa ghi đối tượng.".into();},Err(e)=>self.message=e}
            }
            if let Some(p)=self.preview.as_ref() {
                ui.separator();
                ui.label(&p.description);
                ui.label(format!("UID: {} | Epoch: {} | Revision: {}",p.uid,p.epoch,p.revision));
                ui.label("Kiểm tra tọa độ, đơn vị, đúng bản vẽ trước khi xác nhận.");
            }
            if ui.add_enabled(self.preview.is_some(),egui::Button::new("2. Tôi đồng ý - Vẽ vào CAD")).clicked() {
                if let Some(p)=self.preview.take() {
                    let permitted=drawing_ready(session)&&session.state().is_ok_and(|s|
                        s.uid==p.uid&&s.revision==p.revision&&session.doc_epoch==p.epoch);
                    if !permitted||self.prompt!=p.text {
                        self.message="Preview cũ hoặc bản vẽ thay đổi; vui lòng Preview lại.".into();
                    } else {
                        match session.execute(p.command,&p.params) {
                            Ok(_)=>self.message=format!("Đã thực hiện {} trong CAD Engine. Kiểm tra Undo/Redo.",p.command),
                            Err(e)=>self.message=format!("CAD từ chối lệnh: {e}")
                        }
                    }
                }
            }
            if ui.button("Hủy").clicked() {self.preview=None;self.message="Đã hủy".into();}
            if !self.message.is_empty() {ui.separator();ui.label(&self.message);}
            ui.small("Bộ phân tích lệnh quy tắc; chưa phải AI nhận dạng ảnh hoặc phê duyệt OIDC.");
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
