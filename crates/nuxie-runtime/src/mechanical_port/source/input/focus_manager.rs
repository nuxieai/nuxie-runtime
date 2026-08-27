use crate::mechanical_port::source::input::{
    focus_node::{EdgeBehavior, FocusNode, FocusNodeRef},
    focusable::{Key, KeyModifiers},
};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
pub struct FocusManager {
    primary_focus: Option<FocusNodeRef>,
    root_nodes: Vec<FocusNodeRef>,
    has_focusable_content: bool,
    focusable_content_dirty: bool,
}
impl Default for FocusManager {
    fn default() -> Self {
        Self {
            primary_focus: None,
            root_nodes: Vec::new(),
            has_focusable_content: false,
            focusable_content_dirty: true,
        }
    }
}
impl FocusManager {
    pub fn primary_focus(&self) -> Option<FocusNodeRef> {
        self.primary_focus.clone()
    }
    pub fn has_primary_focus(&self, n: &FocusNodeRef) -> bool {
        self.primary_focus
            .as_ref()
            .is_some_and(|f| Rc::ptr_eq(f, n))
    }
    pub fn has_focus(&self, n: &FocusNodeRef) -> bool {
        n.borrow().has_focus()
    }
    pub fn set_focus(&mut self, node: FocusNodeRef) {
        if self.has_primary_focus(&node) {
            return;
        }
        if !node.borrow().can_focus()
            || !node
                .borrow()
                .focusable
                .as_ref()
                .is_some_and(|f| f.borrow().is_eligible_for_focus_traversal())
        {
            return;
        }
        let old = self.primary_focus.take();
        if let Some(old) = &old {
            old.borrow_mut().blurred();
            let mut p = Some(old.clone());
            while let Some(n) = p {
                n.borrow_mut().set_has_focus(false);
                p = n.borrow().parent();
            }
        }
        let mut p = Some(node.clone());
        while let Some(n) = p {
            n.borrow_mut().set_has_focus(true);
            p = n.borrow().parent();
        }
        node.borrow_mut().focused();
        self.primary_focus = Some(node)
    }
    pub fn clear_focus(&mut self) {
        if let Some(old) = self.primary_focus.take() {
            old.borrow_mut().blurred();
            let mut p = Some(old);
            while let Some(n) = p {
                n.borrow_mut().set_has_focus(false);
                p = n.borrow().parent();
            }
        }
    }
    pub fn add_child(
        &mut self,
        parent: Option<FocusNodeRef>,
        child: FocusNodeRef,
        index: Option<usize>,
    ) {
        self.detach_child(&child);
        if let Some(parent) = parent {
            FocusNode::insert_child(
                &parent,
                index.unwrap_or_else(|| parent.borrow().children().len()),
                child,
            )
        } else {
            let i = index
                .unwrap_or(self.root_nodes.len())
                .min(self.root_nodes.len());
            self.root_nodes.insert(i, child);
        }
        self.focusable_content_dirty = true
    }
    pub fn detach_child(&mut self, child: &FocusNodeRef) {
        if let Some(parent) = child.borrow().parent() {
            FocusNode::remove_child(&parent, child)
        } else {
            self.root_nodes.retain(|n| !Rc::ptr_eq(n, child));
        }
        self.focusable_content_dirty = true
    }
    fn contains(root: &FocusNodeRef, target: &FocusNodeRef) -> bool {
        Rc::ptr_eq(root, target)
            || root
                .borrow()
                .children()
                .iter()
                .any(|c| Self::contains(c, target))
    }
    pub fn remove_child(&mut self, child: &FocusNodeRef) {
        if self
            .primary_focus
            .as_ref()
            .is_some_and(|f| Self::contains(child, f))
        {
            self.clear_focus()
        }
        self.detach_child(child)
    }
    fn gather(nodes: &[FocusNodeRef], out: &mut Vec<FocusNodeRef>) {
        let mut sorted = nodes.to_vec();
        sorted.sort_by_key(|n| n.borrow().tab_index());
        for n in sorted {
            let b = n.borrow();
            let backed = b.focusable.is_some();
            let eligible = backed
                && b.can_focus()
                && b.can_traverse()
                && b.focusable
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .is_eligible_for_focus_traversal();
            let children = b.children().to_vec();
            drop(b);
            if eligible {
                out.push(n.clone())
            }
            Self::gather(&children, out);
        }
    }
    pub fn get_traversable_nodes(&self, scope: Option<&FocusNodeRef>) -> Vec<FocusNodeRef> {
        let mut out = Vec::new();
        match scope {
            Some(s) => Self::gather(s.borrow().children(), &mut out),
            None => Self::gather(&self.root_nodes, &mut out),
        }
        out
    }
    fn focus_linear(&mut self, forward: bool) -> bool {
        let list = self.get_traversable_nodes(None);
        if list.is_empty() {
            return false;
        }
        let i = self
            .primary_focus
            .as_ref()
            .and_then(|f| list.iter().position(|n| Rc::ptr_eq(n, f)));
        let next = if forward {
            i.map_or(0, |i| (i + 1) % list.len())
        } else {
            i.map_or(
                list.len() - 1,
                |i| if i == 0 { list.len() - 1 } else { i - 1 },
            )
        };
        self.set_focus(list[next].clone());
        true
    }
    pub fn focus_next(&mut self) -> bool {
        self.focus_linear(true)
    }
    pub fn focus_previous(&mut self) -> bool {
        self.focus_linear(false)
    }
    fn center(n: &FocusNodeRef) -> Option<(f32, f32, f32, f32)> {
        let b = n.borrow();
        let bounds = if b.has_world_bounds() {
            Some(b.world_bounds)
        } else {
            b.focusable.as_ref().and_then(|f| f.borrow().world_bounds())
        }?;
        Some((
            (bounds.min_x + bounds.max_x) / 2.0,
            (bounds.min_y + bounds.max_y) / 2.0,
            bounds.max_x - bounds.min_x,
            bounds.max_y - bounds.min_y,
        ))
    }
    fn focus_direction(&mut self, d: Direction) -> bool {
        let Some(cur) = self.primary_focus.clone() else {
            return self.focus_next();
        };
        let Some(c) = Self::center(&cur) else {
            return false;
        };
        let mut candidates: Vec<_> = self
            .get_traversable_nodes(None)
            .into_iter()
            .filter(|n| !Rc::ptr_eq(n, &cur))
            .filter_map(|n| {
                let p = Self::center(&n)?;
                let (dx, dy) = (p.0 - c.0, p.1 - c.1);
                let valid = match d {
                    Direction::Left => dx < 0.0,
                    Direction::Right => dx > 0.0,
                    Direction::Up => dy < 0.0,
                    Direction::Down => dy > 0.0,
                };
                if !valid {
                    return None;
                }
                let (primary, cross) = match d {
                    Direction::Left | Direction::Right => (dx.abs(), dy.abs()),
                    _ => (dy.abs(), dx.abs()),
                };
                Some((primary + cross * 0.5, n))
            })
            .collect();
        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        if let Some((_, n)) = candidates.into_iter().next() {
            self.set_focus(n);
            true
        } else {
            match cur.borrow().edge_behavior() {
                EdgeBehavior::ClosedLoop => {
                    self.focus_linear(matches!(d, Direction::Right | Direction::Down))
                }
                _ => false,
            }
        }
    }
    pub fn focus_left(&mut self) -> bool {
        self.focus_direction(Direction::Left)
    }
    pub fn focus_right(&mut self) -> bool {
        self.focus_direction(Direction::Right)
    }
    pub fn focus_up(&mut self) -> bool {
        self.focus_direction(Direction::Up)
    }
    pub fn focus_down(&mut self) -> bool {
        self.focus_direction(Direction::Down)
    }
    pub fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
        self.primary_focus
            .as_ref()
            .is_some_and(|n| n.borrow_mut().key_input(k, m, p, r))
    }
    pub fn text_input(&mut self, t: &str) -> bool {
        self.primary_focus
            .as_ref()
            .is_some_and(|n| n.borrow_mut().text_input(t))
    }
    pub fn gamepad_dispatch(&mut self, i: &dyn core::any::Any) -> bool {
        let mut n = self.primary_focus.clone();
        while let Some(v) = n {
            if v.borrow()
                .focusable
                .as_ref()
                .is_some_and(|f| f.borrow_mut().gamepad_dispatch(i))
            {
                return true;
            }
            n = v.borrow().parent();
        }
        false
    }
    pub fn mark_focusable_content_dirty(&mut self) {
        self.focusable_content_dirty = true
    }
    pub fn has_focusable_content(&mut self) -> bool {
        if self.focusable_content_dirty {
            fn any(ns: &[FocusNodeRef]) -> bool {
                ns.iter().any(|n| {
                    let b = n.borrow();
                    b.focusable.is_some() || b.can_focus() || any(b.children())
                })
            }
            self.has_focusable_content = any(&self.root_nodes);
            self.focusable_content_dirty = false;
        }
        self.has_focusable_content
    }
}
impl Drop for FocusManager {
    fn drop(&mut self) {
        self.clear_focus();
    }
}
