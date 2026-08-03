// Pinned C++ correspondence (d788e8ec):
// include/rive/semantic/{semantic_node,semantic_role,semantic_state,
// semantic_trait,semantic_dirt}.hpp.

use std::cell::{Ref, RefCell, RefMut};
use std::ops::{BitAnd, BitOr, BitOrAssign};
use std::rc::{Rc, Weak};

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
