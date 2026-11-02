//! Pure issue workflow policy; integrate into bim_collab after tests pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Open, InProgress, Resolved, Closed }
pub fn can_transition(from: Status, to: Status, actor: &str, assignee: &str) -> bool {
    if actor.trim().is_empty() || from == Status::Closed { return false; }
    match (from, to) {
        (Status::Open, Status::InProgress) | (Status::InProgress, Status::Resolved) | (Status::Resolved, Status::InProgress) => true,
        (Status::Resolved, Status::Closed) => actor != assignee,
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn closed_issue_cannot_be_modified() { assert!(!can_transition(Status::Closed, Status::Open, "reviewer", "author")); }
    #[test] fn independent_reviewer_required() {
        assert!(!can_transition(Status::Resolved, Status::Closed, "author", "author"));
        assert!(can_transition(Status::Resolved, Status::Closed, "reviewer", "author"));
    }
}
 
#[cfg(test)] mod edge_cases {
 use super::*;
 #[test] fn rejects_no_actor_noop_and_invalid_jumps() {
   assert!(!can_transition(Status::Open, Status::InProgress, " ", "designer"));
   assert!(!can_transition(Status::Open, Status::Closed, "reviewer", "designer"));
   assert!(!can_transition(Status::Resolved, Status::Resolved, "reviewer", "designer"));
   assert!(can_transition(Status::Resolved, Status::InProgress, "reviewer", "designer"));
 }
}