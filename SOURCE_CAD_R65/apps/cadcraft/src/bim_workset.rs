//! BIM worksets and publish/consume lifecycle.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionState {
    Working,
    Review,
    Published,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workset {
    pub id: String,
    pub project_id: String,
    pub floor_id: String,
    pub discipline: String,
    pub code: String,
    pub name: String,
    pub owner_account_id: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRevision {
    pub id: u64,
    pub project_id: String,
    pub workset_id: String,
    pub number: u64,
    pub state: RevisionState,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub published_by: Option<String>,
}

impl ModelRevision {
    pub fn submit_for_review(&mut self) -> bool {
        if self.state != RevisionState::Working { return false; }
        self.state = RevisionState::Review;
        true
    }

    pub fn publish(&mut self, account_id: String) -> bool {
        if self.state != RevisionState::Review { return false; }
        if account_id.trim().is_empty() { return false; }
        self.state = RevisionState::Published;
        self.published_by = Some(account_id);
        true
    }

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumedRevision {
    pub consumer_workset_id: String,
    pub source_workset_id: String,
    pub source_revision: u64,
}

pub fn consume_revision(consumer_workset_id: String, source: &ModelRevision) -> Result<ConsumedRevision, String> {
    if consumer_workset_id.trim().is_empty() || source.project_id.trim().is_empty() || source.workset_id.trim().is_empty() { return Err("Missing workset or project identity".into()); }
    if source.state != RevisionState::Published {
        return Err("Chỉ được consume revision đã Published".into());
    }
    Ok(ConsumedRevision {
        consumer_workset_id,
        source_workset_id: source.workset_id.clone(),
        source_revision: source.number,
    })
}

pub fn supersede_previous(previous: &mut ModelRevision, next: &ModelRevision) -> bool {
    if previous.project_id != next.project_id || previous.workset_id != next.workset_id || previous.id == next.id || previous.state != RevisionState::Published || next.state != RevisionState::Published || next.number <= previous.number {
        return false;
    }
    previous.state = RevisionState::Superseded;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumer_can_only_reference_published_revision() {
        let mut r=ModelRevision{id:1,project_id:"P".into(),workset_id:"W".into(),number:1,state:RevisionState::Working,created_by:"u".into(),reviewed_by:None,published_by:None};
        assert!(consume_revision("C".into(),&r).is_err());
        r.state=RevisionState::Published;
        assert_eq!(consume_revision("C".into(),&r).unwrap().source_revision,1);
    }

    #[test]
    fn revision_requires_review_before_publish() {
        let mut r = ModelRevision {
            id: 1, project_id: "P".into(), workset_id: "W".into(), number: 1,
            state: RevisionState::Working, created_by: "u1".into(), reviewed_by: None, published_by: None
        };
        assert!(!r.publish("u2".into()));
        assert!(r.submit_for_review());
        assert!(r.publish("u2".into()));
        assert_eq!(r.state, RevisionState::Published);
    }
    #[test]
    fn published_revision_cannot_be_superseded_by_another_project() {
        let mut previous = ModelRevision { id:1, project_id:"P1".into(), workset_id:"W".into(), number:1, state:RevisionState::Published, created_by:"u".into(), reviewed_by:None, published_by:Some("u".into()) };
        let next = ModelRevision { id:2, project_id:"P2".into(), workset_id:"W".into(), number:2, state:RevisionState::Published, created_by:"u".into(), reviewed_by:None, published_by:Some("u".into()) };
        assert!(!supersede_previous(&mut previous, &next));
        assert_eq!(previous.state, RevisionState::Published);
    }

    #[test]
    fn empty_consumer_workset_is_rejected() {
        let revision = ModelRevision { id: 10, project_id: "P".into(), workset_id: "W".into(), number: 3, state: RevisionState::Published, created_by: "author".into(), reviewed_by: None, published_by: Some("reviewer".into()) };
        assert!(consume_revision(String::new(), &revision).is_err());
        assert!(consume_revision("C".into(), &revision).is_ok());
    }

    #[test]
    fn publish_rejects_missing_user_and_preserves_review_state() {
        let mut revision = ModelRevision { id:9, project_id:"P".into(), workset_id:"W".into(), number:2, state:RevisionState::Review, created_by:"author".into(), reviewed_by:None, published_by:None };
        assert!(!revision.publish(String::new()));
        assert_eq!(revision.state, RevisionState::Review);
        assert!(revision.published_by.is_none());
        assert!(revision.publish("reviewer".into()));
    }

}
