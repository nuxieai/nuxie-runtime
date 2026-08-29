#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Clone, Debug, PartialEq)]
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
    pub parent_id: i32,
    pub sibling_index: u32,
}
impl Default for SemanticsDiffNode {
    fn default() -> Self {
        Self {
            id: 0,
            role: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            state_flags: 0,
            trait_flags: 0,
            heading_level: 0,
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
            parent_id: -1,
            sibling_index: 0,
        }
    }
}

impl SemanticsDiffNode {
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min_x: self.min_x,
            min_y: self.min_y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }
    pub fn set_bounds(&mut self, value: Bounds) {
        self.min_x = value.min_x;
        self.min_y = value.min_y;
        self.max_x = value.max_x;
        self.max_y = value.max_y;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticsChildrenUpdate {
    pub parent_id: i32,
    pub child_ids: Vec<u32>,
}
impl Default for SemanticsChildrenUpdate {
    fn default() -> Self {
        Self {
            parent_id: -1,
            child_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SemanticsBoundsUpdate {
    pub id: u32,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}
impl SemanticsBoundsUpdate {
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min_x: self.min_x,
            min_y: self.min_y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }
    pub fn set_bounds(&mut self, value: Bounds) {
        self.min_x = value.min_x;
        self.min_y = value.min_y;
        self.max_x = value.max_x;
        self.max_y = value.max_y;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
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
