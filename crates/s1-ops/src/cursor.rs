//! Cursor position and selection types.
//!
//! Represents where the user's caret is in the document, and optionally
//! a selection range. Used by higher-level editing APIs.

use s1_model::NodeId;

/// A position in the document — a specific point within a text node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// The text node containing the cursor.
    pub node_id: NodeId,
    /// Character offset within the text node.
    pub offset: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(node_id: NodeId, offset: usize) -> Self {
        Self { node_id, offset }
    }
}

/// A selection range in the document.
///
/// When `anchor == focus`, this is a collapsed selection (a simple cursor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    /// The anchor (start) of the selection.
    pub anchor: Position,
    /// The focus (end) of the selection. Equals anchor for a collapsed selection.
    pub focus: Position,
}

impl Selection {
    /// A collapsed selection (cursor with no range).
    pub fn collapsed(pos: Position) -> Self {
        Self {
            anchor: pos,
            focus: pos,
        }
    }

    /// A range selection from anchor to focus.
    pub fn range(anchor: Position, focus: Position) -> Self {
        Self { anchor, focus }
    }

    /// Returns `true` if this is a collapsed selection (no range).
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    /// Returns the node IDs involved in this selection.
    /// For a collapsed selection, returns one ID. For a range, may return two.
    pub fn node_ids(&self) -> Vec<NodeId> {
        if self.anchor.node_id == self.focus.node_id {
            vec![self.anchor.node_id]
        } else {
            vec![self.anchor.node_id, self.focus.node_id]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_selection() {
        let pos = Position::new(NodeId::new(0, 5), 3);
        let sel = Selection::collapsed(pos);
        assert!(sel.is_collapsed());
        assert_eq!(sel.anchor, sel.focus);
        assert_eq!(sel.node_ids().len(), 1);
    }

    #[test]
    fn range_selection_same_node() {
        let anchor = Position::new(NodeId::new(0, 5), 0);
        let focus = Position::new(NodeId::new(0, 5), 10);
        let sel = Selection::range(anchor, focus);
        assert!(!sel.is_collapsed());
        assert_eq!(sel.node_ids().len(), 1);
    }

    #[test]
    fn range_selection_different_nodes() {
        let anchor = Position::new(NodeId::new(0, 5), 0);
        let focus = Position::new(NodeId::new(0, 8), 3);
        let sel = Selection::range(anchor, focus);
        assert!(!sel.is_collapsed());
        assert_eq!(sel.node_ids().len(), 2);
    }

    #[test]
    fn position_equality() {
        let a = Position::new(NodeId::new(0, 1), 5);
        let b = Position::new(NodeId::new(0, 1), 5);
        let c = Position::new(NodeId::new(0, 1), 6);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
