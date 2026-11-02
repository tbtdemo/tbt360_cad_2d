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
