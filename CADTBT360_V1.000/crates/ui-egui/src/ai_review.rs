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
    selected_opening_guide: Option<usize>,
    selected_door_arc: Option<usize>,
    replace_handles: Option<Vec<String>>,
    /// Exact snapshot of an explicitly user-selected existing wall entity, verified at commit.
    replace_snapshot: Option<cadcraft_doc::Entity>,
    /// Stage271: exact UI inputs which the operator reviewed and must not change silently.
    review_token: Option<String>,
    image_controls: Option<ImageControlSnapshot>,
    /// Exact four owned entity snapshots, preventing silent edits outside Session revision.
    owned_entities: Option<Vec<cadcraft_doc::Entity>>,
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
struct ImageControlSnapshot {
    opening_index: String,
    door_arc_index: String,
    width: String,
    offset: String,
    flip_hinge: bool,
    flip_swing: bool,
}


#[derive(Clone, Debug)]
struct ManagedDoorWall {
    uid: u64,
    epoch: u64,
    revision: u64,
    command_text: String,
    handles: Vec<String>,
    owned_entities: Vec<cadcraft_doc::Entity>,
}


#[derive(Default)]
pub struct AiReview {
    pub open: bool,
    prompt: String,
    image_path: String,
    image_scale: String,
    opening_index: String,
    door_arc_index: String,
    door_width_override: String,
    door_offset_override: String,
    flip_door_hinge: bool,
    flip_door_swing: bool,
    revised_center: String,
    existing_door_center: String,
    existing_door_width: String,
    existing_open_right: bool,
    managed_wall: Option<ManagedDoorWall>,
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

/// Stage263: add only a selected, human-reviewed opening outline as a separate native PLINE.
 /// Existing image strokes are not removed, joined or changed by this proposal.
fn select_opening_guide(preview:&mut Preview,index:usize)->Result<(),String> {
    if preview.selected_door_arc.is_some() {return Err("Chỉ thêm một loại chỉ dẫn: PLINE ô cửa hoặc ARC mở cửa; cần Preview ảnh lại để đổi".into());}
    let Some((_,proposal,_))=preview.image_evidence.as_ref() else {
        return Err("Chỉ chọn gợi ý ô cửa khi đang Preview ảnh".into());
    };
    let candidate=proposal["pairedWallOpeningCandidates"].as_array()
        .and_then(|items|items.get(index)).ok_or("Không tồn tại ứng viên ô cửa này trong Preview")?;
    if candidate["classification"]!="paired-through-wall-gap-unverified" {
        return Err("Phân loại ứng viên chưa được kiểm chứng, không tạo ô cửa".into());
    }
    let shape=candidate["outlineCAD"].as_array().filter(|v|v.len()==4)
        .ok_or("Ứng viên không có biên dạng 4 đỉnh")?;
    let mut coords=Vec::<[f64;2]>::with_capacity(4);
    for vertex in shape {
        let xy=vertex.as_array().filter(|v|v.len()==2).ok_or("Đỉnh ứng viên phải là [X,Y]")?;
        let (Some(x),Some(y))=(xy[0].as_f64(),xy[1].as_f64()) else {
            return Err("Đỉnh ứng viên không phải tọa độ số".into());
        };
        if !x.is_finite() || !y.is_finite() || x.abs()>1e9 || y.abs()>1e9 {
            return Err("Ứng viên vượt giới hạn CAD".into());
        }
        coords.push([x,y]);
    }
    let twice_area: f64=coords.iter().zip(coords.iter().cycle().skip(1))
        .take(4).map(|(a,b)|a[0]*b[1]-b[0]*a[1]).sum();
    if !twice_area.is_finite() || twice_area.abs()<=1e-8 {
        return Err("Biên dạng ô cửa có diện tích bằng 0".into());
    }
    if let Some(previous)=preview.selected_opening_guide {
        if previous==index {return Ok(());}
        preview.steps.pop();
    }
    preview.steps.push(("pline",json!({"vertices":coords,"closed":true})));
    preview.selected_opening_guide=Some(index);
    preview.description=format!("Ảnh: {} LINE nguồn + 1 PLINE hướng dẫn ứng viên ô cửa #{} (không tự cắt tường, không kết luận là cửa).",
        preview.steps.len()-1,index);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn select_door_swing_arc(preview:&mut Preview,index:usize)->Result<(),String> {
    if preview.selected_opening_guide.is_some() {
        return Err("Đang chọn PLINE ô cửa; cần Preview ảnh lại để chuyển sang cung mở cửa".into());
    }
    let Some((args,previous,bytes))=preview.image_evidence.as_ref() else {
        return Err("Chỉ được thêm cung khi có Preview ảnh".into());
    };
    if std::fs::read(args["path"].as_str().ok_or("Không có đường dẫn ảnh")?)
        .map_err(|e|format!("Không đọc được ảnh khi chọn cung: {e}"))?!=*bytes {
        return Err("Ảnh nguồn đã thay đổi; Preview lại trước khi chọn cửa".into());
    }
    let mut reviewed_args=args.clone();
    reviewed_args["createDoorSwingArcs"]=json!(true);
    reviewed_args["confirmDoorSwingArcs"]=json!(true);
    let reviewed=cadcraft_mcp::raster_vector::preview(&reviewed_args)?;
    if reviewed["lines"]!=previous["lines"]
        || reviewed["pairedWallOpeningCandidates"]!=previous["pairedWallOpeningCandidates"] {
        return Err("Ứng viên cửa hoặc nét nguồn đã đổi; phải Preview lại".into());
    }
    let door=reviewed["doorSwingArcs"].as_array().and_then(|arcs|arcs.get(index))
        .ok_or("Không có cung mở tại chỉ số đã chọn; ứng viên này chưa đủ bằng chứng")?;
    if door["classification"]!="confirmed-drafting-arc-not-door-block" {
        return Err("Đề xuất cung chưa được xác minh để tạo CAD".into());
    }
    let center=door["center"].as_array().filter(|v|v.len()==2).ok_or("Cung không có tâm hợp lệ")?;
    let (Some(cx),Some(cy),Some(radius),Some(start),Some(end))=
        (center[0].as_f64(),center[1].as_f64(),door["radius"].as_f64(),
            door["start"].as_f64(),door["end"].as_f64()) else {
        return Err("Cung có thông tin số không hợp lệ".into());
    };
    if ![cx,cy,radius,start,end].iter().all(|v|v.is_finite())
        || cx.abs()>1e9 || cy.abs()>1e9 || radius<=0.0 || radius>1e8 {
        return Err("Cung nằm ngoài giới hạn hình học CAD".into());
    }
    if let Some(previous_index)=preview.selected_door_arc {
        if previous_index==index {return Ok(());}
        preview.steps.pop();
    }
    preview.steps.push(("arc",json!({"center":[cx,cy],"radius":radius,"start":start,"end":end})));
    preview.selected_door_arc=Some(index);
    preview.image_evidence=Some((reviewed_args,reviewed,bytes.clone()));
    preview.description=format!("Ảnh: {} nét LINE nguồn + ARC cung mở #{} được người dùng chọn; chỉ dẫn CAD 2D, chưa phải Door/BIM.",
        preview.steps.len()-1,index);
    Ok(())
}

/// Stage264: operator-reviewed changes to ONLY the extra drafted ARC.
/// Source image LINEs, chosen source candidate and detector evidence stay immutable.
#[cfg(not(target_arch = "wasm32"))]
fn adjust_reviewed_door_arc(preview:&mut Preview, width_text:&str, offset_text:&str, flip_hinge:bool, flip_swing:bool)->Result<(),String> {
    let index=preview.selected_door_arc.ok_or("Chưa chọn cung mở từ ảnh")?;
    let (_,evidence,bytes)=preview.image_evidence.as_ref().ok_or("Thiếu chứng cứ ảnh đã Preview")?;
    let args=&preview.image_evidence.as_ref().ok_or("Thiếu tham số ảnh")?.0;
    let source=std::fs::read(args["path"].as_str().ok_or("Thiếu đường dẫn ảnh")?)
        .map_err(|e|format!("Không đọc được ảnh: {e}"))?;
    if source!=*bytes {return Err("Ảnh đã bị thay đổi, phải Preview lại".into());}
    let arc=evidence["doorSwingArcs"].as_array().and_then(|items|items.get(index))
        .ok_or("Ứng viên cung đã thay đổi; Preview lại")?;
    if arc["classification"]!="confirmed-drafting-arc-not-door-block" {
        return Err("Chưa đủ chứng cứ để hiệu chỉnh ARC".into());
    }
    let radius=if width_text.trim().is_empty() {arc["radius"].as_f64().ok_or("Thiếu bán kính cung gốc")?}
        else {number(width_text.trim())?};
    let original_radius=arc["radius"].as_f64().ok_or("Thiếu kích thước cửa gốc")?;
    if radius<=1e-6 || radius>1e8 || radius>original_radius*2.0 || radius<original_radius*0.5 {
        return Err("Bề rộng cửa phải dương, trong khoảng 50%–200% ứng viên ảnh; cần xác nhận tỷ lệ".into());
    }
    let offset=if offset_text.trim().is_empty() {0.0} else {number(offset_text.trim())?};
    if offset<0.0 || offset>original_radius*0.4 || (offset>0.0 && radius+offset>original_radius+1e-7) {
        return Err("Dịch bản lề cần trong [0,40% chiều rộng ô mở], và chiều rộng cửa + độ dịch không vượt ô mở nhận dạng".into());
    }
    let source=arc["center"].as_array().filter(|v|v.len()==2).ok_or("Tâm bản lề gốc sai")?;
    let opposite=arc["oppositeHingeCAD"].as_array().filter(|v|v.len()==2).ok_or("Bản lề phía kia sai")?;
    let (Some(sx),Some(sy),Some(ex),Some(ey))=
        (source[0].as_f64(),source[1].as_f64(),opposite[0].as_f64(),opposite[1].as_f64()) else {
        return Err("Không xác định được hai mép ô mở".into());
    };
    let len=(ex-sx).hypot(ey-sy);
    if !len.is_finite() || len<=1e-6 || (len-original_radius).abs()>original_radius*0.15 {
        return Err("Vị trí 2 mép ô mở không phù hợp bề rộng dự kiến".into());
    }
    let fraction=offset/len;
    let (cx,cy)=if flip_hinge {
        (ex+(sx-ex)*fraction,ey+(sy-ey)*fraction)
    } else {
        (sx+(ex-sx)*fraction,sy+(ey-sy)*fraction)
    };
    let (Some(mut leaf_angle),Some(mut opening_angle))=
        (arc["leafAngleDegrees"].as_f64(),arc["openingAngleDegrees"].as_f64()) else {
        return Err("Góc cánh cửa không hợp lệ".into());
    };
    if flip_hinge {opening_angle=(opening_angle+180.0).rem_euclid(360.0);}
    if flip_swing {leaf_angle=(leaf_angle+180.0).rem_euclid(360.0);}
    let delta=(opening_angle-leaf_angle).rem_euclid(360.0);
    let (start,end)=if (delta-90.0).abs()<=12.0 {(leaf_angle,opening_angle)}
        else if (delta-270.0).abs()<=12.0 {(opening_angle,leaf_angle)}
        else {return Err("Lựa chọn bản lề/hướng mở không tạo được cung 90 độ".into())};
    if ![cx,cy,radius,start,end].iter().all(|n|n.is_finite())
        || cx.abs()+radius>1e9 || cy.abs()+radius>1e9 {
        return Err("Kích thước hoặc vị trí cung vượt giới hạn CAD".into());
    }
    let last=preview.steps.last_mut().ok_or("Thiếu đối tượng ARC trong Preview")?;
    if last.0!="arc" {return Err("Phương án không còn cung mở; Preview lại".into());}
    // Update only the draft; the shared image detector evidence remains as originally reviewed.
    last.1=json!({"center":[cx,cy],"radius":radius,"start":start,"end":end});
    preview.description=format!("Ảnh: {} LINE gốc và ARC #{}: rộng {:.2}, dịch bản lề {:.2} về phía trong ô mở, đảo bản lề {}, đảo hướng {}. Không tự cắt tường.",
        preview.steps.len()-1,index,radius,offset,flip_hinge,flip_swing);
    Ok(())
}

/// A managed wall has exactly four native entities produced by this review panel.
/// Arbitrary existing walls are intentionally not eligible for destructive replacement.
fn prepare_managed_wall_shift(session:&Session,managed:&ManagedDoorWall,new_center:&str)->Result<Preview,String> {
    if !drawing_ready(session) {return Err("CAD đang thực hiện lệnh hoặc nhóm Undo".into());}
    let state=session.state().map_err(|e|e.to_string())?;
    if state.uid!=managed.uid || session.doc_epoch!=managed.epoch || state.revision!=managed.revision {
        return Err("Bản vẽ đã đổi từ lần tạo tường/cửa; không được tự thay thế đối tượng".into());
    }
    if managed.handles.len()!=4 || managed.owned_entities.len()!=4 {
        return Err("Không có đủ 4 snapshot hình học tường/cửa để xác minh an toàn".into());
    }
    let mut unique=std::collections::HashSet::new();
    for (raw,snapshot) in managed.handles.iter().zip(&managed.owned_entities) {
        if !unique.insert(raw.as_str()) {return Err("Handle trùng lặp".into());}
        let handle=cadcraft_doc::Handle::parse_hex(raw).ok_or("Handle tường/cửa không hợp lệ")?;
        let entity=session.doc().map_err(|e|e.to_string())?
            .entity(handle).ok_or("Đối tượng tường/cửa đã bị xóa hoặc thay đổi")?;
        if entity.as_ref()!=snapshot {return Err("Hình học hoặc thuộc tính của tường/cửa quản lý đã đổi; Preview lại".into());}
        let layer=session.doc().map_err(|e|e.to_string())?.layer(&entity.common.layer);
        if layer.is_some_and(|l|l.locked) {return Err("Layer của tường/cửa đang khóa".into());}
    }
    let center=number(new_center.trim())?;
    let words:Vec<&str>=managed.command_text.split_whitespace().collect();
    if words.len()!=16 || words.get(0)!=Some(&"vẽ") || words.get(12)!=Some(&"tại") {
        return Err("Chỉ quản lý lệnh tường có cửa theo cú pháp đã xác nhận".into());
    }
    let mut copy=words.iter().map(|s|(*s).to_string()).collect::<Vec<_>>();
    copy[13]=center.to_string();
    let revised=copy.join(" ");
    let Some((steps,description))=parse_door_wall(&revised)? else {
        return Err("Không thể tái tạo hình học tường/cửa".into());
    };
    if steps.len()!=4 {return Err("Chỉ cho phép thay đúng 4 đối tượng tường/cửa".into());}
    Ok(Preview {uid:managed.uid,epoch:managed.epoch,revision:managed.revision,
        text:revised,steps,description:format!("Sẽ thay thế chỉ 4 entity AI Review đã tạo; {description}"),
        image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
        replace_handles:Some(managed.handles.clone()),replace_snapshot:None,
        review_token:Some(format!("managed|{new_center}")),image_controls:None,
        owned_entities:Some(managed.owned_entities.clone())})
}

/// Stage268: derive the centerline of a four-vertex rotated rectangle.
/// Refuses skewed/trapezoid/self-intersecting PLINEs and never guesses from a bounding box.
fn rectangle_wall_axis(points:&[[f64;2]])->Result<([f64;2],[f64;2],f64),String> {
    if points.len()!=4 || points.iter().flatten().any(|v|!v.is_finite()||v.abs()>1e9) {
        return Err("Tường phải có đúng bốn đỉnh hữu hạn".into());
    }
    let edge=|a:[f64;2],b:[f64;2]|[b[0]-a[0],b[1]-a[1]];
    let dot=|a:[f64;2],b:[f64;2]|a[0]*b[0]+a[1]*b[1];
    let cross=|a:[f64;2],b:[f64;2]|a[0]*b[1]-a[1]*b[0];
    let len=|a:[f64;2]|a[0].hypot(a[1]);
    let e0=edge(points[0],points[1]);
    let e1=edge(points[1],points[2]);
    let e2=edge(points[2],points[3]);
    let e3=edge(points[3],points[0]);
    let long0=len(e0);let short0=len(e1);
    if !long0.is_finite()||!short0.is_finite()||long0<=1e-6||short0<=1e-6 {
        return Err("Tường có cạnh suy biến".into());
    }
    let scale=long0.max(short0);
    let tol=(scale*1e-7).max(1e-7);
    if len([e0[0]+e2[0],e0[1]+e2[1]])>tol
        || len([e1[0]+e3[0],e1[1]+e3[1]])>tol
        || dot(e0,e1).abs()>scale*tol
        || cross(e0,e1).abs()<long0*short0*0.99 {
        return Err("PLINE không phải tường hình chữ nhật quay: có thể bị xiên hoặc tự cắt".into());
    }
    let (a,b,thickness,wall_length)=if long0>=short0 {
        ([(points[0][0]+points[3][0])*0.5,(points[0][1]+points[3][1])*0.5],
         [(points[1][0]+points[2][0])*0.5,(points[1][1]+points[2][1])*0.5],
         short0,long0)
    } else {
        ([(points[0][0]+points[1][0])*0.5,(points[0][1]+points[1][1])*0.5],
         [(points[2][0]+points[3][0])*0.5,(points[2][1]+points[3][1])*0.5],
         long0,short0)
    };
    if wall_length<=2.0*thickness || len(edge(a,b))<=1e-6 {
        return Err("Biên dạng quá ngắn hoặc gần hình vuông, chưa thể nhận là tường".into());
    }
    Ok((a,b,thickness))
}

/// Stage270: robust finite-point and segment / convex-quadrilateral interference tests.
/// Equality at a boundary is treated as interference: do not delete or trim a touching neighbour.
fn signed_cross(a:cadcraft_geom::Vec2,b:cadcraft_geom::Vec2,c:cadcraft_geom::Vec2)->f64 {
    (b.x-a.x)*(c.y-a.y)-(b.y-a.y)*(c.x-a.x)
}
fn point_inside_convex_quad(p:cadcraft_geom::Vec2,quad:&[cadcraft_geom::Vec2;4],eps:f64)->bool {
    let signs=(0..4).map(|i|signed_cross(quad[i],quad[(i+1)%4],p));
    let mut positive=false;let mut negative=false;
    for s in signs {
        if s>eps {positive=true;}
        if s< -eps {negative=true;}
    }
    !(positive&&negative)
}
fn on_segment(a:cadcraft_geom::Vec2,b:cadcraft_geom::Vec2,p:cadcraft_geom::Vec2,eps:f64)->bool {
    signed_cross(a,b,p).abs()<=eps
        && p.x>=a.x.min(b.x)-eps && p.x<=a.x.max(b.x)+eps
        && p.y>=a.y.min(b.y)-eps && p.y<=a.y.max(b.y)+eps
}
fn segments_intersect(a:cadcraft_geom::Vec2,b:cadcraft_geom::Vec2,
    c:cadcraft_geom::Vec2,d:cadcraft_geom::Vec2,eps:f64)->bool {
    let o1=signed_cross(a,b,c);let o2=signed_cross(a,b,d);
    let o3=signed_cross(c,d,a);let o4=signed_cross(c,d,b);
    if ((o1>eps&&o2< -eps)||(o1< -eps&&o2>eps))
        && ((o3>eps&&o4< -eps)||(o3< -eps&&o4>eps)) {return true;}
    (o1.abs()<=eps&&on_segment(a,b,c,eps))
        ||(o2.abs()<=eps&&on_segment(a,b,d,eps))
        ||(o3.abs()<=eps&&on_segment(c,d,a,eps))
        ||(o4.abs()<=eps&&on_segment(c,d,b,eps))
}
fn segment_hits_opening(a:cadcraft_geom::Vec2,b:cadcraft_geom::Vec2,
    quad:&[cadcraft_geom::Vec2;4],eps:f64)->bool {
    point_inside_convex_quad(a,quad,eps) || point_inside_convex_quad(b,quad,eps)
        ||(0..4).any(|i|segments_intersect(a,b,quad[i],quad[(i+1)%4],eps))
}
fn polyline_hits_opening(points:&[cadcraft_geom::Vec2],closed:bool,
    quad:&[cadcraft_geom::Vec2;4],eps:f64)->bool {
    if points.iter().any(|p|point_inside_convex_quad(*p,quad,eps)) {return true;}
    if points.windows(2).any(|p|segment_hits_opening(p[0],p[1],quad,eps)) {return true;}
    if closed&&points.len()>2 {
        if segment_hits_opening(points[points.len()-1],points[0],quad,eps) {return true;}
        // A surrounding closed polygon with no edge crossing also obstructs the opening.
        // Ray casting, boundary already handled by segment test.
        let q=quad[0];let mut inside=false;
        for i in 0..points.len() {
            let a=points[i];let b=points[(i+1)%points.len()];
            if (a.y>q.y)!=(b.y>q.y)
                && q.x<(b.x-a.x)*(q.y-a.y)/(b.y-a.y)+a.x {inside=!inside;}
        }
        if inside {return true;}
    }
    false
}
fn precise_neighbour_hits_opening(entity:&cadcraft_doc::Entity,
    quad:&[cadcraft_geom::Vec2;4],eps:f64)->Option<bool> {
    use cadcraft_doc::EntityKind;
    match &entity.kind {
        EntityKind::Line(line)=>{
            let a=cadcraft_geom::Vec2::new(line.a.x,line.a.y);
            let b=cadcraft_geom::Vec2::new(line.b.x,line.b.y);
            if ![a.x,a.y,b.x,b.y].iter().all(|v|v.is_finite()) {return None;}
            Some(segment_hits_opening(a,b,quad,eps))
        }
        EntityKind::LwPolyline(poly) if poly.const_width.abs()<=eps
            && poly.vertices.iter().all(|v|v.bulge.abs()<=eps
                && v.start_width.abs()<=eps&&v.end_width.abs()<=eps)=>{
            let points=poly.vertices.iter().map(|v|cadcraft_geom::Vec2::new(v.p.x,v.p.y))
                .collect::<Vec<_>>();
            if points.len()<2 || points.iter().any(|p|!p.x.is_finite()||!p.y.is_finite()) {return None;}
            Some(polyline_hits_opening(&points,poly.closed,quad,eps))
        }
        // For curves, blocks, hatches, dimensions, wide PLINE, etc., AABB intersection
        // remains a conservative rejection rather than inventing exact geometry.
        _=>None
    }
}

/// Stage269 conservative interference preflight around the rectangular door void.
/// Any entity whose CAD bounds overlap the proposed opening blocks replacement.
/// This can reject safe drawings (false positives), but never silently edits a neighbour.
fn check_door_opening_is_clear(session:&Session,source:cadcraft_doc::Handle,steps:&[(&str,Value)])->Result<(),String> {
    if steps.len()!=4 || steps[0].0!="pline" || steps[1].0!="pline" {
        return Err("Phương án tường/cửa không đúng cấu trúc 4 đối tượng".into());
    }
    let left=steps[0].1["vertices"].as_array().ok_or("Thiếu PLINE mép trái")?;
    let right=steps[1].1["vertices"].as_array().ok_or("Thiếu PLINE mép phải")?;
    if left.len()!=4 || right.len()!=4 {return Err("Biên dạng ô cửa không đủ 4 đỉnh".into());}
    let mut polygon=Vec::new();
    for v in [&left[1],&right[0],&right[3],&left[2]] {
        let p=v.as_array().filter(|p|p.len()==2).ok_or("Sai tọa độ ô mở")?;
        let x=p[0].as_f64().ok_or("Sai X")?;
        let y=p[1].as_f64().ok_or("Sai Y")?;
        if !x.is_finite()||!y.is_finite() {return Err("Tọa độ ô cửa không hữu hạn".into());}
        polygon.push(cadcraft_geom::Vec2::new(x,y));
    }
    let quad:[cadcraft_geom::Vec2;4]=polygon.try_into()
        .map_err(|_|"Khoảng mở không có đúng bốn đỉnh")?;
    let zone=cadcraft_geom::Bounds2::from_points(quad).expand(1e-6);
    if zone.is_empty() {return Err("Khoảng mở không hợp lệ".into());}
    let eps=(zone.width().max(zone.height())*1e-10).max(1e-8);
    let first=signed_cross(quad[0],quad[1],quad[2]);
    if !first.is_finite() || first.abs()<=eps
        || (1..4).any(|i| {
            let turn=signed_cross(quad[i],quad[(i+1)%4],quad[(i+2)%4]);
            !turn.is_finite() || turn*first<=eps*first.abs()
        }) {
        return Err("Khoảng mở không tạo thành tứ giác lồi theo thứ tự hợp lệ".into());
    }
    let doc=session.doc().map_err(|e|e.to_string())?;
    let space=doc.space_of(source).ok_or("Không biết layout của PLINE nguồn")?;
    let store=doc.space(&space).ok_or("Không truy cập được không gian tường")?;
    for entity in store.iter() {
        if entity.handle==source {continue;}
        let other=cadcraft_doc::entity_bounds(doc,entity,0);
        if other.is_empty() {return Err(format!("Entity #{} không có bounds hợp lệ: từ chối cắt an toàn",entity.handle.hex()));}
        if zone.intersects(&other) {
            match precise_neighbour_hits_opening(entity,&quad,eps) {
                Some(false)=>continue, // AABB false positive for validated LINE or thin LWPOLYLINE.
                Some(true)=>return Err(format!("Khoảng mở giao trực tiếp với entity #{}; cần xử lý thủ công",entity.handle.hex())),
                None=>return Err(format!("Entity #{} không hỗ trợ kiểm tra chính xác: từ chối bảo thủ",entity.handle.hex()))
            }
        }
    }
    Ok(())
}

/// Stage268 conservative adapter: one deliberately selected planar four-corner rotated
/// rectangle only. No automatic wall search, no overlapping/intersecting wall mutation.
fn preview_selected_existing_wall(session:&Session,center_text:&str,door_width_text:&str,open_right:bool)->Result<Preview,String> {
    if !drawing_ready(session) {return Err("Kết thúc lệnh CAD trước khi chọn tường".into());}
    let selected=session.selection();
    if selected.len()!=1 {return Err("Chọn chính xác một đối tượng PLINE tường trước khi Preview".into());}
    let handle=selected[0];
    let doc=session.doc().map_err(|e|e.to_string())?;
    let entity=doc.entity(handle).ok_or("Đối tượng đã chọn không còn tồn tại")?;
    if doc.layer(&entity.common.layer).is_some_and(|layer|layer.locked) {
        return Err("Layer đối tượng đang khóa: không được thay thế".into());
    }
    let cadcraft_doc::EntityKind::LwPolyline(poly)=&entity.kind else {
        return Err("Chỉ hỗ trợ LWPOLYLINE tường 4 đỉnh, không nhận LINE/Block/Spline".into());
    };
    if !poly.closed || poly.vertices.len()!=4 || poly.const_width.abs()>1e-8
        || poly.elevation.abs()>1e-8 || poly.vertices.iter().any(|v|
            v.bulge.abs()>1e-8 || v.start_width.abs()>1e-8 || v.end_width.abs()>1e-8) {
        return Err("Tường phải là PLINE kín 4 đỉnh, không cung bulge, không độ rộng nét hoặc cao độ".into());
    }
    let points=poly.vertices.iter().map(|v|[v.p.x,v.p.y]).collect::<Vec<_>>();
    if points.iter().flatten().any(|v|!v.is_finite() || v.abs()>1e9) {
        return Err("Tường có tọa độ ngoài giới hạn".into());
    }
    let (start,end,thick)=rectangle_wall_axis(&points)?;
    let center=number(center_text.trim())?;let door=number(door_width_text.trim())?;
    let side=if open_right {"phải"} else {"trái"};
    let command=format!("vẽ tường có cửa từ {},{} đến {},{} dày {} cửa {} tại {} mở {}",
        start[0],start[1],end[0],end[1],thick,door,center,side);
    let (steps,summary)=parse_door_wall(&command)?
        .ok_or("Không dựng được tường mới từ PLINE được chọn")?;
    if steps.len()!=4 {return Err("Chỉ thay thế PLINE thành đúng bốn entity CAD đã Preview".into());}
    if !entity.common.xdata.is_empty() {
        return Err("PLINE có XData/BIM metadata riêng: chưa hỗ trợ sao chép định danh an toàn, không được cắt".into());
    }
    if entity.common.extrusion!=cadcraft_geom::Vec3::Z || entity.common.thickness.abs()>1e-8 {
        return Err("PLINE có extrusion hoặc thickness ngoài XY: không được thay thế".into());
    }
    if doc.space_of(handle)!=Some(session.space()) {
        return Err("PLINE nằm ở layout khác với CAD Engine đang vẽ".into());
    }
    check_door_opening_is_clear(session,handle,&steps)?;
    let state=session.state().map_err(|e|e.to_string())?;
    Ok(Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
        text:command,steps,description:format!("PLINE đã chọn #{}: {}. Sẽ thay thế đúng đối tượng này sau khi xác nhận. Không tự sửa tường khác.",handle.hex(),summary),
        image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
        replace_handles:Some(vec![handle.hex()]),replace_snapshot:Some((*entity).clone()),
        review_token:Some(format!("existing|{center_text}|{door_width_text}|{open_right}")),image_controls:None,owned_entities:None})
}

/// Stage271: freeze the exact operator controls that produced the approved Preview.
impl AiReview {
    fn image_controls_snapshot(&self)->ImageControlSnapshot {
        ImageControlSnapshot {
            opening_index:self.opening_index.clone(),
            door_arc_index:self.door_arc_index.clone(),
            width:self.door_width_override.clone(),
            offset:self.door_offset_override.clone(),
            flip_hinge:self.flip_door_hinge,
            flip_swing:self.flip_door_swing,
        }
    }
}

fn approved_image_options_unchanged(preview:&Preview,current:&ImageControlSnapshot)->bool {
    preview.image_controls.as_ref().is_none_or(|approved|approved==current)
}

fn review_inputs_match(p:&Preview,prompt:&str,image_path:&str,image_scale:&str,
    existing_center:&str,existing_width:&str,open_right:bool,managed_center:&str)->bool {
    if let Some(token)=&p.review_token {
        let active=if p.replace_snapshot.is_some() {
            format!("existing|{existing_center}|{existing_width}|{open_right}")
        } else {
            format!("managed|{managed_center}")
        };
        active==*token
    } else if p.image_evidence.is_some() {
        format!("{image_path}|{image_scale}")==p.text
    } else {
        prompt==p.text
    }
}

fn returned_handles(result:&Value)->Result<Vec<String>,String> {
    if let Some(handle)=result["handle"].as_str() {return Ok(vec![handle.to_string()]);}
    if let Some(handles)=result["handles"].as_array() {
        return handles.iter().map(|h|h.as_str().map(str::to_string).ok_or("Handle không hợp lệ".into())).collect();
    }
    Err("CAD Engine không trả về handle để quản lý đối tượng".into())
}

/// Stage276: an ERASE result with 0 removed handles is still Ok at command level,
/// but it does not represent a CAD revision step.
/// The engine may return Err after a document mutation (it then bumps Revision).
/// Count that mutation so the same owned Undo group can be closed and rolled back.
fn failed_command_revision_step(before:u64,after:u64)->Result<u64,String> {
    if before==after {return Ok(0);}
    if before.checked_add(1)==Some(after) {return Ok(1);}
    Err("Lệnh lỗi thay đổi Revision ngoài phạm vi một bước; không tự Undo".into())
}

fn erase_revision_step(removed:u64,before:u64,after:u64)->Result<u64,String> {
    if removed==0 {
        if after==before {return Ok(0);}
        return Err("ERASE xóa 0 entity nhưng Revision thay đổi bất thường".into());
    }
    if before.checked_add(1)==Some(after) {return Ok(1);}
    Err("ERASE có xóa đối tượng nhưng Revision không tăng đúng một bước".into())
}

/// Stage278: a revision bump is not sufficient proof that this UNDO group contains
/// a new Snapshot. Check the native undo-stack marker before ending the group.
fn current_undo_group_has_snapshots(session:&Session)->Result<bool,String> {
    let st=session.state().map_err(|e|e.to_string())?;
    let start=st.undo_ctl.group.ok_or("Không có nhóm Undo đang hoạt động")?;
    Ok(st.undo.len()>start)
}

/// One native Undo transaction; replacing old wall entities is allowed only for explicit,
/// still-existing handles owned by this panel and tied to a fixed UID/Epoch/Revision.
fn commit_steps(session:&mut Session,preview:&Preview)->Result<Vec<String>,String> {
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
    if let Some(snapshot)=&preview.replace_snapshot {
        check_door_opening_is_clear(session,snapshot.handle,&preview.steps)?;
    }
    if let Some(old)=&preview.replace_handles {
        if !matches!(old.len(),1|4) || preview.steps.len()!=4
            || (old.len()==1)!=preview.replace_snapshot.is_some()
            || (old.len()==4)!=preview.owned_entities.is_some()
            || preview.owned_entities.as_ref().is_some_and(|snapshots|snapshots.len()!=4) {
            return Err("Chỉ cho phép thay đúng một PLINE do người dùng chọn hoặc trọn bộ 4 entity được quản lý".into());
        }
        let mut unique=std::collections::HashSet::new();
        for (index,raw) in old.iter().enumerate() {
            if !unique.insert(raw.as_str()) {return Err("Danh sách handle thay thế trùng lặp".into());}
            let handle=cadcraft_doc::Handle::parse_hex(raw).ok_or("Handle thay thế sai định dạng")?;
            let doc=session.doc().map_err(|e|e.to_string())?;
            let entity=doc.entity(handle).ok_or("Một đối tượng đã bị xóa: không được thay thế")?;
            if preview.owned_entities.as_ref().is_some_and(|snapshots|snapshots[index]!=*entity.as_ref()) {
                return Err("Bộ tường/cửa đã bị chỉnh sửa sau Preview; không thay thế".into());
            }
            if preview.replace_snapshot.as_ref().is_some_and(|snapshot|snapshot!=entity.as_ref()) {
                return Err("Đối tượng PLINE đã thay đổi từ Preview, phải làm lại".into());
            }
            if doc.layer(&entity.common.layer).is_some_and(|l|l.locked) {
                return Err("Không thay thế đối tượng thuộc layer khóa".into());
            }
        }
    }
    if preview.steps.is_empty() {return Err("Preview không có đối tượng CAD hợp lệ".into());}
    if session.doc_epoch!=preview.epoch || !session.state().is_ok_and(|st|
        st.uid==preview.uid && st.revision==preview.revision) {
        return Err("Bản vẽ đã đổi sau Preview, tuyệt đối không thực thi lệnh".into());
    }
    // Stage274: even a single command goes through UNDO BEGIN/END.
    // A malformed/missing response handle must not leave an untracked CAD mutation.
    let same=|s:&Session,rev:u64,group:bool| {
        s.doc_epoch==preview.epoch && s.state().is_ok_and(|st|
            st.uid==preview.uid && st.revision==rev && st.undo_ctl.group.is_some()==group)
    };
    // The GUI checked state immediately before this function. The transaction is serialized
    // through the UI thread; no arbitrary command is accepted between these operations.
    if !same(session,preview.revision,false) {return Err("Bản vẽ hoặc Revision thay đổi trước giao dịch".into());}
    session.execute("undo",&json!({"mode":"begin"})).map_err(|e|e.to_string())?;
    let mut succeeded=0_u64;
    let mut handles=Vec::<String>::new();
    let mut seen_new_handles=std::collections::HashSet::<String>::new();
    let mut failure:Option<String>=None;
    if let Some(old)=&preview.replace_handles {
        if !same(session,preview.revision,true) {
            failure=Some("Mất giao dịch trước khi xóa đối tượng quản lý".into());
        } else {
            match session.execute("erase",&json!({"handles":old})) {
                Ok(value)=>{
                    // ERASE can return Ok with erased=0 (and NO revision bump).
                    // Count only actual native drawing mutations so we can always close our group.
                    let removed=value["erased"].as_u64().unwrap_or(0);
                    let before=preview.revision.checked_add(succeeded).ok_or("Revision vượt giới hạn")?;
                    let after=session.state().map_err(|e|e.to_string())?.revision;
                    match erase_revision_step(removed,before,after) {
                        Ok(steps)=>succeeded+=steps,
                        Err(error)=>failure=Some(error),
                    }
                    if removed!=old.len() as u64 && failure.is_none() {
                        failure=Some(format!("ERASE chỉ xóa {removed}/{} entity được duyệt; sẽ rollback nếu còn kiểm soát giao dịch",old.len()));
                    }
                }
                Err(error)=>failure=Some(format!("Không xóa được tường/cửa quản lý: {error}"))
            }
        }
    }
    if failure.is_none() {
        for (command,params) in &preview.steps {
            let Some(expected)=preview.revision.checked_add(succeeded) else {
                failure=Some("Revision vượt giới hạn".into());break;
            };
            if !same(session,expected,true) {failure=Some("Bản vẽ bị thay đổi trong lúc AI vẽ".into());break;}
            match session.execute(command,params) {
                Ok(value)=>{
                    succeeded+=1;
                    match returned_handles(&value) {
                        Ok(mut new)=>{
                            if new.len()!=1 || new.iter().any(|raw| {
                                cadcraft_doc::Handle::parse_hex(raw).is_none_or(|h|
                                    session.doc().map_or(true,|doc|doc.entity(h).is_none()))
                            }) {
                                failure=Some(format!("Lệnh {command} không trả đúng một handle CAD tồn tại; yêu cầu rollback"));
                                break;
                            }
                            if !seen_new_handles.insert(new[0].clone())
                                || preview.replace_handles.as_ref().is_some_and(|old|old.contains(&new[0])) {
                                failure=Some("Lệnh AI trả handle bị trùng hoặc tái sử dụng handle bị thay thế; yêu cầu rollback".into());
                                break;
                            }
                            if let Some(snapshot)=&preview.replace_snapshot {
                                for raw in &new {
                                    let Some(handle)=cadcraft_doc::Handle::parse_hex(raw) else {
                                        failure=Some("Không đọc được handle mới".into());break;
                                    };
                                    let common=snapshot.common.clone();
                                    let updated=session.doc_mut().map_err(|e|e.to_string()).and_then(|doc|
                                        doc.modify_entity(handle,|entity|entity.common=common).map_err(|e|e.to_string()));
                                    if let Err(error)=updated {
                                        failure=Some(format!("Không giữ được thuộc tính tường gốc: {error}"));break;
                                    }
                                }
                            }
                            handles.append(&mut new);
                            if failure.is_some() {break;}
                        }
                        Err(error)=>{failure=Some(error);break;}
                    }
                }
                Err(error)=>{
                    // Session::execute may leave a changed document after a command error.
                    // Reconcile observed Revision before deciding whether to rollback.
                    let observed=session.state().map_err(|e|e.to_string())?.revision;
                    match failed_command_revision_step(expected,observed) {
                        Ok(actual)=>succeeded+=actual,
                        Err(e)=>failure=Some(format!("{error}; {e}")),
                    }
                    if failure.is_none() {failure=Some(format!("Lệnh {command} thất bại: {error}"));}
                    break;
                }
            }
        }
    }
    let expected=preview.revision.checked_add(succeeded).ok_or("Revision vượt giới hạn")?;
    if !same(session,expected,true) {
        // A foreign revision is not ours to rollback. Never undo someone else's content.
        return Err("Mất quyền sở hữu giao dịch; không Undo để tránh xóa dữ liệu khác".into());
    }
    let group_has_snapshots=current_undo_group_has_snapshots(session)?;
    session.execute("undo",&json!({"mode":"end"})).map_err(|e|e.to_string())?;
    if let Some(error)=failure {
        // An empty Undo group creates no undo history entry. Even if an earlier
        // operation bumped Revision, never pop a pre-existing user snapshot.
        if group_has_snapshots && same(session,expected,false) {
            session.execute("undo",&json!({"count":1})).map_err(|e|format!("{error}; rollback lỗi: {e}"))?;
        } else if succeeded>0 && !group_has_snapshots {
            return Err(format!("{error}; giao dịch có thay đổi Revision nhưng không tạo Undo Snapshot, giữ nguyên lịch sử người dùng"));
        }
        return Err(error);
    }
    if !group_has_snapshots {
        return Err("CAD báo hoàn tất nhưng không có Undo Snapshot mới: không xác nhận thành công".into());
    }
    Ok(handles)
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
        image_evidence:Some((args,proposal,source_bytes)),selected_opening_guide:None,selected_door_arc:None,replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None})
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
                        text:self.prompt.clone(),steps,description,image_evidence:None,selected_opening_guide:None,selected_door_arc:None,replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None})
                })();
                match next {Ok(p)=>{self.preview=Some(p);self.message="Đã xem trước; chưa ghi đối tượng.".into();},Err(e)=>self.message=e}
            }
            if self.preview.as_ref().is_some_and(|p|p.image_evidence.is_some()) {
                let options=self.preview.as_ref()
                    .and_then(|p|p.image_evidence.as_ref())
                    .and_then(|(_,proposal,_)|proposal["pairedWallOpeningCandidates"].as_array())
                    .map_or(0,Vec::len);
                if options>0 {
                    ui.label(format!("Phát hiện {} ứng viên ô cửa (chưa xác minh). Chọn chỉ số từ 0:",options));
                    ui.text_edit_singleline(&mut self.opening_index);
                    if ui.button("Thêm PLINE chỉ dẫn cho ô cửa đã chọn vào Preview").clicked() {
                        let approved_controls=self.image_controls_snapshot();
                        let selected=self.opening_index.trim().parse::<usize>()
                            .map_err(|_|"Chỉ số ứng viên phải là số nguyên không âm".to_string());
                        match selected {
                            Ok(index)=>{
                                if let Some(p)=&mut self.preview {
                                    match select_opening_guide(p,index) {
                                        Ok(())=>{p.image_controls=Some(approved_controls);self.message="Đã thêm một PLINE chỉ dẫn; xem trước rồi xác nhận. Nét ảnh gốc vẫn giữ nguyên.".into()},
                                        Err(e)=>self.message=e
                                    }
                                }
                            }
                            Err(e)=>self.message=e
                        }
                    }
                }
                let door_options=self.preview.as_ref()
                    .and_then(|p|p.image_evidence.as_ref())
                    .and_then(|(_,proposal,_)|proposal["pairedWallOpeningCandidates"].as_array())
                    .map_or(0,|items|items.iter().filter(|v|v["doorLeafCandidate"]==true).count());
                if door_options>0 {
                    ui.label(format!("Có {} ứng viên cung mở dựa trên nét cánh cửa; chọn chỉ số cung từ 0:",door_options));
                    ui.text_edit_singleline(&mut self.door_arc_index);
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Thêm ARC cung mở của ứng viên đã chọn vào Preview").clicked() {
                        let approved_controls=self.image_controls_snapshot();
                        match self.door_arc_index.trim().parse::<usize>() {
                            Ok(index)=>{
                                if let Some(p)=&mut self.preview {
                                    match select_door_swing_arc(p,index) {
                                        Ok(())=>{p.image_controls=Some(approved_controls);self.message="Đã thêm ARC chỉ dẫn; xem trước và bấm xác nhận riêng. Không tự chỉnh tường gốc.".into()},
                                        Err(e)=>self.message=e
                                    }
                                }
                            }
                            Err(_)=>self.message="Chỉ số ARC phải là số nguyên không âm".into()
                        }
                    }
                }
            }
            if self.preview.as_ref().is_some_and(|p|p.selected_door_arc.is_some()) {
                ui.separator();
                ui.label("Hiệu chỉnh ARC đã chọn trước khi xác nhận:");
                ui.label("Bề rộng cửa CAD (để trống giữ kích thước đề xuất):");
                ui.text_edit_singleline(&mut self.door_width_override);
                ui.label("Dịch bản lề vào trong ô mở (CAD; 0 giữ vị trí gốc):");
                ui.text_edit_singleline(&mut self.door_offset_override);
                ui.checkbox(&mut self.flip_door_hinge,"Đổi vị trí bản lề sang mép kia");
                ui.checkbox(&mut self.flip_door_swing,"Đảo hướng mở");
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Áp dụng hiệu chỉnh vào Preview").clicked() {
                    let approved_controls=self.image_controls_snapshot();
                    if let Some(p)=&mut self.preview {
                        match adjust_reviewed_door_arc(p,&self.door_width_override,&self.door_offset_override,self.flip_door_hinge,self.flip_door_swing) {
                            Ok(())=>{p.image_controls=Some(approved_controls);self.message="Đã cập nhật ARC ở Preview, chưa sửa bản vẽ. Kiểm tra trước khi xác nhận.".into()},
                            Err(e)=>self.message=e
                        }
                    }
                }
            }
            ui.separator();
            ui.label("CẮT Ô CỬA TRÊN TƯỜNG CÓ SẴN: chọn một LWPOLYLINE chữ nhật (cả tường nghiêng)");
            ui.label("Chọn một tường PLINE chữ nhật kín, kể cả nghiêng, rồi nhập khoảng cách tâm cửa dọc trục:");
            ui.text_edit_singleline(&mut self.existing_door_center);
            ui.label("Bề rộng cửa:");
            ui.text_edit_singleline(&mut self.existing_door_width);
            ui.checkbox(&mut self.existing_open_right,"Bản lề phải");
            if ui.button("Preview ô cửa trên PLINE đã chọn").clicked() {
                self.preview=None;
                match preview_selected_existing_wall(session,&self.existing_door_center,
                    &self.existing_door_width,self.existing_open_right) {
                    Ok(p)=>{self.preview=Some(p);self.message="Đã xem trước hình học thay thế, chưa cắt/xóa PLINE gốc".into();},
                    Err(e)=>self.message=format!("Không thể Preview tường được chọn: {e}")
                }
            }
            if self.managed_wall.is_some() {
                ui.separator();
                ui.label("ĐIỀU CHỈNH Ô CỬA ĐÃ TẠO BỞI AI REVIEW (chỉ 4 entity đang quản lý)");
                ui.label("Khoảng cách tâm cửa mới tính từ đầu tường (CAD):");
                ui.text_edit_singleline(&mut self.revised_center);
                if ui.button("Xem trước tường/cửa tại vị trí mới").clicked() {
                    self.preview=None;
                    if let Some(managed)=&self.managed_wall {
                        match prepare_managed_wall_shift(session,managed,&self.revised_center) {
                            Ok(p)=>{self.preview=Some(p);self.message="Đã dựng phương án thay thế, chưa xóa hoặc vẽ gì".into();},
                            Err(e)=>self.message=format!("Không thể điều chỉnh bộ tường/cửa: {e}")
                        }
                    }
                }
            }
            if let Some(p)=self.preview.as_ref() {
                ui.separator();
                ui.label(&p.description);
                if p.image_evidence.is_some() || p.replace_handles.is_some() {
                    // Draw both detected LINE strokes and the explicitly selected PLINE guide.
                    let mut segments:Vec<([f32;2],[f32;2],bool)>=Vec::new();
                    for (command,params) in &p.steps {
                        let to_xy=|value:&Value| -> Option<[f32;2]> {
                            let xy=value.as_array()?;
                            if xy.len()!=2 {return None;}
                            Some([xy[0].as_f64()? as f32,xy[1].as_f64()? as f32])
                        };
                        if *command=="line" {
                            if let Some(points)=params["points"].as_array()
                                && points.len()==2
                                && let (Some(a),Some(b))=(to_xy(&points[0]),to_xy(&points[1])) {
                                segments.push((a,b,false));
                            }
                        } else if *command=="arc" {
                            let center=params["center"].as_array().cloned().unwrap_or_default();
                            if center.len()==2 {
                                if let (Some(cx),Some(cy),Some(radius),Some(start),Some(end))=
                                    (center[0].as_f64(),center[1].as_f64(),
                                        params["radius"].as_f64(),params["start"].as_f64(),params["end"].as_f64()) {
                                    let mut last=None;
                                    for i in 0..=24 {
                                        let angle=(start+(end-start)*f64::from(i)/24.0).to_radians();
                                        let p=[(cx+radius*angle.cos()) as f32,(cy+radius*angle.sin()) as f32];
                                        if let Some(before)=last {segments.push((before,p,true));}
                                        last=Some(p);
                                    }
                                }
                            }
                        } else if *command=="pline" {
                            let vertices=params["vertices"].as_array().cloned().unwrap_or_default();
                            let poly:Vec<[f32;2]>=vertices.iter().filter_map(to_xy).collect();
                            if poly.len()==vertices.len() && poly.len()>=3 {
                                for edge in poly.windows(2) {segments.push((edge[0],edge[1],true));}
                                if params["closed"]==true {segments.push((poly[poly.len()-1],poly[0],true));}
                            }
                        }
                    }
                    if !segments.is_empty() {
                        let xs=segments.iter().flat_map(|(a,b,_)|[a[0],b[0]]);
                        let ys=segments.iter().flat_map(|(a,b,_)|[a[1],b[1]]);
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
                        for (a,b,is_guide) in segments {
                            let color=if is_guide {egui::Color32::YELLOW} else {egui::Color32::LIGHT_BLUE};
                            painter.line_segment([position(a),position(b)],
                                egui::Stroke::new(1.5,color));
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
                    let input_unchanged=review_inputs_match(&p,&self.prompt,&self.image_path,
                        &self.image_scale,&self.existing_door_center,&self.existing_door_width,
                        self.existing_open_right,&self.revised_center);
                    let options_unchanged=approved_image_options_unchanged(&p,&self.image_controls_snapshot());
                    if !permitted||!input_unchanged||!options_unchanged {
                        self.message="Preview cũ hoặc bản vẽ thay đổi; vui lòng Preview lại.".into();
                    } else {
                        match commit_steps(session,&p) {
                            Ok(handles)=>{
                                let is_managed_wall=p.image_evidence.is_none()
                                    && p.steps.len()==4 && p.steps[0].0=="pline"
                                    && p.steps[1].0=="pline" && p.steps[2].0=="line"
                                    && p.steps[3].0=="arc";
                                if is_managed_wall && handles.len()==4 {
                                    if let Ok(state)=session.state() {
                                        self.managed_wall=Some(ManagedDoorWall {
                                            uid:state.uid,epoch:session.doc_epoch,
                                            revision:state.revision,command_text:p.text.clone(),
                                            owned_entities:handles.iter().filter_map(|raw|
                                                cadcraft_doc::Handle::parse_hex(raw)
                                                    .and_then(|id|session.doc().ok()?.entity(id).map(|e|e.as_ref().clone())))
                                                .collect(),
                                            handles
                                        });
                                    } else {self.managed_wall=None;}
                                } else {
                                    self.managed_wall=None;
                                }
                                self.message=format!("Đã thực hiện {} bước CAD. Kiểm tra Undo/Redo.",p.steps.len());
                            }
                            Err(e)=>{self.managed_wall=None;self.message=format!("CAD từ chối giao dịch: {e}")}
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
    #[test]
    fn stage272_freezes_approved_raster_opening_and_door_arc_controls() {
        let mut session=Session::default();
        let state=session.state().unwrap();
        let (steps,description)=plan_from_text("vẽ đường thẳng từ 0,0 đến 100,0").unwrap();
        let mut preview=Preview {
            uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"vẽ đường thẳng từ 0,0 đến 100,0".into(),steps,description,
            image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None,
        };
        let approved=ImageControlSnapshot {
            opening_index:"0".into(),door_arc_index:"1".into(),
            width:"900".into(),offset:"100".into(),
            flip_hinge:false,flip_swing:true,
        };
        assert!(approved_image_options_unchanged(&preview,&approved),
            "plain text-only CAD Preview does not bind the image controls");
        preview.image_controls=Some(approved.clone());
        assert!(approved_image_options_unchanged(&preview,&approved));
        let mut altered=approved.clone();altered.opening_index="2".into();
        assert!(!approved_image_options_unchanged(&preview,&altered));
        altered=approved.clone();altered.door_arc_index="0".into();
        assert!(!approved_image_options_unchanged(&preview,&altered));
        altered=approved.clone();altered.width="800".into();
        assert!(!approved_image_options_unchanged(&preview,&altered));
        altered=approved.clone();altered.offset="150".into();
        assert!(!approved_image_options_unchanged(&preview,&altered));
        altered=approved.clone();altered.flip_hinge=true;
        assert!(!approved_image_options_unchanged(&preview,&altered));
        altered=approved.clone();altered.flip_swing=false;
        assert!(!approved_image_options_unchanged(&preview,&altered));
        assert_eq!(session.doc().unwrap().entity_count(),0);
    }

    #[test]
    fn stage271_rejects_stale_single_native_command_without_mutation() {
        let mut session=Session::default();
        let state=session.state().unwrap();
        let (steps,description)=plan_from_text("vẽ đường thẳng từ 0,0 đến 100,0").unwrap();
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"vẽ đường thẳng từ 0,0 đến 100,0".into(),steps,description,
            image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None};
        assert!(review_inputs_match(&preview,&preview.text,"","","","",false,""));
        assert!(!review_inputs_match(&preview,"vẽ đường thẳng từ 0,0 đến 200,0","","","","",false,""));
        session.execute("line",&json!({"points":[[0,1000],[100,1000]]})).unwrap();
        let original=session.doc().unwrap().entity_count();
        assert!(commit_steps(&mut session,&preview).is_err(),
            "an out-of-date single LINE Preview must not mutate the CAD drawing");
        assert_eq!(session.doc().unwrap().entity_count(),original);
    }

    #[test]
    fn stage271_rejects_changed_controls_and_stale_selected_wall_snapshot() {
        let mut session=Session::default();
        let source=session.execute("pline",&json!({"vertices":[[0,-100],[6000,-100],
            [6000,100],[0,100]],"closed":true})).unwrap();
        let handle=cadcraft_doc::Handle::parse_hex(source["handle"].as_str().unwrap()).unwrap();
        session.set_selection(vec![handle]);
        let preview=preview_selected_existing_wall(&session,"3000","900",false).unwrap();
        assert!(review_inputs_match(&preview,"","","","3000","900",false,""));
        assert!(!review_inputs_match(&preview,"","","","3500","900",false,""),
            "changing center after Preview requires a new user review");
        assert!(!review_inputs_match(&preview,"","","","3000","800",false,""),
            "changing width after Preview requires a new user review");
        assert!(!review_inputs_match(&preview,"","","","3000","900",true,""),
            "changing swing after Preview requires a new user review");
        session.doc_mut().unwrap().modify_entity(handle,|entity|entity.common.ltscale=1.5).unwrap();
        let initial=session.doc().unwrap().entity_count();
        assert!(commit_steps(&mut session,&preview).is_err(),
            "snapshot mismatch must reject replacement even without Revision bump");
        assert_eq!(session.doc().unwrap().entity_count(),initial);
        assert!(session.doc().unwrap().entity(handle).is_some());
    }

    #[test]
    fn stage270_precise_line_intersection_avoids_bounding_box_false_positive() {
        let mut session=Session::default();
        let wall=session.execute("pline",&json!({"vertices":[[0.0,-100.0],[6000.0,-100.0],
            [6000.0,100.0],[0.0,100.0]],"closed":true})).unwrap();
        let wall_id=cadcraft_doc::Handle::parse_hex(wall["handle"].as_str().unwrap()).unwrap();
        let neighbour=session.execute("line",&json!({"points":[[2500.0,90.0],[2600.0,120.0]]})).unwrap();
        let neighbour_id=cadcraft_doc::Handle::parse_hex(neighbour["handles"][0].as_str().unwrap()).unwrap();
        session.set_selection(vec![wall_id]);
        let plan=preview_selected_existing_wall(&session,"3000","900",false)
            .expect("AABB touches opening, but line segment terminates outside the true quadrilateral");
        assert_eq!(session.doc().unwrap().entity_count(),2,"Preview must not mutate CAD");
        let new_handles=commit_steps(&mut session,&plan).unwrap();
        assert_eq!(new_handles.len(),4);
        assert!(session.doc().unwrap().entity(neighbour_id).is_some(),"Unrelated LINE remains untouched");
        session.execute("undo",&json!({"count":1})).unwrap();
        assert!(session.doc().unwrap().entity(wall_id).is_some());
        assert!(session.doc().unwrap().entity(neighbour_id).is_some());
        session.execute("redo",&json!({})).unwrap();
        assert!(session.doc().unwrap().entity(neighbour_id).is_some());
    }

    #[test]
    fn stage270_segment_polygon_cases_cross_touch_and_surround() {
        let quad=[
            cadcraft_geom::Vec2::new(0.0,0.0),
            cadcraft_geom::Vec2::new(10.0,0.0),
            cadcraft_geom::Vec2::new(10.0,2.0),
            cadcraft_geom::Vec2::new(0.0,2.0)
        ];
        let v=|x,y|cadcraft_geom::Vec2::new(x,y);
        assert!(segment_hits_opening(v(-1.0,1.0),v(11.0,1.0),&quad,1e-8));
        assert!(segment_hits_opening(v(-1.0,0.0),v(0.0,0.0),&quad,1e-8));
        assert!(!segment_hits_opening(v(-1.0,2.1),v(11.0,2.1),&quad,1e-8));
        assert!(polyline_hits_opening(&[v(-1.0,-1.0),v(11.0,-1.0),
            v(11.0,3.0),v(-1.0,3.0)],true,&quad,1e-8),
            "surrounding polygon must be considered an obstruction");
        assert!(!polyline_hits_opening(&[v(-2.0,-2.0),v(-1.0,-1.0)],false,&quad,1e-8));
    }

    #[test]
    fn stage269_rejects_neighbour_intersecting_the_door_gap_without_mutation() {
        let mut s=Session::default();
        let wall=s.execute("pline",&json!({"vertices":[[0.0,-100.0],[6000.0,-100.0],
            [6000.0,100.0],[0.0,100.0]],"closed":true})).unwrap();
        let selected=cadcraft_doc::Handle::parse_hex(wall["handle"].as_str().unwrap()).unwrap();
        let other=s.execute("line",&json!({"points":[[3000.0,-600.0],[3000.0,600.0]]})).unwrap();
        let obstruct=cadcraft_doc::Handle::parse_hex(other["handles"][0].as_str().unwrap()).unwrap();
        s.set_selection(vec![selected]);
        let count=s.doc().unwrap().entity_count();
        assert!(preview_selected_existing_wall(&s,"3000","900",false).is_err(),
            "crossing another wall / line must prevent destructive door cut");
        assert_eq!(s.doc().unwrap().entity_count(),count);
        assert!(s.doc().unwrap().entity(selected).is_some());
        assert!(s.doc().unwrap().entity(obstruct).is_some());
    }

    #[test]
    fn stage269_preserves_layer_color_and_refuses_third_party_xdata() {
        let mut s=Session::default();
        let wall=s.execute("pline",&json!({"vertices":[[0.0,-100.0],[6000.0,-100.0],
            [6000.0,100.0],[0.0,100.0]],"closed":true})).unwrap();
        let selected=cadcraft_doc::Handle::parse_hex(wall["handle"].as_str().unwrap()).unwrap();
        s.doc_mut().unwrap().ensure_layer("A-WALL");
        s.doc_mut().unwrap().modify_entity(selected,|entity|{
            entity.common.layer="A-WALL".into();
            entity.common.ltscale=2.5;
        }).unwrap();
        s.set_selection(vec![selected]);
        let plan=preview_selected_existing_wall(&s,"3000","900",false).unwrap();
        let created=commit_steps(&mut s,&plan).unwrap();
        assert_eq!(created.len(),4);
        for raw in &created {
            let id=cadcraft_doc::Handle::parse_hex(raw).unwrap();
            let entity=s.doc().unwrap().entity(id).unwrap();
            assert_eq!(entity.common.layer,"A-WALL");
            assert_eq!(entity.common.ltscale,2.5);
        }
        s.execute("undo",&json!({"count":1})).unwrap();
        assert!(s.doc().unwrap().entity(selected).is_some(),"native Undo restores original wall");
        let second=s.execute("pline",&json!({"vertices":[[0.0,300.0],[6000.0,300.0],
            [6000.0,500.0],[0.0,500.0]],"closed":true})).unwrap();
        let wall2=cadcraft_doc::Handle::parse_hex(second["handle"].as_str().unwrap()).unwrap();
        s.doc_mut().unwrap().modify_entity(wall2,|entity|
            entity.common.xdata.push(cadcraft_doc::RawTag {code:1001,value:"THIRD_PARTY".into()})).unwrap();
        s.set_selection(vec![wall2]);
        let count=s.doc().unwrap().entity_count();
        assert!(preview_selected_existing_wall(&s,"3000","900",false).is_err(),
            "third-party application metadata cannot be duplicated or lost silently");
        assert_eq!(s.doc().unwrap().entity_count(),count);
    }

    #[test]
    fn stage268_rotated_wall_cuts_only_explicitly_selected_rectangle_and_supports_undo_redo() {
        let mut s=Session::default();
        let keep=s.execute("line",&json!({"points":[[0.0,8000.0],[5000.0,8000.0]]})).unwrap();
        let other=cadcraft_doc::Handle::parse_hex(keep["handles"][0].as_str().unwrap()).unwrap();
        // 3-4-5 rotation, wall length 5000 with thickness 200.
        let result=s.execute("pline",&json!({"vertices":[[80.0,-60.0],[3080.0,3940.0],
            [2920.0,4060.0],[-80.0,60.0]],"closed":true})).unwrap();
        let wall=cadcraft_doc::Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        assert!(preview_selected_existing_wall(&s,"2500","900",false).is_err(),"selection is mandatory");
        s.set_selection(vec![wall]);
        let preview=preview_selected_existing_wall(&s,"2500","900",false).unwrap();
        assert_eq!(preview.replace_handles.as_ref().unwrap(),&vec![wall.hex()]);
        assert_eq!(preview.steps.len(),4);
        assert_eq!(preview.steps[0].0,"pline");
        assert_eq!(s.doc().unwrap().entity_count(),2,"Preview is read-only");
        let newest=commit_steps(&mut s,&preview).unwrap();
        assert_eq!(newest.len(),4);
        assert_eq!(s.doc().unwrap().entity_count(),5);
        assert!(s.doc().unwrap().entity(other).is_some());
        s.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(s.doc().unwrap().entity_count(),2);
        assert!(s.doc().unwrap().entity(wall).is_some());
        assert!(s.doc().unwrap().entity(other).is_some());
        s.execute("redo",&json!({})).unwrap();
        assert_eq!(s.doc().unwrap().entity_count(),5);
        assert!(s.doc().unwrap().entity(other).is_some());
    }

    #[test]
    fn stage268_rotated_wall_rejects_skewed_self_intersecting_and_short_candidates() {
        assert!(rectangle_wall_axis(&[[0.0,0.0],[6000.0,0.0],[6200.0,200.0],[0.0,200.0]]).is_err());
        assert!(rectangle_wall_axis(&[[0.0,0.0],[6000.0,0.0],[0.0,200.0],[6000.0,200.0]]).is_err());
        assert!(rectangle_wall_axis(&[[0.0,0.0],[200.0,0.0],[200.0,200.0],[0.0,200.0]]).is_err());
        let (a,b,t)=rectangle_wall_axis(&[[80.0,-60.0],[3080.0,3940.0],[2920.0,4060.0],[-80.0,60.0]]).unwrap();
        assert!((a[0]).abs()<1e-8 && (a[1]).abs()<1e-8);
        assert!((b[0]-3000.0).abs()<1e-8 && (b[1]-4000.0).abs()<1e-8);
        assert!((t-200.0).abs()<1e-8);
    }

    #[test]
    fn stage267_explicitly_selected_existing_wall_is_replaced_without_touching_other_entities() {
        let mut s=Session::default();
        let external=s.execute("line",&json!({"points":[[0.0,9000.0],[6000.0,9000.0]]})).unwrap();
        let keep=cadcraft_doc::Handle::parse_hex(external["handles"][0].as_str().unwrap()).unwrap();
        let result=s.execute("pline",&json!({"vertices":[[0.0,-100.0],[6000.0,-100.0],
            [6000.0,100.0],[0.0,100.0]],"closed":true})).unwrap();
        let wall=cadcraft_doc::Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        assert_eq!(s.doc().unwrap().entity_count(),2);
        assert!(preview_selected_existing_wall(&s,"3000","900",false).is_err(),"selection must be explicit");
        s.set_selection(vec![wall]);
        let preview=preview_selected_existing_wall(&s,"3000","900",false).unwrap();
        assert_eq!(preview.replace_handles.as_ref().unwrap(),&vec![wall.hex()]);
        assert_eq!(preview.steps.len(),4);
        assert_eq!(s.doc().unwrap().entity_count(),2,"Preview cannot alter original wall");
        let created=commit_steps(&mut s,&preview).unwrap();
        assert_eq!(created.len(),4);
        assert_eq!(s.doc().unwrap().entity_count(),5);
        assert!(s.doc().unwrap().entity(wall).is_none());
        assert!(s.doc().unwrap().entity(keep).is_some());
        s.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(s.doc().unwrap().entity_count(),2);
        assert!(s.doc().unwrap().entity(wall).is_some());
        assert!(s.doc().unwrap().entity(keep).is_some());
        s.execute("redo",&json!({})).unwrap();
        assert_eq!(s.doc().unwrap().entity_count(),5);
        assert!(s.doc().unwrap().entity(wall).is_none());
        assert!(s.doc().unwrap().entity(keep).is_some());
    }

    #[test]
    fn stage267_refuses_non_rectangular_or_stale_existing_wall_selection() {
        let mut s=Session::default();
        let non_rectangle=s.execute("pline",&json!({"vertices":[[0,0],[6000,0],[6000,200],[500,200]],"closed":true})).unwrap();
        let non_rect_handle=cadcraft_doc::Handle::parse_hex(non_rectangle["handle"].as_str().unwrap()).unwrap();
        s.set_selection(vec![non_rect_handle]);
        assert!(preview_selected_existing_wall(&s,"3000","900",false).is_err());
        let result=s.execute("pline",&json!({"vertices":[[0.0,-100.0],[6000.0,-100.0],
            [6000.0,100.0],[0.0,100.0]],"closed":true})).unwrap();
        let wall=cadcraft_doc::Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        s.set_selection(vec![wall]);
        assert!(preview_selected_existing_wall(&s,"50","900",false).is_err(),"opening cannot meet wall end");
        let stale=preview_selected_existing_wall(&s,"3000","900",false).unwrap();
        s.execute("line",&json!({"points":[[0,9000],[6000,9000]]})).unwrap();
        let count=s.doc().unwrap().entity_count();
        assert!(commit_steps(&mut s,&stale).is_err(),"revision mismatch must reject destructive replacement");
        assert_eq!(s.doc().unwrap().entity_count(),count);
        assert!(s.doc().unwrap().entity(wall).is_some());
    }

    #[test]
    fn stage277_reconcile_failed_engine_command_revision_without_touching_prior_history() {
        assert_eq!(failed_command_revision_step(25,25).unwrap(),0);
        assert_eq!(failed_command_revision_step(25,26).unwrap(),1);
        assert!(failed_command_revision_step(25,27).is_err());
        assert!(failed_command_revision_step(25,24).is_err());
        assert!(failed_command_revision_step(u64::MAX,0).is_err());
    }

    #[test]
    fn stage281_ai_review_large_native_group_at_full_undo_limit_preserves_handles_and_source() {
        let mut session=Session::default();
        let old=session.execute("line",&json!({"points":[[10000.0,10000.0],[11000.0,10000.0]]})).unwrap();
        let old_handle=cadcraft_doc::Handle::parse_hex(old["handles"][0].as_str().unwrap()).unwrap();
        // The old history is deliberately full; the 128 inserted entities are actual
        // CAD Engine LINE operations via AI Review::commit_steps.
        while session.state().unwrap().undo.len()<cadcraft_engine::undoctl::MAX_UNDO {
            let prior={
                let s=session.state().unwrap();
                cadcraft_engine::Snapshot {label:"prior-history".into(),doc:s.doc.clone(),selection:Vec::new()}
            };
            session.state_mut().unwrap().push_undo(prior);
        }
        let previous_count=session.doc().unwrap().entity_count();
        let state=session.state().unwrap();
        let mut steps=Vec::new();
        for n in 0..128 {
            let y=5000.0+f64::from(n)*20.0;
            steps.push(("line",json!({"points":[[0.0,y],[700.0,y]]})));
        }
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"stage281 batch 128 native LINE".into(),
            steps,description:"Full history import regression".into(),image_evidence:None,
            selected_opening_guide:None,selected_door_arc:None,replace_handles:None,
            replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None};
        let created=commit_steps(&mut session,&preview).unwrap();
        assert_eq!(created.len(),128);
        assert_eq!(created.iter().collect::<std::collections::HashSet<_>>().len(),128);
        assert_eq!(session.state().unwrap().undo.len(),cadcraft_engine::undoctl::MAX_UNDO);
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        assert_eq!(session.doc().unwrap().entity_count(),previous_count+128);
        assert!(session.doc().unwrap().entity(old_handle).is_some());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),previous_count);
        assert!(session.doc().unwrap().entity(old_handle).is_some());
        for raw in &created {
            assert!(session.doc().unwrap().entity(cadcraft_doc::Handle::parse_hex(raw).unwrap()).is_none());
        }
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),previous_count+128);
        for raw in &created {
            assert!(session.doc().unwrap().entity(cadcraft_doc::Handle::parse_hex(raw).unwrap()).is_some());
        }
    }

    #[test]
    fn stage278_undo_snapshot_marker_detects_empty_and_nonempty_groups() {
        let mut session=Session::default();
        session.execute("line",&json!({"points":[[10,500],[50,500]]})).unwrap();
        assert!(current_undo_group_has_snapshots(&session).is_err());
        session.execute("undo",&json!({"mode":"begin"})).unwrap();
        assert!(!current_undo_group_has_snapshots(&session).unwrap(),
            "begin alone cannot create a new undo snapshot");
        session.execute("undo",&json!({"mode":"end"})).unwrap();
        session.execute("undo",&json!({"mode":"begin"})).unwrap();
        session.execute("line",&json!({"points":[[10,600],[50,600]]})).unwrap();
        assert!(current_undo_group_has_snapshots(&session).unwrap(),
            "one native drawing command must create a snapshot within the group");
        session.execute("undo",&json!({"mode":"end"})).unwrap();
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),1);
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
    }

    #[test]
    fn stage278_empty_invalid_ai_transaction_keeps_prior_undo_snapshot() {
        let mut session=Session::default();
        let existing=session.execute("line",&json!({"points":[[0,600],[100,600]]})).unwrap();
        let handle=cadcraft_doc::Handle::parse_hex(existing["handles"][0].as_str().unwrap()).unwrap();
        let state=session.state().unwrap();
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"test-empty-transaction".into(),
            steps:vec![("line",json!({"points":[[2,2],[2,2]]}))],
            description:"invalid LINE".into(),image_evidence:None,
            selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,
            owned_entities:None};
        let before=session.state().unwrap().undo.len();
        assert!(commit_steps(&mut session,&preview).is_err());
        assert_eq!(session.state().unwrap().undo.len(),before);
        assert!(session.doc().unwrap().entity(handle).is_some());
        assert!(session.state().unwrap().undo_ctl.group.is_none());
    }

    #[test]
    fn stage277_empty_failed_ai_group_must_never_undo_previous_user_command() {
        let mut session=Session::default();
        let before=session.execute("line",&json!({"points":[[10,200],[90,200]]})).unwrap();
        let earlier=cadcraft_doc::Handle::parse_hex(before["handles"][0].as_str().unwrap()).unwrap();
        let state=session.state().unwrap();
        let draft=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"intentional-invalid-first-command".into(),
            steps:vec![("line",json!({"points":[[5,5],[5,5]]}))],
            description:"Check empty failed Undo group".into(),image_evidence:None,
            selected_opening_guide:None,selected_door_arc:None,replace_handles:None,
            replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None};
        let rev=state.revision;
        assert!(commit_steps(&mut session,&draft).is_err());
        assert!(session.doc().unwrap().entity(earlier).is_some(),
            "AI failed before any mutation: unrelated previous user LINE must remain");
        assert_eq!(session.doc().unwrap().entity_count(),1);
        assert_eq!(session.state().unwrap().revision,rev,
            "empty failed group must not perform an Undo of earlier history");
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        // User's own Undo history should remain functional.
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
    }

    #[test]
    fn stage277_partial_failure_rolls_back_only_its_group_but_preserves_previous_undo() {
        let mut session=Session::default();
        let previous=session.execute("line",&json!({"points":[[10,200],[90,200]]})).unwrap();
        let earlier=cadcraft_doc::Handle::parse_hex(previous["handles"][0].as_str().unwrap()).unwrap();
        let state=session.state().unwrap();
        let draft=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"intentionally-invalid-second-command".into(),
            steps:vec![("line",json!({"points":[[0,0],[100,0]]})),
                ("line",json!({"points":[[50,50],[50,50]]}))],
            description:"Check grouped partial rollback".into(),image_evidence:None,
            selected_opening_guide:None,selected_door_arc:None,replace_handles:None,
            replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None};
        assert!(commit_steps(&mut session,&draft).is_err());
        assert_eq!(session.doc().unwrap().entity_count(),1,
            "failed AI work must rollback, existing LINE survives");
        assert!(session.doc().unwrap().entity(earlier).is_some());
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0,
            "user should still be able to undo earlier work separately");
    }

    #[test]
    fn stage276_erase_revision_counter_uses_actual_native_mutation() {
        assert_eq!(erase_revision_step(0,10,10).unwrap(),0,
            "an empty successful ERASE must not increment the expected revision");
        assert_eq!(erase_revision_step(1,10,11).unwrap(),1);
        assert_eq!(erase_revision_step(4,10,11).unwrap(),1,
            "deleting 4 entities in one command is one document revision");
        assert!(erase_revision_step(0,10,11).is_err());
        assert!(erase_revision_step(1,10,10).is_err());
        assert!(erase_revision_step(1,u64::MAX,u64::MAX).is_err());
    }

    #[test]
    fn stage276_native_zero_erase_does_not_mutate_and_group_remains_closable() {
        let mut session=Session::default();
        let existing=session.execute("line",&json!({"points":[[0,3000],[100,3000]]})).unwrap();
        let external=cadcraft_doc::Handle::parse_hex(existing["handles"][0].as_str().unwrap()).unwrap();
        let rev=session.state().unwrap().revision;
        session.execute("undo",&json!({"mode":"begin"})).unwrap();
        let result=session.execute("erase",&json!({"handles":["FFFFFFFFFFFFFFFE"]})).unwrap();
        let after=session.state().unwrap().revision;
        assert_eq!(result["erased"],0);
        assert_eq!(erase_revision_step(0,rev,after).unwrap(),0);
        session.execute("undo",&json!({"mode":"end"})).unwrap();
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        assert!(session.doc().unwrap().entity(external).is_some(),
            "zero-erase must not touch the unrelated line");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage275_end_to_end_png_preview_commit_undo_redo_keeps_other_entities() {
        use image::{GrayImage,Luma};
        let mut image=GrayImage::from_pixel(112,64,Luma([255]));
        for x in 10..=96 {image.put_pixel(x,22,Luma([0]));}
        for y in 10..=52 {image.put_pixel(20,y,Luma([0]));}
        let path=std::env::temp_dir().join(format!("cad360-stage275-{}-{}.png",
            std::process::id(),std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        image.save(&path).unwrap();
        let mut session=Session::default();
        let external=session.execute("line",&json!({"points":[[10000,10000],[12000,10000]]})).unwrap();
        let external_id=cadcraft_doc::Handle::parse_hex(external["handles"][0].as_str().unwrap()).unwrap();
        let draft=preview_raster(&session,&path.to_string_lossy(),"2").unwrap();
        assert!(draft.steps.len()>=2,"synthetic PNG contains two real orthogonal pixel strokes");
        assert!(draft.steps.iter().all(|(command,_)|*command=="line"));
        assert_eq!(session.doc().unwrap().entity_count(),1,"Preview must be read-only");
        let created=commit_steps(&mut session,&draft).unwrap();
        assert_eq!(created.len(),draft.steps.len());
        assert_eq!(session.doc().unwrap().entity_count(),created.len()+1);
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),1,"one Undo removes entire PNG import");
        assert!(session.doc().unwrap().entity(external_id).is_some());
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),created.len()+1);
        for raw in &created {
            assert!(session.doc().unwrap().entity(cadcraft_doc::Handle::parse_hex(raw).unwrap()).is_some());
        }
        assert!(session.doc().unwrap().entity(external_id).is_some());
        let _=std::fs::remove_file(path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage275_changed_png_source_must_be_rejected_before_any_cad_mutation() {
        use image::{GrayImage,Luma};
        let mut image=GrayImage::from_pixel(112,64,Luma([255]));
        for x in 10..=96 {image.put_pixel(x,22,Luma([0]));}
        let path=std::env::temp_dir().join(format!("cad360-stage275-change-{}-{}.png",
            std::process::id(),std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        image.save(&path).unwrap();
        let mut session=Session::default();
        let draft=preview_raster(&session,&path.to_string_lossy(),"1").unwrap();
        image.put_pixel(40,22,Luma([255]));
        image.save(&path).unwrap();
        assert!(commit_steps(&mut session,&draft).is_err(),
            "changing image bytes after Preview must refuse the reviewed plan");
        assert_eq!(session.doc().unwrap().entity_count(),0);
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn stage274_single_native_line_uses_group_and_one_undo_redo() {
        let mut session=Session::default();
        let state=session.state().unwrap();
        let (steps,description)=plan_from_text("vẽ đường thẳng từ 0,0 đến 100,0").unwrap();
        let approved=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"vẽ đường thẳng từ 0,0 đến 100,0".into(),steps,description,
            image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,
            owned_entities:None};
        let handles=commit_steps(&mut session,&approved).expect("single native LINE grouped commit");
        assert_eq!(handles.len(),1);
        assert_eq!(session.doc().unwrap().entity_count(),1);
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),1);
    }

    #[test]
    fn stage274_single_command_failure_never_opens_unclosed_undo_group() {
        let mut session=Session::default();
        let state=session.state().unwrap();
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"test-invalid-single".into(),
            steps:vec![("line",json!({"points":[[1,1],[1,1]]}))],
            description:"Must fail safely".into(),
            image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,
            owned_entities:None};
        assert!(commit_steps(&mut session,&preview).is_err());
        assert_eq!(session.doc().unwrap().entity_count(),0);
        assert!(session.state().unwrap().undo_ctl.group.is_none());
    }

    #[test]
    fn stage273_managed_wall_rejects_undetected_property_changes_without_erasing() {
        let mut session=Session::default();
        let text="vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở trái";
        let (steps,description)=plan_from_text(text).unwrap();
        let state=session.state().unwrap();
        let draft=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:text.into(),steps,description,image_evidence:None,selected_opening_guide:None,
            selected_door_arc:None,replace_handles:None,replace_snapshot:None,review_token:None,
            image_controls:None,owned_entities:None};
        let originals=commit_steps(&mut session,&draft).unwrap();
        let owned_entities=originals.iter().map(|raw|{
            let handle=cadcraft_doc::Handle::parse_hex(raw).unwrap();
            session.doc().unwrap().entity(handle).unwrap().as_ref().clone()
        }).collect();
        let state=session.state().unwrap();
        let managed=ManagedDoorWall {uid:state.uid,epoch:session.doc_epoch,
            revision:state.revision,command_text:text.into(),handles:originals.clone(),owned_entities};
        let approved=prepare_managed_wall_shift(&session,&managed,"3500").unwrap();
        let changed_handle=cadcraft_doc::Handle::parse_hex(&originals[0]).unwrap();
        session.doc_mut().unwrap().modify_entity(changed_handle,|entity|
            entity.common.ltscale=1.75).unwrap();
        let count=session.doc().unwrap().entity_count();
        assert!(prepare_managed_wall_shift(&session,&managed,"3500").is_err(),
            "new Preview cannot trust managed wall modified without Revision bump");
        assert!(commit_steps(&mut session,&approved).is_err(),
            "existing Preview must reject modified managed wall snapshots");
        assert_eq!(session.doc().unwrap().entity_count(),count);
        for raw in &originals {
            let handle=cadcraft_doc::Handle::parse_hex(raw).unwrap();
            assert!(session.doc().unwrap().entity(handle).is_some());
        }
    }

    #[test]
    fn stage273_failed_native_group_rolls_back_and_preserves_unrelated_geometry() {
        let mut session=Session::default();
        let external=session.execute("line",&json!({"points":[[0,9000],[5000,9000]]})).unwrap();
        let external_handle=cadcraft_doc::Handle::parse_hex(external["handles"][0].as_str().unwrap()).unwrap();
        let state=session.state().unwrap();
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:"test-rollback".into(),
            steps:vec![
                ("line",json!({"points":[[0,0],[100,0]]})),
                ("line",json!({"points":[[10,10],[10,10]]}))
            ],
            description:"Injected invalid second line".into(),
            image_evidence:None,selected_opening_guide:None,selected_door_arc:None,
            replace_handles:None,replace_snapshot:None,review_token:None,
            image_controls:None,owned_entities:None};
        assert!(commit_steps(&mut session,&preview).is_err());
        assert_eq!(session.doc().unwrap().entity_count(),1,
            "failed grouped CAD operation must not leave an inserted line");
        assert!(session.doc().unwrap().entity(external_handle).is_some());
        assert!(session.state().unwrap().undo_ctl.group.is_none());
    }

    #[test]
    fn stage266_replace_only_managed_wall_door_with_one_undo_and_keep_unrelated_geometry() {
        let mut session=Session::default();
        // Existing, unrelated CAD content must survive an AI Review-managed wall relocation.
        let external=session.execute("line",&json!({"points":[[0.0,9000.0],[6000.0,9000.0]]})).unwrap();
        let external_handle=cadcraft_doc::Handle::parse_hex(external["handles"][0].as_str().unwrap()).unwrap();
        let text="vẽ tường có cửa từ 0,0 đến 6000,0 dày 200 cửa 900 tại 3000 mở trái";
        let (steps,description)=plan_from_text(text).unwrap();
        let state=session.state().unwrap();
        let preview=Preview {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            text:text.into(),steps,description,image_evidence:None,selected_opening_guide:None,
            selected_door_arc:None,replace_handles:None,replace_snapshot:None,review_token:None,image_controls:None,owned_entities:None};
        let originals=commit_steps(&mut session,&preview).unwrap();
        assert_eq!(originals.len(),4);
        assert_eq!(session.doc().unwrap().entity_count(),5);
        let state=session.state().unwrap();
        let owned_entities=originals.iter().map(|raw| {
            let handle=cadcraft_doc::Handle::parse_hex(raw).unwrap();
            session.doc().unwrap().entity(handle).unwrap().as_ref().clone()
        }).collect();
        let managed=ManagedDoorWall {uid:state.uid,epoch:session.doc_epoch,revision:state.revision,
            command_text:text.into(),handles:originals.clone(),owned_entities};
        assert!(prepare_managed_wall_shift(&session,&managed,"200").is_err(),
            "door cannot touch outer wall endpoint");
        assert!(prepare_managed_wall_shift(&session,&managed,"-100").is_err());
        let revised=prepare_managed_wall_shift(&session,&managed,"3500").unwrap();
        assert_eq!(revised.replace_handles.as_ref().unwrap(),&originals);
        assert!(revised.text.contains("tại 3500 mở trái"),"only the center position may change");
        assert_eq!(revised.steps.len(),4);
        assert_eq!(session.doc().unwrap().entity_count(),5,"Preview must be read-only");
        let newest=commit_steps(&mut session,&revised).unwrap();
        assert_eq!(newest.len(),4);
        assert_eq!(session.doc().unwrap().entity_count(),5,"only four reviewed entities get replaced");
        assert_ne!(originals,newest);
        assert!(session.doc().unwrap().entity(external_handle).is_some(),"Unrelated LINE must not be erased");
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),5);
        for raw in &originals {
            assert!(session.doc().unwrap().entity(cadcraft_doc::Handle::parse_hex(raw).unwrap()).is_some());
        }
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),5);
        for raw in &newest {
            assert!(session.doc().unwrap().entity(cadcraft_doc::Handle::parse_hex(raw).unwrap()).is_some());
        }
        assert!(session.doc().unwrap().entity(external_handle).is_some(),"Unrelated LINE survives Undo and Redo");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage265_shift_door_hinge_inside_detected_opening_without_modifying_source_lines() {
        use image::{GrayImage,Luma};
        let mut img=GrayImage::from_pixel(110,48,Luma([255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,Luma([0]));}
            for x in 51..=95 {img.put_pixel(x,y,Luma([0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,Luma([0]));}
        let path=std::env::temp_dir().join(format!("stage265-door-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut session=Session::default();
        let mut preview=preview_raster(&session,&path.to_string_lossy(),"2").unwrap();
        let source_lines=preview.steps.clone();
        select_door_swing_arc(&mut preview,0).unwrap();
        let original_arc=preview.steps.last().unwrap().1.clone();
        let source_radius=original_arc["radius"].as_f64().unwrap();
        let shift=source_radius*0.2;
        let target_radius=source_radius*0.7;
        assert!(adjust_reviewed_door_arc(&mut preview,&source_radius.to_string(),&shift.to_string(),false,false).is_err(),
            "a full width leaf must not be shifted outside the detected gap");
        assert!(adjust_reviewed_door_arc(&mut preview,&target_radius.to_string(),"-1",false,false).is_err());
        assert!(adjust_reviewed_door_arc(&mut preview,&target_radius.to_string(),&(source_radius*0.45).to_string(),false,false).is_err());
        adjust_reviewed_door_arc(&mut preview,&target_radius.to_string(),&shift.to_string(),false,false).unwrap();
        assert_eq!(&preview.steps[..source_lines.len()],&source_lines[..]);
        assert_eq!(preview.steps.last().unwrap().1["radius"],json!(target_radius));
        assert_ne!(preview.steps.last().unwrap().1["center"],original_arc["center"]);
        assert_eq!(session.doc().unwrap().entity_count(),0);
        commit_steps(&mut session,&preview).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),preview.steps.len());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),preview.steps.len());
        let _=std::fs::remove_file(path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage264_reviewed_door_arc_width_hinge_and_swing_preserves_source_and_undo() {
        use image::{GrayImage,Luma};
        let mut img=GrayImage::from_pixel(110,48,Luma([255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,Luma([0]));}
            for x in 51..=95 {img.put_pixel(x,y,Luma([0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,Luma([0]));}
        let path=std::env::temp_dir().join(format!("cad360-stage264-{}-{}.png",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut session=Session::default();
        let mut preview=preview_raster(&session,&path.to_string_lossy(),"2").unwrap();
        let original_lines=preview.steps.clone();
        assert!(adjust_reviewed_door_arc(&mut preview,"","",false,false).is_err());
        select_door_swing_arc(&mut preview,0).unwrap();
        let original=preview.steps.last().unwrap().1.clone();
        let width=original["radius"].as_f64().unwrap()*1.1;
        assert!(adjust_reviewed_door_arc(&mut preview,"0","",false,false).is_err());
        assert!(adjust_reviewed_door_arc(&mut preview,"100000000","",false,false).is_err());
        adjust_reviewed_door_arc(&mut preview,&width.to_string(),"",true,true).unwrap();
        assert_eq!(preview.steps.len(),original_lines.len()+1);
        assert_eq!(&preview.steps[..original_lines.len()],&original_lines[..]);
        assert_eq!(preview.steps.last().unwrap().1["radius"],json!(width));
        assert_ne!(preview.steps.last().unwrap().1["center"],original["center"]);
        assert_eq!(session.doc().unwrap().entity_count(),0);
        commit_steps(&mut session,&preview).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),preview.steps.len());
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),preview.steps.len());
        let _=std::fs::remove_file(path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage263_door_arc_requires_explicit_operator_selection_and_preserves_image_lines() {
        use image::{GrayImage,Luma};
        let mut img=GrayImage::from_pixel(110,48,Luma([255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,Luma([0]));}
            for x in 51..=95 {img.put_pixel(x,y,Luma([0]));}
        }
        for y in 3..=15 {img.put_pixel(39,y,Luma([0]));}
        let path=std::env::temp_dir().join(format!("cad360-stage263-door-{}-{}.png",
            std::process::id(),std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut session=Session::default();
        let mut p=preview_raster(&session,&path.to_string_lossy(),"2").unwrap();
        let original=p.steps.len();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        assert!(select_door_swing_arc(&mut p,999).is_err());
        assert_eq!(p.steps.len(),original);
        select_door_swing_arc(&mut p,0).unwrap();
        assert_eq!(p.steps.len(),original+1);
        assert_eq!(p.steps.last().unwrap().0,"arc");
        assert!(select_opening_guide(&mut p,0).is_err(),
            "a separate proposal is needed to switch annotation types");
        commit_steps(&mut session,&p).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),original+1);
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        let _=std::fs::remove_file(path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stage263_paired_wall_opening_is_explicit_reviewed_native_guide() {
        use image::{GrayImage,Luma};
        let mut img=GrayImage::from_pixel(110,48,Luma([255]));
        for y in [15,22] {
            for x in 5..=39 {img.put_pixel(x,y,Luma([0]));}
            for x in 51..=95 {img.put_pixel(x,y,Luma([0]));}
        }
        let path=std::env::temp_dir().join(format!("cad360-stage263-{}-{}.png",
            std::process::id(),std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        img.save(&path).unwrap();
        let mut session=Session::default();
        let mut p=preview_raster(&session,&path.to_string_lossy(),"2").unwrap();
        let original=p.steps.len();
        assert_eq!(p.image_evidence.as_ref().unwrap().1["pairedWallOpeningCandidateCount"],1);
        assert_eq!(session.doc().unwrap().entity_count(),0);
        assert!(select_opening_guide(&mut p,1).is_err(),"invalid candidate index must fail");
        assert_eq!(p.steps.len(),original,"no approval must keep source-only proposal");
        select_opening_guide(&mut p,0).unwrap();
        assert_eq!(p.steps.len(),original+1);
        assert_eq!(p.steps.last().unwrap().0,"pline");
        assert_eq!(p.steps.last().unwrap().1["closed"],true);
        select_opening_guide(&mut p,0).unwrap();
        assert_eq!(p.steps.len(),original+1,"duplicate click must not duplicate the guide");
        commit_steps(&mut session,&p).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),original+1);
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),0);
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),original+1);
        let _=std::fs::remove_file(path);
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
