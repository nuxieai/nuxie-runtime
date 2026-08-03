use crate::animation::RuntimeInterpolator;
use crate::artboard::{RuntimeComponentListItemInstance, RuntimeComponentListLogicalItem};
use crate::draw::RuntimePathMeasure;
use crate::objects::{InstanceObjectArena, InstanceSlot};
use crate::properties::{
    artboard_index_for_graph, cached_property_key_for_name, property_key_for_name,
};
use crate::view_model::RuntimeOwnedViewModelListHandle;
use nuxie_binary::RuntimeFile;
use nuxie_graph::{ArtboardGraph, ComponentNode};
use nuxie_render_api::RawPath;
use nuxie_schema::definition_by_name;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::sync::OnceLock;

mod bones {
    pub(crate) mod bone {
        include!("bones/bone.rs");
    }

    pub(crate) mod root_bone {
        include!("bones/root_bone.rs");
    }

    pub(crate) mod skinnable {
        include!("bones/skinnable.rs");
    }

    pub(crate) mod tendon {
        include!("bones/tendon.rs");
    }
}

pub use crate::math::mat2d::Mat2D;
use bones::bone::RuntimeBoneState;
use bones::root_bone::{
    x_property_key as root_bone_x_property_key, y_property_key as root_bone_y_property_key,
};
pub(crate) use bones::skinnable::RuntimeSkinnableKind;
use bones::skinnable::RuntimeSkinnableState;
use bones::tendon::RuntimeTendonState;

/// Occurrence-local equivalent of a retained C++ `Component*`.
///
/// The handle addresses the one object slot that owns both the generated
/// fields and the embedded [`RuntimeComponent`] state. It is never a serialized
/// ID and must not be shared between Artboard occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ComponentHandle(usize);

impl ComponentHandle {
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

/// Occurrence-local equivalent of one retained C++ `DataBind*` in
/// `Component::m_collapsables`.
///
/// Authored DataBinds already have one dense container index. This newtype
/// prevents that index from being confused with a Component or object slot
/// while preserving C++ insertion order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct DataBindHandle(usize);

impl DataBindHandle {
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GraphOrder(usize);

impl GraphOrder {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

/// Runtime component dirt and transform state.
///
/// Ported from C++ `src/component.cpp` and `src/transform_component.cpp` for
/// the update-order semantics that M2 exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentDirt(pub u16);

impl ComponentDirt {
    pub const NONE: Self = Self(0);
    pub const COLLAPSED: Self = Self(1 << 0);
    pub const DEPENDENTS: Self = Self(1 << 1);
    pub const COMPONENTS: Self = Self(1 << 2);
    pub const DRAW_ORDER: Self = Self(1 << 3);
    pub const PATH: Self = Self(1 << 4);
    pub const TEXT_SHAPE: Self = Self(1 << 4);
    pub const SKIN: Self = Self(1 << 4);
    pub const VERTICES: Self = Self(1 << 5);
    pub const TEXT_COVERAGE: Self = Self(1 << 5);
    pub const TRANSFORM: Self = Self(1 << 6);
    pub const WORLD_TRANSFORM: Self = Self(1 << 7);
    pub const RENDER_OPACITY: Self = Self(1 << 8);
    pub const PAINT: Self = Self(1 << 9);
    pub const STOPS: Self = Self(1 << 10);
    pub const LAYOUT_STYLE: Self = Self(1 << 11);
    pub const BINDINGS: Self = Self(1 << 12);
    pub const N_SLICER: Self = Self(1 << 13);
    pub const BINDINGS_TARGET: Self = Self(1 << 13);
    pub const SCRIPT_UPDATE: Self = Self(1 << 14);
    pub const CLIPPING: Self = Self(1 << 15);
    pub const FILTHY: Self = Self(0xFFFE);

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

#[cfg(test)]
mod component_dirt_tests {
    use super::ComponentDirt;

    #[test]
    fn bindings_target_matches_cpp_dirt_bit() {
        assert_eq!(ComponentDirt::BINDINGS_TARGET.0, 1 << 13);
        assert_eq!(ComponentDirt::BINDINGS_TARGET.0, ComponentDirt::N_SLICER.0);
    }
}

impl Default for ComponentDirt {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for ComponentDirt {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ComponentDirt {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ComponentDirt {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ComponentDirt {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for ComponentDirt {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateComponentsReport {
    pub did_update: bool,
    pub steps: usize,
    pub updated_locals: Vec<usize>,
    pub max_steps_reached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformProperty {
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
}

impl TransformProperty {
    pub(crate) fn property_name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Rotation => "rotation",
            Self::ScaleX => "scaleX",
            Self::ScaleY => "scaleY",
            Self::Opacity => "opacity",
        }
    }

    pub(crate) fn default_value(self) -> f32 {
        match self {
            Self::X | Self::Y | Self::Rotation => 0.0,
            Self::ScaleX | Self::ScaleY | Self::Opacity => 1.0,
        }
    }

    pub(crate) fn property_key_for_type(self, type_name: &str) -> Option<u16> {
        match self {
            Self::X if type_name == "RootBone" => root_bone_x_property_key(),
            Self::Y if type_name == "RootBone" => root_bone_y_property_key(),
            Self::X => node_x_property_key(),
            Self::Y => node_y_property_key(),
            Self::Rotation => transform_component_rotation_property_key(),
            Self::ScaleX => transform_component_scale_x_property_key(),
            Self::ScaleY => transform_component_scale_y_property_key(),
            Self::Opacity if type_name == "Artboard" => artboard_opacity_property_key(),
            Self::Opacity => transform_component_opacity_property_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransformPropertyKeys {
    type_name: &'static str,
    x: Option<u16>,
    y: Option<u16>,
    rotation: Option<u16>,
    scale_x: Option<u16>,
    scale_y: Option<u16>,
    opacity: Option<u16>,
}

impl TransformPropertyKeys {
    pub(crate) fn for_type(type_name: &'static str) -> Self {
        Self {
            type_name,
            x: property_key_for_name(type_name, TransformProperty::X.property_name()),
            y: property_key_for_name(type_name, TransformProperty::Y.property_name()),
            rotation: property_key_for_name(type_name, TransformProperty::Rotation.property_name()),
            scale_x: property_key_for_name(type_name, TransformProperty::ScaleX.property_name()),
            scale_y: property_key_for_name(type_name, TransformProperty::ScaleY.property_name()),
            opacity: property_key_for_name(type_name, TransformProperty::Opacity.property_name()),
        }
    }

    fn is_for_type(self, type_name: &str) -> bool {
        self.type_name == type_name
    }

    pub(crate) fn key(self, property: TransformProperty) -> Option<u16> {
        match property {
            TransformProperty::X => self.x,
            TransformProperty::Y => self.y,
            TransformProperty::Rotation => self.rotation,
            TransformProperty::ScaleX => self.scale_x,
            TransformProperty::ScaleY => self.scale_y,
            TransformProperty::Opacity => self.opacity,
        }
    }
}

fn node_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "x")
}

fn node_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "y")
}

fn transform_component_rotation_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "rotation")
}

fn transform_component_scale_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "scaleX")
}

fn transform_component_scale_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "scaleY")
}

fn transform_component_opacity_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "opacity")
}

fn artboard_opacity_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Artboard", "opacity")
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeComponentCapabilities {
    pub world_transform: bool,
    pub transform: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AuthoredTransform {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformRuntimeState {
    pub local_transform: Mat2D,
    pub world_transform: Mat2D,
    pub render_opacity: f32,
}

/// Runtime-only state owned by the concrete C++ `Node` subobject.
///
/// This is deliberately distinct from [`TransformRuntimeState::local_transform`]:
/// that matrix is the authored transform, while `Node::m_LocalTransform` is a
/// lazy query cache derived from the settled world transform after constraints.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeNodeState {
    computed_local_transform: Cell<Mat2D>,
    computed_local_needs_recompute: Cell<bool>,
}

impl RuntimeNodeState {
    fn new() -> Self {
        Self {
            computed_local_transform: Cell::new(Mat2D::IDENTITY),
            computed_local_needs_recompute: Cell::new(false),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self::new()
    }

    pub(crate) fn mark_computed_local_dirty(&self) {
        self.computed_local_needs_recompute.set(true);
    }

    pub(crate) fn computed_local_transform(
        &self,
        parent_world: Option<Mat2D>,
        world: Mat2D,
    ) -> Mat2D {
        if self.computed_local_needs_recompute.replace(false) {
            // Pinned `Node::computeLocalTransform` falls back to identity both
            // when there is no parent transform and when inversion fails
            // (`src/node.cpp:26-45`).
            let local = parent_world
                .filter(|parent| parent.determinant() != 0.0)
                .map(|parent| parent.invert_or_identity().multiply(world))
                .unwrap_or(Mat2D::IDENTITY);
            self.computed_local_transform.set(local);
        }
        self.computed_local_transform.get()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSkinState {
    pub(crate) world_transform: Mat2D,
    pub(crate) tendons: Vec<ComponentHandle>,
    pub(crate) skinnable: Option<ComponentHandle>,
    pub(crate) bone_transforms: Vec<Mat2D>,
    #[cfg(test)]
    pub(crate) buffer_rebuilds: usize,
}

impl Default for RuntimeSkinState {
    fn default() -> Self {
        Self {
            world_transform: Mat2D::IDENTITY,
            tendons: Vec::new(),
            skinnable: None,
            bone_transforms: Vec::new(),
            #[cfg(test)]
            buffer_rebuilds: 0,
        }
    }
}

/// Runtime-only fields owned by C++ `Weight`/`CubicWeight`.
///
/// Packed values and indices remain in the occurrence's generated storage;
/// only the settled deformation outputs live on the concrete owner
/// (`include/rive/bones/weight.hpp:12-15`,
/// `include/rive/bones/cubic_weight.hpp:9-15`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct RuntimeWeightState {
    /// Concrete C++ subtype identity used to dispatch the extra
    /// `CubicWeight::{in,out}Translation` members.
    pub(crate) is_cubic: bool,
    pub(crate) translation: (f32, f32),
    pub(crate) in_translation: (f32, f32),
    pub(crate) out_translation: (f32, f32),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeVertexState {
    pub(crate) weight: Option<ComponentHandle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeConstraintKind {
    Distance,
    FollowPath,
    ListFollowPath,
    Ik,
    Rotation,
    Scale,
    Scroll,
    ScrollBar,
    Transform,
    Translation,
    Other,
}

impl RuntimeConstraintKind {
    fn for_type(type_name: &'static str) -> Self {
        match type_name {
            "DistanceConstraint" => Self::Distance,
            "FollowPathConstraint" => Self::FollowPath,
            "ListFollowPathConstraint" => Self::ListFollowPath,
            "IKConstraint" => Self::Ik,
            "RotationConstraint" => Self::Rotation,
            "ScaleConstraint" => Self::Scale,
            "ScrollConstraint" => Self::Scroll,
            "ScrollBarConstraint" => Self::ScrollBar,
            "TransformConstraint" => Self::Transform,
            "TranslationConstraint" => Self::Translation,
            _ => Self::Other,
        }
    }

    pub(crate) fn uses_targeted_base_dependencies(self) -> bool {
        !matches!(
            self,
            Self::FollowPath | Self::ListFollowPath | Self::Other | Self::Scroll | Self::ScrollBar
        )
    }
}

/// Runtime-only members owned by C++ `FollowPathConstraint`.
///
/// `raw_path` is the exact retained `RawPath` owner. Its allocations survive
/// `rewind`/rebuild just as C++ `FollowPathConstraint::m_rawPath` does;
/// `path_measure` is rebuilt from it only by the concrete constraint's
/// dependency-ordered `update` (`follow_path_constraint.cpp:122-147`).
#[derive(Debug, Clone)]
pub(crate) struct RuntimeFollowPathState {
    pub(crate) raw_path: RawPath,
    pub(crate) path_measure: RuntimePathMeasure,
    #[cfg(test)]
    pub(crate) measure_rebuilds: usize,
}

impl RuntimeFollowPathState {
    fn new() -> Self {
        Self {
            raw_path: RawPath::new(),
            path_measure: RuntimePathMeasure::from_raw_path(&RawPath::new()),
            #[cfg(test)]
            measure_rebuilds: 0,
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self::new()
    }
}

/// Runtime-only fields owned by C++ `Path`.
///
/// `shape` is the occurrence-local `m_Shape` pointer rebuilt by
/// `Path::onAddedClean`; its embedded composer is reached through that Shape.
/// `deferred_path_dirt` and `flags` are the state read by
/// `Path::{onDirty,canDeferPathUpdate,update}`
/// (`src/shapes/path.cpp:76-125,300-372`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimePathState {
    pub(crate) shape: Option<ComponentHandle>,
    pub(crate) flags: Cell<u8>,
    pub(crate) deferred_path_dirt: Cell<bool>,
}

impl RuntimePathState {
    pub(crate) const CLIPPING: u8 = 1 << 3;
    pub(crate) const FOLLOW_PATH: u8 = 1 << 4;

    fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }

    pub(crate) fn add_flags(&self, flags: u8) -> bool {
        let previous = self.flags.get();
        self.flags.set(previous | flags);
        previous & flags != flags
    }

    pub(crate) fn is_flagged(&self, flags: u8) -> bool {
        self.flags.get() & flags != 0
    }
}

/// Runtime-only fields owned by C++ `Shape`.
///
/// Paths register in authored order during `Path::onAddedClean`; flags are
/// accumulated by clipping/follow-path/hit-test owners on this exact
/// occurrence (`src/shapes/shape.cpp:20-51`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeShapeState {
    pub(crate) paths: Vec<ComponentHandle>,
    pub(crate) flags: Cell<u8>,
}

impl RuntimeShapeState {
    pub(crate) const CLIPPING: u8 = RuntimePathState::CLIPPING;
    pub(crate) const FOLLOW_PATH: u8 = RuntimePathState::FOLLOW_PATH;
    pub(crate) const NEVER_DEFER_UPDATE: u8 = 1 << 5;

    fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }

    pub(crate) fn add_flags(&self, flags: u8) -> bool {
        let previous = self.flags.get();
        self.flags.set(previous | flags);
        previous & flags != flags
    }

    #[cfg(test)]
    pub(crate) fn is_flagged(&self, flags: u8) -> bool {
        self.flags.get() & flags != 0
    }
}

impl Default for RuntimeFollowPathState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeComponentListOrderCache {
    pub(crate) indices: Vec<usize>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeConstrainableListState {
    pub(crate) constraints: Vec<ComponentHandle>,
    /// C++ `LayoutNodeProvider::m_layoutConstraints`, retained in registration
    /// order. ArtboardComponentList queries the first ScrollConstraint for
    /// virtualization but applies every layout constraint before list and
    /// ordinary constraints (`artboard_component_list.cpp:1333-1358,1746-1755`).
    pub(crate) layout_constraints: Vec<ComponentHandle>,
    /// C++ `ArtboardComponentList::m_Order` cache, retained by the concrete
    /// list occurrence rather than by its containing Artboard.
    pub(crate) order_cache: RefCell<RuntimeComponentListOrderCache>,
    /// C++ `ArtboardComponentList::m_artboardTransforms`, rebuilt during the
    /// concrete owner's world-transform update and consumed read-only by draw.
    pub(crate) item_transforms: Vec<Mat2D>,
    /// C++ `ArtboardComponentList::m_list`, retained by this occurrence.
    pub(crate) source: Option<RuntimeOwnedViewModelListHandle>,
    /// C++ `m_listItems` plus `m_artboardSizes`, retained in logical order
    /// even when virtualization unmounts most row Artboards.
    pub(crate) logical_items: Vec<RuntimeComponentListLogicalItem>,
    /// C++ `m_artboardInstancesMap`/`m_artboardInstances`, owned by the list
    /// occurrence. Each entry owns its row Artboard before its state machines
    /// are destroyed, matching the concrete C++ teardown boundary.
    pub(crate) items: Vec<RuntimeComponentListItemInstance>,
    /// Provider-local visible indices published directly by the retained
    /// `ScrollVirtualizer`, matching `ArtboardComponentList::setVisibleIndices`.
    pub(crate) visible_start: i32,
    pub(crate) visible_end: i32,
    /// C++ `ArtboardComponentList::m_layoutSize`, computed from the complete
    /// logical size vector rather than only mounted rows.
    pub(crate) layout_size: (f32, f32),
}

impl RuntimeConstrainableListState {
    fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeScrollAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeScrollSpace {
    Percent,
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeScrollAxisIntent {
    pub(crate) space: RuntimeScrollSpace,
    pub(crate) value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeElasticScrollPhysicsHelper {
    pub(crate) friction: f32,
    pub(crate) speed_multiplier: f32,
    pub(crate) elastic_factor: f32,
    pub(crate) target: f32,
    pub(crate) current: f32,
    pub(crate) speed: f32,
    pub(crate) snap_target: f32,
    pub(crate) run_range_min: f32,
    pub(crate) run_range_max: f32,
    pub(crate) is_running: bool,
}

impl RuntimeElasticScrollPhysicsHelper {
    pub(crate) fn new(friction: f32, speed_multiplier: f32, elastic_factor: f32) -> Self {
        Self {
            friction,
            speed_multiplier,
            elastic_factor,
            target: 0.0,
            current: 0.0,
            speed: 0.0,
            snap_target: f32::NAN,
            run_range_min: 0.0,
            run_range_max: 0.0,
            is_running: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeScrollPhysicsKind {
    Clamped {
        value: (f32, f32),
    },
    Elastic {
        x: Option<RuntimeElasticScrollPhysicsHelper>,
        y: Option<RuntimeElasticScrollPhysicsHelper>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeScrollPhysicsState {
    pub(crate) kind: RuntimeScrollPhysicsKind,
    pub(crate) last_time_micros: i64,
    pub(crate) is_running: bool,
    pub(crate) speed: (f32, f32),
    pub(crate) acceleration: (f32, f32),
    pub(crate) direction: u64,
    pub(crate) friction: f32,
    pub(crate) speed_multiplier: f32,
    pub(crate) elastic_factor: f32,
}

impl RuntimeScrollPhysicsState {
    pub(crate) fn clamped() -> Self {
        Self {
            kind: RuntimeScrollPhysicsKind::Clamped { value: (0.0, 0.0) },
            last_time_micros: 0,
            is_running: false,
            speed: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            direction: 1,
            friction: 8.0,
            speed_multiplier: 1.0,
            elastic_factor: 0.66,
        }
    }

    pub(crate) fn elastic(friction: f32, speed_multiplier: f32, elastic_factor: f32) -> Self {
        Self {
            kind: RuntimeScrollPhysicsKind::Elastic { x: None, y: None },
            friction,
            speed_multiplier,
            elastic_factor,
            ..Self::clamped()
        }
    }

    /// C++ generated clone copies only authored base fields; runtime motion
    /// state and concrete helpers restart cold.
    pub(crate) fn clone_for_constraint(&self) -> Self {
        match self.kind {
            RuntimeScrollPhysicsKind::Clamped { .. } => Self::clamped(),
            RuntimeScrollPhysicsKind::Elastic { .. } => {
                Self::elastic(self.friction, self.speed_multiplier, self.elastic_factor)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeScrollVirtualizerState {
    pub(crate) visible_start: i32,
    pub(crate) visible_end: i32,
    pub(crate) offset: f32,
    pub(crate) infinite: bool,
    pub(crate) viewport_size: f32,
    pub(crate) direction: RuntimeScrollAxis,
}

impl Default for RuntimeScrollVirtualizerState {
    fn default() -> Self {
        Self {
            visible_start: 0,
            visible_end: 0,
            offset: 0.0,
            infinite: false,
            viewport_size: 0.0,
            direction: RuntimeScrollAxis::X,
        }
    }
}

/// Runtime-only members owned by one C++ `ScrollConstraint` occurrence.
///
/// The parent/content relationship and layout children are retained handles,
/// not Artboard-local IDs. Clone construction copies generated properties but
/// rebuilds these relations and clears the transient scroll/layout state
/// (`scroll_constraint.cpp:14-23,203-237,364-373`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeScrollConstraintState {
    pub(crate) content: Option<ComponentHandle>,
    pub(crate) layout_children: Vec<ComponentHandle>,
    pub(crate) physics: Option<RuntimeScrollPhysicsState>,
    pub(crate) virtualizer: Option<RuntimeScrollVirtualizerState>,
    pub(crate) components_a: TransformComponents,
    pub(crate) components_b: TransformComponents,
    pub(crate) scroll_transform: Mat2D,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) last_frame_offset_x: f32,
    pub(crate) last_frame_offset_y: f32,
    pub(crate) child_constraint_applied_count: usize,
    pub(crate) is_dragging: bool,
    pub(crate) is_scroll_bar_dragging: bool,
    pub(crate) has_list_children: bool,
    pub(crate) intent_x: Option<RuntimeScrollAxisIntent>,
    pub(crate) intent_y: Option<RuntimeScrollAxisIntent>,
    pub(crate) layout_initialized: bool,
}

impl RuntimeScrollConstraintState {
    fn clone_for_occurrence(&self) -> Self {
        Self {
            physics: self
                .physics
                .as_ref()
                .map(RuntimeScrollPhysicsState::clone_for_constraint),
            ..Self::default()
        }
    }

    pub(crate) fn intent(&self, axis: RuntimeScrollAxis) -> Option<RuntimeScrollAxisIntent> {
        match axis {
            RuntimeScrollAxis::X => self.intent_x,
            RuntimeScrollAxis::Y => self.intent_y,
        }
    }

    pub(crate) fn set_intent(
        &mut self,
        axis: RuntimeScrollAxis,
        intent: Option<RuntimeScrollAxisIntent>,
    ) {
        match axis {
            RuntimeScrollAxis::X => self.intent_x = intent,
            RuntimeScrollAxis::Y => self.intent_y = intent,
        }
    }

    pub(crate) fn clear_intent(&mut self, axis: RuntimeScrollAxis) -> bool {
        let had_intent = self.intent(axis).is_some();
        self.set_intent(axis, None);
        had_intent
    }
}

/// Runtime-only members owned by one C++ `ScrollBarConstraint` occurrence.
///
/// The generated `scrollConstraintId` remains in the occurrence's generated
/// storage. The resolved target and transform scratch are construction-time
/// owner state (`scroll_bar_constraint.cpp:50-140`).
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeScrollBarConstraintState {
    pub(crate) scroll_constraint: Option<ComponentHandle>,
    pub(crate) components_a: TransformComponents,
    pub(crate) components_b: TransformComponents,
}

impl RuntimeScrollBarConstraintState {
    fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

/// One retained C++ `IKConstraint::BoneChainLink`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeIkChainLink {
    pub(crate) index: usize,
    pub(crate) bone: ComponentHandle,
    pub(crate) angle: f32,
    pub(crate) transform_components: TransformComponents,
    pub(crate) parent_world_inverse: Mat2D,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeIkState {
    pub(crate) chain: Vec<RuntimeIkChainLink>,
    #[cfg(test)]
    pub(crate) chain_builds: usize,
}

impl RuntimeIkState {
    fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeConstraintScratch {
    None,
    Rotation {
        components_a: TransformComponents,
        components_b: TransformComponents,
    },
    Scale {
        components_a: TransformComponents,
        components_b: TransformComponents,
    },
    Transform {
        components_a: TransformComponents,
        components_b: TransformComponents,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeConstraintBoundsKind {
    Default,
    Layout,
    Text,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeLayoutComponentState {
    layout_node_dirty: Cell<bool>,
    layout_node_revision: Cell<u64>,
    layout: Cell<RuntimeLayoutRect>,
    animation_a: Cell<RuntimeLayoutAnimationData>,
    animation_b: Cell<RuntimeLayoutAnimationData>,
    is_smoothing_animation: Cell<bool>,
    initialized: Cell<bool>,
    just_added_to_host: Cell<bool>,
    animation_style: Cell<u8>,
    interpolation: Cell<u8>,
    interpolation_time: Cell<f32>,
    interpolator: Cell<Option<RuntimeInterpolator>>,
    inherited_interpolation: Cell<u8>,
    inherited_interpolation_time: Cell<f32>,
    inherited_interpolator: Cell<Option<RuntimeInterpolator>>,
    forced_width: Cell<Option<f32>>,
    forced_height: Cell<Option<f32>>,
    position_left_changed: Cell<bool>,
    position_top_changed: Cell<bool>,
    force_update_layout_bounds: Cell<bool>,
    pub(crate) clip_property_key: Option<u16>,
    pub(crate) style_id_property_key: Option<u16>,
    pub(crate) style: Option<ComponentHandle>,
    pub(crate) layout_constraints: Vec<ComponentHandle>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct RuntimeLayoutRect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl RuntimeLayoutRect {
    fn lerp(from: Self, to: Self, factor: f32) -> Self {
        let inverse = 1.0 - factor;
        Self {
            left: to.left * factor + from.left * inverse,
            top: to.top * factor + from.top * inverse,
            width: to.width * factor + from.width * inverse,
            height: to.height * factor + from.height * inverse,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RuntimeLayoutAnimationData {
    elapsed_seconds: f32,
    from: RuntimeLayoutRect,
    to: RuntimeLayoutRect,
}

impl RuntimeLayoutAnimationData {
    fn interpolate(self, factor: f32) -> RuntimeLayoutRect {
        RuntimeLayoutRect::lerp(self.from, self.to, factor)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RuntimeLayoutAdvance {
    pub(crate) keep_going: bool,
    pub(crate) layout_changed: bool,
    pub(crate) size_changed: bool,
}

/// Runtime lifecycle bits owned by one scripted Component occurrence.
///
/// The executable VM table remains in the scripting backend, but C++ keeps
/// `m_isAdvanceActive` and the pending ScriptUpdate dirt on the concrete
/// ScriptedDrawable/ScriptedLayout/ScriptedPathEffect owner. These bits must
/// therefore clone cold with the occurrence rather than living in an
/// Artboard-wide global-id set.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RuntimeScriptedComponentState {
    pub(crate) advance_active: bool,
    pub(crate) update_pending: bool,
}

/// Runtime-only fields owned by one C++ `TextInput` occurrence.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeTextInputState {
    pub(crate) world_bounds: Cell<Option<(f32, f32, f32, f32)>>,
    pub(crate) text_style: Cell<Option<ComponentHandle>>,
    pub(crate) layout_width: Cell<f32>,
    pub(crate) scroll_constraint: Option<ComponentHandle>,
    pub(crate) is_focused: bool,
    pub(crate) is_dragging: bool,
    pub(crate) last_drag_world_position: (f32, f32),
    pub(crate) scroll_x: f32,
    pub(crate) scroll_y: f32,
    pub(crate) source_text: RefCell<Option<String>>,
    pub(crate) raw: RefCell<crate::text::raw_text_input::RawTextInput>,
}

/// Retained fields owned by one pinned C++ `Text` occurrence.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeTextState {
    bounds: Cell<Option<(f32, f32, f32, f32)>>,
    layout_scale_types: Cell<Option<(u64, u64)>>,
}

impl RuntimeTextState {
    fn new() -> Self {
        Self {
            bounds: Cell::new(None),
            layout_scale_types: Cell::new(None),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self::new()
    }

    pub(crate) fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        self.bounds.get()
    }

    pub(crate) fn retain_bounds(&self, bounds: (f32, f32, f32, f32)) {
        self.bounds.set(Some(bounds));
    }

    pub(crate) fn invalidate_bounds(&self) {
        self.bounds.set(None);
    }

    pub(crate) fn retain_layout_scale_types(&self, width: u64, height: u64) {
        self.layout_scale_types.set(Some((width, height)));
    }

    pub(crate) fn effective_sizing(&self, authored: u64) -> u64 {
        match self.layout_scale_types.get() {
            Some((width, height)) if width != 2 && height != 2 => 2,
            _ => authored,
        }
    }
}

impl Default for RuntimeTextInputState {
    fn default() -> Self {
        Self {
            world_bounds: Cell::new(None),
            text_style: Cell::new(None),
            layout_width: Cell::new(f32::NAN),
            scroll_constraint: None,
            is_focused: false,
            is_dragging: false,
            last_drag_world_position: (f32::NAN, f32::NAN),
            scroll_x: 0.0,
            scroll_y: 0.0,
            source_text: RefCell::new(None),
            raw: RefCell::new(crate::text::raw_text_input::RawTextInput::default()),
        }
    }
}

/// Component-facing state of one C++ Drawable or embedded DrawableProxy.
///
/// Renderer resources and linked-list nodes remain in the accepted RF owner;
/// clipping membership belongs to this concrete occurrence and is rebuilt
/// from clone-owned handles during construction.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeDrawableComponentState {
    pub(crate) drawable_flags_property_key: Option<u16>,
    pub(crate) clipping_shapes: Vec<ComponentHandle>,
}

impl RuntimeLayoutComponentState {
    fn new(type_name: &'static str) -> Self {
        Self {
            layout_node_dirty: Cell::new(false),
            layout_node_revision: Cell::new(0),
            layout: Cell::new(RuntimeLayoutRect::default()),
            animation_a: Cell::new(RuntimeLayoutAnimationData::default()),
            animation_b: Cell::new(RuntimeLayoutAnimationData::default()),
            is_smoothing_animation: Cell::new(false),
            initialized: Cell::new(false),
            just_added_to_host: Cell::new(false),
            animation_style: Cell::new(0),
            interpolation: Cell::new(0),
            interpolation_time: Cell::new(0.0),
            interpolator: Cell::new(None),
            inherited_interpolation: Cell::new(0),
            inherited_interpolation_time: Cell::new(0.0),
            inherited_interpolator: Cell::new(None),
            forced_width: Cell::new(None),
            forced_height: Cell::new(None),
            position_left_changed: Cell::new(true),
            position_top_changed: Cell::new(true),
            force_update_layout_bounds: Cell::new(false),
            clip_property_key: property_key_for_name(type_name, "clip"),
            style_id_property_key: property_key_for_name(type_name, "styleId"),
            style: None,
            layout_constraints: Vec::new(),
        }
    }

    pub(crate) fn mark_layout_node_dirty(&self) -> bool {
        let changed = !self.layout_node_dirty.replace(true);
        if changed {
            self.layout_node_revision
                .set(self.layout_node_revision.get().wrapping_add(1));
        }
        changed
    }

    pub(crate) fn layout_node_is_dirty(&self) -> bool {
        self.layout_node_dirty.get()
    }

    #[cfg(test)]
    pub(crate) fn layout_node_revision(&self) -> u64 {
        self.layout_node_revision.get()
    }

    fn clone_for_occurrence(&self) -> Self {
        Self {
            clip_property_key: self.clip_property_key,
            style_id_property_key: self.style_id_property_key,
            ..Self::new("LayoutComponent")
        }
    }

    pub(crate) fn retain_bounds(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        // The delegated solve consumed this retained node's dirty bit.
        self.layout_node_dirty.set(false);
        self.position_left_changed.set(false);
        self.position_top_changed.set(false);
        self.force_update_layout_bounds.set(false);
        let target = RuntimeLayoutRect {
            left: x,
            top: y,
            width,
            height,
        };
        let previous_draw_bounds = if self.animates() {
            self.current_animation_data().to
        } else {
            self.layout.get()
        };
        let draw_bounds_changed = previous_draw_bounds.width != target.width
            || previous_draw_bounds.height != target.height;
        // `Artboard::host` arms `LayoutComponent::m_justAddedToHost`. The
        // first parent-owned Yoga result becomes both the current layout and
        // animation endpoints instead of animating from the standalone
        // occurrence's zero/old position
        // (`artboard.cpp:1061-1073`; `layout_component.cpp:1041-1091`).
        if self.just_added_to_host.replace(false) || !self.initialized.replace(true) {
            self.initialized.set(true);
            self.layout.set(target);
            self.animation_a.set(RuntimeLayoutAnimationData {
                from: target,
                to: target,
                ..RuntimeLayoutAnimationData::default()
            });
            self.animation_b.set(RuntimeLayoutAnimationData {
                from: target,
                to: target,
                ..RuntimeLayoutAnimationData::default()
            });
            return draw_bounds_changed;
        }
        if !self.animates() {
            self.layout.set(target);
            let mut animation = self.animation_a.get();
            animation.to = target;
            animation.elapsed_seconds = 0.0;
            self.animation_a.set(animation);
            self.is_smoothing_animation.set(false);
            return draw_bounds_changed;
        }

        let mut animation = self.current_animation_data();
        if target == animation.to {
            return false;
        }
        if animation.elapsed_seconds != 0.0 {
            if self.is_smoothing_animation.get() {
                self.animation_a.set(self.animation_b.get());
            }
            self.is_smoothing_animation.set(true);
        } else {
            self.is_smoothing_animation.set(false);
        }
        animation = self.current_animation_data();
        animation.from = self.layout.get();
        animation.to = target;
        animation.elapsed_seconds = 0.0;
        self.set_current_animation_data(animation);
        draw_bounds_changed
    }

    pub(crate) fn added_to_host(&self) {
        self.just_added_to_host.set(true);
    }

    pub(crate) fn constraint_bounds(&self) -> (f32, f32, f32, f32) {
        let layout = self.layout.get();
        (
            0.0,
            0.0,
            self.forced_width.get().unwrap_or(layout.width),
            self.forced_height.get().unwrap_or(layout.height),
        )
    }

    pub(crate) fn current_bounds(&self) -> (f32, f32, f32, f32) {
        let layout = self.layout.get();
        (layout.left, layout.top, layout.width, layout.height)
    }

    pub(crate) fn target_bounds(&self) -> (f32, f32, f32, f32) {
        let layout = if self.animates() {
            self.current_animation_data().to
        } else {
            self.layout.get()
        };
        (layout.left, layout.top, layout.width, layout.height)
    }

    pub(crate) fn position(&self) -> (f32, f32) {
        let layout = self.layout.get();
        (layout.left, layout.top)
    }

    pub(crate) fn mark_position_left_changed(&self) {
        self.position_left_changed.set(true);
    }

    pub(crate) fn mark_position_top_changed(&self) {
        self.position_top_changed.set(true);
    }

    pub(crate) fn position_left_changed(&self) -> bool {
        self.position_left_changed.get()
    }

    pub(crate) fn position_top_changed(&self) -> bool {
        self.position_top_changed.get()
    }

    pub(crate) fn force_update_layout_bounds(&self) {
        self.force_update_layout_bounds.set(true);
    }

    pub(crate) fn should_force_update_layout_bounds(&self) -> bool {
        self.force_update_layout_bounds.get()
    }

    pub(crate) fn set_animation_style(
        &self,
        animation_style: u8,
        interpolation: u8,
        interpolation_time: f32,
        interpolator: Option<RuntimeInterpolator>,
    ) {
        self.animation_style.set(animation_style);
        self.interpolation.set(interpolation);
        self.interpolation_time.set(interpolation_time);
        self.interpolator.set(interpolator);
    }

    pub(crate) fn set_inherited_animation_style(
        &self,
        interpolation: u8,
        interpolation_time: f32,
        interpolator: Option<RuntimeInterpolator>,
    ) {
        self.inherited_interpolation.set(interpolation);
        self.inherited_interpolation_time.set(interpolation_time);
        self.inherited_interpolator.set(interpolator);
    }

    pub(crate) fn effective_interpolation(&self) -> u8 {
        match self.animation_style.get() {
            1 => self.inherited_interpolation.get(),
            2 => self.interpolation.get(),
            _ => 0,
        }
    }

    pub(crate) fn effective_interpolation_time(&self) -> f32 {
        match self.animation_style.get() {
            1 => self.inherited_interpolation_time.get(),
            2 => self.interpolation_time.get(),
            _ => 0.0,
        }
    }

    pub(crate) fn effective_interpolator(&self) -> Option<RuntimeInterpolator> {
        match self.animation_style.get() {
            1 => self
                .inherited_interpolator
                .get()
                .or(self.interpolator.get()),
            2 => self.interpolator.get(),
            _ => None,
        }
    }

    fn animates(&self) -> bool {
        self.animation_style.get() != 0
            && self.effective_interpolation() != 0
            && self.effective_interpolation_time() > 0.0
    }

    fn current_animation_data(&self) -> RuntimeLayoutAnimationData {
        if self.is_smoothing_animation.get() {
            self.animation_b.get()
        } else {
            self.animation_a.get()
        }
    }

    fn set_current_animation_data(&self, value: RuntimeLayoutAnimationData) {
        if self.is_smoothing_animation.get() {
            self.animation_b.set(value);
        } else {
            self.animation_a.set(value);
        }
    }

    pub(crate) fn advance_interpolation(
        &self,
        elapsed_seconds: f32,
        animate: bool,
    ) -> RuntimeLayoutAdvance {
        let mut animation = self.current_animation_data();
        if !animate || !self.animates() || animation.to == self.layout.get() {
            return RuntimeLayoutAdvance::default();
        }
        let interpolation_time = self.effective_interpolation_time();
        if self.is_smoothing_animation.get() {
            let mut animation_a = self.animation_a.get();
            let mut animation_b = self.animation_b.get();
            let mut factor = (animation_a.elapsed_seconds / interpolation_time).min(1.0);
            if self.effective_interpolation() != 1
                && let Some(interpolator) = self.effective_interpolator()
            {
                factor = interpolator.transform(factor);
            }
            animation_b.from = animation_a.interpolate(factor);
            self.animation_b.set(animation_b);
            if factor == 1.0 {
                self.animation_a.set(animation_b);
                self.is_smoothing_animation.set(false);
            } else {
                animation_a.elapsed_seconds += elapsed_seconds;
                self.animation_a.set(animation_a);
            }
        }

        animation = self.current_animation_data();
        if animation.elapsed_seconds >= interpolation_time {
            let previous = self.layout.replace(animation.to);
            let size_changed =
                previous.width != animation.to.width || previous.height != animation.to.height;
            if self.is_smoothing_animation.get() {
                self.is_smoothing_animation.set(false);
                self.animation_a.set(RuntimeLayoutAnimationData {
                    elapsed_seconds: 0.0,
                    ..self.animation_b.get()
                });
                let mut animation_b = self.animation_b.get();
                animation_b.elapsed_seconds = 0.0;
                self.animation_b.set(animation_b);
            } else {
                animation.elapsed_seconds = 0.0;
                self.animation_a.set(animation);
            }
            return RuntimeLayoutAdvance {
                keep_going: false,
                layout_changed: previous != animation.to,
                size_changed,
            };
        }

        let mut factor = (animation.elapsed_seconds / interpolation_time).min(1.0);
        if self.effective_interpolation() != 1
            && let Some(interpolator) = self.effective_interpolator()
        {
            factor = interpolator.transform(factor);
        }
        let current = animation.interpolate(factor);
        let previous = self.layout.replace(current);
        animation.elapsed_seconds += elapsed_seconds;
        self.set_current_animation_data(animation);
        RuntimeLayoutAdvance {
            keep_going: factor != 1.0,
            layout_changed: previous != current,
            size_changed: previous.width != current.width || previous.height != current.height,
        }
    }

    pub(crate) fn forced_width(&self, value: f32) {
        self.forced_width.set(Some(value));
    }

    pub(crate) fn forced_height(&self, value: f32) {
        self.forced_height.set(Some(value));
    }

    pub(crate) fn forced_size(&self) -> (Option<f32>, Option<f32>) {
        (self.forced_width.get(), self.forced_height.get())
    }

    pub(crate) fn apply_forced_size(&self, mut width: f32, mut height: f32) -> (f32, f32) {
        if let Some(forced) = self.forced_width.get() {
            width = forced;
        }
        if let Some(forced) = self.forced_height.get() {
            height = forced;
        }
        (width, height)
    }

    pub(crate) fn transform_property(&self, property: TransformProperty) -> Option<f32> {
        let layout = self.layout.get();
        match property {
            TransformProperty::X => Some(layout.left),
            TransformProperty::Y => Some(layout.top),
            _ => None,
        }
    }
}

impl RuntimeDrawableComponentState {
    fn new(type_name: &'static str) -> Self {
        Self {
            drawable_flags_property_key: property_key_for_name(type_name, "drawableFlags"),
            clipping_shapes: Vec::new(),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self {
            drawable_flags_property_key: self.drawable_flags_property_key,
            clipping_shapes: Vec::new(),
        }
    }
}

impl Default for RuntimeConstraintBoundsKind {
    fn default() -> Self {
        Self::Default
    }
}

impl RuntimeConstraintScratch {
    pub(crate) fn for_kind(kind: RuntimeConstraintKind) -> Self {
        match kind {
            RuntimeConstraintKind::Rotation => Self::Rotation {
                components_a: TransformComponents::default(),
                components_b: TransformComponents::default(),
            },
            RuntimeConstraintKind::Scale => Self::Scale {
                components_a: TransformComponents::default(),
                components_b: TransformComponents::default(),
            },
            RuntimeConstraintKind::Transform => Self::Transform {
                components_a: TransformComponents::default(),
                components_b: TransformComponents::default(),
            },
            _ => Self::None,
        }
    }
}

/// Runtime-only fields owned by C++ `Constraint` and its targeted/stateless
/// transform subclasses.
///
/// Generated values remain in the occurrence's sole generated backing store.
/// This payload retains only C++ runtime relations and scratch, plus the
/// module-static generated property keys address that backing store without
/// per-update schema/name lookup.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeConstraintState {
    pub(crate) kind: RuntimeConstraintKind,
    pub(crate) targeted: bool,
    pub(crate) requires_target: bool,
    pub(crate) target: Option<ComponentHandle>,
    pub(crate) scratch: RuntimeConstraintScratch,
}

impl RuntimeConstraintState {
    fn new(type_name: &'static str) -> Self {
        let kind = RuntimeConstraintKind::for_type(type_name);
        Self {
            kind,
            targeted: type_is_a(type_name, "TargetedConstraint"),
            requires_target: !matches!(
                type_name,
                "RotationConstraint" | "ScaleConstraint" | "TranslationConstraint"
            ),
            target: None,
            scratch: RuntimeConstraintScratch::for_kind(kind),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self {
            kind: self.kind,
            targeted: self.targeted,
            requires_target: self.requires_target,
            target: None,
            scratch: RuntimeConstraintScratch::for_kind(self.kind),
        }
    }
}

/// Runtime-only members for concrete Component subclasses reached by FL-A.
///
/// Optional inherited payloads are co-located with the one object occurrence:
/// a `PointsPath`, for example, owns both its `Node` and `Skinnable` subobject
/// state exactly as the C++ object does. They are not independent lookup
/// tables or alternate identities.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeConcreteComponentState {
    pub(crate) node: Option<RuntimeNodeState>,
    pub(crate) layout: Option<RuntimeLayoutComponentState>,
    pub(crate) constraint_bounds: RuntimeConstraintBoundsKind,
    pub(crate) constraint: Option<RuntimeConstraintState>,
    pub(crate) follow_path: Option<RuntimeFollowPathState>,
    pub(crate) ik: Option<RuntimeIkState>,
    pub(crate) constrainable_list: Option<RuntimeConstrainableListState>,
    pub(crate) scroll: Option<RuntimeScrollConstraintState>,
    pub(crate) scroll_bar: Option<RuntimeScrollBarConstraintState>,
    pub(crate) path: Option<RuntimePathState>,
    pub(crate) shape: Option<RuntimeShapeState>,
    pub(crate) bone: Option<RuntimeBoneState>,
    pub(crate) skin: Option<RuntimeSkinState>,
    pub(crate) tendon: Option<RuntimeTendonState>,
    pub(crate) skinnable: Option<RuntimeSkinnableState>,
    pub(crate) weight: Option<RuntimeWeightState>,
    pub(crate) vertex: Option<RuntimeVertexState>,
    pub(crate) scripted: Option<RuntimeScriptedComponentState>,
    pub(crate) text: Option<RuntimeTextState>,
    pub(crate) text_input: Option<RuntimeTextInputState>,
    pub(crate) drawable: Option<RuntimeDrawableComponentState>,
    pub(crate) solo: Option<RuntimeSoloState>,
}

impl RuntimeConcreteComponentState {
    pub(crate) fn for_type(type_name: &'static str) -> Self {
        Self {
            node: type_is_a(type_name, "Node").then(RuntimeNodeState::new),
            layout: type_is_a(type_name, "LayoutComponent")
                .then(|| RuntimeLayoutComponentState::new(type_name)),
            constraint_bounds: if type_is_a(type_name, "Text") {
                RuntimeConstraintBoundsKind::Text
            } else if type_is_a(type_name, "LayoutComponent") {
                RuntimeConstraintBoundsKind::Layout
            } else {
                RuntimeConstraintBoundsKind::Default
            },
            constraint: type_is_a(type_name, "Constraint")
                .then(|| RuntimeConstraintState::new(type_name)),
            follow_path: type_is_a(type_name, "FollowPathConstraint")
                .then(RuntimeFollowPathState::new),
            ik: (type_name == "IKConstraint").then(RuntimeIkState::default),
            constrainable_list: (type_name == "ArtboardComponentList")
                .then(RuntimeConstrainableListState::default),
            scroll: (type_name == "ScrollConstraint").then(RuntimeScrollConstraintState::default),
            scroll_bar: (type_name == "ScrollBarConstraint")
                .then(RuntimeScrollBarConstraintState::default),
            path: type_is_a(type_name, "Path").then(RuntimePathState::default),
            shape: type_is_a(type_name, "Shape").then(RuntimeShapeState::default),
            bone: RuntimeBoneState::for_type(type_name),
            skin: (type_name == "Skin").then(RuntimeSkinState::default),
            tendon: RuntimeTendonState::for_type(type_name),
            skinnable: RuntimeSkinnableState::for_type(type_name),
            weight: type_is_a(type_name, "Weight").then(|| RuntimeWeightState {
                is_cubic: type_name == "CubicWeight",
                ..RuntimeWeightState::default()
            }),
            vertex: type_is_a(type_name, "Vertex").then(RuntimeVertexState::default),
            scripted: matches!(
                type_name,
                "ScriptedDrawable" | "ScriptedLayout" | "ScriptedPathEffect"
            )
            .then(RuntimeScriptedComponentState::default),
            text: type_is_a(type_name, "Text").then(RuntimeTextState::new),
            text_input: (type_name == "TextInput").then(RuntimeTextInputState::default),
            drawable: (type_is_a(type_name, "Drawable") || type_is_a(type_name, "LayoutComponent"))
                .then(|| RuntimeDrawableComponentState::new(type_name)),
            solo: (type_name == "Solo").then(RuntimeSoloState::new),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        Self {
            node: self
                .node
                .as_ref()
                .map(RuntimeNodeState::clone_for_occurrence),
            layout: self
                .layout
                .as_ref()
                .map(RuntimeLayoutComponentState::clone_for_occurrence),
            constraint_bounds: self.constraint_bounds,
            constraint: self
                .constraint
                .as_ref()
                .map(RuntimeConstraintState::clone_for_occurrence),
            follow_path: self
                .follow_path
                .as_ref()
                .map(RuntimeFollowPathState::clone_for_occurrence),
            ik: self.ik.as_ref().map(RuntimeIkState::clone_for_occurrence),
            constrainable_list: self
                .constrainable_list
                .as_ref()
                .map(RuntimeConstrainableListState::clone_for_occurrence),
            scroll: self
                .scroll
                .as_ref()
                .map(RuntimeScrollConstraintState::clone_for_occurrence),
            scroll_bar: self
                .scroll_bar
                .as_ref()
                .map(RuntimeScrollBarConstraintState::clone_for_occurrence),
            path: self
                .path
                .as_ref()
                .map(RuntimePathState::clone_for_occurrence),
            shape: self
                .shape
                .as_ref()
                .map(RuntimeShapeState::clone_for_occurrence),
            bone: self
                .bone
                .as_ref()
                .map(RuntimeBoneState::clone_for_occurrence),
            skin: self.skin.as_ref().map(|_| RuntimeSkinState::default()),
            tendon: self.tendon.as_ref().map(|_| RuntimeTendonState::default()),
            skinnable: self
                .skinnable
                .as_ref()
                .map(RuntimeSkinnableState::clone_for_occurrence),
            weight: self.weight.as_ref().map(|weight| RuntimeWeightState {
                is_cubic: weight.is_cubic,
                ..RuntimeWeightState::default()
            }),
            vertex: self.vertex.as_ref().map(|_| RuntimeVertexState::default()),
            scripted: self
                .scripted
                .as_ref()
                .map(|_| RuntimeScriptedComponentState::default()),
            text: self
                .text
                .as_ref()
                .map(RuntimeTextState::clone_for_occurrence),
            text_input: self
                .text_input
                .as_ref()
                .map(|_| RuntimeTextInputState::default()),
            drawable: self
                .drawable
                .as_ref()
                .map(RuntimeDrawableComponentState::clone_for_occurrence),
            solo: self
                .solo
                .as_ref()
                .map(RuntimeSoloState::clone_for_occurrence),
        }
    }
}

fn type_is_a(type_name: &str, base: &str) -> bool {
    type_name == base
        || definition_by_name(type_name).is_some_and(|definition| definition.is_a(base))
}

impl Default for TransformRuntimeState {
    fn default() -> Self {
        Self {
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TransformComponents {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) rotation: f32,
    pub(crate) skew: f32,
}

impl Default for TransformComponents {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            skew: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeComponent {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub(crate) transform_property_keys: TransformPropertyKeys,
    pub capabilities: RuntimeComponentCapabilities,
    pub(crate) parent: Option<ComponentHandle>,
    /// C++ `TransformComponent::m_ParentTransformComponent`, resolved once in
    /// `onAddedClean`; unlike `parent`, this is nullable for non-transform
    /// containers.
    pub(crate) parent_transform: Option<ComponentHandle>,
    pub(crate) children: Vec<ComponentHandle>,
    pub(crate) constraints: Vec<ComponentHandle>,
    pub(crate) dependents: Vec<ComponentHandle>,
    pub(crate) collapsables: Vec<DataBindHandle>,
    /// Retained `LayoutComponent`s in this occurrence's parent chain,
    /// nearest-first.
    ///
    /// This replaces the former boolean ancestry mirror so callers can
    /// preserve C++ `Drawable::isChildOfLayout` identity, not merely answer
    /// that some layout exists (`src/drawable.cpp:45-59`).
    pub(crate) layout_ancestors: Vec<ComponentHandle>,
    pub(crate) constrained_layout_ancestor: Option<ComponentHandle>,
    pub(crate) graph_order: Option<GraphOrder>,
    pub dirt: ComponentDirt,
    pub(crate) path_revision: Cell<u64>,
    pub transform: TransformRuntimeState,
    pub(crate) concrete: RuntimeConcreteComponentState,
}

impl RuntimeComponent {
    pub(crate) fn path_revision(&self) -> u64 {
        self.path_revision.get()
    }

    pub(crate) fn bump_path_revision(&self) {
        self.path_revision
            .set(self.path_revision.get().wrapping_add(1));
    }

    pub(crate) fn from_graph_component(component: &ComponentNode) -> Self {
        Self {
            local_id: component.local_id,
            global_id: component.global_id,
            type_name: component.type_name,
            transform_property_keys: TransformPropertyKeys::for_type(component.type_name),
            capabilities: RuntimeComponentCapabilities {
                world_transform: component.capabilities.world_transform,
                transform: component.capabilities.transform,
            },
            parent: None,
            parent_transform: None,
            children: Vec::new(),
            constraints: Vec::new(),
            dependents: Vec::new(),
            collapsables: Vec::new(),
            layout_ancestors: Vec::new(),
            constrained_layout_ancestor: None,
            graph_order: None,
            dirt: ComponentDirt::FILTHY,
            path_revision: Cell::new(1),
            transform: TransformRuntimeState::default(),
            concrete: RuntimeConcreteComponentState::for_type(component.type_name),
        }
    }

    pub(crate) fn embedded(
        owner_local_id: usize,
        owner_global_id: u32,
        type_name: &'static str,
    ) -> Self {
        Self {
            local_id: owner_local_id,
            global_id: owner_global_id,
            type_name,
            transform_property_keys: TransformPropertyKeys::for_type(type_name),
            capabilities: RuntimeComponentCapabilities::default(),
            parent: None,
            parent_transform: None,
            children: Vec::new(),
            constraints: Vec::new(),
            dependents: Vec::new(),
            collapsables: Vec::new(),
            layout_ancestors: Vec::new(),
            constrained_layout_ancestor: None,
            graph_order: None,
            dirt: ComponentDirt::FILTHY,
            path_revision: Cell::new(1),
            transform: TransformRuntimeState::default(),
            concrete: RuntimeConcreteComponentState::default(),
        }
    }

    pub(crate) fn transform_property_key(&self, property: TransformProperty) -> Option<u16> {
        if self.transform_property_keys.is_for_type(self.type_name) {
            self.transform_property_keys.key(property)
        } else {
            TransformPropertyKeys::for_type(self.type_name).key(property)
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        let mut cloned = self.clone();
        cloned.parent = None;
        cloned.parent_transform = None;
        cloned.children.clear();
        cloned.constraints.clear();
        cloned.dependents.clear();
        cloned.collapsables.clear();
        cloned.layout_ancestors.clear();
        cloned.constrained_layout_ancestor = None;
        cloned.graph_order = None;
        cloned.dirt = ComponentDirt::FILTHY;
        cloned.transform = TransformRuntimeState::default();
        cloned.concrete = self.concrete.clone_for_occurrence();
        cloned
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }

    pub fn graph_order(&self) -> Option<usize> {
        // Pinned C++ assigns m_GraphOrder only while walking
        // m_DependencyOrder (`artboard.cpp:846-855`; `component.hpp:26,54`).
        // Preserve construction-only unset storage for Components that never
        // enter that schedule; exposing an observed allocator value would
        // manufacture a contract from indeterminate C++ storage (FLR-3).
        self.graph_order.map(GraphOrder::index)
    }

    pub(crate) fn update_transform(&mut self, authored: AuthoredTransform) {
        if !self.capabilities.transform {
            return;
        }

        let mut transform = Mat2D::from_rotation(authored.rotation);
        transform.0[4] = authored.x;
        transform.0[5] = authored.y;
        transform.scale_by_values(authored.scale_x, authored.scale_y);
        self.transform.local_transform = transform;
    }

    pub(crate) fn update_world_transform(&mut self, parent_world: Option<Mat2D>) {
        if self.type_name == "Artboard" || !self.capabilities.transform {
            return;
        }

        self.transform.world_transform = match parent_world {
            Some(parent_world) => parent_world.multiply(self.transform.local_transform),
            None => self.transform.local_transform,
        };
        if let Some(node) = self.concrete.node.as_ref() {
            node.mark_computed_local_dirty();
        }
    }

    pub(crate) fn update_render_opacity(&mut self, opacity: f32, parent_opacity: f32) {
        if !self.capabilities.transform {
            return;
        }

        self.transform.render_opacity = opacity * parent_opacity;
    }

    pub(crate) fn child_opacity(&self, authored_opacity: f32) -> f32 {
        // `WorldTransformComponent::childOpacity` returns its authored
        // opacity, while `TransformComponent` overrides it with settled
        // render opacity (`src/world_transform_component.cpp:8`,
        // `include/rive/transform_component.hpp:42`).
        if self.capabilities.transform {
            self.transform.render_opacity
        } else {
            authored_opacity
        }
    }
}

pub(crate) fn retain_runtime_component_layout_topology(objects: &mut InstanceObjectArena) {
    let handles = objects.component_handles().to_vec();
    for handle in handles {
        let layout_ancestors = runtime_layout_ancestors(handle, objects);
        let constrained_layout_ancestor = runtime_constrained_layout_ancestor(handle, objects);
        if let Some(component) = objects.component_mut(handle) {
            component.layout_ancestors = layout_ancestors;
            component.constrained_layout_ancestor = constrained_layout_ancestor;
        }
    }
}

/// Retain `LayoutComponent::m_style` once during the same onAdded lifecycle
/// that builds Component relations. The style's inherited Component parent is
/// the callback route in the opposite direction.
///
/// Generated style setters notify their already-linked LayoutComponents in
/// insertion order; no mutation-time Component scan is part of the C++ owner
/// model (`src/layout_component.cpp:478-485`;
/// `src/layout/layout_component_style.cpp:208-221`).
pub(crate) fn retain_runtime_layout_component_styles(
    file: &RuntimeFile,
    slots: &[InstanceSlot],
    objects: &mut InstanceObjectArena,
) {
    let animation_style_key = property_key_for_name("LayoutComponentStyle", "animationStyleType");
    let interpolation_key = property_key_for_name("LayoutComponentStyle", "interpolationType");
    let interpolation_time_key = property_key_for_name("LayoutComponentStyle", "interpolationTime");
    let interpolator_id_key = property_key_for_name("LayoutComponentStyle", "interpolatorId");
    let handles = objects.component_handles().to_vec();
    for owner in handles.iter().copied() {
        let style = objects.component(owner).and_then(|component| {
            let layout = component.concrete.layout.as_ref()?;
            let style_local = usize::try_from(
                objects.component_uint_property(owner, layout.style_id_property_key?)?,
            )
            .ok()?;
            objects.component_handle(style_local)
        });
        let style_values = style.map(|style| {
            let style_local = objects
                .component_local_id(style)
                .expect("retained style handle must address its object");
            let animation_style = animation_style_key
                .and_then(|key| objects.component_uint_property(style, key))
                .unwrap_or(0) as u8;
            let interpolation = interpolation_key
                .and_then(|key| objects.component_uint_property(style, key))
                .unwrap_or(0) as u8;
            let interpolation_time = interpolation_time_key
                .and_then(|key| objects.double_property(style_local, key))
                .unwrap_or(0.0);
            let interpolator = interpolator_id_key
                .and_then(|key| objects.component_uint_property(style, key))
                .and_then(|local_id| usize::try_from(local_id).ok())
                .and_then(|local_id| slots.get(local_id))
                .and_then(|slot| file.object(slot.source_global_id as usize))
                .and_then(RuntimeInterpolator::from_object);
            (
                animation_style,
                interpolation,
                interpolation_time,
                interpolator,
            )
        });
        if let Some(component) = objects.component_mut(owner)
            && let Some(layout) = component.concrete.layout.as_mut()
        {
            layout.style = style;
            if let Some((animation_style, interpolation, interpolation_time, interpolator)) =
                style_values
            {
                layout.set_animation_style(
                    animation_style,
                    interpolation,
                    interpolation_time,
                    interpolator,
                );
            }
        }
    }

    // `LayoutComponent::cascadeLayoutStyle` retains the effective parent
    // interpolation on each child occurrence. Dependency order is
    // parent-before-child after the component relation build, so each child
    // reads one already-settled parent owner rather than rediscovering style
    // ancestry per frame (`src/layout_component.cpp:1218-1282`).
    let schedule = objects.dependency_order().to_vec();
    for owner in schedule {
        let inherited = objects
            .component(owner)
            .and_then(|component| component.parent)
            .and_then(|parent| objects.component(parent))
            .and_then(|parent| parent.concrete.layout.as_ref())
            .map(|layout| {
                (
                    layout.effective_interpolation(),
                    layout.effective_interpolation_time(),
                    layout.effective_interpolator(),
                )
            })
            .unwrap_or((0, 0.0, None));
        if let Some(layout) = objects
            .component(owner)
            .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.set_inherited_animation_style(inherited.0, inherited.1, inherited.2);
        }
    }
}

/// Resolves `TextInput::m_scrollConstraint` once during `onAddedClean`.
///
/// C++ walks `parent()->parent()` and retains the first ScrollConstraint in
/// that TransformComponent's ordered constraint list
/// (`src/text/text_input.cpp:87-106`).
pub(crate) fn retain_runtime_text_input_scroll_constraints(objects: &mut InstanceObjectArena) {
    let handles = objects.component_handles().to_vec();
    for text_input in handles {
        if objects
            .component(text_input)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_none()
        {
            continue;
        }
        let scroll_constraint = objects
            .component(text_input)
            .and_then(|component| component.parent)
            .and_then(|parent| objects.component(parent))
            .and_then(|parent| parent.parent)
            .and_then(|owner| objects.component(owner))
            .and_then(|owner| {
                owner.constraints.iter().copied().find(|constraint| {
                    objects
                        .component(*constraint)
                        .is_some_and(|component| component.concrete.scroll.is_some())
                })
            });
        if let Some(state) = objects
            .component_mut(text_input)
            .and_then(|component| component.concrete.text_input.as_mut())
        {
            state.scroll_constraint = scroll_constraint;
        }
    }
}

fn runtime_layout_ancestors(
    mut handle: ComponentHandle,
    objects: &InstanceObjectArena,
) -> Vec<ComponentHandle> {
    // Cycle guard: a malformed-but-accepted file can make `parent_local` form a
    // parent cycle (A -> B -> A). C++ hangs on this input (Path::onAddedClean's
    // unbounded shape-parent walk); Component::validate only checks that a
    // parent resolves, not that the chain is acyclic. We deliberately DIVERGE
    // and terminate the walk, mirroring C++'s own cycle-guard idiom -- the
    // visited-set from DependencySorter::visit (src/dependency_sorter.cpp, the
    // m_Perm/m_Temp sets) -- so an embedded-SDK hang becomes a graceful
    // no-ancestor result. Unreachable on any valid file.
    let mut layouts = Vec::new();
    let mut visited = BTreeSet::new();
    while let Some(component) = objects.component(handle) {
        if component.type_name == "LayoutComponent" {
            layouts.push(handle);
        }
        if !visited.insert(handle) {
            layouts.clear();
            return layouts;
        }
        let Some(parent) = component.parent else {
            return layouts;
        };
        handle = parent;
    }
    layouts
}

fn runtime_constrained_layout_ancestor(
    mut handle: ComponentHandle,
    objects: &InstanceObjectArena,
) -> Option<ComponentHandle> {
    // Cycle guard: see runtime_layout_ancestors above. Same
    // malformed parent-cycle input, same deliberate terminate-where-C++-hangs
    // divergence, same DependencySorter visited-set idiom
    // (src/dependency_sorter.cpp). Terminating early yields no constrained
    // ancestor, which is the safe "as if the chain ended" result.
    let mut saw_constraint = false;
    let mut visited = BTreeSet::new();
    while let Some(component) = objects.component(handle) {
        if component.type_name == "LayoutComponent" {
            return saw_constraint.then_some(handle);
        }
        saw_constraint |= !component.constraints.is_empty();
        if !visited.insert(handle) {
            return None;
        }
        let Some(parent) = component.parent else {
            return None;
        };
        handle = parent;
    }
    None
}

include!("solo.rs");

#[cfg(test)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoloMappingWork {
    pub(crate) analyses: usize,
    pub(crate) batch_queries: usize,
    pub(crate) visited_slots: usize,
}

#[cfg(test)]
thread_local! {
    static SOLO_MAPPING_WORK: Cell<SoloMappingWork> = const {
        Cell::new(SoloMappingWork {
            analyses: 0,
            batch_queries: 0,
            visited_slots: 0,
        })
    };
}

#[cfg(test)]
pub(crate) fn reset_solo_mapping_work() {
    SOLO_MAPPING_WORK.set(SoloMappingWork::default());
}

#[cfg(test)]
pub(crate) fn solo_mapping_work() -> SoloMappingWork {
    SOLO_MAPPING_WORK.get()
}

#[cfg(test)]
fn record_solo_mapping_analysis() {
    SOLO_MAPPING_WORK.with(|slot| {
        let mut work = slot.get();
        work.analyses += 1;
        slot.set(work);
    });
}

#[cfg(test)]
fn record_solo_mapping_batch_query(visited_slots: usize) {
    SOLO_MAPPING_WORK.with(|slot| {
        let mut work = slot.get();
        work.batch_queries += 1;
        work.visited_slots += visited_slots;
        slot.set(work);
    });
}

pub(crate) fn retain_runtime_solos(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    objects: &mut InstanceObjectArena,
) {
    let solo_handles = graph
        .components
        .iter()
        .filter(|component| component.type_name == "Solo")
        .filter_map(|component| objects.component_handle(component.local_id))
        .collect::<Vec<_>>();
    if solo_handles.is_empty() {
        return;
    }

    let runtime_local_by_cpp_local = artboard_index_for_graph(file, graph)
        .map(|artboard_index| runtime_local_by_cpp_artboard_local(file, graph, artboard_index))
        .unwrap_or_default();
    let cpp_local_by_runtime_local = runtime_local_by_cpp_local
        .into_iter()
        .map(|(cpp_local, runtime_local)| (runtime_local, cpp_local))
        .collect::<BTreeMap<_, _>>();

    for solo_handle in solo_handles {
        let cpp_local_ids = objects
            .component(solo_handle)
            .map(|solo| solo.children.clone())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|child| {
                let child_component = objects.component(child)?;
                cpp_local_by_runtime_local
                    .get(&child_component.local_id)
                    .copied()
            })
            .collect();
        if let Some(solo) = objects
            .component_mut(solo_handle)
            .and_then(|component| component.concrete.solo.as_mut())
        {
            solo.cpp_local_ids = cpp_local_ids;
        }
    }
}

fn runtime_local_by_cpp_artboard_local(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboard_index: usize,
) -> BTreeMap<usize, usize> {
    #[cfg(test)]
    record_solo_mapping_analysis();

    let runtime_local_by_global = graph
        .local_objects
        .iter()
        .map(|local_object| (local_object.global_id, local_object.local_id))
        .collect::<BTreeMap<_, _>>();
    let slots = file
        .artboard_local_object_slots(artboard_index)
        .unwrap_or_default();

    #[cfg(test)]
    record_solo_mapping_batch_query(slots.len());

    slots
        .into_iter()
        .enumerate()
        .filter_map(|(cpp_local, object)| {
            object.and_then(|object| {
                runtime_local_by_global
                    .get(&object.id)
                    .copied()
                    .map(|runtime_local| (cpp_local, runtime_local))
            })
        })
        .collect()
}

#[cfg(test)]
mod constraint_state_tests {
    use super::{
        RuntimeConcreteComponentState, RuntimeConstraintKind, RuntimeConstraintScratch,
        TransformComponents,
    };

    #[test]
    fn stateless_constraint_scratch_exists_only_on_exact_cpp_leaf_types() {
        for (type_name, expected_kind) in [
            ("DistanceConstraint", RuntimeConstraintKind::Distance),
            ("RotationConstraint", RuntimeConstraintKind::Rotation),
            ("ScaleConstraint", RuntimeConstraintKind::Scale),
            ("TransformConstraint", RuntimeConstraintKind::Transform),
            ("TranslationConstraint", RuntimeConstraintKind::Translation),
        ] {
            let state = RuntimeConcreteComponentState::for_type(type_name)
                .constraint
                .expect("constraint payload");
            assert_eq!(state.kind, expected_kind);
            assert_eq!(
                matches!(
                    (expected_kind, state.scratch),
                    (
                        RuntimeConstraintKind::Rotation,
                        RuntimeConstraintScratch::Rotation { .. }
                    ) | (
                        RuntimeConstraintKind::Scale,
                        RuntimeConstraintScratch::Scale { .. }
                    ) | (
                        RuntimeConstraintKind::Transform,
                        RuntimeConstraintScratch::Transform { .. }
                    ) | (
                        RuntimeConstraintKind::Distance | RuntimeConstraintKind::Translation,
                        RuntimeConstraintScratch::None
                    )
                ),
                true,
                "{type_name} scratch shape"
            );
        }
    }

    #[test]
    fn list_follow_path_retains_its_concrete_cpp_constraint_kind() {
        let follow_path = RuntimeConcreteComponentState::for_type("FollowPathConstraint")
            .constraint
            .expect("follow-path constraint payload");
        let list_follow_path = RuntimeConcreteComponentState::for_type("ListFollowPathConstraint")
            .constraint
            .expect("list-follow-path constraint payload");

        assert_eq!(follow_path.kind, RuntimeConstraintKind::FollowPath);
        assert_eq!(list_follow_path.kind, RuntimeConstraintKind::ListFollowPath);
        assert!(
            RuntimeConcreteComponentState::for_type("ListFollowPathConstraint")
                .follow_path
                .is_some()
        );
    }

    #[test]
    fn constraint_clone_resets_target_and_exact_leaf_scratch() {
        let mut state = RuntimeConcreteComponentState::for_type("RotationConstraint")
            .constraint
            .expect("rotation state");
        state.target = Some(super::ComponentHandle::from_index(17));
        state.scratch = RuntimeConstraintScratch::Rotation {
            components_a: TransformComponents {
                rotation: 1.0,
                ..TransformComponents::default()
            },
            components_b: TransformComponents {
                scale_x: 2.0,
                ..TransformComponents::default()
            },
        };

        let cloned = state.clone_for_occurrence();
        assert_eq!(cloned.target, None);
        match cloned.scratch {
            RuntimeConstraintScratch::Rotation {
                components_a,
                components_b,
            } => {
                assert_eq!(components_a, TransformComponents::default());
                assert_eq!(components_b, TransformComponents::default());
            }
            _ => panic!("rotation clone must retain the Rotation scratch shape"),
        }
    }
}

#[cfg(test)]
mod advancing_owner_tests {
    use super::RuntimeConcreteComponentState;

    #[test]
    fn layout_interpolation_uses_cpp_apply_then_increment_timing() {
        let layout = RuntimeConcreteComponentState::for_type("LayoutComponent")
            .layout
            .expect("layout state");
        layout.set_animation_style(2, 1, 1.0, None);
        layout.retain_bounds(0.0, 0.0, 10.0, 20.0);
        layout.retain_bounds(100.0, 50.0, 30.0, 40.0);

        let first = layout.advance_interpolation(0.25, true);
        assert!(first.keep_going);
        assert!(!first.layout_changed);
        assert_eq!(layout.constraint_bounds(), (0.0, 0.0, 10.0, 20.0));

        let second = layout.advance_interpolation(0.25, true);
        assert!(second.keep_going);
        assert!(second.layout_changed);
        assert!(second.size_changed);
        assert_eq!(layout.constraint_bounds(), (0.0, 0.0, 15.0, 25.0));

        let _ = layout.advance_interpolation(0.5, true);
        let final_step = layout.advance_interpolation(0.25, true);
        assert!(!final_step.keep_going);
        assert_eq!(layout.constraint_bounds(), (0.0, 0.0, 30.0, 40.0));
    }

    #[test]
    fn text_input_clone_rebuilds_scroll_link_and_drag_state_cold() {
        let mut concrete = RuntimeConcreteComponentState::for_type("TextInput");
        let text_input = concrete.text_input.as_mut().expect("text input state");
        text_input.scroll_constraint = Some(super::ComponentHandle::from_index(7));
        text_input.is_dragging = true;
        text_input.last_drag_world_position = (12.0, 34.0);
        text_input.scroll_x = 45.0;
        text_input.scroll_y = -45.0;

        let cloned = concrete.clone_for_occurrence();
        let cloned = cloned.text_input.expect("cloned text input state");
        assert_eq!(cloned.scroll_constraint, None);
        assert!(!cloned.is_dragging);
        assert!(cloned.last_drag_world_position.0.is_nan());
        assert!(cloned.last_drag_world_position.1.is_nan());
        assert_eq!((cloned.scroll_x, cloned.scroll_y), (0.0, 0.0));
    }
}
