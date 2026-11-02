//! BIM collaboration sync core.
//! Network transport is intentionally abstract so the R65 dependency graph stays pinned.
//! A future TBT Gateway adapter can implement this trait with the project's approved HTTP stack.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConflict {
    pub source_handle: String,
    pub base_revision: u64,
    pub local_revision: u64,
    pub remote_revision: u64,
    pub remote_updated_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushResult {
    pub accepted_change_set_ids: Vec<u64>,
    pub project_revision: u64,
    pub conflicts: Vec<RemoteConflict>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bootstrap {
    pub project_id: String,
    pub project_revision: u64,
    pub floors: Vec<FloorDto>,
    pub permissions: Vec<PermissionDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloorDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub elevation_m: f64,
    pub height_m: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionDto {
    pub account_id: String,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub can_view: bool,
    pub can_edit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresenceDto {
    pub account_id: String,
    pub floor_id: String,
    pub discipline: String,
    pub selected_object_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseResult {
    pub granted: bool,
    pub owner_account_id: Option<String>,
    pub expires_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEnvelope {
    pub id: u64,
    pub account_id: String,
    pub device_id: String,
    pub base_project_revision: u64,
    pub items: Vec<ChangeEnvelopeItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEnvelopeItem {
    pub source_handle: String,
    pub operation: String,
    pub base_revision: u64,
    pub local_revision: u64,
    pub payload_json: String,
}

pub trait CollaborationTransport {
    fn bootstrap(&mut self, project_id: &str) -> Result<Bootstrap, String>;
    fn pull_changes(&mut self, project_id: &str, after_revision: u64) -> Result<(u64, Vec<ChangeEnvelope>), String>;
    fn push_changes(&mut self, project_id: &str, changes: &[ChangeEnvelope]) -> Result<PushResult, String>;
    fn update_presence(&mut self, project_id: &str, presence: &PresenceDto) -> Result<(), String>;
    fn acquire_lease(&mut self, project_id: &str, source_handle: &str, revision: u64) -> Result<LeaseResult, String>;
    fn release_lease(&mut self, project_id: &str, source_handle: &str) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncHealth {
    #[default]
    Offline,
    Healthy,
    Pending,
    Conflict,
    Error,
}

#[derive(Clone, Debug, Default)]
pub struct SyncRuntime {
    pub project_revision: u64,
    pub last_pull_revision: u64,
    pub last_error: Option<String>,
    pub health: SyncHealth,
}

impl SyncRuntime {
    pub fn mark_offline(&mut self) {
        self.health = SyncHealth::Offline;
        self.last_error = None;
    }

    pub fn apply_push_result(&mut self, result: &PushResult) {
        self.project_revision = result.project_revision;
        self.health = if result.conflicts.is_empty() { SyncHealth::Healthy } else { SyncHealth::Conflict };
        self.last_error = None;
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.health = SyncHealth::Error;
        self.last_error = Some(message.into());
    }
}

/// In-memory transport used by branch tests and UI prototyping.
/// It deliberately models revision conflicts instead of last-write-wins.
#[derive(Default)]
pub struct MemoryTransport {
    pub project_revision: u64,
    pub object_revisions: std::collections::BTreeMap<String, u64>,
    pub presences: std::collections::BTreeMap<String, PresenceDto>,
    pub leases: std::collections::BTreeMap<String, String>,
    project_revisions: std::collections::BTreeMap<String, u64>,
    scoped_objects: std::collections::BTreeMap<(String, String), u64>,
    scoped_leases: std::collections::BTreeMap<(String, String), String>,
    applied_changes: std::collections::BTreeMap<(String, u64), ChangeEnvelope>,
    scoped_presences: std::collections::BTreeMap<(String, String), PresenceDto>,
    change_journal: std::collections::BTreeMap<String, Vec<(u64, ChangeEnvelope)>>,
}

impl CollaborationTransport for MemoryTransport {
    fn bootstrap(&mut self, project_id: &str) -> Result<Bootstrap, String> {
        Ok(Bootstrap { project_id: project_id.to_string(), project_revision: *self.project_revisions.get(project_id).unwrap_or(&0), floors: Vec::new(), permissions: Vec::new() })
    }

    fn pull_changes(&mut self, project_id: &str, after_revision: u64) -> Result<(u64, Vec<ChangeEnvelope>), String> {
        let current = self.project_revisions.get(project_id).copied().unwrap_or(0);
        if after_revision > current { return Err("cursor is ahead of current project revision".into()); }
        let changes = self.change_journal.get(project_id).map(|entries| entries.iter().filter(|(revision, _)| *revision > after_revision).map(|(_, change)| change.clone()).collect()).unwrap_or_default();
        Ok((current, changes))
    }

    fn push_changes(&mut self, project_id: &str, changes: &[ChangeEnvelope]) -> Result<PushResult, String> {
        if project_id.trim().is_empty() { return Err("project_id is required".into()); }
        if changes.is_empty() {
            return Ok(PushResult { accepted_change_set_ids: Vec::new(), project_revision: self.project_revisions.get(project_id).copied().unwrap_or(0), conflicts: Vec::new() });
        }
        let mut unique_ids = std::collections::BTreeSet::new();
        for change in changes {
            if !unique_ids.insert(change.id) { return Err("duplicate change id in batch".into()); }
            if let Some(previous) = self.applied_changes.get(&(project_id.to_string(), change.id)) {
                if previous != change { return Err("change id reused with different content".into()); }
            }
        }
        let all_replayed = changes.iter().all(|change| self.applied_changes.contains_key(&(project_id.to_string(), change.id)));
        if all_replayed {
            return Ok(PushResult { accepted_change_set_ids: changes.iter().map(|c| c.id).collect(), project_revision: self.project_revisions.get(project_id).copied().unwrap_or(0), conflicts: Vec::new() });
        }
        if changes.iter().any(|change| self.applied_changes.contains_key(&(project_id.to_string(), change.id))) {
            return Err("mixed replay and new changes require separate batches".into());
        }
        // Stage revisions first: a rejected batch must not partially modify the project.
        let mut staged = self.scoped_objects.clone();
        let mut conflicts = Vec::new();
        for change in changes {
            if change.account_id.trim().is_empty() || change.device_id.trim().is_empty() || change.items.is_empty() { return Err("invalid change envelope".into()); }
            for item in &change.items {
                if item.source_handle.trim().is_empty() || item.local_revision <= item.base_revision { return Err("invalid object revision".into()); }
                let key = (project_id.to_string(), item.source_handle.clone());
                let remote = staged.get(&key).copied().or_else(|| if project_id == "p1" { self.object_revisions.get(&item.source_handle).copied() } else { None }).unwrap_or(0);
                if remote != item.base_revision {
                    conflicts.push(RemoteConflict {
                        source_handle: item.source_handle.clone(),
                        base_revision: item.base_revision,
                        local_revision: item.local_revision,
                        remote_revision: remote,
                        remote_updated_by: "remote-user".into(),
                    });
                } else {
                    staged.insert(key, item.local_revision);
                }
            }
        }
        if !conflicts.is_empty() {
            return Ok(PushResult { accepted_change_set_ids: Vec::new(), project_revision: self.project_revisions.get(project_id).copied().unwrap_or(0), conflicts });
        }
        // Commit only after every object and revision in the batch was validated.
        self.scoped_objects = staged;
        for change in changes { self.applied_changes.insert((project_id.to_string(), change.id), change.clone()); }
        for change in changes {
            if project_id == "p1" {
                for item in &change.items { self.object_revisions.insert(item.source_handle.clone(), item.local_revision); }
            }
        }
        let revision = self.project_revisions.entry(project_id.to_string()).or_default();
        let journal = self.change_journal.entry(project_id.to_string()).or_default();
        for change in changes {
            *revision += 1;
            journal.push((*revision, change.clone()));
        }
        self.project_revision = self.project_revision.max(*revision);
        Ok(PushResult {
            accepted_change_set_ids: changes.iter().map(|change| change.id).collect(),
            project_revision: *revision,
            conflicts: Vec::new(),
        })
    }

    fn update_presence(&mut self, project_id: &str, presence: &PresenceDto) -> Result<(), String> {
        if project_id.trim().is_empty() || presence.account_id.trim().is_empty() { return Err("project and account are required".into()); }
        self.scoped_presences.insert((project_id.to_string(), presence.account_id.clone()), presence.clone());
        if project_id == "p1" { self.presences.insert(presence.account_id.clone(), presence.clone()); }
        Ok(())
    }

    fn acquire_lease(&mut self, project_id: &str, source_handle: &str, revision: u64) -> Result<LeaseResult, String> {
        if project_id.trim().is_empty() || source_handle.trim().is_empty() { return Err("project and object are required".into()); }
        let key = (project_id.to_string(), source_handle.to_string());
        let remote = self.scoped_objects.get(&key).copied().or_else(|| if project_id == "p1" { self.object_revisions.get(source_handle).copied() } else { None }).unwrap_or(0);
        if remote != revision { return Ok(LeaseResult { granted: false, owner_account_id: None, expires_unix_ms: None }); }
        match self.scoped_leases.get(&key) {
            Some(owner) => Ok(LeaseResult { granted: false, owner_account_id: Some(owner.clone()), expires_unix_ms: None }),
            None => {
                self.scoped_leases.insert(key, "local-user".into());
                Ok(LeaseResult { granted: true, owner_account_id: Some("local-user".into()), expires_unix_ms: None })
            }
        }
    }

    fn release_lease(&mut self, project_id: &str, source_handle: &str) -> Result<(), String> {
        self.scoped_leases.remove(&(project_id.to_string(), source_handle.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(id: u64, handle: &str, base: u64, local: u64) -> ChangeEnvelope {
        ChangeEnvelope {
            id,
            account_id: "u1".into(),
            device_id: "pc1".into(),
            base_project_revision: 0,
            items: vec![ChangeEnvelopeItem {
                source_handle: handle.into(),
                operation: "UPDATE".into(),
                base_revision: base,
                local_revision: local,
                payload_json: "{}".into(),
            }],
        }
    }

    #[test]
    fn memory_transport_accepts_matching_revision_and_rejects_stale_update() {
        let mut transport = MemoryTransport::default();
        transport.object_revisions.insert("A".into(), 2);
        let ok = transport.push_changes("p1", &[change(1, "A", 2, 3)]).unwrap();
        assert_eq!(ok.accepted_change_set_ids, vec![1]);
        assert!(ok.conflicts.is_empty());
        let stale = transport.push_changes("p1", &[change(2, "A", 2, 4)]).unwrap();
        assert!(stale.accepted_change_set_ids.is_empty());
        assert_eq!(stale.conflicts[0].remote_revision, 3);
    }

    #[test]
    fn lease_denies_second_editor_until_release() {
        let mut transport = MemoryTransport::default();
        assert!(transport.acquire_lease("p1", "A", 0).unwrap().granted);
        assert!(!transport.acquire_lease("p1", "A", 1).unwrap().granted);
        transport.release_lease("p1", "A").unwrap();
        // Releasing a lock does not advance the object's revision.
        assert!(!transport.acquire_lease("p1", "A", 1).unwrap().granted);
        assert!(transport.acquire_lease("p1", "A", 0).unwrap().granted);
    }

    #[test]
    fn independent_projects_cannot_share_object_revisions_or_leases() {
        let mut transport = MemoryTransport::default();
        assert_eq!(transport.push_changes("p1", &[change(1, "A", 0, 1)]).unwrap().project_revision, 1);
        assert_eq!(transport.push_changes("p2", &[change(1, "A", 0, 1)]).unwrap().project_revision, 1);
        assert!(transport.acquire_lease("p1", "A", 1).unwrap().granted);
        assert!(transport.acquire_lease("p2", "A", 1).unwrap().granted);
        assert!(!transport.acquire_lease("p1", "A", 1).unwrap().granted);
        assert_eq!(transport.bootstrap("p2").unwrap().project_revision, 1);
    }

    #[test]
    fn five_parallel_editors_cannot_overwrite_same_revision() {
        use std::sync::{Arc, Mutex, Barrier};
        let transport = Arc::new(Mutex::new(MemoryTransport::default()));
        let gate = Arc::new(Barrier::new(5));
        let threads: Vec<_> = (0..5).map(|id| {
            let transport = Arc::clone(&transport);
            let gate = Arc::clone(&gate);
            std::thread::spawn(move || {
                gate.wait();
                let result = transport.lock().unwrap().push_changes("shared", &[change(id + 1, "same-object", 0, 1)]).unwrap();
                result.accepted_change_set_ids.len()
            })
        }).collect();
        let accepted: usize = threads.into_iter().map(|t| t.join().unwrap()).sum();
        assert_eq!(accepted, 1, "only one editor may commit a matching base revision");
        let state = transport.lock().unwrap();
        assert_eq!(state.project_revisions.get("shared").copied().unwrap_or(0), 1);
    }

    #[test]
    fn conflicting_batch_does_not_commit_earlier_changes() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("shared", &[change(1, "B", 0, 1)]).unwrap();
        let result = transport.push_changes("shared", &[change(2, "A", 0, 1), change(3, "B", 0, 2)]).unwrap();
        assert!(result.accepted_change_set_ids.is_empty());
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(transport.scoped_objects.get(&("shared".into(), "A".into())), None);
        assert_eq!(transport.project_revisions.get("shared").copied(), Some(1));
    }

    #[test]
    fn replayed_batch_is_idempotent_and_empty_batch_is_noop() {
        let mut transport = MemoryTransport::default();
        let a = change(7, "A", 0, 1);
        let first = transport.push_changes("shared", &[a.clone()]).unwrap();
        let retry = transport.push_changes("shared", &[a]).unwrap();
        let empty = transport.push_changes("shared", &[]).unwrap();
        assert_eq!(first.project_revision, 1);
        assert_eq!(retry.project_revision, 1);
        assert_eq!(retry.accepted_change_set_ids, vec![7]);
        assert_eq!(empty.project_revision, 1);
    }

    #[test]
    fn changed_duplicate_id_is_rejected_without_mutation() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("shared", &[change(7, "A", 0, 1)]).unwrap();
        assert!(transport.push_changes("shared", &[change(7, "B", 0, 1)]).is_err());
        assert_eq!(transport.project_revisions.get("shared"), Some(&1));
        assert!(!transport.scoped_objects.contains_key(&("shared".into(), "B".into())));
    }

    #[test]
    fn a_future_pull_cursor_is_rejected_instead_of_falsifying_server_revision() {
        let mut transport = MemoryTransport::default();
        assert!(transport.pull_changes("shared", 7).is_err());
        assert_eq!(transport.pull_changes("shared", 0).unwrap().0, 0);
    }

    #[test]
    fn lease_requires_exact_revision_even_for_a_new_object() {
        let mut transport = MemoryTransport::default();
        assert!(!transport.acquire_lease("shared", "A", 50).unwrap().granted);
        assert!(transport.acquire_lease("shared", "A", 0).unwrap().granted);
    }

    #[test]
    fn presence_is_isolated_by_project_even_for_same_account() {
        let mut transport = MemoryTransport::default();
        let mut p = PresenceDto { account_id: "engineer".into(), floor_id: "L1".into(), discipline: "ARCH".into(), selected_object_id: None };
        transport.update_presence("project-a", &p).unwrap();
        p.floor_id = "L10".into();
        transport.update_presence("project-b", &p).unwrap();
        assert_eq!(transport.scoped_presences.get(&("project-a".into(), "engineer".into())).unwrap().floor_id, "L1");
        assert_eq!(transport.scoped_presences.get(&("project-b".into(), "engineer".into())).unwrap().floor_id, "L10");
    }

    #[test]
    fn pull_returns_only_committed_changes_after_cursor_per_project() {
        let mut transport = MemoryTransport::default();
        let a = change(1, "A", 0, 1);
        let b = change(2, "B", 0, 1);
        transport.push_changes("p-a", &[a.clone(), b.clone()]).unwrap();
        transport.push_changes("p-b", &[change(1, "Z", 0, 1)]).unwrap();
        let (head, all) = transport.pull_changes("p-a", 0).unwrap();
        assert_eq!(head, 2);
        assert_eq!(all, vec![a.clone(), b.clone()]);
        let (head, tail) = transport.pull_changes("p-a", 1).unwrap();
        assert_eq!(head, 2);
        assert_eq!(tail, vec![b]);
        assert_eq!(transport.pull_changes("p-b", 0).unwrap().1.len(), 1);
        assert!(transport.pull_changes("p-a", 3).is_err());
        transport.push_changes("p-a", &[a]).unwrap();
        assert_eq!(transport.pull_changes("p-a", 0).unwrap().1.len(), 2, "idempotent replay must not append the journal");
    }

    #[test]
    fn rejected_batch_never_enters_pull_journal() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("p-a", &[change(1, "A", 0, 1)]).unwrap();
        let stale = transport.push_changes("p-a", &[change(2, "B", 0, 1), change(3, "A", 0, 2)]).unwrap();
        assert_eq!(stale.conflicts.len(), 1);
        let (_, changes) = transport.pull_changes("p-a", 0).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].id, 1);
    }

    #[test]
    fn change_without_device_id_is_rejected_without_journal_entry() {
        let mut transport = MemoryTransport::default();
        let mut envelope = change(1, "A", 0, 1);
        envelope.device_id.clear();
        assert!(transport.push_changes("shared", &[envelope]).is_err());
        assert!(transport.pull_changes("shared", 0).unwrap().1.is_empty());
    }

    #[test]
    fn duplicate_ids_in_same_batch_are_rejected_without_side_effects() {
        let mut transport = MemoryTransport::default();
        let a = change(1, "A", 0, 1);
        let b = change(1, "B", 0, 1);
        assert!(transport.push_changes("shared", &[a, b]).is_err());
        assert!(transport.pull_changes("shared", 0).unwrap().1.is_empty());
        assert_eq!(transport.bootstrap("shared").unwrap().project_revision, 0);
    }

    #[test]
    fn five_projects_with_same_handle_remain_independent() {
        let mut transport = MemoryTransport::default();
        for id in 0..5 {
            let project = format!("site-{id}");
            let result = transport.push_changes(&project, &[change(1, "A", 0, 1)]).unwrap();
            assert_eq!(result.project_revision, 1);
            assert_eq!(transport.pull_changes(&project, 0).unwrap().1.len(), 1);
        }
        for id in 0..5 {
            let project = format!("site-{id}");
            assert_eq!(transport.bootstrap(&project).unwrap().project_revision, 1);
        }
    }

    #[test]
    fn mixed_replay_and_new_batch_cannot_partially_commit() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("shared", &[change(1, "A", 0, 1)]).unwrap();
        assert!(transport.push_changes("shared", &[change(1, "A", 0, 1), change(2, "B", 0, 1)]).is_err());
        assert_eq!(transport.pull_changes("shared", 0).unwrap().1.len(), 1);
        assert_eq!(transport.bootstrap("shared").unwrap().project_revision, 1);
    }

    #[test]
    fn sync_runtime_reports_conflict_state() {
        let mut runtime = SyncRuntime::default();
        runtime.apply_push_result(&PushResult {
            accepted_change_set_ids: vec![],
            project_revision: 5,
            conflicts: vec![RemoteConflict {
                source_handle: "A".into(),
                base_revision: 1,
                local_revision: 2,
                remote_revision: 3,
                remote_updated_by: "u2".into(),
            }],
        });
        assert_eq!(runtime.health, SyncHealth::Conflict);
        assert_eq!(runtime.project_revision, 5);
    }
}
