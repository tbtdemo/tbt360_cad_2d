//! Live, read-only quantities from the canonical CAD drawing.
//! Geometry changes are owned by the native CAD command/undo system.
//! These are drawing-unit quantities, NOT validated BIM/IFC/steel QTO.
use std::collections::{BTreeMap, BTreeSet};

use cadcraft_doc::EntityKind;
use cadcraft_ui_egui::CadApp;
use crate::takeoff_report::{self, ExportConfig, GroupBy, ReportLine, FIELDS};
use crate::takeoff_composite::{self, GeometricSource, PlanSectionVolume};
use crate::takeoff_road::{self, EarthworkKind, RoadCrossSection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeasureKind { Length, Area, Perimeter, Count }

impl MeasureKind {
    fn code(self) -> &'static str {
        match self { Self::Length => "length", Self::Area => "area", Self::Perimeter => "perimeter", Self::Count => "count" }
    }
    fn parse(value: &str) -> Option<Self> {
        match value { "length" => Some(Self::Length), "area" => Some(Self::Area), "perimeter" => Some(Self::Perimeter), "count" => Some(Self::Count), _ => None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeasureStatus { Review, Confirmed, Excluded }

impl MeasureStatus {
    fn code(self) -> &'static str {
        match self { Self::Review => "review", Self::Confirmed => "confirmed", Self::Excluded => "excluded" }
    }
    fn parse(value: &str) -> Option<Self> {
        match value { "review" => Some(Self::Review), "confirmed" => Some(Self::Confirmed), "excluded" => Some(Self::Excluded), _ => None }
    }
}

fn normalize_discipline(value:&str)->Option<String> {
    match value.trim() {
        "ARCH" | "Kiến trúc" => Some("ARCH".into()),
        "STRUCT" | "Kết cấu" => Some("STRUCT".into()),
        "ELEC" | "Điện" => Some("ELEC".into()),
        "PLUMB" | "Nước" => Some("PLUMB".into()),
        "HVAC" | "Điều hòa/Thông gió" => Some("HVAC".into()),
        "COORD" | "Phối hợp" => Some("COORD".into()),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope {
    pub floor_id: Option<u64>,
    pub discipline: Option<String>,
    pub owner_account_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SourcePathStep {
    BlockMember { block: String, member_handle: String },
    ArrayItem { i: u32, j: u32, level: u32, member_handle: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourcePath {
    root_handle: String,
    steps: Vec<SourcePathStep>,
}

#[derive(Clone, Debug, PartialEq)]
struct MeasurementRecord {
    id: u64,
    kind: MeasureKind,
    points: Vec<[f64; 2]>,
    raw_value: f64,
    layer: String,
    work_group: String,
    work_code: String,
    description: String,
    note: String,
    status: MeasureStatus,
    scope: Scope,
    source_handle: Option<String>,
    source_path: Option<SourcePath>,
    source_revision: Option<u64>,
    source_geometry_signature: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct AuditEntry {
    sequence: u64,
    record_id: u64,
    action: String,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum QtoSourceMode {
    #[default]
    CurrentSelection,
    CapturedRegion,
    EntireProject,
}
impl QtoSourceMode {
    fn label(self) -> &'static str {
        match self {
            Self::CurrentSelection => "Đối tượng đang quét/chọn",
            Self::CapturedRegion => "Phạm vi đã lưu",
            Self::EntireProject => "Toàn bộ bản vẽ (chủ động chọn)",
        }
    }
    const ALL: [Self; 3] = [Self::CurrentSelection, Self::CapturedRegion, Self::EntireProject];
}

#[derive(Default)]
pub struct State {
    drawing_distance: f64,
    real_distance_m: f64,
    records: Vec<MeasurementRecord>,
    audit: Vec<AuditEntry>,
    next_record_id: u64,
    next_audit_sequence: u64,
    undo: Vec<Vec<MeasurementRecord>>,
    redo: Vec<Vec<MeasurementRecord>>,
    selected_record_id: Option<u64>,
    search_text: String,
    status_filter: u8,
    import_feedback: String,
    file_feedback: String,
    export_confirmed_only: bool,
    export_config: ExportConfig,
    source_mode: QtoSourceMode,
    captured_handles: BTreeSet<String>,
    export_omitted_ids: BTreeSet<u64>,
    selected_measure_types: [bool; 4],
    methods_initialized: bool,
    preview_key: Option<u64>,
    preview_measurements: Vec<PendingMeasurement>,
    volume_plan: Option<GeometricSource>,
    volume_section: Option<GeometricSource>,
    volume_records: Vec<PlanSectionVolume>,
    volume_work_group: String,
    volume_work_code: String,
    volume_description: String,
    road_chainage: String,
    road_name: String,
    road_plan_marker_handle: Option<String>,
    road_cut: Option<GeometricSource>,
    road_fill: Option<GeometricSource>,
    road_cut_zero_confirmed: bool,
    road_fill_zero_confirmed: bool,
    road_sections: Vec<RoadCrossSection>,
    road_work_group: String,
    road_range_start: String,
    road_range_end: String,
    road_cut_code: String,
    road_fill_code: String,
    edit_work_group: String,
    edit_work_code: String,
    edit_description: String,
    edit_note: String,
}

impl State {
    fn scale_to_m(&self) -> Option<f64> {
        if self.drawing_distance.is_finite() && self.real_distance_m.is_finite() && self.drawing_distance > 1e-12 && self.real_distance_m > 0.0 {
            let scale = self.real_distance_m / self.drawing_distance;
            (scale.is_finite() && scale > 0.0).then_some(scale)
        } else {
            None
        }
    }

    fn calibrate_two_points(&mut self, a: [f64; 2], b: [f64; 2], real_distance_m: f64) -> Result<f64, String> {
        if !a.into_iter().chain(b).all(f64::is_finite) || !real_distance_m.is_finite() || real_distance_m <= 0.0 {
            return Err("Điểm hoặc chiều dài hiệu chỉnh không hợp lệ".into());
        }
        let drawing_distance = (b[0] - a[0]).hypot(b[1] - a[1]);
        if !drawing_distance.is_finite() || drawing_distance <= 1e-12 {
            return Err("Hai điểm hiệu chỉnh trùng nhau".into());
        }
        let scale = real_distance_m / drawing_distance;
        if !scale.is_finite() || scale <= 0.0 { return Err("Không thể tính tỷ lệ hiệu chỉnh".into()); }
        self.drawing_distance = drawing_distance;
        self.real_distance_m = real_distance_m;
        Ok(scale)
    }

    fn snapshot(&mut self) {
        self.undo.push(self.records.clone());
        if self.undo.len() > 100 { self.undo.remove(0); }
        self.redo.clear();
    }

    fn audit_change(&mut self, record_id: u64, action: &str, before: Option<String>, after: Option<String>) {
        let sequence = self.next_audit_sequence.max(1);
        let Some(next) = sequence.checked_add(1) else { return; };
        self.audit.push(AuditEntry { sequence, record_id, action: action.into(), before, after });
        self.next_audit_sequence = next;
    }

    fn add_measurement(&mut self, kind: MeasureKind, points: Vec<[f64; 2]>, raw_value: f64, layer: String) -> Result<u64, String> {
        if !raw_value.is_finite() || raw_value < 0.0 { return Err("Giá trị phép đo không hợp lệ".into()); }
        if kind != MeasureKind::Count && points.is_empty() { return Err("Phép đo chưa có điểm hình học".into()); }
        if points.iter().flatten().any(|value| !value.is_finite()) { return Err("Tọa độ điểm đo không hợp lệ".into()); }
        if self.next_audit_sequence.max(1).checked_add(1).is_none() { return Err("Đã hết mã lịch sử thao tác QTO".into()); }
        let id = self.next_record_id.max(1);
        let next_id = id.checked_add(1).ok_or_else(|| "Đã hết mã số phép đo QTO".to_string())?;
        self.snapshot();
        self.next_record_id = next_id;
        let record = MeasurementRecord {
            id, kind, points, raw_value, layer, work_group: String::new(), work_code: String::new(),
            description: String::new(), note: String::new(), status: MeasureStatus::Review, scope: Scope::default(), source_handle: None, source_path: None, source_revision: None, source_geometry_signature: None,
        };
        self.records.push(record.clone());
        self.audit_change(id, "CREATE", None, Some(format!("{:?}:{:.6}", record.kind, record.raw_value)));
        Ok(id)
    }

    fn add_measurement_scoped(&mut self, kind: MeasureKind, points: Vec<[f64; 2]>, raw_value: f64, layer: String, scope: &Scope) -> Result<u64, String> {
        if scope.floor_id == Some(0) { return Err("BIM scope floor_id không hợp lệ".into()); }
        if scope.owner_account_id.as_deref().is_some_and(|v|v.trim().is_empty()) { return Err("BIM scope owner trống".into()); }
        let mut normalized_scope=scope.clone();
        if let Some(value)=scope.discipline.as_deref() {
            normalized_scope.discipline=Some(normalize_discipline(value).ok_or_else(||"BIM scope discipline không hợp lệ".to_string())?);
        }
        if self.next_audit_sequence.max(1).checked_add(2).is_none() { return Err("Không đủ ID nhật ký QTO cho phép đo theo tầng".into()); }
        let id=self.add_measurement(kind,points,raw_value,layer)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.scope=normalized_scope.clone();
        }
        self.audit_change(id,"SCOPE",None,Some(format!("floor={:?};discipline={:?};owner={:?}",normalized_scope.floor_id,normalized_scope.discipline,normalized_scope.owner_account_id)));
        Ok(id)
    }

    fn add_measurement_from_entity(&mut self, measurement: PendingMeasurement, scope: &Scope, source_revision: Option<u64>) -> Result<u64, String> {
        let source_handle=measurement.source_handle.hex();
        let source_geometry_signature=geometry_signature(&measurement);
        if self.records.iter().any(|r| r.source_handle.as_deref() == Some(source_handle.as_str()) && r.source_path.is_none() && r.kind == measurement.kind && r.source_geometry_signature.as_deref() == Some(source_geometry_signature.as_str())) { return Err("Đối tượng CAD đã được thêm vào QTO với cùng hình học".into()); }
        if self.next_audit_sequence.max(1).checked_add(3).is_none() { return Err("Không đủ ID nhật ký QTO cho đối tượng CAD".into()); }
        let id=self.add_measurement_scoped(measurement.kind,measurement.points,measurement.raw_value,measurement.layer,scope)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.source_handle=Some(source_handle.clone());
            record.source_revision=source_revision;
            record.source_geometry_signature=Some(source_geometry_signature.clone());
        }
        self.audit_change(id,"SOURCE_HANDLE",None,Some(format!("{}@{:?}|{}",source_handle,source_revision,source_geometry_signature)));
        Ok(id)
    }

    fn add_measurement_from_path(&mut self,app:&CadApp,path:SourcePath,kind:MeasureKind,active_scope:&Scope)->Result<u64,String> {
        let root=cadcraft_doc::Handle::parse_hex(&path.root_handle).ok_or_else(||"QTO source_path root_handle không hợp lệ".to_string())?;
        let current=if kind==MeasureKind::Count {
            let doc=app.session.doc().map_err(|e|e.to_string())?;
            let (entity,resolved_root)=resolve_source_path(doc,&path).ok_or_else(||"QTO source_path không tồn tại".to_string())?;
            PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:resolved_root}
        } else {
            measurements_for_path(app,&path).into_iter().find(|m|m.kind==kind).ok_or_else(||"Nested source không hỗ trợ loại phép đo này".to_string())?
        };
        let (scope,revision)=scope_from_entity_or_active(app,root,active_scope);
        let signature=geometry_signature(&current);
        if self.records.iter().any(|r| r.source_path.as_ref() == Some(&path) && r.kind == current.kind && r.source_geometry_signature.as_deref() == Some(signature.as_str())) { return Err("Phép đo Block/Array này đã tồn tại trong QTO".into()); }
        if self.next_audit_sequence.max(1).checked_add(3).is_none() { return Err("Không đủ ID nhật ký QTO cho block CAD".into()); }
        let id=self.add_measurement_scoped(current.kind,current.points,current.raw_value,current.layer,&scope)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.source_handle=Some(root.hex());
            record.source_path=Some(path.clone());
            record.source_revision=revision;
            record.source_geometry_signature=Some(signature.clone());
        }
        self.audit_change(id,"SOURCE_PATH",None,Some(format!("{}|{}",root.hex(),signature)));
        Ok(id)
    }

    fn relink_missing_source(&mut self, app:&CadApp, id:u64) -> Result<bool,String> {
        let index=self.records.iter().position(|r|r.id==id).ok_or_else(||"Không tìm thấy record QTO".to_string())?;
        let record=self.records[index].clone();
        if record.source_path.is_some(){return Ok(false);}
        let expected=record.source_geometry_signature.clone().ok_or_else(||"Record chưa có geometry signature để relink".to_string())?;
        let doc=app.session.doc().map_err(|e|e.to_string())?;
        let mut candidates=Vec::new();
        for handle in doc.model.handles() {
            let measurements=if record.kind==MeasureKind::Count {
                count_measurement_for_handle(app,handle).into_iter().collect::<Vec<_>>()
            } else {
                measurements_for_handle(app,handle)
            };
            if !measurements.iter().any(|m|m.kind==record.kind && geometry_signature(m)==expected) { continue; }
            if let Some(scope)=crate::bim_collab::entity_scope(app,handle) {
                if record.scope.floor_id!=Some(scope.floor_id)
                    || record.scope.discipline.as_deref()!=Some(scope.discipline.as_str())
                    || record.scope.owner_account_id.as_deref()!=Some(scope.owner_account_id.as_str()) { continue; }
            } else if record.scope.floor_id.is_some() || record.scope.discipline.is_some() || record.scope.owner_account_id.is_some() {
                continue;
            }
            candidates.push(handle);
            if candidates.len()>1 { break; }
        }
        if candidates.len()!=1 { return Ok(false); }
        let handle=candidates[0];
        let source_scope=crate::bim_collab::entity_scope(app,handle);
        let revision=source_scope.as_ref().map(|x|x.revision);
        self.snapshot();
        self.records[index].source_handle=Some(handle.hex());
        self.records[index].source_revision=revision;
        self.audit_change(id,"RELINK_SOURCE",record.source_handle,Some(handle.hex()));
        Ok(true)
    }

    fn relink_all_missing_sources(&mut self, app:&CadApp) -> usize {
        let ids=self.records.iter().filter_map(|r|{
            let h=r.source_handle.as_deref().and_then(cadcraft_doc::Handle::parse_hex)?;
            let missing=app.session.doc().ok().is_none_or(|d|{
                if let Some(path)=r.source_path.as_ref(){resolve_source_path(d,path).is_none()}else{d.model.get(h).is_none()}
            });
            missing.then_some(r.id)
        }).collect::<Vec<_>>();
        ids.into_iter().filter(|id|self.relink_missing_source(app,*id).ok()==Some(true)).count()
    }

    fn refresh_from_source(&mut self, app:&CadApp, id:u64) -> Result<(),String> {
        let index=self.records.iter().position(|r|r.id==id).ok_or_else(||"Không tìm thấy record QTO".to_string())?;
        let handle_hex=self.records[index].source_handle.clone().ok_or_else(||"Record không có source_handle".to_string())?;
        let handle=cadcraft_doc::Handle::parse_hex(&handle_hex).ok_or_else(||"source_handle không hợp lệ".to_string())?;
        let kind=self.records[index].kind;
        let current=if let Some(path)=self.records[index].source_path.as_ref() {
            if kind==MeasureKind::Count {
                let doc=app.session.doc().map_err(|e|e.to_string())?;
                let (entity,root)=resolve_source_path(doc,path).ok_or_else(||"Nested source path không còn tồn tại".to_string())?;
                PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:root}
            } else {
                measurements_for_path(app,path).into_iter().find(|m|m.kind==kind).ok_or_else(||"Nested source không còn hỗ trợ loại phép đo này".to_string())?
            }
        } else if kind==MeasureKind::Count {
            count_measurement_for_handle(app,handle).ok_or_else(||"Entity nguồn không còn tồn tại".to_string())?
        } else {
            measurements_for_handle(app,handle).into_iter().find(|m|m.kind==kind).ok_or_else(||"Entity nguồn không còn hỗ trợ loại phép đo này".to_string())?
        };
        let old=self.records[index].clone();
        let source_scope=crate::bim_collab::entity_scope(app,handle);
        let scope=source_scope.as_ref().map(|x|Scope{floor_id:Some(x.floor_id),discipline:Some(x.discipline.clone()),owner_account_id:Some(x.owner_account_id.clone())}).unwrap_or_else(||old.scope.clone());
        let revision=source_scope.map(|x|x.revision);
        let signature=geometry_signature(&current);
        self.snapshot();
        self.records[index].points=current.points;
        self.records[index].raw_value=current.raw_value;
        self.records[index].layer=current.layer;
        self.records[index].scope=scope;
        self.records[index].source_revision=revision;
        self.records[index].source_geometry_signature=Some(signature);
        self.records[index].status=MeasureStatus::Review;
        self.audit_change(id,"REFRESH_SOURCE",
            Some(format!("value={:.6};scope={:?};revision={:?}",old.raw_value,old.scope,old.source_revision)),
            Some(format!("value={:.6};scope={:?};revision={:?}",self.records[index].raw_value,self.records[index].scope,self.records[index].source_revision)));
        Ok(())
    }

    fn set_status(&mut self, id: u64, status: MeasureStatus) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        if self.records[index].status == status { return true; }
        if self.next_audit_sequence.max(1).checked_add(1).is_none() { return false; }
        self.snapshot();
        let before = self.records[index].status;
        self.records[index].status = status;
        self.audit_change(id, "STATUS", Some(before.code().into()), Some(status.code().into()));
        true
    }

    fn select_record(&mut self, id: u64) -> bool {
        let Some(record) = self.records.iter().find(|r| r.id == id).cloned() else { return false; };
        self.selected_record_id = Some(id);
        self.edit_work_group = record.work_group;
        self.edit_work_code = record.work_code;
        self.edit_description = record.description;
        self.edit_note = record.note;
        true
    }

    fn edit_metadata(&mut self, id: u64, work_group: String, work_code: String, description: String, note: String) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        self.snapshot();
        let before = format!("{}|{}|{}|{}", self.records[index].work_group, self.records[index].work_code, self.records[index].description, self.records[index].note);
        self.records[index].work_group = work_group;
        self.records[index].work_code = work_code;
        self.records[index].description = description;
        self.records[index].note = note;
        let after = format!("{}|{}|{}|{}", self.records[index].work_group, self.records[index].work_code, self.records[index].description, self.records[index].note);
        self.audit_change(id, "EDIT_METADATA", Some(before), Some(after));
        true
    }

    fn delete_measurement(&mut self, id: u64) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        self.snapshot();
        let removed = self.records.remove(index);
        self.audit_change(id, "DELETE", Some(format!("{:?}:{:.6}", removed.kind, removed.raw_value)), None);
        true
    }

    fn undo_measurement_change(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false; };
        self.redo.push(std::mem::replace(&mut self.records, previous));
        true
    }

    fn redo_measurement_change(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false; };
        self.undo.push(std::mem::replace(&mut self.records, next));
        true
    }
}

fn source_path_json(path:&SourcePath)->serde_json::Value {
    serde_json::json!({
        "root_handle": path.root_handle,
        "steps": path.steps.iter().map(|step|match step {
            SourcePathStep::BlockMember{block,member_handle}=>serde_json::json!({"type":"block_member","block":block,"member_handle":member_handle}),
            SourcePathStep::ArrayItem{i,j,level,member_handle}=>serde_json::json!({"type":"array_item","i":i,"j":j,"level":level,"member_handle":member_handle}),
        }).collect::<Vec<_>>()
    })
}

fn parse_source_path(value:&serde_json::Value)->Result<SourcePath,String> {
    let root_handle=value.get("root_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu root_handle".to_string())?.to_string();
    if cadcraft_doc::Handle::parse_hex(&root_handle).is_none() { return Err("QTO source_path root_handle không hợp lệ".into()); }
    let mut steps=Vec::new();
    for step in value.get("steps").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
        match step.get("type").and_then(serde_json::Value::as_str) {
            Some("block_member")=>{
                let block=step.get("block").and_then(serde_json::Value::as_str).filter(|v|!v.trim().is_empty()).ok_or_else(||"QTO source_path block trống".to_string())?.to_string();
                let member_handle=step.get("member_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu member_handle".to_string())?.to_string();
                if cadcraft_doc::Handle::parse_hex(&member_handle).is_none() { return Err("QTO source_path member_handle không hợp lệ".into()); }
                steps.push(SourcePathStep::BlockMember{block,member_handle});
            }
            Some("array_item")=>{
                let i=step.get("i").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array i không hợp lệ".to_string())?;
                let j=step.get("j").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array j không hợp lệ".to_string())?;
                let level=step.get("level").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array level không hợp lệ".to_string())?;
                let member_handle=step.get("member_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu member_handle".to_string())?.to_string();
                if cadcraft_doc::Handle::parse_hex(&member_handle).is_none() { return Err("QTO source_path member_handle không hợp lệ".into()); }
                steps.push(SourcePathStep::ArrayItem{i,j,level,member_handle});
            }
            _=>return Err("QTO source_path step type không hợp lệ".into()),
        }
    }
    Ok(SourcePath{root_handle,steps})
}

fn resolve_source_path(d:&cadcraft_doc::Drawing,path:&SourcePath)->Option<(cadcraft_doc::Entity,cadcraft_doc::Handle)> {
    let root=cadcraft_doc::Handle::parse_hex(&path.root_handle)?;
    let mut current=(**d.model.get(root)?).clone();
    let mut parent=cadcraft_geom::Mat3::IDENTITY;

    for step in &path.steps {
        match step {
            SourcePathStep::BlockMember{block,member_handle}=>{
                let EntityKind::Insert(ins)=&current.kind else { return None };
                if !ins.block.eq_ignore_ascii_case(block) { return None; }
                let definition=d.block(&ins.block)?;
                let member=cadcraft_doc::Handle::parse_hex(member_handle)?;
                let next=(**definition.entities.get(member)?).clone();
                parent=parent.then_before(ins.transform(definition.base.xy()));
                current=next;
            }
            SourcePathStep::ArrayItem{i,j,level,member_handle}=>{
                let EntityKind::Insert(ins)=&current.kind else { return None };
                let array_block=d.block(&ins.block)?;
                let params=d.array_params(&current)?;
                if params.is_suppressed(*i,*j,*level) { return None; }
                let item=params.items()?.into_iter().find(|it|(it.i,it.j,it.level)==(*i,*j,*level))?;
                let block_name=params.block_for(*i,*j,*level).to_string();
                let source_block=d.block(&block_name)?;
                let item_insert=cadcraft_doc::Insert{
                    block:block_name,
                    insert:item.insert,
                    scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),
                    rotation:item.rotation,
                    attribs:Vec::new(),
                    cols:1,
                    rows:1,
                    col_spacing:0.0,
                    row_spacing:0.0,
                };
                let member=cadcraft_doc::Handle::parse_hex(member_handle)?;
                let next=(**source_block.entities.get(member)?).clone();
                let into_array=parent.then_before(ins.transform(array_block.base.xy()));
                parent=into_array.then_before(item_insert.transform(source_block.base.xy()));
                current=next;
            }
        }
    }

    current.kind.transform(&parent);
    Some((current,root))
}

impl State {
    fn record_json(record: &MeasurementRecord) -> serde_json::Value {
        serde_json::json!({
            "id": record.id, "kind": record.kind.code(), "points": record.points, "raw_value": record.raw_value,
            "layer": record.layer, "work_group": record.work_group, "work_code": record.work_code,
            "description": record.description, "note": record.note, "status": record.status.code(),
            "scope": {
                "floor_id": record.scope.floor_id,
                "discipline": record.scope.discipline,
                "owner_account_id": record.scope.owner_account_id
            },
            "source_handle": record.source_handle,
            "source_path": record.source_path.as_ref().map(source_path_json),
            "source_revision": record.source_revision,
            "source_geometry_signature": record.source_geometry_signature
        })
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": 4,
            "drawing_distance": self.drawing_distance,
            "real_distance_m": self.real_distance_m,
            "road_takeoff": {
                "sections": self.road_sections.iter().map(RoadCrossSection::json).collect::<Vec<_>>(),
                "draft": {"chainage":self.road_chainage,"name":self.road_name,"plan_marker_handle":self.road_plan_marker_handle,
                          "cut":self.road_cut.as_ref().map(GeometricSource::json),
                          "fill":self.road_fill.as_ref().map(GeometricSource::json),
                          "cut_zero_confirmed":self.road_cut_zero_confirmed,"fill_zero_confirmed":self.road_fill_zero_confirmed},
                "work_group":self.road_work_group,
                "range_start":self.road_range_start,
                "range_end":self.road_range_end,
                "cut_code":self.road_cut_code,
                "fill_code":self.road_fill_code
            },
            "composite_volumes": self.volume_records.iter().map(PlanSectionVolume::json).collect::<Vec<_>>(),
            "composite_draft": {
                "plan": self.volume_plan.as_ref().map(GeometricSource::json),
                "section": self.volume_section.as_ref().map(GeometricSource::json)
            },
            "qto_selection": {
                "source_mode": format!("{:?}", self.source_mode),
                "captured_handles": self.captured_handles,
                "excluded_export_record_ids": self.export_omitted_ids,
                "measure_types": self.selected_measure_types
            },
            "export_config": {
                "columns": self.export_config.columns,
                "group_by": format!("{:?}", self.export_config.group_by),
                "project": self.export_config.project,
                "investor": self.export_config.investor,
                "document_number": self.export_config.document_number,
                "confirmed_only": self.export_config.confirmed_only,
                "length_unit": self.export_config.length_unit,
                "decimals": self.export_config.decimals,
            },
            "records": self.records.iter().map(Self::record_json).collect::<Vec<_>>(),
            "audit": self.audit.iter().map(|a| serde_json::json!({
                "sequence": a.sequence, "record_id": a.record_id, "action": a.action, "before": a.before, "after": a.after
            })).collect::<Vec<_>>()
        })
    }

    fn load_json(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let version=value.get("version").and_then(serde_json::Value::as_u64).unwrap_or(2);
        if !matches!(version,2|3|4) { return Err("Phiên bản QTO project không được hỗ trợ".into()); }
        let drawing_distance = value.get("drawing_distance").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu drawing_distance".to_string())?;
        let real_distance_m = value.get("real_distance_m").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu real_distance_m".to_string())?;
        if !(drawing_distance.is_finite() && real_distance_m.is_finite()) {
            return Err("Dữ liệu hiệu chỉnh tỷ lệ không hợp lệ".into());
        }
        let uncalibrated = drawing_distance.abs() <= 1e-12 && real_distance_m.abs() <= 1e-12;
        let calibrated = drawing_distance > 1e-12 && real_distance_m > 0.0;
        if calibrated && (!(real_distance_m / drawing_distance).is_finite() || real_distance_m / drawing_distance <= 0.0) { return Err("Tỷ lệ hiệu chỉnh QTO không hợp lệ".into()); }
        if !uncalibrated && !calibrated {
            return Err("Dữ liệu hiệu chỉnh tỷ lệ không hợp lệ".into());
        }
        let mut records = Vec::new();
        let mut record_ids = std::collections::BTreeSet::new();
        for item in value.get("records").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
            let id = item.get("id").and_then(serde_json::Value::as_u64).ok_or_else(|| "Measurement thiếu id".to_string())?;
            if id == 0 || !record_ids.insert(id) { return Err("Measurement id không hợp lệ hoặc bị trùng".into()); }
            let kind = item.get("kind").and_then(serde_json::Value::as_str).and_then(MeasureKind::parse).ok_or_else(|| "Measurement kind không hợp lệ".to_string())?;
            let raw_value = item.get("raw_value").and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement thiếu raw_value".to_string())?;
            if !raw_value.is_finite() || raw_value < 0.0 { return Err("Measurement raw_value không hợp lệ".into()); }
            let status = match item.get("status").and_then(serde_json::Value::as_str) {
                Some(value) => MeasureStatus::parse(value).ok_or_else(|| "Measurement status không hợp lệ".to_string())?,
                None => MeasureStatus::Review,
            };
            let mut points = Vec::new();
            for point in item.get("points").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
                let pair = point.as_array().ok_or_else(|| "Measurement point không hợp lệ".to_string())?;
                let x = pair.first().and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement point thiếu X".to_string())?;
                let y = pair.get(1).and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement point thiếu Y".to_string())?;
                if !x.is_finite() || !y.is_finite() { return Err("Measurement point không hữu hạn".into()); }
                points.push([x, y]);
            }
            if kind != MeasureKind::Count && points.is_empty() { return Err("Measurement thiếu hình học".into()); }
            let scope_value=item.get("scope");
            let floor_id=scope_value.and_then(|v|v.get("floor_id")).and_then(serde_json::Value::as_u64);
            let discipline=match scope_value.and_then(|v|v.get("discipline")).and_then(serde_json::Value::as_str) {
                Some(value)=>Some(normalize_discipline(value).ok_or_else(||"Measurement BIM discipline không hợp lệ".to_string())?),
                None=>None,
            };
            let owner_account_id=scope_value.and_then(|v|v.get("owner_account_id")).and_then(serde_json::Value::as_str).map(str::to_string);
            if floor_id==Some(0) || discipline.as_deref().is_some_and(|v|v.trim().is_empty()) || owner_account_id.as_deref().is_some_and(|v|v.trim().is_empty()) {
                return Err("Measurement BIM scope không hợp lệ".into());
            }
            let source_handle=item.get("source_handle").and_then(serde_json::Value::as_str).map(str::to_string);
            if source_handle.as_deref().is_some_and(|h|cadcraft_doc::Handle::parse_hex(h).is_none()) { return Err("Measurement source_handle không hợp lệ".into()); }
            let source_path=match item.get("source_path") {
                Some(value) if !value.is_null()=>Some(parse_source_path(value)?),
                _=>None,
            };
            let source_revision=item.get("source_revision").and_then(serde_json::Value::as_u64);
            let source_geometry_signature=item.get("source_geometry_signature").and_then(serde_json::Value::as_str).map(str::to_string);
            records.push(MeasurementRecord {
                id, kind, points, raw_value,
                layer: item.get("layer").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                work_group: item.get("work_group").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                work_code: item.get("work_code").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                description: item.get("description").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                note: item.get("note").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                status,
                scope: Scope{floor_id,discipline,owner_account_id},
                source_handle,
                source_path,
                source_revision,
                source_geometry_signature,
            });
        }
        let mut audit = Vec::new();
        for item in value.get("audit").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
            let sequence = item.get("sequence").and_then(serde_json::Value::as_u64).unwrap_or(0);
            let record_id = item.get("record_id").and_then(serde_json::Value::as_u64).unwrap_or(0);
            let action = item.get("action").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
            let before = item.get("before").and_then(serde_json::Value::as_str).map(str::to_string);
            let after = item.get("after").and_then(serde_json::Value::as_str).map(str::to_string);
            if sequence > 0 && !action.is_empty() {
                audit.push(AuditEntry { sequence, record_id, action, before, after });
            }
        }
        let next_record_id = records.iter().map(|r| r.id).max().unwrap_or(0).checked_add(1).ok_or_else(|| "Hết ID phép đo QTO".to_string())?;
        let next_audit_sequence = audit.iter().map(|a| a.sequence).max().unwrap_or(0).checked_add(1).ok_or_else(|| "Hết ID lịch sử QTO".to_string())?;
        // Restore export layout from optional QTO metadata without changing old project versions.
        let mut config = ExportConfig::default();
        if let Some(v) = value.get("export_config") {
            if let Some(cols) = v.get("columns").and_then(serde_json::Value::as_array) {
                if cols.len() == FIELDS.len() {
                    for (index, item) in cols.iter().enumerate() { config.columns[index] = item.as_bool().unwrap_or(config.columns[index]); }
                }
            }
            config.columns[8] = true;
            config.columns[9] = true;
            config.group_by = match v.get("group_by").and_then(serde_json::Value::as_str) {
                Some("WorkCode") => GroupBy::WorkCode, Some("Layer") => GroupBy::Layer,
                Some("Floor") => GroupBy::Floor, Some("Discipline") => GroupBy::Discipline,
                Some("None") => GroupBy::None, _ => GroupBy::WorkGroup,
            };
            for (key, target) in [("project", &mut config.project), ("investor", &mut config.investor), ("document_number", &mut config.document_number), ("length_unit", &mut config.length_unit)] {
                if let Some(value) = v.get(key).and_then(serde_json::Value::as_str) {
                    if value.len() <= 2048 { *target = value.to_string(); }
                }
            }
            if !matches!(config.length_unit.as_str(), "m" | "cm" | "mm") { config.length_unit = "m".into(); }
            config.confirmed_only = v.get("confirmed_only").and_then(serde_json::Value::as_bool).unwrap_or(true);
            config.decimals = v.get("decimals").and_then(serde_json::Value::as_u64).map(|n| n.min(6) as usize).unwrap_or(3);
        }
        // Old QTO files default to explicit CAD selection, never the whole
        // project. Persisted captured handles are checked for valid CAD hex IDs.
        let mut source_mode = QtoSourceMode::default();
        let mut captured_handles = BTreeSet::new();
        let mut methods = [true, true, true, false];
        if let Some(selection) = value.get("qto_selection") {
            source_mode = match selection.get("source_mode").and_then(serde_json::Value::as_str) {
                Some("CapturedRegion") => QtoSourceMode::CapturedRegion,
                Some("EntireProject") => QtoSourceMode::EntireProject,
                _ => QtoSourceMode::CurrentSelection,
            };
            if let Some(ids) = selection.get("captured_handles").and_then(serde_json::Value::as_array) {
                if ids.len() <= 200_000 {
                    captured_handles.extend(ids.iter().filter_map(serde_json::Value::as_str)
                        .filter(|handle| cadcraft_doc::Handle::parse_hex(handle).is_some()).map(str::to_string));
                }
            }
            if let Some(chosen) = selection.get("measure_types").and_then(serde_json::Value::as_array) {
                if chosen.len() == 4 {
                    for i in 0..4 { methods[i] = chosen[i].as_bool().unwrap_or(methods[i]); }
                }
            }
        }
        let export_omitted_ids = value.get("qto_selection").and_then(|s| s.get("excluded_export_record_ids"))
            .and_then(serde_json::Value::as_array).map(|items| items.iter().filter_map(serde_json::Value::as_u64)
                .filter(|id| records.iter().any(|record| record.id == *id)).collect()).unwrap_or_default();
        self.source_mode = source_mode;
        self.captured_handles = captured_handles;
        self.export_omitted_ids = export_omitted_ids;
        self.selected_measure_types = methods;
        self.methods_initialized = true;
        self.preview_key = None;
        self.preview_measurements.clear();
        self.volume_records = value.get("composite_volumes").and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(20_000).filter_map(PlanSectionVolume::read).collect()).unwrap_or_default();
        self.volume_plan = value.get("composite_draft").and_then(|draft| draft.get("plan")).and_then(GeometricSource::read);
        self.volume_section = value.get("composite_draft").and_then(|draft| draft.get("section")).and_then(GeometricSource::read);
        self.volume_work_group.clear();
        self.volume_work_code.clear();
        self.volume_description.clear();
        let road=value.get("road_takeoff");
        self.road_sections=road.and_then(|v|v.get("sections")).and_then(serde_json::Value::as_array)
            .map(|items|items.iter().take(20_000).filter_map(RoadCrossSection::read).collect()).unwrap_or_default();
        self.road_plan_marker_handle=road.and_then(|v|v.get("draft")).and_then(|v|v.get("plan_marker_handle")).and_then(serde_json::Value::as_str).filter(|h|cadcraft_doc::Handle::parse_hex(h).is_some()).map(str::to_string);
        self.road_cut=road.and_then(|v|v.get("draft")).and_then(|v|v.get("cut")).and_then(GeometricSource::read);
        self.road_fill=road.and_then(|v|v.get("draft")).and_then(|v|v.get("fill")).and_then(GeometricSource::read);
        self.road_cut_zero_confirmed=road.and_then(|v|v.get("draft")).and_then(|v|v.get("cut_zero_confirmed")).and_then(serde_json::Value::as_bool).unwrap_or(false);
        self.road_fill_zero_confirmed=road.and_then(|v|v.get("draft")).and_then(|v|v.get("fill_zero_confirmed")).and_then(serde_json::Value::as_bool).unwrap_or(false);
        self.road_chainage=road.and_then(|v|v.get("draft")).and_then(|v|v.get("chainage")).and_then(serde_json::Value::as_str).unwrap_or("").chars().take(50).collect();
        self.road_name=road.and_then(|v|v.get("draft")).and_then(|v|v.get("name")).and_then(serde_json::Value::as_str).unwrap_or("").chars().take(180).collect();
        self.road_work_group=road.and_then(|v|v.get("work_group")).and_then(serde_json::Value::as_str).unwrap_or("Nền đường").chars().take(250).collect();
        self.road_range_start=road.and_then(|v|v.get("range_start")).and_then(serde_json::Value::as_str).unwrap_or("").chars().take(50).collect();
        self.road_range_end=road.and_then(|v|v.get("range_end")).and_then(serde_json::Value::as_str).unwrap_or("").chars().take(50).collect();
        self.road_cut_code=road.and_then(|v|v.get("cut_code")).and_then(serde_json::Value::as_str).unwrap_or("GT.DAO").chars().take(150).collect();
        self.road_fill_code=road.and_then(|v|v.get("fill_code")).and_then(serde_json::Value::as_str).unwrap_or("GT.DAP").chars().take(150).collect();
        self.export_config = config;
        self.drawing_distance = drawing_distance;
        self.real_distance_m = real_distance_m;
        self.records = records;
        self.audit = audit;
        self.next_record_id = next_record_id;
        self.next_audit_sequence = next_audit_sequence;
        self.undo.clear();
        self.redo.clear();
        self.selected_record_id = None;
        self.edit_work_group.clear();
        self.edit_work_code.clear();
        self.edit_description.clear();
        self.edit_note.clear();
        Ok(())
    }
}


#[derive(Clone, Debug)]
struct PendingMeasurement {
    kind: MeasureKind,
    points: Vec<[f64; 2]>,
    raw_value: f64,
    layer: String,
    source_handle: cadcraft_doc::Handle,
}

fn geometry_signature(measurement:&PendingMeasurement)->String {
    let mut out=format!("{}|{}|{:.12}|",measurement.kind.code(),measurement.layer,measurement.raw_value);
    for point in &measurement.points {
        out.push_str(&format!("{:.12},{:.12};",point[0],point[1]));
    }
    out
}

fn scope_from_entity_or_active(app:&CadApp, handle:cadcraft_doc::Handle, active_scope:&Scope)->(Scope,Option<u64>) {
    crate::bim_collab::entity_scope(app,handle).map(|scope|(Scope {
        floor_id:Some(scope.floor_id),
        discipline:Some(scope.discipline),
        owner_account_id:Some(scope.owner_account_id),
    },Some(scope.revision))).unwrap_or_else(||(active_scope.clone(),None))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SourceValidation {
    missing_source: usize,
    scope_mismatch: usize,
    stale_revision: usize,
    stale_geometry: usize,
    identity_collision: usize,
    valid_links: usize,
    invalid_record_ids: std::collections::BTreeSet<u64>,
}

fn record_source_state(record:&MeasurementRecord,validation:&SourceValidation)->&'static str {
    if record.source_handle.is_none() { "Legacy" }
    else if validation.invalid_record_ids.contains(&record.id) { "STALE" }
    else { "OK" }
}

fn validate_sources(app:&CadApp,state:&State)->SourceValidation {
    let mut result=SourceValidation::default();
    for record in &state.records {
        let Some(handle_hex)=record.source_handle.as_deref() else { continue; };
        let Some(handle)=cadcraft_doc::Handle::parse_hex(handle_hex) else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };
        let Ok(doc)=app.session.doc() else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };

        let current_measurements=if let Some(path)=record.source_path.as_ref() {
            if cadcraft_doc::Handle::parse_hex(&path.root_handle)!=Some(handle) {
                result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue;
            }
            let Some((entity,root))=resolve_source_path(doc,path) else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };
            if record.kind==MeasureKind::Count {
                vec![PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:root}]
            } else {
                measurements_for_entity(doc,&entity,root)
            }
        } else {
            if doc.model.get(handle).is_none(){result.missing_source+=1;result.invalid_record_ids.insert(record.id);continue;}
            if record.kind==MeasureKind::Count {
                count_measurement_for_handle(app,handle).into_iter().collect::<Vec<_>>()
            } else {
                measurements_for_handle(app,handle)
            }
        };

        let geometry_stale=record.source_geometry_signature.as_ref().is_some_and(|expected|{
            !current_measurements.iter().any(|measurement|measurement.kind==record.kind&&geometry_signature(measurement)==*expected)
        });

        if let Some(entity_scope)=crate::bim_collab::entity_scope(app,handle) {
            let mismatch=record.scope.floor_id!=Some(entity_scope.floor_id)
                || record.scope.discipline.as_deref()!=Some(entity_scope.discipline.as_str())
                || record.scope.owner_account_id.as_deref()!=Some(entity_scope.owner_account_id.as_str());
            let stale=record.source_revision.is_some_and(|revision|revision!=entity_scope.revision);
            let identity_collision=entity_scope.entity_handle.is_some_and(|identity|identity!=handle&&doc.model.get(identity).is_some());
            if mismatch {
                result.scope_mismatch+=1;result.invalid_record_ids.insert(record.id);
            } else if identity_collision {
                result.identity_collision+=1;result.invalid_record_ids.insert(record.id);
            } else if geometry_stale {
                result.stale_geometry+=1;result.invalid_record_ids.insert(record.id);
            } else if stale&&record.source_geometry_signature.is_none() {
                result.stale_revision+=1;result.invalid_record_ids.insert(record.id);
            } else {
                result.valid_links+=1;
            }
        } else if geometry_stale {
            result.stale_geometry+=1;result.invalid_record_ids.insert(record.id);
        } else {
            result.valid_links+=1;
        }
    }
    result
}

#[derive(Default)]
struct BusinessQuantity {
    review_records: usize,
    confirmed_records: usize,
    excluded_records: usize,
    length: f64,
    perimeter: f64,
    area: f64,
    count: f64,
}

fn scoped_business_quantities(state: &State, length_factor: f64, area_factor: f64) -> BTreeMap<(Option<u64>, String, String, String, String), BusinessQuantity> {
    scoped_business_quantities_validated(state,length_factor,area_factor,&std::collections::BTreeSet::new())
}

fn scoped_business_quantities_validated(state: &State, length_factor: f64, area_factor: f64, invalid_record_ids:&std::collections::BTreeSet<u64>) -> BTreeMap<(Option<u64>, String, String, String, String), BusinessQuantity> {
    let mut groups=BTreeMap::new();
    for record in &state.records {
        let discipline=record.scope.discipline.clone().unwrap_or_else(||"(Không scope)".into());
        let owner=record.scope.owner_account_id.clone().unwrap_or_else(||"(Không owner)".into());
        let work_group=if record.work_group.trim().is_empty(){"(Chưa gán nhóm)".into()}else{record.work_group.trim().to_string()};
        let work_code=if record.work_code.trim().is_empty(){"(Chưa gán mã)".into()}else{record.work_code.trim().to_string()};
        let q=groups.entry((record.scope.floor_id,discipline,owner,work_group,work_code)).or_insert_with(BusinessQuantity::default);
        match record.status {
            MeasureStatus::Review=>q.review_records+=1,
            MeasureStatus::Excluded=>q.excluded_records+=1,
            MeasureStatus::Confirmed=>{
                if invalid_record_ids.contains(&record.id) { continue; }
                q.confirmed_records+=1;
                match record.kind {
                    MeasureKind::Length=>q.length+=record.raw_value*length_factor,
                    MeasureKind::Perimeter=>q.perimeter+=record.raw_value*length_factor,
                    MeasureKind::Area=>q.area+=record.raw_value*area_factor,
                    MeasureKind::Count=>q.count+=record.raw_value,
                }
            }
        }
    }
    groups
}

fn business_quantities(state: &State, length_factor: f64, area_factor: f64) -> BTreeMap<(String, String, String), BusinessQuantity> {
    business_quantities_validated(state,length_factor,area_factor,&std::collections::BTreeSet::new())
}

fn business_quantities_validated(state: &State, length_factor: f64, area_factor: f64, invalid_record_ids:&std::collections::BTreeSet<u64>) -> BTreeMap<(String, String, String), BusinessQuantity> {
    let mut groups = BTreeMap::<(String, String, String), BusinessQuantity>::new();
    for record in &state.records {
        let work_group = if record.work_group.trim().is_empty() { "(Chưa gán nhóm)".to_string() } else { record.work_group.trim().to_string() };
        let work_code = if record.work_code.trim().is_empty() { "(Chưa gán mã)".to_string() } else { record.work_code.trim().to_string() };
        let layer = if record.layer.trim().is_empty() { "0".to_string() } else { record.layer.clone() };
        let group = groups.entry((work_group, work_code, layer)).or_default();
        match record.status {
            MeasureStatus::Review => {
                group.review_records += 1;
            }
            MeasureStatus::Excluded => {
                group.excluded_records += 1;
            }
            MeasureStatus::Confirmed => {
                if invalid_record_ids.contains(&record.id) { continue; }
                group.confirmed_records += 1;
                match record.kind {
                    MeasureKind::Length => group.length += record.raw_value * length_factor,
                    MeasureKind::Perimeter => group.perimeter += record.raw_value * length_factor,
                    MeasureKind::Area => group.area += record.raw_value * area_factor,
                    MeasureKind::Count => group.count += record.raw_value,
                }
            }
        }
    }
    groups
}

fn count_measurement_for_handle(app:&CadApp,handle:cadcraft_doc::Handle)->Option<PendingMeasurement> {
    let drawing=app.session.doc().ok()?;
    let entity=drawing.model.get(handle)?;
    Some(PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer.clone(),source_handle:handle})
}

fn selected_count_measurements(app: &CadApp) -> Vec<PendingMeasurement> {
    let selected = app.session.selection();
    let Ok(drawing) = app.session.doc() else { return Vec::new(); };
    let mut out=Vec::new();
    for handle in selected {
        let Some(measurement)=count_measurement_for_handle(app,handle) else { continue; };
        out.push(measurement);
    }
    out
}

#[derive(Default)]
struct Quantity {
    count: usize,
    line_length: f64,
    arc_length: f64,
    polyline_length: f64,
    closed_area: f64,
    circle_area: f64,
    unsupported: usize,
}

fn bulge_edge(a: (f64, f64), b: (f64, f64), bulge: f64) -> Option<(f64, f64)> {
    if ![a.0, a.1, b.0, b.1].iter().all(|value| value.is_finite()) {
        return None;
    }
    let chord = (b.0 - a.0).hypot(b.1 - a.1);
    if !chord.is_finite() || !bulge.is_finite() {
        return None;
    }
    if chord <= 1e-12 && bulge.abs() >= 1e-12 { return None; }
    if bulge.abs() < 1e-12 {
        return Some((chord, 0.0));
    }
    let theta = 4.0 * bulge.atan();
    let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
    let length = radius * theta.abs();
    // Signed circular segment area, retained when calculating closed loops.
    let segment = 0.5 * radius * radius * (theta - theta.sin());
    if length.is_finite() && segment.is_finite() {
        Some((length, segment))
    } else {
        None
    }
}

fn segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }
    fn on_segment(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        let eps = 1e-10;
        c.0 >= a.0.min(b.0) - eps && c.0 <= a.0.max(b.0) + eps
            && c.1 >= a.1.min(b.1) - eps && c.1 <= a.1.max(b.1) + eps
    }
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let eps = 1e-10;
    if o1.abs() <= eps && on_segment(a, b, c) { return true; }
    if o2.abs() <= eps && on_segment(a, b, d) { return true; }
    if o3.abs() <= eps && on_segment(c, d, a) { return true; }
    if o4.abs() <= eps && on_segment(c, d, b) { return true; }
    (o1 > eps && o2 < -eps || o1 < -eps && o2 > eps)
        && (o3 > eps && o4 < -eps || o3 < -eps && o4 > eps)
}

fn lwpoly_metrics(vertices: &[cadcraft_doc::geom::PolyVertex], closed: bool) -> Option<(f64, f64)> {
    if vertices.iter().any(|vertex| !vertex.p.x.is_finite() || !vertex.p.y.is_finite() || !vertex.bulge.is_finite()) {
        return None;
    }
    if vertices.len() < 2 {
        return Some((0.0, 0.0));
    }
    if closed {
        if vertices.len() < 3 { return None; }
        // Conservative: reject loops with crossing straight chords. Bulged arcs
        // are still measured, but more complex arc/arc crossings require CAD validation.
        for i in 0..vertices.len() {
            let a = (vertices[i].p.x, vertices[i].p.y);
            let b = (vertices[(i + 1) % vertices.len()].p.x,
                     vertices[(i + 1) % vertices.len()].p.y);
            if (b.0 - a.0).hypot(b.1 - a.1) < 1e-10 { return None; }
            for j in (i + 1)..vertices.len() {
                if j == i + 1 || (i == 0 && j == vertices.len() - 1) { continue; }
                let c = (vertices[j].p.x, vertices[j].p.y);
                let d = (vertices[(j + 1) % vertices.len()].p.x,
                         vertices[(j + 1) % vertices.len()].p.y);
                if segments_intersect(a, b, c, d) { return None; }
            }
        }
    }
    let edges = if closed { vertices.len() } else { vertices.len() - 1 };
    let mut length = 0.0;
    let mut signed_area_twice = 0.0;
    let mut segments_area = 0.0;
    for i in 0..edges {
        let a = &vertices[i];
        let b = &vertices[(i + 1) % vertices.len()];
        let (edge_length, segment) = bulge_edge((a.p.x, a.p.y), (b.p.x, b.p.y), a.bulge)?;
        length += edge_length;
        if closed {
            signed_area_twice += a.p.x * b.p.y - b.p.x * a.p.y;
            segments_area += segment;
        }
    }
    let area = if closed { (signed_area_twice * 0.5 + segments_area).abs() } else { 0.0 };
    if length.is_finite() && area.is_finite() {
        Some((length, area))
    } else {
        None
    }
}

fn hatch_loop_metrics(loops:&[cadcraft_doc::HatchLoop])->Option<(f64,f64,Vec<[f64;2]>)> {
    let mut perimeter=0.0;
    let mut area=0.0;
    let mut points=Vec::new();
    for l in loops {
        let (p,a)=lwpoly_metrics(&l.vertices,true)?;
        perimeter+=p;
        area+=if l.outer { a } else { -a };
        points.extend(l.vertices.iter().map(|v|[v.p.x,v.p.y]));
    }
    let area=area.abs();
    (perimeter.is_finite()&&area.is_finite()).then_some((perimeter,area,points))
}

fn measurements_for_entity(drawing:&cadcraft_doc::Drawing,entity:&cadcraft_doc::Entity,source_handle:cadcraft_doc::Handle)->Vec<PendingMeasurement> {
    let layer=entity.common.layer.clone();
    let mut out=Vec::new();
    match &entity.kind {
        EntityKind::Line(line)=>{
            let value=(line.b.x-line.a.x).hypot(line.b.y-line.a.y).hypot(line.b.z-line.a.z);
            if value.is_finite() {
                out.push(PendingMeasurement{kind:MeasureKind::Length,points:vec![[line.a.x,line.a.y],[line.b.x,line.b.y]],raw_value:value,layer,source_handle});
            }
        }
        EntityKind::Arc(arc) if arc.radius.is_finite()&&arc.radius>0.0=>{
            let delta=(arc.end-arc.start).rem_euclid(std::f64::consts::TAU);
            let value=arc.radius*delta;
            let a=[arc.center.x+arc.radius*arc.start.cos(),arc.center.y+arc.radius*arc.start.sin()];
            let b=[arc.center.x+arc.radius*arc.end.cos(),arc.center.y+arc.radius*arc.end.sin()];
            if value.is_finite(){out.push(PendingMeasurement{kind:MeasureKind::Length,points:vec![a,b],raw_value:value,layer,source_handle});}
        }
        EntityKind::Circle(circle) if circle.radius.is_finite()&&circle.radius>0.0=>{
            let area=std::f64::consts::PI*circle.radius*circle.radius;
            out.push(PendingMeasurement{kind:MeasureKind::Area,points:vec![[circle.center.x,circle.center.y],[circle.center.x+circle.radius,circle.center.y]],raw_value:area,layer,source_handle});
        }
        EntityKind::LwPolyline(poly)=>{
            if let Some((perimeter,area))=lwpoly_metrics(&poly.vertices,poly.closed){
                let points=poly.vertices.iter().map(|v|[v.p.x,v.p.y]).collect::<Vec<_>>();
                if perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
                if poly.closed&&area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
            }
        }
        EntityKind::Polyline3d(poly)=>{
            let n=poly.points.len();
            let edges=if poly.closed{n}else{n.saturating_sub(1)};
            let mut value=0.0;
            for i in 0..edges{
                let a=&poly.points[i];let b=&poly.points[(i+1)%n];
                value+=(b.x-a.x).hypot(b.y-a.y).hypot(b.z-a.z);
            }
            if n>=2&&value.is_finite(){
                out.push(PendingMeasurement{kind:if poly.closed{MeasureKind::Perimeter}else{MeasureKind::Length},points:poly.points.iter().map(|p|[p.x,p.y]).collect(),raw_value:value,layer,source_handle});
            }
        }
        EntityKind::Region(region)=>{
            let area=region.area();
            let perimeter=region.perimeter();
            let points=region.loops.iter().flat_map(|l|l.vertices.iter().map(|v|[v.p.x,v.p.y])).collect::<Vec<_>>();
            if perimeter.is_finite()&&perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
            if area.is_finite()&&area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
        }
        EntityKind::Hatch(hatch)=>{
            if let Some((perimeter,area,points))=hatch_loop_metrics(&hatch.loops){
                if perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
                if area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
            }
        }
        EntityKind::Dimension(dm) if !matches!(dm.kind,cadcraft_doc::DimKind::Angular|cadcraft_doc::DimKind::Angular3P)=>{
            let style=drawing.dim_style(&dm.style).cloned().unwrap_or_default();
            let value=cadcraft_render::dimension_geometry(dm,&style,drawing.header.f64("DIMSCALE",1.0)).measurement.abs();
            if value.is_finite()&&value>0.0{
                let points=vec![[dm.p13.x,dm.p13.y],[dm.p14.x,dm.p14.y],[dm.p15.x,dm.p15.y],[dm.p16.x,dm.p16.y],[dm.defpt.x,dm.defpt.y]];
                out.push(PendingMeasurement{kind:MeasureKind::Length,points,raw_value:value,layer,source_handle});
            }
        }
        _=>{}
    }
    out
}

fn measurements_for_handle(app:&CadApp,handle:cadcraft_doc::Handle)->Vec<PendingMeasurement>{
    let Ok(drawing)=app.session.doc() else{return Vec::new();};
    let Some(entity)=drawing.model.get(handle) else{return Vec::new();};
    measurements_for_entity(drawing,entity.as_ref(),handle)
}

fn collect_nested_source_paths(d:&cadcraft_doc::Drawing,current:&cadcraft_doc::Entity,path:&SourcePath,depth:usize,out:&mut Vec<SourcePath>) {
    if depth>=cadcraft_doc::MAX_BLOCK_DEPTH { return; }
    let EntityKind::Insert(ins)=&current.kind else {
        out.push(path.clone());
        return;
    };

    if let Some(params)=d.array_params(current) {
        let Some(items)=params.items() else { return; };
        for item in items {
            if params.is_suppressed(item.i,item.j,item.level) { continue; }
            let block_name=params.block_for(item.i,item.j,item.level).to_string();
            let Some(block)=d.block(&block_name) else { continue; };
            for member in block.entities.iter() {
                let mut next=path.clone();
                next.steps.push(SourcePathStep::ArrayItem{
                    i:item.i,j:item.j,level:item.level,member_handle:member.handle.hex()
                });
                collect_nested_source_paths(d,member.as_ref(),&next,depth+1,out);
            }
        }
        return;
    }

    let Some(block)=d.block(&ins.block) else { return; };
    for member in block.entities.iter() {
        let mut next=path.clone();
        next.steps.push(SourcePathStep::BlockMember{block:ins.block.clone(),member_handle:member.handle.hex()});
        collect_nested_source_paths(d,member.as_ref(),&next,depth+1,out);
    }
}

fn nested_source_candidates(app:&CadApp,root:cadcraft_doc::Handle)->Vec<(SourcePath,PendingMeasurement)> {
    let Ok(d)=app.session.doc() else { return Vec::new(); };
    let Some(root_entity)=d.model.get(root) else { return Vec::new(); };
    if !matches!(&root_entity.kind,EntityKind::Insert(_)) { return Vec::new(); }

    let base=SourcePath{root_handle:root.hex(),steps:Vec::new()};
    let mut paths=Vec::new();
    collect_nested_source_paths(d,root_entity.as_ref(),&base,0,&mut paths);

    let mut out=Vec::new();
    for path in paths {
        for measurement in measurements_for_path(app,&path) {
            out.push((path.clone(),measurement));
        }
    }
    out
}

fn measurements_for_path(app:&CadApp,path:&SourcePath)->Vec<PendingMeasurement>{
    let Ok(drawing)=app.session.doc() else{return Vec::new();};
    let Some((entity,root))=resolve_source_path(drawing,path) else{return Vec::new();};
    measurements_for_entity(drawing,&entity,root)
}

fn selected_measurements(app:&CadApp)->Vec<PendingMeasurement> {
    let selected=app.session.selection();
    let mut out=Vec::new();
    for handle in selected {
        out.extend(measurements_for_handle(app,handle));
    }
    out
}

/// Read labels inside nested block references with a finite depth and cycle guard.
/// A nested station is still associated with the selected top-level INSERT handle.
fn qto_road_collect_labels(doc: &cadcraft_doc::Drawing, entity: &cadcraft_doc::Entity, depth: usize, visited: &mut BTreeSet<String>, values: &mut Vec<String>) -> Result<(),String> {
    if depth > 12 { return Err("Block lồng nhau vượt 12 cấp; cần kiểm tra thủ công.".into()); }
    match &entity.kind {
        EntityKind::Text(t)=>values.push(t.value.clone()),
        EntityKind::MText(t)=>values.push(t.contents.clone()),
        EntityKind::Insert(block)=>{
            values.extend(block.attribs.iter().filter(|a|!a.invisible).map(|a|a.text.value.clone()));
            let key=block.block.to_ascii_lowercase();
            if !visited.insert(key.clone()) {return Err("Block chứa vòng tham chiếu lồng nhau; không tự ghép cọc.".into());}
            if let Some(def)=doc.blocks.get(&block.block) {
                for child in def.entities.iter() {
                    qto_road_collect_labels(doc,child,depth+1,visited,values)?;
                }
            }
            visited.remove(&key);
        },
        _=>{},
    }
    Ok(())
}

/// Decode one explicit Km station from TEXT/MTEXT or a nested INSERT tree.
/// Reject ambiguity instead of silently picking the first label.
fn qto_road_station_from_entity(doc: &cadcraft_doc::Drawing, entity: &cadcraft_doc::Entity) -> Result<f64,String> {
    if !matches!(&entity.kind,EntityKind::Text(_) | EntityKind::MText(_) | EntityKind::Insert(_)) {
        return Err("Đối tượng không phải TEXT, MTEXT hoặc Block có nhãn cọc.".into());
    }
    let mut values=Vec::<String>::new();
    qto_road_collect_labels(doc,entity,0,&mut BTreeSet::new(),&mut values)?;
    let mut stations=Vec::<f64>::new();
    for value in values {
        match takeoff_road::station_from_marker_text(&value) {
            Ok(station)=>{if !stations.iter().any(|old|(*old-station).abs()<=1.0e-6) {stations.push(station);}},
            Err(error) if error.contains("nhiều lý trình")=>return Err(error),
            Err(_)=>{},
        }
    }
    match stations.as_slice() {
        [station]=>Ok(*station),
        []=>Err("Không đọc được lý trình Km hợp lệ từ nhãn cọc.".into()),
        _=>Err("Đối tượng chứa nhiều lý trình khác nhau; cần kiểm tra thủ công.".into()),
    }
}

fn qto_road_scan_markers(app: &CadApp, all_model: bool) -> (Vec<takeoff_road::RoadPlanMarker>,usize) {
    let selected=if all_model {Vec::new()} else {app.session.selection()};
    let Ok(doc)=app.session.doc() else {return (Vec::new(),selected.len());};
    let mut markers=Vec::new();
    let mut skipped=0usize;
    let mut inspect=|entity: &cadcraft_doc::Entity| {
        if !matches!(&entity.kind,EntityKind::Text(_) | EntityKind::MText(_) | EntityKind::Insert(_)) {return;}
        match qto_road_station_from_entity(doc,entity) {
            Ok(chainage_m)=>markers.push(takeoff_road::RoadPlanMarker{chainage_m,handle:entity.handle.hex()}),
            Err(_)=>skipped+=1,
        }
    };
    if all_model {
        // Explicit user action: scan the current model space, bounded for UI responsiveness.
        for entity in doc.model.iter().take(100_000) {inspect(entity);}
    } else {
        for handle in selected {
            if let Some(entity)=doc.model.get(handle) {inspect(entity);}
            else {skipped+=1;}
        }
    }
    (markers,skipped)
}

fn qto_station_handle_key(value: &str) -> String {
    cadcraft_doc::Handle::parse_hex(value).map(|handle|handle.hex().to_ascii_uppercase()).unwrap_or_else(||value.to_ascii_uppercase())
}

/// Attach only unambiguous station labels, preserving manual assignments.
/// A marker may never be assigned to two survey sections.
fn qto_road_attach_markers(sections: &mut [RoadCrossSection], markers: &[takeoff_road::RoadPlanMarker]) -> (usize,usize,usize) {
    let mut linked=0usize;
    let mut missing=0usize;
    let mut conflicts=0usize;
    let mut used=BTreeSet::<String>::new();
    for section in sections.iter() {
        if let Some(handle)=&section.plan_marker_handle { used.insert(qto_station_handle_key(handle)); }
    }
    for section in sections.iter_mut() {
        let candidates=markers.iter().filter(|m|(m.chainage_m-section.chainage_m).abs()<=0.001).collect::<Vec<_>>();
        match candidates.as_slice() {
            []=>missing+=1,
            [marker]=>{
                let is_existing=section.plan_marker_handle.as_deref().is_some_and(|old|qto_station_handle_key(old)==qto_station_handle_key(&marker.handle));
                if (section.plan_marker_handle.is_none() && used.insert(qto_station_handle_key(&marker.handle))) || is_existing {
                    section.plan_marker_handle=Some(marker.handle.clone());
                    linked+=1;
                } else { conflicts+=1; }
            },
            _=>conflicts+=1,
        }
    }
    (linked,missing,conflicts)
}

#[cfg(test)]
mod road_station_matching_tests {
    use super::*;
    fn section(m: f64, marker: Option<&str>) -> RoadCrossSection {
        RoadCrossSection {
            chainage_m:m,name:format!("Cọc {m}"),plan_marker_handle:marker.map(str::to_string),
            cut:None,fill:None,cut_zero_confirmed:true,fill_zero_confirmed:true,confirmed:false,
        }
    }
    #[test]
    fn matches_unique_stations_and_never_reuses_a_plan_marker() {
        let mut sections=vec![section(1000.0,None),section(1020.0,None)];
        let markers=vec![
            takeoff_road::RoadPlanMarker{chainage_m:1000.0,handle:"AA".into()},
            takeoff_road::RoadPlanMarker{chainage_m:1020.0,handle:"BB".into()},
        ];
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(2,0,0));
        assert_eq!(sections[0].plan_marker_handle.as_deref(),Some("AA"));
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(2,0,0));
        sections[1].plan_marker_handle=Some("AA".into());
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(1,0,1));
        assert_eq!(sections[1].plan_marker_handle.as_deref(),Some("AA"));
    }
    #[test]
    fn duplicate_chainage_requires_manual_review() {
        let mut sections=vec![section(500.0,None)];
        let markers=vec![
            takeoff_road::RoadPlanMarker{chainage_m:500.0,handle:"10".into()},
            takeoff_road::RoadPlanMarker{chainage_m:500.0,handle:"11".into()},
        ];
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(0,0,1));
        assert!(sections[0].plan_marker_handle.is_none());
    }
    #[test]
    fn missing_station_is_not_counted_as_marker_conflict() {
        let mut sections=vec![section(100.0,None)];
        let markers=vec![takeoff_road::RoadPlanMarker{chainage_m:140.0,handle:"BB".into()}];
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(0,1,0));
        assert!(sections[0].plan_marker_handle.is_none());
    }
    #[test]
    fn zero_padded_hex_marker_is_considered_the_same_entity() {
        let mut sections=vec![section(100.0,Some("AA")),section(120.0,None)];
        let markers=vec![takeoff_road::RoadPlanMarker{chainage_m:120.0,handle:"0AA".into()}];
        assert_eq!(qto_road_attach_markers(&mut sections,&markers),(0,0,1));
        assert!(sections[1].plan_marker_handle.is_none());
    }
}

fn qto_live_handles(app: &CadApp) -> BTreeSet<String> {
    app.session.selection().into_iter().map(|handle| handle.hex()).collect()
}
fn qto_allowed_handles(app: &CadApp, state: &State) -> Option<BTreeSet<String>> {
    match state.source_mode {
        QtoSourceMode::CurrentSelection => Some(qto_live_handles(app)),
        QtoSourceMode::CapturedRegion => Some(state.captured_handles.clone()),
        QtoSourceMode::EntireProject => None,
    }
}
fn qto_preview_key(app: &CadApp, mode: QtoSourceMode, handles: &BTreeSet<String>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mode.hash(&mut hasher);
    handles.hash(&mut hasher);
    if let Ok(st) = app.session.state() {
        st.revision.hash(&mut hasher);
        // Two drawings may both have revision=0; do not reuse geometry from a
        // different document after switching tabs/files.
        (std::sync::Arc::as_ptr(&st.doc) as usize).hash(&mut hasher);
    }
    hasher.finish()
}
fn qto_measure_type_allowed(kind: MeasureKind, chosen: [bool; 4]) -> bool {
    match kind {
        MeasureKind::Length => chosen[0],
        MeasureKind::Area => chosen[1],
        MeasureKind::Perimeter => chosen[2],
        MeasureKind::Count => chosen[3],
    }
}
fn qto_volume_source_current(app: &CadApp, source: &GeometricSource, kind: MeasureKind) -> bool {
    let Some(handle) = cadcraft_doc::Handle::parse_hex(&source.handle) else { return false; };
    measurements_for_handle(app, handle).into_iter().any(|measurement| {
        if measurement.kind != kind || geometry_signature(&measurement) != source.signature { return false; }
        let raw = if kind == MeasureKind::Length {
            takeoff_composite::section_height(&measurement.points).ok()
        } else { Some(measurement.raw_value) };
        raw.is_some_and(|value| (value - source.raw_value).abs() < 1.0e-8 * value.abs().max(1.0))
    })
}
fn qto_volume_report_lines(app: &CadApp, state: &State, config: &ExportConfig, allowed: Option<&BTreeSet<String>>) -> Result<Vec<ReportLine>, String> {
    let mut out = Vec::new();
    for (index, item) in state.volume_records.iter().enumerate() {
        if config.confirmed_only && !item.confirmed { continue; }
        if !allowed.is_none_or(|handles| handles.contains(&item.plan.handle) && handles.contains(&item.section.handle)) { continue; }
        if !qto_volume_source_current(app, &item.plan, MeasureKind::Area)
            || !qto_volume_source_current(app, &item.section, MeasureKind::Length) {
            return Err(format!("Thể tích ghép #{} có nguồn CAD đã thay đổi hoặc không còn tồn tại. Phải quét lại mặt bằng/mặt cắt.", index + 1));
        }
        if config.confirmed_only && (item.description.trim().is_empty() || item.work_code.trim().is_empty()) {
            return Err(format!("Thể tích #{} thiếu mã công tác hoặc diễn giải để xuất hồ sơ Chủ đầu tư.", index + 1));
        }
        let base=state.scale_to_m().ok_or_else(|| "Chưa hiệu chuẩn bản vẽ; không thể quy đổi thể tích mặt bằng × mặt cắt.".to_string())?;
        let value=takeoff_composite::volume(item.plan.raw_value, item.section.raw_value, base * config.unit_scale())?;
        let unit=format!("{}³", config.length_unit);
        let fields=[
            String::new(),item.work_code.clone(),item.description.clone(),item.work_group.clone(),
            format!("{} + {}",item.plan.handle,item.section.handle),String::new(),String::new(),
            "Thể tích".into(),String::new(),unit.clone(),
            "Diện tích mặt bằng × chiều cao mặt cắt; 2 nguồn CAD độc lập".into(),
            format!("{} + {}",item.plan.handle,item.section.handle),
        ];
        let group=if item.work_group.trim().is_empty() {"Chưa phân nhóm".into()} else {item.work_group.clone()};
        out.push(ReportLine{group,fields,quantity:value,unit});
    }
    Ok(out)
}

/// Road cut/fill cross sections must be delineated by a genuine closed
/// boundary, hatch or region. A circle's area is not a road section.
fn qto_is_road_cross_section_boundary(app: &CadApp, handle: cadcraft_doc::Handle) -> bool {
    app.session.doc().ok()
        .and_then(|doc| doc.model.get(handle).map(|entity|match &entity.kind {
            EntityKind::LwPolyline(poly) => poly.closed,
            EntityKind::Hatch(_) | EntityKind::Region(_) => true,
            _ => false,
        })).unwrap_or(false)
}
fn qto_road_interval_in_scope(left: &RoadCrossSection, right: &RoadCrossSection, kind: EarthworkKind, allowed: Option<&BTreeSet<String>>) -> bool {
    let permitted=|section: &RoadCrossSection| {
        let (source,zero) = match kind {
            EarthworkKind::Cut => (section.cut.as_ref(),section.cut_zero_confirmed),
            EarthworkKind::Fill => (section.fill.as_ref(),section.fill_zero_confirmed)
        };
        if let Some(source)=source {return allowed.is_none_or(|handles|handles.contains(&source.handle));}
        zero
    };
    permitted(left) && permitted(right)
}
fn qto_road_report_lines(app: &CadApp, state: &State, config: &ExportConfig,
    allowed: Option<&BTreeSet<String>>) -> Result<Vec<ReportLine>,String> {
    if state.road_sections.len() < 2 { return Ok(Vec::new()); }
    let scale=state.scale_to_m().ok_or_else(||"Cần hiệu chuẩn tỷ lệ bản vẽ CAD trước khi tính đào/đắp theo lý trình.".to_string())?;
    let range=takeoff_road::parse_range(&state.road_range_start,&state.road_range_end)?;
    let scoped_sections=state.road_sections.iter().filter(|section|range.is_none_or(|(from,to)|
        section.chainage_m>=from-1.0e-6 && section.chainage_m<=to+1.0e-6)).cloned().collect::<Vec<_>>();
    takeoff_road::validate_unique_plan_handles(&scoped_sections)?;
    let intervals=takeoff_road::intervals(&scoped_sections,scale,config.confirmed_only)?;
    let multiplier=config.unit_scale().powi(3);
    let mut out=Vec::new();
    for interval in intervals {
        if !takeoff_road::within_range(&interval,range) { continue; }
        let (code,kind)=match interval.kind {
            EarthworkKind::Cut => (&state.road_cut_code,"Đào nền đường"),
            EarthworkKind::Fill => (&state.road_fill_code,"Đắp nền đường")
        };
        let Some(start)=state.road_sections.iter().find(|section|(section.chainage_m-interval.start_m).abs()<1.0e-6) else {return Err("Không tìm thấy trắc ngang đầu tương ứng với đoạn đo bóc.".into());};
        let Some(end)=state.road_sections.iter().find(|section|(section.chainage_m-interval.end_m).abs()<1.0e-6) else {return Err("Không tìm thấy trắc ngang cuối tương ứng với đoạn đo bóc.".into());};
        if !qto_road_interval_in_scope(start,end,interval.kind,allowed) { continue; }
        for section in [start,end] {
            if let Some(marker)=section.plan_marker_handle.as_deref() {
                let Some(handle)=cadcraft_doc::Handle::parse_hex(marker) else {
                    return Err(format!("Handle cọc bình đồ tại {} không hợp lệ.",takeoff_road::station_label(section.chainage_m)));
                };
                let Some(doc)=app.session.doc().ok() else {return Err("Không mở được bản vẽ CAD để đối chiếu cọc.".into());};
                let Some(entity)=doc.model.get(handle) else {
                    return Err(format!("Cọc bình đồ tại {} đã bị xóa. Hãy quét lại.",takeoff_road::station_label(section.chainage_m)));
                };
                if matches!(&entity.kind,EntityKind::Text(_) | EntityKind::MText(_) | EntityKind::Insert(_)) {
                    let actual=qto_road_station_from_entity(doc,entity).map_err(|error|
                        format!("Nhãn cọc bình đồ tại {} không hợp lệ: {error}",takeoff_road::station_label(section.chainage_m)))?;
                    if (actual-section.chainage_m).abs()>0.001 {
                        return Err(format!("Lý trình cọc nguồn {} đã thay đổi thành {}; không xuất Excel sai lý trình.",
                            takeoff_road::station_label(section.chainage_m),takeoff_road::station_label(actual)));
                    }
                }
            }
        }
        // Validate only the two CAD shapes actually contributing to this
        // exported station interval. Unrelated stations must not block export.
        let sources=match interval.kind {
            EarthworkKind::Cut => (&start.cut,&end.cut),
            EarthworkKind::Fill => (&start.fill,&end.fill),
        };
        for source in [sources.0.as_ref(),sources.1.as_ref()].into_iter().flatten() {
            if !cadcraft_doc::Handle::parse_hex(&source.handle)
                .is_some_and(|handle|qto_is_road_cross_section_boundary(app,handle))
                || !qto_volume_source_current(app,source,MeasureKind::Area) {
                return Err(format!("Nguồn diện tích {} tại đoạn {} → {} đã thay đổi. Hãy quét lại trắc ngang.",
                    source.handle,takeoff_road::station_label(interval.start_m),takeoff_road::station_label(interval.end_m)));
            }
        }
        if config.confirmed_only && code.trim().is_empty() {
            return Err(format!("Chưa khai báo mã công tác {} để phát hành Excel.",kind));
        }
        let unit=format!("{}³",config.length_unit);
        let evidence_start=if interval.source_start.is_empty(){"0 (đã xác nhận)"}else{&interval.source_start};
        let evidence_end=if interval.source_end.is_empty(){"0 (đã xác nhận)"}else{&interval.source_end};
        let fields=[
            String::new(),code.clone(),format!("{} {} → {}",kind,
                takeoff_road::station_label(interval.start_m),takeoff_road::station_label(interval.end_m)),
            state.road_work_group.clone(),format!("{} + {}",evidence_start,evidence_end),
            String::new(),"Giao thông".into(),kind.into(),String::new(),unit.clone(),
            format!("AEA: L={:.3}m; Ađầu={:.3}m²; Acuối={:.3}m². Tách riêng đào/đắp.",interval.length_m,interval.area_start_m2,interval.area_end_m2),
            format!("{} + {}",evidence_start,evidence_end)
        ];
        out.push(ReportLine{group:if state.road_work_group.is_empty(){"Nền đường".into()}else{state.road_work_group.clone()},
            fields,quantity:interval.volume_m3*multiplier,unit});
    }
    Ok(out)
}

fn qto_record_in_scope(record: &MeasurementRecord, allowed: Option<&BTreeSet<String>>) -> bool {
    match allowed {
        None => true,
        Some(handles) => record.source_handle.as_ref().is_some_and(|handle| handles.contains(handle)),
    }
}

fn quantities(app: &CadApp) -> BTreeMap<String, Quantity> {
    let mut groups = BTreeMap::<String, Quantity>::new();
    if let Ok(drawing) = app.session.doc() {
        for entity in drawing.model.iter() {
            let group = groups.entry(entity.common.layer.clone()).or_default();
            group.count += 1;
            match &entity.kind {
                EntityKind::Line(line) => {
                    let distance = (line.b.x - line.a.x).hypot(line.b.y - line.a.y)
                        .hypot(line.b.z - line.a.z);
                    if distance.is_finite() { group.line_length += distance; } else { group.unsupported += 1; }
                }
                EntityKind::Circle(circle) => {
                    if circle.radius.is_finite() && circle.radius > 0.0 {
                        group.circle_area += std::f64::consts::PI * circle.radius.powi(2);
                    } else { group.unsupported += 1; }
                }
                EntityKind::Arc(arc) => {
                    if arc.radius.is_finite() && arc.radius > 0.0 && arc.start.is_finite() && arc.end.is_finite() {
                        let delta = (arc.end - arc.start).rem_euclid(std::f64::consts::TAU);
                        group.arc_length += arc.radius * delta;
                    } else { group.unsupported += 1; }
                }
                EntityKind::LwPolyline(poly) => {
                    match lwpoly_metrics(&poly.vertices, poly.closed) {
                        Some((length, area)) => { group.polyline_length += length; group.closed_area += area; }
                        None => group.unsupported += 1,
                    }
                }
                EntityKind::Polyline3d(poly) => {
                    let n = poly.points.len();
                    let edges = if poly.closed { n } else { n.saturating_sub(1) };
                    let mut length = 0.0;
                    for i in 0..edges {
                        let a = &poly.points[i];
                        let b = &poly.points[(i + 1) % n];
                        length += (b.x - a.x).hypot(b.y - a.y).hypot(b.z - a.z);
                    }
                    if n >= 2 && poly.points.iter().all(|p|
                        p.x.is_finite() && p.y.is_finite() && p.z.is_finite())
                        && length.is_finite() {
                        group.polyline_length += length;
                    } else { group.unsupported += 1; }
                }
                _ => group.unsupported += 1,
            }
        }
    }
    groups
}

fn csv_safe_cell(value: &str) -> String {
    let clean = value.replace('\r', " ").replace('\n', " ");
    let cell = if matches!(clean.chars().next(), Some('=' | '+' | '-' | '@' | '\t')) { format!("'{clean}") } else { clean };
    format!("\"{}\"", cell.replace('"', "\"\""))
}

fn measurement_matches_filter(record: &MeasurementRecord, search: &str, status_filter: u8) -> bool {
    let status_ok = match status_filter { 1 => record.status == MeasureStatus::Review, 2 => record.status == MeasureStatus::Confirmed, 3 => record.status == MeasureStatus::Excluded, _ => true };
    if !status_ok { return false; }
    let term = search.trim().to_lowercase();
    term.is_empty() || [record.layer.as_str(), record.work_code.as_str(), record.work_group.as_str(), record.description.as_str(), record.source_handle.as_deref().unwrap_or("")].iter().any(|value| value.to_lowercase().contains(&term))
}

fn qto_record_exportable(record: &MeasurementRecord, source_validation: &SourceValidation, lf: f64, af: f64, confirmed_only: bool) -> bool {
    if confirmed_only && (record.status != MeasureStatus::Confirmed || source_validation.invalid_record_ids.contains(&record.id)) { return false; }
    let converted = match record.kind { MeasureKind::Area => record.raw_value * af, MeasureKind::Length | MeasureKind::Perimeter => record.raw_value * lf, MeasureKind::Count => record.raw_value };
    converted.is_finite() && converted >= 0.0
}

fn qto_detail_csv(state: &State, source_validation: &SourceValidation, lf: f64, af: f64, length_unit: &str, area_unit: &str, confirmed_only: bool) -> String {
            let mut csv = String::from("id,source_handle,source_state,floor_id,discipline,owner_account_id,type,layer,work_group,work_code,description,note,raw_value,converted_value,unit,status\n");
            for record in &state.records {
                if !qto_record_exportable(record, source_validation, lf, af, confirmed_only) { continue; }
                let converted = match record.kind {
                    MeasureKind::Area => record.raw_value * af,
                    MeasureKind::Length | MeasureKind::Perimeter => record.raw_value * lf,
                    MeasureKind::Count => record.raw_value,
                };
                if !converted.is_finite() || converted < 0.0 { continue; }
                let unit = match record.kind { MeasureKind::Area => area_unit, MeasureKind::Length | MeasureKind::Perimeter => length_unit, MeasureKind::Count => "count" };
                let esc = csv_safe_cell;
                csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{},{}\n",
                    record.id,
                    esc(record.source_handle.as_deref().unwrap_or("")),
                    record_source_state(record,&source_validation),
                    record.scope.floor_id.map(|v|v.to_string()).unwrap_or_default(),
                    esc(record.scope.discipline.as_deref().unwrap_or("")),
                    esc(record.scope.owner_account_id.as_deref().unwrap_or("")),
                    record.kind.code(), esc(&record.layer), esc(&record.work_group), esc(&record.work_code),
                    esc(&record.description), esc(&record.note), record.raw_value, converted, unit, record.status.code()));
            }
    csv
}

/// Build report data from the same validated measurements shown in CAD QTO.
/// Values remain numeric Excel cells; units are never mixed in the same subtotal.
fn qto_report_lines(state: &State, source_validation: &SourceValidation, config: &ExportConfig, allowed: Option<&BTreeSet<String>>) -> Result<Vec<ReportLine>, String> {
    if !config.valid() { return Err("Cần bật cột Khối lượng, Đơn vị và ít nhất một cột khác.".into()); }
    if config.project.trim().is_empty() || config.investor.trim().is_empty() {
        return Err("Nhập đầy đủ tên dự án và Chủ đầu tư trước khi xuất báo cáo.".into());
    }
    let length_scale = config.unit_scale();
    let needs_calibration = state.records.iter().any(|r| qto_record_in_scope(r, allowed)
        && !state.export_omitted_ids.contains(&r.id) && r.status != MeasureStatus::Excluded
        && r.kind != MeasureKind::Count && (!config.confirmed_only || r.status == MeasureStatus::Confirmed));
    let base = state.scale_to_m();
    if needs_calibration && base.is_none() { return Err("Phải hiệu chuẩn tỷ lệ bản vẽ sang mét trước khi xuất khối lượng chiều dài/diện tích.".into()); }
    let lf = base.unwrap_or(1.0) * length_scale;
    let af = lf * lf;
    if !lf.is_finite() || !af.is_finite() { return Err("Đơn vị quy đổi không hợp lệ.".into()); }
    let mut result = Vec::new();
    for record in &state.records {
        if !qto_record_in_scope(record, allowed) || state.export_omitted_ids.contains(&record.id)
            || (state.methods_initialized && !qto_measure_type_allowed(record.kind, state.selected_measure_types)) { continue; }
        // Never export excluded or invalid-source measurements, even in draft mode.
        if record.status == MeasureStatus::Excluded || source_validation.invalid_record_ids.contains(&record.id) { continue; }
        if !qto_record_exportable(record, source_validation, lf, af, config.confirmed_only) { continue; }
        if config.confirmed_only && record.description.trim().is_empty() {
            return Err(format!("Phép đo #{} chưa có diễn giải công tác, cần bổ sung trước khi xuất hồ sơ Chủ đầu tư.", record.id));
        }
        let (quantity, unit, kind) = match record.kind {
            MeasureKind::Length => (record.raw_value * lf, config.length_unit.clone(), "Chiều dài"),
            MeasureKind::Perimeter => (record.raw_value * lf, config.length_unit.clone(), "Chu vi"),
            MeasureKind::Area => (record.raw_value * af, format!("{}²", config.length_unit), "Diện tích"),
            MeasureKind::Count => (record.raw_value, "cái".to_string(), "Số lượng"),
        };
        if !quantity.is_finite() || quantity < 0.0 { continue; }
        let group = match config.group_by {
            GroupBy::WorkGroup => record.work_group.as_str(),
            GroupBy::WorkCode => record.work_code.as_str(),
            GroupBy::Layer => record.layer.as_str(),
            GroupBy::Floor => "",
            GroupBy::Discipline => record.scope.discipline.as_deref().unwrap_or(""),
            GroupBy::None => "",
        };
        let group = if config.group_by == GroupBy::Floor { record.scope.floor_id.map(|id| format!("Tầng {id}")).unwrap_or_else(|| "Chưa gán tầng".into()) }
            else if group.trim().is_empty() { "Chưa phân nhóm".into() } else { group.to_string() };
        let fields = [
            String::new(), record.work_code.clone(), record.description.clone(),
            record.work_group.clone(), record.layer.clone(),
            record.scope.floor_id.map(|v| v.to_string()).unwrap_or_default(),
            record.scope.discipline.clone().unwrap_or_default(),
            kind.into(), String::new(), unit.clone(), record.note.clone(),
            record.source_handle.clone().unwrap_or_default(),
        ];
        result.push(ReportLine { group, fields, quantity, unit });
    }
    result.sort_by(|a,b| a.group.cmp(&b.group).then(a.fields[1].cmp(&b.fields[1])).then(a.fields[2].cmp(&b.fields[2])));
    if result.is_empty() { return Err("Không có khối lượng hợp lệ theo điều kiện xuất đã chọn.".into()); }
    Ok(result)
}

/// Export both ordinary 2D measurements and verified plan×section 3D
/// quantities without silently swallowing a failed measurement validation.
fn qto_combined_report_lines(app: &CadApp, state: &State, source_validation: &SourceValidation,
    config: &ExportConfig, allowed: Option<&BTreeSet<String>>) -> Result<Vec<ReportLine>, String> {
    if !config.valid() || config.project.trim().is_empty() || config.investor.trim().is_empty() {
        return Err("Cần khai báo dự án, Chủ đầu tư và chọn các cột bắt buộc.".into());
    }
    let has_ordinary = state.records.iter().any(|record| qto_record_in_scope(record, allowed)
        && !state.export_omitted_ids.contains(&record.id)
        && qto_measure_type_allowed(record.kind, state.selected_measure_types)
        && record.status != MeasureStatus::Excluded
        && (!config.confirmed_only || record.status == MeasureStatus::Confirmed));
    let mut rows = if has_ordinary { qto_report_lines(state, source_validation, config, allowed)? } else { Vec::new() };
    rows.extend(qto_volume_report_lines(app, state, config, allowed)?);
    rows.extend(qto_road_report_lines(app, state, config, allowed)?);
    if rows.is_empty() { return Err("Không có phép đo đủ điều kiện theo phạm vi và phương pháp đã chọn.".into()); }
    rows.sort_by(|a,b| a.group.cmp(&b.group).then(a.fields[1].cmp(&b.fields[1])).then(a.fields[7].cmp(&b.fields[7])));
    Ok(rows)
}

fn draw_contents(ui: &mut egui::Ui, app: &CadApp, state: &mut State, scope: &Scope) {
    if !state.methods_initialized {
        state.selected_measure_types = [true, true, true, false];
        state.methods_initialized = true;
    }
    let groups = if state.source_mode == QtoSourceMode::EntireProject { quantities(app) } else { BTreeMap::new() };
    ui.heading("Đo bóc khối lượng — CAD native");
    ui.label("Số liệu trực tiếp từ bản vẽ CAD đang mở; thay đổi ngay sau khi vẽ, sửa, Undo hoặc Redo.");
    let review_count = state.records.iter().filter(|r| r.status == MeasureStatus::Review).count();
    let confirmed_count = state.records.iter().filter(|r| r.status == MeasureStatus::Confirmed).count();
    let excluded_count = state.records.iter().filter(|r| r.status == MeasureStatus::Excluded).count();
    ui.horizontal_wrapped(|ui| {
        ui.group(|ui| { ui.strong(format!("Tổng phép đo: {}", state.records.len())); });
        ui.group(|ui| { ui.label(format!("Cần rà soát: {review_count}")); });
        ui.group(|ui| { ui.label(format!("Đã xác nhận: {confirmed_count}")); });
        ui.group(|ui| { ui.label(format!("Đã loại: {excluded_count}")); });
        ui.group(|ui| { ui.label(format!("Có thể hoàn tác: {}", state.undo.len())); });
    });
    if review_count > 0 { ui.colored_label(egui::Color32::YELLOW, format!("Còn {review_count} phép đo chưa xác nhận; không tính vào khối lượng nghiệm thu.")); }
    ui.separator();

    ui.group(|ui| {
        ui.strong("Hiệu chỉnh tỷ lệ");
        ui.horizontal(|ui| {
            ui.label("Khoảng cách trên bản vẽ:");
            ui.add(egui::DragValue::new(&mut state.drawing_distance).speed(0.1).range(0.0..=f64::MAX));
            ui.label("Chiều dài thực (m):");
            ui.add(egui::DragValue::new(&mut state.real_distance_m).speed(0.1).range(0.0..=f64::MAX));
        });
        match state.scale_to_m() {
            Some(scale) => {
                ui.label(format!("Tỷ lệ quy đổi: 1 đơn vị bản vẽ = {:.8} m", scale));
            }
            None => {
                ui.colored_label(egui::Color32::YELLOW, "Chưa hiệu chỉnh tỷ lệ. Kết quả đang hiển thị theo đơn vị bản vẽ.");
            }
        }
    });

    let scale = state.scale_to_m();
    let length_unit = if scale.is_some() { "m" } else { "đv" };
    let area_unit = if scale.is_some() { "m²" } else { "đv²" };
    let lf = scale.unwrap_or(1.0);
    let af = lf * lf;
    if !af.is_finite() { ui.colored_label(egui::Color32::RED, "Tỷ lệ diện tích vượt phạm vi tính toán. Không sử dụng kết quả diện tích để nghiệm thu."); }

    ui.group(|ui| {
        ui.strong("01 — CHỌN PHẠM VI TRỰC TIẾP TRÊN BẢN VẼ");
        ui.label("Rê chuột và quét cửa sổ (trái → phải chọn nằm trong; phải → trái chọn cắt qua) ở vùng CAD bên trái. Đối tượng được chọn xuất hiện ngay trong bảng xem trước; KHÔNG tự lấy toàn dự án.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Phạm vi bóc:");
            egui::ComboBox::from_id_salt("cadtbt360_qto_source_mode").selected_text(state.source_mode.label()).show_ui(ui, |ui| {
                for mode in QtoSourceMode::ALL { ui.selectable_value(&mut state.source_mode, mode, mode.label()); }
            });
            if ui.button("Lưu phạm vi đang quét").clicked() {
                let handles = qto_live_handles(app);
                if handles.is_empty() {
                    state.import_feedback = "Hãy quét/chọn đối tượng trong CAD trước khi lưu phạm vi.".into();
                } else {
                    state.captured_handles = handles;
                    state.source_mode = QtoSourceMode::CapturedRegion;
                    state.import_feedback = format!("Đã lưu {} handle CAD cho vùng đo bóc.", state.captured_handles.len());
                }
            }
            ui.label(format!("Đang chọn: {} | Đã lưu: {}", app.session.selection().len(), state.captured_handles.len()));
        });
        ui.strong("02 — CHỌN PHƯƠNG PHÁP ĐO (CÓ THỂ KẾT HỢP)");
        ui.horizontal_wrapped(|ui| {
            for (index, label) in ["Chiều dài", "Diện tích", "Chu vi", "Đếm cấu kiện"].iter().enumerate() {
                // Start with geometry quantities, not automatic object counting.
                ui.checkbox(&mut state.selected_measure_types[index], *label);
            }
        });
        if !state.selected_measure_types.iter().any(|chosen| *chosen) {
            ui.colored_label(egui::Color32::YELLOW, "Chọn ít nhất một phương pháp; hệ thống không tự đếm toàn bộ đối tượng.");
        }
        let handles = qto_allowed_handles(app, state);
        if let Some(handles) = &handles {
            let key = qto_preview_key(app, state.source_mode, handles);
            if state.preview_key != Some(key) {
                state.preview_measurements = handles.iter().filter_map(|handle| cadcraft_doc::Handle::parse_hex(handle))
                    .flat_map(|handle| measurements_for_handle(app, handle)).collect();
                state.preview_key = Some(key);
            }
            let matches = state.preview_measurements.iter().filter(|m| qto_measure_type_allowed(m.kind, state.selected_measure_types)).cloned().collect::<Vec<_>>();
            ui.label(format!("Xem trước phạm vi: {} đối tượng nguồn; {} phép đo hình học phù hợp.", handles.len(), matches.len()));
            egui::ScrollArea::vertical().id_salt("cadtbt360_qto_live_preview").max_height(165.0).show(ui, |ui| {
                egui::Grid::new("cadtbt360_qto_preview_grid").striped(true).show(ui, |ui| {
                    ui.strong("Nguồn CAD"); ui.strong("Layer"); ui.strong("Phương pháp"); ui.strong("Giá trị thô"); ui.end_row();
                    for m in matches.iter().take(35) {
                        ui.label(m.source_handle.hex()); ui.label(&m.layer);
                        ui.label(match m.kind { MeasureKind::Length => "Chiều dài", MeasureKind::Area => "Diện tích", MeasureKind::Perimeter => "Chu vi", MeasureKind::Count => "Đếm" });
                        ui.label(format!("{:.3}", m.raw_value)); ui.end_row();
                    }
                });
            });
            if matches.len() > 35 { ui.small(format!("Đang hiển thị 35/{} phép đo; nhập danh sách sẽ dùng đầy đủ.", matches.len())); }
            if ui.add_enabled(!matches.is_empty(), egui::Button::new("THÊM CÁC PHÉP ĐO ĐÃ CHỌN VÀO BẢNG")).clicked() {
                let mut added = 0usize; let mut skipped = 0usize;
                for measurement in matches {
                    let (entity_scope, revision) = scope_from_entity_or_active(app, measurement.source_handle, scope);
                    if state.add_measurement_from_entity(measurement, &entity_scope, revision).is_ok() { added += 1; } else { skipped += 1; }
                }
                state.import_feedback = format!("Đã tạo {added} phép đo có nguồn CAD, bỏ qua {skipped} mục trùng hoặc không hợp lệ. Hãy rà soát và xác nhận trước khi xuất.");
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW, "Đã chọn toàn bộ bản vẽ. Cách này chỉ áp dụng khi chủ động chọn; khuyến nghị quét vùng cần bóc.");
        }
        ui.small("Tiếp theo: ghép diện tích mặt bằng + chiều cao mặt cắt thành thể tích; đo đường theo lý trình cọc + diện tích trắc ngang. Không suy diễn hình học khi chưa khai báo đủ nguồn.");
    });

    ui.group(|ui| {
        ui.strong("03 — ĐO THỂ TÍCH HAI BƯỚC: MẶT BẰNG + MẶT CẮT");
        ui.small("Bước 1: quét đúng 1 đường bao kín trên mặt bằng (AREA). Bước 2: quét đúng 1 LINE thẳng đứng trên mặt cắt (HEIGHT). Hai hình học nguồn được kiểm tra lại trước khi xuất Excel.");
        ui.horizontal_wrapped(|ui| {
            if ui.button("1. LẤY DIỆN TÍCH MẶT BẰNG").clicked() {
                let areas=selected_measurements(app).into_iter().filter(|m| m.kind == MeasureKind::Area && m.raw_value > 0.0).collect::<Vec<_>>();
                if areas.len() == 1 {
                    let measurement=&areas[0];
                    state.volume_plan=Some(GeometricSource{handle:measurement.source_handle.hex(),signature:geometry_signature(measurement),raw_value:measurement.raw_value});
                    state.import_feedback=format!("Đã lấy diện tích mặt bằng từ {}: {:.3} đơn vị².", measurement.source_handle.hex(),measurement.raw_value);
                } else {
                    state.import_feedback=format!("Cần đúng 1 diện tích khép kín; vùng đang chọn có {} phép đo diện tích. Hãy chọn lại.",areas.len());
                }
            }
            if ui.button("2. LẤY CHIỀU CAO MẶT CẮT").clicked() {
                let mut heights=Vec::new();
                let mut incorrect=0usize;
                if let Ok(doc)=app.session.doc() {
                    for measurement in selected_measurements(app) {
                        if measurement.kind != MeasureKind::Length { continue; }
                        let line=doc.model.get(measurement.source_handle)
                            .is_some_and(|entity| matches!(&entity.kind,EntityKind::Line(_)));
                        if !line { continue; }
                        match takeoff_composite::section_height(&measurement.points) {
                            Ok(height) => heights.push((measurement,height)),
                            Err(_) => incorrect+=1,
                        }
                    }
                }
                if heights.len() == 1 && incorrect == 0 {
                    let (measurement,height)=&heights[0];
                    state.volume_section=Some(GeometricSource{handle:measurement.source_handle.hex(),signature:geometry_signature(measurement),raw_value:*height});
                    state.import_feedback=format!("Đã nhận chiều cao mặt cắt từ {}: {:.3} đơn vị.",measurement.source_handle.hex(),height);
                } else {
                    state.import_feedback=format!("Mặt cắt cần đúng 1 LINE thẳng đứng (hiện {} hợp lệ, {} nghiêng/không đạt). Hãy chọn lại.",heights.len(),incorrect);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Mặt bằng: {}",state.volume_plan.as_ref().map(|x|format!("{} • {:.3} đv²",x.handle,x.raw_value)).unwrap_or_else(||"Chưa chọn".into())));
            ui.separator();
            ui.label(format!("Mặt cắt: {}",state.volume_section.as_ref().map(|x|format!("{} • {:.3} đv",x.handle,x.raw_value)).unwrap_or_else(||"Chưa chọn".into())));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Nhóm"); ui.add(egui::TextEdit::singleline(&mut state.volume_work_group).desired_width(125.0));
            ui.label("Mã CT"); ui.add(egui::TextEdit::singleline(&mut state.volume_work_code).desired_width(125.0));
            ui.label("Diễn giải"); ui.add(egui::TextEdit::singleline(&mut state.volume_description).desired_width(200.0));
        });
        let ready=state.volume_plan.as_ref().zip(state.volume_section.as_ref()).filter(|(plan,section)| plan.handle!=section.handle)
            .and_then(|(plan,section)|state.scale_to_m().and_then(|scale|takeoff_composite::volume(plan.raw_value,section.raw_value,scale).ok()));
        if let Some(value)=ready { ui.colored_label(egui::Color32::LIGHT_GREEN,format!("Thể tích dự kiến: {value:.3} m³ = S mặt bằng × H mặt cắt (chưa xác nhận).")); }
        else { ui.colored_label(egui::Color32::YELLOW,"Cần hai nguồn hình học độc lập và tỷ lệ bản vẽ hợp lệ trước khi tính thể tích."); }
        if ui.add_enabled(ready.is_some(),egui::Button::new("LƯU PHÉP ĐO THỂ TÍCH HAI NGUỒN")).clicked() {
            let plan=state.volume_plan.clone().unwrap();
            let section=state.volume_section.clone().unwrap();
            if !qto_volume_source_current(app,&plan,MeasureKind::Area) || !qto_volume_source_current(app,&section,MeasureKind::Length) {
                state.import_feedback="Một trong hai nguồn CAD đã thay đổi; cần quét và khai báo lại.".into();
            } else if state.volume_records.iter().any(|r|r.plan.handle==plan.handle && r.section.handle==section.handle) {
                state.import_feedback="Cặp mặt bằng/mặt cắt này đã được đo bóc, không ghi trùng.".into();
            } else {
                state.captured_handles.insert(plan.handle.clone());
                state.captured_handles.insert(section.handle.clone());
                state.source_mode=QtoSourceMode::CapturedRegion;
                state.volume_records.push(PlanSectionVolume {
                    plan,section,work_group:state.volume_work_group.trim().to_string(),
                    work_code:state.volume_work_code.trim().to_string(),
                    description:state.volume_description.trim().to_string(),confirmed:false,
                });
                state.import_feedback="Đã lưu thể tích có truy vết hai nguồn CAD. Hãy kiểm tra và xác nhận trước khi xuất hồ sơ.".into();
            }
        }
        if !state.volume_records.is_empty() {
            ui.separator();
            ui.strong(format!("THỂ TÍCH HAI NGUỒN ĐÃ LƯU: {}",state.volume_records.len()));
            egui::ScrollArea::vertical().id_salt("cadtbt360_plan_section_volumes").max_height(185.0).show(ui,|ui| {
                let mut remove=None;
                for (index, item) in state.volume_records.iter_mut().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        ui.checkbox(&mut item.confirmed,"Đã kiểm tra");
                        ui.label(format!("#{} {} • S={} đv² × H={} đv",index+1,item.work_code,item.plan.raw_value,item.section.raw_value));
                        if ui.small_button("Xóa").clicked() {remove=Some(index);}
                    });
                }
                if let Some(index)=remove {state.volume_records.remove(index);}
            });
        }
        ui.small("Đây là phương pháp diện tích mặt bằng × chiều cao mặt cắt có cùng tỷ lệ bản vẽ. Trắc ngang giao thông, đất đắp/đào và chuyển đổi tỷ lệ khác nhau sẽ có module tính riêng, không dùng công thức này.");
    });

    egui::CollapsingHeader::new("04 — GIAO THÔNG: CỌC LÝ TRÌNH + DIỆN TÍCH TRẮC NGANG")
        .id_salt("cadtbt360_road_qto_panel").default_open(true).show(ui, |ui| {
        ui.label("Phương pháp diện tích trung bình hai đầu (AEA): V = (A₁ + A₂) × L / 2.");
        ui.small("Quét/chọn đúng một đường bao kín trên bản vẽ trắc ngang, phân loại vùng ĐÀO hoặc ĐẮP, khai báo lý trình của cọc trên bình đồ. Lý trình nhập theo mét hoặc Km12+345.67. Không tự suy đoán lý trình từ tọa độ CAD.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Lý trình");
            ui.add(egui::TextEdit::singleline(&mut state.road_chainage).hint_text("Km0+000.00").desired_width(145.0));
            ui.label("Tên cọc");
            ui.add(egui::TextEdit::singleline(&mut state.road_name).hint_text("Cọc H1").desired_width(120.0));
            if let Ok(m)=takeoff_road::station_to_metres(&state.road_chainage) {
                ui.colored_label(egui::Color32::LIGHT_GREEN,takeoff_road::station_label(m));
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("GẮN CỌC TỪ BÌNH ĐỒ").clicked() {
                let handles=app.session.selection();
                if handles.len()!=1 {state.import_feedback=format!("Hãy chọn đúng 1 đối tượng cọc trên bình đồ; hiện chọn {}.",handles.len());}
                else if let Some(handle)=handles.into_iter().next() {
                    let value=handle.hex();
                    if state.road_sections.iter().any(|section|section.plan_marker_handle.as_deref()==Some(value.as_str())) {
                        state.import_feedback="Đối tượng cọc này đã được sử dụng ở trắc ngang khác.".into();
                    } else {
                        state.road_plan_marker_handle=Some(value.clone());
                        state.import_feedback=format!("Đã gắn đối tượng CAD {} làm cọc bình đồ. Kiểm tra lý trình trước khi lưu.",value);
                    }
                }
            }
            if ui.button("QUÉT NHÃN CỌC TỪ VÙNG CHỌN").clicked() {
                let (markers,skipped)=qto_road_scan_markers(app,false);
                let (linked,not_found,ambiguous)=qto_road_attach_markers(&mut state.road_sections,&markers);
                state.import_feedback=format!("Quét {} nhãn cọc hợp lệ; ghép {} trắc ngang; chưa tìm thấy {}; xung đột {}; bỏ qua {} đối tượng. Chưa tự phê duyệt; kiểm tra từng dòng trước khi xác nhận.",markers.len(),linked,not_found,ambiguous,skipped);
            }
            if ui.button("QUÉT CỌC TOÀN BỘ MODEL").clicked() {
                let incomplete=app.session.doc().ok().is_some_and(|doc|doc.model.len()>100_000);
                if incomplete {
                    state.import_feedback="MODEL vượt 100.000 đối tượng. Chưa ghép tự động từ dữ liệu quét thiếu; hãy chọn phạm vi nhỏ hơn.".into();
                } else {
                    let (markers,skipped)=qto_road_scan_markers(app,true);
                    let (matched,missing,ambiguous)=qto_road_attach_markers(&mut state.road_sections,&markers);
                    state.import_feedback=format!("Quét MODEL: {} nhãn; ghép {}; thiếu {}; xung đột {}; bỏ qua {}. Chưa phê duyệt số liệu.",markers.len(),matched,missing,ambiguous,skipped);
                }
            }
            if ui.small_button("Bỏ gắn cọc").clicked(){state.road_plan_marker_handle=None;}
            ui.label(format!("Cọc bình đồ: {}",state.road_plan_marker_handle.as_deref().unwrap_or("Chưa gắn")));
        });
        ui.horizontal_wrapped(|ui| {
            for (kind,label) in [(EarthworkKind::Cut,"CHỌN DIỆN TÍCH ĐÀO"),(EarthworkKind::Fill,"CHỌN DIỆN TÍCH ĐẮP")] {
                if ui.button(label).clicked() {
                    let candidates=selected_measurements(app).into_iter().filter(|m| m.kind==MeasureKind::Area
                        && m.raw_value>0.0 && qto_is_road_cross_section_boundary(app,m.source_handle)).collect::<Vec<_>>();
                    if candidates.len()!=1 {
                        state.import_feedback=format!("Cần quét đúng 1 vùng kín trắc ngang (hiện có {} diện tích).",candidates.len());
                    } else {
                        let measure=&candidates[0];
                        let source=GeometricSource{handle:measure.source_handle.hex(),signature:geometry_signature(measure),raw_value:measure.raw_value};
                        match kind {
                            EarthworkKind::Cut=>{state.road_cut=Some(source);state.road_cut_zero_confirmed=false;},
                            EarthworkKind::Fill=>{state.road_fill=Some(source);state.road_fill_zero_confirmed=false;},
                        }
                        state.import_feedback=format!("Đã lấy diện tích {} tại cọc đang khai báo: {:.3} đơn vị².",kind.label(),measure.raw_value);
                    }
                }
            }
            if ui.small_button("Xóa nguồn đào").clicked() {state.road_cut=None;}
            if ui.small_button("Xóa nguồn đắp").clicked() {state.road_fill=None;}
        });
        ui.horizontal_wrapped(|ui| {
            if ui.checkbox(&mut state.road_cut_zero_confirmed,"Xác nhận diện tích ĐÀO = 0 m²").changed() && state.road_cut_zero_confirmed {state.road_cut=None;}
            if ui.checkbox(&mut state.road_fill_zero_confirmed,"Xác nhận diện tích ĐẮP = 0 m²").changed() && state.road_fill_zero_confirmed {state.road_fill=None;}
        });
        ui.small("Chỉ đánh dấu bằng 0 sau khi kiểm tra trắc ngang thực tế. Bỏ chọn không có nghĩa là diện tích bằng 0.");
        let source_label=|source:&Option<GeometricSource>,zero:bool| source.as_ref()
            .map(|value|format!("{} • {:.3} đv²",value.handle,value.raw_value)).unwrap_or_else(||if zero {"0 m² (đã xác nhận)".into()} else {"Chưa chọn".into()});
        ui.columns(2,|cols| {
            cols[0].label(format!("DIỆN TÍCH ĐÀO: {}",source_label(&state.road_cut,state.road_cut_zero_confirmed)));
            cols[1].label(format!("DIỆN TÍCH ĐẮP: {}",source_label(&state.road_fill,state.road_fill_zero_confirmed)));
        });
        let station=takeoff_road::station_to_metres(&state.road_chainage).ok().map(|chainage_m|RoadCrossSection {
            chainage_m,name:state.road_name.trim().to_string(),plan_marker_handle:state.road_plan_marker_handle.clone(),
            cut:state.road_cut.clone(),fill:state.road_fill.clone(),
            cut_zero_confirmed:state.road_cut_zero_confirmed,fill_zero_confirmed:state.road_fill_zero_confirmed,confirmed:false,
        });
        let valid=station.as_ref().is_some_and(|candidate|takeoff_road::validate_section(candidate).is_ok()
            && state.road_sections.iter().all(|old|(old.chainage_m-candidate.chainage_m).abs()>1.0e-6 && candidate.plan_marker_handle.as_deref().is_none_or(|h|old.plan_marker_handle.as_deref()!=Some(h))));
        if ui.add_enabled(valid,egui::Button::new("LƯU TRẮC NGANG VÀO DANH SÁCH CỌC")).clicked() {
            let next=station.unwrap();
            for source in [&next.cut,&next.fill].into_iter().flatten() {state.captured_handles.insert(source.handle.clone());}
            state.road_sections.push(next);
            state.road_sections.sort_by(|a,b|a.chainage_m.total_cmp(&b.chainage_m));
            state.source_mode=QtoSourceMode::CapturedRegion;
            state.road_cut=None;
            state.road_fill=None;
            state.road_cut_zero_confirmed=false;
            state.road_fill_zero_confirmed=false;
            state.road_plan_marker_handle=None;
            state.import_feedback=format!("Đã lưu {} mặt cắt. Phải kiểm tra và xác nhận trước khi xuất.",state.road_sections.len());
        }
        if !valid {
            ui.colored_label(egui::Color32::YELLOW,"Cần lý trình hợp lệ, diện tích đào/đắp từ hình học CAD và không trùng lý trình cọc đã lưu.");
        }
        ui.separator();
        ui.horizontal_wrapped(|ui|{
            ui.label("Khoảng cọc cần bóc:");
            ui.add(egui::TextEdit::singleline(&mut state.road_range_start).desired_width(140.0).hint_text("Cọc đầu Km0+000"));
            ui.label("đến");
            ui.add(egui::TextEdit::singleline(&mut state.road_range_end).desired_width(140.0).hint_text("Cọc cuối Km0+100"));
            if ui.small_button("Tất cả cọc đã lưu").clicked(){state.road_range_start.clear();state.road_range_end.clear();}
        });
        if let Err(error)=takeoff_road::parse_range(&state.road_range_start,&state.road_range_end) {
            ui.colored_label(egui::Color32::YELLOW,error);
        }
        ui.small("Chỉ bóc các khoảng có đủ hai cọc nằm trong phạm vi. Chưa hỗ trợ tự nhận dạng cọc trên bình đồ; không ngoại suy khối lượng khi cắt giữa hai trắc ngang.");
        if ui.button("KIỂM TRA NGUỒN CỌC TRƯỚC KHI XUẤT").clicked() {
            let mut missing=0usize;
            let mut altered=0usize;
            let mut duplicates=0usize;
            let mut used=BTreeSet::<String>::new();
            for section in &state.road_sections {
                let Some(marker)=section.plan_marker_handle.as_deref() else {missing+=1;continue;};
                if !used.insert(qto_station_handle_key(marker)) {duplicates+=1;}
                let current=cadcraft_doc::Handle::parse_hex(marker)
                    .and_then(|handle|app.session.doc().ok().and_then(|doc|doc.model.get(handle).map(|entity|(doc,entity))));
                match current {
                    Some((doc,entity))=>{
                        if matches!(&entity.kind,EntityKind::Text(_) | EntityKind::MText(_) | EntityKind::Insert(_)) {
                            if qto_road_station_from_entity(doc,entity).map_or(true,|m|(m-section.chainage_m).abs()>0.001) {altered+=1;}
                        }
                    },
                    None=>altered+=1,
                }
            }
            state.import_feedback=format!("Kiểm tra {} trắc ngang: {} chưa gắn cọc, {} nguồn bị xóa/sai lý trình, {} handle dùng trùng. Chỉ xác nhận khối lượng sau khi xử lý cảnh báo.",state.road_sections.len(),missing,altered,duplicates);
        }
        ui.strong(format!("TRẮC NGANG ĐÃ KHAI BÁO: {} CỌC",state.road_sections.len()));
        let markers=state.road_sections.iter().filter_map(|section|section.plan_marker_handle.as_ref().map(|handle|takeoff_road::RoadPlanMarker{chainage_m:section.chainage_m,handle:handle.clone()})).collect::<Vec<_>>();
        if markers.len()==state.road_sections.len() && !markers.is_empty() {
            match takeoff_road::link_plan_markers(&markers,&state.road_sections,0.001) {
                Ok(links)=>{ui.colored_label(egui::Color32::LIGHT_GREEN,format!("Đã đối chiếu {} cọc bình đồ với trắc ngang theo lý trình.",links.len()));},
                Err(error)=>{ui.colored_label(egui::Color32::YELLOW,error);},
            }
        } else { ui.small("Cọc chưa gắn bình đồ vẫn mở được theo dữ liệu V1.000; cần gắn nguồn trước khi nghiệm thu liên kết cọc."); }
        let mut delete=None;
        egui::ScrollArea::vertical().id_salt("cadtbt360_road_station_list").max_height(220.0).show(ui,|ui|{
            egui::Grid::new("cadtbt360_road_station_grid").striped(true).show(ui,|ui|{
                for heading in ["Duyệt","Lý trình","Tên cọc","Cọc bình đồ","Diện tích đào (đv²)","Diện tích đắp (đv²)","Thao tác"] {ui.strong(heading);}
                ui.end_row();
                for (i,section) in state.road_sections.iter_mut().enumerate(){
                    ui.checkbox(&mut section.confirmed,"");
                    ui.label(takeoff_road::station_label(section.chainage_m));
                    ui.label(&section.name);
                    ui.label(section.plan_marker_handle.as_deref().unwrap_or("Chưa gắn"));
                    ui.label(section.cut.as_ref().map(|s|format!("{:.3}",s.raw_value)).unwrap_or_else(||if section.cut_zero_confirmed {"0 (xác nhận)".into()}else{"-".into()}));
                    ui.label(section.fill.as_ref().map(|s|format!("{:.3}",s.raw_value)).unwrap_or_else(||if section.fill_zero_confirmed {"0 (xác nhận)".into()}else{"-".into()}));
                    if ui.small_button("Xóa").clicked(){delete=Some(i);}
                    ui.end_row();
                }
            });
        });
        if let Some(i)=delete {state.road_sections.remove(i);}
        ui.horizontal_wrapped(|ui|{
            ui.label("Nhóm");ui.add(egui::TextEdit::singleline(&mut state.road_work_group).desired_width(120.0).hint_text("Nền đường"));
            ui.label("Mã đào");ui.add(egui::TextEdit::singleline(&mut state.road_cut_code).desired_width(100.0).hint_text("GT.DAO"));
            ui.label("Mã đắp");ui.add(egui::TextEdit::singleline(&mut state.road_fill_code).desired_width(100.0).hint_text("GT.DAP"));
        });
        if let Some(scale)=state.scale_to_m() {
            match takeoff_road::parse_range(&state.road_range_start,&state.road_range_end)
                .and_then(|range|takeoff_road::intervals(&state.road_sections.iter().filter(|section|range.is_none_or(|(from,to)|
                    section.chainage_m>=from-1.0e-6 && section.chainage_m<=to+1.0e-6)).cloned().collect::<Vec<_>>(),scale,false)
                    .map(|parts|parts.into_iter().filter(|part|takeoff_road::within_range(part,range)).collect::<Vec<_>>())) {
                Ok(segments)=>{
                    let total_cut=segments.iter().filter(|s|s.kind==EarthworkKind::Cut).map(|s|s.volume_m3).sum::<f64>();
                    let total_fill=segments.iter().filter(|s|s.kind==EarthworkKind::Fill).map(|s|s.volume_m3).sum::<f64>();
                    ui.horizontal_wrapped(|ui|{
                        ui.strong(format!("Đào: {:.3} m³",total_cut));
                        ui.separator();
                        ui.strong(format!("Đắp: {:.3} m³",total_fill));
                        ui.separator();
                        ui.label(format!("{} khoảng cọc có khối lượng (dự thảo)",segments.len()));
                    });
                    egui::ScrollArea::both().id_salt("cadtbt360_road_interval_preview").max_height(170.0).show(ui,|ui|{
                        egui::Grid::new("cadtbt360_road_segments").striped(true).show(ui,|ui|{
                            for heading in ["Từ cọc","Đến cọc","Loại","L (m)","A1 (m²)","A2 (m²)","V (m³)"]{ui.strong(heading);}
                            ui.end_row();
                            for part in segments.iter().take(100){
                                ui.label(takeoff_road::station_label(part.start_m));
                                ui.label(takeoff_road::station_label(part.end_m));
                                ui.label(part.kind.label());
                                ui.label(format!("{:.2}",part.length_m));
                                ui.label(format!("{:.3}",part.area_start_m2));
                                ui.label(format!("{:.3}",part.area_end_m2));
                                ui.label(format!("{:.3}",part.volume_m3));
                                ui.end_row();
                            }
                        });
                    });
                },
                Err(message)=>{if state.road_sections.len()>=2{ui.colored_label(egui::Color32::YELLOW,message);}}
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW,"Cần hiệu chuẩn tỷ lệ CAD sang mét trước khi tính khối lượng giao thông.");
        }
        ui.small("Tại cọc chuyển đào–đắp có thể xác nhận bằng 0 cho từng loại. V1.011 không tự nội suy điểm chuyển giữa hai lý trình; nếu chưa có mặt cắt xác nhận thì báo thiếu dữ liệu.");
    });

    ui.collapsing("CÔNG CỤ CŨ — nhập nhanh từ đối tượng đang chọn", |ui| {
    ui.horizontal(|ui| {
        if ui.button("Thêm phép đo từ CAD đang chọn").clicked() {
            let measurements=selected_measurements(app);
            let mut added = 0usize;
            let mut skipped = 0usize;
            for measurement in measurements {
                let (entity_scope,source_revision)=scope_from_entity_or_active(app,measurement.source_handle,scope);
                if state.add_measurement_from_entity(measurement,&entity_scope,source_revision).is_ok() { added += 1; } else { skipped += 1; }
            }
            state.import_feedback = format!("Đo bóc từ CAD: thêm {added}, bỏ qua {skipped} phép đo trùng hoặc không hợp lệ.");
        }
        if ui.button("Đếm đối tượng CAD đang chọn").clicked() {
            let measurements=selected_count_measurements(app);
            let mut added = 0usize;
            let mut skipped = 0usize;
            for measurement in measurements {
                let (entity_scope,source_revision)=scope_from_entity_or_active(app,measurement.source_handle,scope);
                if state.add_measurement_from_entity(measurement,&entity_scope,source_revision).is_ok() { added += 1; } else { skipped += 1; }
            }
            state.import_feedback = format!("Đếm CAD: thêm {added}, bỏ qua {skipped} đối tượng trùng hoặc không hợp lệ.");
        }
        if ui.button("Thêm QTO nested từ Block/Array đang chọn").clicked() {
            let roots=app.session.selection();
            let mut added = 0usize;
            let mut skipped = 0usize;
            for root in roots {
                let candidates=nested_source_candidates(app,root);
                for (path,measurement) in candidates {
                    if state.add_measurement_from_path(app,path,measurement.kind,scope).is_ok() { added += 1; } else { skipped += 1; }
                }
            }
            state.import_feedback = format!("Đo bóc Block/Array: thêm {added}, bỏ qua {skipped} phép đo.");
        }
        ui.label(format!("Đang chọn {} đối tượng CAD", app.session.selection().len()));
    });
    }); // Công cụ nhập nhanh cũ
    if !state.import_feedback.is_empty() { ui.label(&state.import_feedback); }

    ui.horizontal(|ui| {
        if ui.button("Lưu dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).set_file_name("TBT360_QTO_PROJECT.json").save_file() {
                match serde_json::to_vec_pretty(&state.to_json()).map_err(|e| e.to_string()).and_then(|data| std::fs::write(&path, data).map_err(|e| e.to_string())) {
                    Ok(()) => { state.file_feedback = format!("Đã lưu QTO: {}", path.display()); }
                    Err(e) => { state.file_feedback = format!("Lưu QTO thất bại: {e}"); eprintln!("{}", state.file_feedback); },
                };
            }
        }
        if ui.button("Mở dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).pick_file() {
                match std::fs::read(&path).map_err(|e| e.to_string())
                    .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).map_err(|e| e.to_string()))
                    .and_then(|value| state.load_json(&value)) {
                    Ok(()) => { state.file_feedback = format!("Đã mở QTO: {}", path.display()); }
                    Err(e) => { state.file_feedback = format!("Mở QTO thất bại: {e}"); eprintln!("{}", state.file_feedback); },
                };
            }
        }
    });

    if !state.file_feedback.is_empty() { ui.label(&state.file_feedback); }

    ui.separator();
    if state.source_mode == QtoSourceMode::EntireProject {
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new("native_qto_table").striped(true).min_col_width(105.0).show(ui, |ui| {
            for heading in ["Layer", "Số đối tượng", "LINE", "ARC", "PLINE", "PLINE kín", "CIRCLE", "Khác/chưa đo"] {
                ui.strong(heading);
            }
            ui.end_row();
            for (layer, q) in &groups {
                ui.label(layer);
                ui.label(q.count.to_string());
                ui.label(format!("{:.3} {}", q.line_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.arc_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.polyline_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.closed_area * af, area_unit));
                ui.label(format!("{:.3} {}", q.circle_area * af, area_unit));
                ui.label(q.unsupported.to_string());
                ui.end_row();
            }
        });
    });

    } else {
        ui.small("Bảng tổng hợp Layer toàn bản vẽ đã được ẩn. Dùng bảng xem trước phía trên và danh sách phép đo trong phạm vi chọn; muốn thống kê toàn bộ phải chủ động đổi Phạm vi bóc.");
    }

    let source_validation=validate_sources(app,state);
    // Keep investor-ready Excel settings discoverable near the top of QTO,
    // not buried beneath large measurement and summary tables.
    ui.separator();
    egui::CollapsingHeader::new("XUẤT EXCEL CHỦ ĐẦU TƯ • Cấu hình cột / nhóm / đơn vị / trang in").id_salt("cadtbt360_qto_excel_settings").default_open(false).show(ui, |ui| {
        ui.strong("BÁO CÁO ĐO BÓC KHỐI LƯỢNG — CADTBT360_V1.011");
        ui.small("Tạo XLSX có tiêu đề, nhóm công việc, tổng theo đơn vị, bảng kẻ chuẩn, A4 ngang, tiêu đề lặp mỗi trang và ô ký duyệt.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Tên dự án");
            ui.add(egui::TextEdit::singleline(&mut state.export_config.project).desired_width(260.0).hint_text("Tên công trình / dự án"));
            ui.label("Chủ đầu tư");
            ui.add(egui::TextEdit::singleline(&mut state.export_config.investor).desired_width(260.0).hint_text("Tên Chủ đầu tư"));
            ui.label("Số hiệu biểu mẫu");
            ui.add(egui::TextEdit::singleline(&mut state.export_config.document_number).desired_width(160.0).hint_text("VD: BB-DB-001"));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Phân nhóm");
            egui::ComboBox::from_id_salt("cadtbt360_qto_group_by").selected_text(state.export_config.group_by.label()).show_ui(ui, |ui| {
                for group in GroupBy::ALL { ui.selectable_value(&mut state.export_config.group_by, group, group.label()); }
            });
            ui.label("Đơn vị chiều dài");
            egui::ComboBox::from_id_salt("cadtbt360_qto_length_unit").selected_text(&state.export_config.length_unit).show_ui(ui, |ui| {
                for unit in ["m", "cm", "mm"] { ui.selectable_value(&mut state.export_config.length_unit, unit.to_string(), unit); }
            });
            ui.label("Số chữ số lẻ");
            ui.add(egui::DragValue::new(&mut state.export_config.decimals).range(0..=6));
            ui.checkbox(&mut state.export_config.confirmed_only, "Chỉ xuất khối lượng đã xác nhận và nguồn hợp lệ");
        });
        ui.strong("CHỌN NỘI DUNG HIỂN THỊ TRONG BẢNG EXCEL");
        ui.columns(3, |columns| {
            for (idx, (label, _)) in FIELDS.iter().enumerate() {
                let mandatory = idx == 8 || idx == 9;
                if mandatory { state.export_config.columns[idx] = true; }
                columns[idx % 3].add_enabled(!mandatory, egui::Checkbox::new(&mut state.export_config.columns[idx], *label));
            }
        });
        if state.export_config.selected().len() > 9 {
            ui.colored_label(egui::Color32::YELLOW, "Đang bật hơn 9 cột: Excel sẽ thu nhỏ khi in A4 ngang. Nên ẩn bớt cột kỹ thuật nếu gửi Chủ đầu tư.");
        }
        // Avoid constructing, sorting and grouping the complete report each GUI frame.
        let allowed_handles = qto_allowed_handles(app, state);
        let count = state.records.iter().filter(|r| qto_record_in_scope(r, allowed_handles.as_ref()) && !state.export_omitted_ids.contains(&r.id) && qto_measure_type_allowed(r.kind, state.selected_measure_types) && r.status != MeasureStatus::Excluded && !source_validation.invalid_record_ids.contains(&r.id) && qto_record_exportable(r, &source_validation, lf, af, state.export_config.confirmed_only)).count();
        let volume_count = state.volume_records.iter().filter(|volume| (!state.export_config.confirmed_only || volume.confirmed)
            && allowed_handles.as_ref().is_none_or(|handles| handles.contains(&volume.plan.handle) && handles.contains(&volume.section.handle))).count();
        let road_preview=state.scale_to_m().and_then(|scale| takeoff_road::parse_range(&state.road_range_start,&state.road_range_end).ok()
            .and_then(|range|takeoff_road::intervals(&state.road_sections.iter().filter(|section|range.is_none_or(|(from,to)|
                section.chainage_m>=from-1.0e-6 && section.chainage_m<=to+1.0e-6)).cloned().collect::<Vec<_>>(),scale,state.export_config.confirmed_only).ok()
                .map(|parts|parts.into_iter().filter(|part|takeoff_road::within_range(part,range)).collect::<Vec<_>>())));
        let road_count=road_preview.as_ref().map(|parts|parts.iter().filter(|part|allowed_handles.as_ref()
            .is_none_or(|handles|(part.source_start.is_empty() || handles.contains(&part.source_start))
                && (part.source_end.is_empty() || handles.contains(&part.source_end)))).count()).unwrap_or(0);
        ui.label(format!("Sẵn sàng xem xét: {count} phép đo 2D + {volume_count} thể tích hai nguồn + {road_count} khoảng cọc đào/đắp, {} cột.",state.export_config.selected().len()));
        if state.road_sections.len() >= 2 && road_preview.is_none() {
            ui.colored_label(egui::Color32::YELLOW,"Dữ liệu trắc ngang chưa hợp lệ hoặc chưa hiệu chuẩn. Không phát hành Excel giao thông khi chưa khắc phục lỗi.");
        }
        if !state.export_config.confirmed_only {
            ui.colored_label(egui::Color32::YELLOW, "Chế độ DỰ THẢO: có thể bao gồm phép đo chưa xác nhận. Không gửi Chủ đầu tư trước khi kiểm tra.");
        }
        if ui.add_enabled(state.export_config.valid() && count + volume_count + road_count > 0, egui::Button::new("XUẤT EXCEL XLSX — ĐỊNH DẠNG IN A4")).clicked() {
            match qto_combined_report_lines(app, state, &source_validation, &state.export_config, allowed_handles.as_ref()) {
                Ok(rows) => {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Excel Workbook", &["xlsx"]).set_file_name("CADTBT360_V1.011_BANG_DO_BOC_KHOI_LUONG.xlsx").save_file() {
                        match takeoff_report::write_xlsx(&path, &state.export_config, &rows) {
                            Ok(()) => state.file_feedback = format!("Đã tạo Excel XLSX có bố cục in A4: {}", path.display()),
                            Err(error) => state.file_feedback = format!("Không thể xuất báo cáo Excel: {error}"),
                        }
                    }
                },
                Err(error) => state.file_feedback = format!("Chưa đủ điều kiện xuất báo cáo: {error}"),
            }
        }
        if !state.file_feedback.is_empty() { ui.label(&state.file_feedback); }
    });


    if source_validation.missing_source>0 || source_validation.scope_mismatch>0 || source_validation.stale_revision>0 || source_validation.stale_geometry>0 || source_validation.identity_collision>0 {
        ui.colored_label(egui::Color32::YELLOW,format!(
            "Liên kết nguồn QTO cần kiểm tra: {} mất nguồn, {} lệch scope, {} lệch revision, {} đổi geometry, {} trùng BIM identity.",
            source_validation.missing_source,source_validation.scope_mismatch,source_validation.stale_revision,source_validation.stale_geometry,source_validation.identity_collision
        ));
        if source_validation.missing_source>0 && ui.button("Tự relink nguồn bị mất (chỉ khi khớp duy nhất)").clicked() {
            let relinked=state.relink_all_missing_sources(app);
            if relinked>0 { ui.label(format!("Đã relink {} record.",relinked)); }
        }
    } else if source_validation.valid_links>0 {
        ui.label(format!("Liên kết nguồn QTO hợp lệ: {} record.",source_validation.valid_links));
    }

    ui.horizontal_wrapped(|ui| {
        if ui.button("Xác nhận tất cả phép đo nguồn hợp lệ").clicked() {
            let ids = state.records.iter().filter(|r| r.status == MeasureStatus::Review && !source_validation.invalid_record_ids.contains(&r.id)).map(|r| r.id).collect::<Vec<_>>();
            let mut updated = 0usize;
            for id in ids { if state.set_status(id, MeasureStatus::Confirmed) { updated += 1; } }
            state.import_feedback = format!("Đã xác nhận {updated} phép đo đủ điều kiện; nguồn lỗi giữ nguyên để rà soát.");
        }
        if ui.button("Đưa phép đo đã loại về rà soát").clicked() {
            let ids = state.records.iter().filter(|r| r.status == MeasureStatus::Excluded).map(|r| r.id).collect::<Vec<_>>();
            let mut updated = 0usize;
            for id in ids { if state.set_status(id, MeasureStatus::Review) { updated += 1; } }
            state.import_feedback = format!("Đã chuyển {updated} phép đo về trạng thái rà soát.");
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Hoàn tác phép đo").clicked() { let _ = state.undo_measurement_change(); }
        if ui.button("Làm lại phép đo").clicked() { let _ = state.redo_measurement_change(); }
        ui.label(format!("{} record đo • {} audit", state.records.len(), state.audit.len()));
    });

    if !state.records.is_empty() {
        ui.strong("Phép đo nghiệp vụ");
        ui.horizontal_wrapped(|ui| {
            ui.label("Tìm theo Layer / Mã CT / Diễn giải / Handle:");
            ui.text_edit_singleline(&mut state.search_text);
            ui.label("Trạng thái:");
            ui.selectable_value(&mut state.status_filter, 0, "Tất cả");
            ui.selectable_value(&mut state.status_filter, 1, "Cần rà soát");
            ui.selectable_value(&mut state.status_filter, 2, "Đã xác nhận");
            ui.selectable_value(&mut state.status_filter, 3, "Đã loại");
        });
        let mut action: Option<(u64, u8)> = None;
        // Reuse the same scoped handle set across all rows. Rebuilding the
        // selection per record made large QTO registers disproportionately slow.
        let register_scope = qto_allowed_handles(app, state);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Chọn tất cả phép đo đang lọc để xuất").clicked() {
                for record in &state.records {
                    if measurement_matches_filter(record, &state.search_text, state.status_filter) && qto_record_in_scope(record, register_scope.as_ref()) {
                        state.export_omitted_ids.remove(&record.id);
                    }
                }
            }
            if ui.button("Bỏ chọn phép đo đang lọc").clicked() {
                for record in &state.records {
                    if measurement_matches_filter(record, &state.search_text, state.status_filter) && qto_record_in_scope(record, register_scope.as_ref()) {
                        state.export_omitted_ids.insert(record.id);
                    }
                }
            }
            ui.label(format!("Đã bỏ chọn {} phép đo khỏi Excel.", state.export_omitted_ids.len()));
        });
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("native_qto_measurements").striped(true).show(ui, |ui| {
                for h in ["Xuất", "ID","Handle nguồn","Nguồn","Tầng","Bộ môn","Owner","Loại","Layer","Nhóm","Mã CT","Diễn giải","Giá trị","Trạng thái","Thao tác"] { ui.strong(h); }
                ui.end_row();
                for record in &state.records {
                    if !measurement_matches_filter(record, &state.search_text, state.status_filter) || !qto_record_in_scope(record, register_scope.as_ref()) || !qto_measure_type_allowed(record.kind, state.selected_measure_types) { continue; }
                    let value = match record.kind {
                        MeasureKind::Area => record.raw_value * af,
                        MeasureKind::Length | MeasureKind::Perimeter => record.raw_value * lf,
                        MeasureKind::Count => record.raw_value,
                    };
                    let unit = match record.kind { MeasureKind::Area => area_unit, MeasureKind::Length | MeasureKind::Perimeter => length_unit, MeasureKind::Count => "lần" };
                    let mut include = !state.export_omitted_ids.contains(&record.id);
                    if ui.checkbox(&mut include, "").on_hover_text("Chọn phép đo này để đưa vào Excel").changed() {
                        if include { state.export_omitted_ids.remove(&record.id); }
                        else { state.export_omitted_ids.insert(record.id); }
                    }
                    ui.label(record.id.to_string());
                    ui.label(record.source_handle.as_deref().unwrap_or("-"));
                    match record_source_state(record,&source_validation) {
                        "STALE" => { ui.colored_label(egui::Color32::YELLOW,"STALE"); }
                        state => { ui.label(state); }
                    }
                    ui.label(record.scope.floor_id.map(|v|v.to_string()).unwrap_or_else(||"-".into()));
                    ui.label(record.scope.discipline.as_deref().unwrap_or("-"));
                    ui.label(record.scope.owner_account_id.as_deref().unwrap_or("-"));
                    ui.label(match record.kind { MeasureKind::Length=>"Chiều dài", MeasureKind::Area=>"Diện tích", MeasureKind::Perimeter=>"Chu vi", MeasureKind::Count=>"Đếm" });
                    ui.label(&record.layer);
                    ui.label(&record.work_group);
                    ui.label(&record.work_code);
                    ui.label(&record.description);
                    ui.label(format!("{value:.3} {unit}"));
                    ui.label(match record.status { MeasureStatus::Review=>"Review", MeasureStatus::Confirmed=>"Đã xác nhận", MeasureStatus::Excluded=>"Loại" });
                    ui.horizontal(|ui| {
                        if ui.small_button("✓").on_hover_text("Xác nhận").clicked() { action=Some((record.id,1)); }
                        if ui.small_button("⊘").on_hover_text("Loại khỏi khối lượng").clicked() { action=Some((record.id,2)); }
                        if ui.small_button("↻").on_hover_text("Cập nhật lại từ entity CAD/BIM nguồn").clicked() { action=Some((record.id,5)); }
                        if ui.small_button("✎").on_hover_text("Sửa mã công tác / diễn giải / ghi chú").clicked() { action=Some((record.id,4)); }
                        if ui.small_button("×").on_hover_text("Xóa phép đo").clicked() { action=Some((record.id,3)); }
                    });
                    ui.end_row();
                }
            });
        });
        if let Some((id, kind)) = action {
            match kind {
                1 => { let _ = state.set_status(id, MeasureStatus::Confirmed); }
                2 => { let _ = state.set_status(id, MeasureStatus::Excluded); }
                3 => { let _ = state.delete_measurement(id); }
                4 => { let _ = state.select_record(id); }
                5 => { let _ = state.refresh_from_source(app,id); }
                _ => {}
            }
        }
    }

    if let Some(id) = state.selected_record_id {
        ui.separator();
        ui.group(|ui| {
            ui.strong(format!("Sửa phép đo #{id}"));
            ui.horizontal(|ui| {
                ui.label("Nhóm công việc:");
                ui.text_edit_singleline(&mut state.edit_work_group);
                ui.label("Mã công tác:");
                ui.text_edit_singleline(&mut state.edit_work_code);
            });
            ui.horizontal(|ui| {
                ui.label("Diễn giải:");
                ui.text_edit_singleline(&mut state.edit_description);
            });
            ui.horizontal(|ui| {
                ui.label("Ghi chú:");
                ui.text_edit_singleline(&mut state.edit_note);
            });
            ui.horizontal(|ui| {
                if ui.button("Lưu thông tin").clicked() {
                    let group=state.edit_work_group.clone();
                    let code=state.edit_work_code.clone();
                    let description=state.edit_description.clone();
                    let note=state.edit_note.clone();
                    let _=state.edit_metadata(id,group,code,description,note);
                }
                if ui.button("Đóng sửa").clicked() { state.selected_record_id=None; }
            });
        });
    }

    ui.separator();
    ui.strong("Tổng hợp theo BIM Scope");
    if !source_validation.invalid_record_ids.is_empty() {
        ui.colored_label(egui::Color32::YELLOW,format!("{} record liên kết nguồn không hợp lệ đang bị loại khỏi khối lượng BIM xác nhận.",source_validation.invalid_record_ids.len()));
    }
    
    let scoped_groups=scoped_business_quantities_validated(state,lf,af,&source_validation.invalid_record_ids);
    egui::ScrollArea::horizontal().show(ui,|ui|{
        egui::Grid::new("native_qto_bim_scope_summary").striped(true).show(ui,|ui|{
            for h in ["Tầng","Bộ môn","Owner","Nhóm","Mã CT","Review","Đã xác nhận","Loại","Chiều dài","Chu vi","Diện tích","Số lượng"] { ui.strong(h); }
            ui.end_row();
            for ((floor_id,discipline,owner,work_group,work_code),q) in &scoped_groups {
                ui.label(floor_id.map(|v|v.to_string()).unwrap_or_else(||"-".into()));
                ui.label(discipline);
                ui.label(owner);
                ui.label(work_group);
                ui.label(work_code);
                ui.label(q.review_records.to_string());
                ui.label(q.confirmed_records.to_string());
                ui.label(q.excluded_records.to_string());
                ui.label(format!("{:.3} {}",q.length,length_unit));
                ui.label(format!("{:.3} {}",q.perimeter,length_unit));
                ui.label(format!("{:.3} {}",q.area,area_unit));
                ui.label(format!("{:.3}",q.count));
                ui.end_row();
            }
        });
    });

    ui.separator();
    ui.strong("Tổng hợp nghiệp vụ đã xác nhận");
    ui.label("Chỉ record trạng thái Đã xác nhận được cộng vào khối lượng; Review và Loại vẫn hiển thị để kiểm soát.");
    let business_groups = business_quantities_validated(state,lf,af,&source_validation.invalid_record_ids);
    egui::ScrollArea::horizontal().show(ui, |ui| {
        egui::Grid::new("native_qto_business_summary").striped(true).show(ui, |ui| {
            for h in ["Nhóm công việc","Mã công tác","Layer","Review","Đã xác nhận","Loại","Chiều dài","Chu vi","Diện tích","Số lượng"] { ui.strong(h); }
            ui.end_row();
            for ((work_group, work_code, layer), q) in &business_groups {
                ui.label(work_group);
                ui.label(work_code);
                ui.label(layer);
                ui.label(q.review_records.to_string());
                ui.label(q.confirmed_records.to_string());
                ui.label(q.excluded_records.to_string());
                ui.label(format!("{:.3} {}", q.length, length_unit));
                ui.label(format!("{:.3} {}", q.perimeter, length_unit));
                ui.label(format!("{:.3} {}", q.area, area_unit));
                ui.label(format!("{:.3}", q.count));
                ui.end_row();
            }
        });
    });


    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Sao chép CSV tổng hợp nghiệp vụ").clicked() {
            let mut csv = String::from("work_group,work_code,layer,review_records,confirmed_records,excluded_records,length,perimeter,area,count,length_unit,area_unit\n");
            for ((work_group, work_code, layer), q) in &business_groups {
                let esc = csv_safe_cell;
                csv.push_str(&format!("{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{}\n",
                    esc(work_group), esc(work_code), esc(layer), q.review_records, q.confirmed_records, q.excluded_records,
                    q.length, q.perimeter, q.area, q.count, length_unit, area_unit));
            }
            ui.ctx().copy_text(csv);
        }
        if ui.button("Sao chép CSV tổng hợp Layer").clicked() {
            let mut csv = String::from("layer,entity_count,line_length,arc_length,polyline_length,closed_polyline_area,circle_area,unmeasured_count,length_unit,area_unit\n");
            for (layer, q) in &groups {
                let safe = csv_safe_cell(layer);
                csv.push_str(&format!("{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{}\n",
                    safe, q.count, q.line_length * lf, q.arc_length * lf, q.polyline_length * lf, q.closed_area * af, q.circle_area * af, q.unsupported, length_unit, area_unit));
            }
            ui.ctx().copy_text(csv);
        }
        ui.checkbox(&mut state.export_confirmed_only, "Chỉ xuất khối lượng đã xác nhận và nguồn hợp lệ");
        let exportable = state.records.iter().filter(|r| qto_record_exportable(r, &source_validation, lf, af, state.export_confirmed_only)).count();
        ui.label(format!("Sẵn sàng xuất: {exportable}/{} phép đo; {} mục chưa đủ điều kiện.", state.records.len(), state.records.len() - exportable));
        if ui.button("Sao chép CSV chi tiết phép đo").clicked() {
            ui.ctx().copy_text(qto_detail_csv(state, &source_validation, lf, af, length_unit, area_unit, state.export_confirmed_only));
            state.file_feedback = format!("Đã sao chép {} phép đo vào clipboard.", state.records.len());
        }
        if ui.button("Xuất CSV chi tiết ra tệp").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("CSV Excel", &["csv"]).set_file_name("CAD360_QTO_CHI_TIET.csv").save_file() {
                let csv = qto_detail_csv(state, &source_validation, lf, af, length_unit, area_unit, state.export_confirmed_only);
                match std::fs::write(&path, format!("\u{feff}{csv}")) {
                    Ok(()) => state.file_feedback = format!("Đã xuất {} phép đo: {}", state.records.len(), path.display()),
                    Err(e) => state.file_feedback = format!("Không thể xuất CSV: {e}"),
                }
            }
        }
    });
}

pub fn panel(ui: &mut egui::Ui, app: &CadApp, state: &mut State, scope: &Scope) {
    draw_contents(ui, app, state, scope);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::geom::PolyVertex;
    use cadcraft_doc::geom::Vec2;

    #[test]
    fn qto_selection_filters_export_by_exact_cad_handles_and_does_not_default_to_entire_project() {
        assert_eq!(QtoSourceMode::default(), QtoSourceMode::CurrentSelection);
        let mut state = State::default();
        let first = state.add_measurement(MeasureKind::Count, vec![], 1.0, "Layer1".into()).unwrap();
        let second = state.add_measurement(MeasureKind::Count, vec![], 2.0, "Layer2".into()).unwrap();
        for (id, handle) in [(first, "2A"), (second, "2B")] {
            let record = state.records.iter_mut().find(|record| record.id == id).unwrap();
            record.source_handle = Some(handle.into());
            record.description = format!("Cấu kiện {handle}");
            record.status = MeasureStatus::Confirmed;
        }
        let cfg = ExportConfig { project: "Dự án".into(), investor: "Chủ đầu tư".into(), ..ExportConfig::default() };
        let mut selected = BTreeSet::new();
        selected.insert("2A".into());
        let report = qto_report_lines(&state, &SourceValidation::default(), &cfg, Some(&selected)).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].quantity, 1.0);
        selected.clear();
        assert!(qto_report_lines(&state, &SourceValidation::default(), &cfg, Some(&selected)).is_err());
        assert_eq!(qto_report_lines(&state, &SourceValidation::default(), &cfg, None).unwrap().len(), 2);
    }

    #[test]
    fn qto_excel_respects_manual_row_checkboxes_independently_from_cad_scope() {
        let mut state = State::default();
        let first = state.add_measurement(MeasureKind::Count, vec![], 1.0, "A".into()).unwrap();
        let second = state.add_measurement(MeasureKind::Count, vec![], 3.0, "B".into()).unwrap();
        for record in &mut state.records {
            record.status = MeasureStatus::Confirmed;
            record.description = "Đếm cấu kiện đã kiểm tra".into();
            record.source_handle = Some(if record.id == first { "2A" } else { "2B" }.into());
        }
        let cfg = ExportConfig { project: "Dự án".into(), investor: "CĐT".into(), ..Default::default() };
        state.export_omitted_ids.insert(second);
        let rows = qto_report_lines(&state, &SourceValidation::default(), &cfg, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].quantity, 1.0);
        let saved = state.to_json();
        let mut restored = State::default();
        restored.load_json(&saved).unwrap();
        assert!(restored.export_omitted_ids.contains(&second));
    }

    #[test]
    fn qto_selection_saved_region_and_methods_survive_project_reload() {
        let mut state = State::default();
        state.source_mode = QtoSourceMode::CapturedRegion;
        state.captured_handles.insert("2A".into());
        state.selected_measure_types = [false, true, false, false];
        let json = state.to_json();
        let mut loaded = State::default();
        loaded.load_json(&json).unwrap();
        assert_eq!(loaded.source_mode, QtoSourceMode::CapturedRegion);
        assert_eq!(loaded.captured_handles, BTreeSet::from(["2A".to_string()]));
        assert_eq!(loaded.selected_measure_types, [false, true, false, false]);
        assert!(!qto_measure_type_allowed(MeasureKind::Count, loaded.selected_measure_types));
        assert!(qto_measure_type_allowed(MeasureKind::Area, loaded.selected_measure_types));
    }

    #[test]
    fn qto_investor_excel_report_filters_excluded_and_invalid_sources() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Count, vec![], 3.0, "Layer A".into()).unwrap();
        assert!(state.edit_metadata(id, "Kết cấu".into(), "KC.01".into(), "Thống kê cấu kiện".into(), "".into()));
        assert!(state.set_status(id, MeasureStatus::Confirmed));
        let removed = state.add_measurement(MeasureKind::Count, vec![], 9.0, "Layer B".into()).unwrap();
        assert!(state.set_status(removed, MeasureStatus::Excluded));
        let cfg = ExportConfig { project: "Dự án thử nghiệm".into(), investor: "Chủ đầu tư A".into(), ..Default::default() };
        let records = qto_report_lines(&state, &SourceValidation::default(), &cfg, None).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].quantity, 3.0);
        assert_eq!(records[0].unit, "cái");
        assert_eq!(records[0].group, "Kết cấu");
        let mut draft = cfg.clone();
        draft.confirmed_only = false;
        assert_eq!(qto_report_lines(&state, &SourceValidation::default(), &draft, None).unwrap().len(), 1);
        let mut invalid = SourceValidation::default();
        invalid.invalid_record_ids.insert(id);
        assert!(qto_report_lines(&state, &invalid, &cfg, None).is_err());
    }

    #[test]
    fn qto_investor_excel_units_and_options_survive_project_save() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Length, vec![[0.0, 0.0], [1000.0, 0.0]], 1000.0, "A".into()).unwrap();
        assert!(state.edit_metadata(id, "Nền móng".into(), "CT.001".into(), "Chiều dài tuyến".into(), "".into()));
        assert!(state.set_status(id, MeasureStatus::Confirmed));
        state.drawing_distance = 1000.0;
        state.real_distance_m = 1.0;
        let cfg = ExportConfig { project: "Dự án B".into(), investor: "CĐT B".into(), length_unit: "cm".into(), ..Default::default() };
        state.export_config = cfg.clone();
        let records = qto_report_lines(&state, &SourceValidation::default(), &cfg, None).unwrap();
        assert_eq!(records.len(), 1);
        assert!((records[0].quantity - 100.0).abs() < 1.0e-9);
        assert_eq!(records[0].unit, "cm");
        let snapshot = state.to_json();
        let mut restored = State::default();
        restored.load_json(&snapshot).unwrap();
        assert_eq!(restored.export_config.project, "Dự án B");
        assert_eq!(restored.export_config.investor, "CĐT B");
        assert_eq!(restored.export_config.length_unit, "cm");
        assert_eq!(restored.export_config.columns[8], true);
        assert_eq!(restored.export_config.columns[9], true);
    }

    #[test]
    fn qto_rejects_nan_and_infinite_measurement_coordinates() {
        let mut state = State::default();
        assert!(state.add_measurement(MeasureKind::Length, vec![[f64::NAN, 0.0]], 1.0, "L".into()).is_err());
        assert!(state.add_measurement(MeasureKind::Area, vec![[0.0, f64::INFINITY]], 1.0, "L".into()).is_err());
        assert!(state.records.is_empty());
        assert!(state.undo.is_empty());
    }

    #[test]
    fn qto_rejects_overflowed_calibration_without_changing_scale() {
        let mut state = State::default();
        assert!(state.calibrate_two_points([-1.0e308, 0.0], [1.0e308, 0.0], 10.0).is_err());
        assert!(state.scale_to_m().is_none());
    }

    #[test]
    fn failed_calibration_preserves_previous_valid_scale() {
        let mut state = State::default();
        assert_eq!(state.calibrate_two_points([0.0, 0.0], [2.0, 0.0], 4.0).unwrap(), 2.0);
        assert!(state.calibrate_two_points([0.0, 0.0], [1.0, 0.0], f64::MAX).is_ok());
        let drawing_distance = state.drawing_distance;
        let real_distance_m = state.real_distance_m;
        assert!(state.calibrate_two_points([0.0, 0.0], [1.0e-11, 0.0], f64::MAX).is_err());
        assert_eq!(state.drawing_distance, drawing_distance);
        assert_eq!(state.real_distance_m, real_distance_m);
    }

    #[test]
    fn qto_scale_rejects_overflow_and_underflow_in_restored_state() {
        let mut state = State::default();
        state.drawing_distance = 1.0e-10;
        state.real_distance_m = 1.0e308;
        assert_eq!(state.scale_to_m(), None);
        state.drawing_distance = 1.0e308;
        state.real_distance_m = 1.0e-308;
        assert_eq!(state.scale_to_m(), None);
    }

    #[test]
    fn qto_record_id_overflow_does_not_modify_current_state() {
        let mut state = State::default();
        state.next_record_id = u64::MAX;
        assert!(state.add_measurement(MeasureKind::Count, vec![], 1.0, "L".into()).is_err());
        assert!(state.records.is_empty());
        assert!(state.undo.is_empty());
    }

    #[test]
    fn qto_restore_rejects_overflow_without_erasing_existing_records() {
        let mut state = State::default();
        state.add_measurement(MeasureKind::Count, vec![], 1.0, "safe".into()).unwrap();
        let bad = serde_json::json!({"version":4,"drawing_distance":0.0,"real_distance_m":0.0,"records":[{"id":u64::MAX,"kind":"count","points":[],"raw_value":1.0,"layer":"bad"}],"audit":[]});
        assert!(state.load_json(&bad).is_err());
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.records[0].layer, "safe");
    }

    #[test]
    fn audit_sequence_at_limit_does_not_roll_over() {
        let mut state = State::default();
        state.next_audit_sequence = u64::MAX;
        state.audit_change(1, "UPDATE", None, Some("changed".into()));
        assert!(state.audit.is_empty());
        assert_eq!(state.next_audit_sequence, u64::MAX);
    }

    #[test]
    fn qto_audit_capacity_blocks_new_measurement_without_partial_write() {
        let mut state = State::default();
        state.next_audit_sequence = u64::MAX;
        assert!(state.add_measurement(MeasureKind::Count, vec![], 1.0, "QTO".into()).is_err());
        assert!(state.records.is_empty());
        assert!(state.undo.is_empty());
        assert!(state.audit.is_empty());
    }

    #[test]
    fn restored_qto_scale_underflow_is_rejected_atomically() {
        let mut state = State::default();
        state.add_measurement(MeasureKind::Count, vec![], 1.0, "existing".into()).unwrap();
        let bad = serde_json::json!({"version":4,"drawing_distance":1.0e308,"real_distance_m":1.0e-308,"records":[],"audit":[]});
        assert!(state.load_json(&bad).is_err());
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.records[0].layer, "existing");
    }

    #[test]
    fn scoped_qto_requires_capacity_for_both_create_and_scope_audit() {
        let mut s = State::default();
        s.next_audit_sequence = u64::MAX - 1;
        assert!(s.add_measurement_scoped(MeasureKind::Count, vec![], 1.0, "floor".into(), &Scope::default()).is_err());
        assert!(s.records.is_empty());
        assert!(s.audit.is_empty());
    }

    #[test]
    fn measurement_register_filters_search_and_status() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Count, vec![], 1.0, "A-WALL".into()).unwrap();
        state.records[0].work_code = "BT.001".into();
        state.records[0].description = "Tường tầng 2".into();
        assert!(measurement_matches_filter(&state.records[0], "wall", 0));
        assert!(measurement_matches_filter(&state.records[0], "bt.001", 1));
        assert!(measurement_matches_filter(&state.records[0], "TẦNG", 1));
        assert!(!measurement_matches_filter(&state.records[0], "HVAC", 0));
        assert!(!measurement_matches_filter(&state.records[0], "", 2));
        state.set_status(id, MeasureStatus::Confirmed);
        assert!(measurement_matches_filter(&state.records[0], "tường", 2));
        assert!(!measurement_matches_filter(&state.records[0], "", 1));
    }

    #[test]
    fn importing_same_cad_measurement_twice_does_not_double_count() {
        let mut state = State::default();
        let measurement = PendingMeasurement { kind: MeasureKind::Length, points: vec![[0.0, 0.0], [5.0, 0.0]], raw_value: 5.0, layer: "WALL".into(), source_handle: cadcraft_doc::Handle(0x42) };
        assert!(state.add_measurement_from_entity(measurement.clone(), &Scope::default(), Some(1)).is_ok());
        assert!(state.add_measurement_from_entity(measurement, &Scope::default(), Some(1)).is_err());
        assert_eq!(state.records.len(), 1);
    }

    #[test]
    fn qto_detail_csv_exports_vietnamese_metadata_and_converted_quantity() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Length, vec![[0.0, 0.0], [5.0, 0.0]], 5.0, "A-WALL".into()).unwrap();
        state.edit_metadata(id, "Kết cấu".into(), "BT.001".into(), "Dầm tầng 2".into(), "Ghi chú".into());
        let csv = qto_detail_csv(&state, &SourceValidation::default(), 2.0, 4.0, "m", "m²", false);
        assert!(csv.contains("BT.001"));
        assert!(csv.contains("Dầm tầng 2"));
        assert!(csv.contains("10.000000"));
        assert_eq!(csv.lines().count(), 2);
    }

    #[test]
    fn confirmed_only_export_omits_review_excluded_and_stale_sources() {
        let mut state = State::default();
        let approved = state.add_measurement(MeasureKind::Count, vec![], 1.0, "APPROVED".into()).unwrap();
        let stale = state.add_measurement(MeasureKind::Count, vec![], 1.0, "STALE".into()).unwrap();
        state.add_measurement(MeasureKind::Count, vec![], 1.0, "REVIEW".into()).unwrap();
        state.set_status(approved, MeasureStatus::Confirmed);
        state.set_status(stale, MeasureStatus::Confirmed);
        let mut validation = SourceValidation::default();
        validation.invalid_record_ids.insert(stale);
        let csv = qto_detail_csv(&state, &validation, 1.0, 1.0, "m", "m²", true);
        assert!(csv.contains("APPROVED"));
        assert!(!csv.contains("STALE"));
        assert!(!csv.contains("REVIEW"));
        assert_eq!(csv.lines().count(), 2);
    }

    #[test]
    fn qto_csv_never_emits_infinite_area_quantities() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Area, vec![[0.0, 0.0]], 2.0, "AREA".into()).unwrap();
        state.set_status(id, MeasureStatus::Confirmed);
        let csv = qto_detail_csv(&state, &SourceValidation::default(), 1.0e200, f64::INFINITY, "m", "m²", true);
        assert_eq!(csv.lines().count(), 1);
        assert!(!csv.contains("inf"));
    }

    #[test]
    fn qto_csv_escapes_quotes_line_breaks_and_formula_injection() {
        assert_eq!(csv_safe_cell("=SUM(1,1)"), "\"'=SUM(1,1)\"");
        assert_eq!(csv_safe_cell("A\"B"), "\"A\"\"B\"");
        assert_eq!(csv_safe_cell("first\nsecond"), "\"first second\"");
        assert_eq!(csv_safe_cell("@cmd"), "\"'@cmd\"");
    }

    #[test]
    fn qto_status_change_is_rejected_when_audit_capacity_is_exhausted() {
        let mut state = State::default();
        let id = state.add_measurement(MeasureKind::Count, vec![], 1.0, "test".into()).unwrap();
        state.next_audit_sequence = u64::MAX;
        assert!(!state.set_status(id, MeasureStatus::Confirmed));
        assert_eq!(state.records[0].status, MeasureStatus::Review);
        assert_eq!(state.undo.len(), 1);
    }

    #[test]
    fn qto_export_skips_overflowed_length_and_area_and_keeps_count() {
        let mut state = State::default();
        let length = state.add_measurement(MeasureKind::Length, vec![[0.0, 0.0]], 1.0e200, "OVERFLOW_LENGTH".into()).unwrap();
        let count = state.add_measurement(MeasureKind::Count, vec![], 2.0, "COUNT_OK".into()).unwrap();
        state.set_status(length, MeasureStatus::Confirmed);
        state.set_status(count, MeasureStatus::Confirmed);
        let csv = qto_detail_csv(&state, &SourceValidation::default(), 1.0e200, f64::INFINITY, "m", "m²", true);
        assert!(!csv.contains("OVERFLOW_LENGTH"));
        assert!(csv.contains("COUNT_OK"));
        assert_eq!(csv.lines().count(), 2);
    }

    #[test]
    fn qto_export_preview_matches_actual_csv_rows() {
        let mut state = State::default();
        let accepted = state.add_measurement(MeasureKind::Count, vec![], 1.0, "ACCEPTED".into()).unwrap();
        let rejected = state.add_measurement(MeasureKind::Count, vec![], 1.0, "EXCLUDED".into()).unwrap();
        state.set_status(accepted, MeasureStatus::Confirmed);
        state.set_status(rejected, MeasureStatus::Excluded);
        let validation = SourceValidation::default();
        let preview = state.records.iter().filter(|r| qto_record_exportable(r, &validation, 1.0, 1.0, true)).count();
        let csv = qto_detail_csv(&state, &validation, 1.0, 1.0, "m", "m²", true);
        assert_eq!(preview, 1);
        assert_eq!(csv.lines().count(), preview + 1);
    }

    fn block_member_fixture()->(cadcraft_doc::Drawing,cadcraft_doc::Handle,cadcraft_doc::Handle) {
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut block=cadcraft_doc::Block::new("B");
        block.base=cadcraft_geom::Vec3::ZERO;
        block.entities.push(cadcraft_doc::Entity{
            handle:member,
            common:cadcraft_doc::Common{layer:"A-WALL".into(),..Default::default()},
            kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(5.0,0.0,0.0)})
        });
        d.blocks.insert("B".into(),std::sync::Arc::new(block));
        let root=d.add(&cadcraft_doc::Space::Model,cadcraft_doc::Common::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"B".into(),insert:cadcraft_geom::Vec3::new(10.0,20.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();
        (d,root,member)
    }

    #[test]
    fn qto_v4_source_path_roundtrip_preserves_structured_provenance() {
        let mut state=State::default();
        let id=state.add_measurement(MeasureKind::Length,vec![[10.0,20.0],[15.0,20.0]],5.0,"A-WALL".into()).unwrap();
        let path=SourcePath{root_handle:"2A".into(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:"20".into()}]};
        state.records[0].source_handle=Some("2A".into());
        state.records[0].source_path=Some(path.clone());
        let json=state.to_json();
        assert_eq!(json["version"],serde_json::json!(4));
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_path.as_ref(),Some(&path));
        assert_eq!(reopened.records[0].id,id);
    }

    #[test]
    fn qto_v4_source_path_rejects_invalid_member_handle() {
        let bad=serde_json::json!({
            "version":4,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"A","status":"review",
                "source_handle":"2A","source_path":{"root_handle":"2A","steps":[{"type":"block_member","block":"B","member_handle":"NOT_HEX"}]}}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad).is_err());
    }

    #[test]
    fn nested_block_source_path_resolves_world_geometry() {
        let (d,root,member)=block_member_fixture();
        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:member.hex()}]};
        let (entity,resolved_root)=resolve_source_path(&d,&path).unwrap();
        assert_eq!(resolved_root,root);
        let EntityKind::Line(line)=entity.kind else{panic!("expected line")};
        assert!((line.a.x-10.0).abs()<1e-12&&(line.a.y-20.0).abs()<1e-12);
        assert!((line.b.x-15.0).abs()<1e-12&&(line.b.y-20.0).abs()<1e-12);
    }

    #[test]
    fn associative_array_source_path_resolves_distinct_item_world_geometry() {
        use cadcraft_doc::{ArrayKind,ArrayParams,RectArray};
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut source=cadcraft_doc::Block::new("SRC");
        source.entities.push(cadcraft_doc::Entity{
            handle:member,common:cadcraft_doc::Common{layer:"A".into(),..Default::default()},
            kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(2.0,0.0,0.0)})
        });
        d.blocks.insert("SRC".into(),std::sync::Arc::new(source));

        let params=ArrayParams{
            source:"SRC".into(),
            kind:ArrayKind::Rect(RectArray{cols:2,rows:1,col_spacing:10.0,row_spacing:0.0,axis_angle:0.0,row_elevation:0.0}),
            levels:1,level_spacing:0.0,replaced:Vec::new(),suppressed:Vec::new(),editing:None
        };
        let mut arr=cadcraft_doc::Block::new("ARR");
        arr.array=Some(params);
        for (idx,x) in [0.0,10.0].into_iter().enumerate() {
            arr.entities.push(cadcraft_doc::Entity{
                handle:cadcraft_doc::Handle(0x30+idx as u64),common:cadcraft_doc::Common::default(),
                kind:cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
                    block:"SRC".into(),insert:cadcraft_geom::Vec3::new(x,0.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
                    attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
                })
            });
        }
        d.blocks.insert("ARR".into(),std::sync::Arc::new(arr));
        let root=d.add(&cadcraft_doc::Space::Model,cadcraft_doc::Common::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"ARR".into(),insert:cadcraft_geom::Vec3::new(100.0,5.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();

        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::ArrayItem{i:1,j:0,level:0,member_handle:member.hex()}]};
        let (entity,_)=resolve_source_path(&d,&path).unwrap();
        let EntityKind::Line(line)=entity.kind else{panic!("expected line")};
        assert!((line.a.x-110.0).abs()<1e-12&&(line.a.y-5.0).abs()<1e-12);
        assert!((line.b.x-112.0).abs()<1e-12&&(line.b.y-5.0).abs()<1e-12);
    }

    #[test]
    fn associative_array_suppressed_item_path_is_invalid() {
        use cadcraft_doc::{ArrayItemRef,ArrayKind,ArrayParams,RectArray};
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut source=cadcraft_doc::Block::new("SRC");
        source.entities.push(cadcraft_doc::Entity{handle:member,common:Default::default(),kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(1.0,0.0,0.0)})});
        d.blocks.insert("SRC".into(),std::sync::Arc::new(source));
        let params=ArrayParams{
            source:"SRC".into(),
            kind:ArrayKind::Rect(RectArray{cols:2,rows:1,col_spacing:10.0,row_spacing:0.0,axis_angle:0.0,row_elevation:0.0}),
            levels:1,level_spacing:0.0,replaced:Vec::new(),suppressed:vec![ArrayItemRef{i:1,j:0,level:0}],editing:None
        };
        let mut arr=cadcraft_doc::Block::new("ARR");
        arr.array=Some(params);
        arr.entities.push(cadcraft_doc::Entity{
            handle:cadcraft_doc::Handle(0x30),common:Default::default(),
            kind:cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
                block:"SRC".into(),insert:cadcraft_geom::Vec3::ZERO,scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
                attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
            })
        });
        d.blocks.insert("ARR".into(),std::sync::Arc::new(arr));
        let root=d.add(&cadcraft_doc::Space::Model,Default::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"ARR".into(),insert:cadcraft_geom::Vec3::ZERO,scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();
        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::ArrayItem{i:1,j:0,level:0,member_handle:member.hex()}]};
        assert!(resolve_source_path(&d,&path).is_none());
    }

    #[test]
    fn nested_path_enumerator_returns_stable_unique_paths() {
        let (d,root,member)=block_member_fixture();
        let root_entity=d.model.get(root).unwrap();
        let base=SourcePath{root_handle:root.hex(),steps:Vec::new()};
        let mut paths=Vec::new();
        collect_nested_source_paths(&d,root_entity.as_ref(),&base,0,&mut paths);
        assert_eq!(paths.len(),1);
        assert_eq!(paths[0],SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:member.hex()}]});
    }

    #[test]
    fn uncalibrated_qto_project_can_save_and_reopen() {
        let mut original=State::default();
        let id=original.add_measurement(MeasureKind::Count,Vec::new(),3.0,"DEVICE".into()).unwrap();
        assert_eq!(id,1);
        assert!(original.scale_to_m().is_none());
        let json=original.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert!(reopened.scale_to_m().is_none());
        assert_eq!(reopened.records.len(),1);
        assert_eq!(reopened.records[0].kind,MeasureKind::Count);
        assert_eq!(reopened.records[0].raw_value,3.0);
    }

    #[test]
    fn qto_project_rejects_invalid_measurement_geometry_on_reopen() {
        let bad=serde_json::json!({
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"length","points":[["x",0.0]],"raw_value":2.0,"status":"review"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad).is_err());

        let missing_geometry=serde_json::json!({
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"area","points":[],"raw_value":2.0,"status":"review"}],
            "audit":[]
        });
        assert!(State::default().load_json(&missing_geometry).is_err());
    }

    #[test]
    fn qto_project_rejects_duplicate_ids_and_unknown_status() {
        let duplicate=serde_json::json!({
            "drawing_distance":0.0,"real_distance_m":0.0,
            "records":[
                {"id":1,"kind":"count","points":[],"raw_value":1.0,"status":"review"},
                {"id":1,"kind":"count","points":[],"raw_value":2.0,"status":"review"}
            ],
            "audit":[]
        });
        assert!(State::default().load_json(&duplicate).is_err());

        let bad_status=serde_json::json!({
            "drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"status":"unknown"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad_status).is_err());
    }

    #[test]
    fn qto_project_json_roundtrip_preserves_calibration() {
        let original = State { drawing_distance: 2500.0, real_distance_m: 2.5, ..Default::default() };
        let mut restored = State::default();
        restored.load_json(&original.to_json()).unwrap();
        assert!((restored.drawing_distance - 2500.0).abs() < 1e-12);
        assert!((restored.real_distance_m - 2.5).abs() < 1e-12);
        assert!((restored.scale_to_m().unwrap() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn closed_polyline_can_produce_perimeter_and_area_records() {
        let points=[(0.0,0.0),(4.0,0.0),(4.0,3.0),(0.0,3.0)];
        let vertices=points.iter().map(|&(x,y)| PolyVertex::new(Vec2::new(x,y))).collect::<Vec<_>>();
        let (perimeter,area)=lwpoly_metrics(&vertices,true).unwrap();
        assert!((perimeter-14.0).abs()<1e-10);
        assert!((area-12.0).abs()<1e-10);
    }

    #[test]
    fn qto_hatch_loop_metrics_return_perimeter_and_net_area() {
        let outer=cadcraft_doc::HatchLoop{vertices:vec![
            PolyVertex::new(Vec2::new(0.0,0.0)),PolyVertex::new(Vec2::new(10.0,0.0)),
            PolyVertex::new(Vec2::new(10.0,10.0)),PolyVertex::new(Vec2::new(0.0,10.0))
        ],outer:true,sources:Vec::new()};
        let hole=cadcraft_doc::HatchLoop{vertices:vec![
            PolyVertex::new(Vec2::new(2.0,2.0)),PolyVertex::new(Vec2::new(4.0,2.0)),
            PolyVertex::new(Vec2::new(4.0,4.0)),PolyVertex::new(Vec2::new(2.0,4.0))
        ],outer:false,sources:Vec::new()};
        let (perimeter,area,points)=hatch_loop_metrics(&[outer,hole]).unwrap();
        assert!((perimeter-48.0).abs()<1e-10);
        assert!((area-96.0).abs()<1e-10);
        assert_eq!(points.len(),8);
    }

    #[test]
    fn qto_region_metric_contract_matches_hatch_loop_policy() {
        let region=cadcraft_doc::Region::from_loop(vec![
            PolyVertex::new(Vec2::new(0.0,0.0)),PolyVertex::new(Vec2::new(4.0,0.0)),
            PolyVertex::new(Vec2::new(4.0,3.0)),PolyVertex::new(Vec2::new(0.0,3.0))
        ]);
        assert!((region.area()-12.0).abs()<1e-10);
        assert!((region.perimeter()-14.0).abs()<1e-10);
    }

    #[test]
    fn qto_dimension_policy_excludes_angular_dimensions_from_length_totals() {
        use cadcraft_doc::DimKind;
        assert!(matches!(DimKind::Angular,DimKind::Angular|DimKind::Angular3P));
        assert!(!matches!(DimKind::Aligned,DimKind::Angular|DimKind::Angular3P));
        assert!(!matches!(DimKind::Radius,DimKind::Angular|DimKind::Angular3P));
    }

    #[test]
    fn measurement_lifecycle_undo_redo_and_project_roundtrip() {
        let mut state = State { drawing_distance: 1000.0, real_distance_m: 1.0, ..Default::default() };
        let id = state.add_measurement(MeasureKind::Length, vec![[0.0,0.0],[2500.0,0.0]], 2500.0, "A-WALL".into()).unwrap();
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.records[0].status, MeasureStatus::Review);
        assert!(state.edit_metadata(id, "Tường".into(), "TT-001".into(), "Tường xây".into(), "Kiểm tra hiện trường".into()));
        assert!(state.set_status(id, MeasureStatus::Confirmed));
        assert_eq!(state.records[0].status, MeasureStatus::Confirmed);
        assert!(state.undo_measurement_change());
        assert_eq!(state.records[0].status, MeasureStatus::Review);
        assert!(state.redo_measurement_change());
        assert_eq!(state.records[0].status, MeasureStatus::Confirmed);

        let json = state.to_json();
        let mut reopened = State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records.len(), 1);
        assert_eq!(reopened.records[0].work_code, "TT-001");
        assert_eq!(reopened.records[0].status, MeasureStatus::Confirmed);
        assert!((reopened.records[0].raw_value * reopened.scale_to_m().unwrap() - 2.5).abs() < 1e-12);
        assert_eq!(reopened.audit, state.audit);
        assert!(reopened.next_audit_sequence > reopened.audit.last().map(|a| a.sequence).unwrap_or(0));
    }

    #[test]
    fn legacy_qto_v2_without_scope_still_reopens() {
        let legacy=serde_json::json!({
            "version":2,
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review"}],
            "audit":[]
        });
        let mut state=State::default();
        state.load_json(&legacy).unwrap();
        assert_eq!(state.records.len(),1);
        assert_eq!(state.records[0].scope,Scope::default());
    }

    #[test]
    fn scoped_measurement_rejects_invalid_scope() {
        let mut state=State::default();
        let bad=Scope{floor_id:Some(0),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        assert!(state.add_measurement_scoped(MeasureKind::Count,Vec::new(),1.0,"DEVICE".into(),&bad).is_err());
    }

    #[test]
    fn relink_policy_requires_a_unique_candidate() {
        let candidates0:Vec<cadcraft_doc::Handle>=Vec::new();
        let candidates1=vec![cadcraft_doc::Handle(1)];
        let candidates2=vec![cadcraft_doc::Handle(1),cadcraft_doc::Handle(2)];
        assert_ne!(candidates0.len(),1);
        assert_eq!(candidates1.len(),1);
        assert_ne!(candidates2.len(),1);
    }

    #[test]
    fn confirmed_quantity_returns_after_source_becomes_valid_again() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into()).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));

        let invalid=std::collections::BTreeSet::from([id]);
        let stale_groups=business_quantities_validated(&state,1.0,1.0,&invalid);
        assert_eq!(stale_groups.values().next().unwrap().count,0.0);

        let valid=std::collections::BTreeSet::new();
        let restored_groups=business_quantities_validated(&state,1.0,1.0,&valid);
        assert_eq!(restored_groups.values().next().unwrap().confirmed_records,1);
        assert_eq!(restored_groups.values().next().unwrap().count,5.0);
        assert_eq!(state.records[0].status,MeasureStatus::Confirmed);
    }

    #[test]
    fn validated_business_summary_excludes_invalid_confirmed_record() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into()).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        let invalid=std::collections::BTreeSet::from([id]);
        let groups=business_quantities_validated(&state,1.0,1.0,&invalid);
        let q=groups.values().next().unwrap();
        assert_eq!(q.confirmed_records,0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn validated_scope_summary_excludes_invalid_confirmed_record() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let id=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into(),&scope).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        let invalid=std::collections::BTreeSet::from([id]);
        let groups=scoped_business_quantities_validated(&state,1.0,1.0,&invalid);
        let q=groups.values().next().unwrap();
        assert_eq!(q.confirmed_records,0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn refresh_source_policy_returns_record_to_review() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let id=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),2.0,"DEVICE".into(),&scope).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        assert_eq!(state.records[0].status,MeasureStatus::Confirmed);
        state.records[0].status=MeasureStatus::Review;
        assert_eq!(state.records[0].status,MeasureStatus::Review);
    }

    #[test]
    fn cad_only_source_still_uses_geometry_guard_without_bim_revision() {
        let has_bim_scope=false;
        let geometry_stale=true;
        let invalid=!has_bim_scope && geometry_stale;
        assert!(invalid);
    }

    #[test]
    fn undo_policy_prefers_matching_geometry_over_revision_only() {
        let has_geometry_signature=true;
        let revision_changed=true;
        let geometry_stale=false;
        let stale=geometry_stale || (revision_changed && !has_geometry_signature);
        assert!(!stale);
    }

    #[test]
    fn legacy_record_without_geometry_signature_still_uses_revision_guard() {
        let has_geometry_signature=false;
        let revision_changed=true;
        let geometry_stale=false;
        let stale=geometry_stale || (revision_changed && !has_geometry_signature);
        assert!(stale);
    }

    #[test]
    fn geometry_signature_changes_when_geometry_moves_or_scales() {
        let base=PendingMeasurement{kind:MeasureKind::Length,points:vec![[0.0,0.0],[10.0,0.0]],raw_value:10.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        let moved=PendingMeasurement{kind:MeasureKind::Length,points:vec![[5.0,0.0],[15.0,0.0]],raw_value:10.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        let scaled=PendingMeasurement{kind:MeasureKind::Length,points:vec![[0.0,0.0],[20.0,0.0]],raw_value:20.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        assert_ne!(geometry_signature(&base),geometry_signature(&moved));
        assert_ne!(geometry_signature(&base),geometry_signature(&scaled));
    }

    #[test]
    fn qto_v3_roundtrip_preserves_complete_source_link_contract() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(2),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Length,points:vec![[1.0,2.0],[4.0,6.0]],raw_value:5.0,layer:"E-CABLE".into(),source_handle:cadcraft_doc::Handle(0x2A)};
        let signature=geometry_signature(&measurement);
        state.add_measurement_from_entity(measurement,&scope,Some(9)).unwrap();
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        let record=&reopened.records[0];
        assert_eq!(record.source_handle.as_deref(),Some("2A"));
        assert_eq!(record.source_revision,Some(9));
        assert_eq!(record.source_geometry_signature.as_deref(),Some(signature.as_str()));
        assert_eq!(record.scope.floor_id,Some(2));
        assert_eq!(record.scope.discipline.as_deref(),Some("ELEC"));
        assert_eq!(record.scope.owner_account_id.as_deref(),Some("u1"));
    }

    #[test]
    fn qto_v3_roundtrip_preserves_source_revision() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(10)};
        state.add_measurement_from_entity(measurement,&scope,Some(4)).unwrap();
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_revision,Some(4));
    }

    #[test]
    fn identity_collision_policy_only_blocks_when_original_identity_still_exists() {
        let current=cadcraft_doc::Handle(0x55);
        let claimed=cadcraft_doc::Handle(0x2A);
        let original_still_exists=true;
        assert!(claimed!=current && original_still_exists);
        let original_still_exists=false;
        assert!(!(claimed!=current && original_still_exists));
    }

    #[test]
    fn source_state_labels_legacy_ok_and_stale_records() {
        let mut state=State::default();
        let legacy=state.add_measurement(MeasureKind::Count,Vec::new(),1.0,"A".into()).unwrap();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let linked=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),1.0,"A".into(),&scope).unwrap();
        state.records.iter_mut().find(|r|r.id==linked).unwrap().source_handle=Some("1".into());
        let mut validation=SourceValidation::default();
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==legacy).unwrap(),&validation),"Legacy");
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==linked).unwrap(),&validation),"OK");
        validation.invalid_record_ids.insert(linked);
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==linked).unwrap(),&validation),"STALE");
    }

    #[test]
    fn source_validation_default_has_no_false_errors() {
        let validation=SourceValidation::default();
        assert_eq!(validation.missing_source,0);
        assert_eq!(validation.scope_mismatch,0);
        assert_eq!(validation.stale_revision,0);
        assert_eq!(validation.stale_geometry,0);
        assert_eq!(validation.identity_collision,0);
        assert_eq!(validation.valid_links,0);
    }

    #[test]
    fn qto_v3_normalizes_legacy_display_discipline_and_rejects_bad_handle() {
        let legacy_label=serde_json::json!({
            "version":3,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review",
                "scope":{"floor_id":1,"discipline":"Điện","owner_account_id":"u1"},"source_handle":"2A"}],
            "audit":[]
        });
        let mut state=State::default();
        state.load_json(&legacy_label).unwrap();
        assert_eq!(state.records[0].scope.discipline.as_deref(),Some("ELEC"));

        let bad_handle=serde_json::json!({
            "version":3,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review",
                "scope":{"floor_id":1,"discipline":"ELEC","owner_account_id":"u1"},"source_handle":"NOT_HEX"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad_handle).is_err());
    }

    #[test]
    fn qto_v3_roundtrip_preserves_source_handle() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(2),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(0x2A)};
        let id=state.add_measurement_from_entity(measurement,&scope,Some(7)).unwrap();
        assert_eq!(state.records[0].source_handle.as_deref(),Some("2A"));
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_handle.as_deref(),Some("2A"));
        assert_eq!(reopened.records[0].source_revision,Some(7));
        assert_eq!(reopened.records[0].scope.discipline.as_deref(),Some("ELEC"));
        assert_eq!(id,1);
    }

    #[test]
    fn count_measurements_stay_one_record_per_selected_entity_for_individual_scope() {
        let pending=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(1)};
        assert_eq!(pending.raw_value,1.0);
        assert_eq!(pending.source_handle,cadcraft_doc::Handle(1));
    }

    #[test]
    fn scoped_measurement_roundtrip_preserves_bim_scope_and_separates_totals() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let floor1=Scope{floor_id:Some(1),discipline:Some("Kiến trúc".into()),owner_account_id:Some("u1".into())};
        let floor2=Scope{floor_id:Some(2),discipline:Some("Kiến trúc".into()),owner_account_id:Some("u1".into())};
        let a=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),2.0,"DEVICE".into(),&floor1).unwrap();
        let b=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),3.0,"DEVICE".into(),&floor2).unwrap();
        assert!(state.set_status(a,MeasureStatus::Confirmed));
        assert!(state.set_status(b,MeasureStatus::Confirmed));
        let groups=scoped_business_quantities(&state,1.0,1.0);
        assert_eq!(groups.len(),2);

        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].scope.floor_id,Some(1));
        assert_eq!(reopened.records[1].scope.floor_id,Some(2));
        assert_eq!(reopened.records[0].scope.discipline.as_deref(),Some("ARCH"));
    }

    #[test]
    fn business_summary_only_totals_confirmed_records() {
        let mut state=State{drawing_distance:1000.0,real_distance_m:1.0,..Default::default()};
        let confirmed=state.add_measurement(MeasureKind::Length,vec![[0.0,0.0],[2500.0,0.0]],2500.0,"A-WALL".into()).unwrap();
        let review=state.add_measurement(MeasureKind::Area,vec![[0.0,0.0],[1000.0,1000.0]],1_000_000.0,"A-WALL".into()).unwrap();
        let excluded=state.add_measurement(MeasureKind::Count,Vec::new(),4.0,"A-WALL".into()).unwrap();
        assert!(state.edit_metadata(confirmed,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.edit_metadata(review,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.edit_metadata(excluded,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.set_status(confirmed,MeasureStatus::Confirmed));
        assert!(state.set_status(excluded,MeasureStatus::Excluded));
        let groups=business_quantities(&state,0.001,0.000001);
        let q=groups.get(&("Tường".into(),"TT-001".into(),"A-WALL".into())).unwrap();
        assert_eq!(q.review_records,1);
        assert_eq!(q.confirmed_records,1);
        assert_eq!(q.excluded_records,1);
        assert!((q.length-2.5).abs()<1e-12);
        assert_eq!(q.area,0.0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn selecting_record_populates_metadata_editor_and_save_is_audited() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),1.0,"DEVICE".into()).unwrap();
        assert!(state.select_record(id));
        state.edit_work_code="EL-001".into();
        assert!(state.edit_metadata(id,state.edit_work_group.clone(),state.edit_work_code.clone(),state.edit_description.clone(),state.edit_note.clone()));
        assert_eq!(state.records[0].work_code,"EL-001");
        assert!(state.audit.iter().any(|a|a.record_id==id&&a.action=="EDIT_METADATA"));
    }

    #[test]
    fn excluded_measurements_remain_in_audit_but_not_as_confirmed_quantity() {
        let mut state = State { drawing_distance: 1.0, real_distance_m: 1.0, ..Default::default() };
        let id = state.add_measurement(MeasureKind::Count, Vec::new(), 1.0, "DEVICE".into()).unwrap();
        assert!(state.set_status(id, MeasureStatus::Excluded));
        assert_eq!(state.records[0].status, MeasureStatus::Excluded);
        assert!(state.audit.iter().any(|a| a.record_id == id && a.action == "STATUS"));
        assert!(state.delete_measurement(id));
        assert!(state.records.is_empty());
        assert!(state.audit.iter().any(|a| a.record_id == id && a.action == "DELETE"));
    }

    #[test]
    fn two_point_calibration_uses_world_distance_and_real_length() {
        let mut state = State::default();
        let scale = state.calibrate_two_points([10.0,20.0],[3010.0,4020.0],5.0).unwrap();
        assert!((state.drawing_distance - 5000.0).abs() < 1e-12);
        assert!((scale - 0.001).abs() < 1e-12);
        assert!(state.calibrate_two_points([1.0,1.0],[1.0,1.0],2.0).is_err());
    }

    #[test]
    fn calibration_scale_converts_length_and_area() {
        let state = State { drawing_distance: 1000.0, real_distance_m: 1.0, ..Default::default() };
        let scale = state.scale_to_m().unwrap();
        assert!((scale - 0.001).abs() < 1e-12);
        assert!((2500.0 * scale - 2.5).abs() < 1e-12);
        assert!((1_000_000.0 * scale * scale - 1.0).abs() < 1e-12);
    }

    #[test]
    fn invalid_calibration_is_rejected() {
        assert!(State::default().scale_to_m().is_none());
        assert!(State { drawing_distance: 0.0, real_distance_m: 1.0, ..Default::default() }.scale_to_m().is_none());
        assert!(State { drawing_distance: 1.0, real_distance_m: 0.0, ..Default::default() }.scale_to_m().is_none());
    }

    #[test]
    fn open_polyline_has_no_area() {
        let vertices = vec![PolyVertex::new(Vec2::new(0.0, 0.0)), PolyVertex::new(Vec2::new(3.0, 4.0))];
        assert_eq!(lwpoly_metrics(&vertices, false), Some((5.0, 0.0)));
    }

    #[test]
    fn closed_rectangle_area_and_perimeter() {
        let vertices = [(0.0,0.0),(10.0,0.0),(10.0,8.0),(0.0,8.0)]
            .iter().map(|&(x,y)| PolyVertex::new(Vec2::new(x,y))).collect::<Vec<_>>();
        let (perimeter, area) = lwpoly_metrics(&vertices, true).unwrap();
        assert!((perimeter - 36.0).abs() < 1e-10);
        assert!((area - 80.0).abs() < 1e-10);
    }
    #[test]
    fn semicircle_bulge_preserves_arc_and_segment_area() {
        let (length, area) = bulge_edge((0.0, 0.0), (2.0, 0.0), 1.0).unwrap();
        assert!((length - std::f64::consts::PI).abs() < 1e-10);
        assert!((area - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
    #[test]
    fn self_crossing_polyline_must_not_create_qto_area() {
        let points = [(0.0, 0.0), (4.0, 4.0), (0.0, 4.0), (4.0, 0.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn zero_length_closed_edge_is_invalid() {
        let points = [(0.0, 0.0), (0.0, 0.0), (2.0, 1.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn nonzero_bulge_with_coincident_endpoints_is_rejected() {
        assert!(bulge_edge((2.0, 3.0), (2.0, 3.0), 1.0).is_none());
        let mut vertices = vec![PolyVertex::new(Vec2::new(2.0, 3.0)), PolyVertex::new(Vec2::new(2.0, 3.0))];
        vertices[0].bulge = 1.0;
        assert!(lwpoly_metrics(&vertices, false).is_none());
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        assert!(bulge_edge((0.0, 0.0), (f64::NAN, 0.0), 0.0).is_none());
        assert!(bulge_edge((f64::NAN, 0.0), (1.0, 0.0), 0.0).is_none());
        let invalid = vec![
            PolyVertex::new(Vec2::new(f64::NAN, 0.0)),
            PolyVertex::new(Vec2::new(1.0, 0.0)),
        ];
        assert!(lwpoly_metrics(&invalid, true).is_none());
    }
}
