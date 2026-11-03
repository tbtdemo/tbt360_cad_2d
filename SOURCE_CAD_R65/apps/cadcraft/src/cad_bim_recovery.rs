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
    pub error: Option<String>,
}

impl Recovery {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root, last_save: Instant::now(), last_signature: None, error: None }
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
            Ok(()) => { self.last_signature = Some(signature); self.error = None; }
            Err(error) => { log::error!("CAD+BIM recovery autosave: {error}"); self.error = Some(error); }
        }
        ctx.request_repaint_after(INTERVAL);
    }

    pub fn has_recovery(&self) -> bool {
        self.root.as_deref().is_some_and(|root| latest_pair(root).is_some())
    }

    pub fn restore_latest(&mut self, app: &mut CadApp, bim: &mut bim_collab::State) -> Result<(), String> {
        let root = self.root.as_deref().ok_or("Chưa xác định được thư mục phục hồi")?;
        let pair = latest_pair(root).ok_or("Không tìm thấy bản phục hồi CAD+BIM hợp lệ")?;
        let (drawing_path, metadata) = check_pair(&pair)?;
        // Validate BIM before opening CAD geometry; never overwrite newer unsynced BIM edits.
        let mut proposed = bim_collab::State::default();
        proposed.load_snapshot_json(&metadata)?;
        if bim.pending_changes.iter().any(|change| !change.synced) {
            return Err("Có ChangeSet BIM chưa đồng bộ. Hãy lưu bản hiện tại trước khi phục hồi.".into());
        }
        let title = format!("CAD+BIM phục hồi {}", pair.file_name().unwrap_or_default().to_string_lossy());
        app.run("recover", serde_json::json!({ "path": drawing_path.to_string_lossy().to_string(), "copy": true, "title": title }))?;
        *bim = proposed;
        self.last_signature = None;
        Ok(())
    }

    pub fn save_now(&mut self, app: &CadApp, bim: &bim_collab::State) -> Result<(), String> {
        let root = self.root.as_deref().ok_or("Chưa có thư mục tự động lưu CAD+BIM")?;
        save_pair(root, app, bim)?;
        self.last_save = Instant::now();
        self.last_signature = app.session.docs.get(app.session.active).map(|doc| (doc.uid, doc.revision, bim.dirty_local_changes));
        self.error = None;
        Ok(())
    }
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
    let drawing = app.session.doc().map_err(|e| format!("Không có bản vẽ CAD: {e}"))?;
    let dxf = cadcraft_io::write(drawing, "cad360_recovery.dxf").map_err(|e| format!("Không xuất được DXF: {e}"))?;
    if dxf.is_empty() || dxf.len() as u64 > MAX_RECOVERY_BYTES { return Err("Kích thước hình học DXF không hợp lệ".into()); }
    let workspace = serde_json::to_vec_pretty(&bim.snapshot_json()).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let name = new_generation_name()?;
    let pending = root.join(format!("{name}.pending"));
    let committed = root.join(name);
    std::fs::create_dir(&pending).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        write_synced(&pending.join("drawing.dxf"), &dxf)?;
        write_synced(&pending.join("workspace.json"), &workspace)?;
        let manifest = serde_json::json!({
            "format": 1, "drawing_bytes": dxf.len(), "workspace_bytes": workspace.len(),
            "drawing_checksum": content_checksum(&dxf), "workspace_checksum": content_checksum(&workspace),
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

fn check_pair(dir: &Path) -> Result<(PathBuf, serde_json::Value), String> {
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
    generations(root).into_iter().find(|dir| check_pair(dir).is_ok())
}

fn prune_generations(root: &Path) {
    for stale in generations(root).into_iter().skip(MAX_GENERATIONS) {
        let _ = std::fs::remove_dir_all(stale);
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
    fn metadata_pair_rejects_missing_geometry() {
        let dir = std::env::temp_dir().join(format!("tbt-paired-recovery-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("commit.json"), br#"{"format":1,"drawing_bytes":12,"workspace_bytes":10}"#).unwrap();
        assert!(check_pair(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
