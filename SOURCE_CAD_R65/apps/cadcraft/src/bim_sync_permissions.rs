//! Independent authorization and optimistic concurrency check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditError { Unauthorized, Conflict, Closed }
pub fn ensure_edit(can_edit: bool, closed: bool, expected_revision: u64, current_revision: u64) -> Result<u64, EditError> {
    if !can_edit { return Err(EditError::Unauthorized); }
    if closed { return Err(EditError::Closed); }
    if expected_revision != current_revision { return Err(EditError::Conflict); }
    current_revision.checked_add(1).ok_or(EditError::Conflict)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn reject_unauthorized_stale_and_closed_updates() {
        assert_eq!(ensure_edit(false, false, 4, 4), Err(EditError::Unauthorized));
        assert_eq!(ensure_edit(true, false, 3, 4), Err(EditError::Conflict));
        assert_eq!(ensure_edit(true, true, 4, 4), Err(EditError::Closed));
        assert_eq!(ensure_edit(true, false, 4, 4), Ok(5));
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    #[test]
    fn revision_overflow_is_a_conflict() {
        assert_eq!(ensure_edit(true, false, u64::MAX, u64::MAX), Err(EditError::Conflict));
    }
    #[test]
    fn no_permission_is_checked_before_revision() {
        assert_eq!(ensure_edit(false, true, 2, 3), Err(EditError::Unauthorized));
    }
}
