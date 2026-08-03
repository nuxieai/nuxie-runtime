//! Direct retained-node port of pinned src/input/focus_node.cpp (B6-0239 / C12).

use std::sync::atomic::{AtomicU64, Ordering};

use super::focusable::RuntimeFocusable;

/// Stable identity for one node in a focus tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusNodeId(u64);

impl FocusNodeId {
    pub(crate) fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusPoint {
    pub x: f32,
    pub y: f32,
}

impl FocusPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl FocusBounds {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    pub fn center(self) -> FocusPoint {
        FocusPoint::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }

    fn is_valid(self) -> bool {
        [self.min_x, self.min_y, self.max_x, self.max_y]
            .into_iter()
            .all(f32::is_finite)
            && self.min_x < self.max_x
            && self.min_y < self.max_y
    }
}

/// Runtime state for one authored focus target.
#[derive(Debug, Clone)]
pub struct FocusNode {
    pub(crate) parent: Option<FocusNodeId>,
    pub(crate) children: Vec<FocusNodeId>,
    pub(crate) focusable: Option<RuntimeFocusable>,
    can_focus: bool,
    can_touch: bool,
    can_traverse: bool,
    eligible: bool,
    tab_index: i16,
    name: Vec<u8>,
    edge_behavior: FocusEdgeBehavior,
    bounds: Option<FocusBounds>,
    position: Option<FocusPoint>,
    pub(crate) has_focus: bool,
}

impl FocusNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            focusable: None,
            can_focus: true,
            can_touch: true,
            can_traverse: true,
            eligible: true,
            tab_index: 0,
            name: Vec::new(),
            edge_behavior: FocusEdgeBehavior::ParentScope,
            bounds: None,
            position: None,
            has_focus: false,
        }
    }

    pub fn structural_scope() -> Self {
        Self {
            focusable: None,
            can_focus: false,
            can_touch: false,
            can_traverse: false,
            ..Self::new()
        }
    }

    pub fn can_focus(&self) -> bool {
        self.can_focus
    }

    pub fn set_can_focus(&mut self, value: bool) {
        self.can_focus = value;
    }

    pub fn can_touch(&self) -> bool {
        self.can_touch
    }

    pub fn set_can_touch(&mut self, value: bool) {
        self.can_touch = value;
    }

    pub fn can_traverse(&self) -> bool {
        self.can_traverse
    }

    pub fn set_can_traverse(&mut self, value: bool) {
        self.can_traverse = value;
    }

    pub fn is_eligible(&self) -> bool {
        self.eligible
    }

    pub fn set_eligible(&mut self, value: bool) {
        self.eligible = value;
    }

    pub fn tab_index(&self) -> i16 {
        self.tab_index
    }

    pub fn set_tab_index(&mut self, value: i16) {
        self.tab_index = value;
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn set_name(&mut self, value: impl Into<Vec<u8>>) {
        self.name = value.into();
    }

    pub fn edge_behavior(&self) -> FocusEdgeBehavior {
        self.edge_behavior
    }

    pub fn set_edge_behavior(&mut self, value: FocusEdgeBehavior) {
        self.edge_behavior = value;
    }

    pub fn bounds(&self) -> Option<FocusBounds> {
        self.bounds
    }

    pub fn set_bounds(&mut self, value: Option<FocusBounds>) {
        self.bounds = value.filter(|bounds| bounds.is_valid());
    }

    pub fn position(&self) -> Option<FocusPoint> {
        self.position
    }

    pub fn set_position(&mut self, value: Option<FocusPoint>) {
        self.position = value.filter(|point| point.x.is_finite() && point.y.is_finite());
    }

    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub(crate) fn focusable(&self) -> Option<RuntimeFocusable> {
        self.focusable
    }

    pub(crate) fn set_focusable(&mut self, focusable: RuntimeFocusable) {
        self.focusable = Some(focusable);
    }

    pub(crate) fn clear_focusable(&mut self) {
        self.focusable = None;
    }

    // Direct `focus_node.cpp` link mutations. FocusManager resolves stable
    // arena ids, while FocusNode owns the parent/children writes themselves.
    pub(crate) fn set_parent(&mut self, parent: Option<FocusNodeId>) {
        self.parent = parent;
    }

    pub(crate) fn insert_child(&mut self, index: usize, child: FocusNodeId) {
        self.children.retain(|candidate| *candidate != child);
        self.children.insert(index.min(self.children.len()), child);
    }

    pub(crate) fn remove_child(&mut self, child: FocusNodeId) {
        self.children.retain(|candidate| *candidate != child);
    }
}

impl Default for FocusNode {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusEdgeBehavior {
    #[default]
    ParentScope,
    ClosedLoop,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEventKind {
    Focused,
    Blurred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEvent {
    pub node_id: FocusNodeId,
    pub kind: FocusEventKind,
}

impl FocusEvent {
    pub fn new(node_id: FocusNodeId, kind: FocusEventKind) -> Self {
        Self { node_id, kind }
    }
}
