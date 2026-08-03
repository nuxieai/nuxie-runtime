// Pinned C++ correspondence (d788e8ec):
// include/rive/semantic/semantic_snapshot.hpp:1-114.

use crate::semantic_node::SemanticBounds;

/// Flattened semantic node payload emitted in an incremental tree diff.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticsDiffNode {
    pub id: u32,
    pub role: u32,
    pub label: String,
    pub value: String,
    pub hint: String,
    pub state_flags: u32,
    pub trait_flags: u32,
    pub heading_level: u32,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    /// `-1` denotes a root.
    pub parent_id: i32,
    /// Position under the parent, or root position when `parent_id == -1`.
    pub sibling_index: u32,
}

impl SemanticsDiffNode {
    pub fn bounds(&self) -> SemanticBounds {
        SemanticBounds::new(self.min_x, self.min_y, self.max_x, self.max_y)
    }

    pub fn set_bounds(&mut self, value: SemanticBounds) {
        self.min_x = value.min_x;
        self.min_y = value.min_y;
        self.max_x = value.max_x;
        self.max_y = value.max_y;
    }
}
/// Authoritative ordered child list for one semantic parent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticsChildrenUpdate {
    /// `-1` denotes the roots list.
    pub parent_id: i32,
    pub child_ids: Vec<u32>,
}

/// Allocation-light geometry-only semantic update.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SemanticsBoundsUpdate {
    pub id: u32,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl SemanticsBoundsUpdate {
    pub fn bounds(&self) -> SemanticBounds {
        SemanticBounds::new(self.min_x, self.min_y, self.max_x, self.max_y)
    }

    pub fn set_bounds(&mut self, value: SemanticBounds) {
        self.min_x = value.min_x;
        self.min_y = value.min_y;
        self.max_x = value.max_x;
        self.max_y = value.max_y;
    }
}

/// Incremental accessibility-tree delta.
///
/// Arrays preserve pinned C++ ordering: current-tree pre-order for added,
/// moved, semantic, and geometry updates; previous-tree pre-order for removals.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticsDiff {
    pub frame_number: u64,
    pub tree_version: u64,
    pub root_id: u32,
    pub removed: Vec<u32>,
    pub added: Vec<SemanticsDiffNode>,
    pub moved: Vec<SemanticsDiffNode>,
    pub children_updated: Vec<SemanticsChildrenUpdate>,
    pub updated_semantic: Vec<SemanticsDiffNode>,
    pub updated_geometry: Vec<SemanticsBoundsUpdate>,
}

impl SemanticsDiff {
    pub fn is_empty(&self) -> bool {
        self.removed.is_empty()
            && self.added.is_empty()
            && self.moved.is_empty()
            && self.children_updated.is_empty()
            && self.updated_semantic.is_empty()
            && self.updated_geometry.is_empty()
    }
}
