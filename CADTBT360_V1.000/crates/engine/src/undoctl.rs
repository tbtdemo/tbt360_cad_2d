//! UNDO control: marks, BEgin/End groups and the Control/Auto settings of one drawing, kept on top
//! of its undo stack ([`DocState::undo`]). Every command records its step through
//! [`DocState::push_undo`], so marks, groups, Control and the stack limit apply to all of them
//! alike; a command never needs to know about them.

use crate::{DocState, Snapshot};

/// The most undo steps a drawing keeps; the oldest are dropped first.
pub const MAX_UNDO: usize = 2000;
/// The most marks a drawing keeps; the oldest are dropped first.
const MAX_MARKS: usize = 1000;

/// UNDO Control: how much undo information is kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Control {
    /// Full undo (the default).
    #[default]
    All,
    /// Undo is off: nothing is recorded.
    None,
    /// Only the last step can be undone.
    One,
}

/// The UNDO settings and markers of one drawing.
#[derive(Clone, Debug)]
pub struct UndoCtl {
    pub control: Control,
    /// UNDO Auto. Every command already records one undo step for all it does, so this is kept and
    /// reported (UNDOCTL) only.
    pub auto: bool,
    /// UNDO Control Combine: consecutive zooms and pans as one step. View changes record no undo
    /// steps, so this is kept and reported only.
    pub combine_view: bool,
    /// UNDO Control Layer: layer property changes as one step. Kept and reported only.
    pub combine_layer: bool,
    /// Undo-stack depths at each UNDO Mark, oldest first.
    pub marks: Vec<usize>,
    /// Undo-stack depth when the open BEgin group started.
    pub group: Option<usize>,
}

impl Default for UndoCtl {
    fn default() -> Self {
        UndoCtl { control: Control::All, auto: true, combine_view: true, combine_layer: true, marks: Vec::new(), group: None }
    }
}

impl UndoCtl {
    /// The UNDOCTL bit code: 1 undo on, 2 only one step, 4 Auto, 8 a group is open, 16 combine
    /// zooms and pans, 32 combine layer changes.
    pub fn bits(&self) -> i64 {
        let mut b = 0;
        if self.control != Control::None {
            b |= 1;
        }
        if self.control == Control::One {
            b |= 2;
        }
        if self.auto {
            b |= 4;
        }
        if self.group.is_some() {
            b |= 8;
        }
        if self.combine_view {
            b |= 16;
        }
        if self.combine_layer {
            b |= 32;
        }
        b
    }
}

impl DocState {
    /// Record an undo step (`snap` holds the drawing as it was before the change) and clear redo.
    /// UNDO Control None records nothing and One keeps just this step; the stack holds at most
    /// [`MAX_UNDO`] steps.
    pub fn push_undo(&mut self, snap: Snapshot) {
        self.redo.clear();
        match self.undo_ctl.control {
            Control::None => {
                self.undo.clear();
                return;
            }
            Control::One => {
                self.undo.clear();
                self.undo.push(snap);
                return;
            }
            Control::All => {}
        }
        // Marks above the current depth were undone; a new step makes them unreachable for good.
        let depth = self.undo.len();
        self.undo_ctl.marks.retain(|m| *m <= depth);
        if let Some(g) = self.undo_ctl.group
            && g > depth
        {
            self.undo_ctl.group = Some(depth);
        }
        self.undo.push(snap);
        // Stage279: during UNDO BEGIN, only the first snapshot is needed to
        // restore the document to its exact pre-group state. Collapse newer
        // snapshots *before* enforcing MAX_UNDO; otherwise a long AI import
        // can evict its own first snapshot and corrupt single-step rollback.
        if let Some(start) = self.undo_ctl.group {
            let keep = start.saturating_add(1);
            if self.undo.len() > keep {self.undo.truncate(keep);}
        }
        if let Some(extra) = self.undo.len().checked_sub(MAX_UNDO).filter(|e| *e > 0) {
            self.undo.drain(..extra);
            for m in &mut self.undo_ctl.marks {
                *m = m.saturating_sub(extra);
            }
            if let Some(g) = &mut self.undo_ctl.group {
                *g = g.saturating_sub(extra);
            }
        }
    }

    /// UNDO Mark at the current depth. Returns the number of marks.
    pub fn undo_mark(&mut self) -> usize {
        let depth = self.undo.len();
        let marks = &mut self.undo_ctl.marks;
        marks.retain(|m| *m <= depth);
        marks.push(depth);
        if marks.len() > MAX_MARKS {
            marks.remove(0);
        }
        marks.len()
    }

    /// The marks still reachable from the current depth.
    pub fn undo_marks(&self) -> usize {
        let depth = self.undo.len();
        self.undo_ctl.marks.iter().filter(|m| **m <= depth).count()
    }

    /// UNDO BEgin: the steps from here to UNDO End become one. BEgin while a group is open ends that
    /// group first (returns true then).
    pub fn begin_undo_group(&mut self) -> bool {
        let ended = self.end_undo_group();
        self.undo_ctl.group = Some(self.undo.len());
        ended
    }

    /// UNDO End (also on save): collapse the steps since BEgin into one undo step. Returns whether
    /// a group was open; End with no open group does nothing.
    pub fn end_undo_group(&mut self) -> bool {
        let Some(start) = self.undo_ctl.group.take() else { return false };
        // The first step of the group holds the drawing as it was at BEgin.
        if self.undo.len() > start {
            self.undo.truncate(start + 1);
            if let Some(first) = self.undo.get_mut(start) {
                first.label = "Group".into();
            }
        }
        // States inside the group no longer exist.
        self.undo_ctl.marks.retain(|m| *m <= start);
        true
    }

    /// UNDO Control: All, None (drops all undo information) or One (keeps only the last step).
    /// Any open group and all marks are dropped when leaving All.
    pub fn set_undo_control(&mut self, c: Control) {
        self.undo_ctl.control = c;
        if c == Control::All {
            return;
        }
        self.undo_ctl.group = None;
        self.undo_ctl.marks.clear();
        self.redo.clear();
        let keep = if c == Control::One { 1 } else { 0 };
        let drop = self.undo.len().saturating_sub(keep);
        self.undo.drain(..drop);
    }
}

// Stage279: a very long native group must keep its first pre-group snapshot even
// when ordinary history is at the MAX_UNDO ceiling.
#[cfg(test)]
mod stage279_tests {
    use super::*;
    use cadcraft_doc::Drawing;

    fn snapshot(st:&DocState,label:&str)->Snapshot {
        Snapshot {label:label.into(),doc:st.doc.clone(),selection:Vec::new()}
    }

    #[test]
    fn stage279_undo_group_stays_single_snapshot_at_max_depth() {
        let mut st=DocState::new(Drawing::default(),"Stage279",None);
        for _ in 0..MAX_UNDO {
            let snap=snapshot(&st,"prior");
            st.push_undo(snap);
        }
        assert_eq!(st.undo.len(),MAX_UNDO);
        assert!(!st.begin_undo_group());
        let original=st.doc.clone();
        for _ in 0..256 {
            let snap=snapshot(&st,"native");
            st.push_undo(snap);
            let start=st.undo_ctl.group.expect("group remains open");
            assert_eq!(st.undo.len(),start+1,"never evict the group baseline");
            assert_eq!(st.undo.len(),MAX_UNDO);
            assert!(std::sync::Arc::ptr_eq(&st.undo[start].doc,&original));
        }
        assert!(st.end_undo_group());
        assert_eq!(st.undo.len(),MAX_UNDO);
        assert!(std::sync::Arc::ptr_eq(&st.undo.last().unwrap().doc,&original));
    }

    #[test]
    fn stage280_native_line_commands_near_undo_limit_preserve_source_and_redo() {
        use crate::Session;
        use serde_json::json;
        let mut session=Session::default();
        let earlier=session.execute("line",&json!({"points":[[5000.0,5000.0],[5200.0,5000.0]]})).unwrap();
        let earlier_handle=cadcraft_doc::Handle::parse_hex(earlier["handles"][0].as_str().unwrap()).unwrap();
        // Fill history with lightweight snapshots without growing the actual drawing.
        // Following operations use the genuine Session::execute CAD command and Undo/Redo.
        while session.state().unwrap().undo.len()<MAX_UNDO {
            let previous={
                let state=session.state().unwrap();
                Snapshot {label:"prior-history".into(),doc:state.doc.clone(),selection:Vec::new()}
            };
            session.state_mut().unwrap().push_undo(previous);
        }
        assert_eq!(session.state().unwrap().undo.len(),MAX_UNDO);
        let baseline=session.doc().unwrap().entity_count();
        session.execute("undo",&json!({"mode":"begin"})).unwrap();
        let mut native_handles=Vec::new();
        for n in 0..96 {
            let y=6000.0+f64::from(n)*50.0;
            let response=session.execute("line",&json!({"points":[[100.0,y],[900.0,y]]})).unwrap();
            let handle=cadcraft_doc::Handle::parse_hex(response["handles"][0].as_str().unwrap()).unwrap();
            native_handles.push(handle);
            assert_eq!(session.state().unwrap().undo.len(),MAX_UNDO);
            assert_eq!(session.state().unwrap().undo_ctl.group,Some(MAX_UNDO-1));
        }
        assert_eq!(session.doc().unwrap().entity_count(),baseline+native_handles.len());
        assert!(session.execute("undo",&json!({"mode":"end"})).is_ok());
        assert!(session.state().unwrap().undo_ctl.group.is_none());
        assert_eq!(session.state().unwrap().undo.len(),MAX_UNDO);
        session.execute("undo",&json!({"count":1})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),baseline);
        assert!(session.doc().unwrap().entity(earlier_handle).is_some());
        for h in &native_handles {assert!(session.doc().unwrap().entity(*h).is_none());}
        session.execute("redo",&json!({})).unwrap();
        assert_eq!(session.doc().unwrap().entity_count(),baseline+native_handles.len());
        assert!(session.doc().unwrap().entity(earlier_handle).is_some());
        for h in &native_handles {assert!(session.doc().unwrap().entity(*h).is_some());}
    }

    #[test]
    fn stage280_native_zero_mutation_group_near_history_limit_never_undoes_prior_line() {
        use crate::Session;
        use serde_json::json;
        let mut session=Session::default();
        let earlier=session.execute("line",&json!({"points":[[5000.0,5000.0],[5100.0,5000.0]]})).unwrap();
        let h=cadcraft_doc::Handle::parse_hex(earlier["handles"][0].as_str().unwrap()).unwrap();
        while session.state().unwrap().undo.len()<MAX_UNDO {
            let snapshot={
                let state=session.state().unwrap();
                Snapshot {label:"prior-history".into(),doc:state.doc.clone(),selection:Vec::new()}
            };
            session.state_mut().unwrap().push_undo(snapshot);
        }
        session.execute("undo",&json!({"mode":"begin"})).unwrap();
        let response=session.execute("erase",&json!({"handles":["FFFFFFFFFFFFFFFE"]})).unwrap();
        assert_eq!(response["erased"],0);
        assert_eq!(session.state().unwrap().undo_ctl.group,Some(MAX_UNDO));
        assert_eq!(session.state().unwrap().undo.len(),MAX_UNDO);
        session.execute("undo",&json!({"mode":"end"})).unwrap();
        assert_eq!(session.state().unwrap().undo.len(),MAX_UNDO);
        assert!(session.doc().unwrap().entity(h).is_some());
    }

    #[test]
    fn stage279_group_with_many_more_steps_than_history_limit_remains_undoable() {
        let mut st=DocState::new(Drawing::default(),"Stage279",None);
        st.begin_undo_group();
        let original=st.doc.clone();
        for _ in 0..(MAX_UNDO+32) {
            let snap=snapshot(&st,"large raster import");
            st.push_undo(snap);
            assert_eq!(st.undo.len(),1);
            assert_eq!(st.undo_ctl.group,Some(0));
        }
        assert!(st.end_undo_group());
        assert_eq!(st.undo.len(),1);
        assert!(std::sync::Arc::ptr_eq(&st.undo[0].doc,&original));
    }
}
