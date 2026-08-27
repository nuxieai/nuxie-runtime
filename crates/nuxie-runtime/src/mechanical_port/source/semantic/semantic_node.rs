use crate::mechanical_port::source::semantic::semantic_snapshot::Bounds;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type SemanticNodeRef = Rc<RefCell<SemanticNode>>;

pub struct SemanticNode {
    pub(crate) id: u32,
    pub(crate) parent: Weak<RefCell<SemanticNode>>,
    pub(crate) children: Vec<SemanticNodeRef>,
    pub role: u32,
    pub state_flags: u32,
    pub label: String,
    pub value: String,
    pub hint: String,
    pub heading_level: u32,
    pub bounds: Bounds,
    pub trait_flags: u32,
    pub core_owner: Option<usize>,
    pub is_boundary_node: bool,
    pub semantic_data: Option<usize>,
    pub boundary_artboard: Option<usize>,
}
impl SemanticNode {
    pub fn new(id: u32) -> SemanticNodeRef {
        Rc::new(RefCell::new(Self {
            id,
            parent: Weak::new(),
            children: Vec::new(),
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
}
