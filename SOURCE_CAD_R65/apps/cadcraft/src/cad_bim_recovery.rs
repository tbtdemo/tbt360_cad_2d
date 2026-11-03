//! Paired periodic recovery of CAD geometry and BIM workspace.
//! A completed generation is a directory containing drawing.dxf, workspace.json and commit.json.
//! Incomplete generations are never offered for recovery.
use std::{path::{Path, PathBuf}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use cadcraft_ui_egui::CadApp;
use crate::bim_collab;

const INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_GENERATIONS: usize = 3;
const MAX_RECOVERY_BYTES: u64 = 512 * 1024 * 1024;

pub struct Recovery {
    root: Option<PathBuf>,
    last_save: Instant,
    last_signature: Option<(u64, u64, usize)>,
    available: bool,
    pub error: Option<String>,
}

impl Recovery {
    pub fn new(root: Option<PathBuf>) -> Self {
        let available = root.as_deref().is_some_and(|dir| latest_pair(dir).is_some());
        Self { root, last_save: Instant::now(), last_signature: None, available, error: None }
    }

    pub fn tick(&mut self, app: &CadApp, bim: &bim_collab::State, ctx: &egui::Context) {
        let elapsed = self.last_save.elapsed();
        if elapsed < INTERVAL {
            ctx.request_repaint_after(INTERVAL - elapsed);
            return;
        }
        self.last_save = Instant::now();
        let Some(root) = self.root.as_deref() else { return };
        let Some(doc) = app.session.docs.get(app.session.active) else { return };
        let signature = (doc.uid, doc.revision, bim.dirty_local_changes);
        match save_pair(root, app, bim) {
            Ok(()) => { self.last_signature = Some(signature); self.available = true; self.error = None; }
            Err(error) => { log::error!("CAD+BIM recovery autosave: {error}"); self.error = Some(error); }
        }
        ctx.request_repaint_after(INTERVAL);
    }

    pub fn has_recovery(&self) -> bool {
        self.available
    }

    pub fn restore_latest(&mut self, app: &mut CadApp, bim: &mut bim_collab::State) -> Result<(), String> {
        let root = self.root.as_deref().ok_or("Chưa xác định được thư mục phục hồi")?;
        let pair = latest_pair(root).ok_or("Không tìm thấy bản phục hồi CAD+BIM hợp lệ")?;
        let (drawings, metadata, active) = check_bundle(&pair)?;
        // Validate every drawing before opening any tab. A valid checksum alone is insufficient.
        for drawing_path in &drawings {
            let geometry = std::fs::read(drawing_path).map_err(|e| format!("Không đọc được DXF phục hồi: {e}"))?;
            validate_recovery_dxf(&geometry)?;
        }
        let mut proposed = bim_collab::State::default();
        proposed.load_snapshot_json(&metadata)?;
        if bim.pending_changes.iter().any(|change| !change.synced) {
            return Err("Có ChangeSet BIM chưa đồng bộ. Hãy lưu bản hiện tại trước khi phục hồi.".into());
        }
        // Recovery is append-only: preserve original tabs and selection if any later import fails.
        let original_count = app.session.docs.len();
        let original_active = app.session.active;
        let result = (|| -> Result<(), String> {
            for (index, drawing_path) in drawings.iter().enumerate().filter(|(index, _)| *index != active)
                .chain(drawings.iter().enumerate().filter(|(index, _)| *index == active)) {
                let title = format!("CAD+BIM phục hồi {} - bản vẽ {}", pair.file_name().unwrap_or_default().to_string_lossy(), index + 1);
                app.run("recover", serde_json::json!({ "path": drawing_path.to_string_lossy().to_string(), "copy": true, "title": title }))?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            app.session.docs.truncate(original_count);
            app.session.active = original_active;
            return Err(format!("Phục hồi CAD+BIM thất bại, đã đóng các tab mới: {error}"));
        }
        *bim = proposed;
        self.last_signature = None;
        Ok(())
    }

    pub fn save_now(&mut self, app: &CadApp, bim: &bim_collab::State) -> Result<(), String> {
        let root = self.root.as_deref().ok_or("Chưa có thư mục tự động lưu CAD+BIM")?;
        save_pair(root, app, bim)?;
        self.last_save = Instant::now();
        self.last_signature = app.session.docs.get(app.session.active).map(|doc| (doc.uid, doc.revision, bim.dirty_local_changes));
        self.available = true;
        self.error = None;
        Ok(())
    }
}

fn validate_recovery_dxf(bytes: &[u8]) -> Result<(), String> {
    cadcraft_io::read(bytes, "cad360_recovery.dxf")
        .map(|_| ())
        .map_err(|error| format!("Hình học DXF không đọc được: {error}"))
}

fn content_checksum(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn new_generation_name() -> Result<String, String> {
    let ns = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    Ok(format!("pair-{}-{ns}", std::process::id()))
}

fn save_pair(root: &Path, app: &CadApp, bim: &bim_collab::State) -> Result<(), String> {
    if app.session.docs.is_empty() { return Err("Không có bản vẽ CAD đang mở".into()); }
    if app.session.docs.len() > 256 { return Err("Một phiên phục hồi hỗ trợ tối đa 256 bản vẽ CAD".into()); }
    if app.session.active >= app.session.docs.len() { return Err("Chỉ số bản vẽ đang hoạt động không hợp lệ".into()); }
    let mut drawings = Vec::with_capacity(app.session.docs.len());
    let mut total_bytes = 0u64;
    for (index, doc) in app.session.docs.iter().enumerate() {
        let name = format!("drawing-{index:04}.dxf");
        let dxf = cadcraft_io::write(doc, "cad360_recovery.dxf").map_err(|e| format!("Không xuất được DXF bản vẽ {}: {e}", index + 1))?;
        total_bytes = total_bytes.checked_add(dxf.len() as u64).ok_or("Tổng dung lượng CAD vượt giới hạn")?;
        if dxf.is_empty() || total_bytes > MAX_RECOVERY_BYTES { return Err("Tổng dung lượng CAD phục hồi không hợp lệ hoặc vượt 512 MB".into()); }
        drawings.push((name, dxf, doc.uid, doc.revision));
    }
    let workspace = serde_json::to_vec_pretty(&bim.snapshot_json()).map_err(|e| e.to_string())?;
    if workspace.is_empty() || workspace.len() > 64 * 1024 * 1024 { return Err("Workspace BIM vượt giới hạn phục hồi 64 MiB".into()); }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let name = new_generation_name()?;
    let pending = root.join(format!("{name}.pending"));
    let committed = root.join(name);
    std::fs::create_dir(&pending).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        let mut documents = Vec::with_capacity(drawings.len());
        for (name, dxf, uid, revision) in &drawings {
            write_synced(&pending.join(name), dxf)?;
            documents.push(serde_json::json!({
                "file": name, "bytes": dxf.len(), "checksum": content_checksum(dxf),
                "uid": uid, "revision": revision
            }));
        }
        write_synced(&pending.join("workspace.json"), &workspace)?;
        let manifest = serde_json::json!({
            "format": 2, "documents": documents, "active": app.session.active,
            "workspace_bytes": workspace.len(), "workspace_checksum": content_checksum(&workspace),
            "project_id": bim.project_id,
            "created_at_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis()
        });
        write_synced(&pending.join("commit.json"), &serde_json::to_vec(&manifest).map_err(|e| e.to_string())?)?;
        std::fs::rename(&pending, &committed).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() { let _ = std::fs::remove_dir_all(&pending); }
    result?;
    prune_generations(root);
    Ok(())
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

fn check_bundle(dir: &Path) -> Result<(Vec<PathBuf>, serde_json::Value, usize), String> {
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("commit.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let format = manifest.get("format").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if format == 1 {
        let (path, workspace) = check_pair_v1(dir)?;
        return Ok((vec![path], workspace, 0));
    }
    if format != 2 { return Err("Định dạng phục hồi không hợp lệ".into()); }
    let docs = manifest.get("documents").and_then(serde_json::Value::as_array).ok_or("Thiếu danh sách bản vẽ CAD")?;
    if docs.is_empty() || docs.len() > 256 { return Err("Số bản vẽ CAD không hợp lệ".into()); }
    let active = manifest.get("active").and_then(serde_json::Value::as_u64).ok_or("Thiếu chỉ số bản vẽ đang hoạt động")? as usize;
    if active >= docs.len() { return Err("Chỉ số bản vẽ đang hoạt động vượt giới hạn".into()); }
    let mut paths = Vec::with_capacity(docs.len());
    let mut total = 0u64;
    for (index, doc) in docs.iter().enumerate() {
        let expected_name = format!("drawing-{index:04}.dxf");
        if doc.get("file").and_then(serde_json::Value::as_str) != Some(expected_name.as_str()) { return Err("Tên file CAD không khớp manifest".into()); }
        let path = dir.join(expected_name);
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        total = total.checked_add(bytes.len() as u64).ok_or("Dung lượng CAD vượt giới hạn")?;
        if bytes.is_empty() || total > MAX_RECOVERY_BYTES || doc.get("bytes").and_then(serde_json::Value::as_u64) != Some(bytes.len() as u64) || doc.get("checksum").and_then(serde_json::Value::as_str) != Some(content_checksum(&bytes).as_str()) {
            return Err("Một bản vẽ CAD bị thiếu hoặc sai checksum".into());
        }
        paths.push(path);
    }
    let workspace_bytes = std::fs::read(dir.join("workspace.json")).map_err(|e| e.to_string())?;
    if workspace_bytes.is_empty() || workspace_bytes.len() > 64 * 1024 * 1024 ||
        manifest.get("workspace_bytes").and_then(serde_json::Value::as_u64) != Some(workspace_bytes.len() as u64) ||
        manifest.get("workspace_checksum").and_then(serde_json::Value::as_str) != Some(content_checksum(&workspace_bytes).as_str()) {
        return Err("Metadata BIM không đầy đủ hoặc sai checksum".into());
    }
    let workspace: serde_json::Value = serde_json::from_slice(&workspace_bytes).map_err(|e| e.to_string())?;
    if workspace.get("project_id") != manifest.get("project_id") || workspace.get("version").and_then(serde_json::Value::as_u64) != Some(2) {
        return Err("Dữ liệu BIM và bản CAD không cùng snapshot".into());
    }
    Ok((paths, workspace, active))
}

fn check_pair(dir: &Path) -> Result<(PathBuf, serde_json::Value), String> {
    let (drawings, workspace, active) = check_bundle(dir)?;
    Ok((drawings[active].clone(), workspace))
}

fn check_pair_v1(dir: &Path) -> Result<(PathBuf, serde_json::Value), String> {
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("commit.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    if manifest.get("format").and_then(serde_json::Value::as_u64) != Some(1) { return Err("Định dạng phục hồi không hợp lệ".into()); }
    let drawing_path = dir.join("drawing.dxf");
    let actual_dxf = std::fs::metadata(&drawing_path).map_err(|e| e.to_string())?.len();
    let expected_dxf = manifest.get("drawing_bytes").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if actual_dxf == 0 || actual_dxf > MAX_RECOVERY_BYTES || actual_dxf != expected_dxf { return Err("Hình học CAD không đầy đủ".into()); }
    let metadata_path = dir.join("workspace.json");
    let actual_json = std::fs::metadata(&metadata_path).map_err(|e| e.to_string())?.len();
    let expected_json = manifest.get("workspace_bytes").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if actual_json == 0 || actual_json > 64 * 1024 * 1024 || actual_json != expected_json { return Err("Metadata BIM không đầy đủ".into()); }
    let drawing_bytes = std::fs::read(&drawing_path).map_err(|e| e.to_string())?;
    let workspace_bytes = std::fs::read(metadata_path).map_err(|e| e.to_string())?;
    if manifest.get("drawing_checksum").and_then(serde_json::Value::as_str) != Some(content_checksum(&drawing_bytes).as_str())
        || manifest.get("workspace_checksum").and_then(serde_json::Value::as_str) != Some(content_checksum(&workspace_bytes).as_str()) {
        return Err("Checksum bản phục hồi CAD+BIM không khớp".into());
    }
    let workspace: serde_json::Value = serde_json::from_slice(&workspace_bytes).map_err(|e| e.to_string())?;
    if workspace.get("project_id") != manifest.get("project_id") || workspace.get("version").and_then(serde_json::Value::as_u64) != Some(2) { return Err("Dữ liệu BIM và bản CAD không cùng snapshot".into()); }
    Ok((drawing_path, workspace))
}

fn generations(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else { return Vec::new() };
    let mut dirs: Vec<PathBuf> = entries.flatten().filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("pair-") && !entry.file_name().to_string_lossy().ends_with(".pending"))
        .map(|entry| entry.path()).collect();
    dirs.sort_by(|a, b| {
        let time = |path: &PathBuf| std::fs::metadata(path).and_then(|meta| meta.modified()).unwrap_or(UNIX_EPOCH);
        time(b).cmp(&time(a)).then_with(|| b.file_name().cmp(&a.file_name()))
    });
    dirs
}

fn latest_pair(root: &Path) -> Option<PathBuf> {
    generations(root).into_iter().find(|dir| check_bundle(dir).is_ok())
}

fn prune_generations(root: &Path) {
    // Only count verified pairs toward the retention limit. Corrupted/new incomplete
    // folders must never displace the last recoverable CAD geometry and BIM state.
    let mut verified = 0usize;
    for candidate in generations(root) {
        if check_bundle(&candidate).is_err() { continue; }
        verified += 1;
        if verified > MAX_GENERATIONS {
            if let Err(error) = std::fs::remove_dir_all(&candidate) {
                log::warn!("Cannot prune old CAD+BIM recovery {}: {error}", candidate.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_generation_is_never_offered_for_recovery() {
        let dir = std::env::temp_dir().join(format!("tbt-paired-recovery-test-{}", std::process::id()));
        let pending = dir.join("pair-1.pending");
        std::fs::create_dir_all(&pending).unwrap();
        std::fs::write(pending.join("drawing.dxf"), b"partial").unwrap();
        assert!(latest_pair(&dir).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn complete_pair_can_be_validated_and_corruption_is_detected() {
        let root = std::env::temp_dir().join(format!("tbt-pair-integrity-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let pair = root.join("pair-01");
        std::fs::create_dir_all(&pair).unwrap();
        let geometry = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
        let metadata = br#"{"version":2,"project_id":"unit-test"}"#;
        std::fs::write(pair.join("drawing.dxf"), geometry).unwrap();
        std::fs::write(pair.join("workspace.json"), metadata).unwrap();
        let manifest = serde_json::json!({
            "format":1,"drawing_bytes":geometry.len(),"workspace_bytes":metadata.len(),
            "drawing_checksum":content_checksum(geometry),"workspace_checksum":content_checksum(metadata),
            "project_id":"unit-test"
        });
        std::fs::write(pair.join("commit.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(check_pair(&pair).is_ok());
        assert_eq!(latest_pair(&root).as_deref(), Some(pair.as_path()));
        std::fs::write(pair.join("drawing.dxf"), b"changed contents that should not be trusted").unwrap();
        assert!(check_pair(&pair).is_err());
        assert!(latest_pair(&root).is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    fn corrupted_newer_generations_do_not_prune_last_good_backup() {
        let root = std::env::temp_dir().join(format!("tbt-pair-prune-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&root).unwrap();
        let good = root.join("pair-good");
        std::fs::create_dir(&good).unwrap();
        let geometry = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
        let metadata = br#"{"version":2,"project_id":"keep-good"}"#;
        std::fs::write(good.join("drawing.dxf"), geometry).unwrap();
        std::fs::write(good.join("workspace.json"), metadata).unwrap();
        let manifest = serde_json::json!({
            "format": 1, "drawing_bytes": geometry.len(), "workspace_bytes": metadata.len(),
            "drawing_checksum": content_checksum(geometry), "workspace_checksum": content_checksum(metadata),
            "project_id": "keep-good"
        });
        std::fs::write(good.join("commit.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
        for n in 0..6 {
            let broken = root.join(format!("pair-broken-{n}"));
            std::fs::create_dir(&broken).unwrap();
            std::fs::write(broken.join("commit.json"), b"invalid").unwrap();
        }
        prune_generations(&root);
        assert!(good.exists(), "a valid recovery must not be pruned by malformed generations");
        assert_eq!(latest_pair(&root).as_deref(), Some(good.as_path()));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn invalid_dxf_cannot_be_accepted_for_geometry_recovery() {
        let invalid = b"this is not a DXF drawing at all";
        assert!(validate_recovery_dxf(invalid).is_err());
    }
    #[test]
    fn metadata_pair_rejects_missing_geometry() {
        let dir = std::env::temp_dir().join(format!("tbt-paired-recovery-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("commit.json"), br#"{"format":1,"drawing_bytes":12,"workspace_bytes":10}"#).unwrap();
        assert!(check_pair(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn multi_drawing_bundle_validates_every_tab_and_active_selection() {
        let root = std::env::temp_dir().join(format!("tbt-multi-recovery-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let pair = root.join("pair-multi");
        std::fs::create_dir_all(&pair).unwrap();
        let geometry = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
        let workspace = br#"{"version":2,"project_id":"multi-test"}"#;
        let docs: Vec<_> = (0..3).map(|index| {
            let name = format!("drawing-{index:04}.dxf");
            std::fs::write(pair.join(&name), geometry).unwrap();
            serde_json::json!({"file":name,"bytes":geometry.len(),"checksum":content_checksum(geometry),"uid":index + 100,"revision":index})
        }).collect();
        std::fs::write(pair.join("workspace.json"), workspace).unwrap();
        let manifest = serde_json::json!({
            "format":2, "documents":docs, "active":1,
            "workspace_bytes":workspace.len(), "workspace_checksum":content_checksum(workspace),
            "project_id":"multi-test"
        });
        std::fs::write(pair.join("commit.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
        let (paths, metadata, active) = check_bundle(&pair).unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(active, 1);
        assert_eq!(metadata["project_id"], "multi-test");
        assert_eq!(check_pair(&pair).unwrap().0, pair.join("drawing-0001.dxf"));
        std::fs::write(pair.join("drawing-0002.dxf"), b"corrupted").unwrap();
        assert!(check_bundle(&pair).is_err(), "corruption in an inactive tab must invalidate the whole snapshot");
        assert!(latest_pair(&root).is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }

}
