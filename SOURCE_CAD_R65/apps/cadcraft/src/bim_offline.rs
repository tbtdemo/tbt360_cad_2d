//! Offline-first BIM synchronization state machine.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Connectivity {
    #[default]
    Offline,
    Connecting,
    Online,
    Degraded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QueueState {
    #[default]
    Local,
    Pending,
    Synced,
    Conflict,
    Rejected,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncCheckpoint {
    pub remote_project_revision: u64,
    pub last_pulled_revision: u64,
    pub local_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflineQueueItem {
    pub local_sequence: u64,
    pub change_set_id: String,
    pub state: QueueState,
    pub retry_count: u32,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct OfflineSyncState {
    pub connectivity: Connectivity,
    pub checkpoint: SyncCheckpoint,
    pub queue: Vec<OfflineQueueItem>,
}

impl OfflineSyncState {
    pub fn enqueue(&mut self, change_set_id: String) -> u64 {
        if change_set_id.trim().is_empty() { return 0; }
        if let Some(existing) = self.queue.iter().find(|item| item.change_set_id == change_set_id) { return existing.local_sequence; }
        let Some(next) = self.checkpoint.local_sequence.checked_add(1) else { return 0; };
        self.checkpoint.local_sequence = next;
        let seq = self.checkpoint.local_sequence;
        self.queue.push(OfflineQueueItem { local_sequence: seq, change_set_id, state: QueueState::Local, retry_count: 0, last_error: None });
        seq
    }

    pub fn begin_reconnect(&mut self) {
        self.connectivity = Connectivity::Connecting;
        for item in &mut self.queue {
            if matches!(item.state, QueueState::Local | QueueState::Pending) { item.state = QueueState::Pending; }
        }
    }

    pub fn mark_online(&mut self, remote_revision: u64) {
        self.connectivity = Connectivity::Online;
        self.checkpoint.remote_project_revision = self.checkpoint.remote_project_revision.max(remote_revision);
    }

    pub fn acknowledge_synced(&mut self, local_sequence: u64, remote_revision: u64) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            if matches!(item.state, QueueState::Rejected | QueueState::Conflict) { return; }
            item.state = QueueState::Synced;
            item.last_error = None;
            self.checkpoint.remote_project_revision = self.checkpoint.remote_project_revision.max(remote_revision);
        }
    }

    pub fn retry_delay_seconds(&self, local_sequence: u64) -> Option<u64> {
        let item = self.queue.iter().find(|i| i.local_sequence == local_sequence)?;
        if !matches!(item.state, QueueState::Local | QueueState::Pending) { return None; }
        Some((1u64 << item.retry_count.min(6)).min(60))
    }

    pub fn mark_retry_error(&mut self, local_sequence: u64, message: String) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            if matches!(item.state, QueueState::Synced | QueueState::Rejected | QueueState::Conflict) { return; }
            item.retry_count = item.retry_count.saturating_add(1);
            item.state = QueueState::Pending;
            item.last_error = Some(message);
        }
        if self.queue.iter().any(|i| i.local_sequence == local_sequence && i.state == QueueState::Pending) { self.connectivity = Connectivity::Degraded; }
    }

    pub fn mark_conflict(&mut self, local_sequence: u64, message: String) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            if matches!(item.state, QueueState::Synced | QueueState::Rejected) { return; }
            item.state = QueueState::Conflict;
            item.last_error = Some(message);
        }
    }

    pub fn resolve_conflict_for_retry(&mut self, local_sequence: u64) -> bool {
        let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) else { return false; };
        if item.state != QueueState::Conflict { return false; }
        item.state = QueueState::Pending;
        item.retry_count = 0;
        item.last_error = None;
        true
    }

    pub fn mark_pulled(&mut self, revision: u64) -> bool {
        if revision < self.checkpoint.last_pulled_revision || revision > self.checkpoint.remote_project_revision { return false; }
        self.checkpoint.last_pulled_revision = revision;
        true
    }

    pub fn pending_count(&self) -> usize {
        self.queue.iter().filter(|i| !matches!(i.state, QueueState::Synced | QueueState::Rejected)).count()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_is_bounded_and_acknowledgement_advances_checkpoint() {
        let mut s=OfflineSyncState::default();
        let seq=s.enqueue("c1".into());
        s.mark_retry_error(seq,"network".into());
        assert_eq!(s.retry_delay_seconds(seq),Some(2));
        for _ in 0..10 { s.mark_retry_error(seq,"network".into()); }
        assert_eq!(s.retry_delay_seconds(seq),Some(60));
        s.acknowledge_synced(seq,12);
        assert_eq!(s.checkpoint.remote_project_revision,12);
        assert_eq!(s.pending_count(),0);
    }

    #[test]
    fn offline_changes_move_to_pending_on_reconnect() {
        let mut s = OfflineSyncState::default();
        s.enqueue("c1".into());
        s.enqueue("c2".into());
        assert_eq!(s.pending_count(), 2);
        s.begin_reconnect();
        assert!(s.queue.iter().all(|i| i.state == QueueState::Pending));
        s.mark_online(8);
        assert_eq!(s.connectivity, Connectivity::Online);
        assert_eq!(s.checkpoint.remote_project_revision, 8);
    }
    #[test]
    fn late_errors_do_not_reopen_synced_queue_items() {
        let mut s = OfflineSyncState::default();
        let seq = s.enqueue("c1".into());
        s.acknowledge_synced(seq, 12);
        s.mark_retry_error(seq, "late network failure".into());
        s.mark_conflict(seq, "late conflict".into());
        assert_eq!(s.queue[0].state, QueueState::Synced);
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn stale_remote_status_cannot_reduce_checkpoint() {
        let mut s = OfflineSyncState::default();
        s.mark_online(30);
        s.mark_online(18);
        assert_eq!(s.checkpoint.remote_project_revision, 30);
    }

    #[test]
    fn conflict_is_not_retried_until_explicitly_resolved() {
        let mut state = OfflineSyncState::default();
        let sequence = state.enqueue("edit-1".into());
        state.mark_conflict(sequence, "stale object".into());
        state.mark_retry_error(sequence, "late network failure".into());
        assert_eq!(state.queue[0].state, QueueState::Conflict);
        assert_eq!(state.queue[0].retry_count, 0);
        assert!(state.resolve_conflict_for_retry(sequence));
        assert_eq!(state.queue[0].state, QueueState::Pending);
        state.mark_retry_error(sequence, "temporary failure".into());
        assert_eq!(state.queue[0].retry_count, 1);
    }

    #[test]
    fn delayed_failure_does_not_mark_online_connection_degraded() {
        let mut s = OfflineSyncState::default();
        let n = s.enqueue("c".into());
        s.mark_online(8);
        s.acknowledge_synced(n, 9);
        s.mark_retry_error(n, "late".into());
        assert_eq!(s.connectivity, Connectivity::Online);
    }

    #[test]
    fn exhausted_sequence_does_not_insert_duplicate_item() {
        let mut state = OfflineSyncState::default();
        state.checkpoint.local_sequence = u64::MAX;
        assert_eq!(state.enqueue("later".into()), 0);
        assert_eq!(state.queue.len(), 0);
    }

    #[test]
    fn repeated_offline_enqueue_reuses_original_sequence() {
        let mut s = OfflineSyncState::default();
        let first = s.enqueue("same-change".into());
        let second = s.enqueue("same-change".into());
        assert_eq!(first, second);
        assert_eq!(s.queue.len(), 1);
        assert_eq!(s.checkpoint.local_sequence, first);
        assert_eq!(s.enqueue("  ".into()), 0);
        assert_eq!(s.queue.len(), 1);
    }

    #[test]
    fn pulled_revision_must_be_monotonic_and_not_exceed_remote() {
        let mut s = OfflineSyncState::default();
        s.mark_online(20);
        assert!(s.mark_pulled(15));
        assert!(!s.mark_pulled(14));
        assert!(!s.mark_pulled(21));
        assert_eq!(s.checkpoint.last_pulled_revision, 15);
        assert!(s.mark_pulled(20));
    }

    #[test]
    fn completed_or_conflicted_entries_do_not_receive_retry_schedules() {
        let mut s = OfflineSyncState::default();
        let done = s.enqueue("done".into());
        let blocked = s.enqueue("blocked".into());
        s.acknowledge_synced(done, 1);
        s.mark_conflict(blocked, "revision mismatch".into());
        assert_eq!(s.retry_delay_seconds(done), None);
        assert_eq!(s.retry_delay_seconds(blocked), None);
        assert!(s.resolve_conflict_for_retry(blocked));
        assert_eq!(s.retry_delay_seconds(blocked), Some(1));
    }

    #[test]
    fn late_acknowledgement_does_not_clear_unresolved_conflict() {
        let mut state = OfflineSyncState::default();
        let seq = state.enqueue("conflicted".into());
        state.mark_conflict(seq, "stale".into());
        state.acknowledge_synced(seq, 99);
        assert_eq!(state.queue[0].state, QueueState::Conflict);
        assert_eq!(state.checkpoint.remote_project_revision, 0);
        assert!(state.resolve_conflict_for_retry(seq));
        state.acknowledge_synced(seq, 2);
        assert_eq!(state.queue[0].state, QueueState::Synced);
    }

}
