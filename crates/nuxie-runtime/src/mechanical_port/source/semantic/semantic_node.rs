use crate::mechanical_port::source::{
    core::CoreHandle,
    semantic::{
        semantic_manager::{RuntimeSemanticManagerHandle, RuntimeSemanticManagerWeakHandle},
        semantic_provider::semantic_source_is_visible,
        semantic_snapshot::Bounds,
        semantic_state::SemanticState,
    },
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type SemanticNodeRef = Rc<RefCell<SemanticNode>>;

pub struct SemanticNode {
    pub(crate) id: u32,
    pub(crate) parent: Weak<RefCell<SemanticNode>>,
    pub(crate) children: Vec<SemanticNodeRef>,
    pub(crate) manager: Option<RuntimeSemanticManagerWeakHandle>,
    pub role: u32,
    pub state_flags: u32,
    pub label: String,
    pub value: String,
    pub hint: String,
    pub heading_level: u32,
    pub bounds: Bounds,
    pub trait_flags: u32,
    pub core_owner: Option<CoreHandle>,
    pub is_boundary_node: bool,
    /// Owner used for actions/focus and live hidden-state checks while flattening.
    /// Boundary nodes have no SemanticData owner.
    pub semantic_data: Option<CoreHandle>,
    pub boundary_artboard: Option<CoreHandle>,
}
impl SemanticNode {
    /// Actions require a live, attached path through one semantic occurrence.
    /// Recheck this at dispatch: visibility or membership can change after
    /// a listener has been queued without changing the target's own flags.
    pub fn is_action_eligible(node: &SemanticNodeRef) -> bool {
        if node
            .borrow()
            .core_owner
            .as_ref()
            .is_some_and(|owner| !semantic_source_is_visible(owner))
        {
            return false;
        }
        Self::attached_without(node, SemanticState::DISABLED.0 | SemanticState::HIDDEN.0)
    }

    /// Nuxie: a reveal (`semantic_reveal`) needs the node in one live tree
    /// and not hidden, itself or above. Unlike an action it does not ask
    /// whether the node is drawn (clipped away, or under opacity 0) or
    /// disabled: a host reveals a node to show it, and a disabled control
    /// out of view is still read.
    pub fn is_reveal_eligible(node: &SemanticNodeRef) -> bool {
        Self::attached_without(node, SemanticState::HIDDEN.0)
    }

    /// The node and each ancestor belong to its manager and carry none of
    /// `flags`, and its root is the manager's. Each node is visited once; a
    /// cycle answers false.
    fn attached_without(node: &SemanticNodeRef, flags: u32) -> bool {
        let Some(manager) = node.borrow().manager() else {
            return false;
        };
        let mut seen = std::collections::HashSet::new();
        let mut ancestor = Some(node.clone());
        while let Some(current) = ancestor {
            if !seen.insert(Rc::as_ptr(&current)) {
                return false;
            }
            let entry = current.borrow();
            if !entry.manager().is_some_and(|owner| owner.ptr_eq(&manager))
                || entry.state_flags & flags != 0
            {
                return false;
            }
            ancestor = entry.parent();
            if ancestor.is_none() {
                return manager.with_semantic_manager(|manager| manager.contains_root(&current));
            }
        }
        true
    }

    pub fn new(id: u32) -> SemanticNodeRef {
        Rc::new(RefCell::new(Self {
            id,
            parent: Weak::new(),
            children: Vec::new(),
            manager: None,
            role: 0,
            state_flags: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            bounds: Bounds {
                min_x: f32::MAX,
                min_y: f32::MAX,
                max_x: -f32::MAX,
                max_y: -f32::MAX,
            },
            trait_flags: 0,
            core_owner: None,
            is_boundary_node: false,
            semantic_data: None,
            boundary_artboard: None,
        }))
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn parent(&self) -> Option<SemanticNodeRef> {
        self.parent.upgrade()
    }
    pub fn children(&self) -> &[SemanticNodeRef] {
        &self.children
    }
    pub fn manager(&self) -> Option<RuntimeSemanticManagerHandle> {
        self.manager
            .as_ref()
            .and_then(RuntimeSemanticManagerWeakHandle::upgrade)
    }
}
