use crate::mechanical_port::source::{
    input::{
        focus_manager::{RuntimeFocusManagerHandle, RuntimeFocusManagerWeakHandle},
        focusable::{Focusable, Key, KeyModifiers},
    },
    semantic::semantic_snapshot::Bounds,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub type FocusNodeRef = Rc<RefCell<FocusNode>>;
pub type FocusableRef = Rc<RefCell<dyn Focusable>>;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum EdgeBehavior {
    #[default]
    ParentScope = 0,
    ClosedLoop = 1,
    Stop = 2,
    Unknown = 3,
}
pub struct FocusNode {
    pub(crate) focusable: Option<FocusableRef>,
    pub(crate) parent: Weak<RefCell<FocusNode>>,
    pub(crate) manager: RuntimeFocusManagerWeakHandle,
    pub(crate) children: Vec<FocusNodeRef>,
    pub name: String,
    pub world_bounds: Bounds,
    flags: u8,
    tab_index: i16,
    #[cfg(feature = "tools")]
    pub is_collapsed: bool,
}
impl FocusNode {
    const CAN_FOCUS: u8 = 1;
    const CAN_TOUCH: u8 = 2;
    const CAN_TRAVERSE: u8 = 4;
    const HAS_FOCUS: u8 = 32;
    pub fn new(focusable: Option<FocusableRef>) -> FocusNodeRef {
        Rc::new(RefCell::new(Self {
            focusable,
            parent: Weak::new(),
            manager: RuntimeFocusManagerWeakHandle::default(),
            children: Vec::new(),
            name: String::new(),
            world_bounds: Bounds::default(),
            flags: 7,
            tab_index: 0,
            #[cfg(feature = "tools")]
            is_collapsed: false,
        }))
    }
    pub fn make_structural_scope() -> FocusNodeRef {
        let n = Self::new(None);
        {
            let mut n = n.borrow_mut();
            n.set_can_focus(false);
            n.set_can_traverse(false);
            n.set_can_touch(false);
        }
        n
    }
    pub fn focusable(&self) -> Option<FocusableRef> {
        self.focusable.clone()
    }
    pub fn set_focusable(&mut self, focusable: Option<FocusableRef>) {
        let backing_changed = self.focusable.is_none() != focusable.is_none();
        self.focusable = focusable;
        if backing_changed {
            self.invalidate_focusable_content();
        }
    }
    pub fn clear_focusable(&mut self) {
        self.set_focusable(None);
    }
    fn flag(&self, f: u8) -> bool {
        self.flags & f != 0
    }
    fn set_flag(&mut self, f: u8, v: bool) {
        if v { self.flags |= f } else { self.flags &= !f }
    }
    pub fn can_focus(&self) -> bool {
        self.flag(Self::CAN_FOCUS)
    }
    pub fn set_can_focus(&mut self, v: bool) {
        if self.can_focus() != v {
            self.set_flag(Self::CAN_FOCUS, v);
            self.invalidate_focusable_content();
        }
    }
    pub fn can_touch(&self) -> bool {
        self.flag(Self::CAN_TOUCH)
    }
    pub fn set_can_touch(&mut self, v: bool) {
        self.set_flag(Self::CAN_TOUCH, v)
    }
    pub fn can_traverse(&self) -> bool {
        self.flag(Self::CAN_TRAVERSE)
    }
    pub fn set_can_traverse(&mut self, v: bool) {
        self.set_flag(Self::CAN_TRAVERSE, v)
    }
    pub fn has_focus(&self) -> bool {
        self.flag(Self::HAS_FOCUS)
    }
    pub(crate) fn set_has_focus(&mut self, v: bool) {
        self.set_flag(Self::HAS_FOCUS, v)
    }
    pub fn edge_behavior(&self) -> EdgeBehavior {
        match (self.flags >> 3) & 3 {
            1 => EdgeBehavior::ClosedLoop,
            2 => EdgeBehavior::Stop,
            3 => EdgeBehavior::Unknown,
            _ => EdgeBehavior::ParentScope,
        }
    }
    pub fn set_edge_behavior(&mut self, v: EdgeBehavior) {
        self.set_edge_behavior_raw(v as u8)
    }
    pub(crate) fn set_edge_behavior_raw(&mut self, value: u8) {
        self.flags = (self.flags & !(3 << 3)) | (value << 3)
    }
    pub fn tab_index(&self) -> i32 {
        self.tab_index as i32
    }
    pub fn set_tab_index(&mut self, v: i32) {
        self.tab_index = v as i16
    }
    pub fn has_world_bounds(&self) -> bool {
        !self.world_bounds.is_empty_or_nan()
    }
    pub fn clear_world_bounds(&mut self) {
        self.world_bounds = Bounds::default()
    }
    pub fn parent(&self) -> Option<FocusNodeRef> {
        self.parent.upgrade()
    }
    pub fn children(&self) -> &[FocusNodeRef] {
        &self.children
    }
    pub fn manager(&self) -> Option<RuntimeFocusManagerHandle> {
        self.manager.upgrade()
    }
    fn invalidate_focusable_content(&self) {
        self.manager.invalidate_focusable_content();
    }
    pub fn is_scope(&self) -> bool {
        !self.children.is_empty()
    }
    pub fn add_child(parent: &FocusNodeRef, child: FocusNodeRef) {
        let index = parent.borrow().children.len();
        Self::insert_child(parent, index, child)
    }
    pub fn insert_child(parent: &FocusNodeRef, index: usize, child: FocusNodeRef) {
        Self::remove_from_parent(&child);
        child.borrow_mut().parent = Rc::downgrade(parent);
        let mut parent = parent.borrow_mut();
        let index = index.min(parent.children.len());
        parent.children.insert(index, child);
        parent.invalidate_focusable_content();
    }
    pub fn remove_child(parent: &FocusNodeRef, child: &FocusNodeRef) {
        if child
            .borrow()
            .parent()
            .as_ref()
            .is_none_or(|actual_parent| !Rc::ptr_eq(actual_parent, parent))
        {
            return;
        }
        child.borrow_mut().parent = Weak::new();
        let mut parent = parent.borrow_mut();
        if let Some(index) = parent.children.iter().position(|n| Rc::ptr_eq(n, child)) {
            parent.children.remove(index);
        }
        parent.invalidate_focusable_content();
    }
    pub fn remove_from_parent(child: &FocusNodeRef) {
        let parent = child.borrow().parent();
        if let Some(parent) = parent {
            Self::remove_child(&parent, child);
        }
    }
    pub fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
        self.focusable
            .as_ref()
            .is_some_and(|v| v.borrow_mut().key_input(k, m, p, r))
    }
    pub fn text_input(&mut self, t: &str) -> bool {
        self.focusable
            .as_ref()
            .is_some_and(|v| v.borrow_mut().text_input(t))
    }
    pub fn focused(&mut self) {
        if let Some(f) = &self.focusable {
            f.borrow_mut().focused()
        }
    }
    pub fn blurred(&mut self) {
        if let Some(f) = &self.focusable {
            f.borrow_mut().blurred()
        }
    }
}
