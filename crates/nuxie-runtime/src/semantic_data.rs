// Pinned C++ correspondence (4ac7b327):
// src/semantic/semantic_data.cpp:1-572 and
// include/rive/semantic/semantic_data.hpp:1-79.

use std::rc::Rc;

use crate::ArtboardInstance;
use crate::components::Mat2D;
use crate::semantic_manager::SemanticManager;
use crate::semantic_provider::{
    ResolvedSemanticData, SemanticProvider, semantic_string_property, semantic_uint_property,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SemanticActionType {
    Tap = 0,
    Increase = 1,
    Decrease = 2,
}

impl SemanticActionType {
    pub fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Tap,
            1 => Self::Increase,
            2 => Self::Decrease,
            _ => return None,
        })
    }
}

pub trait SemanticListener: std::fmt::Debug {
    fn on_semantic_tap(&self);
    fn on_semantic_increase(&self);
    fn on_semantic_decrease(&self);
}

#[derive(Debug, Clone)]
pub struct RuntimeSemanticData {
    pub local_id: usize,
    pub parent_local_id: Option<usize>,
    role: u32,
    label: String,
    value: String,
    hint: String,
    heading_level: u32,
    trait_flags: u32,
    state_flags: u32,
    semantic_node: Option<SemanticNodeHandle>,
    semantic_manager_identity: Option<u64>,
    tree_parent: Option<SemanticNodeHandle>,
    semantic_listeners: Vec<Rc<dyn SemanticListener>>,
    bounds_retry_pending: bool,
    excluded_from_tree: bool,
}

impl RuntimeSemanticData {
    pub fn new(local_id: usize, parent_local_id: Option<usize>) -> Self {
        Self {
            local_id,
            parent_local_id,
            role: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            trait_flags: 0,
            state_flags: 0,
            semantic_node: None,
            semantic_manager_identity: None,
            tree_parent: None,
            semantic_listeners: Vec::new(),
            bounds_retry_pending: false,
            excluded_from_tree: false,
        }
    }

    pub(crate) fn from_artboard(artboard: &ArtboardInstance, local_id: usize) -> Self {
        let mut data = Self::new(local_id, artboard.component_parent_local(local_id));
        data.role = semantic_uint_property(artboard, local_id, "role");
        data.label = semantic_string_property(artboard, local_id, "label");
        data.value = semantic_string_property(artboard, local_id, "value");
        data.hint = semantic_string_property(artboard, local_id, "hint");
        data.heading_level = semantic_uint_property(artboard, local_id, "headingLevel");
        data.trait_flags = semantic_uint_property(artboard, local_id, "traitFlags");
        data.state_flags = semantic_uint_property(artboard, local_id, "stateFlags");
        data
    }

    /// Refresh authored and provider-derived state on the retained node.
    ///
    /// The Focused bit is manager/focus-runtime state rather than an authored
    /// property, so a generated `stateFlags` refresh must preserve it. The
    /// Focusable trait is likewise projected from a retained sibling
    /// `FocusData` on every refresh instead of being snapshotted once.
    pub(crate) fn synchronize_from_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
        manager: &mut SemanticManager,
        root_transform: Mat2D,
    ) {
        let role = semantic_uint_property(artboard, self.local_id, "role");
        let label = semantic_string_property(artboard, self.local_id, "label");
        let value = semantic_string_property(artboard, self.local_id, "value");
        let hint = semantic_string_property(artboard, self.local_id, "hint");
        let heading_level = semantic_uint_property(artboard, self.local_id, "headingLevel");
        let authored_traits = semantic_uint_property(artboard, self.local_id, "traitFlags");
        let trait_flags = if self.parent_has_focus_data(artboard) {
            authored_traits | SemanticTrait::FOCUSABLE.0
        } else {
            authored_traits
        };
        let authored_states = semantic_uint_property(artboard, self.local_id, "stateFlags");
        let retained_focus = self.semantic_node.as_ref().map_or(0, |node| {
            node.borrow().state_flags() & SemanticState::FOCUSED.0
        });
        let state_flags = authored_states | retained_focus;

        self.set_role(role, Some(manager));
        self.set_label(label, Some(manager));
        self.set_value(value, Some(manager));
        self.set_hint(hint, Some(manager));
        self.set_heading_level(heading_level, Some(manager));
        self.set_trait_flags(trait_flags, Some(manager));
        self.set_state_flags(state_flags, Some(manager), artboard);
        self.apply_inferred_semantics_if_needed(artboard, Some(manager));
        self.update_world_bounds_with_root_transform(artboard, Some(manager), root_transform);
    }

    pub fn has_semantic_node(&self) -> bool {
        self.semantic_node.is_some()
    }

    pub fn semantic_id(&self) -> u32 {
        self.semantic_node
            .as_ref()
            .map_or(0, |node| node.borrow().id())
    }

    pub fn semantic_node(&mut self, artboard: &mut ArtboardInstance) -> SemanticNodeHandle {
        if let Some(node) = &self.semantic_node {
            return node.clone();
        }
        let node = SemanticNodeHandle::new(0);
        {
            let mut node = node.borrow_mut();
            node.set_core_owner_local_id(self.parent_local_id);
            node.set_semantic_data_local_id(Some(self.local_id));
            node.set_role(self.role);
            node.set_label(self.label.clone());
            node.set_value(self.value.clone());
            node.set_hint(self.hint.clone());
            node.set_heading_level(self.heading_level);
            node.set_state_flags(self.state_flags);
            let mut traits = self.trait_flags;
            if self.parent_has_focus_data(artboard) {
                traits |= SemanticTrait::FOCUSABLE.0;
            }
            node.set_trait_flags(traits);
        }
        self.semantic_node = Some(node.clone());
        self.apply_inferred_semantics_if_needed(artboard, None);
        self.bounds_retry_pending = true;
        self.update_world_bounds(artboard, None);
        node
    }

    pub(crate) fn prepare_for_tree(&mut self, artboard: &mut ArtboardInstance) {
        self.semantic_node(artboard);
        self.excluded_from_tree = self.should_exclude_from_tree(artboard);
    }

    pub fn node_handle(&self) -> Option<SemanticNodeHandle> {
        self.semantic_node.clone()
    }

    fn parent_has_focus_data(&self, artboard: &ArtboardInstance) -> bool {
        let Some(parent) = self
            .parent_local_id
            .and_then(|local| artboard.component(local))
        else {
            return false;
        };
        parent.children.iter().any(|child| {
            artboard
                .component_local_id(*child)
                .is_some_and(|local| artboard.runtime_object_type_name(local) == Some("FocusData"))
        })
    }

    pub fn attach(
        &mut self,
        manager: &mut SemanticManager,
        parent: Option<&SemanticNodeHandle>,
        artboard: &mut ArtboardInstance,
    ) -> u32 {
        let node = self.semantic_node(artboard);
        self.tree_parent = parent.cloned();
        self.semantic_manager_identity = Some(manager.identity());
        self.excluded_from_tree = self.should_exclude_from_tree(artboard);
        if self.excluded_from_tree {
            return node.borrow().id();
        }
        let id = manager.add_child(parent, node);
        id
    }

    /// Reconcile live exclusion and parentage for an already retained node.
    /// Collapsed layout state can change without touching `stateFlags`, and a
    /// mounted occurrence can acquire a different closest semantic ancestor,
    /// so both decisions must be revisited on every tree synchronization.
    pub(crate) fn reconcile_tree_membership(
        &mut self,
        manager: &mut SemanticManager,
        parent: Option<&SemanticNodeHandle>,
        artboard: &mut ArtboardInstance,
        root_transform: Mat2D,
    ) {
        let Some(node) = self.semantic_node.clone() else {
            return;
        };
        let exclude = self.should_exclude_from_tree(artboard);
        let attached = node.borrow().manager_identity() == Some(manager.identity());
        let current_parent_id = node.borrow().parent_id();
        let desired_parent_id = parent.map(|parent| parent.borrow().id());
        let parent_changed = current_parent_id != desired_parent_id;

        if attached && (exclude || parent_changed) {
            manager.remove_child(&node);
        }

        self.tree_parent = parent.cloned();
        self.excluded_from_tree = exclude;
        self.semantic_manager_identity = Some(manager.identity());
        if !exclude && node.borrow().manager_identity().is_none() {
            manager.add_child(parent, node);
            self.bounds_retry_pending = true;
            self.update_world_bounds_with_root_transform(artboard, Some(manager), root_transform);
            self.apply_inferred_semantics_if_needed(artboard, Some(manager));
        }
    }

    pub fn detach(&mut self, manager: &mut SemanticManager) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        if node.borrow().manager_identity() == Some(manager.identity()) {
            manager.remove_child(node);
        }
        self.semantic_manager_identity = None;
        self.tree_parent = None;
    }

    pub fn set_focused_state(&mut self, focused: bool, manager: Option<&mut SemanticManager>) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let mut flags = node.borrow().state_flags();
        if focused {
            flags |= SemanticState::FOCUSED.0;
        } else {
            flags &= !SemanticState::FOCUSED.0;
        }
        if node.borrow().state_flags() == flags {
            return;
        }
        node.borrow_mut().set_state_flags(flags);
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn set_role(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.role == value {
            return false;
        }
        self.role = value;
        self.update_node_content(manager, |node| node.set_role(value));
        true
    }

    pub fn set_label(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.label == value {
            return false;
        }
        self.label = value.clone();
        self.update_node_content(manager, |node| node.set_label(value));
        true
    }

    pub fn set_value(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.value == value {
            return false;
        }
        self.value = value.clone();
        self.update_node_content(manager, |node| node.set_value(value));
        true
    }

    pub fn set_hint(
        &mut self,
        value: impl Into<String>,
        manager: Option<&mut SemanticManager>,
    ) -> bool {
        let value = value.into();
        if self.hint == value {
            return false;
        }
        self.hint = value.clone();
        self.update_node_content(manager, |node| node.set_hint(value));
        true
    }

    pub fn set_heading_level(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.heading_level == value {
            return false;
        }
        self.heading_level = value;
        self.update_node_content(manager, |node| node.set_heading_level(value));
        true
    }

    pub fn set_trait_flags(&mut self, value: u32, manager: Option<&mut SemanticManager>) -> bool {
        if self.trait_flags == value {
            return false;
        }
        self.trait_flags = value;
        self.update_node_content(manager, |node| node.set_trait_flags(value));
        true
    }

    pub fn set_state_flags(
        &mut self,
        value: u32,
        mut manager: Option<&mut SemanticManager>,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        if self.state_flags == value {
            return false;
        }
        let hidden_changed = (self.state_flags ^ value) & SemanticState::HIDDEN.0 != 0;
        self.state_flags = value;
        self.update_node_content(manager.as_deref_mut(), |node| node.set_state_flags(value));
        if hidden_changed && let Some(manager) = manager {
            self.sync_tree_visibility(manager, artboard);
        }
        true
    }

    fn update_node_content<F>(&mut self, manager: Option<&mut SemanticManager>, update: F)
    where
        F: FnOnce(&mut crate::SemanticNode),
    {
        let Some(node) = &self.semantic_node else {
            return;
        };
        update(&mut node.borrow_mut());
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn apply_inferred_semantics_if_needed(
        &mut self,
        artboard: &ArtboardInstance,
        manager: Option<&mut SemanticManager>,
    ) {
        if self.role != 0 || !self.label.is_empty() {
            return;
        }
        let (Some(node), Some(parent_local)) = (&self.semantic_node, self.parent_local_id) else {
            return;
        };
        let mut inferred = ResolvedSemanticData::default();
        if !crate::semantic_inference_registry::resolve_inferred_semantics(
            artboard,
            parent_local,
            &mut inferred,
        ) {
            return;
        }
        if node.borrow().role() == inferred.role && node.borrow().label() == inferred.label {
            return;
        }
        {
            let mut node = node.borrow_mut();
            node.set_role(inferred.role);
            node.set_label(inferred.label);
        }
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::CONTENT);
        }
    }

    pub fn update_world_bounds(
        &mut self,
        artboard: &mut ArtboardInstance,
        manager: Option<&mut SemanticManager>,
    ) {
        self.update_world_bounds_with_root_transform(artboard, manager, Mat2D::IDENTITY);
    }

    pub(crate) fn update_world_bounds_with_root_transform(
        &mut self,
        artboard: &mut ArtboardInstance,
        manager: Option<&mut SemanticManager>,
        root_transform: Mat2D,
    ) {
        let (Some(node), Some(parent_local)) = (&self.semantic_node, self.parent_local_id) else {
            return;
        };
        let bounds = SemanticProvider::semantic_bounds_with_root_transform(
            artboard,
            parent_local,
            root_transform,
        );
        if bounds.is_empty_or_nan() && self.bounds_retry_pending {
            artboard.add_dirt(
                self.local_id,
                crate::components::ComponentDirt::WORLD_TRANSFORM,
                false,
            );
            return;
        }
        self.bounds_retry_pending = false;
        if node.borrow().bounds() == bounds {
            return;
        }
        node.borrow_mut().set_bounds(bounds);
        if let Some(manager) = manager {
            manager.mark_node_dirty(node.borrow().id(), SemanticDirt::BOUNDS);
        }
    }

    fn should_exclude_from_tree(&self, artboard: &ArtboardInstance) -> bool {
        if has_semantic_state(self.state_flags, SemanticState::HIDDEN) {
            return true;
        }
        let Some(parent_local) = self.parent_local_id else {
            return true;
        };
        if artboard.component(parent_local).is_none() {
            return true;
        }
        artboard.runtime_component_is_collapsed_for_draw(parent_local)
    }

    pub fn sync_tree_visibility(
        &mut self,
        manager: &mut SemanticManager,
        artboard: &mut ArtboardInstance,
    ) {
        let Some(node) = &self.semantic_node else {
            return;
        };
        let exclude = self.should_exclude_from_tree(artboard);
        if exclude == self.excluded_from_tree {
            return;
        }
        self.excluded_from_tree = exclude;
        if exclude {
            if node.borrow().manager_identity() == Some(manager.identity()) {
                manager.remove_child(node);
            }
            self.semantic_manager_identity = Some(manager.identity());
            return;
        }
        if node.borrow().manager_identity().is_none()
            && self.semantic_manager_identity == Some(manager.identity())
        {
            manager.add_child(self.tree_parent.as_ref(), node.clone());
            self.bounds_retry_pending = true;
            self.update_world_bounds(artboard, Some(manager));
            self.apply_inferred_semantics_if_needed(artboard, Some(manager));
        }
    }

    pub fn add_semantic_listener(&mut self, listener: Rc<dyn SemanticListener>) {
        self.semantic_listeners.push(listener);
    }

    pub fn remove_semantic_listener(&mut self, listener: &Rc<dyn SemanticListener>) {
        if let Some(index) = self
            .semantic_listeners
            .iter()
            .position(|candidate| Rc::ptr_eq(candidate, listener))
        {
            self.semantic_listeners.remove(index);
        }
    }

    pub fn fire(&self, action: SemanticActionType) {
        for listener in &self.semantic_listeners {
            match action {
                SemanticActionType::Tap => listener.on_semantic_tap(),
                SemanticActionType::Increase => listener.on_semantic_increase(),
                SemanticActionType::Decrease => listener.on_semantic_decrease(),
            }
        }
    }
}

// Pinned C++ correspondence (d788e8ec):
// include/rive/semantic/{semantic_node,semantic_role,semantic_state,
// semantic_trait,semantic_dirt}.hpp.

use std::cell::{Ref, RefCell, RefMut};
use std::ops::{BitAnd, BitOr, BitOrAssign};
use std::rc::Weak;

/// Bounds in the outermost artboard's coordinate space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SemanticBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl SemanticBounds {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    /// Mirrors `AABB::forExpansion()`, the pinned node's no-bounds sentinel.
    pub const fn for_expansion() -> Self {
        Self::new(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX)
    }

    pub fn is_empty_or_nan(self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.min_x > self.max_x
            || self.min_y > self.max_y
    }

    pub fn expand(&mut self, other: Self) {
        if other.is_empty_or_nan() {
            return;
        }
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SemanticRole {
    None = 0,
    Button = 1,
    Link = 2,
    Checkbox = 3,
    SwitchControl = 4,
    Slider = 5,
    TextField = 6,
    Text = 7,
    Image = 8,
    Group = 9,
    List = 10,
    ListItem = 11,
    Tab = 12,
    TabList = 13,
    Dialog = 14,
    AlertDialog = 15,
    RadioGroup = 16,
    RadioButton = 17,
}

impl SemanticRole {
    pub fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::Button,
            2 => Self::Link,
            3 => Self::Checkbox,
            4 => Self::SwitchControl,
            5 => Self::Slider,
            6 => Self::TextField,
            7 => Self::Text,
            8 => Self::Image,
            9 => Self::Group,
            10 => Self::List,
            11 => Self::ListItem,
            12 => Self::Tab,
            13 => Self::TabList,
            14 => Self::Dialog,
            15 => Self::AlertDialog,
            16 => Self::RadioGroup,
            17 => Self::RadioButton,
            _ => return None,
        })
    }

    pub fn is_interactive(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::Link
                | Self::Checkbox
                | Self::SwitchControl
                | Self::Slider
                | Self::Tab
                | Self::ListItem
                | Self::RadioButton
        )
    }
}

pub fn is_interactive_role(value: u32) -> bool {
    SemanticRole::from_raw(value).is_some_and(SemanticRole::is_interactive)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SemanticTrait(pub u32);

impl SemanticTrait {
    pub const NONE: Self = Self(0);
    pub const EXPANDABLE: Self = Self(1 << 0);
    pub const SELECTABLE: Self = Self(1 << 1);
    pub const CHECKABLE: Self = Self(1 << 2);
    pub const TOGGLEABLE: Self = Self(1 << 3);
    pub const REQUIRABLE: Self = Self(1 << 4);
    pub const ENABLABLE: Self = Self(1 << 5);
    pub const FOCUSABLE: Self = Self(1 << 6);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SemanticState(pub u32);

impl SemanticState {
    pub const NONE: Self = Self(0);
    pub const EXPANDED: Self = Self(1 << 0);
    pub const SELECTED: Self = Self(1 << 1);
    pub const CHECKED: Self = Self(1 << 2);
    pub const MIXED: Self = Self(1 << 3);
    pub const TOGGLED: Self = Self(1 << 4);
    pub const REQUIRED: Self = Self(1 << 5);
    pub const DISABLED: Self = Self(1 << 6);
    pub const FOCUSED: Self = Self(1 << 7);
    pub const HIDDEN: Self = Self(1 << 8);
    pub const LIVE_REGION: Self = Self(1 << 9);
    pub const READ_ONLY: Self = Self(1 << 10);
    pub const MODAL: Self = Self(1 << 11);
    pub const OBSCURED: Self = Self(1 << 12);
    pub const MULTILINE: Self = Self(1 << 13);
}

macro_rules! impl_flags {
    ($type:ty) => {
        impl BitOr for $type {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl BitOrAssign for $type {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl BitAnd for $type {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
    };
}

impl_flags!(SemanticTrait);
impl_flags!(SemanticState);

pub fn has_semantic_trait(flags: u32, value: SemanticTrait) -> bool {
    flags & value.0 != 0
}

pub fn has_semantic_state(flags: u32, value: SemanticState) -> bool {
    flags & value.0 != 0
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SemanticDirt(pub u8);

impl SemanticDirt {
    pub const NONE: Self = Self(0);
    pub const STRUCTURE: Self = Self(1 << 0);
    pub const CONTENT: Self = Self(1 << 1);
    pub const BOUNDS: Self = Self(1 << 2);
    pub const ALL: Self = Self(Self::STRUCTURE.0 | Self::CONTENT.0 | Self::BOUNDS.0);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl_flags!(SemanticDirt);

/// Shared retained identity for one semantic node.
#[derive(Clone)]
pub struct SemanticNodeHandle(Rc<RefCell<SemanticNode>>);

impl std::fmt::Debug for SemanticNodeHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SemanticNodeHandle")
            .field("node", &self.0.borrow())
            .finish()
    }
}

impl PartialEq for SemanticNodeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for SemanticNodeHandle {}

impl SemanticNodeHandle {
    pub fn new(id: u32) -> Self {
        Self(Rc::new(RefCell::new(SemanticNode::new(id))))
    }

    pub fn borrow(&self) -> Ref<'_, SemanticNode> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, SemanticNode> {
        self.0.borrow_mut()
    }

    pub(crate) fn downgrade(&self) -> Weak<RefCell<SemanticNode>> {
        Rc::downgrade(&self.0)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Retained semantic tree node. Structural mutation is manager-owned.
#[derive(Debug)]
pub struct SemanticNode {
    id: u32,
    parent: Option<Weak<RefCell<SemanticNode>>>,
    children: Vec<SemanticNodeHandle>,
    role: u32,
    state_flags: u32,
    label: String,
    value: String,
    hint: String,
    heading_level: u32,
    bounds: SemanticBounds,
    trait_flags: u32,
    core_owner_local_id: Option<usize>,
    semantic_data_local_id: Option<usize>,
    boundary_artboard_local_id: Option<usize>,
    is_boundary_node: bool,
    manager_identity: Option<u64>,
}

impl SemanticNode {
    fn new(id: u32) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            role: 0,
            state_flags: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            bounds: SemanticBounds::for_expansion(),
            trait_flags: 0,
            core_owner_local_id: None,
            semantic_data_local_id: None,
            boundary_artboard_local_id: None,
            is_boundary_node: false,
            manager_identity: None,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn parent_id(&self) -> Option<u32> {
        self.parent
            .as_ref()
            .and_then(Weak::upgrade)
            .map(|parent| parent.borrow().id)
    }

    pub fn children(&self) -> &[SemanticNodeHandle] {
        &self.children
    }

    pub fn role(&self) -> u32 {
        self.role
    }

    pub fn set_role(&mut self, value: u32) {
        self.role = value;
    }

    pub fn state_flags(&self) -> u32 {
        self.state_flags
    }

    pub fn set_state_flags(&mut self, value: u32) {
        self.state_flags = value;
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, value: impl Into<String>) {
        self.label = value.into();
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    pub fn hint(&self) -> &str {
        &self.hint
    }

    pub fn set_hint(&mut self, value: impl Into<String>) {
        self.hint = value.into();
    }

    pub fn heading_level(&self) -> u32 {
        self.heading_level
    }

    pub fn set_heading_level(&mut self, value: u32) {
        self.heading_level = value;
    }

    pub fn bounds(&self) -> SemanticBounds {
        self.bounds
    }

    pub fn set_bounds(&mut self, value: SemanticBounds) {
        self.bounds = value;
    }

    pub fn trait_flags(&self) -> u32 {
        self.trait_flags
    }

    pub fn set_trait_flags(&mut self, value: u32) {
        self.trait_flags = value;
    }

    pub fn core_owner_local_id(&self) -> Option<usize> {
        self.core_owner_local_id
    }

    pub fn set_core_owner_local_id(&mut self, value: Option<usize>) {
        self.core_owner_local_id = value;
    }

    pub fn semantic_data_local_id(&self) -> Option<usize> {
        self.semantic_data_local_id
    }

    pub fn set_semantic_data_local_id(&mut self, value: Option<usize>) {
        self.semantic_data_local_id = value;
    }

    pub fn is_boundary_node(&self) -> bool {
        self.is_boundary_node
    }

    pub fn set_boundary_node(&mut self, value: bool) {
        self.is_boundary_node = value;
    }

    pub fn boundary_artboard_local_id(&self) -> Option<usize> {
        self.boundary_artboard_local_id
    }

    pub fn set_boundary_artboard_local_id(&mut self, value: Option<usize>) {
        self.boundary_artboard_local_id = value;
    }

    pub fn manager_identity(&self) -> Option<u64> {
        self.manager_identity
    }

    pub(crate) fn set_id(&mut self, value: u32) {
        self.id = value;
    }

    pub(crate) fn set_parent(&mut self, value: Option<Weak<RefCell<SemanticNode>>>) {
        self.parent = value;
    }

    pub(crate) fn children_mut(&mut self) -> &mut Vec<SemanticNodeHandle> {
        &mut self.children
    }

    pub(crate) fn set_manager_identity(&mut self, value: Option<u64>) {
        self.manager_identity = value;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;

    fn fixture_data() -> (ArtboardInstance, RuntimeSemanticData) {
        let file = read_runtime_file(include_bytes!(
            "../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv"
        ))
        .expect("semantic fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("semantic fixture graph builds");
        let graph = graphs
            .artboards
            .iter()
            .find(|graph| graph.name.as_deref() == Some("Element"))
            .expect("Element artboard graph");
        let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
            .expect("Element artboard instantiates");
        let local_id = artboard
            .components()
            .iter()
            .find(|component| component.type_name == "SemanticData")
            .expect("Element SemanticData")
            .local_id;
        let data = RuntimeSemanticData::from_artboard(&artboard, local_id);
        (artboard, data)
    }

    #[derive(Debug, Default)]
    struct CountingListener {
        tap: Cell<usize>,
        increase: Cell<usize>,
        decrease: Cell<usize>,
    }

    impl SemanticListener for CountingListener {
        fn on_semantic_tap(&self) {
            self.tap.set(self.tap.get() + 1);
        }

        fn on_semantic_increase(&self) {
            self.increase.set(self.increase.get() + 1);
        }

        fn on_semantic_decrease(&self) {
            self.decrease.set(self.decrease.get() + 1);
        }
    }

    #[test]
    fn listeners_preserve_duplicates_remove_first_and_dispatch_exact_action() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        let listener = Rc::new(CountingListener::default());
        let erased: Rc<dyn SemanticListener> = listener.clone();
        data.add_semantic_listener(erased.clone());
        data.add_semantic_listener(erased.clone());
        data.fire(SemanticActionType::Tap);
        assert_eq!(listener.tap.get(), 2);
        assert_eq!(listener.increase.get(), 0);
        data.remove_semantic_listener(&erased);
        data.fire(SemanticActionType::Increase);
        assert_eq!(listener.increase.get(), 1);
        data.fire(SemanticActionType::Decrease);
        assert_eq!(listener.decrease.get(), 1);
    }

    #[test]
    fn focused_state_preserves_other_bits_and_is_noop_before_node_creation() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        data.set_focused_state(true, None);
        assert!(!data.has_semantic_node());
        let node = SemanticNodeHandle::new(1);
        node.borrow_mut().set_state_flags(SemanticState::SELECTED.0);
        data.semantic_node = Some(node.clone());
        data.set_focused_state(true, None);
        assert_eq!(
            node.borrow().state_flags(),
            SemanticState::SELECTED.0 | SemanticState::FOCUSED.0
        );
        data.set_focused_state(false, None);
        assert_eq!(node.borrow().state_flags(), SemanticState::SELECTED.0);
    }

    #[test]
    fn generated_style_setters_early_out_and_preserve_node_identity() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        let node = SemanticNodeHandle::new(1);
        data.semantic_node = Some(node.clone());
        assert!(data.set_label("hello", None));
        assert!(!data.set_label("hello", None));
        assert!(data.node_handle().is_some_and(|same| same.ptr_eq(&node)));
        assert_eq!(node.borrow().label(), "hello");
    }

    #[test]
    fn semantic_node_is_lazy_stable_and_retains_its_owner_back_reference() {
        let (mut artboard, mut data) = fixture_data();
        assert!(!data.has_semantic_node());
        let first = data.semantic_node(&mut artboard);
        assert!(data.has_semantic_node());
        let second = data.semantic_node(&mut artboard);
        assert!(first.ptr_eq(&second));
        assert_eq!(first.borrow().semantic_data_local_id(), Some(data.local_id));
    }

    #[test]
    fn semantic_node_snapshots_every_authored_property() {
        let (mut artboard, mut data) = fixture_data();
        data.role = SemanticRole::Button as u32;
        data.label = "Submit".into();
        data.value = "$".into();
        data.hint = "Tap to send".into();
        data.heading_level = 2;
        data.trait_flags = SemanticTrait::ENABLABLE.0;
        data.state_flags = SemanticState::SELECTED.0;
        let node = data.semantic_node(&mut artboard);
        let node = node.borrow();
        assert_eq!(node.role(), SemanticRole::Button as u32);
        assert_eq!(node.label(), "Submit");
        assert_eq!(node.value(), "$");
        assert_eq!(node.hint(), "Tap to send");
        assert_eq!(node.heading_level(), 2);
        assert!(has_semantic_trait(
            node.trait_flags(),
            SemanticTrait::ENABLABLE
        ));
        assert!(has_semantic_state(
            node.state_flags(),
            SemanticState::SELECTED
        ));
    }

    #[test]
    fn setters_after_creation_update_the_same_semantic_node() {
        let (mut artboard, mut data) = fixture_data();
        let node = data.semantic_node(&mut artboard);
        assert!(data.set_role(SemanticRole::Link as u32, None));
        assert!(data.set_label("Learn more", None));
        assert!(data.set_value("value", None));
        assert!(data.set_hint("External link", None));
        assert!(data.set_heading_level(2, None));
        assert!(data.set_trait_flags(SemanticTrait::EXPANDABLE.0, None));
        assert!(data.set_state_flags(SemanticState::SELECTED.0, None, &mut artboard));
        let same = data.semantic_node(&mut artboard);
        assert!(same.ptr_eq(&node));
        let node = node.borrow();
        assert_eq!(node.role(), SemanticRole::Link as u32);
        assert_eq!(node.label(), "Learn more");
        assert_eq!(node.value(), "value");
        assert_eq!(node.hint(), "External link");
        assert_eq!(node.heading_level(), 2);
        assert_eq!(node.trait_flags(), SemanticTrait::EXPANDABLE.0);
        assert_eq!(node.state_flags(), SemanticState::SELECTED.0);
    }

    #[test]
    fn retained_data_removal_drops_manager_lookup_and_emits_removed_id() {
        let (mut artboard, mut data) = fixture_data();
        let mut manager = SemanticManager::new();
        let id = data.attach(&mut manager, None, &mut artboard);
        assert!(manager.node_by_id(id).is_some());
        manager.drain_diff().expect("initial semantic diff");
        data.detach(&mut manager);
        assert!(manager.node_by_id(id).is_none());
        assert_eq!(
            manager.drain_diff().expect("removal semantic diff").removed,
            [id]
        );
    }

    #[test]
    fn retained_tree_reconciles_collapsed_visibility_without_a_state_flag_change() {
        let (mut artboard, mut data) = fixture_data();
        let parent_local = data.parent_local_id.expect("SemanticData parent");
        let mut manager = SemanticManager::new();
        data.prepare_for_tree(&mut artboard);
        data.reconcile_tree_membership(&mut manager, None, &mut artboard, Mat2D::IDENTITY);
        let id = data.semantic_id();
        manager.drain_diff().expect("initial semantic diff");

        assert!(artboard.collapse_component(parent_local, true));
        data.reconcile_tree_membership(&mut manager, None, &mut artboard, Mat2D::IDENTITY);
        assert_eq!(
            manager
                .drain_diff()
                .expect("collapsed semantic diff")
                .removed,
            [id]
        );

        assert!(artboard.collapse_component(parent_local, false));
        data.reconcile_tree_membership(&mut manager, None, &mut artboard, Mat2D::IDENTITY);
        assert_eq!(
            manager
                .drain_diff()
                .expect("uncollapsed semantic diff")
                .added
                .iter()
                .map(|node| node.id)
                .collect::<Vec<_>>(),
            [id]
        );
    }

    #[test]
    fn repeated_focused_state_is_an_incremental_no_op() {
        let (mut artboard, mut data) = fixture_data();
        let mut manager = SemanticManager::new();
        data.prepare_for_tree(&mut artboard);
        data.reconcile_tree_membership(&mut manager, None, &mut artboard, Mat2D::IDENTITY);
        manager.drain_diff().expect("initial semantic diff");

        data.set_focused_state(false, Some(&mut manager));
        assert!(
            manager
                .drain_diff()
                .expect("repeated focus semantic diff")
                .is_empty()
        );
    }

    #[test]
    fn listener_removal_unregistered_removal_and_empty_dispatch_are_noops() {
        let mut data = RuntimeSemanticData::new(1, Some(0));
        let registered = Rc::new(CountingListener::default());
        let erased: Rc<dyn SemanticListener> = registered.clone();
        let ghost: Rc<dyn SemanticListener> = Rc::new(CountingListener::default());
        data.add_semantic_listener(erased.clone());
        data.remove_semantic_listener(&ghost);
        data.fire(SemanticActionType::Tap);
        assert_eq!(registered.tap.get(), 1);
        data.remove_semantic_listener(&erased);
        data.fire(SemanticActionType::Tap);
        assert_eq!(registered.tap.get(), 1);
        data.fire(SemanticActionType::Increase);
        data.fire(SemanticActionType::Decrease);
    }
}
