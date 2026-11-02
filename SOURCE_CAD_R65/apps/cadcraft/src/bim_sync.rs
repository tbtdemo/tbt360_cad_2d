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
    pub change_history: Vec<(u64, ChangeEnvelope)>,
}

impl CollaborationTransport for MemoryTransport {
    fn bootstrap(&mut self, project_id: &str) -> Result<Bootstrap, String> {
        Ok(Bootstrap { project_id: project_id.to_string(), project_revision: self.project_revision, floors: Vec::new(), permissions: Vec::new() })
    }

    fn pull_changes(&mut self, _project_id: &str, after_revision: u64) -> Result<(u64, Vec<ChangeEnvelope>), String> {
        let changes = self.change_history.iter().filter(|(revision, _)| *revision > after_revision).map(|(_, change)| change.clone()).collect();
        Ok((self.project_revision, changes))
    }

    fn push_changes(&mut self, _project_id: &str, changes: &[ChangeEnvelope]) -> Result<PushResult, String> {
        for change in changes {
            if change.account_id.trim().is_empty() || change.device_id.trim().is_empty() {
                return Err("Missing account or device identity".into());
            }
            for item in &change.items {
                if item.source_handle.trim().is_empty() || !matches!(item.operation.as_str(), "CREATE" | "UPDATE" | "DELETE") {
                    return Err("Invalid object identity or operation".into());
                }
                if item.base_revision.checked_add(1) != Some(item.local_revision) {
                    return Err("Object revision must advance exactly once".into());
                }
            }
        }
        // Detect idempotency collisions anywhere in the request before committing a preceding batch.
        let mut submitted = std::collections::BTreeMap::new();
        for change in changes {
            let identity = (change.account_id.as_str(), change.device_id.as_str(), change.id);
            if let Some(previous) = submitted.insert(identity, change) {
                if previous != change { return Err("Duplicate changeset identity in request with different content".into()); }
            }
            if let Some((_, existing)) = self.change_history.iter().find(|(_, existing)| existing.id == change.id && existing.account_id == change.account_id && existing.device_id == change.device_id) {
                if existing != change { return Err("Changeset identity reused with different content".into()); }
            }
        }
        // Reserve revision capacity for the entire request; do not allow rollover in a long batch.
        let candidate_count = u64::try_from(changes.iter().filter(|change| !change.items.is_empty() && !self.change_history.iter().any(|(_, recorded)| recorded.eq(*change))).count()).map_err(|_| "Too many changesets")?;
        if self.project_revision.checked_add(candidate_count).is_none() { return Err("Project revision overflow".into()); }
        // Validate the whole request before mutating any revision.
        if let Some(invalid) = changes.iter().find(|change| change.base_project_revision > self.project_revision) {
            return Err(format!("Future project revision {} (server {})", invalid.base_project_revision, self.project_revision));
        }
        let mut accepted = Vec::new();
        let mut conflicts = Vec::new();
        for change in changes {
            // A repeated request must not increment revision or replay writes.
            if let Some((_, recorded)) = self.change_history.iter().find(|(_, recorded)| recorded.id == change.id && recorded.account_id == change.account_id && recorded.device_id == change.device_id) {
                if recorded != change { return Err("Changeset identity reused with different content".into()); }
                continue;
            }
            // An empty changeset must not increment the project revision.
            if change.items.is_empty() { continue; }
            let mut change_conflict = false;
            for item in &change.items {
                let remote_revision = self.object_revisions.get(&item.source_handle).copied().unwrap_or(0);
                if remote_revision != item.base_revision || item.local_revision <= item.base_revision {
                    conflicts.push(RemoteConflict {
                        source_handle: item.source_handle.clone(),
                        base_revision: item.base_revision,
                        local_revision: item.local_revision,
                        remote_revision,
                        remote_updated_by: "remote-user".into(),
                    });
                    change_conflict = true;
                }
            }
            if change_conflict {
                continue;
            }
            // Reject duplicate handles in one changeset: otherwise the last item silently wins.
            let mut handles = std::collections::BTreeSet::new();
            if let Some(duplicate) = change.items.iter().find(|item| !handles.insert(item.source_handle.as_str())) {
                conflicts.push(RemoteConflict {
                    source_handle: duplicate.source_handle.clone(),
                    base_revision: duplicate.base_revision,
                    local_revision: duplicate.local_revision,
                    remote_revision: self.object_revisions.get(&duplicate.source_handle).copied().unwrap_or(0),
                    remote_updated_by: "duplicate-handle-in-batch".into(),
                });
                continue;
            }
            for item in &change.items {
                self.object_revisions.insert(item.source_handle.clone(), item.local_revision);
            }
            accepted.push(change.id);
            self.project_revision += 1;
            self.change_history.push((self.project_revision, change.clone()));
        }
        Ok(PushResult { accepted_change_set_ids: accepted, project_revision: self.project_revision, conflicts })
    }

    fn update_presence(&mut self, _project_id: &str, presence: &PresenceDto) -> Result<(), String> {
        self.presences.insert(presence.account_id.clone(), presence.clone());
        Ok(())
    }

    fn acquire_lease(&mut self, _project_id: &str, source_handle: &str, _revision: u64) -> Result<LeaseResult, String> {
        match self.leases.get(source_handle) {
            Some(owner) => Ok(LeaseResult { granted: false, owner_account_id: Some(owner.clone()), expires_unix_ms: None }),
            None => {
                self.leases.insert(source_handle.to_string(), "local-user".into());
                Ok(LeaseResult { granted: true, owner_account_id: Some("local-user".into()), expires_unix_ms: None })
            }
        }
    }

    fn release_lease(&mut self, _project_id: &str, source_handle: &str) -> Result<(), String> {
        self.leases.remove(source_handle);
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
        let stale = transport.push_changes("p1", &[change(2, "A", 2, 3)]).unwrap();
        assert!(stale.accepted_change_set_ids.is_empty());
        assert_eq!(stale.conflicts[0].remote_revision, 3);
    }

    #[test]
    fn lease_denies_second_editor_until_release() {
        let mut transport = MemoryTransport::default();
        assert!(transport.acquire_lease("p1", "A", 1).unwrap().granted);
        assert!(!transport.acquire_lease("p1", "A", 1).unwrap().granted);
        transport.release_lease("p1", "A").unwrap();
        assert!(transport.acquire_lease("p1", "A", 1).unwrap().granted);
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
    #[test]
    fn duplicate_object_handle_in_one_changeset_is_rejected_atomically() {
        let mut transport = MemoryTransport::default();
        let mut batch = change(3, "A", 0, 1);
        batch.items.push(ChangeEnvelopeItem { source_handle: "A".into(), operation: "UPDATE".into(), base_revision: 0, local_revision: 2, payload_json: "{}".into() });
        let result = transport.push_changes("p1", &[batch]).unwrap();
        assert!(result.accepted_change_set_ids.is_empty());
        assert_eq!(result.conflicts.len(), 1);
        assert!(!transport.object_revisions.contains_key("A"));
        assert_eq!(transport.project_revision, 0);
    }

    #[test]
    fn empty_changeset_does_not_increment_project_revision() {
        let mut transport = MemoryTransport::default();
        let mut empty = change(9, "A", 0, 1);
        empty.items.clear();
        let result = transport.push_changes("p1", &[empty]).unwrap();
        assert!(result.accepted_change_set_ids.is_empty());
        assert_eq!(result.project_revision, 0);
    }

    #[test]
    fn non_increasing_object_revision_is_rejected() {
        let mut transport = MemoryTransport::default();
        transport.object_revisions.insert("A".into(), 3);
        let result = transport.push_changes("p1", &[change(8, "A", 3, 3)]);
        assert!(result.is_err());
        assert_eq!(transport.object_revisions["A"], 3);
        assert_eq!(transport.project_revision, 0);
    }

    #[test]
    fn future_project_revision_is_rejected_without_writes() {
        let mut transport = MemoryTransport::default();
        let mut batch = change(10, "A", 0, 1);
        batch.base_project_revision = 99;
        assert!(transport.push_changes("p1", &[batch]).is_err());
        assert!(transport.object_revisions.is_empty());
    }

    #[test]
    fn future_revision_in_later_batch_cannot_partially_commit_earlier_batch() {
        let mut transport = MemoryTransport::default();
        let mut future = change(12, "B", 0, 1);
        future.base_project_revision = 100;
        assert!(transport.push_changes("p1", &[change(11, "A", 0, 1), future]).is_err());
        assert_eq!(transport.project_revision, 0);
        assert!(transport.object_revisions.is_empty());
    }

    #[test]
    fn batch_metadata_is_validated_before_any_write() {
        let mut transport = MemoryTransport::default();
        let mut invalid = change(22, "B", 0, 1);
        invalid.account_id.clear();
        let result = transport.push_changes("p", &[change(21, "A", 0, 1), invalid]);
        assert!(result.is_err());
        assert_eq!(transport.project_revision, 0);
        assert!(transport.object_revisions.is_empty());
    }

    #[test]
    fn invalid_operation_and_revision_gap_are_rejected() {
        let mut transport = MemoryTransport::default();
        let mut invalid = change(23, "A", 0, 1);
        invalid.items[0].operation = "INVALID".into();
        assert!(transport.push_changes("p", &[invalid]).is_err());
        assert!(transport.push_changes("p", &[change(24, "A", 0, 5)]).is_err());
        assert!(transport.object_revisions.is_empty());
    }

    #[test]
    fn revision_overflow_is_rejected_without_mutation() {
        let mut transport = MemoryTransport::default();
        assert!(transport.push_changes("p", &[change(25, "A", u64::MAX, u64::MAX)]).is_err());
        assert_eq!(transport.project_revision, 0);
    }

    #[test]
    fn change_feed_returns_only_committed_changes_after_cursor() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("p", &[change(31, "A", 0, 1), change(32, "B", 0, 1)]).unwrap();
        let (revision, initial) = transport.pull_changes("p", 0).unwrap();
        assert_eq!(revision, 2);
        assert_eq!(initial.len(), 2);
        assert_eq!(initial[0].id, 31);
        let (_, subsequent) = transport.pull_changes("p", 1).unwrap();
        assert_eq!(subsequent.len(), 1);
        assert_eq!(subsequent[0].id, 32);
        let (_, none) = transport.pull_changes("p", 2).unwrap();
        assert!(none.is_empty());
    }

    #[test]
    fn repeated_changeset_is_idempotent() {
        let mut transport = MemoryTransport::default();
        let batch = change(51, "A", 0, 1);
        let first = transport.push_changes("p", std::slice::from_ref(&batch)).unwrap();
        let second = transport.push_changes("p", &[batch]).unwrap();
        assert_eq!(first.accepted_change_set_ids, vec![51]);
        assert!(second.accepted_change_set_ids.is_empty());
        assert!(second.conflicts.is_empty());
        assert_eq!(transport.project_revision, 1);
        assert_eq!(transport.change_history.len(), 1);
    }

    #[test]
    fn replay_with_same_identity_but_different_payload_is_rejected() {
        let mut transport = MemoryTransport::default();
        let first = change(61, "A", 0, 1);
        transport.push_changes("p", &[first.clone()]).unwrap();
        let mut altered = first;
        altered.items[0].payload_json = "{\"changed\":true}".into();
        assert!(transport.push_changes("p", &[altered]).is_err());
        assert_eq!(transport.project_revision, 1);
        assert_eq!(transport.change_history.len(), 1);
    }

    #[test]
    fn later_idempotency_collision_does_not_partially_commit_earlier_changes() {
        let mut transport = MemoryTransport::default();
        transport.push_changes("p", &[change(81, "A", 0, 1)]).unwrap();
        let mut collision = change(81, "A", 0, 1);
        collision.items[0].payload_json = "different".into();
        let result = transport.push_changes("p", &[change(82, "B", 0, 1), collision]);
        assert!(result.is_err());
        assert_eq!(transport.project_revision, 1);
        assert!(!transport.object_revisions.contains_key("B"));
    }

}
