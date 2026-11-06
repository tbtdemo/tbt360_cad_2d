//! CAD 360 stage258: actual in-app, human-reviewed deterministic room drafting.
//! This is deliberately not an LLM/image recognizer or external-agent approval UI.
use cadcraft_engine::Session;
use serde_json::json;

#[derive(Clone, Debug)]
struct Preview {
    uid: u64,
    epoch: u64,
    revision: u64,
    text: String,
    points: [[f64;2];2],
}

#[derive(Default)]
pub struct AiReview {
    pub open: bool,
    prompt: String,
    preview: Option<Preview>,
    message: String,
}

fn parse_room(text:&str)->Result<[[f64;2];2],String> {
    let normalized=text.trim().to_lowercase().replace(','," ");
    let words:Vec<&str>=normalized.split_whitespace().collect();
    if words.len()!=9 || words[0]!="vẽ" || words[1]!="phòng" || words[2]!="rộng"
        || words[4]!="dài" || words[6]!="tại" {
        return Err("Hỗ trợ: vẽ phòng rộng 6000 dài 4000 tại 0,0".into());
    }
    // Comma separator becomes whitespace: 9 tokens.
    let width=words[3].parse::<f64>().map_err(|_|"Chiều rộng không hợp lệ")?;
    let height=words[5].parse::<f64>().map_err(|_|"Chiều dài không hợp lệ")?;
    let x=words[7].parse::<f64>().map_err(|_|"Tọa độ X không hợp lệ")?;
    let y=words[8].parse::<f64>().map_err(|_|"Tọa độ Y không hợp lệ")?;
    if ![width,height,x,y].iter().all(|v|v.is_finite() && v.abs()<=1e9)
        || width<=0.0 || height<=0.0 || x+width>1e9 || y+height>1e9 {
        return Err("Kích thước hoặc tọa độ nằm ngoài giới hạn CAD".into());
    }
    Ok([[x,y],[x+width,y+height]])
}

impl AiReview {
    pub fn show(&mut self,ctx:&egui::Context,session:&mut Session) {
        if !self.open {return;}
        let mut open=self.open;
        egui::Window::new("CAD 360 - AI Review (ban gioi han)")
            .open(&mut open).resizable(true).default_width(470.0).show(ctx,|ui| {
            ui.label("Lenh mau: ve phong rong 6000 dai 4000 tai 0,0");
            ui.label("Nhập tiếng Việt có dấu: vẽ phòng rộng 6000 dài 4000 tại 0,0");
            ui.text_edit_singleline(&mut self.prompt);
            if ui.button("1. Xem trước").clicked() {
                self.preview=None;
                let next=(|| {
                    let p=parse_room(&self.prompt)?;
                    let s=session.state().map_err(|e|e.to_string())?;
                    if session.prompting() {return Err("Kết thúc lệnh CAD đang chạy trước khi Preview".into());}
                    if s.block_edit.is_some() || s.undo_ctl.group.is_some() {
                        return Err("Không Preview khi Block Editor hoặc nhóm Undo đang mở".into());
                    }
                    Ok(Preview {uid:s.uid,epoch:session.doc_epoch,revision:s.revision,
                        text:self.prompt.clone(),points:p})
                })();
                match next {Ok(p)=>{self.preview=Some(p);self.message="Preview chưa ghi đối tượng".into();},
                    Err(e)=>self.message=e}
            }
            if let Some(p)=self.preview.as_ref() {
                ui.separator();
                ui.label(format!("RECTANG: ({}, {}) -> ({}, {})",p.points[0][0],p.points[0][1],p.points[1][0],p.points[1][1]));
                ui.label(format!("UID: {} | Epoch: {} | Revision: {}",p.uid,p.epoch,p.revision));
                ui.label("Chỉ xác nhận nếu đã kiểm tra tọa độ, tỷ lệ và bản vẽ.");
            }
            let can_commit=self.preview.is_some();
            if ui.add_enabled(can_commit,egui::Button::new("2. Tôi đồng ý - Vẽ vào CAD")).clicked() {
                if let Some(p)=self.preview.take() {
                    let permitted=session.state().is_ok_and(|s|s.uid==p.uid && s.revision==p.revision
                        && session.doc_epoch==p.epoch && s.block_edit.is_none()
                        && s.undo_ctl.group.is_none() && s.undo_ctl.control==cadcraft_engine::undoctl::UndoControl::All);
                    if !permitted || session.prompting() || self.prompt!=p.text {
                        self.message="Bản vẽ, lệnh hoặc Revision đã thay đổi. Hãy Preview lại.".into();
                    } else {
                        let result=session.execute("rectang",&json!({"p1":p.points[0],"p2":p.points[1]}));
                        match result {Ok(_)=>self.message="Đã gửi RECTANG vào CAD Engine. Dùng Undo/Redo chuẩn để kiểm tra.".into(),
                            Err(e)=>self.message=format!("Không thực hiện: {e}")}
                    }
                }
            }
            if ui.button("Hủy phương án").clicked() {self.preview=None;self.message="Đã hủy Preview".into();}
            if !self.message.is_empty(){ui.separator();ui.label(&self.message);}
            ui.small("Đây là công cụ vẽ phòng quy tắc, chưa có nhận dạng ảnh, AI LLM hay duyệt OIDC.");
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
        let p=parse_room("vẽ phòng rộng 6000 dài 4000 tại 0,0").unwrap();
        assert_eq!(p,[[0.0,0.0],[6000.0,4000.0]]);
        assert!(parse_room("vẽ phòng rộng -1 dài 4000 tại 0,0").is_err());
        assert!(parse_room("vẽ phòng rộng NaN dài 4000 tại 0,0").is_err());
        assert!(parse_room("xóa tất cả").is_err());
    }
}
