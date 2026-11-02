//! BCF viewport data validation; camera coordinates never silently accept NaN/Inf.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewpoint { pub issue_id: u64, pub position: [f64; 3], pub direction: [f64; 3], pub selected: Vec<String> }
pub fn validate_viewpoint(v: &Viewpoint) -> Result<(), &'static str> {
    if !v.position.iter().chain(v.direction.iter()).all(|x| x.is_finite()) { return Err("nonfinite camera"); }
    let length_sq: f64 = v.direction.iter().map(|x| x * x).sum();
    if !length_sq.is_finite() || length_sq <= 1e-20 { return Err("invalid camera direction"); }
    if v.selected.iter().any(|s| s.trim().is_empty()) { return Err("empty entity id"); }
    Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn viewpoint_requires_valid_camera() {
        let mut v = Viewpoint { issue_id: 1, position: [0.0,0.0,10.0], direction: [0.0,0.0,-1.0], selected: vec!["A10".into()] };
        assert_eq!(validate_viewpoint(&v), Ok(()));
        v.direction[1] = f64::NAN;
        assert!(validate_viewpoint(&v).is_err());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    #[test]
    fn invalid_direction_and_selection_are_rejected() {
        let mut v = Viewpoint { issue_id: 5, position: [0.0; 3], direction: [0.0; 3], selected: Vec::new() };
        assert!(validate_viewpoint(&v).is_err());
        v.direction = [0.0, 0.0, 1.0];
        v.selected.push(String::new());
        assert!(validate_viewpoint(&v).is_err());
    }
}
