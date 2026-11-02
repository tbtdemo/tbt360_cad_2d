//! Structured issue audit without substring-based issue identification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueEvent { pub issue_id: u64, pub actor: String, pub field: String, pub before: String, pub after: String }
pub fn changes(issue_id: u64, actor: &str, before: &[(&str, &str)], after: &[(&str, &str)]) -> Vec<IssueEvent> {
    before.iter().filter_map(|(field, value)| {
        let new_value = after.iter().find(|(name, _)| name == field)?.1;
        (new_value != *value).then(|| IssueEvent { issue_id, actor: actor.into(), field: (*field).into(), before: (*value).into(), after: new_value.into() })
    }).collect()
}
pub fn for_issue(events: &[IssueEvent], id: u64) -> Vec<&IssueEvent> { events.iter().filter(|e| e.issue_id == id).collect() }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn issue_ids_are_exact_not_prefix_matches() {
        let mut events = changes(1, "alice", &[("priority", "P2")], &[("priority", "P0")]);
        events.extend(changes(10, "bob", &[("priority", "P2")], &[("priority", "P1")]));
        assert_eq!(for_issue(&events, 1).len(), 1);
        assert_eq!(for_issue(&events, 1)[0].after, "P0");
    }
}

#[cfg(test)] mod edge_cases {
 use super::*;
 #[test] fn unchanged_and_missing_fields_do_not_create_events() {
  assert!(changes(4,"reviewer",&[("priority","P1")],&[("priority","P1")]).is_empty());
  assert!(changes(4,"reviewer",&[("status","Open")],&[]).is_empty());
 }
}
#[cfg(test)]
mod unicode_audit_tests {
    use super::*;
    #[test]
    fn vietnamese_field_changes_preserve_exact_text() {
        let events = changes(42, "người duyệt", &[("trạng thái", "Đang xử lý")], &[("trạng thái", "Đã xử lý")]);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].before, "Đang xử lý");
        assert_eq!(events[0].after, "Đã xử lý");
        assert!(for_issue(&events, 43).is_empty());
    }
}
