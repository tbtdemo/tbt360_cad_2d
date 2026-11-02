//! Restricted attachment metadata validation; no disk writes or downloads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment { pub issue_id: u64, pub filename: String, pub size_bytes: u64 }
pub fn validate_attachment(item: &Attachment, max_size: u64) -> Result<(), &'static str> {
    if item.filename.is_empty() || item.filename == "." || item.filename == ".." { return Err("invalid filename"); }
    if item.filename.contains('/') || item.filename.contains('\\') || item.filename.contains(':') || item.filename.contains('\0') { return Err("path separators forbidden"); }
    if item.filename.chars().count() > 255 { return Err("filename too long"); }
    if item.size_bytes == 0 || item.size_bytes > max_size { return Err("invalid file size"); }
    Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn rejects_traversal_and_large_files() {
        let mut a=Attachment {issue_id:1,filename:"clash.png".into(),size_bytes:1024};
        assert_eq!(validate_attachment(&a,10_000), Ok(()));
        a.filename="../secret.pdf".into();
        assert!(validate_attachment(&a,10_000).is_err());
        a.filename="safe.png".into(); a.size_bytes=10_001;
        assert!(validate_attachment(&a,10_000).is_err());
    }
}
