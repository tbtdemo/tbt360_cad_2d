//! Comments and RFI workflow with standalone validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment { pub issue_id: u64, pub author: String, pub message: String, pub sequence: u64 }
pub fn post_comment(issue_id: u64, author: &str, message: &str, sequence: u64) -> Result<Comment, &'static str> {
    if author.trim().is_empty() { return Err("author required"); }
    if message.trim().is_empty() || message.chars().count() > 4000 { return Err("invalid message"); }
    Ok(Comment { issue_id, author: author.trim().into(), message: message.trim().into(), sequence })
}
pub fn thread(comments: &[Comment], id: u64) -> Vec<&Comment> { comments.iter().filter(|c| c.issue_id == id).collect() }
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn rfi_comments_are_scoped_and_validated() {
        let x = post_comment(10, "designer", "Please clarify", 1).unwrap();
        assert!(post_comment(10, "designer", " ", 2).is_err());
        assert!(thread(&[x.clone()], 1).is_empty());
        assert_eq!(thread(&[x], 10).len(), 1);
    }
}
