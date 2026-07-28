use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result};
use nuxie_binary::RuntimeFile;
use nuxie_graph::{
    AdvancingComponentKind, ArtboardGraph, DependencyNode, DependencyNodeKind,
    ResettingComponentKind,
};
use nuxie_render_api::Factory as RenderFactory;
use nuxie_schema::definition_by_name;

use crate::animation::{
    LinearAnimationInstance, RuntimeInterpolator, RuntimeJoystick, RuntimeKeyedCallback,
    RuntimeLinearAnimation, RuntimeLinearAnimationHandle, build_linear_animations,
    build_runtime_joysticks,
};
use crate::artboard_data_bind::{
    RuntimeArtboardAuthoredDataBindStates, RuntimeArtboardContextSourceValue,
    RuntimeArtboardConverterPropertyBindingInstance, RuntimeArtboardCustomPropertyBindingInstance,
    RuntimeArtboardDataBindSourceQueues, RuntimeArtboardDataBindTargetQueues,
    RuntimeArtboardFormulaTokenBindingStates, RuntimeArtboardImageAssetBindingInstance,
    RuntimeArtboardLayoutComputedBindingInstance, RuntimeArtboardListBindingInstance,
    RuntimeArtboardNestedHostBindingInstance, RuntimeArtboardNumericSourceBindingInstance,
    RuntimeArtboardPropertyBindingInstance, RuntimeArtboardRetainedSubordinateConverterOperands,
    RuntimeArtboardSoloBindingInstance, RuntimeArtboardSoloSourceBindingInstance,
    RuntimeArtboardTextListBindingInstance, RuntimeNestedChildContextUpdate,
    RuntimeOwnedDataContext, apply_artboard_name_based_color_data_bind_defaults,
    build_artboard_authored_data_bind_states, build_artboard_converter_property_bindings,
    build_artboard_custom_property_bindings, build_artboard_default_view_model_values,
    build_artboard_formula_token_bindings, build_artboard_image_asset_bindings,
    build_artboard_layout_computed_bindings, build_artboard_list_bindings,
    build_artboard_nested_host_bindings, build_artboard_numeric_source_bindings,
    build_artboard_property_bindings, build_artboard_solo_bindings,
    build_artboard_solo_source_bindings, build_artboard_text_list_bindings,
    build_nested_host_data_bind_source_local_slots, build_nested_host_data_bind_source_locals,
    build_nested_host_view_model_instance_locals,
    reunite_artboard_shared_data_bind_converter_states,
};
use crate::components::{
    AuthoredTransform, ComponentDirt, ComponentHandle, Mat2D, RuntimeComponent,
    RuntimeConstrainableListState, RuntimeIkChainLink, RuntimeSkinnableKind, TransformComponents,
    TransformProperty, UpdateComponentsReport, retain_runtime_component_layout_topology,
    retain_runtime_layout_component_styles, retain_runtime_solos,
    retain_runtime_text_input_scroll_constraints,
};
use crate::constraints::{
    apply_scroll_offset_changed, component_list_virtualization, retain_runtime_scroll_constraints,
    runtime_scroll_double_property, set_runtime_scroll_double_property,
};
use crate::data_bind_graph::{
    RuntimeDataBindGraphConverterBuildCache, RuntimeDataBindGraphFormulaRandomSource,
    RuntimeDataBindGraphValue,
};
use crate::draw::{
    RuntimeClippingShapeList, RuntimeDrawableList, RuntimeInitialNestedLayoutPaintFrame,
    RuntimeLayoutBounds, RuntimeShapeList, retain_runtime_drawable_component_topology,
    runtime_apply_component_list_item_layout_bounds, runtime_component_list_item_base_transforms,
    runtime_component_list_item_layout_size,
};
use crate::objects::{ComponentAddress, InstanceObjectArena, InstanceSlot, ObjectHandle};
use crate::properties::{
    JOYSTICK_FLAG_INVERT_X, JOYSTICK_FLAG_INVERT_Y, RuntimeArtboardDimensions,
    joystick_flags_property_key, joystick_x_property_key, joystick_y_property_key,
    layout_component_style_display_value_property_key, property_key_for_name,
    solid_color_value_property_key, solo_active_component_id_property_key,
    transform_property_for_key,
};
use crate::script_asset::{RuntimeScriptImplementedMethods, RuntimeScriptedObjectOccurrence};
use crate::scripting::{
    NoopScriptHost, RuntimeScriptInstanceHandle, ScriptArtboard, ScriptError, ScriptHost,
    ScriptInstance, ScriptMethod, ScriptValue, ScriptViewModel,
};
use crate::state_machine::{
    RuntimeFileStateMachineActionCatalog, RuntimeNestedStateMachineInstance,
    RuntimeNestedStateMachineReport, RuntimeStateMachine, StateMachineInputKind,
    StateMachineInstance, StateMachineReportedEvent, build_state_machines_with_action_catalog,
};
use crate::view_model::{
    RuntimeFontAssetValue, RuntimeImportedViewModelInstanceContext,
    RuntimeOwnedViewModelListHandle, RuntimeOwnedViewModelListItemEntry,
    set_component_list_item_index,
};
use crate::view_model_cell::RuntimeFileViewModelInstanceCatalog;
use crate::{
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance,
};

fn generated_mat2d(
    objects: &InstanceObjectArena,
    local_id: usize,
    type_name: &'static str,
) -> Mat2D {
    let value = |name, default| {
        property_key_for_name(type_name, name)
            .and_then(|key| objects.double_property(local_id, key))
            .unwrap_or(default)
    };
    Mat2D([
        value("xx", 1.0),
        value("xy", 0.0),
        value("yx", 0.0),
        value("yy", 1.0),
        value("tx", 0.0),
        value("ty", 0.0),
    ])
}

fn deform_point_from_skin(
    point: (f32, f32),
    indices: u32,
    weights: u32,
    skin_world: Mat2D,
    bone_transforms: &[Mat2D],
) -> Option<(f32, f32)> {
    let mut blended = [0.0; 6];
    for index in 0..4 {
        let shift = index * 8;
        let weight = ((weights >> shift) & 0xff) as u8;
        if weight == 0 {
            continue;
        }
        let bone_index = ((indices >> shift) & 0xff) as usize;
        let bone_transform = bone_transforms.get(bone_index)?;
        let normalized_weight = f32::from(weight) / 255.0;
        for (target, value) in blended.iter_mut().zip(bone_transform.0) {
            // Clang contracts each C++ `accumulator += value * weight` in
            // `Weight::deform`; Rust does not contract implicitly. Preserve
            // that per-site rounding exactly (`src/bones/weight.cpp:35-48`;
            // `docs/PORTING.md` §4.2).
            *target = value.mul_add(normalized_weight, *target);
        }
    }
    let skinned = skin_world.transform_point(point.0, point.1);
    Some(Mat2D(blended).transform_point(skinned.0, skinned.1))
}

/// Rejection from attaching host-supplied bytes to one external `FontAsset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalFontAssetError {
    UnknownAsset { asset_id: u32 },
    WrongAssetKind { asset_id: u32, actual: &'static str },
    InvalidFont { asset_id: u32 },
}

/// Inputs that make C++ `Image::updateImageScale()` overwrite the public
/// `scaleX`/`scaleY` fields for a pre-7.2 file. The draw module packs the
/// decoded image identity, dimensions, fit/alignment, and controlled layout
/// size into these words. A public scale write remains authoritative until one
/// of those inputs changes and C++ would run `updateImageScale()` again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeLegacyImageLayoutScaleKey([u64; 12]);

impl RuntimeLegacyImageLayoutScaleKey {
    pub(crate) fn new(words: [u64; 12]) -> Self {
        Self(words)
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimeLegacyImageLayoutScaleState {
    key: RuntimeLegacyImageLayoutScaleKey,
    scale_x: f32,
    scale_y: f32,
    user_scale_x: bool,
    user_scale_y: bool,
}

fn legacy_image_layout_scale_axis(property_key: u16) -> Option<bool> {
    static SCALE_KEYS: std::sync::OnceLock<(Option<u16>, Option<u16>)> = std::sync::OnceLock::new();
    let (scale_x_key, scale_y_key) = *SCALE_KEYS.get_or_init(|| {
        (
            property_key_for_name("Node", "scaleX"),
            property_key_for_name("Node", "scaleY"),
        )
    });
    if scale_x_key == Some(property_key) {
        Some(true)
    } else if scale_y_key == Some(property_key) {
        Some(false)
    } else {
        None
    }
}

impl std::fmt::Display for ExternalFontAssetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAsset { asset_id } => {
                write!(formatter, "file has no asset with semantic id {asset_id}")
            }
            Self::WrongAssetKind { asset_id, actual } => {
                write!(formatter, "asset {asset_id} is {actual}, not FontAsset")
            }
            Self::InvalidFont { asset_id } => {
                write!(formatter, "asset {asset_id} bytes are not a valid font")
            }
        }
    }
}

impl std::error::Error for ExternalFontAssetError {}

#[derive(Debug)]
struct RuntimeArtboardInstanceIdentity(u64);

impl RuntimeArtboardInstanceIdentity {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Clone for RuntimeArtboardInstanceIdentity {
    fn clone(&self) -> Self {
        Self::next()
    }
}

/// Runtime script state is occurrence-owned and must never survive a raw
/// ArtboardInstance clone. Draw/layout code clones artboards transiently; a
/// normal derived clone of these collections would alias one Lua table into
/// multiple concrete occurrences.
#[derive(Debug)]
pub(crate) struct RuntimeScriptState<T>(T);

impl<T: Default> Default for RuntimeScriptState<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Default> Clone for RuntimeScriptState<T> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<T> std::ops::Deref for RuntimeScriptState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for RuntimeScriptState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: IntoIterator> IntoIterator for RuntimeScriptState<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a RuntimeScriptState<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeAdvancingComponent {
    pub(crate) local_id: usize,
    pub(crate) object: ObjectHandle,
    pub(crate) component: Option<ComponentHandle>,
    pub(crate) kind: AdvancingComponentKind,
}

enum RuntimeScriptAdvanceMode<'a> {
    Disabled,
    HostOnly,
    Factory(&'a mut dyn RenderFactory),
}

impl RuntimeScriptAdvanceMode<'_> {
    fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    fn call(
        &mut self,
        instance: &mut dyn ScriptInstance,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        match self {
            Self::Disabled => unreachable!("disabled script advance cannot be dispatched"),
            Self::HostOnly => instance.call_method(ScriptMethod::Advance, args, host),
            Self::Factory(factory) => {
                instance.call_method_with_factory(ScriptMethod::Advance, args, host, *factory)
            }
        }
    }
}

enum RuntimeScriptUpdateMode<'a> {
    HostOnly,
    Factory(&'a mut dyn RenderFactory),
}

impl RuntimeScriptUpdateMode<'_> {
    fn call(
        &mut self,
        instance: &mut dyn ScriptInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        match self {
            Self::HostOnly => instance.call_method(ScriptMethod::Update, &[], host),
            Self::Factory(factory) => {
                instance.call_method_with_factory(ScriptMethod::Update, &[], host, *factory)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeResettingComponent {
    pub(crate) local_id: usize,
    pub(crate) component: ComponentHandle,
    pub(crate) kind: ResettingComponentKind,
}

#[derive(Clone, Copy)]
pub struct RuntimeComponents<'a> {
    arena: &'a InstanceObjectArena,
}

impl<'a> RuntimeComponents<'a> {
    pub fn len(self) -> usize {
        self.arena.component_handles().len()
    }

    pub fn is_empty(self) -> bool {
        self.arena.component_handles().is_empty()
    }

    pub fn iter(self) -> impl Iterator<Item = &'a RuntimeComponent> + 'a {
        self.arena
            .component_handles()
            .iter()
            .filter_map(|handle| self.arena.component(*handle))
    }
}

#[derive(Debug)]
pub struct ArtboardInstance {
    instance_identity: RuntimeArtboardInstanceIdentity,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) clip: bool,
    /// C++ `Artboard::m_FrameOrigin`: clone-owned draw state. Root artboards
    /// default true; mounted nested/scripted/component-list occurrences set it
    /// false at the same ownership boundary as C++.
    pub(crate) frame_origin: Cell<bool>,
    /// C++ `Artboard::m_FrameID`, incremented by the public draw entry before
    /// `drawInternal`; mounted children recurse directly and do not increment.
    pub(crate) frame_id: Cell<u64>,
    pub(crate) slots: Vec<InstanceSlot>,
    pub(crate) objects: InstanceObjectArena,
    pub(crate) joysticks: Vec<RuntimeJoystick>,
    /// C++ `Artboard::m_advancingComponents`, retained in authored object
    /// order and rebuilt against clone-owned occurrences.
    pub(crate) advancing_components: Vec<RuntimeAdvancingComponent>,
    /// C++ `Artboard::m_Resettables`, retained in authored object order.
    pub(crate) resetting_components: Vec<RuntimeResettingComponent>,
    /// C++ `Artboard::m_ComponentLists`, retained in authored object order.
    /// Handles address the concrete list occurrences in `objects`; steady
    /// paths never rescan Components to rediscover this family
    /// (`src/artboard.cpp:330-395`).
    component_lists: Vec<ComponentHandle>,
    pub(crate) joysticks_apply_before_update: bool,
    pub(crate) linear_animations: Arc<Vec<RuntimeLinearAnimation>>,
    /// C++ uses one process-global empty LinearAnimation for unresolved
    /// AnimationState/BlendAnimation pointers. Runtime definitions can retain
    /// single-threaded script handles in Rust, so each Artboard owns one shared
    /// safe equivalent and all of its unresolved occurrences point here.
    pub(crate) empty_linear_animation: Arc<RuntimeLinearAnimation>,
    pub(crate) state_machines: Arc<Vec<RuntimeStateMachine>>,
    pub(crate) script_instances_by_global:
        RuntimeScriptState<BTreeMap<u32, RuntimeScriptedObjectOccurrence>>,
    /// Generation of concrete script occurrence attachments. Rust's
    /// authenticated facade mounts these after Artboard cloning, while C++
    /// mounts them before state-machine input-group construction.
    script_attachment_generation: u64,
    pub(crate) scripted_data_converter_instances_by_global:
        RuntimeScriptState<BTreeMap<u32, RuntimeScriptInstanceHandle>>,
    has_scripted_drawables: bool,
    nested_script_owned_contexts: BTreeMap<u32, RuntimeOwnedViewModelInstance>,
    script_update_error: Option<ScriptError>,
    /// C++ `Artboard::m_FocusManager`: a mounted child Artboard retains the
    /// parent state-machine focus domain so component-list rows and nested
    /// state machines can install it at their exact link boundary.
    ///
    /// The root `StateMachineInstance` remains the projection owner. This is
    /// only a shared, non-projecting domain handle and is reset on cold clone.
    external_focus_domain: Option<crate::focus::RuntimeFocusTree>,
    pub(crate) nested_artboards: RuntimeNestedArtboards,
    pub(crate) nested_artboard_locals: Vec<usize>,
    newly_uncollapsed_nested_artboards: BTreeSet<usize>,
    pub(crate) graph_global_id: u32,
    build_context: Option<RuntimeArtboardBuildContext>,
    pub(crate) nested_context_source_tree_cache: Cell<Option<(u64, bool)>>,
    nested_layout_bounds: Option<RuntimeNestedLayoutBoundsFrame>,
    pub(crate) artboard_data_bind_values: BTreeMap<Arc<[u32]>, RuntimeDataBindGraphValue>,
    pub(crate) artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource,
    pub(crate) artboard_owned_view_model_context: Option<RuntimeOwnedViewModelContext>,
    pub(crate) artboard_owned_data_context: Option<RuntimeOwnedDataContext>,
    pub(crate) artboard_owned_view_model_handle: Option<RuntimeOwnedViewModelContextHandle>,
    pub(crate) artboard_authored_data_bind_states: RuntimeArtboardAuthoredDataBindStates,
    /// Structural ViewModel replacement pushes a relink request just as C++
    /// `ViewModelInstance::addDependent` does; steady frames never poll a
    /// mutation generation (`data_context.cpp:265-332,399-442`).
    pub(crate) artboard_owned_view_model_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink,
    pub(crate) artboard_property_bindings: Vec<RuntimeArtboardPropertyBindingInstance>,
    pub(crate) artboard_image_asset_bindings: Vec<RuntimeArtboardImageAssetBindingInstance>,
    pub(crate) artboard_data_bind_target_queues: RuntimeArtboardDataBindTargetQueues,
    pub(crate) artboard_data_bind_source_queues: RuntimeArtboardDataBindSourceQueues,
    pub(crate) artboard_retained_subordinate_converter_operands:
        Vec<RuntimeArtboardRetainedSubordinateConverterOperands>,
    pub(crate) artboard_custom_property_bindings: Vec<RuntimeArtboardCustomPropertyBindingInstance>,
    pub(crate) artboard_layout_computed_bindings: Vec<RuntimeArtboardLayoutComputedBindingInstance>,
    pub(crate) artboard_numeric_source_bindings: Vec<RuntimeArtboardNumericSourceBindingInstance>,
    pub(crate) artboard_formula_token_bindings: RuntimeArtboardFormulaTokenBindingStates,
    pub(crate) artboard_converter_property_bindings:
        Vec<RuntimeArtboardConverterPropertyBindingInstance>,
    pub(crate) artboard_solo_bindings: Vec<RuntimeArtboardSoloBindingInstance>,
    pub(crate) artboard_solo_source_bindings: Vec<RuntimeArtboardSoloSourceBindingInstance>,
    pub(crate) artboard_nested_host_bindings: Vec<RuntimeArtboardNestedHostBindingInstance>,
    pub(crate) artboard_list_bindings: Vec<RuntimeArtboardListBindingInstance>,
    pub(crate) artboard_text_list_bindings: Vec<RuntimeArtboardTextListBindingInstance>,
    pub(crate) artboard_context_source_values_scratch: Vec<RuntimeArtboardContextSourceValue>,
    pub(crate) artboard_nested_child_context_updates_scratch: Vec<RuntimeNestedChildContextUpdate>,
    /// C++ nested artboards retain authored view-model instances by pointer,
    /// so clean frames do not reconcile detached copies. Rust only needs the
    /// full ordered reconciliation after a source value or context changes.
    pub(crate) stateful_nested_view_model_contexts_dirty: bool,
    pub(crate) artboard_data_bind_dirty_epoch: u64,
    pub(crate) artboard_data_bind_processed_epoch: u64,
    pub(crate) image_asset_overrides: BTreeMap<usize, Option<u32>>,
    text_style_font_overrides: BTreeMap<usize, RuntimeFontAssetValue>,
    has_legacy_image_layout_scales: Cell<bool>,
    legacy_image_layout_scales: RefCell<BTreeMap<usize, RuntimeLegacyImageLayoutScaleState>>,
    external_font_assets: Arc<BTreeMap<u32, Arc<[u8]>>>,
    /// C++ File/ImageAsset ownership projected into the runtime occurrence
    /// tree. Every clone retains the same file-owned owner list; Images borrow
    /// RenderImage from it and never from a facade scene cache.
    pub(crate) runtime_image_assets: RefCell<Option<Arc<crate::draw::RuntimeImageAssetOwners>>>,
    /// C++ `ArtboardInstance` owns the concrete renderer-facing members of
    /// every object in its cloned graph. Rust attaches the backend late, but
    /// the resulting resources still follow this exact occurrence through
    /// draw, clone, and drop; the host facade owns no parallel scene cache.
    pub(crate) render_resources: RefCell<crate::draw::RuntimeOccurrenceRenderResources>,
    /// Query-only retained geometry follows the Artboard occurrence, matching
    /// C++ Shape/PathComposer bounds and hit-test members. Hosts do not own or
    /// synchronize a parallel geometry scene cache.
    pub(crate) geometry_state: RefCell<crate::draw::RuntimeGeometryState>,
    pub(crate) dirt_depth: usize,
    pub(crate) cache_epoch: u64,
    pub(crate) prepared_epoch: u64,
    pub(crate) path_epoch: u64,
    pub(crate) layout_epoch: u64,
    text_affecting_locals: Vec<bool>,
    // C++ SolidColor mutates its attached RenderPaint when its property dirt
    // is applied. Renderer resources live outside the Rust instance, so retain
    // the equivalent per-mutator revision for a cheap draw-time handoff.
    solid_color_paint_revisions: Vec<u64>,
    /// C++ `Artboard::m_Drawables`/`m_FirstDrawable`: clone-owned drawable
    /// objects linked in live draw order. Import graph order only seeds it.
    pub(crate) runtime_drawables: RuntimeDrawableList,
    /// C++ `Shape::{m_PathComposer,m_Paths}` plus
    /// `ShapePaintContainer::m_ShapePaints`: clone-owned ordered memberships.
    pub(crate) runtime_shapes: RuntimeShapeList,
    /// C++ `ClippingShape::{m_Shapes,m_path,m_clipPath}`: clone-owned source
    /// membership and retained CPU/backend clip path, rebuilt only by the
    /// ClippingShape dependency node.
    pub(crate) runtime_clipping_shapes: RuntimeClippingShapeList,
    /// Clone-owned C++ `Mesh` objects and NSlicer-owned `SliceMesh` objects.
    /// Backend buffers are members of these occurrences, not the facade paint
    /// cache; `RuntimeMeshList::clone` implements C++ Mesh/NSlicer clone rules.
    pub(crate) runtime_meshes: crate::draw::RuntimeMeshList,
    pub(crate) did_change: Cell<bool>,
    pub(crate) layout_constraint_bounds_enabled: bool,
    pub(crate) layout_constraint_bounds: Option<Arc<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

impl Clone for ArtboardInstance {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            instance_identity: self.instance_identity.clone(),
            width: self.width,
            height: self.height,
            origin_x: self.origin_x,
            origin_y: self.origin_y,
            clip: self.clip,
            frame_origin: self.frame_origin.clone(),
            frame_id: self.frame_id.clone(),
            slots: self.slots.clone(),
            objects: self.objects.clone(),
            joysticks: self.joysticks.clone(),
            advancing_components: self.advancing_components.clone(),
            resetting_components: self.resetting_components.clone(),
            component_lists: self.component_lists.clone(),
            joysticks_apply_before_update: self.joysticks_apply_before_update,
            linear_animations: self.linear_animations.clone(),
            empty_linear_animation: self.empty_linear_animation.clone(),
            state_machines: self.state_machines.clone(),
            script_instances_by_global: self.script_instances_by_global.clone(),
            script_attachment_generation: self.script_attachment_generation,
            scripted_data_converter_instances_by_global: self
                .scripted_data_converter_instances_by_global
                .clone(),
            has_scripted_drawables: self.has_scripted_drawables,
            nested_script_owned_contexts: self.nested_script_owned_contexts.clone(),
            script_update_error: None,
            external_focus_domain: None,
            nested_artboards: self.nested_artboards.clone(),
            nested_artboard_locals: self.nested_artboard_locals.clone(),
            newly_uncollapsed_nested_artboards: self.newly_uncollapsed_nested_artboards.clone(),
            graph_global_id: self.graph_global_id,
            build_context: self.build_context.clone(),
            nested_context_source_tree_cache: self.nested_context_source_tree_cache.clone(),
            nested_layout_bounds: self.nested_layout_bounds.clone(),
            artboard_data_bind_values: self.artboard_data_bind_values.clone(),
            artboard_formula_random_source: self.artboard_formula_random_source.clone(),
            artboard_owned_view_model_context: self.artboard_owned_view_model_context.clone(),
            artboard_owned_data_context: self.artboard_owned_data_context.clone(),
            artboard_owned_view_model_handle: self.artboard_owned_view_model_handle.clone(),
            artboard_authored_data_bind_states: self.artboard_authored_data_bind_states.clone(),
            artboard_owned_view_model_rebind_sink: self
                .artboard_owned_view_model_rebind_sink
                .clone(),
            artboard_property_bindings: self.artboard_property_bindings.clone(),
            artboard_image_asset_bindings: self.artboard_image_asset_bindings.clone(),
            artboard_data_bind_target_queues: self.artboard_data_bind_target_queues.clone(),
            artboard_data_bind_source_queues: self.artboard_data_bind_source_queues.clone(),
            artboard_retained_subordinate_converter_operands: self
                .artboard_retained_subordinate_converter_operands
                .clone(),
            artboard_custom_property_bindings: self.artboard_custom_property_bindings.clone(),
            artboard_layout_computed_bindings: self.artboard_layout_computed_bindings.clone(),
            artboard_numeric_source_bindings: self.artboard_numeric_source_bindings.clone(),
            artboard_formula_token_bindings: self.artboard_formula_token_bindings.clone(),
            artboard_converter_property_bindings: self.artboard_converter_property_bindings.clone(),
            artboard_solo_bindings: self.artboard_solo_bindings.clone(),
            artboard_solo_source_bindings: self.artboard_solo_source_bindings.clone(),
            artboard_nested_host_bindings: self.artboard_nested_host_bindings.clone(),
            artboard_list_bindings: self.artboard_list_bindings.clone(),
            artboard_text_list_bindings: self.artboard_text_list_bindings.clone(),
            artboard_context_source_values_scratch: self
                .artboard_context_source_values_scratch
                .clone(),
            artboard_nested_child_context_updates_scratch: self
                .artboard_nested_child_context_updates_scratch
                .clone(),
            stateful_nested_view_model_contexts_dirty: self
                .stateful_nested_view_model_contexts_dirty,
            artboard_data_bind_dirty_epoch: self.artboard_data_bind_dirty_epoch,
            artboard_data_bind_processed_epoch: self.artboard_data_bind_processed_epoch,
            image_asset_overrides: self.image_asset_overrides.clone(),
            text_style_font_overrides: self.text_style_font_overrides.clone(),
            has_legacy_image_layout_scales: self.has_legacy_image_layout_scales.clone(),
            legacy_image_layout_scales: self.legacy_image_layout_scales.clone(),
            external_font_assets: self.external_font_assets.clone(),
            runtime_image_assets: self.runtime_image_assets.clone(),
            render_resources: self.render_resources.clone(),
            geometry_state: self.geometry_state.clone(),
            dirt_depth: 0,
            cache_epoch: self.cache_epoch,
            prepared_epoch: self.prepared_epoch,
            // Core clone starts the new occurrence dirty and rebuilds derived
            // Path state from clone-owned Components; it does not inherit an
            // Artboard-side invalidation generation from the source
            // (`artboard.hpp:548-601`; `artboard.cpp:1038-1057`).
            path_epoch: 1,
            layout_epoch: self.layout_epoch,
            text_affecting_locals: self.text_affecting_locals.clone(),
            solid_color_paint_revisions: self.solid_color_paint_revisions.clone(),
            runtime_drawables: self.runtime_drawables.clone(),
            runtime_shapes: self.runtime_shapes.clone(),
            runtime_clipping_shapes: self.runtime_clipping_shapes.clone(),
            runtime_meshes: self.runtime_meshes.clone(),
            did_change: self.did_change.clone(),
            layout_constraint_bounds_enabled: self.layout_constraint_bounds_enabled,
            layout_constraint_bounds: self.layout_constraint_bounds.clone(),
        };

        // Core clones generated fields into fresh Components, then reruns the
        // same dirty/clean/dependency phases against clone-owned objects.
        // Never copy the source occurrence's runtime pointer graph
        // (`artboard.hpp:548-601`; `artboard.cpp:1038-1057`).
        if let Some((file, graph)) = cloned.build_context.as_ref().and_then(|context| {
            let graph_index = context
                .artboard_index_by_global
                .get(usize::try_from(cloned.graph_global_id).ok()?)
                .copied()
                .flatten()?;
            Some((
                Arc::clone(&context.file),
                context.artboards.get(graph_index)?.clone(),
            ))
        }) {
            Self::build_component_occurrence_relations(&mut cloned.objects, &graph)
                .expect("a validated source occurrence must rebuild the same dependency graph");
            retain_runtime_layout_component_styles(&file, &cloned.slots, &mut cloned.objects);
            retain_runtime_solos(&file, &graph, &mut cloned.objects);
            retain_runtime_scroll_constraints(&file, &graph, &mut cloned.objects);
            retain_runtime_text_input_scroll_constraints(&mut cloned.objects);
            retain_runtime_drawable_component_topology(&graph, &mut cloned.objects);
            (
                cloned.advancing_components,
                cloned.resetting_components,
                cloned.component_lists,
            ) = Self::build_component_interface_schedules(&cloned.objects, &graph);
            cloned
                .runtime_shapes
                .rebuild_component_memberships(&cloned.objects);
            cloned.initialize_path_target_flags(&graph);
            cloned.initialize_component_data_bind_collapsables(&file, &graph);
        }

        // Generated C++ clones start with `ComponentDirt::Filthy` and clear
        // custom DataBind flags. Re-run authored Solo/Layout collapse only
        // after the clone-owned Component ↔ DataBind links are rebuilt.
        cloned.rederive_initial_component_collapse();
        cloned
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEventPropertyValue {
    Number(f32),
    Bool(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u64),
    Trigger(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeEventProperty {
    pub name: Option<String>,
    pub value: RuntimeEventPropertyValue,
}

#[derive(Debug)]
pub(crate) struct RuntimeNestedArtboardInstance {
    // Rust drops fields in declaration order. C++ releases nested animations
    // (including StateMachineInstances that can reference m_Instance) before
    // destroying m_Instance (`nested_artboard.cpp:48-64`).
    pub(crate) animations: Vec<RuntimeNestedAnimationInstance>,
    pub(crate) child: Box<ArtboardInstance>,
    pub(crate) render_cache_revision: u64,
    /// C++ child objects own their backend members. This sidecar follows the
    /// mounted occurrence through replacement/drop and is rebuilt on clone.
    pub(crate) render_resources: RefCell<crate::draw::RuntimeOccurrenceRenderResources>,
    /// C++ configures an intermediate shader on the one mounted child before
    /// `NestedArtboardLayout::takeLayoutData()` permanently transfers layout
    /// ownership. Retain only that narrow initial paint state until the paint
    /// cache consumes it; animation, scripts, view models, and geometry remain
    /// exclusively owned by `child`.
    pub(crate) initial_layout_paint_frame: RefCell<Option<RuntimeInitialNestedLayoutPaintFrame>>,
    pub(crate) layout_data_transferred: bool,
    /// Parent solve that last refreshed the constraint space transferred to
    /// this mounted child. Child-local layout writes must not refresh that
    /// space during the same transfer; only a new parent solve (or assigned
    /// bounds) corresponds to Yoga's `hasNewLayout` lifecycle.
    layout_data_transfer_key: Option<RuntimeNestedLayoutDataTransferKey>,
    pub(crate) data_bind_path_ids: Option<Vec<u32>>,
    pub(crate) data_bind_path_is_relative: bool,
    pub(crate) stateful_view_model_instance_local: Option<usize>,
    pub(crate) stateful_view_model_instance_locals_by_id: BTreeMap<u32, usize>,
    pub(crate) stateful_view_model_context: Option<RuntimeOwnedViewModelInstance>,
    pub(crate) stateful_global_view_model_contexts: BTreeMap<usize, RuntimeOwnedViewModelInstance>,
    pub(crate) data_bind_property_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_image_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_context_source_locals_by_path: BTreeMap<Vec<u32>, usize>,
    is_paused: bool,
    speed: f32,
    quantize: f32,
    cumulated_seconds: f32,
}

impl Clone for RuntimeNestedArtboardInstance {
    fn clone(&self) -> Self {
        // A normal artboard clone is a new occurrence. C++ gives that mounted
        // child a fresh `takeLayoutData()` lifecycle, so it must produce its
        // own one-time initial paint frame rather than inherit a consumed (or
        // pending) frame from the source occurrence.
        let mut child = self.child.as_ref().clone();
        child.reset_layout_constraint_bounds_for_new_occurrence();
        let animations = self
            .animations
            .iter()
            .map(|animation| match animation {
                RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence.cold_clone(&mut child))
                }
                // The separate NestedSimpleAnimation and NestedRemap clone
                // families remain outside FL-C3. Preserve their established
                // behavior here; only the touched NestedStateMachine owner is
                // reconstructed cold in this slice.
                animation => animation.clone(),
            })
            .collect();
        Self {
            child: Box::new(child),
            render_cache_revision: self.render_cache_revision,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: self.data_bind_path_ids.clone(),
            data_bind_path_is_relative: self.data_bind_path_is_relative,
            stateful_view_model_instance_local: self.stateful_view_model_instance_local,
            stateful_view_model_instance_locals_by_id: self
                .stateful_view_model_instance_locals_by_id
                .clone(),
            stateful_view_model_context: self.stateful_view_model_context.clone(),
            stateful_global_view_model_contexts: self.stateful_global_view_model_contexts.clone(),
            data_bind_property_source_locals: self.data_bind_property_source_locals.clone(),
            data_bind_image_source_locals: self.data_bind_image_source_locals.clone(),
            data_bind_context_source_locals_by_path: self
                .data_bind_context_source_locals_by_path
                .clone(),
            animations,
            is_paused: self.is_paused,
            speed: self.speed,
            quantize: self.quantize,
            cumulated_seconds: self.cumulated_seconds,
        }
    }
}

/// Mounted nested-artboard occurrences, retained contiguously like C++
/// `Artboard::m_NestedArtboards` while keeping local-id lookup constant-time.
///
/// Local ids index the small side table only; iteration walks the compact,
/// sorted entries and never scans gaps in the artboard's object-id space.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeNestedArtboards {
    entries: Vec<(usize, RuntimeNestedArtboardInstance)>,
    entry_by_local: Vec<Option<usize>>,
    state_machine_owner_by_local: Vec<Option<(usize, usize)>>,
}

impl RuntimeNestedArtboards {
    pub(crate) fn get(&self, local_id: &usize) -> Option<&RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get(*local_id).copied().flatten()?;
        self.entries.get(entry).map(|(_, nested)| nested)
    }

    pub(crate) fn get_mut(
        &mut self,
        local_id: &usize,
    ) -> Option<&mut RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get(*local_id).copied().flatten()?;
        self.entries.get_mut(entry).map(|(_, nested)| nested)
    }

    fn contains_key(&self, local_id: &usize) -> bool {
        self.entry_by_local
            .get(*local_id)
            .is_some_and(Option::is_some)
    }

    fn insert(
        &mut self,
        local_id: usize,
        nested: RuntimeNestedArtboardInstance,
    ) -> Option<RuntimeNestedArtboardInstance> {
        if self.entry_by_local.len() <= local_id {
            self.entry_by_local.resize(local_id.saturating_add(1), None);
        }
        if let Some(entry) = self.entry_by_local[local_id] {
            let previous = std::mem::replace(&mut self.entries[entry].1, nested);
            self.rebuild_state_machine_index();
            return Some(previous);
        }

        let entry = self
            .entries
            .binary_search_by_key(&local_id, |(candidate, _)| *candidate)
            .unwrap_or_else(|entry| entry);
        self.entries.insert(entry, (local_id, nested));
        for (entry, (local_id, _)) in self.entries.iter().enumerate().skip(entry) {
            self.entry_by_local[*local_id] = Some(entry);
        }
        self.rebuild_state_machine_index();
        None
    }

    fn remove(&mut self, local_id: &usize) -> Option<RuntimeNestedArtboardInstance> {
        let entry = self.entry_by_local.get_mut(*local_id)?.take()?;
        let (_, nested) = self.entries.remove(entry);
        for (entry, (local_id, _)) in self.entries.iter().enumerate().skip(entry) {
            self.entry_by_local[*local_id] = Some(entry);
        }
        self.rebuild_state_machine_index();
        Some(nested)
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &usize> {
        self.entries.iter().map(|(local_id, _)| local_id)
    }

    fn iter(&self) -> impl Iterator<Item = (&usize, &RuntimeNestedArtboardInstance)> {
        self.entries
            .iter()
            .map(|(local_id, nested)| (local_id, nested))
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &RuntimeNestedArtboardInstance> {
        self.entries.iter().map(|(_, nested)| nested)
    }

    pub(crate) fn values_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RuntimeNestedArtboardInstance> {
        self.entries.iter_mut().map(|(_, nested)| nested)
    }

    pub(crate) fn state_machine(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeNestedStateMachineInstance> {
        let (host_entry, animation_index) = self
            .state_machine_owner_by_local
            .get(local_id)
            .copied()
            .flatten()?;
        match self
            .entries
            .get(host_entry)?
            .1
            .animations
            .get(animation_index)?
        {
            RuntimeNestedAnimationInstance::StateMachine(occurrence)
                if occurrence.local_id() == local_id =>
            {
                Some(occurrence)
            }
            _ => None,
        }
    }

    pub(crate) fn state_machine_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut RuntimeNestedStateMachineInstance> {
        let (host_entry, animation_index) = self
            .state_machine_owner_by_local
            .get(local_id)
            .copied()
            .flatten()?;
        match self
            .entries
            .get_mut(host_entry)?
            .1
            .animations
            .get_mut(animation_index)?
        {
            RuntimeNestedAnimationInstance::StateMachine(occurrence)
                if occurrence.local_id() == local_id =>
            {
                Some(occurrence)
            }
            _ => None,
        }
    }

    fn rebuild_state_machine_index(&mut self) {
        self.state_machine_owner_by_local.clear();
        for (host_entry, (_, nested)) in self.entries.iter().enumerate() {
            for (animation_index, animation) in nested.animations.iter().enumerate() {
                let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                    continue;
                };
                let local_id = occurrence.local_id();
                if self.state_machine_owner_by_local.len() <= local_id {
                    self.state_machine_owner_by_local
                        .resize(local_id.saturating_add(1), None);
                }
                self.state_machine_owner_by_local[local_id] = Some((host_entry, animation_index));
            }
        }
    }
}

impl std::ops::Index<&usize> for RuntimeNestedArtboards {
    type Output = RuntimeNestedArtboardInstance;

    fn index(&self, local_id: &usize) -> &Self::Output {
        self.get(local_id)
            .unwrap_or_else(|| panic!("no nested artboard mounted at local id {local_id}"))
    }
}

/// Ported from C++ `src/artboard_component_list.cpp`: one persistent child
/// artboard and its selected state machines for an owned view-model list item.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeComponentListItemInstance {
    // C++ erases row state machines before row ArtboardInstances so listener
    // groups cannot observe destroyed FocusData
    // (`artboard_component_list.cpp:1582-1586`).
    pub(crate) state_machines: Vec<StateMachineInstance>,
    pub(crate) child: Box<ArtboardInstance>,
    /// Backend members for this one mounted row occurrence.
    pub(crate) render_resources: RefCell<crate::draw::RuntimeOccurrenceRenderResources>,
    pub(crate) context: RuntimeOwnedViewModelHandle,
    /// Pushed C++ `ViewModelInstance::m_dependents` relink channel. Scalar
    /// cells notify the mounted child's binds directly; ViewModel-reference
    /// replacement dirties this sink (`viewmodel_instance.cpp:118-188,346-415`).
    pub(crate) context_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink,
    /// C++ `ArtboardListDrawIndexDependent`: scalar writes invalidate the
    /// retained paint-order indices without polling the ViewModel graph.
    pub(crate) draw_index_sink: Option<crate::view_model_cell::RuntimeCellDirtSink>,
    pub(crate) occurrence_identity: u64,
    pub(crate) logical_index: usize,
    /// Last parent-assigned layout size observed while preparing this mounted
    /// occurrence. C++ writes `Artboard::layoutBounds()` back into the full
    /// logical `m_artboardSizes` topology after layout; the next list sync
    /// consumes this value before selecting its virtual window.
    pub(crate) settled_layout_size: Cell<Option<(f32, f32)>>,
    pub(crate) transform: Mat2D,
    pub(crate) render_cache_revision: u64,
}

impl RuntimeComponentListItemInstance {
    fn context_is_current(&self, context: &RuntimeOwnedViewModelHandle) -> bool {
        self.context.ptr_eq(context)
            && !self
                .context_rebind_sink
                .peek_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
    }

    fn consume_context_rebind_dirt(&self) {
        self.context_rebind_sink.take_dirt();
    }
}

fn component_list_draw_index_sink(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelHandle,
) -> Option<crate::view_model_cell::RuntimeCellDirtSink> {
    let property_name = file
        .view_model_property_for_symbol(context.borrow().view_model_index(), 16)?
        .string_property("name")?;
    let cell = context
        .borrow()
        .number_cell_by_property_name(property_name)?;
    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
    cell.add_dependent(&sink);
    Some(sink)
}

/// One exact descent edge from a retained root artboard to a nested occurrence.
///
/// This is a runtime-internal address primitive. Higher layers should wrap it
/// in their own epoch-fenced semantic cursor rather than exposing local ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeArtboardOccurrenceSegment {
    NestedArtboard {
        host_local_id: usize,
    },
    ComponentListItem {
        host_local_id: usize,
        item_index: usize,
        /// Stable identity of the mounted list occurrence captured with the
        /// hit. Indices are only positions in the current mounted window and
        /// may be reused after a list replacement or reorder.
        occurrence_identity: u64,
    },
}

/// Full logical topology retained independently from the mounted artboards,
/// matching C++ `m_listItems` and `m_artboardSizes`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeComponentListLogicalItem {
    pub(crate) occurrence_identity: u64,
    pub(crate) context: RuntimeOwnedViewModelHandle,
    pub(crate) size: (f32, f32),
    pub(crate) mapped_artboard_global: Option<u32>,
}

#[cfg(test)]
fn component_list_contexts_retain_same_handles(
    existing: &[RuntimeComponentListItemInstance],
    incoming: &[RuntimeOwnedViewModelHandle],
) -> bool {
    existing.len() == incoming.len()
        && existing
            .iter()
            .zip(incoming)
            .all(|(item, context)| item.context.ptr_eq(context))
}

#[derive(Debug, Clone)]
struct RuntimeArtboardBuildContext {
    file: Arc<RuntimeFile>,
    file_view_model_instances: RuntimeFileViewModelInstanceCatalog,
    state_machine_actions: RuntimeFileStateMachineActionCatalog,
    artboards: Arc<Vec<ArtboardGraph>>,
    artboard_index_by_global: Arc<Vec<Option<usize>>>,
    nested_structure_epoch: Arc<AtomicU64>,
    paint_preparation_epoch: Arc<AtomicU64>,
    external_font_assets: Arc<BTreeMap<u32, Arc<[u8]>>>,
}

fn build_artboard_index_by_global(artboards: &[ArtboardGraph]) -> Vec<Option<usize>> {
    let slot_count = artboards
        .iter()
        .filter_map(|graph| usize::try_from(graph.global_id).ok())
        .max()
        .map_or(0, |maximum| maximum.saturating_add(1));
    let mut indices = vec![None; slot_count];
    for (index, graph) in artboards.iter().enumerate() {
        if let Ok(global_id) = usize::try_from(graph.global_id)
            && let Some(slot) = indices.get_mut(global_id)
        {
            *slot = Some(index);
        }
    }
    indices
}

fn build_text_affecting_locals(slots: &[InstanceSlot], objects: &InstanceObjectArena) -> Vec<bool> {
    let mut result = vec![false; slots.len()];
    let Some(parent_key) = property_key_for_name("Component", "parentId") else {
        return result;
    };
    for slot in slots {
        let mut current_local = slot.local_id;
        let mut remaining = slots.len().saturating_add(1);
        while remaining != 0 {
            remaining -= 1;
            if matches!(
                slots.get(current_local).and_then(|slot| slot.type_name),
                Some("Text" | "TextInput")
            ) {
                if let Some(affects_text) = result.get_mut(slot.local_id) {
                    *affects_text = true;
                }
                break;
            }
            let Some(parent_local) = objects
                .uint_property(current_local, parent_key)
                .and_then(|parent| usize::try_from(parent).ok())
            else {
                break;
            };
            if parent_local == current_local || parent_local >= slots.len() {
                break;
            }
            current_local = parent_local;
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeNestedLayoutBoundsCacheKey {
    graph_global_id: u32,
    layout_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeNestedLayoutDataTransferKey {
    parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    assigned_bounds: RuntimeLayoutBounds,
    child_layout_epoch: u64,
}

#[derive(Debug, Clone)]
struct RuntimeNestedLayoutBoundsFrame {
    key: RuntimeNestedLayoutBoundsCacheKey,
    bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeNestedAnimationInstance {
    Simple {
        local_id: usize,
        animation: LinearAnimationInstance,
        is_playing: bool,
        speed: f32,
        mix: f32,
    },
    Remap {
        local_id: usize,
        animation: LinearAnimationInstance,
        mix: f32,
    },
    StateMachine(RuntimeNestedStateMachineInstance),
}

fn state_machine_requires_outer_update_probe(instance: &StateMachineInstance) -> bool {
    instance.post_update_probe_pending() || instance.requires_post_update_state_probe()
}

impl ArtboardInstance {
    fn build_component_interface_schedules(
        objects: &InstanceObjectArena,
        graph: &ArtboardGraph,
    ) -> (
        Vec<RuntimeAdvancingComponent>,
        Vec<RuntimeResettingComponent>,
        Vec<ComponentHandle>,
    ) {
        // The graph rows are a construction-only projection of the exact
        // `m_Objects` visitation and C++ family switches. Advancing accepts
        // Core (not Component), so ScriptedDataConverter deliberately carries
        // no ComponentHandle (`advancing_component.cpp:17-44`;
        // `artboard.cpp:330-395`).
        let advancing = graph
            .advancing_components
            .iter()
            .filter_map(|entry| {
                Some(RuntimeAdvancingComponent {
                    local_id: entry.local_id,
                    object: objects.object_handle(entry.local_id)?,
                    component: objects.component_handle(entry.local_id),
                    kind: entry.kind,
                })
            })
            .collect();
        let resetting = graph
            .resetting_components
            .iter()
            .filter_map(|entry| {
                Some(RuntimeResettingComponent {
                    local_id: entry.local_id,
                    component: objects.component_handle(entry.local_id)?,
                    kind: entry.kind,
                })
            })
            .collect();
        let component_lists = objects
            .component_handles()
            .iter()
            .copied()
            .filter(|handle| {
                objects
                    .component(*handle)
                    .is_some_and(|component| component.concrete.constrainable_list.is_some())
            })
            .collect();
        (advancing, resetting, component_lists)
    }

    fn reset_layout_constraint_bounds_for_new_occurrence(&mut self) {
        self.layout_constraint_bounds_enabled = false;
        self.layout_constraint_bounds = None;
    }

    fn added_to_host(&self) {
        if let Some(layout) = self
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.added_to_host();
        }
    }

    /// Validate bytes against both font backends used by runtime text.
    #[must_use]
    pub fn external_font_bytes_are_parseable(bytes: &[u8]) -> bool {
        crate::text::embedded_font_is_parseable(bytes)
    }

    /// Clone used only by draw/layout evaluation of the same concrete
    /// occurrence. Unlike the public occurrence clone, this explicitly keeps
    /// the VM table handles needed to render scripted drawables. Lifecycle
    /// queues remain fresh so the transient view cannot advance the scripts.
    pub(crate) fn clone_for_transient_layout(&self) -> Self {
        let mut cloned = self.clone();
        cloned.restore_transient_component_lists_from(self);
        cloned.restore_transient_occurrence_identities_from(self);
        cloned.restore_transient_script_handles_from(self);
        cloned.restore_transient_layout_transfer_state_from(self);
        cloned
    }

    fn restore_transient_component_lists_from(&mut self, source: &Self) {
        let locals = source.component_list_locals();
        for local_id in locals {
            let Some(source_state) = source.component_list_state(local_id).cloned() else {
                continue;
            };
            if let Some(cloned_state) = self.component_list_state_mut(local_id) {
                *cloned_state = source_state;
            }
        }
    }

    fn restore_transient_layout_transfer_state_from(&mut self, source: &Self) {
        // Transient draw/layout clones view the same mounted occurrence. Copy
        // whether layout ownership already transferred, but never copy its
        // pending one-shot paint frame: only the authoritative instance may
        // consume that renderer event.
        self.layout_constraint_bounds_enabled = source.layout_constraint_bounds_enabled;
        self.layout_constraint_bounds = source.layout_constraint_bounds.clone();
        for (local_id, source_nested) in source.nested_artboards.iter() {
            if let Some(cloned_nested) = self.nested_artboards.get_mut(local_id) {
                // A transient layout clone is a non-mutating view of this
                // exact mounted occurrence, not a generated public clone.
                // Restore its live nested-animation snapshot after the public
                // clone path deliberately reconstructed NestedStateMachine
                // occurrences cold.
                cloned_nested.animations = source_nested.animations.clone();
                cloned_nested.layout_data_transferred = source_nested.layout_data_transferred;
                cloned_nested.layout_data_transfer_key = source_nested.layout_data_transfer_key;
                cloned_nested.initial_layout_paint_frame.replace(None);
                cloned_nested
                    .child
                    .restore_transient_layout_transfer_state_from(&source_nested.child);
            }
        }
        for local_id in source.component_list_locals() {
            let Some(source_items) = source.component_list_items(local_id) else {
                continue;
            };
            let Some(cloned_items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for (cloned_item, source_item) in cloned_items.iter_mut().zip(source_items) {
                cloned_item
                    .child
                    .restore_transient_layout_transfer_state_from(&source_item.child);
            }
        }
    }

    fn restore_transient_occurrence_identities_from(&mut self, source: &Self) {
        // A transient layout clone is another view of the same mounted
        // occurrence, not a newly-instanced artboard. C++ applies layout to
        // that occurrence in place, so occurrence-keyed render state (notably
        // TextStylePaint's opacity paint pool) survives across frames.
        self.instance_identity = RuntimeArtboardInstanceIdentity(source.instance_identity.0);
        for (local_id, source_nested) in source.nested_artboards.iter() {
            if let Some(cloned_nested) = self.nested_artboards.get_mut(local_id) {
                cloned_nested
                    .child
                    .restore_transient_occurrence_identities_from(&source_nested.child);
            }
        }
        for local_id in source.component_list_locals() {
            let Some(source_items) = source.component_list_items(local_id) else {
                continue;
            };
            let Some(cloned_items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for (cloned_item, source_item) in cloned_items.iter_mut().zip(source_items) {
                cloned_item
                    .child
                    .restore_transient_occurrence_identities_from(&source_item.child);
            }
        }
    }

    fn restore_transient_script_handles_from(&mut self, source: &Self) {
        self.script_instances_by_global.0 = source.script_instances_by_global.0.clone();
        self.scripted_data_converter_instances_by_global.0 =
            source.scripted_data_converter_instances_by_global.0.clone();
        for (local_id, source_nested) in source.nested_artboards.iter() {
            if let Some(cloned_nested) = self.nested_artboards.get_mut(local_id) {
                cloned_nested
                    .child
                    .restore_transient_script_handles_from(&source_nested.child);
            }
        }
        for local_id in source.component_list_locals() {
            let Some(source_items) = source.component_list_items(local_id) else {
                continue;
            };
            let Some(cloned_items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for (cloned_item, source_item) in cloned_items.iter_mut().zip(source_items) {
                cloned_item
                    .child
                    .restore_transient_script_handles_from(&source_item.child);
            }
        }
    }

    fn build_component_occurrence_relations(
        objects: &mut InstanceObjectArena,
        graph: &ArtboardGraph,
    ) -> Result<()> {
        objects.reset_component_relations();
        let root = objects
            .root()
            .context("artboard occurrence is missing its root Component")?;
        let parent_key = property_key_for_name("Component", "parentId")
            .context("Component.parentId is missing from the runtime schema")?;

        // C++ onAddedDirty runs in authored object order. Shape forwards that
        // phase to its embedded PathComposer immediately after its own base
        // Component, so parent/child and constraint insertion are observable.
        for component in &graph.components {
            let handle = objects
                .component_handle(component.local_id)
                .context("authored Component handle is missing")?;
            if handle != root {
                let parent_local = objects
                    .uint_property(component.local_id, parent_key)
                    .and_then(|parent| usize::try_from(parent).ok())
                    .context("Component parentId does not resolve to an object slot")?;
                let parent = objects
                    .component_handle(parent_local)
                    .context("Component parentId does not resolve to a Component occurrence")?;
                let parent_type = objects
                    .component(parent)
                    .map(|component| component.type_name)
                    .unwrap_or("<missing>");
                if !objects.is_container_component(parent) {
                    anyhow::bail!(
                        "Component {} local {} parent local {} type {} is not a ContainerComponent",
                        component.type_name,
                        component.local_id,
                        parent_local,
                        parent_type
                    );
                }
                if !objects.link_parent(handle, parent) {
                    anyhow::bail!("Component parent link could not be retained");
                }
                let is_vertex = objects
                    .component(handle)
                    .is_some_and(|component| component.concrete.vertex.is_some());
                if is_vertex
                    && let Some(skinnable) = objects
                        .component_mut(parent)
                        .and_then(|parent| parent.concrete.skinnable.as_mut())
                {
                    // Vertex::onAddedDirty registers on Path/Mesh in authored
                    // object order. Keep that exact owner list for Skin
                    // deformation; no graph vertex scan is needed at update.
                    skinnable.vertices.push(handle);
                }
            }
            if let Some(composer) = objects.path_composer_handle(component.local_id) {
                objects.link_parent(composer, root);
            }
            let constraint_state = objects
                .component(handle)
                .and_then(|component| component.concrete.constraint);
            if let Some(constraint_state) = constraint_state {
                let parent = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                    .context("Constraint is missing its parent Component")?;
                if !objects
                    .component(parent)
                    .is_some_and(|parent| parent.capabilities.transform)
                {
                    anyhow::bail!("Constraint parent is not a TransformComponent");
                }
                objects.add_constraint(parent, handle);

                // TargetedConstraint::onAddedDirty resolves and validates its
                // retained target after Constraint has registered on the
                // constrained parent (`src/constraints/targeted_constraint.cpp:
                // 23-39`). A missing target is legal only for the three
                // concrete optional-target families.
                if constraint_state.targeted {
                    let target_local = constraint_state
                        .targeted
                        .then(crate::constraints::targeted_constraint_target_id_property_key)
                        .flatten()
                        .and_then(|key| objects.uint_property(component.local_id, key))
                        .and_then(|target| usize::try_from(target).ok());
                    let target = target_local.and_then(|local| objects.component_handle(local));
                    if target_local.is_some_and(|local| objects.contains_object(local))
                        && target.is_none_or(|target| {
                            !objects
                                .component(target)
                                .is_some_and(|target| target.capabilities.transform)
                        })
                    {
                        anyhow::bail!(
                            "TargetedConstraint targetId does not resolve to a TransformComponent"
                        );
                    }
                    if constraint_state.requires_target && target.is_none() {
                        anyhow::bail!("TargetedConstraint is missing its required target");
                    }
                    objects
                        .component_mut(handle)
                        .expect("Constraint handle was validated")
                        .concrete
                        .constraint
                        .as_mut()
                        .expect("Constraint occurrence owns Constraint state")
                        .target = target;
                }
            }
        }

        // Complete the A3 concrete dirty phase in authored object order.
        // These are the retained members C++ resolves before any clean or
        // buildDependencies call; immutable graph rows are not runtime truth.
        for component in &graph.components {
            let handle = objects
                .component_handle(component.local_id)
                .context("authored Component handle is missing")?;
            match component.type_name {
                "Skin" => {
                    let parent = objects
                        .component(handle)
                        .and_then(|component| component.parent)
                        .context("Skin is missing its parent Component")?;
                    let parent_is_skinnable = objects
                        .component(parent)
                        .and_then(|parent| parent.concrete.skinnable.as_ref())
                        .is_some();
                    let world_transform = generated_mat2d(objects, component.local_id, "Skin");
                    objects
                        .component_mut(handle)
                        .expect("Skin handle was validated")
                        .concrete
                        .skin
                        .as_mut()
                        .expect("Skin occurrence owns Skin state")
                        .world_transform = world_transform;
                    if parent_is_skinnable {
                        objects
                            .component_mut(handle)
                            .expect("Skin handle was validated")
                            .concrete
                            .skin
                            .as_mut()
                            .expect("Skin occurrence owns Skin state")
                            .skinnable = Some(parent);
                        objects
                            .component_mut(parent)
                            .expect("Skinnable parent handle was validated")
                            .concrete
                            .skinnable
                            .as_mut()
                            .expect("Skin parent owns Skinnable state")
                            .skin = Some(handle);
                    }
                    // `Skin::onAddedDirty` writes m_WorldTransform before
                    // checking the parent, then returns MissingObject for a
                    // non-Skinnable parent. Artboard::canContinue treats that
                    // status as non-fatal, so malformed legacy files retain a
                    // null Skinnable rather than failing instantiation
                    // (`skin.cpp:13-39`; `artboard.cpp:204-245,264-288`).
                }
                "Tendon" => {
                    let bone_id_key = property_key_for_name("Tendon", "boneId")
                        .context("Tendon.boneId is missing from the runtime schema")?;
                    let bone_local = objects
                        .uint_property(component.local_id, bone_id_key)
                        .and_then(|bone| usize::try_from(bone).ok())
                        .context("Tendon boneId does not resolve to an object slot")?;
                    let bone = objects
                        .component_handle(bone_local)
                        .context("Tendon boneId does not resolve to a Component occurrence")?;
                    if objects
                        .component(bone)
                        .and_then(|bone| bone.concrete.bone.as_ref())
                        .is_none()
                    {
                        anyhow::bail!("Tendon boneId does not resolve to a Bone");
                    }
                    let inverse_bind =
                        generated_mat2d(objects, component.local_id, "Tendon").invert_or_identity();
                    let tendon = objects
                        .component_mut(handle)
                        .expect("Tendon handle was validated")
                        .concrete
                        .tendon
                        .as_mut()
                        .expect("Tendon occurrence owns Tendon state");
                    tendon.inverse_bind = inverse_bind;
                    tendon.bone = Some(bone);
                }
                type_name
                    if definition_by_name(type_name)
                        .is_some_and(|definition| definition.is_a("Weight")) =>
                {
                    let parent = objects
                        .component(handle)
                        .and_then(|component| component.parent)
                        .context("Weight is missing its parent Component")?;
                    if objects
                        .component(parent)
                        .and_then(|parent| parent.concrete.vertex.as_ref())
                        .is_none()
                    {
                        anyhow::bail!("Weight parent is not a Vertex");
                    }
                    objects
                        .component_mut(parent)
                        .expect("Vertex parent handle was validated")
                        .concrete
                        .vertex
                        .as_mut()
                        .expect("Weight parent owns Vertex state")
                        .weight = Some(handle);
                }
                _ => {}
            }
        }

        // Complete the A3/A5 clean phase in authored object order. Transform's
        // parent pointer is a distinct typed relationship, Bone registers
        // with a Bone parent (RootBone deliberately bypasses that rule),
        // Tendon registers on its Skin in source order, FollowPath retains its
        // target path membership, and IK builds its one FK chain.
        for component in &graph.components {
            let handle = objects
                .component_handle(component.local_id)
                .context("authored Component handle is missing")?;
            if objects
                .component(handle)
                .is_some_and(|component| component.capabilities.transform)
            {
                let parent_transform = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                    .filter(|parent| {
                        objects
                            .component(*parent)
                            .is_some_and(|parent| parent.capabilities.world_transform)
                    });
                objects
                    .component_mut(handle)
                    .expect("Transform handle was validated")
                    .parent_transform = parent_transform;
            }

            // `Path::onAddedClean` walks the live parent chain to its Shape,
            // stores that pointer, and registers itself on the Shape in
            // authored order. The embedded PathComposer is then reached from
            // that retained Shape, never rediscovered through immutable graph
            // rows (`src/shapes/path.cpp:76-96`).
            if objects
                .component(handle)
                .and_then(|component| component.concrete.path.as_ref())
                .is_some()
            {
                let mut ancestor = objects
                    .component(handle)
                    .and_then(|component| component.parent);
                let shape = loop {
                    let Some(candidate) = ancestor else {
                        anyhow::bail!("Path is missing its owning Shape");
                    };
                    if objects
                        .component(candidate)
                        .and_then(|component| component.concrete.shape.as_ref())
                        .is_some()
                    {
                        break candidate;
                    }
                    ancestor = objects
                        .component(candidate)
                        .and_then(|component| component.parent);
                };
                let shape_local = objects
                    .component_local_id(shape)
                    .context("Shape handle is missing its object identity")?;
                objects
                    .path_composer_handle(shape_local)
                    .context("Shape is missing its embedded PathComposer")?;
                objects
                    .component_mut(handle)
                    .expect("Path handle was validated")
                    .concrete
                    .path
                    .as_mut()
                    .expect("Path occurrence owns Path state")
                    .shape = Some(shape);
                let paths = &mut objects
                    .component_mut(shape)
                    .expect("Shape handle was validated")
                    .concrete
                    .shape
                    .as_mut()
                    .expect("Shape occurrence owns Shape state")
                    .paths;
                assert!(
                    !paths.contains(&handle),
                    "C++ Shape::addPath requires unique Path registration"
                );
                paths.push(handle);
            }

            let is_non_root_bone = objects
                .component(handle)
                .and_then(|component| component.concrete.bone.as_ref())
                .is_some_and(|bone| !bone.is_root);
            if is_non_root_bone {
                let parent = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                    .context("Bone is missing its parent Component")?;
                let Some(parent_bone) = objects
                    .component_mut(parent)
                    .and_then(|parent| parent.concrete.bone.as_mut())
                else {
                    anyhow::bail!("Bone parent is not a Bone");
                };
                parent_bone.child_bones.push(handle);
            } else if component.type_name == "Tendon" {
                let parent = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                    .context("Tendon is missing its parent Component")?;
                let Some(skin) = objects
                    .component_mut(parent)
                    .and_then(|parent| parent.concrete.skin.as_mut())
                else {
                    anyhow::bail!("Tendon parent is not a Skin");
                };
                skin.tendons.push(handle);
            }

            if component.type_name == "IKConstraint" {
                let tip = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                    .context("IKConstraint is missing its constrained parent")?;
                if objects
                    .component(tip)
                    .and_then(|component| component.concrete.bone.as_ref())
                    .is_none()
                {
                    anyhow::bail!("IKConstraint parent is not a Bone");
                }

                let mut reverse_chain = vec![tip];
                let mut bone = tip;
                let mut remaining = objects
                    .uint_property(
                        component.local_id,
                        crate::constraints::IK_PARENT_BONE_COUNT_PROPERTY_KEY,
                    )
                    .unwrap_or(0);
                while remaining > 0 {
                    let Some(parent) = objects
                        .component(bone)
                        .and_then(|component| component.parent)
                    else {
                        break;
                    };
                    if objects
                        .component(parent)
                        .and_then(|component| component.concrete.bone.as_ref())
                        .is_none()
                    {
                        break;
                    }
                    remaining -= 1;
                    bone = parent;
                    let peers = &mut objects
                        .component_mut(bone)
                        .expect("IK ancestor Bone was validated")
                        .concrete
                        .bone
                        .as_mut()
                        .expect("IK ancestor owns Bone state")
                        .peer_constraints;
                    assert!(
                        !peers.contains(&handle),
                        "C++ Bone::addPeerConstraint requires unique IK registration"
                    );
                    peers.push(handle);
                    reverse_chain.push(bone);
                }

                // C++ stores the FK chain root-to-tip after first collecting
                // it tip-to-root (`ik_constraint.cpp:25-50`).
                let chain = reverse_chain
                    .iter()
                    .rev()
                    .copied()
                    .enumerate()
                    .map(|(index, bone)| RuntimeIkChainLink {
                        index,
                        bone,
                        angle: 0.0,
                        transform_components: TransformComponents::default(),
                        parent_world_inverse: Mat2D::IDENTITY,
                    })
                    .collect();
                let retained = objects
                    .component_mut(handle)
                    .expect("IKConstraint handle was validated")
                    .concrete
                    .ik
                    .as_mut()
                    .expect("IKConstraint owns retained chain state");
                retained.chain = chain;
                #[cfg(test)]
                {
                    retained.chain_builds += 1;
                }

                // `IKConstraint::onAddedClean` makes every first-level
                // off-chain Transform child of each ancestor depend on the
                // constrained tip (`ik_constraint.cpp:52-72`).
                for index in 1..reverse_chain.len() {
                    let ancestor = reverse_chain[index];
                    let chain_child = reverse_chain[index - 1];
                    let children = objects
                        .component(ancestor)
                        .map(|component| component.children.clone())
                        .unwrap_or_default();
                    for child in children {
                        if child != chain_child
                            && objects
                                .component(child)
                                .is_some_and(|component| component.capabilities.transform)
                        {
                            objects.add_dependent(tip, child);
                        }
                    }
                }
            }
        }

        // TextStyle creates its optional helper inside that occurrence's
        // onAddedClean, then immediately runs the helper's dirty/clean phases
        // (`src/text/text_style.cpp:45-70`). Preserve that object-order
        // construction point rather than pre-attaching every helper as a
        // family batch.
        for component in &graph.components {
            let Some(helper) = graph
                .text_variation_helpers
                .iter()
                .find(|helper| helper.text_style_local == component.local_id)
            else {
                continue;
            };
            let handle = if objects
                .text_variation_helper_handle(helper.text_style_local)
                .is_some()
            {
                objects
                    .relink_text_variation_helper_owner(helper.text_style_local)
                    .context("TextVariationHelper cannot retain its rebuilt TextStyle parent")?
            } else {
                objects
                    .attach_text_variation_helper(
                        helper.text_style_local,
                        RuntimeComponent::embedded(
                            helper.text_style_local,
                            helper.text_style_global,
                            "TextVariationHelper",
                        ),
                    )
                    .context("TextStyle cannot own its TextVariationHelper")?
            };
            if !objects.link_parent(handle, root) {
                anyhow::bail!("TextVariationHelper parent link could not be retained");
            }
        }

        // Consume the construction-only insertion blueprint once. Every edge
        // lands on the exact authored or embedded Component occurrence.
        let dependency_handles = graph
            .dependency_nodes
            .iter()
            .map(|node| match node.kind {
                DependencyNodeKind::Component { local_id, .. } => {
                    objects.component_handle(local_id)
                }
                DependencyNodeKind::PathComposer { shape_local, .. } => {
                    objects.path_composer_handle(shape_local)
                }
                DependencyNodeKind::TextVariationHelper {
                    text_style_local, ..
                } => objects.text_variation_helper_handle(text_style_local),
            })
            .collect::<Vec<_>>();
        let mut ordered_edges =
            Vec::with_capacity(graph.dependency_node_edges_in_insertion_order.len());
        let mut scheduled_edges = BTreeSet::new();
        let mut push_edge = |edge_index: usize| {
            if scheduled_edges.insert(edge_index) {
                ordered_edges.push(edge_index);
            }
        };

        // C++ invokes each concrete buildDependencies in authored object
        // order. Shape invokes its embedded PathComposer before Super, and
        // TextStyle invokes its helper before its own parent dependency
        // (`shape.cpp:262-276`; `text_style.cpp:128-136`). The import graph
        // stores a family-complete edge set; interleave the already-closed
        // embedded owners here before consuming the remaining family edges.
        for component in &graph.components {
            let handle = objects
                .component_handle(component.local_id)
                .context("authored Component handle is missing during dependency construction")?;
            if let Some(composer_node) = graph.dependency_nodes.iter().position(|node| {
                matches!(
                    node.kind,
                    DependencyNodeKind::PathComposer { shape_local, .. }
                        if shape_local == component.local_id
                )
            }) {
                for (edge_index, edge) in graph
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .enumerate()
                {
                    if edge.dependent_node == composer_node
                        && matches!(
                            edge.kind,
                            nuxie_graph::DependencyKind::PathComposerShape
                                | nuxie_graph::DependencyKind::PathComposerPath
                        )
                    {
                        push_edge(edge_index);
                    }
                }
            }
            if let Some(helper_node) = graph.dependency_nodes.iter().position(|node| {
                matches!(
                    node.kind,
                    DependencyNodeKind::TextVariationHelper {
                        text_style_local,
                        ..
                    } if text_style_local == component.local_id
                )
            }) {
                for (edge_index, edge) in graph
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .enumerate()
                {
                    if (edge.source_node == helper_node || edge.dependent_node == helper_node)
                        && matches!(
                            edge.kind,
                            nuxie_graph::DependencyKind::TextVariationHelperArtboard
                                | nuxie_graph::DependencyKind::TextVariationHelperText
                        )
                    {
                        push_edge(edge_index);
                    }
                }
            }

            // Skin has no Super::buildDependencies. At the Skin object's
            // authored build point it walks retained Tendons, adding each
            // Bone and peer-constraint parent in Tendon order before
            // allocating its owned transform buffer (`src/bones/skin.cpp:
            // 53-77`). Derive the edges from those retained owners so clone
            // construction observes live clone-owned boneId relationships.
            if component.type_name == "Skin" {
                let skin = objects
                    .component_handle(component.local_id)
                    .context("Skin handle is missing during dependency construction")?;
                let tendons = objects
                    .component(skin)
                    .and_then(|component| component.concrete.skin.as_ref())
                    .map(|skin| skin.tendons.clone())
                    .unwrap_or_default();
                for tendon in tendons {
                    let Some(bone) = objects
                        .component(tendon)
                        .and_then(|component| component.concrete.tendon.as_ref())
                        .and_then(|tendon| tendon.bone)
                    else {
                        continue;
                    };
                    objects.add_dependent(bone, skin);
                    let peer_constraints = objects
                        .component(bone)
                        .and_then(|component| component.concrete.bone.as_ref())
                        .map(|bone| bone.peer_constraints.clone())
                        .unwrap_or_default();
                    for constraint in peer_constraints {
                        if let Some(parent) = objects
                            .component(constraint)
                            .and_then(|component| component.parent)
                        {
                            objects.add_dependent(parent, skin);
                        }
                    }
                }
            }

            // Ordinary TargetedConstraint::buildDependencies makes the
            // constrained parent update after the retained target. IK calls
            // this base before adding its own target->constraint edge.
            if let Some(constraint) = objects
                .component(handle)
                .and_then(|component| component.concrete.constraint)
                && constraint.targeted
                && constraint.kind.uses_targeted_base_dependencies()
                && let (Some(target), Some(parent)) = (
                    constraint.target,
                    objects
                        .component(handle)
                        .and_then(|component| component.parent),
                )
            {
                objects.add_dependent(target, parent);
            }

            if objects
                .component(handle)
                .and_then(|component| component.concrete.follow_path.as_ref())
                .is_some()
            {
                let target = objects
                    .component(handle)
                    .and_then(|component| component.concrete.constraint)
                    .and_then(|constraint| constraint.target)
                    .context("FollowPathConstraint is missing its retained target")?;
                let source = objects
                    .component(target)
                    .and_then(|component| component.concrete.shape.as_ref())
                    .and_then(|_| {
                        objects
                            .component_local_id(target)
                            .and_then(|local| objects.path_composer_handle(local))
                    })
                    .or_else(|| {
                        objects
                            .component(target)
                            .and_then(|component| component.concrete.path.as_ref())
                            .and_then(|path| path.shape)
                            .and_then(|shape| objects.component_local_id(shape))
                            .and_then(|shape_local| objects.path_composer_handle(shape_local))
                            .or_else(|| {
                                objects
                                    .component(target)
                                    .and_then(|component| component.concrete.path.as_ref())
                                    .map(|_| target)
                            })
                    });
                if let Some(source) = source {
                    objects.add_dependent(source, handle);
                }
                if let Some(parent) = objects
                    .component(handle)
                    .and_then(|component| component.parent)
                {
                    objects.add_dependent(handle, parent);
                }

                // ListFollowPath calls its FollowPath Super first and then
                // registers itself once on the exact ConstrainableList owner
                // (`list_follow_path_constraint.cpp:57-66`).
                if objects
                    .component(handle)
                    .and_then(|component| component.concrete.constraint)
                    .is_some_and(|state| {
                        state.kind == crate::components::RuntimeConstraintKind::ListFollowPath
                    })
                    && let Some(list) = objects
                        .component(handle)
                        .and_then(|component| component.parent)
                    && let Some(constraints) = objects
                        .component_mut(list)
                        .and_then(|component| component.concrete.constrainable_list.as_mut())
                        .map(|list| &mut list.constraints)
                {
                    assert!(
                        !constraints.contains(&handle),
                        "C++ ConstrainableList requires unique constraint registration"
                    );
                    constraints.push(handle);
                }
            }

            if component.type_name == "IKConstraint"
                && let Some(target) = objects
                    .component(handle)
                    .and_then(|component| component.concrete.constraint)
                    .and_then(|constraint| constraint.target)
            {
                // `IKConstraint::buildDependencies` runs TargetedConstraint
                // first (above), then makes the constraint itself depend on
                // the same target (`ik_constraint.cpp:8-19`).
                objects.add_dependent(target, handle);
            }

            // Mesh::buildDependencies adds Skin before its explicit parent
            // edge (`src/shapes/mesh.cpp:152-160`).
            if component.type_name == "Mesh" {
                let mesh = objects
                    .component_handle(component.local_id)
                    .context("Mesh handle is missing during dependency construction")?;
                if let Some(skin) = objects
                    .component(mesh)
                    .and_then(|component| component.concrete.skinnable.as_ref())
                    .and_then(|skinnable| skinnable.skin)
                {
                    objects.add_dependent(skin, mesh);
                }
            }
            for (edge_index, edge) in graph
                .dependency_node_edges_in_insertion_order
                .iter()
                .enumerate()
            {
                if edge.kind != nuxie_graph::DependencyKind::ParentChild {
                    continue;
                }
                let Some(DependencyNode {
                    kind: DependencyNodeKind::Component { local_id, .. },
                    ..
                }) = graph.dependency_nodes.get(edge.dependent_node)
                else {
                    continue;
                };
                if *local_id == component.local_id {
                    push_edge(edge_index);
                }
            }

            // PointsPath::buildDependencies calls Super first and then adds
            // its retained Skin dependency (`src/shapes/points_path.cpp:
            // 12-19`), the opposite relative position from Mesh.
            if component.type_name == "PointsPath" {
                let path = objects
                    .component_handle(component.local_id)
                    .context("PointsPath handle is missing during dependency construction")?;
                if let Some(skin) = objects
                    .component(path)
                    .and_then(|component| component.concrete.skinnable.as_ref())
                    .and_then(|skinnable| skinnable.skin)
                {
                    objects.add_dependent(skin, path);
                }
            }
        }
        for edge_index in 0..graph.dependency_node_edges_in_insertion_order.len() {
            push_edge(edge_index);
        }

        for edge_index in ordered_edges {
            let edge = &graph.dependency_node_edges_in_insertion_order[edge_index];
            if matches!(
                edge.kind,
                nuxie_graph::DependencyKind::TargetedConstraint
                    | nuxie_graph::DependencyKind::IkConstraintTarget
                    | nuxie_graph::DependencyKind::IkConstraintTipChild
                    | nuxie_graph::DependencyKind::DrawTargetDrawable
                    | nuxie_graph::DependencyKind::DrawRulesTarget
                    | nuxie_graph::DependencyKind::SkinBone
                    | nuxie_graph::DependencyKind::SkinBoneConstraintParent
                    | nuxie_graph::DependencyKind::SkinMesh
                    | nuxie_graph::DependencyKind::SkinPointsPath
                    | nuxie_graph::DependencyKind::FollowPathConstraintParent
                    | nuxie_graph::DependencyKind::FollowPathConstraintTargetPathComposer
                    | nuxie_graph::DependencyKind::FollowPathConstraintTargetPath
            ) {
                continue;
            }
            let Some(mut source) = dependency_handles.get(edge.source_node).copied().flatten()
            else {
                continue;
            };
            let Some(mut dependent) = dependency_handles
                .get(edge.dependent_node)
                .copied()
                .flatten()
            else {
                continue;
            };
            if edge.kind == nuxie_graph::DependencyKind::ParentChild {
                let Some(current_parent) = objects
                    .component(dependent)
                    .and_then(|component| component.parent)
                else {
                    continue;
                };
                source = current_parent;
            } else if edge.kind == nuxie_graph::DependencyKind::TextVariationHelperText {
                let Some(current_text) = objects.text_variation_helper_text(source) else {
                    continue;
                };
                dependent = current_text;
            }
            objects.add_dependent(source, dependent);
        }

        // Skin allocates exactly one transform slot for identity plus one per
        // retained Tendon after dependencies are built. Initial FILTHY update
        // settles the non-identity slots before they are consumed.
        for component in &graph.components {
            if component.type_name != "Skin" {
                continue;
            }
            let handle = objects
                .component_handle(component.local_id)
                .context("Skin handle is missing")?;
            let skin = objects
                .component_mut(handle)
                .and_then(|component| component.concrete.skin.as_mut())
                .context("Skin occurrence is missing its concrete state")?;
            skin.bone_transforms = vec![Mat2D::IDENTITY; skin.tendons.len() + 1];
        }
        objects.sort_dependencies_from_root();
        retain_runtime_component_layout_topology(objects);
        Ok(())
    }

    pub fn from_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Result<Self> {
        Self::from_graph_with_file_view_model_instances(
            file,
            graph,
            RuntimeFileViewModelInstanceCatalog::new(file),
        )
    }

    #[doc(hidden)]
    pub fn from_graph_with_file_view_model_instances(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        file_view_model_instances: RuntimeFileViewModelInstanceCatalog,
    ) -> Result<Self> {
        let artboards = vec![graph.clone()];
        let context = RuntimeArtboardBuildContext {
            file: Arc::new(file.clone()),
            file_view_model_instances,
            state_machine_actions: RuntimeFileStateMachineActionCatalog::new(file),
            artboards: Arc::new(artboards.clone()),
            artboard_index_by_global: Arc::new(build_artboard_index_by_global(&artboards)),
            nested_structure_epoch: Arc::new(AtomicU64::new(0)),
            paint_preparation_epoch: Arc::new(AtomicU64::new(0)),
            external_font_assets: Arc::new(BTreeMap::new()),
        };
        Self::from_graph_inner(
            file,
            graph,
            &artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
        )
    }

    pub fn from_graph_with_artboards(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
    ) -> Result<Self> {
        Self::from_graph_with_artboards_and_external_fonts(file, graph, artboards, &BTreeMap::new())
    }

    /// Instantiate an artboard tree with a validated file-owned external font
    /// snapshot keyed by semantic `FileAsset.assetId`.
    pub fn from_graph_with_artboards_and_external_fonts(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        external_font_assets: &BTreeMap<u32, Arc<[u8]>>,
    ) -> Result<Self> {
        Self::from_graph_with_artboards_external_fonts_and_file_view_model_instances(
            file,
            graph,
            artboards,
            external_font_assets,
            RuntimeFileViewModelInstanceCatalog::new(file),
        )
    }

    #[doc(hidden)]
    pub fn from_graph_with_artboards_external_fonts_and_file_view_model_instances(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        external_font_assets: &BTreeMap<u32, Arc<[u8]>>,
        file_view_model_instances: RuntimeFileViewModelInstanceCatalog,
    ) -> Result<Self> {
        Self::from_graph_with_artboards_external_fonts_and_file_catalogs(
            file,
            graph,
            artboards,
            external_font_assets,
            file_view_model_instances,
            RuntimeFileStateMachineActionCatalog::new(file),
        )
    }

    #[doc(hidden)]
    pub fn from_graph_with_artboards_external_fonts_and_file_catalogs(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        external_font_assets: &BTreeMap<u32, Arc<[u8]>>,
        file_view_model_instances: RuntimeFileViewModelInstanceCatalog,
        state_machine_actions: RuntimeFileStateMachineActionCatalog,
    ) -> Result<Self> {
        let context = RuntimeArtboardBuildContext {
            file: Arc::new(file.clone()),
            file_view_model_instances,
            state_machine_actions,
            artboards: Arc::new(artboards.to_vec()),
            artboard_index_by_global: Arc::new(build_artboard_index_by_global(artboards)),
            nested_structure_epoch: Arc::new(AtomicU64::new(0)),
            paint_preparation_epoch: Arc::new(AtomicU64::new(0)),
            external_font_assets: Arc::new(external_font_assets.clone()),
        };
        Self::from_graph_inner(
            file,
            graph,
            artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
        )
    }

    fn from_graph_inner(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        visiting: &mut BTreeSet<u32>,
        build_context: Option<RuntimeArtboardBuildContext>,
        layout_constraint_bounds_enabled: bool,
    ) -> Result<Self> {
        let external_font_assets = build_context
            .as_ref()
            .map(|context| Arc::clone(&context.external_font_assets))
            .unwrap_or_default();
        let inserted = visiting.insert(graph.global_id);
        let dimensions =
            RuntimeArtboardDimensions::from_object(file.object(graph.global_id as usize));
        let mut slots = Vec::new();
        for local_object in &graph.local_objects {
            let object = file.object(local_object.global_id as usize);
            if local_object.type_name.is_some() && object.is_none() {
                anyhow::bail!(
                    "local object {} global id {} is missing",
                    local_object.local_id,
                    local_object.global_id
                );
            }
            slots.push(InstanceSlot {
                local_id: local_object.local_id,
                source_global_id: local_object.global_id,
                type_name: local_object.type_name,
                name: local_object.name.clone(),
            });
        }
        let mut objects = InstanceObjectArena::from_slots(file, &slots);
        apply_artboard_name_based_color_data_bind_defaults(file, graph, &mut objects);

        for component in &graph.components {
            file.object(component.global_id as usize).with_context(|| {
                format!("component global id {} is missing", component.global_id)
            })?;
            objects
                .attach_component(
                    component.local_id,
                    RuntimeComponent::from_graph_component(component),
                )
                .with_context(|| {
                    format!(
                        "component local id {} is missing or duplicate",
                        component.local_id
                    )
                })?;
        }
        for composer in &graph.path_composers {
            objects
                .attach_path_composer(
                    composer.shape_local,
                    RuntimeComponent::embedded(
                        composer.shape_local,
                        composer.shape_global,
                        "PathComposer",
                    ),
                )
                .with_context(|| {
                    format!(
                        "shape local id {} cannot own its PathComposer",
                        composer.shape_local
                    )
                })?;
        }

        Self::build_component_occurrence_relations(&mut objects, graph)?;
        retain_runtime_layout_component_styles(file, &slots, &mut objects);
        retain_runtime_solos(file, graph, &mut objects);
        retain_runtime_scroll_constraints(file, graph, &mut objects);
        retain_runtime_text_input_scroll_constraints(&mut objects);
        retain_runtime_drawable_component_topology(graph, &mut objects);
        let (advancing_components, resetting_components, component_lists) =
            Self::build_component_interface_schedules(&objects, graph);
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let mut linear_animations =
            build_linear_animations(file, graph, &slots, &mut converter_cache);
        let joysticks = build_runtime_joysticks(graph, &linear_animations);
        let state_machine_actions = build_context
            .as_ref()
            .map(|context| context.state_machine_actions.clone())
            .unwrap_or_else(|| RuntimeFileStateMachineActionCatalog::new(file));
        let state_machines = build_state_machines_with_action_catalog(
            file,
            graph,
            &linear_animations,
            &mut converter_cache,
            &state_machine_actions,
        );
        for state_machine in &state_machines {
            for layer in state_machine.layers.iter() {
                layer.validate_imported_references()?;
            }
        }
        let artboard_data_bind_values = build_artboard_default_view_model_values(file, graph);
        let mut artboard_authored_data_bind_states =
            build_artboard_authored_data_bind_states(file, graph, &objects);
        let mut artboard_property_bindings =
            build_artboard_property_bindings(file, graph, &mut converter_cache);
        let artboard_image_asset_bindings = build_artboard_image_asset_bindings(file, graph);
        let mut artboard_custom_property_bindings =
            build_artboard_custom_property_bindings(file, graph, &mut converter_cache);
        reunite_artboard_shared_data_bind_converter_states(
            &mut artboard_authored_data_bind_states,
            &mut artboard_property_bindings,
            &mut artboard_custom_property_bindings,
        );
        let artboard_layout_computed_bindings =
            build_artboard_layout_computed_bindings(file, graph);
        let artboard_numeric_source_bindings = build_artboard_numeric_source_bindings(file, graph);
        let artboard_formula_token_bindings =
            build_artboard_formula_token_bindings(file, graph, &mut converter_cache);
        let artboard_converter_property_bindings =
            build_artboard_converter_property_bindings(file, graph, &mut converter_cache);
        let artboard_list_bindings =
            build_artboard_list_bindings(file, graph, &mut converter_cache);
        let artboard_data_bind_target_queues = RuntimeArtboardDataBindTargetQueues::new(
            &artboard_property_bindings,
            &artboard_image_asset_bindings,
            &artboard_converter_property_bindings,
            &artboard_list_bindings,
        );
        let artboard_solo_bindings = build_artboard_solo_bindings(file, graph);
        let artboard_solo_source_bindings = build_artboard_solo_source_bindings(file, graph);
        let artboard_nested_host_bindings = build_artboard_nested_host_bindings(file, graph);
        let artboard_text_list_bindings = build_artboard_text_list_bindings(file, graph);
        let artboard_data_bind_source_queues = RuntimeArtboardDataBindSourceQueues::new(
            &artboard_custom_property_bindings,
            &artboard_layout_computed_bindings,
            &artboard_numeric_source_bindings,
            &artboard_solo_source_bindings,
        );
        for animation in &mut linear_animations {
            for keyed_object in Arc::make_mut(&mut animation.keyed_objects) {
                for keyed_property in &mut keyed_object.keyed_properties {
                    keyed_property.target.set_data_bind_observed(
                        artboard_data_bind_source_queues.observes_target_property(
                            keyed_object.target_local_id,
                            keyed_property.property_key,
                        ),
                    );
                }
            }
        }
        let nested_artboards = if inserted {
            build_runtime_nested_artboard_instances(
                file,
                graph,
                artboards,
                &slots,
                &objects,
                visiting,
                build_context.clone(),
            )?
        } else {
            RuntimeNestedArtboards::default()
        };
        if inserted {
            visiting.remove(&graph.global_id);
        }
        let nested_artboard_locals = nested_artboards.keys().copied().collect::<Vec<_>>();

        let text_affecting_locals = build_text_affecting_locals(&slots, &objects);
        let solid_color_paint_revisions = vec![
            1;
            slots
                .iter()
                .map(|slot| slot.local_id)
                .max()
                .map_or(0, |local_id| local_id.saturating_add(1))
        ];
        let runtime_drawables = RuntimeDrawableList::from_graph(graph, &objects);
        let mut instance = Self {
            instance_identity: RuntimeArtboardInstanceIdentity::next(),
            width: dimensions.width,
            height: dimensions.height,
            origin_x: dimensions.origin_x,
            origin_y: dimensions.origin_y,
            clip: dimensions.clip,
            frame_origin: Cell::new(true),
            frame_id: Cell::new(0),
            slots,
            objects,
            joysticks,
            advancing_components,
            resetting_components,
            component_lists,
            joysticks_apply_before_update: graph.joysticks_apply_before_update,
            linear_animations: Arc::new(linear_animations),
            empty_linear_animation: Arc::new(RuntimeLinearAnimation::empty()),
            state_machines: Arc::new(state_machines),
            script_instances_by_global: RuntimeScriptState::default(),
            script_attachment_generation: 0,
            scripted_data_converter_instances_by_global: RuntimeScriptState::default(),
            has_scripted_drawables: graph.components.iter().any(|component| {
                definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("ScriptedDrawable"))
            }),
            nested_script_owned_contexts: BTreeMap::new(),
            script_update_error: None,
            external_focus_domain: None,
            nested_artboards,
            nested_artboard_locals,
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: graph.global_id,
            build_context,
            nested_context_source_tree_cache: Cell::new(None),
            nested_layout_bounds: None,
            artboard_data_bind_values,
            artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            artboard_owned_view_model_context: None,
            artboard_owned_data_context: None,
            artboard_owned_view_model_handle: None,
            artboard_authored_data_bind_states,
            artboard_owned_view_model_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink::new(
            ),
            artboard_property_bindings,
            artboard_image_asset_bindings,
            artboard_data_bind_target_queues,
            artboard_data_bind_source_queues,
            artboard_retained_subordinate_converter_operands: Vec::new(),
            artboard_custom_property_bindings,
            artboard_layout_computed_bindings,
            artboard_numeric_source_bindings,
            artboard_formula_token_bindings,
            artboard_converter_property_bindings,
            artboard_solo_bindings,
            artboard_solo_source_bindings,
            artboard_nested_host_bindings,
            artboard_list_bindings,
            artboard_text_list_bindings,
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            stateful_nested_view_model_contexts_dirty: true,
            artboard_data_bind_dirty_epoch: 1,
            artboard_data_bind_processed_epoch: 0,
            image_asset_overrides: BTreeMap::new(),
            text_style_font_overrides: BTreeMap::new(),
            has_legacy_image_layout_scales: Cell::new(false),
            legacy_image_layout_scales: RefCell::new(BTreeMap::new()),
            external_font_assets,
            runtime_image_assets: RefCell::new(None),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            geometry_state: RefCell::new(crate::draw::RuntimeGeometryState::default()),
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_epoch: 1,
            text_affecting_locals,
            solid_color_paint_revisions,
            runtime_drawables,
            runtime_shapes: RuntimeShapeList::from_graph(graph),
            runtime_clipping_shapes: RuntimeClippingShapeList::from_graph(graph),
            runtime_meshes: crate::draw::RuntimeMeshList::from_graph(graph),
            did_change: Cell::new(true),
            layout_constraint_bounds_enabled,
            layout_constraint_bounds: None,
        };
        instance
            .runtime_shapes
            .rebuild_component_memberships(&instance.objects);
        instance.initialize_path_target_flags(graph);
        // C++ `DataBind::initialize` links every authored bind to its exact
        // Component target in import order before Solo/Layout can publish
        // initial collapse transitions (`data_bind.cpp:608-615`;
        // `component.cpp:108-127`).
        instance.initialize_component_data_bind_collapsables(file, graph);
        instance.apply_initial_component_collapse_callbacks_in_authored_order();
        instance.initialize_runtime_shape_paint_owners(graph);
        let nested_host_locals = instance.nested_artboard_locals.clone();
        for host_local_id in nested_host_locals {
            instance.sync_nested_artboard_root_opacity(host_local_id);
        }

        Ok(instance)
    }

    fn initialize_path_target_flags(&mut self, graph: &ArtboardGraph) {
        let component_handles = self.objects.component_handles().to_vec();
        for constraint in component_handles {
            if self
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.follow_path.as_ref())
                .is_none()
            {
                continue;
            }
            let Some(target) = self
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.constraint)
                .and_then(|constraint| constraint.target)
            else {
                continue;
            };
            if let Some(shape) = self
                .objects
                .component_mut(target)
                .and_then(|component| component.concrete.shape.as_mut())
            {
                shape.add_flags(crate::components::RuntimeShapeState::FOLLOW_PATH);
            } else if let Some(path) = self
                .objects
                .component_mut(target)
                .and_then(|component| component.concrete.path.as_mut())
            {
                path.add_flags(crate::components::RuntimePathState::FOLLOW_PATH);
            }
        }

        // `ClippingShape::onAddedClean` marks every source-subtree Shape as
        // clipping before the first dependency update. Those exact Shape
        // occurrences own the flag; their Paths consult it through `m_Shape`
        // (`src/shapes/clipping_shape.cpp:91-117`;
        // `src/shapes/path.cpp:111-125`).
        for clipping_shape in &graph.clipping_shapes {
            for &shape_local in &clipping_shape.shape_locals {
                if let Some(shape) = self
                    .objects
                    .component_for_local_mut(shape_local)
                    .and_then(|component| component.concrete.shape.as_mut())
                {
                    shape.add_flags(crate::components::RuntimeShapeState::CLIPPING);
                }
            }
        }

        // TextFollowPathModifier owns the same target flag producer as
        // FollowPathConstraint. Resolve its authored target once during the
        // concrete clean phase and mark the exact target occurrence
        // (`src/text/text_follow_path_modifier.cpp:34-49`).
        if let Some(target_key) = property_key_for_name("TextFollowPathModifier", "targetId") {
            for component in &graph.components {
                if component.type_name != "TextFollowPathModifier" {
                    continue;
                }
                let Some(target) = self
                    .objects
                    .uint_property(component.local_id, target_key)
                    .and_then(|target| usize::try_from(target).ok())
                    .and_then(|target| self.objects.component_handle(target))
                else {
                    continue;
                };
                if let Some(shape) = self
                    .objects
                    .component(target)
                    .and_then(|component| component.concrete.shape.as_ref())
                {
                    shape.add_flags(crate::components::RuntimeShapeState::FOLLOW_PATH);
                } else if let Some(path) = self
                    .objects
                    .component(target)
                    .and_then(|component| component.concrete.path.as_ref())
                {
                    path.add_flags(crate::components::RuntimePathState::FOLLOW_PATH);
                }
            }
        }
    }

    fn initialize_state_machine_hit_path_flags(&mut self, state_machine: &RuntimeStateMachine) {
        let mut hit_shapes = BTreeSet::new();
        for listener in state_machine.listeners.iter() {
            if !listener
                .listener_types
                .iter()
                .copied()
                .any(crate::state_machine::RuntimeListenerType::is_pointer_hit)
            {
                continue;
            }
            let Some(shape_handle) = self.objects.component_handle(listener.target_local_id) else {
                continue;
            };
            if !hit_shapes.insert(shape_handle) {
                continue;
            }
            if let Some(shape) = self
                .objects
                .component(shape_handle)
                .and_then(|component| component.concrete.shape.as_ref())
            {
                // C++ sets neverDeferUpdate and recursively publishes Path
                // dirt on the Shape at HitExpandable construction
                // (`state_machine_instance.cpp:1651-1661`).
                shape.add_flags(crate::components::RuntimeShapeState::NEVER_DEFER_UPDATE);
                self.add_component_dirt(shape_handle, ComponentDirt::PATH, true);
            }
        }
    }

    /// Return the external font bytes visible to this concrete runtime tree.
    pub fn external_font_asset_bytes(&self, asset_id: u32) -> Option<&[u8]> {
        self.external_font_assets.get(&asset_id).map(AsRef::as_ref)
    }

    /// Replace the validated external-font snapshot for this complete runtime
    /// tree, including contexts used by children materialized later.
    pub fn replace_external_font_asset_snapshot(
        &mut self,
        external_font_assets: &BTreeMap<u32, Arc<[u8]>>,
    ) {
        self.apply_external_font_asset_snapshot(Arc::new(external_font_assets.clone()));
    }

    fn apply_external_font_asset_snapshot(
        &mut self,
        external_font_assets: Arc<BTreeMap<u32, Arc<[u8]>>>,
    ) {
        self.external_font_assets = Arc::clone(&external_font_assets);
        if let Some(context) = self.build_context.as_mut() {
            context.external_font_assets = Arc::clone(&external_font_assets);
        }
        for nested in self.nested_artboards.values_mut() {
            nested
                .child
                .apply_external_font_asset_snapshot(Arc::clone(&external_font_assets));
        }
        for list_index in 0..self.component_list_count() {
            let Some(local_id) = self.component_list_local_at(list_index) else {
                continue;
            };
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child
                        .apply_external_font_asset_snapshot(Arc::clone(&external_font_assets));
                }
            }
        }
        self.mark_text_changed();
        self.mark_path_changed();
        self.mark_layout_changed();
    }

    pub fn component(&self, local_id: usize) -> Option<&RuntimeComponent> {
        self.objects.component_for_local(local_id)
    }

    /// Attach a VM-owned script instance to a scripted object global id.
    ///
    /// Ported toward C++ `src/scripted/scripted_drawable.cpp`: the runtime draw
    /// path owns the `ScriptedDrawable` envelope, while the backend VM owns the
    /// instance table and `draw(self, renderer)` method.
    pub fn set_script_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) {
        self.set_script_instance_for_global_with_implemented_methods(
            global_id,
            instance,
            RuntimeScriptImplementedMethods::METHOD_MASK,
        );
    }

    /// Attach one VM table with the exact ScriptAsset method mask copied by
    /// pinned C++ `ScriptAsset::initScriptedObjectWith`.
    #[doc(hidden)]
    pub fn set_script_instance_for_global_with_implemented_methods(
        &mut self,
        global_id: u32,
        mut instance: Box<dyn ScriptInstance>,
        serialized_implemented_methods: u32,
    ) {
        self.has_scripted_drawables = true;
        let user_init_pending = instance.user_init_pending().unwrap_or(false);
        let advance_active =
            !user_init_pending && instance.has_method(ScriptMethod::Advance).unwrap_or(false);
        self.script_instances_by_global.insert(
            global_id,
            RuntimeScriptedObjectOccurrence::new(instance, serialized_implemented_methods),
        );
        self.script_attachment_generation = self.script_attachment_generation.wrapping_add(1);
        if self.set_script_owner_lifecycle(global_id, advance_active, !user_init_pending)
            && !user_init_pending
            && let Some(component) = self.script_component_handle_for_global(global_id)
        {
            // VM attachment completes the concrete ScriptedDrawable owner
            // after the Artboard's initial FILTHY pass may already have run.
            // Publish the same ScriptUpdate bit that C++ consumes at this
            // owner's ordinary dependency slot (`scripted_drawable.cpp:
            // 347-374`; `component.cpp:222-241`).
            self.add_component_dirt(component, ComponentDirt::SCRIPT_UPDATE, false);
        }
    }

    /// Whether this artboard instance already owns a script instance for the
    /// file-global scripted-object id.
    pub fn has_script_instance_for_global(&self, global_id: u32) -> bool {
        self.script_instances_by_global.contains_key(&global_id)
    }

    /// Rearm a scripted drawable's `advance` callback after an input event.
    ///
    /// This is the Rust lifecycle seam for C++ `ScriptedDrawable::wakeAdvance`:
    /// pointer, keyboard, gamepad, and text events can make a previously idle
    /// script active again and invalidate its paint output.
    pub fn wake_script_advance_for_global(&mut self, global_id: u32) -> bool {
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return false;
        };
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.set_script_owner_advance_active(global_id, true);
        }

        let local_id = self.component_local_for_global(global_id);
        if let Some(local_id) = local_id {
            self.add_dirt(local_id, ComponentDirt::PAINT, false);
        }
        true
    }

    pub fn graph_global_id(&self) -> u32 {
        self.graph_global_id
    }

    pub fn set_script_path_effect_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) {
        self.set_script_instance_for_global(global_id, instance);
        let local_id = self.component_local_for_global(global_id);
        if let Some(local_id) = local_id {
            // Cold hydration happened before the VM instance could be attached
            // to this ArtboardInstance. Replay the component dirt left by
            // C++ `setNumberInput`/siblings and
            // `ScriptedPathEffect::didHydrateScriptInputs`.
            self.add_dirt(local_id, ComponentDirt::SCRIPT_UPDATE, false);
            self.add_dirt(local_id, ComponentDirt::PAINT, true);
        }
    }

    /// Complete C++ `ScriptedPathEffect::didHydrateScriptInputs` after a
    /// bind-time input replay (`scripted_path_effect.cpp:15-19`).
    pub fn did_hydrate_script_inputs_for_global(&mut self, global_id: u32) -> bool {
        if !self
            .script_component_handle_for_global(global_id)
            .and_then(|handle| self.objects.component(handle))
            .is_some_and(|component| component.type_name == "ScriptedPathEffect")
        {
            return false;
        }
        let Some(local_id) = self
            .components()
            .iter()
            .find(|component| component.global_id == global_id)
            .map(|component| component.local_id)
        else {
            return false;
        };
        self.add_dirt(local_id, ComponentDirt::PAINT, true)
    }

    /// Runs the C++ `ScriptedDrawable::update` phase for scripts dirtied by
    /// initialization or input hydration.
    pub fn update_script_instances(&mut self) -> Result<bool, ScriptError> {
        self.update_script_instances_with(|instance, host| {
            instance.call_method(ScriptMethod::Update, &[], host)
        })
    }

    /// Runs pending scripted-object updates with a renderer factory available
    /// to back any `Paint` values allocated by user code.
    pub fn update_script_instances_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        self.update_script_instances_with(|instance, host| {
            instance.call_method_with_factory(ScriptMethod::Update, &[], host, factory)
        })
    }

    fn update_script_instances_with(
        &mut self,
        mut call_update: impl FnMut(
            &mut dyn ScriptInstance,
            &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError>,
    ) -> Result<bool, ScriptError> {
        // ScriptedDrawable::update is a normal Component update. Preserve the
        // dependency-sorted owner order rather than draining an Artboard-wide
        // global-id set (`component.cpp:222-241`;
        // `scripted_drawable.cpp:347-374`).
        let pending = self
            .objects
            .dependency_order()
            .iter()
            .copied()
            .filter_map(|component| {
                let owner = self.objects.component(component)?;
                owner
                    .concrete
                    .scripted
                    .is_some_and(|scripted| scripted.update_pending)
                    .then_some((component, owner.global_id))
            })
            .collect::<Vec<_>>();
        let mut did_update = false;
        let mut host = NoopScriptHost;
        for (index, (component, global_id)) in pending.iter().copied().enumerate() {
            if self
                .objects
                .component(component)
                .is_some_and(|component| component.type_name == "ScriptedPathEffect")
            {
                self.set_script_owner_update_pending(component, false);
                continue;
            }
            if self
                .objects
                .component(component)
                .is_some_and(RuntimeComponent::is_collapsed)
            {
                continue;
            }
            self.set_script_owner_update_pending(component, false);
            let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
                continue;
            };
            let mut instance = handle.borrow_mut();
            let has_update = match instance.has_method(ScriptMethod::Update) {
                Ok(has_update) => has_update,
                Err(error) => {
                    for (component, _) in &pending[index..] {
                        self.set_script_owner_update_pending(*component, true);
                    }
                    return Err(error);
                }
            };
            if !has_update {
                continue;
            }
            if let Err(error) = call_update(instance.as_mut(), &mut host) {
                for (component, _) in &pending[index..] {
                    self.set_script_owner_update_pending(*component, true);
                }
                return Err(error);
            }
            did_update = true;
        }
        Ok(did_update)
    }

    pub fn advance_script_instances(&mut self, seconds: f32) -> Result<bool, ScriptError> {
        self.advance_script_instances_with(seconds, |instance, args, host| {
            instance.call_method(ScriptMethod::Advance, args, host)
        })
    }

    pub fn advance_script_instances_with_factory(
        &mut self,
        seconds: f32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        self.advance_script_instances_with(seconds, |instance, args, host| {
            instance.call_method_with_factory(ScriptMethod::Advance, args, host, factory)
        })
    }

    fn advance_script_instances_with(
        &mut self,
        seconds: f32,
        mut call_advance: impl FnMut(
            &mut dyn ScriptInstance,
            &[ScriptValue],
            &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError>,
    ) -> Result<bool, ScriptError> {
        if seconds == 0.0 {
            return Ok(false);
        }
        // C++ does not drain a file-global script table here. Each
        // ScriptedDrawable/ScriptedLayout/ScriptedPathEffect occupies its own
        // slot in Artboard::m_advancingComponents, and that retained
        // insertion-order list is the dispatch authority
        // (`advancing_component.cpp:17-44`;
        // `scripted_drawable.cpp:376-397`;
        // `scripted_path_effect.cpp:111-130`).
        let mut did_advance = false;
        let mut host = NoopScriptHost;
        for index in 0..self.advancing_components.len() {
            let entry = self.advancing_components[index];
            if !matches!(
                entry.kind,
                AdvancingComponentKind::ScriptedDrawable
                    | AdvancingComponentKind::ScriptedLayout
                    | AdvancingComponentKind::ScriptedPathEffect
            ) {
                continue;
            }
            did_advance |=
                self.advance_script_component_with(entry, seconds, &mut host, &mut call_advance)?;
        }
        Ok(did_advance)
    }

    fn advance_script_component_with(
        &mut self,
        entry: RuntimeAdvancingComponent,
        seconds: f32,
        host: &mut dyn ScriptHost,
        call_advance: &mut impl FnMut(
            &mut dyn ScriptInstance,
            &[ScriptValue],
            &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError>,
    ) -> Result<bool, ScriptError> {
        if seconds == 0.0 {
            return Ok(false);
        }
        let Some(component) = entry.component else {
            return Ok(false);
        };
        let Some((global_id, active, collapsed)) =
            self.objects.component(component).and_then(|owner| {
                Some((
                    owner.global_id,
                    owner.concrete.scripted.as_ref()?.advance_active,
                    owner.is_collapsed(),
                ))
            })
        else {
            return Ok(false);
        };
        if !active || (entry.kind != AdvancingComponentKind::ScriptedPathEffect && collapsed) {
            return Ok(false);
        }

        // C++ clears m_isAdvanceActive before entering user code. A true
        // result rearms the same owner; false parks it until wakeAdvance
        // (`scripted_drawable.cpp:376-397`;
        // `scripted_path_effect.cpp:111-130`).
        self.set_script_owner_advance_active_handle(component, false);
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let result = match call_advance(
            handle.borrow_mut().as_mut(),
            &[ScriptValue::Number(f64::from(seconds))],
            host,
        ) {
            Ok(result) => result,
            Err(error) => {
                self.set_script_owner_advance_active_handle(component, true);
                return Err(error);
            }
        };
        if result != ScriptValue::Bool(true) {
            return Ok(false);
        }
        self.set_script_owner_advance_active_handle(component, true);
        if entry.kind != AdvancingComponentKind::ScriptedPathEffect {
            self.add_dirt(entry.local_id, ComponentDirt::PAINT, false);
        }
        Ok(true)
    }

    fn component_local_for_global(&self, global_id: u32) -> Option<usize> {
        self.components()
            .iter()
            .find(|component| component.global_id == global_id)
            .map(|component| component.local_id)
    }

    fn script_component_handle_for_global(&self, global_id: u32) -> Option<ComponentHandle> {
        self.objects
            .component_handles()
            .iter()
            .copied()
            .find(|handle| {
                self.objects
                    .component(*handle)
                    .is_some_and(|component| component.global_id == global_id)
            })
    }

    fn set_script_owner_lifecycle(
        &mut self,
        global_id: u32,
        advance_active: bool,
        update_pending: bool,
    ) -> bool {
        let Some(component) = self.script_component_handle_for_global(global_id) else {
            return false;
        };
        let Some(scripted) = self
            .objects
            .component_mut(component)
            .and_then(|component| component.concrete.scripted.as_mut())
        else {
            return false;
        };
        scripted.advance_active = advance_active;
        scripted.update_pending = update_pending;
        true
    }

    fn set_script_owner_advance_active(&mut self, global_id: u32, active: bool) -> bool {
        let Some(component) = self.script_component_handle_for_global(global_id) else {
            return false;
        };
        self.set_script_owner_advance_active_handle(component, active)
    }

    fn set_script_owner_advance_active_handle(
        &mut self,
        component: ComponentHandle,
        active: bool,
    ) -> bool {
        let Some(scripted) = self
            .objects
            .component_mut(component)
            .and_then(|component| component.concrete.scripted.as_mut())
        else {
            return false;
        };
        scripted.advance_active = active;
        true
    }

    fn set_script_owner_update_pending(
        &mut self,
        component: ComponentHandle,
        pending: bool,
    ) -> bool {
        let Some(scripted) = self
            .objects
            .component_mut(component)
            .and_then(|component| component.concrete.scripted.as_mut())
        else {
            return false;
        };
        scripted.update_pending = pending;
        true
    }

    fn mark_script_owner_update_pending(&mut self, global_id: u32) -> bool {
        let Some(component) = self.script_component_handle_for_global(global_id) else {
            return false;
        };
        self.set_script_owner_update_pending(component, true)
    }

    /// Re-runs user `init` after C++ clears a scripted object's data context.
    pub fn reinitialize_script_instances(&mut self) -> Result<bool, ScriptError> {
        let mut did_initialize = false;
        let mut host = NoopScriptHost;
        let instances = self
            .script_instances_by_global
            .iter()
            .map(|(global_id, handle)| (*global_id, handle.clone()))
            .collect::<Vec<_>>();
        for (global_id, handle) in instances {
            let mut instance = handle.borrow_mut();
            if !instance.has_method(ScriptMethod::Init)? {
                continue;
            }
            instance.call_method(ScriptMethod::Init, &[], &mut host)?;
            let advance_active = instance.has_method(ScriptMethod::Advance).unwrap_or(false);
            self.set_script_owner_lifecycle(global_id, advance_active, true);
            did_initialize = true;
        }
        Ok(did_initialize)
    }

    pub fn reinitialize_script_instances_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut did_initialize = false;
        let mut host = NoopScriptHost;
        let instances = self
            .script_instances_by_global
            .iter()
            .map(|(global_id, handle)| (*global_id, handle.clone()))
            .collect::<Vec<_>>();
        for (global_id, handle) in instances {
            let mut instance = handle.borrow_mut();
            if !instance.has_method(ScriptMethod::Init)? {
                continue;
            }
            instance.call_method_with_factory(ScriptMethod::Init, &[], &mut host, factory)?;
            let advance_active = instance.has_method(ScriptMethod::Advance).unwrap_or(false);
            self.set_script_owner_lifecycle(global_id, advance_active, true);
            did_initialize = true;
        }
        Ok(did_initialize)
    }

    pub fn reinitialize_script_instance_with_factory(
        &mut self,
        global_id: u32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let mut instance = handle.borrow_mut();
        if !instance.has_method(ScriptMethod::Init)? {
            return Ok(false);
        }
        let initialized = instance.call_init_with_factory(&mut NoopScriptHost, factory)?;
        if initialized {
            let advance_active = instance.has_method(ScriptMethod::Advance).unwrap_or(false);
            self.set_script_owner_lifecycle(global_id, advance_active, true);
        } else {
            self.set_script_owner_lifecycle(global_id, false, false);
        }
        Ok(initialized)
    }

    pub fn script_user_init_pending_for_global(&self, global_id: u32) -> Result<bool, ScriptError> {
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let pending = handle.borrow_mut().user_init_pending()?;
        Ok(pending)
    }

    pub fn prepare_script_init_retry_with_factory(
        &mut self,
        global_id: u32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let mut instance = handle.borrow_mut();
        if !instance.user_init_pending()? {
            return Ok(false);
        }
        instance.prepare_init_retry_with_factory(factory)?;
        Ok(true)
    }

    pub fn set_script_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        value: ScriptValue,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_input(name, value)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.set_script_owner_advance_active(global_id, true);
        }
        self.mark_script_owner_update_pending(global_id);
        let local_id = self.component_local_for_global(global_id);
        if let Some(local_id) = local_id {
            // Direct counterpart of `ScriptedObject::setNumberInput` and its
            // sibling setters: every authored input write schedules
            // `ScriptUpdate` on component-backed scripted objects
            // (`scripted_object.cpp:61-117`).
            self.add_dirt(local_id, ComponentDirt::SCRIPT_UPDATE, false);
        }
        Ok(())
    }

    pub fn set_script_artboard_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_artboard_input(name, artboard)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.set_script_owner_advance_active(global_id, true);
        }
        self.mark_script_owner_update_pending(global_id);
        Ok(())
    }

    pub fn set_script_view_model_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        view_model: ScriptViewModel,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_view_model_input(name, view_model)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.set_script_owner_advance_active(global_id, true);
        }
        self.mark_script_owner_update_pending(global_id);
        Ok(())
    }

    pub fn set_script_context_view_model(
        &mut self,
        view_model: Option<ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        for handle in self.script_instances_by_global.values() {
            handle
                .borrow_mut()
                .set_context_view_model(view_model.clone())?;
        }
        Ok(())
    }

    pub fn mark_script_update_for_global(&mut self, global_id: u32) -> bool {
        if !self.script_instances_by_global.contains_key(&global_id) {
            return false;
        }
        self.mark_script_owner_update_pending(global_id)
    }

    pub(crate) fn script_instance_for_global(
        &self,
        global_id: u32,
    ) -> Option<RuntimeScriptInstanceHandle> {
        self.script_instances_by_global
            .get(&global_id)
            .map(RuntimeScriptedObjectOccurrence::instance)
    }

    pub(crate) fn script_implemented_methods_for_global(
        &self,
        global_id: u32,
    ) -> Option<RuntimeScriptImplementedMethods> {
        self.script_instances_by_global
            .get(&global_id)
            .map(RuntimeScriptedObjectOccurrence::implemented_methods)
    }

    pub(crate) fn script_attachment_generation(&self) -> u64 {
        self.script_attachment_generation
    }

    pub fn slot(&self, local_id: usize) -> Option<&InstanceSlot> {
        self.slots.get(local_id)
    }

    pub fn slots(&self) -> &[InstanceSlot] {
        &self.slots
    }

    /// Snapshot authored custom properties attached to one event, preserving
    /// their component/local order.
    pub fn event_properties(&self, event_local_id: usize) -> Vec<RuntimeEventProperty> {
        let Some(event) = self.component_handle(event_local_id) else {
            return Vec::new();
        };
        (0..self.component_child_len(event))
            .filter_map(|index| self.component_child_at(event, index))
            .filter_map(|handle| self.objects.component(handle))
            .filter_map(|component| {
                let key = property_key_for_name(component.type_name, "propertyValue")?;
                let value = match component.type_name {
                    "CustomPropertyNumber" => RuntimeEventPropertyValue::Number(
                        self.double_property(component.local_id, key)?,
                    ),
                    "CustomPropertyBoolean" => RuntimeEventPropertyValue::Bool(
                        self.bool_property(component.local_id, key)?,
                    ),
                    "CustomPropertyString" => RuntimeEventPropertyValue::String(
                        self.string_property(component.local_id, key)?.to_vec(),
                    ),
                    "CustomPropertyColor" => RuntimeEventPropertyValue::Color(
                        self.color_property(component.local_id, key)?,
                    ),
                    "CustomPropertyEnum" => RuntimeEventPropertyValue::Enum(
                        self.uint_property(component.local_id, key)?,
                    ),
                    "CustomPropertyTrigger" => RuntimeEventPropertyValue::Trigger(
                        self.uint_property(component.local_id, key)?,
                    ),
                    _ => return None,
                };
                let name_key = property_key_for_name(component.type_name, "name")?;
                Some(RuntimeEventProperty {
                    name: self
                        .string_property(component.local_id, name_key)
                        .map(|value| String::from_utf8_lossy(value).into_owned())
                        .filter(|name| !name.is_empty()),
                    value,
                })
            })
            .collect()
    }

    pub(crate) fn component_mut(&mut self, local_id: usize) -> Option<&mut RuntimeComponent> {
        self.objects.component_for_local_mut(local_id)
    }

    pub(crate) fn component_list_state(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeConstrainableListState> {
        self.component(local_id)?
            .concrete
            .constrainable_list
            .as_ref()
    }

    pub(crate) fn component_list_state_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut RuntimeConstrainableListState> {
        self.component_mut(local_id)?
            .concrete
            .constrainable_list
            .as_mut()
    }

    pub(crate) fn component_list_items(
        &self,
        local_id: usize,
    ) -> Option<&[RuntimeComponentListItemInstance]> {
        Some(&self.component_list_state(local_id)?.items)
    }

    pub(crate) fn component_list_items_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut Vec<RuntimeComponentListItemInstance>> {
        Some(&mut self.component_list_state_mut(local_id)?.items)
    }

    pub(crate) fn component_list_locals(&self) -> impl Clone + Iterator<Item = usize> + '_ {
        self.component_lists
            .iter()
            .map(|handle| self.component_at(*handle).local_id)
    }

    pub(crate) fn component_list_count(&self) -> usize {
        self.component_lists.len()
    }

    pub(crate) fn component_list_local_at(&self, index: usize) -> Option<usize> {
        Some(
            self.component_at(*self.component_lists.get(index)?)
                .local_id,
        )
    }

    pub(crate) fn set_component_list_source(
        &mut self,
        local_id: usize,
        source: Option<RuntimeOwnedViewModelListHandle>,
    ) {
        if let Some(list) = self.component_list_state_mut(local_id) {
            list.source = source;
        }
    }

    pub fn components(&self) -> RuntimeComponents<'_> {
        RuntimeComponents {
            arena: &self.objects,
        }
    }

    pub(crate) fn component_at(&self, handle: ComponentHandle) -> &RuntimeComponent {
        self.objects
            .component(handle)
            .expect("runtime component handle must address its occurrence")
    }

    pub(crate) fn component_at_mut(&mut self, handle: ComponentHandle) -> &mut RuntimeComponent {
        self.objects
            .component_mut(handle)
            .expect("runtime component handle must address its occurrence")
    }

    pub(crate) fn component_handle(&self, local_id: usize) -> Option<ComponentHandle> {
        self.objects.component_handle(local_id)
    }

    /// Exact Component/Drawable/LayoutComponent virtual hit-test chain.
    ///
    /// The `ComponentHandle` is the occurrence-local counterpart of the C++
    /// pointer. Callers that start from a `DrawableProxy` pass its retained
    /// `hittableComponent()` target, preserving both boolean arguments
    /// (`src/drawable.cpp:62-77`; `src/component.cpp:97-105`;
    /// `src/layout_component.cpp:49-80`).
    pub(crate) fn component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        if owner.concrete.layout.is_some() {
            return self.layout_component_hit_test_point(
                component,
                position,
                skip_on_unclipped,
                is_primary_hit,
            );
        }
        if owner.concrete.drawable.is_some() {
            return self.drawable_component_hit_test_point(
                component,
                position,
                skip_on_unclipped,
                is_primary_hit,
            );
        }
        self.base_component_hit_test_point(component, position, skip_on_unclipped, is_primary_hit)
    }

    fn layout_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        let Some(layout) = owner.concrete.layout.as_ref() else {
            return false;
        };
        let world = owner.transform.world_transform;
        if world.determinant() == 0.0 {
            return false;
        }

        let clip = layout
            .clip_property_key
            .and_then(|key| self.objects.component_bool_property(component, key))
            .unwrap_or(owner.type_name == "Artboard" && self.clip);
        if !(skip_on_unclipped && !clip) {
            let mut local = world
                .invert_or_identity()
                .transform_point(position.0, position.1);
            let (_, _, width, height) = layout.constraint_bounds();
            if owner.type_name == "Artboard" && (self.origin_x != 0.0 || self.origin_y != 0.0) {
                local.0 += self.origin_x * width;
                local.1 += self.origin_y * height;
            }
            if local.0 < 0.0 || local.0 > width || local.1 < 0.0 || local.1 > height {
                return false;
            }
        }

        // LayoutComponent explicitly invokes Drawable's implementation after
        // its local-bounds check, changing only skipOnUnclipped to true.
        self.drawable_component_hit_test_point(component, position, true, is_primary_hit)
    }

    fn drawable_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        let hidden = owner.is_collapsed()
            || owner
                .concrete
                .drawable
                .as_ref()
                .and_then(|drawable| drawable.drawable_flags_property_key)
                .and_then(|key| self.objects.component_uint_property(component, key))
                .is_some_and(|flags| flags & 1 != 0);
        if hidden {
            return false;
        }

        // Ordinary Drawable::hittableComponent returns `this`; proxy callers
        // have already supplied the proxy target at this method's public
        // boundary.
        self.base_component_hit_test_point(component, position, skip_on_unclipped, is_primary_hit)
    }

    fn base_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        let Some(parent) = self
            .objects
            .component(component)
            .and_then(|component| component.parent)
        else {
            return true;
        };
        self.component_hit_test_point(parent, position, skip_on_unclipped, false)
    }

    pub(crate) fn component_parent_handle(
        &self,
        handle: ComponentHandle,
    ) -> Option<ComponentHandle> {
        self.objects.component(handle)?.parent
    }

    pub(crate) fn component_local_id(&self, handle: ComponentHandle) -> Option<usize> {
        self.objects.component_local_id(handle)
    }

    pub(crate) fn component_parent_local(&self, local_id: usize) -> Option<usize> {
        let handle = self.component_handle(local_id)?;
        let parent = self.component_parent_handle(handle)?;
        self.objects.component_local_id(parent)
    }

    pub(crate) fn runtime_object_type_name(&self, local_id: usize) -> Option<&'static str> {
        self.slot(local_id).and_then(|slot| slot.type_name)
    }

    pub(crate) fn component_child_len(&self, handle: ComponentHandle) -> usize {
        self.objects.child_len(handle)
    }

    pub(crate) fn component_child_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<ComponentHandle> {
        self.objects.child_at(handle, index)
    }

    pub(crate) fn runtime_file(&self) -> Option<&RuntimeFile> {
        self.build_context
            .as_ref()
            .map(|context| context.file.as_ref())
    }

    pub(crate) fn runtime_file_arc(&self) -> Option<Arc<RuntimeFile>> {
        self.build_context
            .as_ref()
            .map(|context| Arc::clone(&context.file))
    }

    pub(crate) fn runtime_file_view_model_instances(
        &self,
    ) -> Option<RuntimeFileViewModelInstanceCatalog> {
        self.build_context
            .as_ref()
            .map(|context| context.file_view_model_instances.clone())
    }

    /// Construct an imported context already attached to this artboard's
    /// canonical file occurrence. Trigger writes made before state-machine
    /// binding therefore mutate the same retained C++ instance immediately.
    pub fn imported_view_model_instance_context(
        &self,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<RuntimeImportedViewModelInstanceContext> {
        let context = self.build_context.as_ref()?;
        let instance = context
            .file_view_model_instances
            .instance(view_model_index, instance_index)?;
        RuntimeImportedViewModelInstanceContext::from_file_trigger_instance(
            context.file.as_ref(),
            view_model_index,
            instance_index,
            instance,
        )
    }

    pub(crate) fn nested_structure_epoch(&self) -> Option<u64> {
        self.build_context
            .as_ref()
            .map(|context| context.nested_structure_epoch.load(Ordering::Relaxed))
    }

    pub(crate) fn tree_paint_preparation_epoch(&self) -> Option<u64> {
        self.build_context
            .as_ref()
            .map(|context| context.paint_preparation_epoch.load(Ordering::Relaxed))
    }

    fn mark_tree_paint_preparation_changed(&self) {
        if let Some(context) = self.build_context.as_ref() {
            context
                .paint_preparation_epoch
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn mark_nested_structure_changed(&self) {
        self.nested_context_source_tree_cache.set(None);
        if let Some(context) = self.build_context.as_ref() {
            context
                .nested_structure_epoch
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn runtime_graph(&self) -> Option<&ArtboardGraph> {
        self.runtime_graph_for_global(self.graph_global_id)
    }

    pub(crate) fn runtime_graph_for_global(&self, graph_global_id: u32) -> Option<&ArtboardGraph> {
        let context = self.build_context.as_ref()?;
        let index = context
            .artboard_index_by_global
            .get(usize::try_from(graph_global_id).ok()?)
            .copied()
            .flatten()?;
        context.artboards.get(index)
    }

    pub fn linear_animation(&self, index: usize) -> Option<&RuntimeLinearAnimation> {
        self.linear_animations.get(index)
    }

    pub fn linear_animations(&self) -> &[RuntimeLinearAnimation] {
        self.linear_animations.as_slice()
    }

    pub fn state_machine(&self, index: usize) -> Option<&RuntimeStateMachine> {
        self.state_machines.get(index)
    }

    pub fn state_machines(&self) -> &[RuntimeStateMachine] {
        self.state_machines.as_slice()
    }

    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        self.mark_artboard_data_bind_work_dirty();
        self.mark_changed();
        self.mark_layout_changed();
        // C++ layout settlement adds Path dirt when the solved width or
        // height changes, before LayoutComponent::update rebuilds the
        // Artboard-owned local/world paths
        // (`layout_component.cpp:1116-1124`, `artboard.cpp:1138-1157`).
        self.add_dirt(0, ComponentDirt::PATH | ComponentDirt::COMPONENTS, false);
        true
    }

    /// Current root-artboard dimensions after runtime layout and data binding.
    pub fn artboard_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Current authored artboard bounds in artboard coordinates.
    ///
    /// Rive stores the origin as normalized fractions of width and height;
    /// the logical top-left is therefore the negative origin offset.
    pub fn artboard_bounds(&self) -> (f32, f32, f32, f32) {
        (
            -self.width * self.origin_x,
            -self.height * self.origin_y,
            self.width,
            self.height,
        )
    }

    /// Whether authored nested or component-list players still need a future
    /// advance. Hosts use this independently from the selected root player so
    /// a static root cannot prematurely settle a playing child artboard.
    pub fn has_ongoing_nested_work(&self) -> bool {
        self.nested_artboards
            .values()
            .any(RuntimeNestedArtboardInstance::has_ongoing_work)
            || self.component_list_locals().into_iter().any(|local_id| {
                self.component_list_items(local_id).is_some_and(|items| {
                    items.iter().any(|item| {
                        item.state_machines
                            .iter()
                            .any(StateMachineInstance::needs_advance)
                            || item.child.has_ongoing_nested_work()
                    })
                })
            })
    }

    pub(crate) fn artboard_property_value(&self, property_type: u64) -> f32 {
        match property_type {
            0 => self.width,
            1 => self.height,
            2 => self.width / self.height,
            _ => 0.0,
        }
    }

    /// Reads one typed color property from the live object arena.
    ///
    /// Returns `None` when either the local object or a color property with
    /// this key does not exist. Schema defaults are already materialized in
    /// the object arena, so a matching property returns its current value
    /// even when the source record omitted that default.
    pub fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.objects.color_property(local_id, property_key)
    }

    pub(crate) fn solid_color_value(&self, local_id: usize) -> Option<u32> {
        self.objects.solid_color_value(local_id)
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_color_property(&mut self, local_id: usize, property_key: u16, value: u32) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    pub(crate) fn set_keyed_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_generated_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    /// C++ keyed animations retain a concrete Core pointer, so a known
    /// `SolidColor::colorValue` write does not rediscover its type or property
    /// on every frame. Keep the same observer and invalidation effects as the
    /// generic color setter while skipping branches that cannot apply to a
    /// SolidColor target (text, view-model, gradient, and layout topology).
    pub(crate) fn set_keyed_solid_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        data_bind_observed: bool,
        value: u32,
    ) -> bool {
        let Some(previous) = self.objects.replace_solid_color_value(local_id, value) else {
            return false;
        };
        // Generated C++ setters return before the property callback when the
        // stored value is unchanged (`solid_color_base.hpp:38-46`). Active
        // animations may apply the same keyed value every frame; do not
        // rebuild or reconfigure the retained ShapePaint owner in that case.
        if previous == value {
            return false;
        }
        if data_bind_observed {
            self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        }
        self.mark_changed();
        // Pinned C++ `SolidColor::colorValueChanged` immediately calls
        // `renderOpacityChanged` and mutates the ShapePaint-owned paint
        // (`solid_color.cpp:23-54`). It does not dirty or reconstruct the
        // ShapePaint owner.
        self.settle_runtime_solid_color_callback(local_id, value);
        if let Some(revision) = self.solid_color_paint_revisions.get_mut(local_id) {
            *revision = revision.wrapping_add(1);
        }
        self.mark_tree_paint_preparation_changed();
        self.mark_prepared_changed_for_solid_color_visibility(Some(previous), value);
        true
    }

    fn after_color_property_set(
        &mut self,
        local_id: usize,
        property_key: u16,
        previous: Option<u32>,
        value: u32,
    ) -> bool {
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("SolidColor")
            && solid_color_value_property_key() == Some(property_key)
        {
            // Pinned C++ `SolidColor::colorValueChanged` immediately calls
            // `renderOpacityChanged`, which mutates the already-owned
            // RenderPaint in place (`solid_color.cpp:23-54`). It does not
            // queue a complete ShapePaint reconstruction for draw time.
            self.settle_runtime_solid_color_callback(local_id, value);
            if let Some(revision) = self.solid_color_paint_revisions.get_mut(local_id) {
                *revision = revision.wrapping_add(1);
            }
            // SolidColor mutates its retained paint in place, so it does not
            // invalidate local prepared geometry. A parent preparation frame
            // still needs to observe the nested paint value change.
            self.mark_tree_paint_preparation_changed();
        } else {
            self.mark_runtime_shape_property_changed(local_id, false);
        }
        self.mark_prepared_changed_for_color_property(local_id, property_key, previous, value);
        self.apply_color_property_changed(local_id, property_key);
        true
    }

    fn settle_runtime_solid_color_callback(&self, local_id: usize, value: u32) {
        let Some(context) = self.build_context.as_ref() else {
            return;
        };
        let Ok(graph_global_id) = usize::try_from(self.graph_global_id) else {
            return;
        };
        let Some(graph_index) = context
            .artboard_index_by_global
            .get(graph_global_id)
            .copied()
            .flatten()
        else {
            return;
        };
        let graphs = Arc::clone(&context.artboards);
        let Some(graph) = graphs.get(graph_index) else {
            return;
        };
        self.settle_runtime_solid_color_callback_with_graph(local_id, value, graph);
    }

    pub(crate) fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedBool")
            && property_key_for_name("NestedBool", "nestedValue") == Some(property_key)
        {
            // `NestedBool::nestedValue()` reads the live SMIBool occurrence
            // and returns false when the nested input cannot be resolved
            // (`src/animation/nested_bool.cpp:36-48`).
            return Some(self.nested_bool_value(local_id).unwrap_or(false));
        }
        self.objects.bool_property(local_id, property_key)
    }

    pub(crate) fn shape_paint_is_visible(&self, local_id: usize) -> Option<bool> {
        self.objects.shape_paint_is_visible(local_id)
    }

    pub(crate) fn shape_paint_blend_mode_value(&self, local_id: usize) -> Option<u64> {
        self.objects.shape_paint_blend_mode_value(local_id)
    }

    pub(crate) fn fill_rule(&self, local_id: usize) -> Option<u64> {
        self.objects.fill_rule(local_id)
    }

    pub(crate) fn stroke_transform_affects_stroke(&self, local_id: usize) -> Option<bool> {
        self.objects.stroke_transform_affects_stroke(local_id)
    }

    pub(crate) fn stroke_thickness(&self, local_id: usize) -> Option<f32> {
        self.objects.stroke_thickness(local_id)
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_bool_property(&mut self, local_id: usize, property_key: u16, value: bool) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedBool")
            && property_key_for_name("NestedBool", "nestedValue") == Some(property_key)
        {
            return self.set_nested_bool_value(local_id, value);
        }
        if !self
            .objects
            .set_bool_property(local_id, property_key, value)
        {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        let affects_effect_path = property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        );
        self.mark_runtime_shape_property_changed(local_id, affects_effect_path);
        self.apply_bool_property_changed(local_id, property_key, value);
        true
    }

    pub(crate) fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.objects.uint_property(local_id, property_key)
    }

    pub(crate) fn resolved_image_asset_global(
        &self,
        local_id: Option<usize>,
        authored_asset_global: Option<u32>,
    ) -> Option<u32> {
        local_id
            .and_then(|local_id| self.image_asset_overrides.get(&local_id))
            .copied()
            .unwrap_or(authored_asset_global)
    }

    pub(crate) fn set_image_asset_override(
        &mut self,
        local_id: usize,
        asset_global: Option<u32>,
    ) -> bool {
        if self.image_asset_overrides.get(&local_id) == Some(&asset_global) {
            return false;
        }
        self.image_asset_overrides.insert(local_id, asset_global);
        self.mark_artboard_data_bind_work_dirty();
        self.mark_changed();
        self.mark_prepared_changed();
        true
    }

    pub(crate) fn text_style_font_override(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        self.text_style_font_overrides.get(&local_id)
    }

    pub(crate) fn set_text_style_font_override(
        &mut self,
        local_id: usize,
        value: RuntimeFontAssetValue,
    ) -> bool {
        let unchanged = self
            .text_style_font_overrides
            .get(&local_id)
            .is_some_and(|current| {
                current.file_asset_index() == value.file_asset_index()
                    && match (current.live_font_bytes_arc(), value.live_font_bytes_arc()) {
                        (Some(current), Some(next)) => {
                            Arc::ptr_eq(current, next) || current.as_ref() == next.as_ref()
                        }
                        (None, None) => true,
                        _ => false,
                    }
            });
        if unchanged {
            return false;
        }
        self.text_style_font_overrides.insert(local_id, value);
        self.mark_text_style_shape_dirty(local_id);
        self.mark_path_changed();
        self.mark_layout_changed();
        true
    }

    /// Reads one typed double property from the live object arena.
    ///
    /// Returns `None` when either the local object or a double property with
    /// this key does not exist. Schema defaults are already materialized in
    /// the object arena, so a matching property returns its current value
    /// even when the source record omitted that default.
    pub fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedNumber")
            && property_key_for_name("NestedNumber", "nestedValue") == Some(property_key)
        {
            // `NestedNumber::nestedValue()` reads the live SMINumber
            // occurrence and returns 0 when it cannot be resolved
            // (`src/animation/nested_number.cpp:36-48`).
            return Some(self.nested_number_value(local_id).unwrap_or(0.0));
        }
        self.has_legacy_image_layout_scales
            .get()
            .then(|| self.legacy_image_layout_public_scale(local_id, property_key))
            .flatten()
            .or_else(|| runtime_scroll_double_property(self, local_id, property_key))
            .or_else(|| self.objects.double_property(local_id, property_key))
    }

    /// Mirrors the legacy branch of C++ `Image::updateImageScale()`. Files
    /// before 7.2 expose the layout fit through public scale fields; a later
    /// user/animation write wins until another fit-driving input changes.
    pub(crate) fn resolve_legacy_image_layout_scale(
        &self,
        local_id: usize,
        key: RuntimeLegacyImageLayoutScaleKey,
        fit_scale_x: f32,
        fit_scale_y: f32,
    ) -> (f32, f32) {
        self.has_legacy_image_layout_scales.set(true);
        let mut states = self.legacy_image_layout_scales.borrow_mut();
        let state = states
            .entry(local_id)
            .and_modify(|state| {
                if state.key != key {
                    *state = RuntimeLegacyImageLayoutScaleState {
                        key,
                        scale_x: fit_scale_x,
                        scale_y: fit_scale_y,
                        user_scale_x: false,
                        user_scale_y: false,
                    };
                }
            })
            .or_insert(RuntimeLegacyImageLayoutScaleState {
                key,
                scale_x: fit_scale_x,
                scale_y: fit_scale_y,
                user_scale_x: false,
                user_scale_y: false,
            });
        let authored_scale_x = property_key_for_name("Node", "scaleX")
            .and_then(|property_key| self.objects.double_property(local_id, property_key))
            .unwrap_or(1.0);
        let authored_scale_y = property_key_for_name("Node", "scaleY")
            .and_then(|property_key| self.objects.double_property(local_id, property_key))
            .unwrap_or(1.0);
        (
            if state.user_scale_x {
                authored_scale_x
            } else {
                state.scale_x
            },
            if state.user_scale_y {
                authored_scale_y
            } else {
                state.scale_y
            },
        )
    }

    fn legacy_image_layout_public_scale(&self, local_id: usize, property_key: u16) -> Option<f32> {
        let axis_x = legacy_image_layout_scale_axis(property_key)?;
        let states = self.legacy_image_layout_scales.borrow();
        let state = states.get(&local_id)?;
        match (axis_x, state.user_scale_x, state.user_scale_y) {
            (true, false, _) => Some(state.scale_x),
            (false, _, false) => Some(state.scale_y),
            _ => None,
        }
    }

    fn has_legacy_image_layout_scale(&self, local_id: usize, property_key: u16) -> bool {
        self.has_legacy_image_layout_scales.get()
            && legacy_image_layout_scale_axis(property_key).is_some()
            && self
                .legacy_image_layout_scales
                .borrow()
                .contains_key(&local_id)
    }

    fn mark_legacy_image_layout_scale_written(&self, local_id: usize, property_key: u16) -> bool {
        if !self.has_legacy_image_layout_scales.get() {
            return false;
        }
        let Some(axis_x) = legacy_image_layout_scale_axis(property_key) else {
            return false;
        };
        let mut states = self.legacy_image_layout_scales.borrow_mut();
        let Some(state) = states.get_mut(&local_id) else {
            return false;
        };
        if axis_x {
            let changed = !state.user_scale_x;
            state.user_scale_x = true;
            changed
        } else {
            let changed = !state.user_scale_y;
            state.user_scale_y = true;
            changed
        }
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_double_property(&mut self, local_id: usize, property_key: u16, value: f32) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedNumber")
            && property_key_for_name("NestedNumber", "nestedValue") == Some(property_key)
        {
            return self.set_nested_number_value(local_id, value);
        }
        if let Some(changed) =
            set_runtime_scroll_double_property(self, local_id, property_key, value)
        {
            if !changed {
                return false;
            }
            let _ = self
                .objects
                .set_generated_double_property(local_id, property_key, value);
            return self.after_double_property_set(local_id, property_key, value);
        }
        if self.has_legacy_image_layout_scale(local_id, property_key)
            && self.double_property(local_id, property_key) == Some(value)
        {
            return false;
        }
        let object_changed = self
            .objects
            .set_double_property(local_id, property_key, value);
        let legacy_scale_changed =
            self.mark_legacy_image_layout_scale_written(local_id, property_key);
        if !object_changed && !legacy_scale_changed {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    pub(crate) fn set_keyed_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedNumber")
            && property_key_for_name("NestedNumber", "nestedValue") == Some(property_key)
        {
            return self.set_nested_number_value(local_id, value);
        }
        if let Some(changed) =
            set_runtime_scroll_double_property(self, local_id, property_key, value)
        {
            if !changed {
                return false;
            }
            let _ = self
                .objects
                .set_generated_double_property(local_id, property_key, value);
            return self.after_double_property_set(local_id, property_key, value);
        }
        if self.has_legacy_image_layout_scale(local_id, property_key)
            && self.double_property(local_id, property_key) == Some(value)
        {
            return false;
        }
        let object_changed =
            self.objects
                .set_generated_double_property(local_id, property_key, value);
        let legacy_scale_changed =
            self.mark_legacy_image_layout_scale_written(local_id, property_key);
        if !object_changed && !legacy_scale_changed {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    fn after_double_property_set(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        // Generated C++ setters assign backing storage, run the concrete
        // changed callback, then notify property listeners. Transform dirt
        // must therefore be visible before a DataBind observes the write.
        self.apply_double_property_changed(local_id, property_key, value);
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        let affects_effect_path = property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        );
        self.mark_runtime_shape_property_changed(local_id, affects_effect_path);
        true
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_uint_property(&mut self, local_id: usize, property_key: u16, value: u64) -> bool {
        if !self
            .objects
            .set_uint_property(local_id, property_key, value)
        {
            return false;
        }
        // Generated uint setters follow the same backing → concrete callback
        // → notification order as doubles. DistanceConstraint::modeValue is
        // the A4 positive callback that makes this ordering observable.
        self.apply_uint_property_changed(local_id, property_key);
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        let affects_effect_path = property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        );
        self.mark_runtime_shape_property_changed(local_id, affects_effect_path);
        true
    }

    pub(crate) fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        self.objects.string_property(local_id, property_key)
    }

    pub(crate) fn text_list_runs(&self, text_local: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.artboard_text_list_bindings
            .iter()
            .find(|binding| binding.target_local_id() == text_local)
            .map(RuntimeArtboardTextListBindingInstance::text_runs)
            .unwrap_or_default()
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_string_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: Vec<u8>,
    ) -> bool {
        if !self
            .objects
            .set_string_property(local_id, property_key, value)
        {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        self.apply_string_property_changed(local_id, property_key);
        true
    }

    /// Set the first root-artboard `TextValueRun` with the exact authored
    /// component name. Resolution follows component/local order and does not
    /// traverse nested artboards.
    ///
    /// `None` means no matching root text run exists. `Some(false)` means the
    /// existing run already contains `value`; `Some(true)` means it changed.
    pub fn set_root_text_value_run(&mut self, name: &str, value: Vec<u8>) -> Option<bool> {
        let text_property_key = property_key_for_name("TextValueRun", "text")?;
        let local_id = self.root_text_value_run_local_id(name)?;
        if self.string_property(local_id, text_property_key) == Some(value.as_slice()) {
            return Some(false);
        }
        Some(self.set_string_property(local_id, text_property_key, value))
    }

    /// Whether this root artboard contains an exactly named `TextValueRun`.
    /// Nested-artboard occurrences are deliberately outside this lookup.
    pub fn has_root_text_value_run(&self, name: &str) -> bool {
        self.root_text_value_run_local_id(name).is_some()
    }

    fn root_text_value_run_local_id(&self, name: &str) -> Option<usize> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.type_name == Some("TextValueRun") && slot.name.as_deref() == Some(name)
            })
            .min_by_key(|slot| slot.local_id)
            .map(|slot| slot.local_id)
    }

    pub fn apply_linear_animation(&mut self, index: usize, seconds: f32, mix: f32) -> bool {
        let definitions = self.linear_animations.clone();
        let Some(animation) = definitions.get(index) else {
            return false;
        };
        animation.apply(self, seconds, mix)
    }

    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.linear_animation_instance_with_speed(index, 1.0)
    }

    pub fn linear_animation_instance_with_speed(
        &self,
        index: usize,
        speed_multiplier: f32,
    ) -> Option<LinearAnimationInstance> {
        let animation = self.linear_animation(index)?;
        Some(LinearAnimationInstance::new(
            RuntimeLinearAnimationHandle::new(index),
            animation,
            speed_multiplier,
        ))
    }

    pub fn advance_linear_animation_instance(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(animation) = instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
        else {
            return false;
        };
        instance.advance(animation, elapsed_seconds)
    }

    pub fn advance_linear_animation_instance_with_events(
        &mut self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let (mut changed, keyed_callbacks) = {
            let Some(animation) = instance
                .animation_handle()
                .resolve(&self.linear_animations, &self.empty_linear_animation)
            else {
                return false;
            };
            if !animation.has_keyed_callbacks {
                return instance.advance(animation, elapsed_seconds);
            }
            let mut keyed_callbacks = Vec::new();
            let changed = instance.advance_with_events(
                animation,
                elapsed_seconds,
                reported_events,
                &mut keyed_callbacks,
            );
            (changed, keyed_callbacks)
        };
        for callback in keyed_callbacks {
            changed |= self.apply_keyed_callback(callback);
        }
        changed
    }

    pub fn apply_linear_animation_instance(
        &mut self,
        instance: &LinearAnimationInstance,
        mix: f32,
    ) -> bool {
        instance.apply(self, mix)
    }

    pub fn linear_animation_instance_keep_going(&self, instance: &LinearAnimationInstance) -> bool {
        let Some(animation) = instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
        else {
            return false;
        };
        instance.keep_going(animation)
    }

    pub(crate) fn linear_animation_instance_definition(
        &self,
        instance: &LinearAnimationInstance,
    ) -> Option<&RuntimeLinearAnimation> {
        instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
    }

    pub(crate) fn linear_animation_definition(
        &self,
        handle: RuntimeLinearAnimationHandle,
    ) -> Option<&RuntimeLinearAnimation> {
        handle.resolve(&self.linear_animations, &self.empty_linear_animation)
    }

    pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
        let definitions = Arc::clone(&self.state_machines);
        let state_machine = definitions.get(index)?;
        let mut instance = StateMachineInstance::new(index, state_machine, self);
        // C++ builds inputs, layers, and per-instance data binds before its
        // listener pass creates HitExpandable owners and publishes Shape Path
        // dirt (`state_machine_instance.cpp:1707-1831`; helper `:1651-1661`).
        self.initialize_state_machine_hit_path_flags(state_machine);
        if let Some(context) = self.artboard_owned_view_model_handle.as_ref() {
            instance.bind_owned_view_model_context_handle(context);
            instance.mark_scripted_constructor_context_prebound();
        } else if let Some(data_context) = self.artboard_owned_data_context.as_ref() {
            instance.bind_owned_view_model_data_context(data_context);
            instance.mark_scripted_constructor_context_prebound();
        } else if let Some(context) = self.artboard_owned_view_model_context.as_ref() {
            instance.bind_owned_view_model_contexts(context);
            instance.mark_scripted_constructor_context_prebound();
        }
        Some(instance)
    }

    /// The completed ordered view-model context currently retained by this
    /// artboard, when it was bound through the composite context API.
    pub fn owned_view_model_context(&self) -> Option<&RuntimeOwnedViewModelContext> {
        self.artboard_owned_view_model_context.as_ref()
    }

    /// Resolve a named input on the selected state machine attached to one
    /// exact nested/component-list occurrence.
    pub fn occurrence_state_machine_input(
        &self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
        name: &str,
    ) -> Option<(usize, StateMachineInputKind)> {
        let state_machine = self.occurrence_state_machine(occurrence, state_machine_index)?;
        let input_index = state_machine.input_index_named(name)?;
        Some((input_index, state_machine.input(input_index)?.kind()))
    }

    /// Write a boolean to the selected state machine on one exact retained
    /// nested/component-list occurrence.
    pub fn set_occurrence_state_machine_bool(
        &mut self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
        input_index: usize,
        value: bool,
    ) -> Option<bool> {
        let state_machine = self.occurrence_state_machine_mut(occurrence, state_machine_index)?;
        if state_machine
            .input(input_index)
            .is_none_or(|input| input.kind() != StateMachineInputKind::Bool)
        {
            return None;
        }
        Some(state_machine.set_bool(input_index, value))
    }

    fn occurrence_state_machine(
        &self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
    ) -> Option<&StateMachineInstance> {
        let (last, prefix) = occurrence.split_last()?;
        let mut parent = self;
        for segment in prefix {
            parent = match *segment {
                RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => {
                    parent.nested_artboards.get(&host_local_id)?.child.as_ref()
                }
                RuntimeArtboardOccurrenceSegment::ComponentListItem {
                    host_local_id,
                    item_index,
                    occurrence_identity,
                } => {
                    let item = parent
                        .component_list_items(host_local_id)?
                        .get(item_index)?;
                    if item.occurrence_identity != occurrence_identity {
                        return None;
                    }
                    item.child.as_ref()
                }
            };
        }
        match *last {
            RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => parent
                .nested_artboards
                .get(&host_local_id)?
                .animations
                .iter()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                        occurrence.state_machine().filter(|state_machine| {
                            state_machine.state_machine_index() == state_machine_index
                        })
                    }
                    _ => None,
                }),
            RuntimeArtboardOccurrenceSegment::ComponentListItem {
                host_local_id,
                item_index,
                occurrence_identity,
            } => {
                let item = parent
                    .component_list_items(host_local_id)?
                    .get(item_index)?;
                if item.occurrence_identity != occurrence_identity {
                    return None;
                }
                item.state_machines
                    .iter()
                    .find(|machine| machine.state_machine_index() == state_machine_index)
            }
        }
    }

    fn occurrence_state_machine_mut(
        &mut self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
    ) -> Option<&mut StateMachineInstance> {
        let (last, prefix) = occurrence.split_last()?;
        let mut parent = self;
        for segment in prefix {
            parent = match *segment {
                RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => parent
                    .nested_artboards
                    .get_mut(&host_local_id)?
                    .child
                    .as_mut(),
                RuntimeArtboardOccurrenceSegment::ComponentListItem {
                    host_local_id,
                    item_index,
                    occurrence_identity,
                } => {
                    let item = parent
                        .component_list_items_mut(host_local_id)?
                        .get_mut(item_index)?;
                    if item.occurrence_identity != occurrence_identity {
                        return None;
                    }
                    item.child.as_mut()
                }
            };
        }
        match *last {
            RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => parent
                .nested_artboards
                .get_mut(&host_local_id)?
                .animations
                .iter_mut()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                        occurrence.state_machine_mut().filter(|state_machine| {
                            state_machine.state_machine_index() == state_machine_index
                        })
                    }
                    _ => None,
                }),
            RuntimeArtboardOccurrenceSegment::ComponentListItem {
                host_local_id,
                item_index,
                occurrence_identity,
            } => {
                let item = parent
                    .component_list_items_mut(host_local_id)?
                    .get_mut(item_index)?;
                if item.occurrence_identity != occurrence_identity {
                    return None;
                }
                item.state_machines
                    .iter_mut()
                    .find(|machine| machine.state_machine_index() == state_machine_index)
            }
        }
    }

    /// Ported from C++ `src/artboard_component_list.cpp::updateList`,
    /// `findArtboard`, and `createArtboardAt`.
    pub(crate) fn sync_component_list_items(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        contexts: Vec<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(build_context) = self.build_context.clone() else {
            return false;
        };
        let Some(parent_graph) = build_context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)
        else {
            return false;
        };
        let Some(component_list) = parent_graph
            .component_lists
            .iter()
            .find(|list| list.local_id == list_local_id)
        else {
            return false;
        };

        let entries = self
            .component_list_state(list_local_id)
            .and_then(|list| list.source.as_ref())
            .map(|source| source.item_entries_with_logical_indices(file))
            .unwrap_or_else(|| {
                contexts
                    .into_iter()
                    .enumerate()
                    .map(|(index, instance)| {
                        set_component_list_item_index(file, &mut instance.borrow_mut(), index);
                        let occurrence_identity = instance.borrow().instance_identity();
                        RuntimeOwnedViewModelListItemEntry {
                            // NumberToList owns stable, unique generated VMIs.
                            occurrence_identity,
                            instance,
                        }
                    })
                    .collect()
            });
        let resolve_map_rule = |context: &RuntimeOwnedViewModelInstance| {
            let view_model_index = context.view_model_index();
            component_list
                .map_rules
                .iter()
                .find(|rule| rule.view_model_id == view_model_index as i64)
        };
        let resolve_child_graph = |context: &RuntimeOwnedViewModelInstance| {
            let view_model_index = context.view_model_index();
            let mapped_index =
                resolve_map_rule(context).and_then(|rule| usize::try_from(rule.artboard_id).ok());
            mapped_index
                .and_then(|index| build_context.artboards.get(index))
                .or_else(|| {
                    build_context.artboards.iter().find(|graph| {
                        file.object(graph.global_id as usize)
                            .and_then(|artboard| artboard.uint_property("viewModelId"))
                            .and_then(|value| usize::try_from(value).ok())
                            == Some(view_model_index)
                    })
                })
        };

        let previous_logical = self
            .component_list_state_mut(list_local_id)
            .map(|list| std::mem::take(&mut list.logical_items))
            .unwrap_or_default();
        let mut logical_items = Vec::with_capacity(entries.len());
        for entry in entries {
            let mapped_artboard_global =
                resolve_child_graph(&entry.instance.borrow()).map(|graph| graph.global_id);
            let previous = previous_logical.iter().find(|item| {
                item.occurrence_identity == entry.occurrence_identity
                    && item.mapped_artboard_global == mapped_artboard_global
            });
            let settled_size = self
                .component_list_items(list_local_id)
                .and_then(|items| {
                    items.iter().find(|item| {
                        item.occurrence_identity == entry.occurrence_identity
                            && Some(item.child.graph_global_id) == mapped_artboard_global
                    })
                })
                .and_then(|item| item.settled_layout_size.get());
            let size = settled_size
                .or_else(|| previous.map(|item| item.size))
                .unwrap_or_else(|| {
                    mapped_artboard_global
                        .and_then(|global_id| file.object(global_id as usize))
                        .map(|artboard| {
                            (
                                artboard.double_property("width").unwrap_or(0.0),
                                artboard.double_property("height").unwrap_or(0.0),
                            )
                        })
                        .unwrap_or((0.0, 0.0))
                });
            logical_items.push(RuntimeComponentListLogicalItem {
                occurrence_identity: entry.occurrence_identity,
                context: entry.instance,
                size,
                mapped_artboard_global,
            });
        }
        let logical_changed = previous_logical.len() != logical_items.len()
            || previous_logical
                .iter()
                .zip(&logical_items)
                .any(|(before, after)| {
                    before.occurrence_identity != after.occurrence_identity
                        || before.mapped_artboard_global != after.mapped_artboard_global
                        || before.size != after.size
                });
        let desired = if component_list_virtualization(self, list_local_id).is_some() {
            // A virtualized list's mounted occurrences are the retained
            // interface state. ScrollVirtualizer adds/removes them directly;
            // list synchronization only preserves still-live identities.
            self.component_list_items(list_local_id)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| item.logical_index)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            (0..logical_items.len()).collect()
        };
        let desired = desired
            .into_iter()
            .filter(|index| {
                logical_items
                    .get(*index)
                    .is_some_and(|item| item.mapped_artboard_global.is_some())
            })
            .collect::<Vec<_>>();
        let existing_matches = self
            .component_list_items(list_local_id)
            .is_some_and(|existing| {
                existing.len() == desired.len()
                    && existing.iter().zip(&desired).all(|(item, index)| {
                        let logical = &logical_items[*index];
                        item.logical_index == *index
                            && item.occurrence_identity == logical.occurrence_identity
                            && item.context_is_current(&logical.context)
                    })
            });
        if existing_matches {
            if let Some(list) = self.component_list_state_mut(list_local_id) {
                list.logical_items = logical_items;
            }
            if logical_changed {
                if let Some(list) = self.component_list_state(list_local_id) {
                    *list.order_cache.borrow_mut() = Default::default();
                }
                self.mark_layout_changed();
                self.mark_prepared_changed();
            }
            return logical_changed;
        }

        // C++ keys mounted artboards/state machines by the list-item wrapper,
        // not by its VMI. Preserve overlapping wrapper occurrences across
        // reorder and virtual-window changes.
        let previous_items = self
            .component_list_state_mut(list_local_id)
            .map(|list| std::mem::take(&mut list.items))
            .unwrap_or_default();
        let mut reusable_items = previous_items.into_iter().map(Some).collect::<Vec<_>>();
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let mut item_context_changed = false;
        let mut items = Vec::with_capacity(desired.len());
        for logical_index in desired {
            let logical = logical_items[logical_index].clone();
            let context = logical.context.clone();
            if let Some(existing_index) = reusable_items.iter().position(|candidate| {
                candidate.as_ref().is_some_and(|item| {
                    item.occurrence_identity == logical.occurrence_identity
                        && Some(item.child.graph_global_id) == logical.mapped_artboard_global
                })
            }) {
                let mut item = reusable_items[existing_index]
                    .take()
                    .expect("component-list identity match must retain an item");
                if !item.context_is_current(&context) {
                    item.context = context.clone();
                    item.context_rebind_sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    context.add_rebind_dependent(&item.context_rebind_sink);
                    item.draw_index_sink = component_list_draw_index_sink(file, &context);
                    let child_data_context = RuntimeOwnedDataContext::with_local_handles(
                        [context.clone()],
                        Some(&parent_data_context),
                    );
                    item.child.bind_owned_view_model_artboard_data_context(
                        file,
                        &child_data_context,
                        true,
                        true,
                    );
                    // The row-first DataContext bind clears the
                    // public facade while doing so. Restore the facade after
                    // the bind so scripting observes the same row main that
                    // C++ installs in `ArtboardComponentList::bindArtboard`
                    // (`artboard_component_list.cpp:1530-1543`).
                    item.child.artboard_owned_view_model_context = Some(
                        RuntimeOwnedViewModelContext::from_main_handle(context.clone()),
                    );
                    for state_machine in &mut item.state_machines {
                        if state_machine.bind_owned_view_model_data_context(&child_data_context) {
                            state_machine.advance_data_context();
                        }
                    }
                    item.child.advance_artboard_data_binds_with_elapsed(0.0);
                    item.child.update_pass();
                    item.consume_context_rebind_dirt();
                    item_context_changed = true;
                }
                item.logical_index = logical_index;
                items.push(item);
                continue;
            }
            if let Some(item) = self.create_component_list_item_instance(
                file,
                list_local_id,
                logical_index,
                logical,
            ) {
                items.push(item);
            }
        }
        if let Some(list) = self.component_list_state_mut(list_local_id) {
            list.item_transforms = items.iter().map(|item| item.transform).collect();
            list.logical_items = logical_items;
            list.items = items;
        }
        if let Some(list) = self.component_list_state(list_local_id) {
            *list.order_cache.borrow_mut() = Default::default();
        }
        let changed = !existing_matches || logical_changed || item_context_changed;
        if changed {
            self.mark_nested_structure_changed();
            self.mark_layout_changed();
            self.mark_prepared_changed();
        }
        changed
    }

    /// Mount one retained `VirtualizingComponent` item.
    ///
    /// The FL-A adapter deliberately calls this one index at a time, matching
    /// `ScrollVirtualizer::virtualize` → `VirtualizingComponent::addVirtualizable`
    /// (`scroll_virtualizer.cpp:244-293`). Artboard/state-machine pooling and
    /// recorder restoration remain owned by the later
    /// `src/artboard_component_list.cpp` FL-D row.
    fn create_component_list_item_instance(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        logical_index: usize,
        logical: RuntimeComponentListLogicalItem,
    ) -> Option<RuntimeComponentListItemInstance> {
        let build_context = self.build_context.clone()?;
        let parent_graph = build_context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)?;
        let component_list = parent_graph
            .component_lists
            .iter()
            .find(|list| list.local_id == list_local_id)?;
        let child_graph = logical.mapped_artboard_global.and_then(|global_id| {
            build_context
                .artboards
                .iter()
                .find(|graph| graph.global_id == global_id)
        })?;
        let context = logical.context;
        let mut visiting = BTreeSet::from([self.graph_global_id]);
        let mut child = ArtboardInstance::from_graph_inner(
            file,
            child_graph,
            &build_context.artboards,
            &mut visiting,
            Some(build_context.clone()),
            false,
        )
        .ok()?;
        child.set_frame_origin(false);
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let child_data_context = RuntimeOwnedDataContext::with_local_handles(
            [context.clone()],
            Some(&parent_data_context),
        );
        child.bind_owned_view_model_artboard_data_context(file, &child_data_context, true, true);
        child.artboard_owned_view_model_context = Some(
            RuntimeOwnedViewModelContext::from_main_handle(context.clone()),
        );
        let selected_machine_indices = component_list
            .map_rules
            .iter()
            .find(|rule| rule.view_model_id == context.borrow().view_model_index() as i64)
            .filter(|rule| !rule.state_machine_ids.is_empty())
            .map(|rule| rule.state_machine_ids.clone())
            .unwrap_or_else(|| {
                let default_state_machine_index = file
                    .object(child_graph.global_id as usize)
                    .and_then(|artboard| artboard.uint_property("defaultStateMachineId"));
                vec![component_list_default_state_machine_index(
                    default_state_machine_index,
                    child.state_machines.len(),
                )]
            });
        let mut state_machines = Vec::with_capacity(selected_machine_indices.len());
        for state_machine_index in selected_machine_indices {
            let Some(mut state_machine) = child.state_machine_instance(state_machine_index) else {
                continue;
            };
            state_machine.bind_owned_view_model_data_context(&child_data_context);
            // C++ `ArtboardComponentList::linkStateMachineToArtboard` installs
            // the row DataContext and immediately runs `updateDataBinds(false)`
            // before the first state advance
            // (`artboard_component_list.cpp:1492-1543`).
            state_machine.advance_data_context();
            state_machines.push(state_machine);
        }
        if let Some(parent_focus) = self.external_focus_domain.as_ref() {
            let child_identity = child.instance_identity();
            for state_machine in &mut state_machines {
                // C++ `ArtboardComponentList::linkStateMachineToArtboard`
                // installs the parent Artboard's FocusManager immediately
                // after the row machine is created and data-bound
                // (`artboard_component_list.cpp:641-678`).
                state_machine.install_external_focus(parent_focus, child_identity);
            }
            child.install_external_focus_domain(parent_focus);
        }
        child.advance_artboard_data_binds_with_elapsed(0.0);
        child.update_pass();
        let context_rebind_sink = crate::view_model_cell::RuntimeCellDirtSink::new();
        context.add_rebind_dependent(&context_rebind_sink);
        let draw_index_sink = component_list_draw_index_sink(file, &context);
        Some(RuntimeComponentListItemInstance {
            child: Box::new(child),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines,
            context_rebind_sink,
            draw_index_sink,
            context,
            occurrence_identity: logical.occurrence_identity,
            logical_index,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            // Render caches outlive list topology changes. Seed each row with
            // stable wrapper identity so a same-length replacement cannot
            // reuse the prior occupant's paint cache.
            render_cache_revision: logical.occurrence_identity,
        })
    }

    /// Literal Rust surface for the pinned `VirtualizingComponent` interface
    /// (`include/rive/virtualizing_component.hpp:24-39`).
    pub(crate) fn virtualizing_component_has_item(
        &self,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        self.component_list_items(list_local_id)
            .is_some_and(|items| items.iter().any(|item| item.logical_index == logical_index))
    }

    pub(crate) fn add_component_list_virtualizable(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        if self.virtualizing_component_has_item(list_local_id, logical_index) {
            return false;
        }
        let Some(logical) = self
            .component_list_state(list_local_id)
            .and_then(|list| list.logical_items.get(logical_index))
            .cloned()
        else {
            return false;
        };
        let Some(item) =
            self.create_component_list_item_instance(file, list_local_id, logical_index, logical)
        else {
            return false;
        };
        let transform = item.transform;
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return false;
        };
        list.items.push(item);
        list.item_transforms.push(transform);
        *list.order_cache.borrow_mut() = Default::default();
        self.mark_nested_structure_changed();
        self.mark_layout_changed();
        self.mark_prepared_changed();
        true
    }

    pub(crate) fn remove_component_list_virtualizable(
        &mut self,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return false;
        };
        let Some(index) = list
            .items
            .iter()
            .position(|item| item.logical_index == logical_index)
        else {
            return false;
        };
        // State machines precede the child Artboard in the item struct, so
        // dropping the mounted item preserves the C++ teardown order. FL-D
        // replaces this drop with the C++ resource pools.
        list.items.remove(index);
        if index < list.item_transforms.len() {
            list.item_transforms.remove(index);
        }
        *list.order_cache.borrow_mut() = Default::default();
        self.mark_nested_structure_changed();
        self.mark_layout_changed();
        self.mark_prepared_changed();
        true
    }

    pub(crate) fn set_component_list_visible_indices(
        &mut self,
        list_local_id: usize,
        start: i32,
        end: i32,
    ) {
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return;
        };
        list.visible_start = start;
        list.visible_end = end;
        *list.order_cache.borrow_mut() = Default::default();
    }

    pub(crate) fn set_component_list_virtualizable_position(
        &mut self,
        list_local_id: usize,
        logical_index: usize,
        position: (f32, f32),
    ) {
        let uses_layout = self
            .component(list_local_id)
            .and_then(|component| component.parent)
            .and_then(|parent| self.objects.component(parent))
            .is_some_and(|parent| parent.concrete.layout.is_some());
        let origin = self
            .component_list_state(list_local_id)
            .and_then(|list| {
                list.items
                    .iter()
                    .find(|item| item.logical_index == logical_index)
            })
            .map(|item| {
                let size = crate::draw::runtime_component_list_item_layout_size(item);
                let origin = if uses_layout {
                    (size.0 * item.child.origin_x, size.1 * item.child.origin_y)
                } else {
                    (0.0, 0.0)
                };
                origin
            })
            .unwrap_or((0.0, 0.0));
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return;
        };
        let Some(item_index) = list
            .items
            .iter()
            .position(|item| item.logical_index == logical_index)
        else {
            return;
        };
        // `Artboard::origin()` is negative when mounted with FrameOrigin off,
        // so C++ `position - origin` adds the mounted layout origin here.
        // Retain the final owner transform at the same mutation site
        // (`artboard_component_list.cpp:1728-1740`;
        // `artboard.cpp:1729-1734`).
        let transform = Mat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            position.0 + origin.0,
            position.1 + origin.1,
        ]);
        list.items[item_index].transform = transform;
        if let Some(retained) = list.item_transforms.get_mut(item_index) {
            *retained = transform;
        }
    }

    pub(crate) fn component_list_virtualizable_layout_position(
        &self,
        list_local_id: usize,
        logical_index: usize,
    ) -> (f32, f32) {
        self.component_list_state(list_local_id)
            .and_then(|list| {
                list.items
                    .iter()
                    .find(|item| item.logical_index == logical_index)
            })
            .and_then(|item| {
                item.child
                    .component(0)
                    .and_then(|component| component.concrete.layout.as_ref())
                    .map(|layout| layout.position())
            })
            .unwrap_or((0.0, 0.0))
    }

    pub(crate) fn component_list_virtualizable_changed(&mut self, list_local_id: usize) {
        if let Some(list) = self.component_list_state(list_local_id) {
            *list.order_cache.borrow_mut() = Default::default();
        }
    }

    /// Literal owner-level `ArtboardComponentList::updateLayoutBounds` tail:
    /// update mounted row bounds once, write those sizes into the full logical
    /// vector, compute `m_layoutSize`, then force the retained virtualizer
    /// (`artboard_component_list.cpp:245-260,1758-1788`).
    fn update_component_list_layout_bounds(&mut self, root_transform: Mat2D) -> bool {
        // C++ deliberately reaches `computeLayoutBounds()` even when the
        // mounted-row loop is empty. That owner tail recomputes the logical
        // list size and force-settles its ScrollVirtualizer; returning early
        // would leave an empty-mounted virtualized list one settlement behind
        // (`artboard_component_list.cpp:245-260,1758-1788`).
        let mut assigned_bounds = self.runtime_component_list_assigned_layout_bounds();
        for list_local in self.component_list_locals() {
            let Some(items) = self.component_list_items(list_local) else {
                continue;
            };
            assigned_bounds.entry(list_local).or_insert_with(|| {
                items
                    .iter()
                    .map(|item| {
                        let (width, height) = runtime_component_list_item_layout_size(item);
                        RuntimeLayoutBounds {
                            x: 0.0,
                            y: 0.0,
                            width,
                            height,
                        }
                    })
                    .collect()
            });
        }
        let mut changed = false;
        for list_index in 0..self.component_list_count() {
            let Some(list_local) = self.component_list_local_at(list_index) else {
                continue;
            };
            let bounds = assigned_bounds.remove(&list_local).unwrap_or_default();
            let mut measured_sizes = Vec::new();
            let virtualized =
                crate::constraints::component_list_virtualization(self, list_local).is_some();
            if let Some(items) = self.component_list_items_mut(list_local) {
                for (item, bounds) in items.iter_mut().zip(bounds) {
                    let previous_size = runtime_component_list_item_layout_size(item);
                    changed |=
                        runtime_apply_component_list_item_layout_bounds(&mut item.child, bounds);
                    if item.settled_layout_size.get().is_none() {
                        // C++ marks the row as newly hosted before its first
                        // parent-owned `updateLayoutBounds`; the later
                        // component traversal performs `updatePass(false)`.
                        item.child.added_to_host();
                    }
                    if let Some(layout) = item
                        .child
                        .component(0)
                        .and_then(|component| component.concrete.layout.as_ref())
                    {
                        // The mounted root Yoga node remains owned by the
                        // hosting layout. Retain its parent-local location
                        // before the later child component traversal, which
                        // must not solve the occurrence as a standalone root
                        // (`artboard_component_list.cpp:220-229`;
                        // `scroll_virtualizer.cpp:269-291`).
                        layout.retain_bounds(bounds.x, bounds.y, bounds.width, bounds.height);
                    }
                    // The transferred root Yoga node is the size owner after
                    // the parent solve. Do not immediately run a standalone
                    // child solve and overwrite its parent-local location.
                    let measured = (bounds.width, bounds.height);
                    if virtualized && measured != previous_size {
                        // `setVirtualizablePosition` stores `position -
                        // artboard->origin()`. A same-frame layout-width
                        // change therefore moves the retained translation by
                        // exactly the origin delta before draw
                        // (`artboard_component_list.cpp:1728-1740`;
                        // `artboard.cpp:1729-1734`).
                        item.transform.0[4] += (measured.0 - previous_size.0) * item.child.origin_x;
                        item.transform.0[5] += (measured.1 - previous_size.1) * item.child.origin_y;
                    }
                    item.settled_layout_size.set(Some(measured));
                    measured_sizes.push((item.logical_index, measured));
                }
            }
            let transforms = runtime_component_list_item_base_transforms(self, list_local);
            if let Some(list) = self.component_list_state_mut(list_local)
                && list.item_transforms != transforms
            {
                list.item_transforms = transforms;
                changed = true;
            }
            if let Some(list_component) = self.component_handle(list_local) {
                // This Rust tail performs the parent-owned Yoga solve after
                // the dependency walk. Reapply the same list constraints that
                // C++ runs immediately after
                // `updateArtboardsWorldTransform`; otherwise the refreshed
                // hosted-layout bases overwrite FollowPath's retained result
                // (`artboard_component_list.cpp:1300-1358`).
                changed |= crate::constraints::apply_list_constraints(self, list_component);
            }

            let style_local = self.layout_component_style_local(list_local);
            let is_row = style_local
                .and_then(|local| {
                    property_key_for_name("LayoutComponentStyle", "flexDirectionValue")
                        .and_then(|key| self.uint_property(local, key))
                })
                .map(|direction| matches!(direction, 2 | 3))
                .unwrap_or(true);
            let gap_property = if is_row {
                "gapHorizontal"
            } else {
                "gapVertical"
            };
            let gap = style_local
                .and_then(|local| {
                    property_key_for_name("LayoutComponentStyle", gap_property)
                        .and_then(|key| self.double_property(local, key))
                })
                .unwrap_or(0.0);
            let scroll_constraint = self.component_list_state(list_local).and_then(|list| {
                list.layout_constraints
                    .iter()
                    .copied()
                    .find(|constraint| self.component_at(*constraint).concrete.scroll.is_some())
            });
            if let Some(list) = self.component_list_state_mut(list_local) {
                for (logical_index, size) in measured_sizes {
                    if let Some(logical) = list.logical_items.get_mut(logical_index) {
                        changed |= logical.size != size;
                        logical.size = size;
                    }
                }
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                let item_count = list.logical_items.len();
                for (index, item) in list.logical_items.iter().enumerate() {
                    let real_gap = if index + 1 == item_count { 0.0 } else { gap };
                    if is_row {
                        width += item.size.0 + real_gap;
                        height = height.max(item.size.1);
                    } else {
                        width = width.max(item.size.0);
                        height += item.size.1 + real_gap;
                    }
                }
                changed |= list.layout_size != (width, height);
                list.layout_size = (width, height);
            }
            if let Some(constraint) = scroll_constraint {
                changed |= crate::constraints::constrain_scroll_virtualizer(self, constraint, true);
            }
            let roots = self
                .runtime_component_list_child_root_transforms(root_transform)
                .remove(&list_local)
                .unwrap_or_default();
            if let Some(items) = self.component_list_items_mut(list_local) {
                for (item_index, item) in items.iter_mut().enumerate() {
                    let child_root_transform =
                        roots.get(item_index).copied().unwrap_or(root_transform);
                    let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
                    changed |= item
                        .child
                        .update_pass_with_script_mode(&mut script_mode, child_root_transform);
                }
            }
        }
        changed
    }

    pub fn advance_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_state_machine_instance_with_context(instance, elapsed_seconds, None)
    }

    fn advance_state_machine_instance_with_context(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let definitions = self.state_machine_definition_owner(instance);
        let Some(state_machine) = definitions.get(instance.state_machine_index()) else {
            return false;
        };
        let mut owned_context = owned_context;
        // C++ `applyEvents()` consumes only reports not delivered to
        // listeners yet, at new-frame start, and loops chained reports before
        // layer advance (`state_machine_instance.cpp:2320-2343,2555-2565`).
        // Each C++ iteration clears the batch before notifying it. Drain the
        // complete prefix consumed by the bounded chained loop; reports
        // created later by the layer pass remain pending for the next frame's
        // `applyEvents()`.
        let next_event_index = instance.next_unapplied_reported_event_index();
        instance.apply_local_event_listeners(self, next_event_index, owned_context.as_deref_mut());
        let applied_report_count = instance.next_unapplied_reported_event_index();
        instance.discard_reported_event_prefix(applied_report_count);

        match owned_context.as_deref_mut() {
            Some(context) => instance.advance_with_owned_view_model_context(
                self,
                state_machine,
                elapsed_seconds,
                context,
            ),
            None => instance.advance_preserving_reported_events(
                self,
                state_machine,
                elapsed_seconds,
                None,
            ),
        }
    }

    fn advance_state_machine_instance_preserving_events(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let definitions = self.state_machine_definition_owner(instance);
        let Some(state_machine) = definitions.get(instance.state_machine_index()) else {
            return false;
        };
        // This is C++'s `newFrame=false` follow-up after direct nested-event
        // notification. It advances layers but does not call `applyEvents`;
        // reports created here remain queued for the next ordinary frame
        // (`state_machine_instance.cpp:2555-2565`).
        instance.advance_preserving_reported_events(
            self,
            state_machine,
            elapsed_seconds,
            owned_context,
        )
    }

    fn advance_state_machine_instance_after_state_probe(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let definitions = self.state_machine_definition_owner(instance);
        let Some(state_machine) = definitions.get(instance.state_machine_index()) else {
            return false;
        };
        instance.advance_after_state_probe(self, state_machine, elapsed_seconds)
    }

    pub(crate) fn try_change_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
    ) -> bool {
        // Root and component-list machines complete their direct-input
        // transition loop during ordinary advance. Mounted nested machines
        // additionally owe one C++-matching outer-update probe.
        if !state_machine_requires_outer_update_probe(instance) {
            return false;
        }
        self.try_change_state_machine_instance_unconditionally(instance)
    }

    fn try_change_state_machine_instance_unconditionally(
        &mut self,
        instance: &mut StateMachineInstance,
    ) -> bool {
        let definitions = self.state_machine_definition_owner(instance);
        let Some(state_machine) = definitions.get(instance.state_machine_index()) else {
            return false;
        };
        instance.try_change_state(self, state_machine)
    }

    fn state_machine_definition_owner(
        &self,
        instance: &StateMachineInstance,
    ) -> Arc<Vec<RuntimeStateMachine>> {
        instance
            .retained_state_machine_definitions()
            .unwrap_or_else(|| Arc::clone(&self.state_machines))
    }

    /// Advance several state-machine instances on this artboard while
    /// advancing nested artboards only once for the frame.
    ///
    /// Nested events are delivered to each root machine in caller order. A
    /// machine notified by those events is settled once more at zero elapsed
    /// time, matching the single-machine pipeline without multiplying nested
    /// animation time by the number of root machines.
    pub fn advance_state_machine_instances_with_nested(
        &mut self,
        instances: &mut [StateMachineInstance],
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_state_machine_instances_with_nested_context(instances, elapsed_seconds, None)
    }

    pub fn advance_state_machine_instances_with_nested_and_owned_view_model_context(
        &mut self,
        instances: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.advance_state_machine_instances_with_nested_context(
            instances,
            elapsed_seconds,
            Some(context),
        )
    }

    fn advance_state_machine_instances_with_nested_context(
        &mut self,
        instances: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let mut changed = false;
        for instance in instances.iter_mut() {
            changed |= self.advance_state_machine_instance_with_context(
                instance,
                elapsed_seconds,
                owned_context.as_deref_mut(),
            );
        }

        let mut nested_events = Vec::new();
        changed |=
            self.advance_nested_artboards_collect_events(elapsed_seconds, Some(&mut nested_events));
        for instance in instances.iter_mut() {
            let mut notified = false;
            for (host_local, events) in &nested_events {
                notified |= match owned_context.as_deref_mut() {
                    Some(context) => instance.notify_events_with_owned_view_model_context(
                        self,
                        Some(*host_local),
                        events,
                        context,
                    ),
                    None => instance.notify_events(self, Some(*host_local), events),
                };
            }
            if notified {
                changed |= self.advance_state_machine_instance_preserving_events(
                    instance,
                    0.0,
                    owned_context.as_deref_mut(),
                );
            }
        }
        changed
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_nested_artboards_collect_events(elapsed_seconds, None)
    }

    /// Report imported nested-state-machine occurrence ownership and empty
    /// forwarding behavior for pinned-C++ differentials.
    #[doc(hidden)]
    pub fn runtime_nested_state_machine_reports(&self) -> Vec<RuntimeNestedStateMachineReport> {
        self.nested_artboards
            .values()
            .flat_map(|nested| {
                nested
                    .animations
                    .iter()
                    .filter_map(|animation| match animation {
                        RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                            Some(occurrence.empty_contract_report(&nested.child))
                        }
                        RuntimeNestedAnimationInstance::Simple { .. }
                        | RuntimeNestedAnimationInstance::Remap { .. } => None,
                    })
            })
            .collect()
    }

    pub fn try_visit_nested_artboard_instances_mut<E>(
        &mut self,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        self.try_visit_nested_artboard_instances_mut_at_depth(1, visitor)
    }

    fn try_visit_nested_artboard_instances_mut_at_depth<E>(
        &mut self,
        depth: usize,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        for nested in self.nested_artboards.values_mut() {
            visitor(depth, nested.child.graph_global_id, nested.child.as_mut())?;
            nested
                .child
                .try_visit_nested_artboard_instances_mut_at_depth(depth + 1, visitor)?;
        }
        Ok(())
    }

    /// Visit every concrete child artboard occurrence in this runtime tree,
    /// including ordinary nested artboards and component-list item artboards.
    pub fn try_visit_artboard_tree_instances_mut<E>(
        &mut self,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        self.try_visit_artboard_tree_instances_mut_at_depth(1, visitor)
    }

    fn try_visit_artboard_tree_instances_mut_at_depth<E>(
        &mut self,
        depth: usize,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        for nested in self.nested_artboards.values_mut() {
            visitor(depth, nested.child.graph_global_id, nested.child.as_mut())?;
            nested
                .child
                .try_visit_artboard_tree_instances_mut_at_depth(depth.saturating_add(1), visitor)?;
        }
        for list_index in 0..self.component_list_count() {
            let Some(local_id) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for item in items {
                visitor(depth, item.child.graph_global_id, item.child.as_mut())?;
                item.child.try_visit_artboard_tree_instances_mut_at_depth(
                    depth.saturating_add(1),
                    visitor,
                )?;
            }
        }
        Ok(())
    }

    pub fn bind_nested_artboard_owned_context_for_graph(
        &mut self,
        file: &RuntimeFile,
        graph_global_id: u32,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let mut changed = false;
        for nested in self.nested_artboards.values_mut() {
            if nested.child.graph_global_id == graph_global_id {
                let context_chain: [&[usize]; 1] = [&[]];
                changed |=
                    nested.bind_owned_view_model_animation_contexts(file, context, &context_chain);
                changed |= nested
                    .child
                    .bind_owned_view_model_artboard_context(file, context);
            } else {
                changed |= nested.child.bind_nested_artboard_owned_context_for_graph(
                    file,
                    graph_global_id,
                    context,
                );
            }
        }
        changed
    }

    pub fn set_nested_script_owned_context_for_graph(
        &mut self,
        graph_global_id: u32,
        context: RuntimeOwnedViewModelInstance,
    ) {
        self.nested_script_owned_contexts
            .insert(graph_global_id, context);
    }

    pub fn rebind_nested_script_owned_contexts(&mut self, file: &RuntimeFile) -> bool {
        let contexts = self
            .nested_script_owned_contexts
            .iter()
            .map(|(graph_global_id, context)| (*graph_global_id, context.clone()))
            .collect::<Vec<_>>();
        let mut changed = false;
        for (graph_global_id, context) in contexts {
            changed |=
                self.bind_nested_artboard_owned_context_for_graph(file, graph_global_id, &context);
        }
        changed
    }

    pub fn advance_nested_artboards_with_state_machine(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> bool {
        let mut nested_events = Vec::new();
        self.advance_nested_artboards_collect_events(elapsed_seconds, Some(&mut nested_events));
        let mut notified_state_machine = false;
        for (host_local, events) in nested_events {
            notified_state_machine |= state_machine.notify_events(self, Some(host_local), &events);
        }
        notified_state_machine
    }

    /// Advances the complete retained `Artboard::m_advancingComponents`
    /// sequence, including scripted owners in their authored object-order
    /// slots, and only then advances this artboard's DataBinds.
    ///
    /// This is the direct Rust boundary for C++
    /// `Artboard::advanceInternal` (`src/artboard.cpp:1468-1480`).
    pub fn advance_frame_components(&mut self, elapsed_seconds: f32) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::HostOnly;
        self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            &mut script_mode,
            None,
        )
    }

    /// Factory-aware form of [`Self::advance_frame_components`].
    pub fn advance_frame_components_with_factory(
        &mut self,
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::Factory(factory);
        self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            &mut script_mode,
            None,
        )
    }

    pub fn advance_frame_components_with_state_machine(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> Result<bool, ScriptError> {
        let mut nested_events = Vec::new();
        let mut script_mode = RuntimeScriptAdvanceMode::HostOnly;
        self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            &mut script_mode,
            Some(&mut nested_events),
        )?;
        let mut notified_state_machine = false;
        for (host_local, events) in nested_events {
            notified_state_machine |= state_machine.notify_events(self, Some(host_local), &events);
        }
        Ok(notified_state_machine)
    }

    /// Complete factory-bearing frame advance for several root state-machine
    /// occurrences and the one retained mixed-family advancing list.
    ///
    /// Root machines advance in caller order; nested Artboards advance once
    /// from their authored `m_advancingComponents` slot; their reports are
    /// then delivered to each root in caller order. This is the multi-machine
    /// form of the pinned `advanceAndApply`/`Artboard::advanceInternal`
    /// interleave (`state_machine_instance.cpp:2555-2584`;
    /// `artboard.cpp:1463-1480`).
    pub fn advance_frame_components_with_state_machines(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::HostOnly;
        self.advance_frame_components_with_state_machines_and_mode(
            state_machines,
            elapsed_seconds,
            &mut script_mode,
        )
    }

    pub fn advance_frame_components_with_state_machines_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::Factory(factory);
        self.advance_frame_components_with_state_machines_and_mode(
            state_machines,
            elapsed_seconds,
            &mut script_mode,
        )
    }

    fn advance_frame_components_with_state_machines_and_mode(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        for state_machine in state_machines.iter_mut() {
            changed |= self.advance_state_machine_instance(state_machine, elapsed_seconds);
        }

        let mut nested_events = Vec::new();
        changed |= self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            script_mode,
            Some(&mut nested_events),
        )?;
        for state_machine in state_machines.iter_mut() {
            let mut notified = false;
            for (host_local, events) in &nested_events {
                notified |= state_machine.notify_events(self, Some(*host_local), events);
            }
            if notified {
                changed |=
                    self.advance_state_machine_instance_preserving_events(state_machine, 0.0, None);
            }
        }
        Ok(changed)
    }

    fn advance_frame_components_collect_events_with_mode(
        &mut self,
        elapsed_seconds: f32,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> Result<bool, ScriptError> {
        let mut changed = self.advance_retained_components_collect_events_with_scripts(
            elapsed_seconds,
            true,
            script_mode,
            nested_events,
        )?;
        changed |= self.advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        Ok(changed)
    }

    fn advance_nested_artboards_collect_events(
        &mut self,
        elapsed_seconds: f32,
        nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> bool {
        self.advance_retained_components_collect_events(elapsed_seconds, true, nested_events)
    }

    fn advance_retained_components_collect_events(
        &mut self,
        elapsed_seconds: f32,
        new_frame: bool,
        nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> bool {
        let mut script_mode = RuntimeScriptAdvanceMode::Disabled;
        self.advance_retained_components_collect_events_with_scripts(
            elapsed_seconds,
            new_frame,
            &mut script_mode,
            nested_events,
        )
        .expect("disabled script dispatch cannot fail")
    }

    fn advance_retained_components_collect_events_with_scripts(
        &mut self,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> Result<bool, ScriptError> {
        if self.advancing_components.is_empty() {
            return Ok(false);
        }
        let has_nested = self
            .advancing_components
            .iter()
            .any(|entry| entry.kind == AdvancingComponentKind::NestedArtboard);
        let layout_frame =
            (new_frame && has_nested).then(|| self.runtime_nested_artboard_layout_bounds_frame());
        let mut initial_layout_paint_evaluations = BTreeMap::new();
        if let Some(layout_frame) = layout_frame.as_ref() {
            for entry in &self.advancing_components {
                if entry.kind != AdvancingComponentKind::NestedArtboard {
                    continue;
                }
                let host_local = entry.local_id;
                if self
                    .component(host_local)
                    .is_some_and(RuntimeComponent::is_collapsed)
                {
                    continue;
                }
                if self
                    .component(host_local)
                    .is_none_or(|component| component.type_name != "NestedArtboardLayout")
                {
                    continue;
                }
                let Some(nested) = self.nested_artboards.get(&host_local) else {
                    continue;
                };
                if nested.layout_data_transferred
                    || layout_frame
                        .bounds
                        .as_ref()
                        .as_ref()
                        .and_then(|bounds| bounds.get(&host_local))
                        .is_none()
                {
                    continue;
                }
                if nested.initial_layout_paint_frame.borrow().is_none() {
                    initial_layout_paint_evaluations
                        .insert(host_local, nested.child.as_ref().clone());
                }
            }
        }
        let mut changed = false;
        let mut script_host = NoopScriptHost;
        for index in 0..self.advancing_components.len() {
            let entry = self.advancing_components[index];
            changed |= match entry.kind {
                AdvancingComponentKind::NestedArtboard => self.advance_nested_artboard_entry(
                    entry.local_id,
                    elapsed_seconds,
                    new_frame,
                    script_mode,
                    layout_frame.as_ref(),
                    &mut initial_layout_paint_evaluations,
                    nested_events.as_mut().map(|events| &mut **events),
                )?,
                AdvancingComponentKind::ArtboardComponentList => self
                    .advance_component_list_entry(
                        entry.local_id,
                        elapsed_seconds,
                        new_frame,
                        script_mode,
                    )?,
                AdvancingComponentKind::ScrollConstraint => {
                    entry.component.is_some_and(|constraint| {
                        crate::constraints::advance_scroll_constraint(
                            self,
                            constraint,
                            elapsed_seconds,
                            true,
                            new_frame,
                        )
                    })
                }
                AdvancingComponentKind::ScriptedDataConverter => self
                    .advance_scripted_data_converter_entry(entry.local_id, elapsed_seconds, true),
                AdvancingComponentKind::LayoutComponent => {
                    self.advance_layout_component_entry(entry, elapsed_seconds, new_frame)
                }
                AdvancingComponentKind::TextInput => {
                    self.advance_text_input_entry(entry, elapsed_seconds)
                }
                AdvancingComponentKind::ScriptedDrawable
                | AdvancingComponentKind::ScriptedLayout
                | AdvancingComponentKind::ScriptedPathEffect
                    if script_mode.is_enabled() =>
                {
                    self.advance_script_component_with(
                        entry,
                        elapsed_seconds,
                        &mut script_host,
                        &mut |instance, args, host| script_mode.call(instance, args, host),
                    )?
                }
                // Root Artboard participates in the C++ advancing interface
                // only when a concrete root owner adds work; ordinary roots
                // have no additional advance body.
                AdvancingComponentKind::Artboard
                | AdvancingComponentKind::ScriptedDrawable
                | AdvancingComponentKind::ScriptedLayout
                | AdvancingComponentKind::ScriptedPathEffect => false,
            };
        }
        Ok(changed)
    }

    fn advance_layout_component_entry(
        &mut self,
        entry: RuntimeAdvancingComponent,
        elapsed_seconds: f32,
        new_frame: bool,
    ) -> bool {
        if !new_frame {
            return false;
        }
        let Some(component) = entry.component else {
            return false;
        };
        let Some(advance) = self.objects.component(component).and_then(|component| {
            (!component.is_collapsed())
                .then(|| component.concrete.layout.as_ref())
                .flatten()
                .map(|layout| layout.advance_interpolation(elapsed_seconds, true))
        }) else {
            return false;
        };
        if advance.size_changed {
            self.add_dirt(entry.local_id, ComponentDirt::PATH, false);
        }
        if advance.layout_changed {
            // `LayoutComponent::applyInterpolation` writes `m_layout`, then
            // propagates size and marks the exact owner world-transform dirty
            // (`src/layout_component.cpp:1329-1401`).
            self.add_dirt(entry.local_id, ComponentDirt::WORLD_TRANSFORM, true);
            self.layout_epoch = self.layout_epoch.wrapping_add(1);
            self.runtime_drawables.mark_layout_resources_dirty();
        }
        advance.keep_going
    }

    fn advance_text_input_entry(
        &mut self,
        entry: RuntimeAdvancingComponent,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(component) = entry.component else {
            return false;
        };
        let Some((is_dragging, scroll_constraint, scroll_x, scroll_y, last_position)) =
            self.objects.component(component).and_then(|component| {
                let state = component.concrete.text_input.as_ref()?;
                Some((
                    state.is_dragging,
                    state.scroll_constraint,
                    state.scroll_x,
                    state.scroll_y,
                    state.last_drag_world_position,
                ))
            })
        else {
            return false;
        };
        if !is_dragging {
            if let Some(state) = self
                .objects
                .component_mut(component)
                .and_then(|component| component.concrete.text_input.as_mut())
            {
                state.scroll_x = 0.0;
                state.scroll_y = 0.0;
            }
            return false;
        }
        let Some(scroll_constraint) = scroll_constraint else {
            return false;
        };
        if scroll_x == 0.0 && scroll_y == 0.0 {
            return false;
        }

        crate::constraints::advance_text_input_scroll(
            self,
            scroll_constraint,
            scroll_x,
            scroll_y,
            elapsed_seconds,
        );
        if last_position.0.is_finite() && last_position.1.is_finite() {
            // RawTextInput selection placement is retained by the text owner;
            // the Paint dirt is the frame-loop-visible side effect after
            // `moveCursorTo` (`src/text/text_input.cpp:751-760`).
            self.add_dirt(entry.local_id, ComponentDirt::PAINT, false);
        }
        true
    }

    fn advance_scripted_data_converter_entry(
        &mut self,
        local_id: usize,
        elapsed_seconds: f32,
        advance_nested: bool,
    ) -> bool {
        // `ScriptedDataConverter::advanceComponent` zeros elapsed time when
        // AdvanceNested is absent, and `advance` then rejects zero before
        // calling user code (`scripted_data_converter.cpp:190-211`).
        let elapsed_seconds = if advance_nested { elapsed_seconds } else { 0.0 };
        if elapsed_seconds == 0.0 {
            return false;
        }
        let Some(global_id) = self.slots.get(local_id).map(|slot| slot.source_global_id) else {
            return false;
        };
        let Some(handle) = self
            .scripted_data_converter_instances_by_global
            .get(&global_id)
            .cloned()
        else {
            return false;
        };
        let mut host = NoopScriptHost;
        let Ok(result) = handle.borrow_mut().call_method(
            ScriptMethod::Advance,
            &[ScriptValue::Number(f64::from(elapsed_seconds))],
            &mut host,
        ) else {
            return false;
        };
        if result != ScriptValue::Bool(true) {
            return false;
        }
        self.mark_scripted_data_converter_dirty(global_id);
        true
    }

    fn advance_nested_artboard_entry(
        &mut self,
        host_local: usize,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        layout_frame: Option<&RuntimeNestedLayoutBoundsFrame>,
        initial_layout_paint_evaluations: &mut BTreeMap<usize, ArtboardInstance>,
        nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> Result<bool, ScriptError> {
        if self
            .component(host_local)
            .is_some_and(RuntimeComponent::is_collapsed)
        {
            return Ok(false);
        }
        if let Some(layout_frame) = layout_frame {
            let layout_data_transferred = self
                .nested_artboards
                .get(&host_local)
                .is_some_and(|nested| nested.layout_data_transferred);
            if layout_data_transferred {
                self.apply_nested_artboard_layout_bounds(
                    host_local,
                    layout_frame.bounds.as_ref().as_ref(),
                    layout_frame.key,
                );
            } else if let Some(paint_evaluation) =
                initial_layout_paint_evaluations.remove(&host_local)
            {
                self.capture_initial_nested_artboard_layout_paint_frame(
                    host_local,
                    layout_frame.bounds.as_ref().as_ref(),
                    paint_evaluation,
                );
            }
        }

        let mut reported_events = Vec::new();
        let (keep_going, child_dirty) =
            if let Some(nested) = self.nested_artboards.get_mut(&host_local) {
                let keep_going = if new_frame {
                    nested.advance(
                        elapsed_seconds,
                        script_mode,
                        nested_events.as_ref().map(|_| &mut reported_events),
                    )?
                } else {
                    nested.advance_outer_update()
                };
                (keep_going, nested.child.has_dirt(ComponentDirt::COMPONENTS))
            } else {
                (false, false)
            };
        if let Some(nested_events) = nested_events
            && !reported_events.is_empty()
        {
            nested_events.push((host_local, reported_events));
        }
        if child_dirty {
            self.add_dirt(host_local, ComponentDirt::COMPONENTS, false);
        }
        Ok(keep_going)
    }

    fn advance_component_list_entry(
        &mut self,
        list_local: usize,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
    ) -> Result<bool, ScriptError> {
        if self
            .component(list_local)
            .is_none_or(RuntimeComponent::is_collapsed)
            || self
                .component_list_items(list_local)
                .is_none_or(|items| items.is_empty())
        {
            return Ok(false);
        }
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let mut source_changed = false;
        let mut child_dirty = false;
        let mut keep_going = false;
        let Some(items) = self.component_list_items_mut(list_local) else {
            return Ok(false);
        };
        for item in items {
            let mut row_changed = false;
            if !item.context_is_current(&item.context)
                && let Some(file) = item.child.runtime_file_arc()
            {
                let child_data_context = RuntimeOwnedDataContext::with_local_handles(
                    [item.context.clone()],
                    Some(&parent_data_context),
                );
                row_changed |= item.child.bind_owned_view_model_artboard_data_context(
                    &file,
                    &child_data_context,
                    true,
                    true,
                );
                item.child.artboard_owned_view_model_context = Some(
                    RuntimeOwnedViewModelContext::from_main_handle(item.context.clone()),
                );
                for state_machine in &mut item.state_machines {
                    if state_machine.bind_owned_view_model_data_context(&child_data_context) {
                        row_changed = true;
                        row_changed |= state_machine.advance_data_context();
                    }
                }
                item.consume_context_rebind_dirt();
                source_changed = true;
            }
            for state_machine in &mut item.state_machines {
                if new_frame {
                    row_changed |= item
                        .child
                        .advance_state_machine_instance(state_machine, elapsed_seconds);
                } else if item
                    .child
                    .try_change_state_machine_instance_unconditionally(state_machine)
                {
                    // Component-list rows use the literal unguarded
                    // `tryChangeState()` branch when `NewFrame` is absent.
                    // A row mounted during the preceding update therefore
                    // enters its initial state in this same outer pass
                    // (`artboard_component_list.cpp:827-854`).
                    row_changed = true;
                    row_changed |= item.child.advance_state_machine_instance_after_state_probe(
                        state_machine,
                        elapsed_seconds,
                    );
                }
            }
            row_changed |= if new_frame {
                item.child
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        true,
                        script_mode,
                        None,
                    )?
            } else {
                item.child
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        false,
                        script_mode,
                        None,
                    )?
            };
            row_changed |= item
                .child
                .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
            child_dirty |= item.child.has_dirt(ComponentDirt::COMPONENTS);
            if item
                .context_rebind_sink
                .peek_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
            {
                source_changed = true;
            }
            if row_changed {
                item.settled_layout_size.set(None);
            }
            keep_going |= row_changed;
        }
        if source_changed {
            self.mark_component_list_source_changed();
        }
        if child_dirty {
            self.add_dirt(list_local, ComponentDirt::COMPONENTS, false);
        }
        Ok(keep_going)
    }

    fn runtime_nested_artboard_layout_bounds_frame(&mut self) -> RuntimeNestedLayoutBoundsFrame {
        let key = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: self.graph_global_id,
            layout_epoch: self.layout_epoch,
        };
        if self
            .nested_layout_bounds
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.nested_layout_bounds = Some(RuntimeNestedLayoutBoundsFrame {
                key,
                bounds: Arc::new(self.compute_runtime_nested_artboard_layout_bounds()),
            });
        }

        self.nested_layout_bounds
            .as_ref()
            .expect("nested layout bounds frame was just populated")
            .clone()
    }

    fn compute_runtime_nested_artboard_layout_bounds(
        &self,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        if !self.nested_artboard_locals.iter().any(|local_id| {
            self.component(*local_id)
                .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        }) {
            return None;
        }
        let context = self.build_context.as_ref()?;
        let runtime = context.file.clone();
        let graph = context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)?
            .clone();
        self.runtime_taffy_layout_bounds(&graph, Some(runtime.as_ref()))
    }

    fn capture_initial_nested_artboard_layout_paint_frame(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        mut paint_evaluation: ArtboardInstance,
    ) {
        if !self
            .component(host_local_id)
            .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        {
            return;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return;
        };
        // C++ configures paints on this one mounted occurrence before
        // NestedArtboardLayout transfers its constraint space. Evaluate that
        // source-side shader state only on a script-free temporary occurrence.
        paint_evaluation.detach_initial_nested_layout_paint_binding_contexts();
        paint_evaluation.set_artboard_dimensions(bounds.width, bounds.height);
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            paint_evaluation.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            paint_evaluation.set_double_property(0, height_key, bounds.height);
        }
        paint_evaluation.update_components();
        let before_bind = paint_evaluation.capture_initial_nested_layout_paint_frame();
        paint_evaluation.advance_artboard_data_binds();
        paint_evaluation.update_components();
        let frame = paint_evaluation.capture_initial_nested_layout_paint_frame();
        if !frame.changed_from(&before_bind) {
            return;
        }
        if let Some(nested) = self.nested_artboards.get_mut(&host_local_id)
            && !nested.layout_data_transferred
            && nested.initial_layout_paint_frame.borrow().is_none()
        {
            nested.initial_layout_paint_frame.replace(Some(frame));
        }
    }

    fn apply_nested_artboard_layout_bounds_after_parent_solve(&mut self) -> bool {
        if !self.nested_artboard_locals.iter().any(|host_local_id| {
            self.component(*host_local_id)
                .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        }) {
            return false;
        }
        let layout_frame = self.runtime_nested_artboard_layout_bounds_frame();
        let mut changed = false;
        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            if self
                .component(host_local_id)
                .is_some_and(RuntimeComponent::is_collapsed)
            {
                continue;
            }
            changed |= self.apply_nested_artboard_layout_bounds(
                host_local_id,
                layout_frame.bounds.as_ref().as_ref(),
                layout_frame.key,
            );
        }
        changed
    }

    fn apply_nested_artboard_layout_bounds(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    ) -> bool {
        if !self
            .component(host_local_id)
            .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        {
            return false;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return false;
        };
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };

        let first_transfer = !nested.layout_data_transferred;
        let refresh_constraint_bounds = nested.layout_data_transfer_key.is_none_or(|key| {
            key.parent_layout != parent_layout
                || key.assigned_bounds != bounds
                || key.child_layout_epoch != nested.child.layout_epoch
        });
        let mut changed = nested
            .child
            .set_artboard_dimensions(bounds.width, bounds.height);
        if first_transfer {
            // The recursive host bind above has applied the rounded initial
            // values but has not yet consumed their component dirt. Settle
            // that unconstrained component state before taking the one Yoga
            // layout snapshot owned by the parent.
            changed |= nested.child.update_components().did_update;
        }

        // Match NestedArtboardLayout's mounted ordering: the constraint space
        // exists before its root LayoutComponent width/height dirt is raised.
        // Reversing these two operations changes the first layout solve.
        if refresh_constraint_bounds {
            nested.child.refresh_layout_constraint_bounds();
            changed = true;
        } else {
            changed |= !nested.child.layout_constraint_bounds_enabled;
            nested.child.enable_layout_constraint_bounds();
        }
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            changed |= nested.child.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            changed |= nested
                .child
                .set_double_property(0, height_key, bounds.height);
        }
        nested.layout_data_transferred = true;
        if changed {
            nested.child.update_pass();
        }
        // Record after assigned-root writes and their child update pass. Those
        // writes dirty the transferred root node themselves; only a later
        // child layout generation should emulate C++ `markHostingLayoutDirty`
        // and request another parent-owned constraint refresh.
        nested.layout_data_transfer_key = Some(RuntimeNestedLayoutDataTransferKey {
            parent_layout,
            assigned_bounds: bounds,
            child_layout_epoch: nested.child.layout_epoch,
        });
        changed
    }

    pub fn set_transform_property(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        value: f32,
    ) -> bool {
        let Some(component) = self.component(local_id) else {
            return false;
        };
        let property_key = component.transform_property_key(property);
        let Some(property_key) = property_key else {
            return false;
        };
        self.set_transform_property_with_key(local_id, property, property_key, value)
    }

    pub(crate) fn set_transform_property_with_key(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
        value: f32,
    ) -> bool {
        let Some(component) = self.component(local_id) else {
            return false;
        };
        if !component.capabilities.transform {
            return false;
        }

        let Some(current) = self.transform_property_with_key(local_id, property, property_key)
        else {
            return false;
        };
        if current == value {
            return false;
        }
        let object_changed =
            self.objects
                .set_generated_double_property(local_id, property_key, value);
        let legacy_scale_changed =
            self.mark_legacy_image_layout_scale_written(local_id, property_key);
        if !object_changed && !legacy_scale_changed {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);

        match property {
            TransformProperty::Opacity => {
                self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
            }
            TransformProperty::X
            | TransformProperty::Y
            | TransformProperty::Rotation
            | TransformProperty::ScaleX
            | TransformProperty::ScaleY => {
                let handle = self
                    .component_handle(local_id)
                    .expect("validated Transform has a Component handle");
                self.mark_transform_dirty_handle(handle);
            }
        }
        true
    }

    pub fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        let component = self
            .component(local_id)
            .filter(|component| component.capabilities.transform)?;
        let property_key = component.transform_property_key(property)?;
        self.transform_property_with_key(local_id, property, property_key)
    }

    pub(crate) fn transform_property_with_key(
        &self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
    ) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)?;
        Some(
            self.double_property(local_id, property_key)
                .unwrap_or_else(|| property.default_value()),
        )
    }

    pub(crate) fn authored_transform(&self, local_id: usize) -> AuthoredTransform {
        let component = self.component(local_id);
        let (x, y) = if component
            .and_then(|component| component.concrete.bone.as_ref())
            .is_some_and(|bone| !bone.is_root)
        {
            let parent_length = self
                .component_handle(local_id)
                .and_then(|handle| self.objects.component(handle))
                .and_then(|component| component.parent)
                .and_then(|parent| self.objects.component_local_id(parent))
                .and_then(|parent_local| self.bone_length(parent_local))
                .unwrap_or(0.0);
            (parent_length, 0.0)
        } else {
            (
                self.transform_property(local_id, TransformProperty::X)
                    .unwrap_or_else(|| TransformProperty::X.default_value()),
                self.transform_property(local_id, TransformProperty::Y)
                    .unwrap_or_else(|| TransformProperty::Y.default_value()),
            )
        };

        AuthoredTransform {
            x,
            y,
            rotation: self
                .transform_property(local_id, TransformProperty::Rotation)
                .unwrap_or_else(|| TransformProperty::Rotation.default_value()),
            scale_x: self
                .transform_property(local_id, TransformProperty::ScaleX)
                .unwrap_or_else(|| TransformProperty::ScaleX.default_value()),
            scale_y: self
                .transform_property(local_id, TransformProperty::ScaleY)
                .unwrap_or_else(|| TransformProperty::ScaleY.default_value()),
            opacity: self
                .transform_property(local_id, TransformProperty::Opacity)
                .unwrap_or_else(|| TransformProperty::Opacity.default_value()),
        }
    }

    pub(crate) fn bone_length(&self, local_id: usize) -> Option<f32> {
        self.component(local_id)?.concrete.bone.as_ref()?;
        self.objects
            .double_property_by_name(local_id, "length")
            .or(Some(0.0))
    }

    pub(crate) fn runtime_skinnable_handle_has_skin(&self, handle: ComponentHandle) -> bool {
        self.objects
            .component(handle)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .is_some_and(|skinnable| skinnable.skin.is_some())
    }

    pub(crate) fn runtime_node_computed_local_transform(&self, local_id: usize) -> Option<Mat2D> {
        let handle = self.component_handle(local_id)?;
        let component = self.objects.component(handle)?;
        let node = component.concrete.node.as_ref()?;
        let parent_world = component
            .parent_transform
            .and_then(|parent| self.objects.component(parent))
            .map(|parent| parent.transform.world_transform);
        Some(node.computed_local_transform(parent_world, component.transform.world_transform))
    }

    pub(crate) fn runtime_vertex_weight_state(
        &self,
        vertex_local: usize,
    ) -> Option<crate::components::RuntimeWeightState> {
        let vertex = self.component_handle(vertex_local)?;
        let weight = self
            .objects
            .component(vertex)?
            .concrete
            .vertex
            .as_ref()?
            .weight?;
        self.objects.component(weight)?.concrete.weight
    }

    /// Settle one retained Weight/CubicWeight from the Skin-owned transform
    /// buffer. The caller supplies live Vertex points; packed indices/values
    /// are always read from this occurrence's generated storage.
    pub(crate) fn deform_runtime_vertex_weight(
        &mut self,
        vertex_local: usize,
        point: (f32, f32),
        cubic_points: Option<((f32, f32), (f32, f32))>,
    ) -> bool {
        let Some(vertex) = self.component_handle(vertex_local) else {
            return false;
        };
        let Some(weight) = self
            .objects
            .component(vertex)
            .and_then(|component| component.concrete.vertex.as_ref())
            .and_then(|vertex| vertex.weight)
        else {
            return false;
        };
        let Some(skinnable) = self
            .objects
            .component(vertex)
            .and_then(|component| component.parent)
        else {
            return false;
        };
        let Some(skin) = self
            .objects
            .component(skinnable)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .and_then(|skinnable| skinnable.skin)
        else {
            return false;
        };
        let Some(weight_local) = self.objects.component_local_id(weight) else {
            return false;
        };
        let Some(is_cubic_weight) = self
            .objects
            .component(weight)
            .and_then(|component| component.concrete.weight.as_ref())
            .map(|weight| weight.is_cubic)
        else {
            return false;
        };
        let values = property_key_for_name("Weight", "values")
            .and_then(|key| self.uint_property(weight_local, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(255);
        let indices = property_key_for_name("Weight", "indices")
            .and_then(|key| self.uint_property(weight_local, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1);

        let Some(skin_state) = self
            .objects
            .component(skin)
            .and_then(|component| component.concrete.skin.as_ref())
        else {
            return false;
        };
        let Some(translation) = deform_point_from_skin(
            point,
            indices,
            values,
            skin_state.world_transform,
            &skin_state.bone_transforms,
        ) else {
            return false;
        };
        let cubic_translations = if is_cubic_weight {
            let (in_point, out_point) = cubic_points.unwrap_or((point, point));
            let in_values = property_key_for_name("CubicWeight", "inValues")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(255);
            let in_indices = property_key_for_name("CubicWeight", "inIndices")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(1);
            let out_values = property_key_for_name("CubicWeight", "outValues")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(255);
            let out_indices = property_key_for_name("CubicWeight", "outIndices")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(1);
            let Some(in_translation) = deform_point_from_skin(
                in_point,
                in_indices,
                in_values,
                skin_state.world_transform,
                &skin_state.bone_transforms,
            ) else {
                return false;
            };
            let Some(out_translation) = deform_point_from_skin(
                out_point,
                out_indices,
                out_values,
                skin_state.world_transform,
                &skin_state.bone_transforms,
            ) else {
                return false;
            };
            Some((in_translation, out_translation))
        } else {
            None
        };

        let state = self
            .objects
            .component_mut(weight)
            .and_then(|component| component.concrete.weight.as_mut())
            .expect("validated Weight handle owns Weight state");
        state.translation = translation;
        if let Some((in_translation, out_translation)) = cubic_translations {
            state.in_translation = in_translation;
            state.out_translation = out_translation;
        }
        true
    }

    pub fn has_dirt(&self, dirt: ComponentDirt) -> bool {
        self.objects
            .root()
            .and_then(|root| self.objects.component(root))
            .is_some_and(|component| component.dirt.contains(dirt))
    }

    #[cfg(test)]
    fn set_artboard_dirt_for_test(&mut self, dirt: ComponentDirt) {
        let root = self
            .objects
            .root()
            .expect("test Artboard occurrence has a root Component");
        self.objects
            .component_mut(root)
            .expect("test Artboard root Component remains live")
            .dirt = dirt;
    }

    pub fn did_change(&self) -> bool {
        self.did_change.get()
    }

    pub fn frame_origin(&self) -> bool {
        self.frame_origin.get()
    }

    pub fn set_frame_origin(&self, frame_origin: bool) {
        self.frame_origin.set(frame_origin);
    }

    pub fn frame_id(&self) -> u64 {
        self.frame_id.get()
    }

    pub(crate) fn begin_draw_frame(&self) {
        self.frame_id.set(self.frame_id.get().wrapping_add(1));
    }

    pub(crate) fn cache_epoch(&self) -> u64 {
        self.cache_epoch
    }

    pub(crate) fn instance_identity(&self) -> u64 {
        self.instance_identity.0
    }

    pub(crate) fn prepared_epoch(&self) -> u64 {
        self.prepared_epoch
    }

    pub(crate) fn path_epoch(&self) -> u64 {
        self.path_epoch
    }

    pub(crate) fn layout_epoch(&self) -> u64 {
        self.layout_epoch
    }

    pub(crate) fn solid_color_paint_revision(&self, local_id: usize) -> u64 {
        self.solid_color_paint_revisions
            .get(local_id)
            .copied()
            .unwrap_or_default()
    }

    fn mark_changed(&mut self) {
        self.did_change.set(true);
        self.cache_epoch = self.cache_epoch.wrapping_add(1);
    }

    pub(crate) fn mark_artboard_data_bind_work_dirty(&mut self) {
        self.artboard_data_bind_dirty_epoch = self.artboard_data_bind_dirty_epoch.wrapping_add(1);
    }

    fn mark_stateful_nested_view_model_contexts_dirty_for_local(&mut self, local_id: usize) {
        if self
            .slot(local_id)
            .and_then(|slot| slot.type_name)
            .is_some_and(|type_name| type_name.starts_with("ViewModelInstance"))
        {
            self.stateful_nested_view_model_contexts_dirty = true;
        }
    }

    pub(crate) fn mark_prepared_changed(&mut self) {
        self.prepared_epoch = self.prepared_epoch.wrapping_add(1);
        self.mark_tree_paint_preparation_changed();
    }

    fn mark_world_transform_changed(&mut self) {
        self.prepared_epoch = self.prepared_epoch.wrapping_add(1);
        self.mark_tree_paint_preparation_changed();
    }

    pub(crate) fn enable_layout_constraint_bounds(&mut self) {
        if self.layout_constraint_bounds_enabled {
            return;
        }
        self.refresh_layout_constraint_bounds();
    }

    pub(crate) fn refresh_layout_constraint_bounds(&mut self) {
        self.layout_constraint_bounds_enabled = true;
        self.layout_constraint_bounds = self.runtime_graph().and_then(|graph| {
            self.runtime_taffy_layout_bounds(graph, self.runtime_file())
                .map(Arc::new)
        });
        self.enqueue_artboard_parametric_layout_control_sources();
        let layout_locals = self
            .components()
            .iter()
            .filter(|component| component.type_name == "LayoutComponent")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        for local_id in layout_locals {
            self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub(crate) fn mark_layout_changed(&mut self) {
        self.layout_epoch = self.layout_epoch.wrapping_add(1);
        self.runtime_drawables.mark_layout_resources_dirty();
        self.runtime_drawables.mark_text_resources_dirty();
        self.mark_prepared_changed();
    }

    pub(crate) fn mark_path_changed(&mut self) {
        self.path_epoch = self.path_epoch.wrapping_add(1);
        self.runtime_drawables.mark_layout_resources_dirty();
        self.runtime_drawables.mark_text_resources_dirty();
        self.mark_prepared_changed();
    }

    fn mark_runtime_shape_property_changed(&mut self, local_id: usize, affects_effect_path: bool) {
        let paint_locals = self
            .runtime_shapes
            .mark_property_changed(local_id, affects_effect_path);
        if !affects_effect_path {
            return;
        }
        self.mark_path_changed();
        if paint_locals.is_empty() {
            self.add_dirt(local_id, ComponentDirt::PATH, false);
        } else {
            // Preserve the existing RD-1 Trim/Dash/Feather adapter while
            // FL-A replaces the Component schedule: the sidecar already
            // resolves each retained ShapePaint owner, and the new schedule
            // must publish the same PATH dirt on that owner. ParametricPath
            // and the complete effect owner family remain pending in FL-E
            // (`dash.cpp:39-48`, `stroke_effect.cpp:13-25`,
            // `shape_paint.cpp:193-205`).
            for paint_local in paint_locals {
                self.add_dirt(paint_local, ComponentDirt::PATH, false);
            }
        }
    }

    fn mark_text_changed(&mut self) {
        self.runtime_drawables.mark_text_resources_dirty();
    }

    fn mark_text_changed_for_local(&mut self, local_id: usize) {
        if self
            .text_affecting_locals
            .get(local_id)
            .copied()
            .unwrap_or(false)
        {
            self.mark_text_changed();
        }
    }

    fn mark_component_list_source_changed(&mut self) {
        // An item-owned write can feed arbitrary bindings on the parent. Until
        // those dependencies are indexed, conservatively invalidate every
        // parent rendering cache that can consume the retained list source.
        self.mark_changed();
        self.mark_path_changed();
        self.mark_layout_changed();
    }

    fn mark_draw_order_changed(&mut self) {
        self.mark_prepared_changed();
    }

    fn mark_clipping_changed(&mut self) {
        self.mark_prepared_changed();
    }

    fn mark_render_opacity_changed(&mut self) {
        self.mark_prepared_changed();
    }

    fn mark_prepared_changed_for_property(&mut self, local_id: usize, property_key: u16) {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if property_may_affect_prepared_frame(type_name, property_key) {
            self.mark_prepared_changed();
        }
    }

    fn mark_prepared_changed_for_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        previous: Option<u32>,
        next: u32,
    ) {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("SolidColor")
            && solid_color_value_property_key() == Some(property_key)
        {
            self.mark_prepared_changed_for_solid_color_visibility(previous, next);
        } else {
            self.mark_prepared_changed_for_property(local_id, property_key);
        }
    }

    fn mark_prepared_changed_for_solid_color_visibility(
        &mut self,
        previous: Option<u32>,
        next: u32,
    ) {
        let next_visible = (next >> 24) != 0;
        if previous.is_none_or(|previous| ((previous >> 24) != 0) != next_visible) {
            self.mark_prepared_changed();
        }
    }

    fn mark_layout_changed_for_property(&mut self, local_id: usize, property_key: u16) {
        if self.property_affects_layout(local_id, property_key) {
            self.mark_layout_changed();
        }
    }

    fn property_affects_layout(&self, local_id: usize, property_key: u16) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if matches!(
            type_name,
            Some("LayoutComponentStyle" | "NestedArtboardLayout")
        ) {
            return true;
        }

        if type_name == Some("ArtboardComponentListOverride") {
            return [
                "instanceWidth",
                "instanceHeight",
                "instanceWidthUnitsValue",
                "instanceHeightUnitsValue",
                "instanceWidthScaleType",
                "instanceHeightScaleType",
            ]
            .into_iter()
            .any(|name| {
                property_key_for_name("ArtboardComponentListOverride", name) == Some(property_key)
            });
        }

        if matches!(type_name, Some("Text")) {
            return [
                "alignValue",
                "sizingValue",
                "overflowValue",
                "width",
                "height",
                "verticalTrimValue",
            ]
            .into_iter()
            .any(|name| property_key_for_name("Text", name) == Some(property_key));
        }

        if matches!(type_name, Some("TextStyle" | "TextStylePaint")) {
            return ["fontSize", "lineHeight", "letterSpacing"]
                .into_iter()
                .any(|name| property_key_for_name("TextStyle", name) == Some(property_key));
        }

        if matches!(type_name, Some("TextValueRun")) {
            return property_key_for_name("TextValueRun", "text") == Some(property_key)
                || property_key_for_name("TextValueRun", "styleId") == Some(property_key);
        }

        if matches!(type_name, Some("TextInput")) {
            return property_key_for_name("TextInput", "text") == Some(property_key)
                || property_key_for_name("TextInput", "multiline") == Some(property_key);
        }

        matches!(
            type_name,
            Some("Artboard" | "LayoutComponent" | "NestedArtboard")
        ) && (property_key_for_name("Artboard", "width") == Some(property_key)
            || property_key_for_name("Artboard", "height") == Some(property_key)
            || property_key_for_name("LayoutComponent", "width") == Some(property_key)
            || property_key_for_name("LayoutComponent", "height") == Some(property_key)
            || property_key_for_name("LayoutComponent", "styleId") == Some(property_key)
            || property_key_for_name("LayoutComponent", "fractionalWidth") == Some(property_key)
            || property_key_for_name("LayoutComponent", "fractionalHeight") == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidthScaleType")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeightScaleType")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidthUnitsValue")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeightUnitsValue")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidth") == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeight")
                == Some(property_key))
    }

    pub fn clear_component_dirt(&mut self, local_id: usize) {
        if let Some(component) = self.component_mut(local_id) {
            component.dirt = ComponentDirt::NONE;
        }
    }

    pub fn add_dirt(&mut self, local_id: usize, dirt: ComponentDirt, recurse: bool) -> bool {
        let Some(handle) = self.component_handle(local_id) else {
            return false;
        };
        self.add_component_dirt(handle, dirt, recurse)
    }

    fn add_component_dirt(
        &mut self,
        handle: ComponentHandle,
        dirt: ComponentDirt,
        recurse: bool,
    ) -> bool {
        if dirt.is_empty() {
            return false;
        }

        let Some(component) = self.objects.component(handle) else {
            return false;
        };
        if component.dirt.contains(dirt) {
            return false;
        }

        // C++ Component::addDirt publishes the accumulated mask before any
        // concrete callback can re-enter this owner.
        let accumulated = {
            let component = self
                .objects
                .component_mut(handle)
                .expect("component handle was resolved above");
            component.dirt |= dirt;
            component.dirt
        };
        self.dispatch_component_on_dirty(handle, accumulated);
        self.on_component_dirty_handle(handle);

        if recurse {
            let dependent_count = self.objects.dependent_len(handle);
            for index in 0..dependent_count {
                let Some(dependent) = self.objects.dependent_at(handle, index) else {
                    continue;
                };
                self.add_component_dirt(dependent, dirt, true);
            }
        }
        true
    }

    /// Literal `TransformComponent::markTransformDirty`: recursive World dirt
    /// is gated by the transition that newly adds Transform dirt. Repeating a
    /// transform setter while Transform is already pending must not re-dirty a
    /// clean dependent subtree (`src/transform_component.cpp:54-61`).
    fn mark_transform_dirty_handle(&mut self, handle: ComponentHandle) -> bool {
        if !self.add_component_dirt(handle, ComponentDirt::TRANSFORM, false) {
            return false;
        }
        self.add_component_dirt(handle, ComponentDirt::WORLD_TRANSFORM, true);
        true
    }

    fn dispatch_component_on_dirty(&mut self, handle: ComponentHandle, accumulated: ComponentDirt) {
        let Some(component) = self.objects.component(handle) else {
            return;
        };
        let local_id = component.local_id;
        let constraint_parent = component
            .concrete
            .constraint
            .as_ref()
            .and_then(|_| component.parent);
        let constraint_is_ik = component.concrete.constraint.is_some_and(|constraint| {
            constraint.kind == crate::components::RuntimeConstraintKind::Ik
        });
        let skin_skinnable = component
            .concrete
            .skin
            .as_ref()
            .and_then(|skin| skin.skinnable);
        let path_has_deferred_dirt = component
            .concrete
            .path
            .as_ref()
            .is_some_and(|path| path.deferred_path_dirt.get());
        let Some(address) = self.objects.address(handle) else {
            return;
        };
        // Neither embedded owner inherits the authored object's concrete
        // callbacks. PathComposer::onDirty only services deferred path dirt,
        // which this retained owner never defers; TextVariationHelper has no
        // onDirty override. In particular, do not run broad Shape/Text/Mesh
        // invalidation merely because these embedded Components share their
        // owner's serialized local id.
        if !matches!(address, ComponentAddress::Authored(_)) {
            return;
        }

        // Constraint::onDirty is unconditional: any accumulated dirt on the
        // concrete Constraint marks its retained constrained parent before
        // Artboard::onComponentDirty observes the Constraint itself
        // (`src/constraints/constraint.cpp:35-40`).
        if constraint_is_ik {
            self.mark_ik_constraint_dirty(local_id);
        } else if let Some(parent) = constraint_parent {
            self.mark_transform_dirty_handle(parent);
        }

        // Exact `Skin::onDirty`: call only the retained Skinnable, before the
        // outer Skin reaches Artboard::onComponentDirty. PointsPath and Mesh
        // intentionally consume different dirt families
        // (`src/bones/skin.cpp:88-94`,
        // `src/shapes/points_path.cpp:43-52`,
        // `src/shapes/mesh.cpp:84-85`).
        if let Some(skinnable) = skin_skinnable {
            match self
                .objects
                .component(skinnable)
                .and_then(|component| component.concrete.skinnable.as_ref())
                .map(|skinnable| skinnable.kind)
            {
                Some(RuntimeSkinnableKind::PointsPath) => {
                    self.add_component_dirt(skinnable, ComponentDirt::PATH, false);
                }
                Some(RuntimeSkinnableKind::Mesh) => {
                    self.add_component_dirt(skinnable, ComponentDirt::VERTICES, false);
                }
                _ => {}
            }
        }

        // A deferred C++ Path does not keep itself continuously dirty.
        // Instead, its next ordinary onDirty callback re-adds Path dirt,
        // guaranteeing one rebuild when the Shape becomes observable again
        // (`src/shapes/path.cpp:336-347`).
        if path_has_deferred_dirt {
            self.add_component_dirt(handle, ComponentDirt::PATH, false);
        }

        let text_variation_helper = accumulated
            .contains(ComponentDirt::TEXT_SHAPE)
            .then(|| self.objects.text_variation_helper_handle(local_id))
            .flatten();
        let text_style_parent = text_variation_helper
            .and_then(|_| self.objects.component(handle))
            .and_then(|component| component.parent);

        if !(accumulated
            & (ComponentDirt::TEXT_SHAPE
                | ComponentDirt::WORLD_TRANSFORM
                | ComponentDirt::RENDER_OPACITY
                | ComponentDirt::PAINT))
            .is_empty()
        {
            self.runtime_drawables
                .mark_text_resource_dirty_for_local(local_id);
        }
        self.runtime_meshes
            .mark_component_dirt(local_id, accumulated);
        if accumulated.contains(ComponentDirt::LAYOUT_STYLE) {
            self.mark_layout_changed();
        }
        if component_dirt_affects_path_epoch(accumulated) {
            self.mark_path_changed();
        } else if accumulated.contains(ComponentDirt::WORLD_TRANSFORM) {
            self.mark_world_transform_changed();
        }
        if accumulated.contains(ComponentDirt::DRAW_ORDER) {
            self.mark_draw_order_changed();
        }
        if accumulated.contains(ComponentDirt::CLIPPING) {
            self.mark_clipping_changed();
        }

        // Concrete Path callbacks own the transition from source dirt to
        // PathComposer::Path dirt. Keep retained geometry invalidation on the
        // resource owner, but schedule the embedded Component itself through
        // the occurrence's one dependency graph (`path.cpp:327-350`,
        // `shape.cpp:99-108`).
        for shape_local in self
            .runtime_shapes
            .on_component_dirty(local_id, accumulated)
        {
            if let Some(composer) = self.objects.path_composer_handle(shape_local) {
                let composer_dirt = ComponentDirt::PATH | (accumulated & ComponentDirt::N_SLICER);
                self.add_component_dirt(composer, composer_dirt, true);
            }
        }

        // Direct port of `TextStyle::onDirty`: shaping dirt first reaches the
        // owning Text and then the style's embedded TextVariationHelper. The
        // helper is an independent Component occurrence because its update
        // must run in dependency order after Artboard and before Text
        // (`src/text/text_style.cpp:22-34`,
        // `src/text/text_variation_helper.cpp:7-17`).
        if let Some(helper) = text_variation_helper {
            if let Some(text) = text_style_parent {
                self.add_component_dirt(text, ComponentDirt::TEXT_SHAPE, false);
            }
            self.add_component_dirt(helper, ComponentDirt::TEXT_SHAPE, false);
        }
    }

    pub fn collapse_component(&mut self, local_id: usize, collapsed: bool) -> bool {
        self.collapse_component_tree(local_id, collapsed)
    }

    fn collapse_component_handle(&mut self, handle: ComponentHandle, collapsed: bool) -> bool {
        let Some(component) = self.objects.component(handle) else {
            return false;
        };

        if component.is_collapsed() == collapsed {
            return false;
        }

        let accumulated = {
            let component = self
                .objects
                .component_mut(handle)
                .expect("component handle was resolved above");
            if collapsed {
                component.dirt |= ComponentDirt::COLLAPSED;
            } else {
                component.dirt &= !ComponentDirt::COLLAPSED;
            }
            component.dirt
        };
        // Component::collapse publishes its new mask to the concrete virtual
        // owner before Artboard and collapsable notifications
        // (`src/component.cpp:76-95`).
        self.dispatch_component_on_dirty(handle, accumulated);
        self.on_component_dirty_handle(handle);
        let collapsable_count = self.objects.collapsable_len(handle);
        for index in 0..collapsable_count {
            let Some(data_bind) = self.objects.collapsable_at(handle, index) else {
                continue;
            };
            self.collapse_artboard_authored_data_bind(data_bind, collapsed);
        }

        // Shape owns an embedded PathComposer and forwards collapse after its
        // base Component transition (`src/shapes/shape.cpp:64-71`). Giving
        // the composer its own Collapsed bit ensures Artboard skips it before
        // clearing pending Path dirt, exactly like the generic C++ traversal.
        if let Some(composer) = self
            .objects
            .component_local_id(handle)
            .and_then(|local_id| self.objects.path_composer_handle(local_id))
        {
            self.collapse_component_handle(composer, collapsed);
        }

        let Some(ComponentAddress::Authored(_)) = self.objects.address(handle) else {
            return true;
        };
        let local_id = self
            .objects
            .component_local_id(handle)
            .expect("authored component handle has an object");
        if !collapsed {
            if self.nested_artboards.contains_key(&local_id) {
                self.newly_uncollapsed_nested_artboards.insert(local_id);
            }
        }
        // Pinned C++ `Path::collapse` forwards every visibility transition
        // through `Shape::pathCollapseChanged` to
        // `PathComposer::pathCollapseChanged` (`path.cpp:384-390`,
        // `shape.cpp:330`, `path_composer.cpp:119-133`). That last method
        // explicitly dirties the composer's dependents even when the
        // composer already carries Path dirt, so do not route this through
        // the ordinary duplicate-dirt early return.
        if let Some(shape_local) = self.runtime_shapes.path_collapse_changed(local_id)
            && let Some(composer) = self.objects.path_composer_handle(shape_local)
        {
            self.add_component_dirt(composer, ComponentDirt::PATH, false);
            let dependent_count = self.objects.dependent_len(composer);
            for index in 0..dependent_count {
                if let Some(dependent) = self.objects.dependent_at(composer, index) {
                    self.add_component_dirt(dependent, ComponentDirt::PATH, true);
                }
            }
        }
        self.mark_path_changed();
        self.mark_layout_changed();
        self.apply_component_collapse_changed(local_id);
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_components_with_hook_recording(
            true,
            &mut script_mode,
            Mat2D::IDENTITY,
            |_, _, _, _| {},
        )
    }

    pub fn update_pass(&mut self) -> bool {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_pass_with_script_mode(&mut script_mode, Mat2D::IDENTITY)
    }

    fn update_pass_with_script_mode(
        &mut self,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        root_transform: Mat2D,
    ) -> bool {
        // Mirrors C++ src/artboard.cpp Artboard::updatePass: data binds run
        // before components, with artboard-host children publishing first.
        self.update_nested_artboard_data_binds_from_hosts(root_transform);
        self.advance_artboard_data_binds_with_root_transform(root_transform, 0.0);
        // C++ transfers a NestedArtboardLayout's Yoga node after the first
        // child-recursive data-bind pass, then reuses that node whenever the
        // parent Yoga graph reports a new layout. The transfer key keeps later
        // precise child-local writes from causing a second solve in the same
        // outer update while still refreshing genuine parent assignments.
        let mut did_update = self.apply_nested_artboard_layout_bounds_after_parent_solve();
        if self.joysticks_apply_before_update {
            did_update |= self.apply_joysticks(true);
        }
        // Updating a nested host's inherited opacity writes the mounted
        // child's root property. C++ leaves that child work for the next
        // outer pass, so retain a host marker after this pass instead of
        // either drawing stale child opacity or eagerly collapsing the
        // bounded outer-update sequence into this component walk.
        let mut deferred_nested_opacity_hosts = BTreeSet::new();
        let mut nested_did_update = false;
        if self
            .update_components_with_hook_recording(
                false,
                script_mode,
                root_transform,
                |instance, local_id, dirt, script_mode| {
                    nested_did_update |= instance.update_nested_artboard_from_host_dirt(
                        local_id,
                        dirt,
                        script_mode,
                        root_transform,
                    );
                    if dirt.contains(ComponentDirt::RENDER_OPACITY)
                        && instance
                            .nested_artboards
                            .get(&local_id)
                            .is_some_and(|nested| nested.child.has_dirt(ComponentDirt::COMPONENTS))
                    {
                        deferred_nested_opacity_hosts.insert(local_id);
                    }
                },
            )
            .did_update
        {
            did_update = true;
        }
        did_update |= nested_did_update;
        if !self.joysticks_apply_before_update {
            let joystick_count = self.joysticks.len();
            for joystick_index in 0..joystick_count {
                let mut nested_did_update = false;
                if !self.joysticks[joystick_index].can_apply_before_update
                    && self
                        .update_components_with_hook_recording(
                            false,
                            script_mode,
                            root_transform,
                            |instance, local_id, dirt, script_mode| {
                                nested_did_update |= instance
                                    .update_nested_artboard_from_host_dirt(
                                        local_id,
                                        dirt,
                                        script_mode,
                                        root_transform,
                                    );
                                if dirt.contains(ComponentDirt::RENDER_OPACITY)
                                    && instance.nested_artboards.get(&local_id).is_some_and(
                                        |nested| nested.child.has_dirt(ComponentDirt::COMPONENTS),
                                    )
                                {
                                    deferred_nested_opacity_hosts.insert(local_id);
                                }
                            },
                        )
                        .did_update
                {
                    did_update = true;
                }
                did_update |= nested_did_update;
                did_update |= self.apply_joystick_at(joystick_index);
            }
            let mut nested_did_update = false;
            if self
                .update_components_with_hook_recording(
                    false,
                    script_mode,
                    root_transform,
                    |instance, local_id, dirt, script_mode| {
                        nested_did_update |= instance.update_nested_artboard_from_host_dirt(
                            local_id,
                            dirt,
                            script_mode,
                            root_transform,
                        );
                        if dirt.contains(ComponentDirt::RENDER_OPACITY)
                            && instance
                                .nested_artboards
                                .get(&local_id)
                                .is_some_and(|nested| {
                                    nested.child.has_dirt(ComponentDirt::COMPONENTS)
                                })
                        {
                            deferred_nested_opacity_hosts.insert(local_id);
                        }
                    },
                )
                .did_update
            {
                did_update = true;
            }
            did_update |= nested_did_update;
        }
        if did_update {
            // C++ `Artboard::updatePass` polls derived target-to-source
            // bindings after `updateComponents`. The clean-frame epoch may
            // already have been consumed by the pre-component binding pass,
            // so wake this post-component pass when a computed numeric source
            // (currently `Shape.length`) needs the settled transforms.
            if !self
                .artboard_data_bind_source_queues
                .persisting_numeric_sources()
                .is_empty()
                && self.artboard_data_bind_dirty_epoch == self.artboard_data_bind_processed_epoch
            {
                self.mark_artboard_data_bind_work_dirty();
            }
            self.update_nested_artboard_data_binds_from_hosts(root_transform);
            self.advance_artboard_data_binds_with_root_transform(root_transform, 0.0);
        }
        if did_update
            || self.component_list_locals().into_iter().any(|local_id| {
                self.component_list_items(local_id).is_some_and(|items| {
                    items
                        .iter()
                        .any(|item| item.settled_layout_size.get().is_none())
                })
            })
        {
            did_update |= self.update_component_list_layout_bounds(root_transform);
        }
        for host_local_id in deferred_nested_opacity_hosts {
            if self
                .nested_artboards
                .get(&host_local_id)
                .is_some_and(|nested| nested.child.has_dirt(ComponentDirt::COMPONENTS))
            {
                did_update |= self.add_dirt(host_local_id, ComponentDirt::COMPONENTS, false);
            }
        }
        did_update
    }

    pub fn update_pass_with_script_errors(&mut self) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_pass_with_script_mode_and_errors(&mut script_mode)
    }

    /// Factory-aware `Artboard::updatePass` whose scripted Components execute
    /// at their retained dependency slots.
    pub fn update_pass_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptUpdateMode::Factory(factory);
        self.update_pass_with_script_mode_and_errors(&mut script_mode)
    }

    fn update_pass_with_script_mode_and_errors(
        &mut self,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
    ) -> Result<bool, ScriptError> {
        self.clear_script_update_error_tree();
        let did_update = self.update_pass_with_script_mode(script_mode, Mat2D::IDENTITY);
        match self.take_script_update_error_tree() {
            Some(error) => {
                self.rearm_pending_script_updates_tree();
                Err(error)
            }
            None => Ok(did_update),
        }
    }

    fn clear_script_update_error_tree(&mut self) {
        self.script_update_error = None;
        for (_, nested) in &mut self.nested_artboards.entries {
            nested.child.clear_script_update_error_tree();
        }
        for list_index in 0..self.component_list_count() {
            let Some(list_local) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(list_local) else {
                continue;
            };
            for item in items {
                item.child.clear_script_update_error_tree();
            }
        }
    }

    fn take_script_update_error_tree(&mut self) -> Option<ScriptError> {
        if let Some(error) = self.script_update_error.take() {
            return Some(error);
        }
        for (_, nested) in &mut self.nested_artboards.entries {
            if let Some(error) = nested.child.take_script_update_error_tree() {
                return Some(error);
            }
        }
        for list_index in 0..self.component_list_count() {
            let Some(list_local) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(list_local) else {
                continue;
            };
            for item in items {
                if let Some(error) = item.child.take_script_update_error_tree() {
                    return Some(error);
                }
            }
        }
        None
    }

    fn rearm_pending_script_updates_tree(&mut self) {
        let schedule_len = self.objects.dependency_order().len();
        for index in 0..schedule_len {
            let Some(component) = self.objects.scheduled_at(index) else {
                continue;
            };
            if self
                .objects
                .component(component)
                .and_then(|component| component.concrete.scripted.as_ref())
                .is_some_and(|scripted| scripted.update_pending)
            {
                self.add_component_dirt(component, ComponentDirt::SCRIPT_UPDATE, false);
            }
        }
        for (_, nested) in &mut self.nested_artboards.entries {
            nested.child.rearm_pending_script_updates_tree();
        }
        for list_index in 0..self.component_list_count() {
            let Some(list_local) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(list_local) else {
                continue;
            };
            for item in items {
                item.child.rearm_pending_script_updates_tree();
            }
        }
    }

    /// Settle the bounded component-update tail used by C++
    /// `StateMachineInstance::advanceAndApply`.
    ///
    /// C++ performs up to five outer update passes. Between passes it advances
    /// nested state changes without replaying ordinary nested animations, then
    /// bubbles remaining component dirt back through each host. That
    /// alternation matters for deep mounts: a parent pass can publish a host
    /// opacity only after its child already updated, leaving the grandchild
    /// dirty until the next outer pass.
    pub fn settle_state_machine_update_passes(&mut self) -> bool {
        self.settle_state_machine_update_passes_with_state_machines(&mut [])
    }

    /// Variant of [`Self::settle_state_machine_update_passes`] that also
    /// probes the root state machines between component passes.
    pub fn settle_state_machine_update_passes_with_state_machines(
        &mut self,
        state_machines: &mut [StateMachineInstance],
    ) -> bool {
        self.reset_outer_state_machine_changed_state_counts(state_machines);
        // A standalone settlement has not performed the main advance that
        // normally schedules root probes. Explicitly request one probe for
        // every supplied root while retaining the guarded after-main path.
        for state_machine in state_machines.iter_mut() {
            state_machine.schedule_post_update_probe();
        }
        self.settle_state_machine_update_passes_after_main_advance(state_machines)
    }

    /// Finish a frame whose root and nested state machines have already run
    /// their main advance. Unlike standalone settlement, this preserves those
    /// per-frame transition counts and adds only unique outer transitions.
    #[doc(hidden)]
    pub fn settle_state_machine_update_passes_after_main_advance(
        &mut self,
        state_machines: &mut [StateMachineInstance],
    ) -> bool {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.settle_state_machine_update_passes_after_main_advance_with_mode(
            state_machines,
            &mut script_mode,
        )
    }

    fn settle_state_machine_update_passes_after_main_advance_with_mode(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
    ) -> bool {
        self.settle_state_machine_update_passes_after_main_advance_with_mode_and_root_vm_reset(
            state_machines,
            script_mode,
            true,
        )
    }

    fn settle_state_machine_update_passes_after_main_advance_with_mode_and_root_vm_reset(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        reset_root_view_models: bool,
    ) -> bool {
        const MAX_OUTER_PASSES: usize = 5;

        let mut changed = false;
        for _ in 0..MAX_OUTER_PASSES {
            changed |= self.update_pass_with_script_mode(script_mode, Mat2D::IDENTITY);
            for state_machine in state_machines.iter_mut() {
                // C++ calls `StateMachineInstance::tryChangeState()` on every
                // outer pass, not only when a transition-specific probe was
                // pre-scheduled. Besides evaluating transitions, that method
                // runs `updateDataBinds(false)`, which propagates the
                // suppressed ViewModelTrigger reset back to a retained
                // ScriptInputTrigger before the next authored edge
                // (`state_machine_instance.cpp:2306-2318,2629-2635`).
                if self.try_change_state_machine_instance_unconditionally(state_machine) {
                    changed = true;
                    changed |=
                        self.advance_state_machine_instance_after_state_probe(state_machine, 0.0);
                }
            }
            changed |= self.advance_outer_update_components();
            if reset_root_view_models {
                for state_machine in state_machines.iter_mut() {
                    state_machine.reset_advanced_data_context();
                }
            }
            self.reset_retained_components();
            // C++ cloned ScriptInput DataBinds are Components, so a trigger
            // reset keeps the outer loop alive through their Bindings dirt.
            // Rust stores those occurrences in the state-machine container;
            // include its exact pending queue in the equivalent stop test.
            if !self.has_dirt(ComponentDirt::COMPONENTS)
                && !state_machines
                    .iter()
                    .any(StateMachineInstance::has_pending_data_bind_work)
            {
                break;
            }
        }
        changed
    }

    pub fn settle_state_machine_update_passes_after_main_advance_with_script_errors(
        &mut self,
        state_machines: &mut [StateMachineInstance],
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.settle_state_machine_update_passes_after_main_advance_with_mode_and_errors(
            state_machines,
            &mut script_mode,
            true,
        )
    }

    /// Script-owned artboards use C++ `advanceAndApply(seconds, false)`: run
    /// the same bounded component settlement and Artboard reset, but leave
    /// the root state-machine DataContext for its owning host frame to
    /// advance/reset (`lua_artboards.cpp:103-115`;
    /// `state_machine_instance.cpp:2601-2665`).
    #[doc(hidden)]
    pub fn settle_state_machine_update_passes_after_main_advance_without_root_view_model_reset_with_script_errors(
        &mut self,
        state_machines: &mut [StateMachineInstance],
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.settle_state_machine_update_passes_after_main_advance_with_mode_and_errors(
            state_machines,
            &mut script_mode,
            false,
        )
    }

    pub fn settle_state_machine_update_passes_after_main_advance_with_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptUpdateMode::Factory(factory);
        self.settle_state_machine_update_passes_after_main_advance_with_mode_and_errors(
            state_machines,
            &mut script_mode,
            true,
        )
    }

    fn settle_state_machine_update_passes_after_main_advance_with_mode_and_errors(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        reset_root_view_models: bool,
    ) -> Result<bool, ScriptError> {
        self.clear_script_update_error_tree();
        let changed = self
            .settle_state_machine_update_passes_after_main_advance_with_mode_and_root_vm_reset(
                state_machines,
                script_mode,
                reset_root_view_models,
            );
        match self.take_script_update_error_tree() {
            Some(error) => {
                self.rearm_pending_script_updates_tree();
                Err(error)
            }
            None => Ok(changed),
        }
    }

    fn reset_retained_components(&mut self) {
        if self.resetting_components.is_empty() {
            return;
        }
        for index in 0..self.resetting_components.len() {
            let entry = self.resetting_components[index];
            match entry.kind {
                ResettingComponentKind::NestedArtboard => {
                    let Some(nested) = self.nested_artboards.get_mut(&entry.local_id) else {
                        continue;
                    };
                    nested.child.reset_retained_components();
                    if let Some(context) = nested.stateful_view_model_context.as_mut() {
                        context.advanced_data_context();
                    }
                }
                ResettingComponentKind::ArtboardComponentList => {
                    let should_reset_instances =
                        self.artboard_component_list_should_reset_instances(entry.local_id);
                    let Some(list) = self.component_list_state_mut(entry.local_id) else {
                        continue;
                    };
                    reset_component_list_instances(list, should_reset_instances);
                }
                ResettingComponentKind::CustomPropertyTrigger => {
                    let Some(property_value_key) =
                        property_key_for_name("CustomPropertyTrigger", "propertyValue")
                    else {
                        continue;
                    };
                    let _ = self.set_uint_property(entry.local_id, property_value_key, 0);
                }
            }
        }
    }

    fn reset_outer_state_machine_changed_state_counts(
        &mut self,
        state_machines: &mut [StateMachineInstance],
    ) {
        for state_machine in state_machines {
            state_machine.reset_changed_state_count_for_outer_settlement();
        }
        for nested in self.nested_artboards.values_mut() {
            nested.reset_outer_state_machine_changed_state_counts();
        }
        for list_index in 0..self.component_list_count() {
            let Some(local_id) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for item in items {
                for state_machine in &mut item.state_machines {
                    state_machine.reset_changed_state_count_for_outer_settlement();
                }
                item.child
                    .reset_outer_state_machine_changed_state_counts(&mut []);
            }
        }
    }

    /// Mirrors `Artboard::advanceInternal` for an outer state-machine update
    /// pass, where `AdvanceNested` is set but `NewFrame` is not.
    fn advance_outer_update_components(&mut self) -> bool {
        // One insertion-ordered mixed-family walk, exactly like
        // `Artboard::m_advancingComponents`; NewFrame is deliberately absent
        // during the outer settlement pass (`artboard.cpp:1463-1480`).
        let mut changed = self.advance_retained_components_collect_events(0.0, false, None);
        changed |= self.advance_artboard_data_binds_with_elapsed(0.0);
        changed
    }

    fn update_nested_artboard_from_host_dirt(
        &mut self,
        host_local_id: usize,
        dirt: ComponentDirt,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        root_transform: Mat2D,
    ) -> bool {
        if !dirt.contains(ComponentDirt::RENDER_OPACITY)
            && !dirt.contains(ComponentDirt::COMPONENTS)
        {
            return false;
        }
        let mut changed = false;
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            changed |= self.sync_nested_artboard_root_opacity(host_local_id);
        }
        if dirt.contains(ComponentDirt::COMPONENTS) {
            let newly_uncollapsed = self
                .newly_uncollapsed_nested_artboards
                .remove(&host_local_id);
            let is_remap_host = self
                .nested_artboards
                .get(&host_local_id)
                .is_some_and(|nested| {
                    nested.animations.iter().any(|animation| {
                        matches!(animation, RuntimeNestedAnimationInstance::Remap { .. })
                    })
                });
            let child_has_component_dirt = self
                .nested_artboards
                .get(&host_local_id)
                .is_some_and(|nested| nested.child.has_dirt(ComponentDirt::COMPONENTS));
            let host_has_data_bindings = self.has_artboard_data_bindings();
            if newly_uncollapsed
                && is_remap_host
                && dirt.contains(ComponentDirt::RENDER_OPACITY)
                && (!child_has_component_dirt || !host_has_data_bindings)
            {
                return changed;
            }
            let child_root_transform = root_transform.multiply(
                self.component(host_local_id)
                    .map(|component| component.transform.world_transform)
                    .unwrap_or(Mat2D::IDENTITY),
            );
            if let Some(nested) = self.nested_artboards.get_mut(&host_local_id) {
                changed |= nested
                    .child
                    .update_pass_with_script_mode(script_mode, child_root_transform);
                if dirt.contains(ComponentDirt::RENDER_OPACITY) {
                    if let Some(frame) = nested.initial_layout_paint_frame.borrow().as_ref() {
                        // C++ consumes the initial nested-layout shader wave in
                        // `NestedArtboard::update(Filthy)`, after the mounted
                        // child's `updatePass(false)` has propagated inherited
                        // opacity (`nested_artboard.cpp:634-652`). The isolated
                        // Rust frame stands in for precisely that wave, so clear
                        // owner events only here; a later Components-only update
                        // remains live and produces the next shader event.
                        nested
                            .child
                            .transfer_owned_shape_gradient_events_to_initial_frame(frame);
                    } else {
                        // C++ mounts the child with the host's current render
                        // opacity, then performs this one recursive update from
                        // `NestedArtboard::update(Filthy)`
                        // (`nested_artboard.cpp:110-135, 626-652`). Rust's
                        // renderer factory attaches later, so any owner states
                        // observed before that host update are implementation
                        // history, not additional C++ shader events. Preserve
                        // the post-update retained state that the factory would
                        // have observed at this exact ownership boundary.
                        nested.child.retain_latest_unrealized_shape_gradient_state();
                    }
                }
            }
        }
        changed
    }

    fn has_artboard_data_bindings(&self) -> bool {
        !self.artboard_property_bindings.is_empty()
            || !self.artboard_image_asset_bindings.is_empty()
            || !self.artboard_custom_property_bindings.is_empty()
            || !self.artboard_layout_computed_bindings.is_empty()
            || !self.artboard_numeric_source_bindings.is_empty()
            || !self.artboard_formula_token_bindings.is_empty()
            || !self.artboard_converter_property_bindings.is_empty()
            || !self.artboard_solo_bindings.is_empty()
            || !self.artboard_solo_source_bindings.is_empty()
            || !self.artboard_nested_host_bindings.is_empty()
            || !self.artboard_list_bindings.is_empty()
    }

    fn sync_nested_artboard_root_opacity(&mut self, host_local_id: usize) -> bool {
        let Some(host_opacity) = self
            .component(host_local_id)
            .map(|component| component.transform.render_opacity)
        else {
            return false;
        };
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };
        nested.set_root_opacity(host_opacity)
    }

    pub fn update_components_with_hook<F>(&mut self, mut hook: F) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt),
    {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_components_with_hook_recording(
            true,
            &mut script_mode,
            Mat2D::IDENTITY,
            |instance, local_id, dirt, _| {
                hook(instance, local_id, dirt);
            },
        )
    }

    fn update_components_with_hook_recording<F>(
        &mut self,
        record_updated_locals: bool,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        root_transform: Mat2D,
        mut hook: F,
    ) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt, &mut RuntimeScriptUpdateMode<'_>),
    {
        self.initialize_clone_backend_if_pending();
        let mut report = UpdateComponentsReport::default();
        let graph_owner = self.build_context.as_ref().and_then(|context| {
            let graph_index = context
                .artboard_index_by_global
                .get(usize::try_from(self.graph_global_id).ok()?)
                .copied()
                .flatten()?;
            Some((Arc::clone(&context.artboards), graph_index))
        });
        // Core::clone copies generated properties into fresh concrete
        // Components, then Artboard::initialize runs one FILTHY traversal over
        // the clone-owned graph. InstanceObjectArena performs that concrete
        // lifecycle rebuild; consume its one-shot wake here instead of
        // polling retained renderer sidecars (`artboard.hpp:557-588`,
        // `artboard.cpp:1214-1248`).
        if !self.has_dirt(ComponentDirt::COMPONENTS) {
            return report;
        }

        // C++ layout propagation settles control sizes before Path::update.
        // Root occurrences do not use `layout_constraint_bounds` as a durable
        // nested-layout override, so compute the same solved frame locally for
        // this dependency traversal. Keep this after the clean-frame return:
        // an unchanged C++ update does not solve the layout tree.
        let layout_bounds = self.layout_constraint_bounds.clone().or_else(|| {
            let (graphs, graph_index) = graph_owner.as_ref()?;
            self.runtime_taffy_layout_bounds(&graphs[*graph_index], self.runtime_file())
                .map(Arc::new)
        });
        if let Some(layout_bounds) = layout_bounds.as_deref() {
            for (&local_id, &bounds) in layout_bounds {
                self.retain_runtime_layout_component_bounds(local_id, bounds, Some(layout_bounds));
            }
        }

        report.did_update = true;
        let max_steps = 100;
        let update_order_len = self.objects.dependency_order().len();

        while self.has_dirt(ComponentDirt::COMPONENTS) && report.steps < max_steps {
            if let Some(root) = self.objects.root()
                && let Some(component) = self.objects.component_mut(root)
            {
                component.dirt &= !ComponentDirt::COMPONENTS;
            }

            for order_index in 0..update_order_len {
                self.dirt_depth = order_index;
                let Some(component_handle) = self.objects.scheduled_at(order_index) else {
                    continue;
                };
                let Some(component) = self.objects.component(component_handle) else {
                    continue;
                };
                let local_id = component.local_id;
                let dirt = component.dirt;
                if dirt.is_empty() || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }
                let Some(address) = self.objects.address(component_handle) else {
                    continue;
                };
                let scheduled_component = self
                    .objects
                    .component_mut(component_handle)
                    .expect("scheduled component handle must remain live");
                scheduled_component.dirt = ComponentDirt::NONE;

                match address {
                    ComponentAddress::Authored(_) => {
                        self.update_component_with_script_mode(
                            component_handle,
                            dirt,
                            script_mode,
                            root_transform,
                        );
                        if let Some((graphs, graph_index)) = graph_owner.as_ref() {
                            self.update_runtime_path_owner(
                                component_handle,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            );
                            self.update_runtime_clipping_shape_owner(local_id, dirt);
                            self.update_runtime_artboard_render_paths(
                                local_id,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            );
                            self.update_runtime_shape_paints_at_dependency_node(
                                local_id,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            );
                            self.update_runtime_mesh_owner(
                                component_handle,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            );
                        }
                        if record_updated_locals {
                            report.updated_locals.push(local_id);
                        }
                        hook(self, local_id, dirt, script_mode);
                    }
                    ComponentAddress::PathComposer(shape) => {
                        let shape_local = shape.local_id();
                        if self
                            .component(shape_local)
                            .is_some_and(RuntimeComponent::is_collapsed)
                        {
                            continue;
                        }
                        if let Some((graphs, graph_index)) = graph_owner.as_ref() {
                            self.update_runtime_path_composer(
                                shape_local,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            );
                        }
                    }
                    ComponentAddress::TextVariationHelper { text, .. } => {
                        self.update_runtime_text_variation_helper(text, dirt);
                    }
                }

                if self.dirt_depth < order_index {
                    break;
                }
            }

            report.steps += 1;
        }

        report.max_steps_reached = self.has_dirt(ComponentDirt::COMPONENTS);
        report
    }

    /// Complete only clone-owned backend initialization before Component
    /// traversal. Component/DataBind ownership links are rebuilt synchronously
    /// by the clone itself, matching C++'s observable clone boundary.
    ///
    pub(super) fn initialize_clone_backend_if_pending(&mut self) {
        if !self.objects.take_clone_backend_initialization_pending() {
            return;
        }
        self.mark_components_dirty();
        let graph_owner = self.build_context.as_ref().and_then(|context| {
            let graph_index = context
                .artboard_index_by_global
                .get(usize::try_from(self.graph_global_id).ok()?)
                .copied()
                .flatten()?;
            Some((Arc::clone(&context.artboards), graph_index))
        });
        let Some((graphs, graph_index)) = graph_owner else {
            return;
        };
        // ShapePaint::onAddedClean creates the clone-owned RenderPaint before
        // Artboard::initialize walks FILTHY Components
        // (`shape_paint.cpp:12-57`, `artboard.hpp:557-588`).
        self.initialize_runtime_shape_paint_owners(&graphs[graph_index]);
    }

    pub(super) fn on_component_dirty_handle(&mut self, handle: ComponentHandle) {
        self.mark_changed();
        self.mark_components_dirty();

        if let Some(order) = self.objects.graph_order(handle)
            && order.index() < self.dirt_depth
        {
            self.dirt_depth = order.index();
        }
    }

    fn mark_components_dirty(&mut self) -> bool {
        // Artboard is itself the root Component in pinned C++. Publishing
        // Components dirt therefore uses the same equality guard as every
        // other `Component::addDirt` before assigning the retained root mask
        // (`src/component.cpp:32-45`; `src/artboard.cpp:1205-1241`).
        let Some(root) = self.objects.root() else {
            return false;
        };
        let Some(component) = self.objects.component_mut(root) else {
            return false;
        };
        if component.dirt.contains(ComponentDirt::COMPONENTS) {
            return false;
        }
        component.dirt |= ComponentDirt::COMPONENTS;
        true
    }

    pub(crate) fn update_component(
        &mut self,
        component_handle: ComponentHandle,
        dirt: ComponentDirt,
    ) {
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_component_with_script_mode(
            component_handle,
            dirt,
            &mut script_mode,
            Mat2D::IDENTITY,
        );
    }

    fn update_component_with_script_mode(
        &mut self,
        component_handle: ComponentHandle,
        dirt: ComponentDirt,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        root_transform: Mat2D,
    ) {
        let local_id = self
            .objects
            .component(component_handle)
            .expect("component handle must remain live")
            .local_id;
        if self
            .objects
            .component(component_handle)
            .and_then(|component| component.concrete.follow_path.as_ref())
            .is_some()
        {
            crate::constraints::update_follow_path_constraint(self, component_handle);
        }
        if dirt.contains(ComponentDirt::TRANSFORM) {
            let authored = self.authored_transform(local_id);
            self.objects
                .component_mut(component_handle)
                .expect("component handle must remain live")
                .update_transform(authored);
        }
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            if self
                .objects
                .component(component_handle)
                .is_some_and(|component| component.concrete.constrainable_list.is_some())
            {
                // C++ `ArtboardComponentList::updateWorldTransform` rebuilds
                // non-virtual row transforms before calling
                // `TransformComponent::updateWorldTransform`. Virtualized
                // transforms remain owned by ScrollVirtualizer; draw only
                // reads `m_artboardTransforms`
                // (`artboard_component_list.cpp:1300-1331`).
                if component_list_virtualization(self, local_id).is_none() {
                    let transforms = runtime_component_list_item_base_transforms(self, local_id);
                    if let Some(list) = self.component_list_state_mut(local_id) {
                        list.item_transforms = transforms;
                    }
                }
            }
            let parent = self
                .objects
                .component(component_handle)
                .expect("component handle must remain live")
                .parent;
            let parent_world = parent
                .and_then(|parent| self.objects.component(parent))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.world_transform);
            let layout_position = self
                .objects
                .component(component_handle)
                .and_then(|component| component.concrete.layout.as_ref())
                .map(|layout| layout.position());
            if let (Some(parent), Some((mut x, mut y))) = (parent, layout_position) {
                // `LayoutComponent::update` first calls Super::update, which
                // computes the ordinary TransformComponent world and invokes
                // the virtual LayoutComponent::updateConstraints. It then
                // replaces that world with the retained Yoga left/top
                // translation and invokes the same constraints a second
                // time. The first constrained world is deliberately
                // overwritten, but its owner-side rendezvous effects
                // (including ScrollConstraint child counts/virtualization)
                // remain observable (`transform_component.cpp:73-103`;
                // `layout_component.cpp:82-121,186-196`).
                self.objects
                    .component_mut(component_handle)
                    .expect("component handle must remain live")
                    .update_world_transform(parent_world);
                crate::constraints::apply_parent_layout_constraints(self, component_handle);
                crate::constraints::apply_list_constraints(self, component_handle);
                crate::constraints::apply_constraints(self, component_handle);

                // The parent Artboard's normalized origin is removed in the
                // LayoutComponent-owned replacement world.
                if self
                    .objects
                    .component(parent)
                    .is_some_and(|parent| parent.type_name == "Artboard")
                {
                    x -= self.width * self.origin_x;
                    y -= self.height * self.origin_y;
                }
                let local = Mat2D([1.0, 0.0, 0.0, 1.0, x, y]);
                self.objects
                    .component_mut(component_handle)
                    .expect("component handle must remain live")
                    .transform
                    .world_transform = parent_world.unwrap_or(Mat2D::IDENTITY).multiply(local);
            } else {
                self.objects
                    .component_mut(component_handle)
                    .expect("component handle must remain live")
                    .update_world_transform(parent_world);
            }
            crate::constraints::apply_parent_layout_constraints(self, component_handle);
            // ArtboardComponentList applies its retained list constraints
            // before ordinary Transform constraints, and the ordinary pass
            // skips ListConstraint subtypes
            // (`artboard_component_list.cpp:1333-1358`).
            crate::constraints::apply_list_constraints(self, component_handle);
            crate::constraints::apply_constraints(self, component_handle);
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            let previous_opacity = self
                .objects
                .component(component_handle)
                .expect("component handle must remain live")
                .transform
                .render_opacity;
            let opacity = self.authored_transform(local_id).opacity;
            let parent_opacity = self
                .objects
                .component(component_handle)
                .expect("component handle must remain live")
                .parent_transform
                .and_then(|parent| {
                    let component = self.objects.component(parent)?;
                    let key = component.transform_property_key(TransformProperty::Opacity)?;
                    let authored_opacity = self
                        .objects
                        .double_property(component.local_id, key)
                        .unwrap_or(1.0);
                    Some(component.child_opacity(authored_opacity))
                })
                .unwrap_or(1.0);
            self.objects
                .component_mut(component_handle)
                .expect("component handle must remain live")
                .update_render_opacity(opacity, parent_opacity);
            if self
                .objects
                .component(component_handle)
                .expect("component handle must remain live")
                .transform
                .render_opacity
                != previous_opacity
            {
                self.mark_render_opacity_changed();
            }
        }
        if self
            .objects
            .component(component_handle)
            .is_some_and(|component| component.concrete.constrainable_list.is_some())
        {
            // The concrete list owner runs these tails after Super::update:
            // mounted child update under Components dirt and inherited
            // Artboard opacity under RenderOpacity dirt
            // (`artboard_component_list.cpp:1254-1297`).
            if dirt.contains(ComponentDirt::RENDER_OPACITY) {
                let opacity = self.component_at(component_handle).transform.render_opacity;
                let opacity_key = property_key_for_name("Artboard", "opacity");
                if let Some(items) = self.component_list_items_mut(local_id) {
                    for item in items {
                        if let Some(key) = opacity_key {
                            item.child.set_double_property(0, key, opacity);
                        }
                    }
                }
            }
            if dirt.contains(ComponentDirt::COMPONENTS) {
                let roots = self
                    .runtime_component_list_child_root_transforms(root_transform)
                    .remove(&local_id)
                    .unwrap_or_default();
                if let Some(items) = self.component_list_items_mut(local_id) {
                    for (item_index, item) in items.iter_mut().enumerate() {
                        let child_root_transform =
                            roots.get(item_index).copied().unwrap_or(root_transform);
                        item.child
                            .update_pass_with_script_mode(script_mode, child_root_transform);
                    }
                }
            }
        }
        self.update_script_component_in_dependency_order(component_handle, dirt, script_mode);
        if self
            .objects
            .component(component_handle)
            .is_some_and(|component| component.concrete.skin.is_some())
        {
            let tendon_count = self
                .objects
                .component(component_handle)
                .and_then(|component| component.concrete.skin.as_ref())
                .map_or(0, |skin| skin.tendons.len());
            for index in 0..tendon_count {
                let transform = self
                    .objects
                    .component(component_handle)
                    .and_then(|component| component.concrete.skin.as_ref())
                    .and_then(|skin| skin.tendons.get(index))
                    .and_then(|tendon| self.objects.component(*tendon))
                    .and_then(|tendon| tendon.concrete.tendon.as_ref())
                    .and_then(|tendon| {
                        let bone = self.objects.component(tendon.bone?)?;
                        Some(bone.transform.world_transform.multiply(tendon.inverse_bind))
                    });
                if let Some(transform) = transform
                    && let Some(slot) = self
                        .objects
                        .component_mut(component_handle)
                        .and_then(|component| component.concrete.skin.as_mut())
                        .and_then(|skin| skin.bone_transforms.get_mut(index + 1))
                {
                    *slot = transform;
                }
            }
            #[cfg(test)]
            if let Some(skin) = self
                .objects
                .component_mut(component_handle)
                .and_then(|component| component.concrete.skin.as_mut())
            {
                skin.buffer_rebuilds += 1;
            }
        }
        if dirt.contains(ComponentDirt::DRAW_ORDER) {
            self.sort_runtime_draw_order();
        }
        if dirt.contains(ComponentDirt::CLIPPING) {
            self.refresh_runtime_drawable_save_operations();
        }
    }

    fn update_script_component_in_dependency_order(
        &mut self,
        component: ComponentHandle,
        dirt: ComponentDirt,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
    ) {
        if !dirt.contains(ComponentDirt::SCRIPT_UPDATE) {
            return;
        }
        let Some((global_id, type_name, pending)) =
            self.objects.component(component).and_then(|owner| {
                Some((
                    owner.global_id,
                    owner.type_name,
                    owner.concrete.scripted.as_ref()?.update_pending,
                ))
            })
        else {
            return;
        };
        if !pending {
            return;
        }
        self.set_script_owner_update_pending(component, false);
        if type_name == "ScriptedPathEffect" {
            // Path effects consume ScriptUpdate by invalidating their local
            // EffectPath and rearming advance; they do not call a user
            // `update` method (`src/scripted/scripted_path_effect.cpp:199-207`).
            self.set_script_owner_advance_active_handle(component, true);
            return;
        }

        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            self.set_script_owner_advance_active_handle(component, true);
            return;
        };
        let mut instance = handle.borrow_mut();
        let result = instance
            .has_method(ScriptMethod::Update)
            .and_then(|has_update| {
                if !has_update {
                    return Ok(ScriptValue::Nil);
                }
                script_mode.call(instance.as_mut(), &mut NoopScriptHost)
            });
        self.set_script_owner_advance_active_handle(component, true);
        if let Err(error) = result {
            self.set_script_owner_update_pending(component, true);
            if self.script_update_error.is_none() {
                self.script_update_error = Some(error);
            }
        }
    }

    pub(crate) fn apply_joysticks(&mut self, can_apply_before_update: bool) -> bool {
        let mut changed = false;
        let joystick_count = self.joysticks.len();
        for joystick_index in 0..joystick_count {
            if self.joysticks[joystick_index].can_apply_before_update == can_apply_before_update {
                changed |= self.apply_joystick_at(joystick_index);
            }
        }
        changed
    }

    fn update_runtime_text_variation_helper(&mut self, text: ComponentHandle, dirt: ComponentDirt) {
        if !dirt.contains(ComponentDirt::TEXT_SHAPE) {
            return;
        }
        if let Some(text_local) = self.component_local_id(text) {
            // C++ rebuilds the variation-bearing Font on the helper update
            // (`src/text/text_variation_helper.cpp:14-17`,
            // `src/text/text_style.cpp:98-124`). Rust's retained text owner
            // rebuilds lazily from the same live axis values, so invalidate
            // precisely that Text occurrence here.
            self.runtime_drawables
                .mark_text_resource_dirty_for_local(text_local);
        }
    }

    fn apply_joystick_at(&mut self, joystick_index: usize) -> bool {
        // Mirrors C++ Artboard::updatePass / Joystick::apply: iterate retained
        // joystick entries instead of cloning the joystick list per pass.
        let Some(joystick) = self.joysticks.get(joystick_index) else {
            return false;
        };
        let local_id = joystick.local_id;
        let x_animation_index = joystick.x_animation_index;
        let y_animation_index = joystick.y_animation_index;
        let nested_remap_dependents_len = joystick.nested_remap_dependents.len();

        let mut changed = false;
        if let Some(animation_index) = x_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, true) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        if let Some(animation_index) = y_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, false) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        for dependent_index in 0..nested_remap_dependents_len {
            let remap_local_id =
                self.joysticks[joystick_index].nested_remap_dependents[dependent_index];
            changed |= self.advance_nested_remap_animation(remap_local_id);
        }
        changed
    }

    pub(crate) fn joystick_axis_seconds(
        &self,
        local_id: usize,
        animation_index: usize,
        is_x_axis: bool,
    ) -> Option<f32> {
        let animation = self.linear_animation(animation_index)?;
        let axis_key = if is_x_axis {
            joystick_x_property_key()
        } else {
            joystick_y_property_key()
        }?;
        let flag = if is_x_axis {
            JOYSTICK_FLAG_INVERT_X
        } else {
            JOYSTICK_FLAG_INVERT_Y
        };
        let mut axis = self.double_property(local_id, axis_key).unwrap_or(0.0);
        let flags = joystick_flags_property_key()
            .and_then(|key| self.uint_property(local_id, key))
            .unwrap_or(0);
        if flags & flag != 0 {
            axis = -axis;
        }
        Some(((axis + 1.0) / 2.0) * animation.duration_seconds())
    }

    pub(crate) fn apply_uint_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        let mut changed = self
            .component(local_id)
            .and_then(|component| component.concrete.constraint)
            .is_some_and(|constraint| {
                crate::constraints::constraint_uint_change_marks_parent_dirty(
                    constraint.kind,
                    property_key,
                ) && self.mark_constraint_parent_transform_dirty(local_id)
            });
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedArtboard")
            && property_key_for_name("NestedArtboard", "artboardId") == Some(property_key)
            && let Some(value) = self.uint_property(local_id, property_key)
        {
            changed |= self.set_nested_artboard_artboard_id(local_id, value);
        }
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("DrawRules")
            && property_key_for_name("DrawRules", "drawTargetId") == Some(property_key)
        {
            // C++ `DrawRules::drawTargetIdChanged` dirties the owning
            // Artboard, not the non-Component DrawRules object.
            changed |= self.add_dirt(0, ComponentDirt::DRAW_ORDER, false);
        }
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("DrawTarget")
            && property_key_for_name("DrawTarget", "placementValue") == Some(property_key)
        {
            // C++ `DrawTarget::placementValueChanged` dirties the owning
            // Artboard, not the non-Component DrawTarget object.
            changed |= self.add_dirt(0, ComponentDirt::DRAW_ORDER, false);
        }
        if solo_active_component_id_property_key() == Some(property_key) {
            if let Some(solo) = self.component_handle(local_id) {
                changed |= self.propagate_solo_collapse(solo);
            }
        }
        if layout_component_style_display_value_property_key() == Some(property_key) {
            changed |= self.propagate_layout_component_display_changed(local_id);
        }
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("LayoutComponentStyle")
            && [
                property_key_for_name("LayoutComponentStyle", "animationStyleType"),
                property_key_for_name("LayoutComponentStyle", "interpolationType"),
                property_key_for_name("LayoutComponentStyle", "interpolatorId"),
            ]
            .contains(&Some(property_key))
        {
            changed |= self.refresh_layout_component_animation_style(local_id);
        }
        changed |= self.apply_nested_trigger_property_changed(local_id, property_key);
        changed
    }

    fn apply_keyed_callback(&mut self, callback: RuntimeKeyedCallback) -> bool {
        let _seconds_delay = callback.seconds_delay;
        match self
            .slot(callback.target_local_id)
            .and_then(|slot| slot.type_name)
        {
            Some("CustomPropertyTrigger")
                if property_key_for_name("CustomPropertyTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("CustomPropertyTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    + 1;
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_bool_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        let constraint_kind = self
            .component_handle(local_id)
            .and_then(|handle| self.objects.component(handle))
            .and_then(|component| component.concrete.constraint)
            .map(|constraint| constraint.kind);
        if crate::constraints::follow_path_orient_property_key() == property_key
            && constraint_kind.is_some_and(|kind| {
                matches!(
                    kind,
                    crate::components::RuntimeConstraintKind::FollowPath
                        | crate::components::RuntimeConstraintKind::ListFollowPath
                )
            })
        {
            return self.mark_constraint_parent_transform_dirty(local_id);
        }
        if crate::constraints::IK_INVERT_DIRECTION_PROPERTY_KEY == property_key
            && constraint_kind == Some(crate::components::RuntimeConstraintKind::Ik)
        {
            return self.mark_ik_constraint_dirty(local_id);
        }
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("Artboard") if property_key_for_name("Artboard", "clip") == Some(property_key) => {
                if self.clip == value {
                    return false;
                }
                self.clip = value;
                true
            }
            Some("ClippingShape")
                if property_key_for_name("ClippingShape", "isVisible") == Some(property_key) =>
            {
                // C++ `ClippingShape::isVisibleChanged` dirties the owning
                // Artboard so its update refreshes save-operation elision.
                self.add_dirt(0, ComponentDirt::CLIPPING, false)
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "isPaused") == Some(property_key) =>
            {
                self.set_nested_artboard_is_paused(local_id, value)
            }
            Some("NestedBool")
                if property_key_for_name("NestedBool", "nestedValue") == Some(property_key) =>
            {
                let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id)
                else {
                    return false;
                };
                self.set_nested_state_machine_bool(state_machine_local_id, input_id, value)
            }
            Some("NestedSimpleAnimation")
                if property_key_for_name("NestedSimpleAnimation", "isPlaying")
                    == Some(property_key) =>
            {
                self.set_nested_simple_animation_is_playing(local_id, value)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_string_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("TextValueRun")
                if property_key_for_name("TextValueRun", "text") == Some(property_key) =>
            {
                self.mark_text_value_run_shape_dirty(local_id)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_color_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("GradientStop")
                if property_key_for_name("GradientStop", "colorValue") == Some(property_key) =>
            {
                self.mark_parent_gradient_stops_dirty(local_id)
            }
            _ => false,
        }
    }

    fn mark_text_value_run_shape_dirty(&mut self, run_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(text_local) = self
            .uint_property(run_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if self.slot(text_local).and_then(|slot| slot.type_name) != Some("Text") {
            return false;
        }

        let mut changed = false;
        changed |= self.add_dirt(text_local, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::WORLD_TRANSFORM, true);
        changed
    }

    pub(crate) fn mark_text_style_shape_dirty(&mut self, style_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(text_local) = self
            .uint_property(style_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if !matches!(
            self.slot(text_local).and_then(|slot| slot.type_name),
            Some("Text" | "TextInput")
        ) {
            return false;
        }

        let mut changed = false;
        changed |= self.add_dirt(style_local_id, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::WORLD_TRANSFORM, true);
        changed
    }

    fn propagate_layout_component_display_changed(&mut self, style_local_id: usize) -> bool {
        let Some(style) = self.component_handle(style_local_id) else {
            return false;
        };
        let Some(layout) = self.objects.component(style).and_then(|component| {
            (component.type_name == "LayoutComponentStyle")
                .then_some(component.parent)
                .flatten()
        }) else {
            return false;
        };
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };

        // LayoutComponentStyle::markLayoutNodeDirty calls its retained
        // Component parent directly; it never scans the Artboard for matching
        // styleId values (`layout_component_style.cpp:208-221`).
        self.propagate_layout_component_display_collapse(layout_local)
            | self.add_dirt(layout_local, ComponentDirt::LAYOUT_STYLE, false)
    }

    fn refresh_layout_component_animation_style(&mut self, style_local_id: usize) -> bool {
        let Some(style) = self.component_handle(style_local_id) else {
            return false;
        };
        let Some(layout) = self
            .objects
            .component(style)
            .and_then(|component| component.parent)
            .filter(|layout| {
                self.objects
                    .component(*layout)
                    .is_some_and(|component| component.concrete.layout.is_some())
            })
        else {
            return false;
        };
        let animation_style = property_key_for_name("LayoutComponentStyle", "animationStyleType")
            .and_then(|key| self.uint_property(style_local_id, key))
            .unwrap_or(0) as u8;
        let interpolation = property_key_for_name("LayoutComponentStyle", "interpolationType")
            .and_then(|key| self.uint_property(style_local_id, key))
            .unwrap_or(0) as u8;
        let interpolation_time = property_key_for_name("LayoutComponentStyle", "interpolationTime")
            .and_then(|key| self.double_property(style_local_id, key))
            .unwrap_or(0.0);
        let interpolator = property_key_for_name("LayoutComponentStyle", "interpolatorId")
            .and_then(|key| self.uint_property(style_local_id, key))
            .and_then(|local_id| usize::try_from(local_id).ok())
            .and_then(|local_id| self.slot(local_id))
            .and_then(|slot| self.runtime_file()?.object(slot.source_global_id as usize))
            .and_then(RuntimeInterpolator::from_object);
        let Some(layout_state) = self
            .objects
            .component(layout)
            .and_then(|component| component.concrete.layout.as_ref())
        else {
            return false;
        };
        layout_state.set_animation_style(
            animation_style,
            interpolation,
            interpolation_time,
            interpolator,
        );
        self.cascade_layout_component_animation_style(layout);
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };
        self.add_dirt(layout_local, ComponentDirt::LAYOUT_STYLE, false)
    }

    fn cascade_layout_component_animation_style(&self, parent: ComponentHandle) {
        let inherited = self
            .objects
            .component(parent)
            .and_then(|component| component.concrete.layout.as_ref())
            .map(|layout| {
                (
                    layout.effective_interpolation(),
                    layout.effective_interpolation_time(),
                    layout.effective_interpolator(),
                )
            })
            .unwrap_or((0, 0.0, None));
        let child_len = self.component_child_len(parent);
        for index in 0..child_len {
            let Some(child) = self.component_child_at(parent, index) else {
                continue;
            };
            let Some(layout) = self
                .objects
                .component(child)
                .and_then(|component| component.concrete.layout.as_ref())
            else {
                continue;
            };
            layout.set_inherited_animation_style(inherited.0, inherited.1, inherited.2);
            self.cascade_layout_component_animation_style(child);
        }
    }

    fn apply_initial_component_collapse_callbacks_in_authored_order(&mut self) -> bool {
        // `Artboard::initialize` invokes every object's `onAddedClean` in
        // authored order. LayoutComponent and Solo both propagate collapse
        // from that callback, so their relative order is observable: a later
        // Solo must be allowed to re-collapse its inactive branch after an
        // earlier ancestor LayoutComponent has propagated its display state
        // (`src/artboard.cpp:316-374`; `src/layout_component.cpp:303-314`;
        // `src/solo.cpp:44-52`).
        let component_handles = self.objects.component_handles().to_vec();
        let mut changed = false;
        for handle in component_handles {
            let Some(component) = self.objects.component(handle) else {
                continue;
            };
            let local_id = component.local_id;
            if component.concrete.solo.is_some() {
                changed |= self.propagate_solo_collapse(handle);
            } else if matches!(component.type_name, "Artboard" | "LayoutComponent") {
                changed |= self.propagate_layout_component_display_collapse(local_id);
            }
        }
        changed
    }

    fn rederive_initial_component_collapse(&mut self) {
        self.apply_initial_component_collapse_callbacks_in_authored_order();
    }

    fn propagate_layout_component_display_collapse(&mut self, layout_local: usize) -> bool {
        self.propagate_layout_component_display_collapse_with_ancestor(layout_local, false)
    }

    // Mirrors C++ src/layout_component.cpp LayoutComponent::propagateCollapse:
    // the propagated value folds in the local display:none state, and each
    // child receives a full-subtree collapse (ContainerComponent::collapse).
    fn propagate_layout_component_display_collapse_with_ancestor(
        &mut self,
        layout_local: usize,
        ancestor_changed: bool,
    ) -> bool {
        // Cycle guard: this and collapse_component_tree_with_ancestor recurse
        // mutually over parent_local-derived children, which a malformed-but-
        // accepted file can make cyclic -> unbounded recursion. Thread a visited
        // set (C++'s DependencySorter::visit idiom, src/dependency_sorter.cpp);
        // on a valid file every component has one parent, so each local is
        // visited at most once and the guard is a no-op.
        let mut visited = BTreeSet::new();
        let Some(layout) = self.component_handle(layout_local) else {
            return false;
        };
        self.propagate_layout_component_display_collapse_with_ancestor_guarded(
            layout,
            ancestor_changed,
            &mut visited,
        )
    }

    fn propagate_layout_component_display_collapse_with_ancestor_guarded(
        &mut self,
        layout: ComponentHandle,
        ancestor_changed: bool,
        visited: &mut BTreeSet<ComponentHandle>,
    ) -> bool {
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };
        let display_hidden =
            self.layout_component_style_local(layout_local)
                .and_then(|style_local| {
                    layout_component_style_display_value_property_key()
                        .and_then(|key| self.uint_property(style_local, key))
                })
                == Some(1);
        let collapsed = display_hidden
            || self
                .component(layout_local)
                .is_some_and(RuntimeComponent::is_collapsed);
        let children = (0..self.component_child_len(layout))
            .filter_map(|index| self.component_child_at(layout, index))
            .collect::<Vec<_>>();

        let mut changed = false;
        for child in children {
            changed |= self.collapse_component_tree_with_ancestor_guarded(
                child,
                collapsed,
                ancestor_changed,
                visited,
            );
        }
        changed
    }

    fn layout_component_style_local(&self, layout_local: usize) -> Option<usize> {
        self.component_handle(layout_local)
            .and_then(|layout| self.objects.component(layout))
            .and_then(|component| component.concrete.layout.as_ref())
            .and_then(|layout| layout.style)
            .and_then(|style| self.objects.component_local_id(style))
    }

    pub(crate) fn apply_double_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if type_name == Some("LayoutComponentStyle")
            && property_key_for_name("LayoutComponentStyle", "interpolationTime")
                == Some(property_key)
        {
            return self.refresh_layout_component_animation_style(local_id);
        }
        if self
            .component(local_id)
            .and_then(|component| component.concrete.constraint)
            .is_some_and(|constraint| {
                crate::constraints::constraint_is_ik_strength_property(
                    constraint.kind,
                    property_key,
                )
            })
        {
            return self.mark_ik_constraint_dirty(local_id);
        }
        if self
            .component(local_id)
            .and_then(|component| component.concrete.constraint)
            .is_some_and(|constraint| {
                crate::constraints::constraint_double_change_marks_parent_dirty(
                    constraint.kind,
                    property_key,
                )
            })
        {
            return self.mark_constraint_parent_transform_dirty(local_id);
        }
        if self
            .component(local_id)
            .and_then(|component| component.concrete.bone.as_ref())
            .is_some()
            && property_key_for_name("Bone", "length") == Some(property_key)
        {
            let Some(handle) = self.component_handle(local_id) else {
                return false;
            };
            let child_count = self
                .objects
                .component(handle)
                .and_then(|component| component.concrete.bone.as_ref())
                .map_or(0, |bone| bone.child_bones.len());
            for index in 0..child_count {
                let child = self
                    .objects
                    .component(handle)
                    .and_then(|component| component.concrete.bone.as_ref())
                    .and_then(|bone| bone.child_bones.get(index))
                    .copied();
                if let Some(child) = child {
                    self.mark_transform_dirty_handle(child);
                }
            }
            return true;
        }
        if path_vertex_property_affects_geometry(type_name, property_key) {
            // Direct port of the concrete Vertex callbacks through
            // PathVertex::markGeometryDirty and Path::markPathDirty. The
            // vertex never owns path dirt; its parent PointsPath does
            // (`vertex.cpp:14-15`, `straight_vertex.cpp:5`,
            // `cubic_{mirrored,asymmetric,detached}_vertex.cpp`,
            // `path_vertex.cpp:21-30`, `path.cpp:327-334`).
            if let Some(path_local) = self.component_parent_local(local_id)
                && let Some(path) = self.component_handle(path_local)
            {
                if let Some(skin) = self
                    .objects
                    .component(path)
                    .and_then(|component| component.concrete.skinnable.as_ref())
                    .and_then(|skinnable| skinnable.skin)
                {
                    self.add_component_dirt(skin, ComponentDirt::SKIN, false);
                }
                self.add_component_dirt(path, ComponentDirt::PATH, false);
            }
            return true;
        }

        if let Some(property) = transform_property_for_key(property_key) {
            match property {
                TransformProperty::Opacity => {
                    self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
                }
                TransformProperty::X
                | TransformProperty::Y
                | TransformProperty::Rotation
                | TransformProperty::ScaleX
                | TransformProperty::ScaleY => {
                    let Some(handle) = self.component_handle(local_id) else {
                        return false;
                    };
                    self.mark_transform_dirty_handle(handle);
                }
            }
            return true;
        }

        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("TextStyleAxis")
                if property_key_for_name("TextStyleAxis", "axisValue") == Some(property_key) =>
            {
                // `TextStyleAxis::axisValueChanged` dirties its TextStyle
                // parent; TextStyle::onDirty then dirties Text and the
                // embedded variation helper (`text_style_axis.cpp:27-30`,
                // `text_style.cpp:22-34`).
                self.component_parent_local(local_id)
                    .is_some_and(|style| self.add_dirt(style, ComponentDirt::TEXT_SHAPE, false))
            }
            Some("MeshVertex")
                if property_key_for_name("Vertex", "x") == Some(property_key)
                    || property_key_for_name("Vertex", "y") == Some(property_key) =>
            {
                // `MeshVertex::markGeometryDirty` calls its parent Mesh's
                // `markDrawableDirty`, which pushes Vertices dirt
                // (`src/shapes/mesh_vertex.cpp:5-8`,
                // `src/shapes/mesh.cpp:14-23`).
                let Some(mesh_local) = self.component_parent_local(local_id) else {
                    return false;
                };
                let Some(mesh) = self.component_handle(mesh_local) else {
                    return false;
                };
                if let Some(skin) = self
                    .objects
                    .component(mesh)
                    .and_then(|component| component.concrete.skinnable.as_ref())
                    .and_then(|skinnable| skinnable.skin)
                {
                    self.add_component_dirt(skin, ComponentDirt::SKIN, false);
                }
                self.add_component_dirt(mesh, ComponentDirt::VERTICES, false)
            }
            Some("AxisX" | "AxisY")
                if property_key_for_name("Axis", "offset") == Some(property_key) =>
            {
                // `Axis::offsetChanged` resolves NSlicerDetails from the
                // parent and pushes NSlicer dirt (`src/layout/axis.cpp:23-29`).
                self.component_parent_local(local_id)
                    .is_some_and(|slicer_local| {
                        self.add_dirt(slicer_local, ComponentDirt::N_SLICER, false)
                    })
            }
            Some("Artboard")
                if local_id == 0
                    && property_key_for_name("Artboard", "originX") == Some(property_key) =>
            {
                self.origin_x = value;
                self.add_dirt(
                    local_id,
                    ComponentDirt::PATH | ComponentDirt::COMPONENTS,
                    false,
                )
            }
            Some("Artboard")
                if local_id == 0
                    && property_key_for_name("Artboard", "originY") == Some(property_key) =>
            {
                self.origin_y = value;
                self.add_dirt(
                    local_id,
                    ComponentDirt::PATH | ComponentDirt::COMPONENTS,
                    false,
                )
            }
            Some("Artboard")
                if local_id == 0
                    && (property_key_for_name("LayoutComponent", "width")
                        == Some(property_key)
                        || property_key_for_name("LayoutComponent", "height")
                            == Some(property_key)) =>
            {
                // Generated width/height callbacks mark the Yoga node dirty;
                // when the solved size changes C++ adds Path dirt and then
                // rebuilds the retained Artboard render paths in the same
                // update pass (`layout_component.cpp:1116-1124,1564-1565`,
                // `artboard.cpp:1138-1157`). Rust's layout solver is not a
                // dependency node, so publish that owner dirt at the callback
                // boundary.
                self.add_dirt(
                    local_id,
                    ComponentDirt::PATH | ComponentDirt::COMPONENTS,
                    false,
                )
            }
            Some("NestedArtboardOrigin")
                if property_key_for_name("NestedArtboardOrigin", "originX")
                    == Some(property_key)
                    || property_key_for_name("NestedArtboardOrigin", "originY")
                        == Some(property_key) =>
            {
                let Some(host_local_id) = self.component_parent_local(local_id) else {
                    return false;
                };
                let Some(origin_x_key) = property_key_for_name("Artboard", "originX") else {
                    return false;
                };
                let Some(origin_y_key) = property_key_for_name("Artboard", "originY") else {
                    return false;
                };
                let Some(origin_x) = property_key_for_name("NestedArtboardOrigin", "originX")
                    .and_then(|key| self.double_property(local_id, key))
                else {
                    return false;
                };
                let Some(origin_y) = property_key_for_name("NestedArtboardOrigin", "originY")
                    .and_then(|key| self.double_property(local_id, key))
                else {
                    return false;
                };
                let changed = self
                    .nested_artboards
                    .get_mut(&host_local_id)
                    .is_some_and(|nested| {
                        let mut changed =
                            nested.child.set_double_property(0, origin_x_key, origin_x);
                        changed |= nested.child.set_double_property(0, origin_y_key, origin_y);
                        changed
                    });
                if changed {
                    self.add_dirt(host_local_id, ComponentDirt::TRANSFORM, false);
                    self.add_dirt(host_local_id, ComponentDirt::WORLD_TRANSFORM, true);
                }
                changed
            }
            Some("LinearGradient" | "RadialGradient")
                if property_key_for_name("LinearGradient", "startX") == Some(property_key)
                    || property_key_for_name("LinearGradient", "startY") == Some(property_key)
                    || property_key_for_name("LinearGradient", "endX") == Some(property_key)
                    || property_key_for_name("LinearGradient", "endY") == Some(property_key) =>
            {
                self.add_dirt(local_id, ComponentDirt::TRANSFORM, false)
            }
            Some("LinearGradient" | "RadialGradient")
                if property_key_for_name("LinearGradient", "opacity") == Some(property_key) =>
            {
                self.add_dirt(local_id, ComponentDirt::PAINT, false)
            }
            Some("GradientStop")
                if property_key_for_name("GradientStop", "position") == Some(property_key) =>
            {
                self.mark_parent_gradient_stops_dirty(local_id)
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "speed") == Some(property_key) =>
            {
                self.set_nested_artboard_speed(local_id, value)
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "quantize") == Some(property_key) =>
            {
                self.set_nested_artboard_quantize(local_id, value)
            }
            Some("NestedNumber")
                if property_key_for_name("NestedNumber", "nestedValue") == Some(property_key) =>
            {
                let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id)
                else {
                    return false;
                };
                self.set_nested_state_machine_number(state_machine_local_id, input_id, value)
            }
            Some("NestedRemapAnimation")
                if property_key_for_name("NestedRemapAnimation", "time") == Some(property_key) =>
            {
                self.set_nested_remap_time(local_id, value)
            }
            Some("NestedSimpleAnimation" | "NestedRemapAnimation")
                if property_key_for_name("NestedLinearAnimation", "mix") == Some(property_key) =>
            {
                self.set_nested_linear_animation_mix(local_id, value)
            }
            Some("NestedSimpleAnimation")
                if property_key_for_name("NestedSimpleAnimation", "speed")
                    == Some(property_key) =>
            {
                self.set_nested_simple_animation_speed(local_id, value)
            }
            Some("ScrollConstraint")
                if [
                    "scrollOffsetX",
                    "scrollOffsetY",
                    "scrollPercentX",
                    "scrollPercentY",
                    "scrollIndex",
                ]
                .into_iter()
                .any(|name| {
                    property_key_for_name("ScrollConstraint", name) == Some(property_key)
                }) =>
            {
                apply_scroll_offset_changed(self, local_id, property_key, value)
                    .unwrap_or_else(|| self.mark_constraint_parent_transform_dirty(local_id))
            }
            _ => false,
        }
    }

    fn mark_constraint_parent_transform_dirty(&mut self, constraint_local_id: usize) -> bool {
        let Some(parent) = self
            .component_handle(constraint_local_id)
            .and_then(|constraint| self.objects.component(constraint))
            .and_then(|constraint| constraint.parent)
        else {
            return false;
        };
        self.mark_transform_dirty_handle(parent)
    }

    fn mark_ik_constraint_dirty(&mut self, constraint_local_id: usize) -> bool {
        let Some(constraint) = self.component_handle(constraint_local_id) else {
            return false;
        };
        let Some(tip) = self
            .objects
            .component(constraint)
            .and_then(|constraint| constraint.parent)
        else {
            return false;
        };
        let mut changed = self.mark_transform_dirty_handle(tip);
        let chain_len = self
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.ik.as_ref())
            .map_or(0, |ik| ik.chain.len());
        for index in 0..chain_len.saturating_sub(1) {
            let bone = self
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.ik.as_ref())
                .and_then(|ik| ik.chain.get(index))
                .map(|link| link.bone);
            if let Some(bone) = bone {
                changed |= self.mark_transform_dirty_handle(bone);
            }
        }
        changed
    }

    fn mark_parent_gradient_stops_dirty(&mut self, stop_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(gradient_local_id) = self
            .uint_property(stop_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if !matches!(
            self.slot(gradient_local_id).and_then(|slot| slot.type_name),
            Some("LinearGradient" | "RadialGradient")
        ) {
            return false;
        }
        self.add_dirt(
            gradient_local_id,
            ComponentDirt::PAINT | ComponentDirt::STOPS,
            false,
        )
    }

    fn set_nested_artboard_is_paused(&mut self, local_id: usize, value: bool) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_is_paused(value)
    }

    fn set_nested_artboard_speed(&mut self, local_id: usize, value: f32) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_speed(value)
    }

    fn set_nested_artboard_quantize(&mut self, local_id: usize, value: f32) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_quantize(value)
    }

    fn insert_nested_artboard_local(&mut self, local_id: usize) {
        if let Err(index) = self.nested_artboard_locals.binary_search(&local_id) {
            self.nested_artboard_locals.insert(index, local_id);
        }
    }

    fn remove_nested_artboard_local(&mut self, local_id: usize) {
        if let Ok(index) = self.nested_artboard_locals.binary_search(&local_id) {
            self.nested_artboard_locals.remove(index);
        }
    }

    pub(crate) fn set_nested_artboard_artboard_id(&mut self, local_id: usize, value: u64) -> bool {
        self.set_nested_artboard_artboard_id_with_force(local_id, value, false)
    }

    pub(crate) fn replace_nested_artboard_artboard_id(
        &mut self,
        local_id: usize,
        value: u64,
    ) -> bool {
        self.set_nested_artboard_artboard_id_with_force(local_id, value, true)
    }

    fn set_nested_artboard_artboard_id_with_force(
        &mut self,
        local_id: usize,
        value: u64,
        force: bool,
    ) -> bool {
        // Mirrors C++ `NestedArtboard::updateArtboard`: `-1` is an explicit
        // null and tears down the mounted child, while any other target that
        // cannot be resolved (including the owning artboard itself) leaves the
        // outgoing child untouched.
        if value == u64::from(u32::MAX) {
            let changed = self.nested_artboards.remove(&local_id).is_some();
            if changed {
                self.remove_nested_artboard_local(local_id);
                self.mark_nested_structure_changed();
                self.stateful_nested_view_model_contexts_dirty = true;
                self.mark_artboard_data_bind_work_dirty();
                self.mark_changed();
                self.mark_prepared_changed();
            }
            return changed;
        }
        let Some(mut nested) = self.runtime_nested_artboard_instance_for_id(local_id, value) else {
            return false;
        };
        if !force
            && self
                .nested_artboards
                .get(&local_id)
                .is_some_and(|existing| {
                    existing.child.graph_global_id == nested.child.graph_global_id
                })
        {
            return false;
        }
        if let Some(existing) = self.nested_artboards.get(&local_id) {
            nested.reuse_owned_stateful_view_model_context(existing);
        }
        nested.render_cache_revision = self.nested_artboards.get(&local_id).map_or(0, |existing| {
            if existing.child.graph_global_id == nested.child.graph_global_id {
                existing.render_cache_revision.saturating_add(1)
            } else {
                0
            }
        });
        self.nested_artboards.insert(local_id, nested);
        self.insert_nested_artboard_local(local_id);
        self.mark_nested_structure_changed();
        if let Some(file) = self.runtime_file_arc() {
            self.rebind_owned_view_model_context_after_nested_artboard_swap(&file, local_id);
        }
        self.stateful_nested_view_model_contexts_dirty = true;
        self.mark_artboard_data_bind_work_dirty();
        self.sync_nested_artboard_root_opacity(local_id);
        self.mark_changed();
        self.mark_prepared_changed();
        true
    }

    fn runtime_nested_artboard_instance_for_id(
        &self,
        host_local_id: usize,
        artboard_id: u64,
    ) -> Option<RuntimeNestedArtboardInstance> {
        if artboard_id == u64::from(u32::MAX) {
            return None;
        }
        let context = self.build_context.as_ref()?;
        let artboard_index = usize::try_from(artboard_id).ok()?;
        let referenced = context.file.artboard(artboard_index)?;
        let child_graph = context
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced.id)?;
        if child_graph.global_id == self.graph_global_id {
            return None;
        }
        let parent_graph = context
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == self.graph_global_id)?;
        let data_bind_path = self
            .slot(host_local_id)
            .and_then(|host| context.file.object(host.source_global_id as usize))
            .and_then(|host_object| {
                context
                    .file
                    .data_bind_path_for_referencer_object(host_object)
            });
        let data_bind_path_is_relative = data_bind_path
            .as_ref()
            .and_then(|path| path.object)
            .and_then(|path| path.bool_property("isRelative"))
            .unwrap_or(false);
        let data_bind_path_ids = data_bind_path.map(|path| {
            if data_bind_path_is_relative {
                path.path_ids
            } else {
                path.resolved_path_ids
            }
        });
        let mut visiting = BTreeSet::new();
        visiting.insert(self.graph_global_id);
        let nested = build_runtime_nested_artboard_instance(
            &context.file,
            parent_graph,
            context.artboards.as_slice(),
            &self.slots,
            &self.objects,
            host_local_id,
            child_graph,
            &mut visiting,
            Some(context.clone()),
            data_bind_path_ids,
            data_bind_path_is_relative,
            self.bool_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "isPaused")?,
            )
            .unwrap_or(false),
            self.double_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "speed")?,
            )
            .unwrap_or(1.0),
            self.double_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "quantize")?,
            )
            .unwrap_or(-1.0),
        )
        .ok()?;
        Some(nested)
    }

    fn apply_nested_trigger_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) != Some("NestedTrigger")
            || property_key_for_name("NestedTrigger", "fire") != Some(property_key)
        {
            return false;
        }
        self.fire_nested_trigger_input(local_id)
    }

    pub(crate) fn nested_input_target(&self, local_id: usize) -> Option<(usize, usize)> {
        let parent_key = property_key_for_name("Component", "parentId")?;
        let input_key = property_key_for_name("NestedInput", "inputId")?;
        let state_machine_local_id =
            usize::try_from(self.uint_property(local_id, parent_key)?).ok()?;
        let input_id = usize::try_from(self.uint_property(local_id, input_key)?).ok()?;
        Some((state_machine_local_id, input_id))
    }

    pub(crate) fn nested_state_machine(
        &self,
        state_machine_local_id: usize,
    ) -> Option<&StateMachineInstance> {
        self.nested_artboards
            .state_machine(state_machine_local_id)
            .and_then(RuntimeNestedStateMachineInstance::state_machine)
    }

    pub(crate) fn nested_state_machine_mut(
        &mut self,
        state_machine_local_id: usize,
    ) -> Option<&mut StateMachineInstance> {
        self.nested_artboards
            .state_machine_mut(state_machine_local_id)
            .and_then(RuntimeNestedStateMachineInstance::state_machine_mut)
    }

    pub(crate) fn set_nested_state_machine_bool(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
        value: bool,
    ) -> bool {
        let Some(state_machine) = self.nested_state_machine_mut(state_machine_local_id) else {
            return false;
        };
        if !state_machine.set_bool(input_id, value) {
            return false;
        }
        state_machine.schedule_post_update_probe();
        true
    }

    pub(crate) fn set_nested_state_machine_number(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
        value: f32,
    ) -> bool {
        let Some(state_machine) = self.nested_state_machine_mut(state_machine_local_id) else {
            return false;
        };
        if !state_machine.set_number(input_id, value) {
            return false;
        }
        state_machine.schedule_post_update_probe();
        true
    }

    pub(crate) fn fire_nested_state_machine_trigger(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
    ) -> bool {
        let Some(state_machine) = self.nested_state_machine_mut(state_machine_local_id) else {
            return false;
        };
        if !state_machine.fire_trigger(input_id) {
            return false;
        }
        state_machine.schedule_post_update_probe();
        true
    }

    fn set_nested_remap_time(&mut self, remap_local_id: usize, time: f32) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.set_remap_time(remap_local_id, time))
    }

    fn set_nested_linear_animation_mix(&mut self, local_id: usize, value: f32) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.set_animation_mix(local_id, value))
    }

    fn set_nested_simple_animation_speed(&mut self, local_id: usize, value: f32) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.set_simple_animation_speed(local_id, value))
    }

    fn set_nested_simple_animation_is_playing(&mut self, local_id: usize, value: bool) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.set_simple_animation_is_playing(local_id, value))
    }

    fn advance_nested_remap_animation(&mut self, remap_local_id: usize) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.advance_remap(remap_local_id))
    }

    pub(crate) fn apply_component_collapse_changed(&mut self, local_id: usize) -> bool {
        let Some(solo) = self.component_handle(local_id) else {
            return false;
        };
        self.propagate_solo_collapse(solo)
    }

    pub(crate) fn set_solo_active_child_by_index(
        &mut self,
        solo_local_id: usize,
        value: f32,
    ) -> bool {
        let rounded = value.round();
        if rounded < 0.0 || !rounded.is_finite() {
            return false;
        }
        let Some(solo) = self.component_handle(solo_local_id) else {
            return false;
        };
        let child_index = rounded as usize;
        let is_child = self
            .objects
            .component(solo)
            .and_then(|component| component.concrete.solo.as_ref().map(|_| component))
            .is_some_and(|component| child_index < component.children.len());
        if !is_child {
            return false;
        }
        self.set_solo_active_child(solo, child_index)
    }

    pub(crate) fn set_solo_active_child_by_name(
        &mut self,
        solo_local_id: usize,
        value: &[u8],
    ) -> bool {
        let Some(solo) = self.component_handle(solo_local_id) else {
            return false;
        };
        let child_count = self
            .objects
            .component(solo)
            .and_then(|component| {
                component
                    .concrete
                    .solo
                    .as_ref()
                    .map(|_| component.children.len())
            })
            .unwrap_or(0);
        for child_index in 0..child_count {
            let child_local_id = self
                .objects
                .component(solo)
                .and_then(|component| component.children.get(child_index).copied())
                .and_then(|child| self.objects.component_local_id(child));
            if child_local_id
                .and_then(|local_id| self.slot(local_id))
                .and_then(|slot| slot.name.as_deref())
                .is_some_and(|name| name.as_bytes() == value)
            {
                return self.set_solo_active_child(solo, child_index);
            }
        }
        false
    }

    pub(crate) fn set_solo_active_child(
        &mut self,
        solo: ComponentHandle,
        child_index: usize,
    ) -> bool {
        let Some((solo_local_id, active_component_property_key, cpp_local_id)) =
            self.objects.component(solo).and_then(|component| {
                let solo = component.concrete.solo.as_ref()?;
                Some((
                    component.local_id,
                    solo.active_component_property_key?,
                    *solo.cpp_local_ids.get(child_index)?,
                ))
            })
        else {
            return false;
        };
        let Ok(cpp_local_id) = u64::try_from(cpp_local_id) else {
            return false;
        };
        // C++ `Solo::updateByIndex`/`updateByName` writes the generated
        // Artboard object-table id; `activeComponentIdChanged` then invokes
        // `propagateCollapse` on this same occurrence (`src/solo.cpp:42-81`).
        self.set_uint_property(solo_local_id, active_component_property_key, cpp_local_id)
    }

    pub(crate) fn propagate_solo_collapse(&mut self, solo: ComponentHandle) -> bool {
        let Some((solo_local_id, solo_collapsed, active_component_property_key, child_count)) =
            self.objects.component(solo).and_then(|component| {
                let state = component.concrete.solo.as_ref()?;
                Some((
                    component.local_id,
                    component.is_collapsed(),
                    state.active_component_property_key?,
                    component.children.len().min(state.cpp_local_ids.len()),
                ))
            })
        else {
            return false;
        };

        let active_cpp_local = self
            .uint_property(solo_local_id, active_component_property_key)
            .and_then(|id| usize::try_from(id).ok());

        let mut changed = false;
        for child_index in 0..child_count {
            let Some((child, cpp_local_id, participates)) =
                self.objects.component(solo).and_then(|component| {
                    let child = *component.children.get(child_index)?;
                    let cpp_local_id = *component
                        .concrete
                        .solo
                        .as_ref()?
                        .cpp_local_ids
                        .get(child_index)?;
                    let child_type = self.objects.component(child)?.type_name;
                    let participates = definition_by_name(child_type).is_none_or(|definition| {
                        !definition.is_a("Constraint") && !definition.is_a("ClippingShape")
                    });
                    Some((child, cpp_local_id, participates))
                })
            else {
                continue;
            };
            let collapsed = if participates {
                solo_collapsed || Some(cpp_local_id) != active_cpp_local
            } else {
                solo_collapsed
            };
            let Some(child_local_id) = self.objects.component_local_id(child) else {
                continue;
            };
            changed |= self.collapse_component_tree(child_local_id, collapsed);
        }
        changed
    }

    pub(crate) fn collapse_component_tree(&mut self, local_id: usize, collapsed: bool) -> bool {
        self.collapse_component_tree_with_ancestor(local_id, collapsed, false)
    }

    pub(crate) fn collapse_component_tree_with_ancestor(
        &mut self,
        local_id: usize,
        collapsed: bool,
        ancestor_changed: bool,
    ) -> bool {
        // Cycle guard entry point: see
        // propagate_layout_component_display_collapse_with_ancestor.
        let mut visited = BTreeSet::new();
        let Some(handle) = self.component_handle(local_id) else {
            return false;
        };
        self.collapse_component_tree_with_ancestor_guarded(
            handle,
            collapsed,
            ancestor_changed,
            &mut visited,
        )
    }

    fn collapse_component_tree_with_ancestor_guarded(
        &mut self,
        handle: ComponentHandle,
        collapsed: bool,
        ancestor_changed: bool,
        visited: &mut BTreeSet<ComponentHandle>,
    ) -> bool {
        // Cycle guard: see propagate_layout_component_display_collapse_with_
        // ancestor. Skip a local already visited on this propagation walk.
        if !visited.insert(handle) {
            return false;
        }
        let changed_here = self.collapse_component_handle(handle, collapsed);
        let mut changed = changed_here;
        if ancestor_changed && !collapsed {
            changed |= self.add_component_dirt(handle, ComponentDirt::FILTHY, false);
        }
        let type_name = self
            .objects
            .component(handle)
            .map(|component| component.type_name);
        match type_name {
            // C++ Solo::collapse (src/solo.cpp) intentionally skips the blind
            // ContainerComponent child walk: Solo::propagateCollapse (already
            // triggered on change via collapse_component ->
            // apply_component_collapse_changed) re-collapses inactive children
            // even while the solo itself becomes visible.
            Some("Solo") => changed,
            // C++ LayoutComponent::collapse routes through
            // LayoutComponent::propagateCollapse, folding the local
            // display:none state into the value pushed onto children.
            Some("Artboard" | "LayoutComponent") => {
                changed
                    | self.propagate_layout_component_display_collapse_with_ancestor_guarded(
                        handle,
                        ancestor_changed || changed_here,
                        visited,
                    )
            }
            _ => {
                let children = (0..self.component_child_len(handle))
                    .filter_map(|index| self.component_child_at(handle, index))
                    .collect::<Vec<_>>();
                for child in children {
                    changed |= self.collapse_component_tree_with_ancestor_guarded(
                        child,
                        collapsed,
                        ancestor_changed || changed_here,
                        visited,
                    );
                }
                // TransformComponent's collapse tail runs after
                // ContainerComponent has propagated to children. Only
                // dependent TransformComponents that actually own
                // constraints receive recursive WorldTransform dirt
                // (`src/transform_component.cpp:18-44`).
                if self
                    .objects
                    .component(handle)
                    .is_some_and(|component| component.capabilities.transform)
                {
                    let dependent_count = self.objects.dependent_len(handle);
                    for index in 0..dependent_count {
                        let dependent = self.objects.dependent_at(handle, index);
                        let constrained_transform = dependent.is_some_and(|dependent| {
                            self.objects.component(dependent).is_some_and(|component| {
                                component.capabilities.transform
                                    && !component.constraints.is_empty()
                            })
                        });
                        if constrained_transform && let Some(dependent) = dependent {
                            changed |= self.add_component_dirt(
                                dependent,
                                ComponentDirt::WORLD_TRANSFORM,
                                true,
                            );
                        }
                    }
                }
                changed
            }
        }
    }
}

impl ArtboardInstance {
    pub(crate) fn install_external_focus_domain(
        &mut self,
        parent_focus: &crate::focus::RuntimeFocusTree,
    ) {
        self.external_focus_domain =
            Some(parent_focus.external_for_owner(self.instance_identity()));
        for (_, nested) in &mut self.nested_artboards.entries {
            nested.install_external_focus_domain(parent_focus);
        }
        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for list_local_id in list_locals {
            let Some(items) = self.component_list_items_mut(list_local_id) else {
                continue;
            };
            for item in items {
                let child_identity = item.child.instance_identity();
                for state_machine in &mut item.state_machines {
                    state_machine.install_external_focus(parent_focus, child_identity);
                }
                item.child.install_external_focus_domain(parent_focus);
            }
        }
    }
}

fn reset_component_list_instances(
    list: &mut RuntimeConstrainableListState,
    should_reset_instances: bool,
) {
    // C++ walks the complete logical `m_listItems` vector, acknowledges every
    // VMI, and only then looks up an optionally mounted row in
    // `m_artboardInstancesMap` (`artboard_component_list.cpp:888-920`).
    // Virtualization must not suppress reset for unmounted logical rows.
    let RuntimeConstrainableListState {
        logical_items,
        items,
        ..
    } = list;
    for logical in logical_items {
        if should_reset_instances {
            logical.context.borrow_mut().advanced_data_context();
        }
        let Some(item) = items
            .iter_mut()
            .find(|item| item.occurrence_identity == logical.occurrence_identity)
        else {
            continue;
        };
        if should_reset_instances {
            if let Some(bound_instance) = item
                .child
                .artboard_owned_view_model_context
                .as_ref()
                .and_then(RuntimeOwnedViewModelContext::main_handle)
                && !bound_instance.ptr_eq(&logical.context)
            {
                bound_instance.borrow_mut().advanced_data_context();
            }
        }
        item.child.reset_retained_components();
    }
}

// Ported from C++ Artboard::defaultStateMachineIndex and
// ArtboardComponentList::createStateMachineInstance. The serialized property
// is an index, despite its historical `Id` name. Missing and out-of-range
// values fall back to the first state machine for component-list children.
fn component_list_default_state_machine_index(
    default_state_machine_id: Option<u64>,
    state_machine_count: usize,
) -> usize {
    default_state_machine_id
        .and_then(|index| usize::try_from(index).ok())
        .filter(|&index| index < state_machine_count)
        .unwrap_or(0)
}

impl RuntimeNestedArtboardInstance {
    fn install_external_focus_domain(&mut self, parent_focus: &crate::focus::RuntimeFocusTree) {
        let child_identity = self.child.instance_identity();
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            occurrence.install_external_focus(parent_focus, child_identity);
        }
        self.child.install_external_focus_domain(parent_focus);
    }

    fn reuse_owned_stateful_view_model_context(&mut self, existing: &Self) -> bool {
        if self.stateful_view_model_instance_local.is_some()
            || existing.stateful_view_model_instance_local.is_some()
        {
            return false;
        }
        let Some(replacement_context) = self.stateful_view_model_context.as_ref() else {
            return false;
        };
        let Some(existing_context) = existing.stateful_view_model_context.as_ref() else {
            return false;
        };
        if replacement_context.view_model_index() != existing_context.view_model_index() {
            return false;
        }
        self.stateful_view_model_context = Some(existing_context.clone());
        true
    }

    fn has_ongoing_work(&self) -> bool {
        if self.is_paused {
            return false;
        }
        self.animations
            .iter()
            .any(|animation| animation.has_ongoing_work(&self.child))
            || self.child.has_ongoing_nested_work()
    }

    pub(crate) fn bind_owned_view_model_animation_contexts(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            changed |= occurrence.bind_owned_view_model_context_chain(file, context, context_chain);
        }
        changed
    }

    pub(crate) fn bind_owned_view_model_animation_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            changed |= occurrence.bind_owned_data_context(data_context);
        }
        changed
    }

    fn advance(
        &mut self,
        elapsed_seconds: f32,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> Result<bool, ScriptError> {
        if self.is_paused {
            return Ok(false);
        }

        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(elapsed_seconds);
        if local_elapsed_seconds == 0.0 && self.quantize >= 0.0 {
            // C++ returns before advancing nested animations on a quantized
            // NewFrame skip, then unconditionally probes nested state machines
            // during the following non-NewFrame outer pass.
            for animation in &mut self.animations {
                if let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation {
                    if let Some(state_machine) = occurrence.state_machine_mut() {
                        state_machine.schedule_post_update_probe();
                    }
                }
            }
            return Ok(true);
        }

        let mut changed = false;
        for animation in &mut self.animations {
            changed |= animation.advance(
                &mut self.child,
                local_elapsed_seconds,
                reported_events.as_mut().map(|events| &mut **events),
            );
        }
        // C++ advances the ENTIRE nested subtree before any data-bind pass
        // reaches it: `NestedArtboard::advanceComponent` only advances
        // animations and `advanceInternal` (src/nested_artboard.cpp:965-1008),
        // while the data binds — including the owned-path target-to-source
        // pulls — run later through `Artboard::updateDataBinds` recursing
        // artboard hosts first (src/artboard.cpp:1195-1201, called from
        // `updatePass` at src/artboard.cpp:1420). Advancing this child's
        // binds before its own nested artboards let a grandchild state
        // machine observe a reverse write one pass earlier than C++ (the
        // db_health_tracker blend consumed the pulled value on the first
        // frame where C++ still blends the pre-pull value).
        changed |= self
            .child
            .advance_retained_components_collect_events_with_scripts(
                local_elapsed_seconds,
                true,
                script_mode,
                None,
            )?;
        // Mirrors C++ src/nested_artboard.cpp NestedArtboard::updateDataBinds.
        changed |= self
            .child
            .advance_artboard_data_binds_with_elapsed(local_elapsed_seconds);
        Ok(changed)
    }

    fn reset_outer_state_machine_changed_state_counts(&mut self) {
        for animation in &mut self.animations {
            if let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation {
                if let Some(state_machine) = occurrence.state_machine_mut() {
                    state_machine.reset_changed_state_count_for_outer_settlement();
                }
            }
        }
        self.child
            .reset_outer_state_machine_changed_state_counts(&mut []);
    }

    /// Advance the non-`NewFrame` portion of C++
    /// `NestedArtboard::advanceComponent`: only state machines whose probe
    /// changes state are applied, followed by the child artboard's advancing
    /// components and data binds.
    fn advance_outer_update(&mut self) -> bool {
        if self.is_paused {
            return false;
        }

        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(0.0);
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            let Some(state_machine) = occurrence.state_machine_mut() else {
                continue;
            };
            if self.child.try_change_state_machine_instance(state_machine) {
                changed = true;
                changed |= self.child.advance_state_machine_instance_after_state_probe(
                    state_machine,
                    local_elapsed_seconds,
                );
            }
        }
        changed |= self.child.advance_outer_update_components();
        changed
    }

    // Mirrors src/nested_artboard.cpp NestedArtboard::calculateLocalElapsedSeconds.
    fn calculate_local_elapsed_seconds(&mut self, elapsed_seconds: f32) -> f32 {
        let mut local_elapsed_seconds =
            elapsed_seconds * if self.speed >= 0.0 { self.speed } else { 1.0 };
        if self.quantize >= 0.0 {
            self.cumulated_seconds += local_elapsed_seconds;
            let quantized_seconds = 1.0 / self.quantize;
            if self.cumulated_seconds > quantized_seconds {
                local_elapsed_seconds =
                    (self.cumulated_seconds / quantized_seconds).floor() * quantized_seconds;
                self.cumulated_seconds -= local_elapsed_seconds;
            } else {
                local_elapsed_seconds = 0.0;
            }
        }
        local_elapsed_seconds
    }

    fn set_root_opacity(&mut self, opacity: f32) -> bool {
        let Some(opacity_key) = property_key_for_name("Artboard", "opacity") else {
            return false;
        };
        self.child.set_double_property(0, opacity_key, opacity)
    }

    fn set_remap_time(&mut self, remap_local_id: usize, time: f32) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                ..
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id {
                continue;
            }
            let Some(linear_animation) = self.child.linear_animation(animation.animation_index())
            else {
                return false;
            };
            let seconds = linear_animation
                .global_to_local_seconds(linear_animation.duration_seconds() * time);
            animation.set_time(linear_animation, seconds);
            return true;
        }
        false
    }

    fn set_animation_mix(&mut self, local_id: usize, value: f32) -> bool {
        for animation in &mut self.animations {
            let (animation_local_id, mix) = match animation {
                RuntimeNestedAnimationInstance::Simple { local_id, mix, .. }
                | RuntimeNestedAnimationInstance::Remap { local_id, mix, .. } => (local_id, mix),
                RuntimeNestedAnimationInstance::StateMachine(_) => continue,
            };
            if *animation_local_id != local_id || *mix == value {
                continue;
            }
            *mix = value;
            return true;
        }
        false
    }

    fn set_simple_animation_speed(&mut self, local_id: usize, value: f32) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Simple {
                local_id: animation_local_id,
                speed,
                ..
            } = animation
            else {
                continue;
            };
            if *animation_local_id != local_id || *speed == value {
                continue;
            }
            *speed = value;
            return true;
        }
        false
    }

    fn set_simple_animation_is_playing(&mut self, local_id: usize, value: bool) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Simple {
                local_id: animation_local_id,
                is_playing,
                ..
            } = animation
            else {
                continue;
            };
            if *animation_local_id != local_id || *is_playing == value {
                continue;
            }
            *is_playing = value;
            return true;
        }
        false
    }

    fn advance_remap(&mut self, remap_local_id: usize) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                mix,
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id || *mix == 0.0 {
                continue;
            }
            return self.child.apply_linear_animation_instance(animation, *mix);
        }
        false
    }

    fn set_is_paused(&mut self, value: bool) -> bool {
        if self.is_paused == value {
            return false;
        }
        self.is_paused = value;
        true
    }

    fn set_speed(&mut self, value: f32) -> bool {
        if self.speed == value {
            return false;
        }
        self.speed = value;
        true
    }

    fn set_quantize(&mut self, value: f32) -> bool {
        if self.quantize == value {
            return false;
        }
        self.quantize = value;
        true
    }
}

impl RuntimeNestedAnimationInstance {
    fn has_ongoing_work(&self, child: &ArtboardInstance) -> bool {
        match self {
            Self::Simple {
                animation,
                is_playing,
                ..
            } => *is_playing && child.linear_animation_instance_keep_going(animation),
            Self::Remap { .. } => false,
            Self::StateMachine(occurrence) => occurrence
                .state_machine()
                .is_some_and(StateMachineInstance::needs_advance),
        }
    }

    fn advance(
        &mut self,
        child: &mut ArtboardInstance,
        elapsed_seconds: f32,
        reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        match self {
            Self::Simple {
                animation,
                is_playing,
                speed,
                mix,
                ..
            } => {
                let mut changed = false;
                if *is_playing {
                    changed |= child
                        .advance_linear_animation_instance(animation, elapsed_seconds * *speed);
                }
                if *mix != 0.0 {
                    changed |= child.apply_linear_animation_instance(animation, *mix);
                }
                changed
            }
            Self::Remap { animation, mix, .. } => {
                if *mix == 0.0 {
                    return false;
                }
                child.apply_linear_animation_instance(animation, *mix)
            }
            Self::StateMachine(occurrence) => {
                occurrence.advance(child, elapsed_seconds, reported_events)
            }
        }
    }
}

fn build_runtime_nested_artboard_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    visiting: &mut BTreeSet<u32>,
    build_context: Option<RuntimeArtboardBuildContext>,
) -> Result<RuntimeNestedArtboards> {
    if artboards.is_empty() {
        return Ok(RuntimeNestedArtboards::default());
    }

    let mut nested_artboards = RuntimeNestedArtboards::default();
    for host in &graph.nested_artboards {
        if !matches!(
            host.type_name,
            "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
        ) {
            continue;
        }

        let Some(host_object) = file.object(host.global_id as usize) else {
            continue;
        };
        let Some(referenced) = file.resolved_artboard_for_referencer_object(host_object) else {
            continue;
        };
        let data_bind_path = file.data_bind_path_for_referencer_object(host_object);
        let data_bind_path_is_relative = data_bind_path
            .as_ref()
            .and_then(|path| path.object)
            .and_then(|path| path.bool_property("isRelative"))
            .unwrap_or(false);
        let data_bind_path_ids = data_bind_path.map(|path| {
            if data_bind_path_is_relative {
                path.path_ids
            } else {
                path.resolved_path_ids
            }
        });
        let Some(child_graph) = artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced.id)
        else {
            continue;
        };
        if visiting.contains(&child_graph.global_id) {
            continue;
        }

        let instance = build_runtime_nested_artboard_instance(
            file,
            graph,
            artboards,
            slots,
            objects,
            host.local_id,
            child_graph,
            visiting,
            build_context.clone(),
            data_bind_path_ids,
            data_bind_path_is_relative,
            host_object.bool_property("isPaused").unwrap_or(false),
            host_object.double_property("speed").unwrap_or(1.0),
            host_object.double_property("quantize").unwrap_or(-1.0),
        )?;
        nested_artboards.insert(host.local_id, instance);
    }

    Ok(nested_artboards)
}

fn build_runtime_nested_artboard_instance(
    file: &RuntimeFile,
    parent_graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    parent_slots: &[InstanceSlot],
    parent_objects: &InstanceObjectArena,
    host_local_id: usize,
    child_graph: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
    build_context: Option<RuntimeArtboardBuildContext>,
    data_bind_path_ids: Option<Vec<u32>>,
    data_bind_path_is_relative: bool,
    is_paused: bool,
    speed: f32,
    quantize: f32,
) -> Result<RuntimeNestedArtboardInstance> {
    let mut child = Box::new(ArtboardInstance::from_graph_inner(
        file,
        child_graph,
        artboards,
        visiting,
        build_context,
        false,
    )?);
    apply_nested_artboard_origin_override(parent_objects, host_local_id, &mut child);
    child.set_frame_origin(false);
    child.added_to_host();
    child.bind_default_view_model_artboard_list_context(file);
    if !child_has_state_machine_data_binds(file, child_graph) {
        child.clear_default_text_property_context();
    }
    let animations =
        runtime_nested_animation_instances(file, parent_graph, host_local_id, &mut child);
    let data_bind_view_model_instance_locals_by_id =
        build_nested_host_view_model_instance_locals(parent_slots, parent_objects, host_local_id);
    let is_stateful = property_key_for_name("NestedArtboard", "isStateful")
        .and_then(|property_key| parent_objects.bool_property(host_local_id, property_key))
        .unwrap_or(false);
    let child_view_model_index = file
        .object(child_graph.global_id as usize)
        .and_then(|artboard| artboard.uint_property("viewModelId"))
        .and_then(|view_model_id| usize::try_from(view_model_id).ok())
        .filter(|&view_model_index| file.view_model(view_model_index).is_some());
    let stateful_view_model_instance_local = is_stateful
        .then_some(child_view_model_index)
        .flatten()
        .and_then(|view_model_index| u32::try_from(view_model_index).ok())
        .and_then(|view_model_id| {
            data_bind_view_model_instance_locals_by_id
                .get(&view_model_id)
                .copied()
        });
    let stateful_view_model_context = if !is_stateful {
        None
    } else if let Some(local_id) = stateful_view_model_instance_local {
        let slot = parent_slots.iter().find(|slot| slot.local_id == local_id);
        slot.and_then(|slot| file.object(slot.source_global_id as usize))
            .and_then(|instance| {
                let view_model_index =
                    usize::try_from(instance.uint_property("viewModelId")?).ok()?;
                RuntimeOwnedViewModelInstance::from_instance_object(
                    file,
                    view_model_index,
                    instance,
                )
            })
    } else {
        child_view_model_index.and_then(|view_model_index| {
            RuntimeOwnedViewModelInstance::from_instance(file, view_model_index, 0)
                .or_else(|| RuntimeOwnedViewModelInstance::new(file, view_model_index))
        })
    };
    let stateful_global_view_model_contexts = data_bind_view_model_instance_locals_by_id
        .iter()
        .filter_map(|(&view_model_id, &local_id)| {
            let view_model_index = usize::try_from(view_model_id).ok()?;
            let view_model = file.view_model(view_model_index)?;
            if view_model.object.uint_property("viewModelType") != Some(2) {
                return None;
            }
            let slot = parent_slots.iter().find(|slot| slot.local_id == local_id)?;
            let instance = file.object(slot.source_global_id as usize)?;
            let context = RuntimeOwnedViewModelInstance::from_instance_object(
                file,
                view_model_index,
                instance,
            )?;
            Some((view_model_index, context))
        })
        .collect();
    let data_bind_source_locals_by_path = build_nested_host_data_bind_source_locals(
        parent_slots,
        parent_objects,
        host_local_id,
        &data_bind_view_model_instance_locals_by_id,
        &child,
    );
    let (data_bind_property_source_locals, data_bind_image_source_locals) =
        build_nested_host_data_bind_source_local_slots(&child, &data_bind_source_locals_by_path);
    Ok(RuntimeNestedArtboardInstance {
        child,
        render_cache_revision: 0,
        render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
        initial_layout_paint_frame: RefCell::new(None),
        layout_data_transferred: false,
        layout_data_transfer_key: None,
        data_bind_path_ids,
        data_bind_path_is_relative,
        stateful_view_model_instance_local,
        stateful_view_model_instance_locals_by_id: data_bind_view_model_instance_locals_by_id,
        stateful_view_model_context,
        stateful_global_view_model_contexts,
        data_bind_property_source_locals,
        data_bind_image_source_locals,
        data_bind_context_source_locals_by_path: data_bind_source_locals_by_path,
        animations,
        is_paused,
        speed,
        quantize,
        cumulated_seconds: 0.0,
    })
}

fn apply_nested_artboard_origin_override(
    parent_objects: &InstanceObjectArena,
    host_local_id: usize,
    child: &mut ArtboardInstance,
) -> bool {
    let Some(host) = parent_objects.component_handle(host_local_id) else {
        return false;
    };
    let origin_local = (0..parent_objects.child_len(host))
        .filter_map(|index| parent_objects.child_at(host, index))
        .filter_map(|child| parent_objects.component(child))
        .find(|component| component.type_name == "NestedArtboardOrigin")
        .map(|component| component.local_id);
    let Some(origin_local) = origin_local else {
        return false;
    };
    let Some(origin_x) = property_key_for_name("NestedArtboardOrigin", "originX")
        .and_then(|key| parent_objects.double_property(origin_local, key))
    else {
        return false;
    };
    let Some(origin_y) = property_key_for_name("NestedArtboardOrigin", "originY")
        .and_then(|key| parent_objects.double_property(origin_local, key))
    else {
        return false;
    };
    let Some(origin_x_key) = property_key_for_name("Artboard", "originX") else {
        return false;
    };
    let Some(origin_y_key) = property_key_for_name("Artboard", "originY") else {
        return false;
    };

    let mut changed = child.set_double_property(0, origin_x_key, origin_x);
    changed |= child.set_double_property(0, origin_y_key, origin_y);
    changed
}

fn child_has_state_machine_data_binds(file: &RuntimeFile, graph: &ArtboardGraph) -> bool {
    crate::properties::artboard_index_for_graph(file, graph).is_some_and(|artboard_index| {
        file.artboard_state_machine_graphs(artboard_index)
            .into_iter()
            .any(|state_machine| !state_machine.data_binds.is_empty())
    })
}

fn runtime_nested_animation_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    host_local_id: usize,
    child: &mut ArtboardInstance,
) -> Vec<RuntimeNestedAnimationInstance> {
    let mut animations = Vec::new();
    for local_object in &graph.local_objects {
        let Some(object) = file.object(local_object.global_id as usize) else {
            continue;
        };
        if object.uint_property("parentId") != Some(host_local_id as u64) {
            continue;
        }

        match object.type_name {
            "NestedSimpleAnimation" => {
                let Some(animation) =
                    nested_simple_animation_instance(local_object.local_id, object, child)
                else {
                    continue;
                };
                animations.push(animation);
            }
            "NestedRemapAnimation" => {
                let Some(animation) =
                    nested_remap_animation_instance(local_object.local_id, object, child)
                else {
                    continue;
                };
                animations.push(animation);
            }
            "NestedStateMachine" => {
                let animation = nested_state_machine_instance(
                    file,
                    graph,
                    local_object.local_id,
                    object,
                    child,
                );
                animations.push(animation);
            }
            _ => {}
        }
    }
    animations
}

fn component_dirt_affects_path_epoch(dirt: ComponentDirt) -> bool {
    // C++ `src/shapes/path.cpp::Path::update` rebuilds raw path geometry for
    // path/nslicer dirt, and only for world-transform dirt when a deformer is
    // present. Plain transform animation is applied at draw time through the
    // shape/world transform and must not churn retained path-command storage.
    !(dirt
        & (ComponentDirt::PATH
            | ComponentDirt::VERTICES
            | ComponentDirt::LAYOUT_STYLE
            | ComponentDirt::N_SLICER))
        .is_empty()
}

fn path_vertex_property_affects_geometry(type_name: Option<&str>, property_key: u16) -> bool {
    let Some(
        type_name @ ("StraightVertex"
        | "CubicMirroredVertex"
        | "CubicAsymmetricVertex"
        | "CubicDetachedVertex"),
    ) = type_name
    else {
        return false;
    };

    let properties: &[&str] = match type_name {
        "StraightVertex" => &["x", "y", "radius"],
        "CubicMirroredVertex" => &["x", "y", "rotation", "distance"],
        "CubicAsymmetricVertex" => &["x", "y", "rotation", "inDistance", "outDistance"],
        "CubicDetachedVertex" => &[
            "x",
            "y",
            "inRotation",
            "inDistance",
            "outRotation",
            "outDistance",
        ],
        _ => unreachable!("path-vertex type was filtered above"),
    };
    properties
        .iter()
        .any(|name| property_key_for_name(type_name, name) == Some(property_key))
}

fn property_affects_effect_path_epoch(type_name: Option<&str>, property_key: u16) -> bool {
    match type_name {
        Some("TrimPath") => ["start", "end", "offset", "modeValue"]
            .iter()
            .any(|name| property_key_for_name("TrimPath", name) == Some(property_key)),
        Some("DashPath") => ["offset", "offsetIsPercentage"]
            .iter()
            .any(|name| property_key_for_name("DashPath", name) == Some(property_key)),
        Some("Dash") => ["length", "lengthIsPercentage"]
            .iter()
            .any(|name| property_key_for_name("Dash", name) == Some(property_key)),
        Some("Feather") => ["spaceValue", "strength", "offsetX", "offsetY", "inner"]
            .iter()
            .any(|name| property_key_for_name("Feather", name) == Some(property_key)),
        _ => false,
    }
}

fn property_may_affect_prepared_frame(type_name: Option<&str>, property_key: u16) -> bool {
    let Some(type_name) = type_name else {
        return true;
    };

    if matches!(
        type_name,
        "NestedNumber"
            | "NestedBool"
            | "NestedTrigger"
            | "NestedInput"
            | "NestedRemapAnimation"
            | "NestedSimpleAnimation"
            | "NestedStateMachine"
            | "StateMachine"
            | "StateMachineLayer"
            | "StateMachineNumber"
            | "StateMachineBool"
            | "StateMachineTrigger"
            | "AnimationState"
            | "AnyState"
            | "EntryState"
            | "ExitState"
            | "StateTransition"
            | "TransitionNumberCondition"
            | "TransitionBoolCondition"
            | "TransitionTriggerCondition"
            | "TransitionValueNumberComparator"
            | "TransitionValueBooleanComparator"
            | "TransitionPropertyArtboardComparator"
            | "TransitionArtboardCondition"
            | "BlendStateDirect"
            | "BlendState1D"
            | "BlendAnimationDirect"
            | "BlendAnimation1D"
            | "BlendStateTransition"
            | "BlendState1DInput"
            | "LinearAnimation"
            | "KeyedObject"
            | "KeyedProperty"
            | "KeyFrameDouble"
            | "KeyFrameColor"
            | "KeyFrameBool"
            | "KeyFrameString"
            | "KeyFrameId"
            | "ListenerTriggerChange"
            | "ListenerAlignTarget"
            | "StateMachineListener"
            | "StateMachineListenerSingle"
            | "FileAssetContents"
            | "FontAsset"
            | "ScriptAsset"
            | "ScriptedDrawable"
            | "ScriptedTransitionCondition"
    ) {
        return false;
    }

    if type_name.starts_with("ViewModel")
        || type_name.starts_with("DataBind")
        || type_name.starts_with("DataConverter")
        || type_name.starts_with("DataEnum")
        || type_name.starts_with("BindableProperty")
        || type_name.starts_with("CustomProperty")
    {
        return false;
    }

    if type_name == "NestedArtboard" {
        return property_key_for_name("NestedArtboard", "artboardId") == Some(property_key);
    }

    // C++ src/shapes/paint/solid_color.cpp updates the retained RenderPaint.
    if type_name == "SolidColor" {
        return solid_color_value_property_key() != Some(property_key);
    }

    true
}

fn nested_simple_animation_instance(
    local_id: usize,
    object: &nuxie_binary::RuntimeObject,
    child: &ArtboardInstance,
) -> Option<RuntimeNestedAnimationInstance> {
    let animation_index = usize::try_from(object.uint_property("animationId")?).ok()?;
    Some(RuntimeNestedAnimationInstance::Simple {
        local_id,
        animation: child.linear_animation_instance(animation_index)?,
        is_playing: object.bool_property("isPlaying").unwrap_or(false),
        speed: object.double_property("speed").unwrap_or(1.0),
        mix: object.double_property("mix").unwrap_or(1.0),
    })
}

fn nested_remap_animation_instance(
    local_id: usize,
    object: &nuxie_binary::RuntimeObject,
    child: &ArtboardInstance,
) -> Option<RuntimeNestedAnimationInstance> {
    let animation_index = usize::try_from(object.uint_property("animationId")?).ok()?;
    let linear_animation = child.linear_animation(animation_index)?;
    let mut animation = child.linear_animation_instance(animation_index)?;
    let time = object.double_property("time").unwrap_or(0.0);
    let seconds =
        linear_animation.global_to_local_seconds(linear_animation.duration_seconds() * time);
    animation.set_time(linear_animation, seconds);
    Some(RuntimeNestedAnimationInstance::Remap {
        local_id,
        animation,
        mix: object.double_property("mix").unwrap_or(1.0),
    })
}

fn nested_state_machine_instance(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
    object: &nuxie_binary::RuntimeObject,
    child: &mut ArtboardInstance,
) -> RuntimeNestedAnimationInstance {
    RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::from_imported(
        file, graph, local_id, object, child,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mat2D;
    use crate::animation::{
        RuntimeKeyFrame, RuntimeKeyFrameCallback, RuntimeKeyedObject, RuntimeKeyedProperty,
        RuntimeKeyedPropertyTarget,
    };
    use crate::components::{
        DataBindHandle, RuntimeComponentCapabilities, SoloMappingWork, TransformRuntimeState,
        reset_solo_mapping_work, solo_mapping_work,
    };
    use crate::data_bind_graph::{
        RuntimeDataBindGraphConverter, runtime_data_bind_graph_reverse_convert_value,
    };
    use crate::properties::property_key_for_name;
    use crate::state_machine::{
        RuntimeBlendState1D, RuntimeBlendState1DSource, RuntimeLayerState,
        RuntimeListenerBoolChange, RuntimeListenerInputTarget, RuntimeListenerNumberChange,
        RuntimeListenerTriggerChange, RuntimeListenerType, RuntimeStateMachineInput,
        RuntimeStateMachineLayer, RuntimeStateMachineListener, ScriptGamepadMappingKind,
        ScriptGamepadSnapshot, ScriptListenerInvocation, StateMachineInputInstance,
    };
    use nuxie_binary::{
        AuthoringProperty, AuthoringRecord, AuthoringValue, BytesValue, FieldValue, RuntimeObject,
        RuntimeProperty, StringValue, read_runtime_file,
    };
    use nuxie_graph::{DependencyNodeKind, GraphFile};
    use nuxie_render_api::RecordingFactory;
    use nuxie_schema::definition_by_name;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    struct UpdateScriptInstance {
        inits: Rc<Cell<usize>>,
        updates: Rc<Cell<usize>>,
    }

    struct AdvanceScriptInstance {
        advances: Rc<Cell<usize>>,
    }

    struct RecordingAdvanceScriptInstance {
        seconds: Rc<RefCell<Vec<f32>>>,
    }

    struct OrderedAdvanceScriptInstance {
        label: u32,
        calls: Rc<RefCell<Vec<u32>>>,
    }

    struct OrderedUpdateScriptInstance {
        label: u32,
        calls: Rc<RefCell<Vec<u32>>>,
        fail_once: Option<Rc<Cell<bool>>>,
    }

    struct AdvanceAndUpdateScriptInstance {
        advances: Rc<Cell<usize>>,
        updates: Rc<Cell<usize>>,
    }

    struct FailOnceAdvanceScriptInstance {
        attempts: Rc<RefCell<Vec<f32>>>,
        should_fail: Rc<Cell<bool>>,
    }

    impl ScriptInstance for AdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].as_number().map(|value| value as f32), Some(0.1));
            let count = self.advances.get() + 1;
            self.advances.set(count);
            Ok(ScriptValue::Bool(count != 2))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for RecordingAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            let seconds = args
                .first()
                .and_then(ScriptValue::as_number)
                .map(|value| value as f32)
                .expect("advance receives seconds");
            self.seconds.borrow_mut().push(seconds);
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for OrderedAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            self.calls.borrow_mut().push(self.label);
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for OrderedUpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Update)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Update);
            self.calls.borrow_mut().push(self.label);
            if self
                .fail_once
                .as_ref()
                .is_some_and(|fail_once| fail_once.replace(false))
            {
                return Err(ScriptError::new("fail once"));
            }
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for AdvanceAndUpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(
                method,
                ScriptMethod::Advance | ScriptMethod::Update
            ))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Advance => {
                    self.advances.set(self.advances.get() + 1);
                    Ok(ScriptValue::Bool(true))
                }
                ScriptMethod::Update => {
                    self.updates.set(self.updates.get() + 1);
                    Ok(ScriptValue::Nil)
                }
                _ => unreachable!("only declared script methods are called"),
            }
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for FailOnceAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            let seconds = args
                .first()
                .and_then(ScriptValue::as_number)
                .map(|value| value as f32)
                .expect("advance receives seconds");
            self.attempts.borrow_mut().push(seconds);
            if self.should_fail.replace(false) {
                return Err(ScriptError::new("fail once"));
            }
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for UpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(method, ScriptMethod::Init | ScriptMethod::Update))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Init => self.inits.set(self.inits.get() + 1),
                ScriptMethod::Update => self.updates.set(self.updates.get() + 1),
                _ => {}
            }
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn synthetic_instance(
        components: Vec<RuntimeComponent>,
        update_order: Vec<usize>,
    ) -> ArtboardInstance {
        let slots = components
            .iter()
            .map(|component| InstanceSlot {
                local_id: component.local_id,
                source_global_id: component.global_id,
                type_name: Some(component.type_name),
                name: None,
            })
            .collect::<Vec<_>>();
        let mut runtime_objects = vec![None; slots.len()];
        for component in &components {
            if component.local_id >= runtime_objects.len() {
                runtime_objects.resize(component.local_id + 1, None);
            }
            runtime_objects[component.local_id] = Some(synthetic_runtime_object(
                component.global_id,
                component.type_name,
                Vec::new(),
            ));
        }
        let mut objects = InstanceObjectArena::from_runtime_objects(runtime_objects);
        for component in components {
            let local_id = component.local_id;
            objects
                .attach_component(local_id, component)
                .expect("synthetic component occurrence must exist once");
        }
        let dependency_handles = update_order
            .iter()
            .filter_map(|local_id| objects.component_handle(*local_id))
            .collect();
        objects.set_dependency_order(dependency_handles);
        if let Some(root) = objects.root()
            && let Some(component) = objects.component_mut(root)
        {
            // Synthetic Artboards model the inherited C++ root Component:
            // Components is loop-control dirt on that same owner, alongside
            // whatever concrete dirt the test supplied.
            component.dirt |= ComponentDirt::COMPONENTS;
        }
        let advancing_components = slots
            .iter()
            .filter_map(|slot| {
                let kind = match slot.type_name? {
                    "Artboard" => AdvancingComponentKind::Artboard,
                    "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
                        AdvancingComponentKind::NestedArtboard
                    }
                    "LayoutComponent" => AdvancingComponentKind::LayoutComponent,
                    "ArtboardComponentList" => AdvancingComponentKind::ArtboardComponentList,
                    "ScrollConstraint" => AdvancingComponentKind::ScrollConstraint,
                    "TextInput" => AdvancingComponentKind::TextInput,
                    "ScriptedDataConverter" => AdvancingComponentKind::ScriptedDataConverter,
                    "ScriptedDrawable" => AdvancingComponentKind::ScriptedDrawable,
                    "ScriptedLayout" => AdvancingComponentKind::ScriptedLayout,
                    "ScriptedPathEffect" => AdvancingComponentKind::ScriptedPathEffect,
                    _ => return None,
                };
                Some(RuntimeAdvancingComponent {
                    local_id: slot.local_id,
                    object: objects.object_handle(slot.local_id)?,
                    component: objects.component_handle(slot.local_id),
                    kind,
                })
            })
            .collect();
        let resetting_components = slots
            .iter()
            .filter_map(|slot| {
                let kind = match slot.type_name? {
                    "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
                        ResettingComponentKind::NestedArtboard
                    }
                    "ArtboardComponentList" => ResettingComponentKind::ArtboardComponentList,
                    "CustomPropertyTrigger" => ResettingComponentKind::CustomPropertyTrigger,
                    _ => return None,
                };
                Some(RuntimeResettingComponent {
                    local_id: slot.local_id,
                    component: objects.component_handle(slot.local_id)?,
                    kind,
                })
            })
            .collect();
        let component_lists = objects
            .component_handles()
            .iter()
            .copied()
            .filter(|handle| {
                objects
                    .component(*handle)
                    .is_some_and(|component| component.concrete.constrainable_list.is_some())
            })
            .collect();

        let text_affecting_locals = build_text_affecting_locals(&slots, &objects);
        let solid_color_paint_revisions = vec![
            1;
            slots
                .iter()
                .map(|slot| slot.local_id)
                .max()
                .map_or(0, |local_id| local_id.saturating_add(1))
        ];
        ArtboardInstance {
            instance_identity: RuntimeArtboardInstanceIdentity::next(),
            width: 0.0,
            height: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            clip: true,
            frame_origin: Cell::new(true),
            frame_id: Cell::new(0),
            slots,
            objects,
            joysticks: Vec::new(),
            advancing_components,
            resetting_components,
            component_lists,
            joysticks_apply_before_update: true,
            linear_animations: Arc::new(Vec::new()),
            empty_linear_animation: Arc::new(RuntimeLinearAnimation::empty()),
            state_machines: Arc::new(Vec::new()),
            script_instances_by_global: RuntimeScriptState::default(),
            script_attachment_generation: 0,
            scripted_data_converter_instances_by_global: RuntimeScriptState::default(),
            has_scripted_drawables: false,
            nested_script_owned_contexts: BTreeMap::new(),
            script_update_error: None,
            external_focus_domain: None,
            nested_artboards: RuntimeNestedArtboards::default(),
            nested_artboard_locals: Vec::new(),
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: 0,
            build_context: None,
            nested_context_source_tree_cache: Cell::new(None),
            nested_layout_bounds: None,
            artboard_data_bind_values: BTreeMap::new(),
            artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            artboard_owned_view_model_context: None,
            artboard_owned_data_context: None,
            artboard_owned_view_model_handle: None,
            artboard_authored_data_bind_states: RuntimeArtboardAuthoredDataBindStates::default(),
            artboard_owned_view_model_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink::new(
            ),
            artboard_property_bindings: Vec::new(),
            artboard_image_asset_bindings: Vec::new(),
            artboard_data_bind_target_queues: RuntimeArtboardDataBindTargetQueues::default(),
            artboard_data_bind_source_queues: RuntimeArtboardDataBindSourceQueues::default(),
            artboard_retained_subordinate_converter_operands: Vec::new(),
            artboard_custom_property_bindings: Vec::new(),
            artboard_layout_computed_bindings: Vec::new(),
            artboard_numeric_source_bindings: Vec::new(),
            artboard_formula_token_bindings: RuntimeArtboardFormulaTokenBindingStates::default(),
            artboard_converter_property_bindings: Vec::new(),
            artboard_solo_bindings: Vec::new(),
            artboard_solo_source_bindings: Vec::new(),
            artboard_nested_host_bindings: Vec::new(),
            artboard_list_bindings: Vec::new(),
            artboard_text_list_bindings: Vec::new(),
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            stateful_nested_view_model_contexts_dirty: true,
            artboard_data_bind_dirty_epoch: 1,
            artboard_data_bind_processed_epoch: 0,
            image_asset_overrides: BTreeMap::new(),
            text_style_font_overrides: BTreeMap::new(),
            has_legacy_image_layout_scales: Cell::new(false),
            legacy_image_layout_scales: RefCell::new(BTreeMap::new()),
            external_font_assets: Arc::new(BTreeMap::new()),
            runtime_image_assets: RefCell::new(None),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            geometry_state: RefCell::new(crate::draw::RuntimeGeometryState::default()),
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_epoch: 1,
            text_affecting_locals,
            solid_color_paint_revisions,
            runtime_drawables: RuntimeDrawableList::default(),
            runtime_shapes: RuntimeShapeList::default(),
            runtime_clipping_shapes: RuntimeClippingShapeList::default(),
            runtime_meshes: crate::draw::RuntimeMeshList::default(),
            did_change: Cell::new(true),
            layout_constraint_bounds_enabled: false,
            layout_constraint_bounds: None,
        }
    }

    fn synthetic_nested_artboard_instance(graph_global_id: u32) -> RuntimeNestedArtboardInstance {
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        child.graph_global_id = graph_global_id;
        RuntimeNestedArtboardInstance {
            child: Box::new(child),
            render_cache_revision: 0,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: None,
            data_bind_path_is_relative: false,
            stateful_view_model_instance_local: None,
            stateful_view_model_instance_locals_by_id: BTreeMap::new(),
            stateful_view_model_context: None,
            stateful_global_view_model_contexts: BTreeMap::new(),
            data_bind_property_source_locals: Vec::new(),
            data_bind_image_source_locals: Vec::new(),
            data_bind_context_source_locals_by_path: BTreeMap::new(),
            animations: Vec::new(),
            is_paused: false,
            speed: 1.0,
            quantize: -1.0,
            cumulated_seconds: 0.0,
        }
    }

    #[test]
    fn nested_artboards_preserve_sorted_iteration_and_sparse_lookup_after_edits() {
        let mut nested_artboards = RuntimeNestedArtboards::default();
        nested_artboards.insert(9, synthetic_nested_artboard_instance(90));
        nested_artboards.insert(2, synthetic_nested_artboard_instance(20));
        nested_artboards.insert(5, synthetic_nested_artboard_instance(50));

        assert_eq!(
            nested_artboards.keys().copied().collect::<Vec<_>>(),
            [2, 5, 9]
        );
        assert_eq!(nested_artboards.get(&5).unwrap().child.graph_global_id, 50);

        let replaced = nested_artboards
            .insert(5, synthetic_nested_artboard_instance(51))
            .expect("existing local is replaced");
        assert_eq!(replaced.child.graph_global_id, 50);
        assert_eq!(nested_artboards.get(&5).unwrap().child.graph_global_id, 51);

        let removed = nested_artboards.remove(&2).expect("local is removed");
        assert_eq!(removed.child.graph_global_id, 20);
        assert!(nested_artboards.get(&2).is_none());
        assert_eq!(nested_artboards.keys().copied().collect::<Vec<_>>(), [5, 9]);
        assert_eq!(nested_artboards.get(&9).unwrap().child.graph_global_id, 90);
    }

    fn authoring_record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn authoring_property(
        type_name: &str,
        property_name: &str,
        value: AuthoringValue,
    ) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}")),
            value,
        }
    }

    #[test]
    fn component_list_context_match_requires_the_same_shared_graph() {
        let bytes = synthetic_riv(9621, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic view model should import");
        let instance = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("synthetic view model should instantiate");
        let retained = RuntimeOwnedViewModelHandle::new(instance);
        let same_graph = retained.clone();
        let forked_graph = RuntimeOwnedViewModelHandle::new(retained.borrow().clone());
        assert_eq!(
            retained.borrow().instance_identity(),
            forked_graph.borrow().instance_identity(),
            "the payload clone deliberately preserves logical instance identity"
        );

        let row = RuntimeComponentListItemInstance {
            child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines: Vec::new(),
            context_rebind_sink: {
                let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                retained.add_rebind_dependent(&sink);
                sink
            },
            draw_index_sink: None,
            context: retained,
            occurrence_identity: 1,
            logical_index: 0,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            render_cache_revision: 1,
        };

        assert!(component_list_contexts_retain_same_handles(
            std::slice::from_ref(&row),
            std::slice::from_ref(&same_graph),
        ));
        assert!(!component_list_contexts_retain_same_handles(
            std::slice::from_ref(&row),
            std::slice::from_ref(&forked_graph),
        ));
    }

    fn empty_state_machine(global_id: u32) -> RuntimeStateMachine {
        RuntimeStateMachine {
            global_id,
            name: None,
            default_view_model_index: None,
            inputs: Arc::new(Vec::new()),
            listeners: Arc::new(Vec::new()),
            layers: Arc::new(Vec::new()),
            bindable_numbers: Arc::new(Vec::new()),
            bindable_integers: Arc::new(Vec::new()),
            bindable_colors: Arc::new(Vec::new()),
            bindable_strings: Arc::new(Vec::new()),
            bindable_enums: Arc::new(Vec::new()),
            bindable_assets: Arc::new(Vec::new()),
            bindable_artboards: Arc::new(Vec::new()),
            bindable_lists: Arc::new(Vec::new()),
            bindable_triggers: Arc::new(Vec::new()),
            bindable_view_models: Arc::new(Vec::new()),
            bindable_booleans: Arc::new(Vec::new()),
            view_model_triggers: Arc::new(Vec::new()),
            transition_duration_bindings: Arc::new(Vec::new()),
            data_bind_templates: Arc::new(Vec::new()),
            scripted_objects: Vec::new(),
            scripted_listener_actions: Vec::new(),
            scripted_object_bindings: Vec::new(),
            action_owners: crate::state_machine::RuntimeActionCoreArena::empty(),
        }
    }

    #[test]
    fn state_machine_instance_retains_the_cpp_authored_definition_owner() {
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        artboard.state_machines = Arc::new(vec![empty_state_machine(11)]);

        let instance = artboard.state_machine_instance(0).expect("state machine");
        let retained_owner = instance
            .retained_state_machine_definitions()
            .expect("artboard-created instances retain the definition");
        let retained = retained_owner.get(0).expect("retained definition");
        let authored = artboard.state_machine(0).expect("authored definition");

        // C++ stores this exact authored `StateMachine*` on the instance
        // (`state_machine_instance.hpp:123,386`;
        // `state_machine_instance.cpp:1707-1711`).
        assert!(std::ptr::eq(retained, authored));
    }

    #[test]
    fn each_state_machine_instance_dirties_each_hit_shape_once() {
        let shape = synthetic_component_for_type(0, "Shape");
        let mut artboard = synthetic_instance(vec![shape], vec![0]);
        let listener = RuntimeStateMachineListener {
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Down],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        };
        let mut definition = empty_state_machine(12);
        definition.listeners = Arc::new(vec![listener.clone(), listener]);
        artboard.state_machines = Arc::new(vec![definition]);
        assert!(artboard.update_components().did_update);

        let first_cache_epoch = artboard.cache_epoch();
        artboard.state_machine_instance(0).expect("first instance");
        assert!(
            artboard
                .component(0)
                .expect("shape")
                .dirt
                .contains(ComponentDirt::PATH),
            "HitExpandable construction immediately executes Shape::addDirt(Path,true), while its instance-local hitLookup deduplicates duplicate Shape listeners (`state_machine_instance.cpp:1651-1666`)"
        );
        assert!(
            artboard
                .component(0)
                .and_then(|component| component.concrete.shape.as_ref())
                .expect("shape state")
                .is_flagged(crate::components::RuntimeShapeState::NEVER_DEFER_UPDATE),
            "listener initialization sets neverDeferUpdate with the same HitExpandable owner before returning the instance (`state_machine_instance.cpp:1651-1661,1827-1831`)"
        );
        assert!(artboard.update_components().did_update);
        assert!(artboard.cache_epoch() > first_cache_epoch);

        let second_cache_epoch = artboard.cache_epoch();
        artboard.state_machine_instance(0).expect("second instance");
        assert!(
            artboard
                .component(0)
                .expect("shape")
                .dirt
                .contains(ComponentDirt::PATH),
            "a fresh C++ StateMachineInstance owns a fresh hitLookup and dirties the already-flagged Shape again"
        );
        assert!(artboard.update_components().did_update);
        assert!(
            artboard.cache_epoch() > second_cache_epoch,
            "every StateMachineInstance creation executes Shape::addDirt(Path,true) (`state_machine_instance.cpp:1651-1666,1827-1831`)"
        );
    }

    fn direct_input_blend_state_machine(global_id: u32) -> RuntimeStateMachine {
        let mut state_machine = empty_state_machine(global_id);
        state_machine.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("blend".to_owned()),
            0.0,
        ))]);
        state_machine.layers = Arc::new(vec![RuntimeStateMachineLayer {
            global_id: 2,
            name: None,
            states: vec![RuntimeLayerState {
                global_id: Some(3),
                type_name: Some("BlendState1DInput"),
                animation: None,
                blend_state_1d: Some(RuntimeBlendState1D {
                    source: RuntimeBlendState1DSource::Input {
                        input_index: Some(0),
                    },
                    animations: Vec::new(),
                }),
                blend_state_direct: None,
                speed: 1.0,
                flags: 0,
                fire_actions: Vec::new(),
                listener_actions: Vec::new(),
                transitions: Vec::new(),
            }],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        }]);
        state_machine
    }

    #[test]
    fn ordinary_direct_input_blend_does_not_require_outer_state_probe() {
        let definition = direct_input_blend_state_machine(11);
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        let mut state_machine = StateMachineInstance::new(0, &definition, &mut artboard);
        artboard.state_machines = Arc::new(vec![definition]);

        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert!(state_machine.needs_advance());
        assert!(!state_machine.requires_post_update_state_probe());
        assert!(!state_machine.post_update_probe_pending());
        assert!(!state_machine_requires_outer_update_probe(&state_machine));

        state_machine.schedule_post_update_probe();
        assert!(state_machine_requires_outer_update_probe(&state_machine));
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
        assert!(!state_machine.post_update_probe_pending());
    }

    #[test]
    fn nested_host_input_write_schedules_outer_state_probe() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                1,
                Some("enabled".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                2,
                Some("amount".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                3,
                Some("fire".to_owned()),
            )),
        ]);
        let mut nested = synthetic_nested_artboard_instance(22);
        let bool_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let number_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let trigger_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        nested.child.state_machines = Arc::new(vec![definition]);
        nested.animations.extend([
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                7,
                bool_state_machine,
                Vec::new(),
            )),
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                8,
                number_state_machine,
                Vec::new(),
            )),
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                9,
                trigger_state_machine,
                Vec::new(),
            )),
        ]);
        let nested_bool = synthetic_component_for_type(0, "NestedBool");
        let nested_number = synthetic_component_for_type(1, "NestedNumber");
        let nested_trigger = synthetic_component_for_type(2, "NestedTrigger");
        let mut parent =
            synthetic_instance(vec![nested_bool, nested_number, nested_trigger], Vec::new());
        let parent_id_key =
            property_key_for_name("Component", "parentId").expect("Component.parentId");
        let input_id_key =
            property_key_for_name("NestedInput", "inputId").expect("NestedInput.inputId");
        for (local_id, state_machine_local_id, input_id) in [(0, 7, 0), (1, 8, 1), (2, 9, 2)] {
            assert!(parent.objects.set_uint_property(
                local_id,
                parent_id_key,
                state_machine_local_id
            ));
            assert!(
                parent
                    .objects
                    .set_uint_property(local_id, input_id_key, input_id)
            );
        }
        parent.nested_artboards.insert(3, nested);

        assert!(parent.set_nested_state_machine_bool(7, 0, true));
        assert!(
            parent
                .nested_state_machine_mut(7)
                .expect("mounted nested state machine")
                .post_update_probe_pending()
        );
        assert!(parent.set_nested_state_machine_number(8, 1, 1.0));
        assert!(
            parent
                .nested_state_machine_mut(8)
                .expect("mounted nested state machine")
                .post_update_probe_pending()
        );
        assert!(
            parent.fire_nested_trigger_input(2),
            "NestedTrigger::fire is a callback and must reach the nested occurrence without a stored-property write"
        );
        assert!(
            parent
                .nested_state_machine_mut(9)
                .expect("mounted nested state machine")
                .post_update_probe_pending()
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(9)
                .expect("mounted nested state machine")
                .input(2)
                .and_then(|input| input.trigger_fired()),
            Some(true)
        );

        let direct_definitions = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                101,
                Some("direct bool".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                102,
                Some("direct number".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                103,
                Some("direct trigger".to_owned()),
            )),
        ]);
        let mut direct_inputs = (0..direct_definitions.len())
            .map(|index| StateMachineInputInstance::new(index, Arc::clone(&direct_definitions)))
            .collect::<Vec<_>>();

        let nested_bool_action = RuntimeListenerBoolChange::for_test(
            0,
            RuntimeListenerInputTarget {
                direct_input_index: Some(0),
                nested_input_local_id: Some(0),
            },
            1,
        );
        let nested_bool_value_key =
            property_key_for_name("NestedBool", "nestedValue").expect("NestedBool.nestedValue");
        let parent_cache_epoch = parent.cache_epoch();
        assert!(
            !nested_bool_action.perform(&mut parent, &mut direct_inputs),
            "the child was independently set true above, so setting true again is an exact no-op"
        );
        assert_eq!(direct_inputs[0].bool_value(), Some(false));
        assert_eq!(
            parent.objects.bool_property(0, nested_bool_value_key),
            Some(false),
            "NestedBoolBase storage is construction-only and is not rewritten by the live setter"
        );
        assert_eq!(
            parent.bool_property(0, nested_bool_value_key),
            Some(true),
            "the virtual getter reads the child SMIBool rather than stale authored storage"
        );
        assert_eq!(
            parent.cache_epoch(),
            parent_cache_epoch,
            "a nested-input listener action dirties only the child state-machine occurrence"
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(7)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::bool_value),
            Some(true)
        );
        assert!(
            RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(0),
                },
                2,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert_eq!(
            parent.objects.bool_property(0, nested_bool_value_key),
            Some(false),
            "toggling the live child must not mutate parent property storage"
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(7)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::bool_value),
            Some(false)
        );
        assert!(
            !RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(99),
                },
                1,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert!(
            !RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(1),
                },
                1,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert_eq!(direct_inputs[0].bool_value(), Some(false));

        let nested_number_value_key =
            property_key_for_name("NestedNumber", "nestedValue").expect("NestedNumber.nestedValue");
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.0, -0.0] {
            assert!(
                RuntimeListenerNumberChange::for_test(
                    0,
                    RuntimeListenerInputTarget {
                        direct_input_index: Some(1),
                        nested_input_local_id: Some(1),
                    },
                    value,
                )
                .perform(&mut parent, &mut direct_inputs)
            );
            let actual = parent
                .nested_state_machine_mut(8)
                .and_then(|machine| machine.input(1))
                .and_then(StateMachineInputInstance::number_value)
                .expect("nested number value");
            assert_eq!(actual.to_bits(), value.to_bits());
            assert_eq!(
                parent
                    .objects
                    .double_property(1, nested_number_value_key)
                    .expect("authored NestedNumber storage")
                    .to_bits(),
                0.0f32.to_bits(),
                "the live NestedNumber setter must not rewrite parent storage"
            );
            assert_eq!(
                parent
                    .double_property(1, nested_number_value_key)
                    .expect("virtual NestedNumber getter")
                    .to_bits(),
                value.to_bits()
            );
            assert_eq!(direct_inputs[1].number_value(), Some(0.0));
        }

        let nested_trigger_action = RuntimeListenerTriggerChange::for_test(
            0,
            RuntimeListenerInputTarget {
                direct_input_index: Some(2),
                nested_input_local_id: Some(2),
            },
        );
        assert!(
            !nested_trigger_action.perform(&mut parent, &mut direct_inputs),
            "the earlier direct fire remains latched until the child advances"
        );
        {
            let nested = parent
                .nested_artboards
                .get_mut(&3)
                .expect("mounted nested artboard");
            let (animations, child) = (&mut nested.animations, &mut nested.child);
            let occurrence = animations
                .iter_mut()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence)
                        if occurrence.local_id() == 9 =>
                    {
                        Some(occurrence)
                    }
                    _ => None,
                })
                .expect("nested trigger state machine");
            let _ = occurrence.advance(child, 0.0, None);
        }
        assert!(nested_trigger_action.perform(&mut parent, &mut direct_inputs));
        assert_eq!(direct_inputs[2].trigger_fired(), Some(false));
        {
            let nested = parent
                .nested_artboards
                .get_mut(&3)
                .expect("mounted nested artboard");
            let (animations, child) = (&mut nested.animations, &mut nested.child);
            let occurrence = animations
                .iter_mut()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence)
                        if occurrence.local_id() == 9 =>
                    {
                        Some(occurrence)
                    }
                    _ => None,
                })
                .expect("nested trigger state machine");
            let _ = occurrence.advance(child, 0.0, None);
        }
        assert!(nested_trigger_action.perform(&mut parent, &mut direct_inputs));
        assert_eq!(direct_inputs[2].trigger_fired(), Some(false));
    }

    #[test]
    fn nested_state_machine_retains_authored_input_slots_and_skips_initial_trigger() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                1,
                Some("duplicate".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                2,
                Some("duplicate".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                3,
                Some("fire".to_owned()),
            )),
        ]);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        let occurrence = RuntimeNestedStateMachineInstance::new(
            7,
            state_machine,
            vec![
                (0, Some("duplicate".to_owned()), Some(true), None),
                (1, Some("duplicate".to_owned()), None, Some(7.5)),
                (2, Some("fire".to_owned()), None, None),
                (99, None, None, None),
            ],
        );

        assert_eq!(occurrence.input_count(), 4);
        assert_eq!(occurrence.input_id_at(0), Some(0));
        assert_eq!(occurrence.input_id_at(3), Some(99));
        assert_eq!(occurrence.input_id_at(4), None);
        assert_eq!(occurrence.input_id_named("duplicate"), Some(0));
        assert_eq!(
            occurrence.input_id_named(""),
            Some(99),
            "an authored nested input with an absent child input has C++'s empty name"
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(0)
                .and_then(|input| input.bool_value()),
            Some(true)
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(1)
                .and_then(|input| input.number_value()),
            Some(7.5)
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(2)
                .and_then(|input| input.trigger_fired()),
            Some(false)
        );
    }

    #[test]
    fn nested_state_machine_forwards_empty_child_results_and_context_lifecycle() {
        let definition = empty_state_machine(11);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        child.state_machines = Arc::new(vec![definition]);
        let mut occurrence = RuntimeNestedStateMachineInstance::new(7, state_machine, Vec::new());

        assert!(!occurrence.hit_test(&child, 0.0, 0.0));
        assert!(!occurrence.pointer_down(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_move(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.pointer_up(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_exit(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.drag_start(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.drag_end(&mut child, 0.0, 0.0, 0.5, 1));
        assert!(!occurrence.try_change_state(&mut child));

        let data_context = RuntimeOwnedDataContext::default();
        assert!(occurrence.bind_owned_data_context(&data_context));
        assert!(occurrence.clear_data_context());
    }

    #[test]
    fn layer_initialization_loop_is_serial_in_authored_order() {
        const STATE_AT_START: u64 = 2 << 1;
        let bytes = synthetic_riv(9597, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineNumber", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                push_synthetic_uint_property(
                    bytes,
                    "ListenerNumberChange",
                    "flags",
                    STATE_AT_START,
                );
                push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 7.0);
            });
            push_synthetic_object(bytes, "ExitState", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "ExitState", &[]);
        });
        let file = read_runtime_file(&bytes).expect("serial-layer fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("serial-layer fixture graphs");
        let mut artboard =
            ArtboardInstance::from_graph(&file, graph.artboards.first().expect("artboard"))
                .expect("serial-layer fixture instances");

        StateMachineInstance::reset_layer_construction_number_snapshots();
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("imported state-machine occurrence");

        assert_eq!(
            StateMachineInstance::layer_construction_number_snapshots(),
            vec![vec![Some(0.0)], vec![Some(7.0)]],
            "the layer-2 initialization observer must run after layer 1's entry action"
        );
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0)
        );
    }

    #[test]
    fn imported_nested_state_machine_retains_missing_child_owner_inputs_and_empty_forwarding() {
        let bytes = synthetic_riv(9598, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "NestedStateMachine", &[("parentId", 0)]);
            push_synthetic_object(bytes, "NestedBool", &[("parentId", 1), ("inputId", 9)]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard_graph = graph.artboards.first().expect("synthetic riv has artboard");
        let nested_object = artboard_graph
            .local_objects
            .iter()
            .find(|local| {
                file.object(local.global_id as usize)
                    .is_some_and(|object| object.type_name == "NestedStateMachine")
            })
            .expect("nested state-machine object");
        let object = file
            .object(nested_object.global_id as usize)
            .expect("nested state-machine runtime object");
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let mut occurrence = RuntimeNestedStateMachineInstance::from_imported(
            &file,
            artboard_graph,
            nested_object.local_id,
            object,
            &mut child,
        );

        assert_eq!(occurrence.animation_id(), u32::MAX as usize);
        assert!(!occurrence.has_state_machine());
        assert_eq!(occurrence.input_count(), 1);
        assert_eq!(occurrence.input_id_at(0), Some(9));
        assert_eq!(occurrence.input_id_at(1), None);
        assert_eq!(occurrence.input_id_named(""), Some(9));
        assert_eq!(occurrence.input_id_named("missing"), None);
        assert!(!occurrence.advance(&mut child, 0.0, None));
        assert!(!occurrence.hit_test(&child, 0.0, 0.0));
        assert!(!occurrence.pointer_down(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_move(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.pointer_up(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_exit(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.drag_start(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.drag_end(&mut child, 0.0, 0.0, 0.5, 1));
        assert!(!occurrence.try_change_state(&mut child));
        assert!(!occurrence.bind_owned_data_context(&RuntimeOwnedDataContext::default()));
        assert!(!occurrence.clear_data_context());

        let clone = occurrence.cold_clone(&mut child);
        assert_eq!(clone.animation_id(), u32::MAX as usize);
        assert!(!clone.has_state_machine());
        assert_eq!(clone.input_count(), 1);
        assert_eq!(clone.input_id_at(0), Some(9));
        assert_eq!(clone.input_id_named(""), Some(9));
    }

    #[test]
    fn public_clone_rebuilds_nested_state_machine_cold_but_transient_clone_keeps_live_value() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_bool(
            1,
            Some("enabled".to_owned()),
            false,
        ))]);
        let mut nested = synthetic_nested_artboard_instance(22);
        nested.child.state_machines = Arc::new(vec![definition.clone()]);
        let state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let mut occurrence = RuntimeNestedStateMachineInstance::new(
            7,
            state_machine,
            vec![(0, Some("enabled".to_owned()), Some(true), None)],
        );
        assert!(
            occurrence
                .state_machine_mut()
                .expect("valid nested state machine")
                .set_bool(0, false)
        );
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(occurrence));

        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent.nested_artboards.insert(3, nested);
        let cloned = parent.clone();
        let transient = parent.clone_for_transient_layout();

        let bool_value = |artboard: &ArtboardInstance| {
            let nested = artboard
                .nested_artboards
                .get(&3)
                .expect("nested occurrence");
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = &nested.animations[0]
            else {
                panic!("nested animation remains a state machine");
            };
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(0)
                .and_then(|input| input.bool_value())
        };
        assert_eq!(
            bool_value(&cloned),
            Some(true),
            "public generated clone reapplies the authored nested input"
        );
        assert_eq!(
            bool_value(&transient),
            Some(false),
            "transient layout clone views the live occurrence snapshot"
        );
    }

    #[test]
    fn quantized_nested_skip_schedules_outer_state_probe() {
        let definition = empty_state_machine(11);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        child.state_machines = Arc::new(vec![definition]);
        let mut nested = RuntimeNestedArtboardInstance {
            child: Box::new(child),
            render_cache_revision: 0,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: None,
            data_bind_path_is_relative: false,
            stateful_view_model_instance_local: None,
            stateful_view_model_instance_locals_by_id: BTreeMap::new(),
            stateful_view_model_context: None,
            stateful_global_view_model_contexts: BTreeMap::new(),
            data_bind_property_source_locals: Vec::new(),
            data_bind_image_source_locals: Vec::new(),
            data_bind_context_source_locals_by_path: BTreeMap::new(),
            animations: vec![RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(1, state_machine, Vec::new()),
            )],
            is_paused: false,
            speed: 1.0,
            quantize: 1.0,
            cumulated_seconds: 0.0,
        };

        let mut script_mode = RuntimeScriptAdvanceMode::Disabled;
        assert!(nested.advance(0.25, &mut script_mode, None).unwrap());
        let RuntimeNestedAnimationInstance::StateMachine(occurrence) = &nested.animations[0] else {
            panic!("nested animation remains a state machine");
        };
        assert!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .post_update_probe_pending()
        );
    }

    #[test]
    fn mounted_nested_state_probe_is_consumed_once() {
        let definition = empty_state_machine(11);
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        let mut state_machine = StateMachineInstance::new(0, &definition, &mut artboard);
        artboard.state_machines = Arc::new(vec![definition]);

        assert!(!state_machine.post_update_probe_pending());
        state_machine.schedule_post_update_probe();
        assert!(state_machine.post_update_probe_pending());
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
        assert!(!state_machine.post_update_probe_pending());
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
    }

    #[test]
    fn artboard_clone_shares_the_file_owned_external_font_snapshot() {
        let mut original = synthetic_instance(Vec::new(), Vec::new());
        let bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        original.external_font_assets = Arc::new(BTreeMap::from([(7, Arc::clone(&bytes))]));

        let cloned = original.clone();
        let cloned_bytes = cloned
            .external_font_assets
            .get(&7)
            .expect("cloned artboard retains external font asset");

        assert!(Arc::ptr_eq(
            &original.external_font_assets,
            &cloned.external_font_assets
        ));
        assert!(Arc::ptr_eq(&bytes, cloned_bytes));
    }

    #[test]
    fn unresolved_nested_artboard_binding_preserves_the_mounted_child() {
        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent
            .nested_artboards
            .insert(3, synthetic_nested_artboard_instance(77));
        parent.nested_artboard_locals.push(3);

        // A synthetic instance has no build context, so every non-null id is
        // unresolvable. C++ keeps the outgoing mounted child in this case.
        assert!(!parent.set_nested_artboard_artboard_id(3, 12));
        assert_eq!(
            parent
                .nested_artboards
                .get(&3)
                .map(|nested| nested.child.graph_global_id),
            Some(77)
        );
        assert_eq!(parent.nested_artboard_locals, [3]);
    }

    #[test]
    fn null_then_unresolved_nested_artboard_binding_stays_absent() {
        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent
            .nested_artboards
            .insert(3, synthetic_nested_artboard_instance(77));
        parent.nested_artboard_locals.push(3);

        assert!(parent.set_nested_artboard_artboard_id(3, u64::from(u32::MAX)));
        assert!(!parent.nested_artboards.contains_key(&3));
        assert!(parent.nested_artboard_locals.is_empty());

        // A later invalid/self target is not an explicit null and therefore
        // cannot resurrect the authored fallback or a new child.
        assert!(!parent.set_nested_artboard_artboard_id(3, 0));
        assert!(!parent.nested_artboards.contains_key(&3));
        assert!(parent.nested_artboard_locals.is_empty());
    }

    #[test]
    fn nested_artboard_swap_immediately_inherits_the_active_parent_context() {
        let number_key = property_key_for_name("Rectangle", "width").expect("rectangle width");
        let artboard = || {
            authoring_record(
                "Artboard",
                vec![authoring_property(
                    "Artboard",
                    "viewModelId",
                    AuthoringValue::Uint(0),
                )],
            )
        };
        let bound_rectangle = |width| {
            vec![
                authoring_record(
                    "Shape",
                    vec![authoring_property(
                        "Shape",
                        "parentId",
                        AuthoringValue::Uint(0),
                    )],
                ),
                authoring_record(
                    "Rectangle",
                    vec![
                        authoring_property("Rectangle", "parentId", AuthoringValue::Uint(1)),
                        authoring_property("Rectangle", "width", AuthoringValue::Double(width)),
                    ],
                ),
                authoring_record(
                    "DataBindContext",
                    vec![
                        authoring_property(
                            "DataBindContext",
                            "propertyKey",
                            AuthoringValue::Uint(u64::from(number_key)),
                        ),
                        authoring_property(
                            "DataBindContext",
                            "sourcePathIds",
                            AuthoringValue::Bytes(vec![0, 0]),
                        ),
                    ],
                ),
            ]
        };
        let mut records = vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record(
                "ViewModel",
                vec![authoring_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Model".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyNumber",
                vec![authoring_property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("width".to_owned()),
                )],
            ),
            artboard(),
            authoring_record(
                "NestedArtboard",
                vec![
                    authoring_property("NestedArtboard", "parentId", AuthoringValue::Uint(0)),
                    authoring_property("NestedArtboard", "artboardId", AuthoringValue::Uint(1)),
                ],
            ),
            artboard(),
        ];
        records.extend(bound_rectangle(1.0));
        records.push(artboard());
        records.extend(bound_rectangle(2.0));
        let file = RuntimeFile::from_authoring_records(records)
            .expect("nested replacement fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("nested replacement graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("parent artboard instance");
        let host_local_id = graphs.artboards[0].nested_artboards[0].local_id;
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 0).expect("owned context");
        assert!(context.set_number_by_property_index(0, 42.0));

        assert!(parent.bind_owned_view_model_artboard_context(&file, &context));
        assert_eq!(
            parent
                .nested_artboards
                .get(&host_local_id)
                .and_then(|nested| nested.child.artboard_data_bind_values.get(&[0, 0][..])),
            Some(&RuntimeDataBindGraphValue::Number(42.0)),
            "the authored child establishes that the synthetic binding resolves"
        );

        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 2));
        let replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("replacement nested occurrence");
        assert_eq!(
            replacement.child.graph_global_id,
            graphs.artboards[2].global_id
        );
        assert_eq!(
            replacement.child.artboard_data_bind_values.get(&[0, 0][..]),
            Some(&RuntimeDataBindGraphValue::Number(42.0)),
            "C++ binds the existing DataContext during NestedArtboard::updateArtboard"
        );
    }

    #[test]
    fn stateful_nested_source_switch_uses_the_replacement_view_model_default() {
        let bytes = synthetic_riv(9700, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceNumber",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 2)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "NestedArtboard",
                &[("parentId", 0), ("artboardId", 1), ("isStateful", 1)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 0)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 2)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
        });
        let file = read_runtime_file(&bytes).expect("stateful source-switch fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("stateful fixture graphs");
        let parent_graph = &graph.artboards[0];
        let host_local_id = parent_graph.nested_artboards[0].local_id;
        let mut parent =
            ArtboardInstance::from_graph_with_artboards(&file, parent_graph, &graph.artboards)
                .expect("parent artboard instance");

        let authored = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("authored nested occurrence");
        assert_eq!(
            authored
                .stateful_view_model_context
                .as_ref()
                .map(RuntimeOwnedViewModelInstance::view_model_index),
            Some(0)
        );
        assert!(authored.stateful_view_model_instance_local.is_some());

        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 2));
        let replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("replacement nested occurrence");
        assert_eq!(
            replacement.child.graph_global_id,
            graph.artboards[2].global_id
        );
        assert_eq!(
            replacement
                .stateful_view_model_context
                .as_ref()
                .map(RuntimeOwnedViewModelInstance::view_model_index),
            Some(1),
            "a stateful source switch with no matching authored child must create the replacement VM default"
        );
        assert_eq!(replacement.stateful_view_model_instance_local, None);
        assert_eq!(
            replacement
                .stateful_global_view_model_contexts
                .get(&2)
                .map(RuntimeOwnedViewModelInstance::view_model_index),
            Some(2),
            "the replacement local main remains combined with authored global contexts"
        );

        let replacement_context_identity = replacement
            .stateful_view_model_context
            .as_ref()
            .expect("generated replacement context")
            .instance_identity();
        assert!(
            parent
                .nested_artboards
                .get_mut(&host_local_id)
                .and_then(|nested| nested.stateful_view_model_context.as_mut())
                .is_some_and(|context| context.set_number_by_property_index(0, 42.0))
        );
        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 3));
        let same_view_model_replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("same-VM replacement occurrence");
        assert_eq!(
            same_view_model_replacement
                .stateful_view_model_context
                .as_ref()
                .map(RuntimeOwnedViewModelInstance::instance_identity),
            Some(replacement_context_identity),
            "an owned replacement context survives a source switch to another artboard with the same VM"
        );
        assert_eq!(
            same_view_model_replacement
                .stateful_view_model_context
                .as_ref()
                .and_then(|context| context.number_value_by_slot(0)),
            Some(42.0),
            "same-VM reuse preserves runtime mutations"
        );
    }

    #[test]
    fn non_stateful_nested_host_does_not_activate_an_authored_child_view_model() {
        let bytes = synthetic_riv(9701, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "NestedArtboard",
                &[("parentId", 0), ("artboardId", 1), ("isStateful", 0)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
        });
        let file = read_runtime_file(&bytes).expect("non-stateful nested fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("non-stateful fixture graphs");
        let parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");
        let host_local_id = graph.artboards[0].nested_artboards[0].local_id;
        let nested = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("authored nested occurrence");

        assert_eq!(nested.stateful_view_model_instance_local, None);
        assert!(nested.stateful_view_model_context.is_none());
        assert!(nested.stateful_global_view_model_contexts.is_empty());
    }

    #[test]
    fn public_artboard_clone_is_cold_but_transient_layout_clone_keeps_scripts() {
        let mut original = synthetic_instance(Vec::new(), Vec::new());
        original.set_script_instance_for_global(
            7,
            Box::new(UpdateScriptInstance {
                inits: Rc::new(Cell::new(0)),
                updates: Rc::new(Cell::new(0)),
            }),
        );
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        child.set_script_instance_for_global(
            8,
            Box::new(UpdateScriptInstance {
                inits: Rc::new(Cell::new(0)),
                updates: Rc::new(Cell::new(0)),
            }),
        );
        child.layout_constraint_bounds_enabled = true;
        child.layout_constraint_bounds = Some(Arc::new(BTreeMap::from([(
            0,
            RuntimeLayoutBounds {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            },
        )])));
        original.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(Some(
                    RuntimeInitialNestedLayoutPaintFrame::default(),
                )),
                layout_data_transferred: true,
                layout_data_transfer_key: Some(RuntimeNestedLayoutDataTransferKey {
                    parent_layout: RuntimeNestedLayoutBoundsCacheKey {
                        graph_global_id: 11,
                        layout_epoch: 3,
                    },
                    assigned_bounds: RuntimeLayoutBounds {
                        x: 1.0,
                        y: 2.0,
                        width: 30.0,
                        height: 40.0,
                    },
                    child_layout_epoch: 5,
                }),
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 1.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );

        let original_identity = original.instance_identity();
        let original_nested_identity = original.nested_artboards[&0].child.instance_identity();
        let cloned = original.clone();
        let transient = original.clone_for_transient_layout();

        assert_ne!(cloned.instance_identity(), original_identity);
        assert_eq!(transient.instance_identity(), original_identity);
        assert_ne!(
            cloned.nested_artboards[&0].child.instance_identity(),
            original_nested_identity
        );
        assert_eq!(
            transient.nested_artboards[&0].child.instance_identity(),
            original_nested_identity
        );
        assert!(!cloned.nested_artboards[&0].layout_data_transferred);
        assert!(
            cloned.nested_artboards[&0]
                .layout_data_transfer_key
                .is_none()
        );
        assert!(
            cloned.nested_artboards[&0]
                .initial_layout_paint_frame
                .borrow()
                .is_none()
        );
        assert!(transient.nested_artboards[&0].layout_data_transferred);
        assert_eq!(
            transient.nested_artboards[&0].layout_data_transfer_key,
            original.nested_artboards[&0].layout_data_transfer_key
        );
        assert!(
            transient.nested_artboards[&0]
                .initial_layout_paint_frame
                .borrow()
                .is_none()
        );
        assert!(
            !cloned.nested_artboards[&0]
                .child
                .layout_constraint_bounds_enabled
        );
        assert!(
            cloned.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .is_none()
        );
        assert!(
            transient.nested_artboards[&0]
                .child
                .layout_constraint_bounds_enabled
        );
        assert!(Arc::ptr_eq(
            transient.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .as_ref()
                .expect("transient constraint bounds"),
            original.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .as_ref()
                .expect("source constraint bounds"),
        ));
        assert!(original.has_script_instance_for_global(7));
        assert!(!cloned.has_script_instance_for_global(7));
        assert!(transient.has_script_instance_for_global(7));
        assert!(
            !cloned
                .nested_artboards
                .get(&0)
                .is_some_and(|nested| nested.child.has_script_instance_for_global(8))
        );
        assert!(
            transient
                .nested_artboards
                .get(&0)
                .is_some_and(|nested| nested.child.has_script_instance_for_global(8))
        );
    }

    #[test]
    fn component_list_owner_clones_cold_but_transient_layout_keeps_retained_rows() {
        let mut original = synthetic_instance(
            vec![synthetic_component_for_type(0, "ArtboardComponentList")],
            vec![0],
        );
        {
            let list = original
                .component_list_state_mut(0)
                .expect("concrete component-list owner");
            list.item_transforms = vec![Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 8.0])];
            list.order_cache.borrow_mut().indices.push(0);
            list.order_cache.borrow_mut().valid = true;
        }

        let cloned = original.clone();
        let transient = original.clone_for_transient_layout();

        let cloned_list = cloned
            .component_list_state(0)
            .expect("clone retains concrete owner type");
        assert!(cloned_list.item_transforms.is_empty());
        assert!(cloned_list.order_cache.borrow().indices.is_empty());
        assert!(!cloned_list.order_cache.borrow().valid);

        let transient_list = transient
            .component_list_state(0)
            .expect("transient retains concrete owner type");
        assert_eq!(transient_list.item_transforms.len(), 1);
        assert_eq!(transient_list.order_cache.borrow().indices, [0]);
        assert!(transient_list.order_cache.borrow().valid);
    }

    #[test]
    fn scripted_updates_run_once_per_attach_or_input_change() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let inits = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(UpdateScriptInstance {
                inits: Rc::clone(&inits),
                updates: Rc::clone(&updates),
            }),
        );

        assert!(instance.update_script_instances().expect("initial update"));
        assert_eq!(updates.get(), 1);
        assert!(!instance.update_script_instances().expect("clean update"));

        instance
            .set_script_input_for_global(0, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(instance.update_script_instances().expect("dirty update"));
        assert_eq!(updates.get(), 2);

        assert!(
            instance
                .reinitialize_script_instances()
                .expect("reinitialize")
        );
        assert_eq!(inits.get(), 1);
        assert!(
            instance
                .update_script_instances()
                .expect("post-init update")
        );
        assert_eq!(updates.get(), 3);
    }

    #[test]
    fn scripted_updates_run_at_their_dependency_slots_and_retry_from_the_failed_owner() {
        let mut first = synthetic_component_for_type(0, "ScriptedDrawable");
        first.global_id = 9;
        let mut second = synthetic_component_for_type(1, "ScriptedDrawable");
        second.global_id = 2;
        let mut instance = synthetic_instance(vec![first, second], vec![1, 0]);
        let calls = Rc::new(RefCell::new(Vec::new()));
        let fail_once = Rc::new(Cell::new(true));
        instance.set_script_instance_for_global(
            9,
            Box::new(OrderedUpdateScriptInstance {
                label: 9,
                calls: Rc::clone(&calls),
                fail_once: None,
            }),
        );
        instance.set_script_instance_for_global(
            2,
            Box::new(OrderedUpdateScriptInstance {
                label: 2,
                calls: Rc::clone(&calls),
                fail_once: Some(Rc::clone(&fail_once)),
            }),
        );

        instance
            .update_pass_with_script_errors()
            .expect_err("the first dependency owner fails once");
        assert_eq!(calls.borrow().as_slice(), [2, 9]);

        instance
            .update_pass_with_script_errors()
            .expect("the failed owner remains dirtied for an exact retry");
        assert_eq!(
            calls.borrow().as_slice(),
            [2, 9, 2],
            "ScriptedDrawable::update consumes ScriptUpdate at the concrete Component's retained \
             dependency slot; a failure rearms that owner without replaying an already-successful \
             later slot (`component.cpp:222-241`; `scripted_drawable.cpp:347-374`)"
        );
    }

    #[test]
    fn nested_scripts_advance_in_place_with_exact_local_speed_adjusted_steps() {
        let seconds = Rc::new(RefCell::new(Vec::new()));
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        child.set_script_instance_for_global(
            0,
            Box::new(RecordingAdvanceScriptInstance {
                seconds: Rc::clone(&seconds),
            }),
        );
        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        parent.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 2.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );
        parent.nested_artboard_locals.push(0);

        let mut factory = RecordingFactory::new();
        parent
            .advance_frame_components_with_factory(0.25, &mut factory)
            .expect("first retained advance succeeds");
        parent
            .advance_frame_components_with_factory(0.125, &mut factory)
            .expect("second retained advance succeeds");

        assert_eq!(seconds.borrow().as_slice(), [0.5, 0.25]);
    }

    #[test]
    fn state_machine_batch_advances_nested_scripts_once() {
        let seconds = Rc::new(RefCell::new(Vec::new()));
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        child.set_script_instance_for_global(
            0,
            Box::new(RecordingAdvanceScriptInstance {
                seconds: Rc::clone(&seconds),
            }),
        );
        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        parent.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 2.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );
        parent.nested_artboard_locals.push(0);
        parent.state_machines = Arc::new(vec![empty_state_machine(11), empty_state_machine(12)]);
        let definitions = Arc::clone(&parent.state_machines);
        let mut machines = definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| StateMachineInstance::new(index, definition, &mut parent))
            .collect::<Vec<_>>();

        let mut factory = RecordingFactory::new();
        parent
            .advance_frame_components_with_state_machines_and_factory(
                &mut machines,
                0.25,
                &mut factory,
            )
            .expect("one mixed-family frame succeeds");

        assert_eq!(seconds.borrow().as_slice(), [0.5]);
    }

    #[test]
    fn failed_script_advance_preserves_active_state_for_exact_retry() {
        let attempts = Rc::new(RefCell::new(Vec::new()));
        let should_fail = Rc::new(Cell::new(true));
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        instance.set_script_instance_for_global(
            0,
            Box::new(FailOnceAdvanceScriptInstance {
                attempts: Rc::clone(&attempts),
                should_fail: Rc::clone(&should_fail),
            }),
        );
        let mut factory = RecordingFactory::new();

        instance
            .advance_frame_components_with_factory(0.5, &mut factory)
            .expect_err("first advance fails");
        instance
            .advance_frame_components_with_factory(0.5, &mut factory)
            .expect("the exact owner is retryable");
        instance
            .advance_frame_components_with_factory(0.25, &mut factory)
            .expect("the following frame remains independent");

        assert_eq!(attempts.borrow().as_slice(), [0.5, 0.5, 0.25]);
    }

    #[test]
    fn scripted_advances_stop_on_false_and_reactivate_on_input_change() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("first advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("second advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("inactive advance")
        );
        assert_eq!(advances.get(), 2);

        instance
            .set_script_input_for_global(0, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("reactivated advance")
        );
        assert_eq!(advances.get(), 3);
    }

    #[test]
    fn scripted_advances_follow_retained_object_order_not_global_id_order() {
        let mut first = synthetic_component_for_type(0, "ScriptedDrawable");
        first.global_id = 9;
        let mut second = synthetic_component_for_type(1, "ScriptedDrawable");
        second.global_id = 2;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let calls = Rc::new(RefCell::new(Vec::new()));
        for global_id in [9, 2] {
            instance.set_script_instance_for_global(
                global_id,
                Box::new(OrderedAdvanceScriptInstance {
                    label: global_id,
                    calls: Rc::clone(&calls),
                }),
            );
        }

        assert!(instance.advance_frame_components(0.1).unwrap());
        assert_eq!(
            calls.borrow().as_slice(),
            [9, 2],
            "the complete frame entry point walks Artboard::m_advancingComponents insertion \
             order, not a late global-id script sweep (`artboard.cpp:1463-1480`; \
             `advancing_component.cpp:17-44`)"
        );
    }

    #[test]
    fn collapsed_scripted_component_defers_update_and_advance_until_visible() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "ScriptedDrawable"),
                synthetic_component_for_type(1, "ScriptedDrawable"),
            ],
            vec![0, 1],
        );
        let inits = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(UpdateScriptInstance {
                inits: Rc::clone(&inits),
                updates: Rc::clone(&updates),
            }),
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            1,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(instance.collapse_component(0, true));
        assert!(instance.collapse_component(1, true));
        assert!(
            !instance
                .update_script_instances()
                .expect("collapsed update is deferred")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("collapsed advance is deferred")
        );
        assert_eq!(updates.get(), 0);
        assert_eq!(advances.get(), 0);

        assert!(instance.collapse_component(0, false));
        assert!(instance.collapse_component(1, false));
        assert!(
            instance
                .update_script_instances()
                .expect("deferred update runs when visible")
        );
        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("armed advance runs when visible")
        );
        assert_eq!(updates.get(), 1);
        assert_eq!(advances.get(), 1);
    }

    #[test]
    fn waking_a_parked_script_advance_rearms_it_and_marks_paint_dirty() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(instance.advance_script_instances(0.1).unwrap());
        assert!(!instance.advance_script_instances(0.1).unwrap());
        instance.clear_component_dirt(0);

        assert!(instance.wake_script_advance_for_global(0));
        assert!(
            instance
                .component(0)
                .expect("scripted drawable component")
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(instance.advance_script_instances(0.1).unwrap());
        assert_eq!(advances.get(), 3);
    }

    #[test]
    fn successful_script_advance_invalidates_paint_without_calling_update() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceAndUpdateScriptInstance {
                advances: Rc::clone(&advances),
                updates: Rc::clone(&updates),
            }),
        );
        assert!(instance.update_script_instances().unwrap());
        assert_eq!(updates.get(), 1);
        instance.clear_component_dirt(0);

        assert!(instance.advance_script_instances(0.1).unwrap());
        assert_eq!(advances.get(), 1);
        assert!(
            instance
                .component(0)
                .expect("scripted drawable component")
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(!instance.update_script_instances().unwrap());
        assert_eq!(updates.get(), 1);
    }

    #[test]
    fn render_opacity_update_invalidates_a_prepared_zero_opacity_frame() {
        let mut instance = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        assert_eq!(instance.component(0).unwrap().transform.render_opacity, 0.0);
        let prepared_epoch = instance.prepared_epoch;

        let component = instance.component_handle(0).unwrap();
        instance.update_component(component, ComponentDirt::RENDER_OPACITY);

        assert_eq!(instance.component(0).unwrap().transform.render_opacity, 1.0);
        assert!(instance.prepared_epoch > prepared_epoch);
    }

    #[test]
    fn nested_animation_runtime_knobs_follow_keyed_parent_properties() {
        let animation_instance = |animation_index| {
            let animation = RuntimeLinearAnimation {
                global_id: animation_index as u32,
                name: None,
                fps: 60,
                duration: 60,
                speed: 1.0,
                loop_value: 0,
                work_start: 0,
                work_end: 60,
                enable_work_area: false,
                quantize: false,
                keyed_objects: Arc::new(Vec::new()),
                key_frame_data_bind_templates: Arc::new(Vec::new()),
                has_keyed_callbacks: false,
            };
            LinearAnimationInstance::new(
                RuntimeLinearAnimationHandle::new(animation_index),
                &animation,
                1.0,
            )
        };

        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboard";
        host.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(host.type_name);
        let mut simple = synthetic_component(1, 1);
        simple.type_name = "NestedSimpleAnimation";
        simple.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(simple.type_name);
        let mut remap = synthetic_component(2, 2);
        remap.type_name = "NestedRemapAnimation";
        remap.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(remap.type_name);

        let mut instance = synthetic_instance(vec![host, simple, remap], Vec::new());
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        let mut nested = synthetic_nested_artboard_instance(7);
        nested.animations = vec![
            RuntimeNestedAnimationInstance::Simple {
                local_id: 1,
                animation: animation_instance(0),
                is_playing: false,
                speed: 1.0,
                mix: 1.0,
            },
            RuntimeNestedAnimationInstance::Remap {
                local_id: 2,
                animation: animation_instance(1),
                mix: 1.0,
            },
        ];
        instance.nested_artboards.insert(0, nested);

        let mix_key = property_key_for_name("NestedLinearAnimation", "mix").expect("mix key");
        let speed_key = property_key_for_name("NestedSimpleAnimation", "speed").expect("speed key");
        let playing_key =
            property_key_for_name("NestedSimpleAnimation", "isPlaying").expect("isPlaying key");
        assert!(instance.set_keyed_double_property(1, mix_key, 0.25));
        assert!(instance.set_keyed_double_property(2, mix_key, 0.0));
        assert!(instance.set_keyed_double_property(1, speed_key, 2.0));
        assert!(instance.set_bool_property(1, playing_key, true));

        let nested = instance.nested_artboards.get(&0).expect("nested host");
        match &nested.animations[0] {
            RuntimeNestedAnimationInstance::Simple {
                is_playing,
                speed,
                mix,
                ..
            } => {
                assert!(*is_playing);
                assert_eq!(*speed, 2.0);
                assert_eq!(*mix, 0.25);
            }
            _ => panic!("expected simple animation"),
        }
        match &nested.animations[1] {
            RuntimeNestedAnimationInstance::Remap { mix, .. } => assert_eq!(*mix, 0.0),
            _ => panic!("expected remap animation"),
        }
    }

    fn synthetic_component(local_id: usize, _graph_order: usize) -> RuntimeComponent {
        RuntimeComponent {
            local_id,
            global_id: local_id as u32,
            type_name: "Node",
            transform_property_keys: crate::components::TransformPropertyKeys::for_type("Node"),
            capabilities: RuntimeComponentCapabilities {
                world_transform: true,
                transform: true,
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
            dirt: ComponentDirt::NONE,
            transform: TransformRuntimeState::default(),
            concrete: crate::components::RuntimeConcreteComponentState::for_type("Node"),
        }
    }

    fn synthetic_component_for_type(local_id: usize, type_name: &'static str) -> RuntimeComponent {
        let mut component = synthetic_component(local_id, local_id);
        let definition =
            nuxie_schema::definition_by_name(type_name).expect("synthetic type exists");
        component.type_name = type_name;
        component.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(type_name);
        component.capabilities = RuntimeComponentCapabilities {
            world_transform: definition.is_a("WorldTransformComponent"),
            transform: definition.is_a("TransformComponent"),
        };
        component.concrete = crate::components::RuntimeConcreteComponentState::for_type(type_name);
        component
    }

    fn synthetic_typed_component(local_id: usize, type_name: &'static str) -> RuntimeComponent {
        let definition = definition_by_name(type_name).expect("synthetic Component type");
        let mut component = synthetic_component(local_id, local_id);
        component.type_name = type_name;
        component.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(type_name);
        component.capabilities = RuntimeComponentCapabilities {
            world_transform: definition.is_a("WorldTransformComponent"),
            transform: definition.is_a("TransformComponent"),
        };
        component.concrete = crate::components::RuntimeConcreteComponentState::for_type(type_name);
        component
    }

    fn synthetic_link_parent(instance: &mut ArtboardInstance, child: usize, parent: usize) {
        let child = instance.component_handle(child).expect("synthetic child");
        let parent = instance.component_handle(parent).expect("synthetic parent");
        assert!(instance.objects.link_parent(child, parent));
        if instance
            .objects
            .component(child)
            .is_some_and(|component| component.capabilities.transform)
            && instance
                .objects
                .component(parent)
                .is_some_and(|component| component.capabilities.world_transform)
        {
            instance
                .objects
                .component_mut(child)
                .expect("synthetic child remains live")
                .parent_transform = Some(parent);
        }
    }

    fn synthetic_add_dependent(instance: &mut ArtboardInstance, source: usize, dependent: usize) {
        let source = instance.component_handle(source).expect("synthetic source");
        let dependent = instance
            .component_handle(dependent)
            .expect("synthetic dependent");
        assert!(instance.objects.add_dependent(source, dependent));
    }

    #[test]
    fn layout_drawable_hit_test_preserves_skip_hidden_and_parent_fallback() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "LayoutComponent")],
            vec![0],
        );
        let layout = instance.component_handle(0).expect("layout handle");
        instance
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout owner")
            .retain_bounds(0.0, 0.0, 100.0, 50.0);

        assert!(instance.component_hit_test_point(layout, (25.0, 25.0), false, true));
        assert!(!instance.component_hit_test_point(layout, (125.0, 25.0), false, true));
        assert!(
            instance.component_hit_test_point(layout, (125.0, 25.0), true, true),
            "skipOnUnclipped bypasses only an unclipped Layout's local bounds"
        );

        let drawable_flags =
            property_key_for_name("LayoutComponent", "drawableFlags").expect("Drawable flags");
        assert!(instance.set_uint_property(0, drawable_flags, 1));
        assert!(
            !instance.component_hit_test_point(layout, (25.0, 25.0), false, true),
            "Drawable::hitTestPoint rejects its sole generated Hidden flag"
        );
    }

    #[test]
    fn node_computed_local_uses_settled_world_and_retained_parent_transform() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Node"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        let parent_world = Mat2D([2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
        let expected_local = Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0]);
        instance.component_mut(0).unwrap().transform.world_transform = parent_world;
        instance.component_mut(1).unwrap().transform.world_transform =
            parent_world.multiply(expected_local);
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();

        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(expected_local)
        );

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([0.0, 0.0, 0.0, 0.0, 7.0, 9.0]);
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );

        instance.component_mut(1).unwrap().parent_transform = None;
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );
    }

    #[test]
    fn transform_render_opacity_uses_cpp_child_opacity_dispatch() {
        let mut world_parent = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Artboard"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut world_parent, 1, 0);
        assert!(
            world_parent
                .objects
                .set_double_property_by_name(0, "opacity", 0.5)
        );
        assert!(
            world_parent
                .objects
                .set_double_property_by_name(1, "opacity", 0.8)
        );
        let root = world_parent.component_handle(0).unwrap();
        world_parent.update_component(root, ComponentDirt::RENDER_OPACITY);
        let child = world_parent.component_handle(1).unwrap();
        world_parent.update_component(child, ComponentDirt::RENDER_OPACITY);
        assert_eq!(
            world_parent.component(1).unwrap().transform.render_opacity,
            0.4
        );

        let mut transform_parent = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Node"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut transform_parent, 1, 0);
        assert!(
            transform_parent
                .objects
                .set_double_property_by_name(0, "opacity", 0.5)
        );
        assert!(
            transform_parent
                .objects
                .set_double_property_by_name(1, "opacity", 0.8)
        );
        transform_parent
            .component_mut(0)
            .unwrap()
            .transform
            .render_opacity = 0.25;
        let child = transform_parent.component_handle(1).unwrap();
        transform_parent.update_component(child, ComponentDirt::RENDER_OPACITY);
        assert_eq!(
            transform_parent
                .component(1)
                .unwrap()
                .transform
                .render_opacity,
            0.2
        );
    }

    #[test]
    fn skin_on_dirty_targets_only_the_retained_skinnable_dirt_family() {
        for (type_name, expected) in [
            ("PointsPath", ComponentDirt::PATH),
            ("Mesh", ComponentDirt::VERTICES),
        ] {
            let mut instance = synthetic_instance(
                vec![
                    synthetic_typed_component(0, type_name),
                    synthetic_typed_component(1, "Skin"),
                ],
                vec![0, 1],
            );
            synthetic_link_parent(&mut instance, 1, 0);
            let skinnable = instance.component_handle(0).unwrap();
            let skin = instance.component_handle(1).unwrap();
            instance
                .component_mut(0)
                .unwrap()
                .concrete
                .skinnable
                .as_mut()
                .unwrap()
                .skin = Some(skin);
            instance
                .component_mut(1)
                .unwrap()
                .concrete
                .skin
                .as_mut()
                .unwrap()
                .skinnable = Some(skinnable);
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
            instance.component_mut(1).unwrap().dirt = ComponentDirt::NONE;
            instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

            assert!(instance.add_component_dirt(skin, ComponentDirt::SKIN, false));
            assert_eq!(instance.component(1).unwrap().dirt, ComponentDirt::SKIN);
            assert!(instance.component(0).unwrap().dirt.contains(expected));
            let other = if expected == ComponentDirt::PATH {
                ComponentDirt::VERTICES
            } else {
                ComponentDirt::PATH
            };
            assert!(!instance.component(0).unwrap().dirt.contains(other));
        }
    }

    #[test]
    fn skin_update_rebuilds_only_its_retained_tendon_buffer() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_typed_component(0, "RootBone"),
                synthetic_typed_component(1, "Tendon"),
                synthetic_typed_component(2, "Skin"),
                synthetic_typed_component(3, "Mesh"),
            ],
            vec![0, 1, 2, 3],
        );
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        let bone_world = Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0]);
        let inverse_bind = Mat2D([1.0, 0.0, 0.0, 1.0, -1.0, -2.0]);
        instance.component_mut(0).unwrap().transform.world_transform = bone_world;
        {
            let tendon_state = instance
                .component_mut(1)
                .unwrap()
                .concrete
                .tendon
                .as_mut()
                .unwrap();
            tendon_state.bone = Some(bone);
            tendon_state.inverse_bind = inverse_bind;
        }
        {
            let skin_state = instance
                .component_mut(2)
                .unwrap()
                .concrete
                .skin
                .as_mut()
                .unwrap();
            skin_state.tendons.push(tendon);
            skin_state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }

        instance.update_component(skin, ComponentDirt::FILTHY);
        let expected = bone_world.multiply(inverse_bind);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(state.bone_transforms, vec![Mat2D::IDENTITY, expected]);
        assert_eq!(state.buffer_rebuilds, 1);

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 11.0]);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(state.bone_transforms[1], expected);
        assert_eq!(state.buffer_rebuilds, 1);

        instance.update_component(skin, ComponentDirt::SKIN);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(
            state.bone_transforms[1],
            Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 11.0]).multiply(inverse_bind)
        );
        assert_eq!(state.buffer_rebuilds, 2);
    }

    fn callback_route_animation(has_keyed_callbacks: bool) -> RuntimeLinearAnimation {
        let keyed_objects = if has_keyed_callbacks {
            vec![RuntimeKeyedObject {
                global_id: 1,
                object_id: 0,
                target_local_id: 0,
                keyed_properties: vec![RuntimeKeyedProperty {
                    global_id: 2,
                    property_key: 0,
                    target: RuntimeKeyedPropertyTarget::Callback {
                        event: Some(StateMachineReportedEvent {
                            event_local_index: 0,
                            event_core_type: 0,
                            name: Some("callback".to_owned()),
                            url: None,
                            target: None,
                            properties: Vec::new(),
                            string_properties: Vec::new(),
                            context: None,
                            seconds_delay: 0.0,
                        }),
                    },
                    key_frames: vec![RuntimeKeyFrame::Callback(RuntimeKeyFrameCallback {
                        global_id: 3,
                        frame: 1,
                        seconds: 1.0,
                    })],
                }],
            }]
        } else {
            Vec::new()
        };
        RuntimeLinearAnimation {
            global_id: 0,
            name: None,
            fps: 1,
            duration: 2,
            speed: 1.0,
            loop_value: 0,
            work_start: 0,
            work_end: 2,
            enable_work_area: false,
            quantize: false,
            keyed_objects: Arc::new(keyed_objects),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks,
        }
    }

    #[test]
    fn animation_advance_routes_only_callback_definitions_through_event_reporting() {
        for (has_keyed_callbacks, expected_events) in [(false, 0), (true, 1)] {
            let mut artboard = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
            artboard.linear_animations =
                Arc::new(vec![callback_route_animation(has_keyed_callbacks)]);
            let mut animation = artboard
                .linear_animation_instance(0)
                .expect("test animation instance");
            let mut events = Vec::new();

            assert!(artboard.advance_linear_animation_instance_with_events(
                &mut animation,
                1.0,
                &mut events,
            ));
            assert_eq!(animation.time(), 1.0);
            assert_eq!(events.len(), expected_events);
        }
    }

    #[test]
    fn artboard_clones_share_one_linear_animation_definition_arena() {
        let mut artboard = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        artboard.linear_animations = Arc::new(vec![callback_route_animation(false)]);

        let cloned = artboard.clone();
        let first = artboard
            .linear_animation_instance(0)
            .expect("first occurrence");
        let second = cloned
            .linear_animation_instance(0)
            .expect("cloned occurrence");

        assert!(Arc::ptr_eq(
            &artboard.linear_animations,
            &cloned.linear_animations
        ));
        assert_eq!(first.animation_index(), second.animation_index());
    }

    #[test]
    fn state_machine_outer_settlement_clears_deep_nested_render_opacity_dirt() {
        let typed_component = |local_id: usize, graph_order: usize, type_name: &'static str| {
            let mut component = synthetic_component(local_id, graph_order);
            component.type_name = type_name;
            component.transform_property_keys =
                crate::components::TransformPropertyKeys::for_type(type_name);
            component
        };

        let mut leaf_root = typed_component(0, 0, "Artboard");
        leaf_root.transform.render_opacity = 0.0;
        let mut leaf = synthetic_instance(vec![leaf_root], vec![0]);
        let opacity_key = property_key_for_name("Artboard", "opacity").expect("opacity key");
        assert!(leaf.set_double_property(0, opacity_key, 0.0));
        leaf.clear_component_dirt(0);
        leaf.set_artboard_dirt_for_test(ComponentDirt::NONE);

        let mut middle_root = typed_component(0, 0, "Artboard");
        middle_root.transform.render_opacity = 1.0;
        let mut middle_host = typed_component(1, 1, "NestedArtboard");
        middle_host.dirt = ComponentDirt::RENDER_OPACITY;
        let mut middle = synthetic_instance(vec![middle_root, middle_host], vec![1]);
        synthetic_link_parent(&mut middle, 1, 0);
        let mut leaf_mount = synthetic_nested_artboard_instance(2);
        leaf_mount.child = Box::new(leaf);
        middle.nested_artboards.insert(1, leaf_mount);
        middle.nested_artboard_locals.push(1);
        middle.set_artboard_dirt_for_test(ComponentDirt::COMPONENTS);

        let mut root_component = typed_component(0, 0, "Artboard");
        root_component.transform.render_opacity = 1.0;
        let mut root_host = typed_component(1, 1, "NestedArtboard");
        root_host.transform.render_opacity = 1.0;
        root_host.dirt = ComponentDirt::COMPONENTS;
        let mut root = synthetic_instance(vec![root_component, root_host], vec![1]);
        synthetic_link_parent(&mut root, 1, 0);
        let mut middle_mount = synthetic_nested_artboard_instance(1);
        middle_mount.child = Box::new(middle);
        root.nested_artboards.insert(1, middle_mount);
        root.nested_artboard_locals.push(1);

        root.update_pass();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert!(leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));

        root.settle_state_machine_update_passes();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 1.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));
    }

    #[test]
    fn component_dirt_bits_match_cpp_layout() {
        assert_eq!(ComponentDirt::NONE.0, 0);
        assert_eq!(ComponentDirt::COLLAPSED.0, 1 << 0);
        assert_eq!(ComponentDirt::DEPENDENTS.0, 1 << 1);
        assert_eq!(ComponentDirt::COMPONENTS.0, 1 << 2);
        assert_eq!(ComponentDirt::DRAW_ORDER.0, 1 << 3);
        assert_eq!(ComponentDirt::PATH.0, 1 << 4);
        assert_eq!(ComponentDirt::TEXT_SHAPE.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::SKIN.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::VERTICES.0, 1 << 5);
        assert_eq!(ComponentDirt::TEXT_COVERAGE.0, ComponentDirt::VERTICES.0);
        assert_eq!(ComponentDirt::TRANSFORM.0, 1 << 6);
        assert_eq!(ComponentDirt::WORLD_TRANSFORM.0, 1 << 7);
        assert_eq!(ComponentDirt::RENDER_OPACITY.0, 1 << 8);
        assert_eq!(ComponentDirt::PAINT.0, 1 << 9);
        assert_eq!(ComponentDirt::STOPS.0, 1 << 10);
        assert_eq!(ComponentDirt::LAYOUT_STYLE.0, 1 << 11);
        assert_eq!(ComponentDirt::BINDINGS.0, 1 << 12);
        assert_eq!(ComponentDirt::N_SLICER.0, 1 << 13);
        assert_eq!(ComponentDirt::SCRIPT_UPDATE.0, 1 << 14);
        assert_eq!(ComponentDirt::CLIPPING.0, 1 << 15);
        assert_eq!(ComponentDirt::FILTHY.0, 0xFFFE);
    }

    #[test]
    fn retained_reset_schedule_recurses_into_nested_artboards() {
        let property_value_key = property_key_for_name("CustomPropertyTrigger", "propertyValue")
            .expect("trigger property value");
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "CustomPropertyTrigger")],
            vec![0],
        );
        assert!(child.set_uint_property(0, property_value_key, 3));

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        let mut nested = synthetic_nested_artboard_instance(1);
        nested.child = Box::new(child);
        parent.nested_artboards.insert(0, nested);

        parent.reset_retained_components();

        assert_eq!(
            parent
                .nested_artboards
                .get(&0)
                .and_then(|nested| { nested.child.uint_property(0, property_value_key) }),
            Some(0),
            "Artboard::reset walks the authored reset schedule recursively \
             (`artboard.cpp:1483-1493`; `nested_artboard.cpp:1035-1043`)"
        );
    }

    #[test]
    fn component_list_reset_visits_unmounted_logical_rows() {
        let (file, _, _) = owned_view_model_action_fixture(9_706, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row context"),
        );
        assert!(context.borrow_mut().set_trigger_by_property_index(2, 1));
        let mut list = RuntimeConstrainableListState::default();
        list.logical_items.push(RuntimeComponentListLogicalItem {
            occurrence_identity: 41,
            context: context.clone(),
            size: (100.0, 20.0),
            mapped_artboard_global: None,
        });
        assert!(
            list.items.is_empty(),
            "logical row is deliberately unmounted"
        );

        reset_component_list_instances(&mut list, true);

        assert_eq!(
            context.borrow().trigger_value_by_property_path(&[2]),
            Some(0),
            "C++ acknowledges every logical m_listItems VMI before the \
             optional m_artboardInstancesMap lookup \
             (`artboard_component_list.cpp:888-920`)"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_swaps_input_and_output_ranges() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 0,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 6.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 6, got {value}"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_preserves_reverse_flag() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 1 << 3,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 4.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 4, got {value}"
        );
    }

    #[test]
    fn add_dirt_recurses_to_graph_dependents() {
        let source = synthetic_component(0, 0);
        let first_dependent = synthetic_component(1, 1);
        let second_dependent = synthetic_component(2, 2);
        let mut instance = synthetic_instance(
            vec![source, first_dependent, second_dependent],
            vec![0, 1, 2],
        );
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 0, 1);
        let source_handle = instance.component_handle(0).expect("source handle");
        let second_handle = instance
            .component_handle(2)
            .expect("second dependent handle");
        assert!(!instance.objects.add_dependent(source_handle, second_handle));
        assert_eq!(
            instance.objects.dependent_at(source_handle, 0),
            instance.component_handle(2)
        );
        assert_eq!(
            instance.objects.dependent_at(source_handle, 1),
            instance.component_handle(1)
        );
        assert_eq!(instance.objects.dependent_len(source_handle), 2);

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        assert!(!instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH | ComponentDirt::COMPONENTS),
            "The root Artboard is the same retained Component dirt owner that publishes Components loop-control dirt (`src/component.cpp:32-45`; `src/artboard.cpp:1205-1241`)"
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            instance
                .component(2)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));

        assert!(!instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH | ComponentDirt::PAINT)
        );
    }

    #[test]
    fn occurrence_dependency_sort_matches_cpp_diamond_visitation_order() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component(0, 0),
                synthetic_component(1, 1),
                synthetic_component(2, 2),
                synthetic_component(3, 3),
            ],
            Vec::new(),
        );
        synthetic_add_dependent(&mut instance, 0, 1);
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 1, 3);
        synthetic_add_dependent(&mut instance, 2, 3);

        assert!(instance.objects.sort_dependencies_from_root());
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter_map(|handle| instance.objects.component_local_id(*handle))
                .collect::<Vec<_>>(),
            vec![0, 2, 1, 3],
            "DependencySorter visits retained dependents in insertion order and front-inserts each completed owner (`src/dependency_sorter.cpp:6-48`)"
        );
    }

    #[test]
    fn occurrence_dependency_sort_publishes_cpp_partial_order_on_cycle() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component(0, 0),
                synthetic_component(1, 1),
                synthetic_component(2, 2),
            ],
            Vec::new(),
        );
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 0, 1);
        synthetic_add_dependent(&mut instance, 1, 0);

        assert!(!instance.objects.sort_dependencies_from_root());
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter_map(|handle| instance.objects.component_local_id(*handle))
                .collect::<Vec<_>>(),
            vec![2],
            "DependencySorter::sort ignores visit's cycle result and publishes owners completed before the cyclic branch (`src/dependency_sorter.cpp:6-10`)"
        );
        assert_eq!(
            instance
                .component(2)
                .and_then(|component| component.graph_order),
            Some(crate::components::GraphOrder::new(0))
        );
        assert_eq!(
            instance
                .component(0)
                .and_then(|component| component.graph_order),
            None
        );
        assert_eq!(
            instance
                .component(1)
                .and_then(|component| component.graph_order),
            None
        );
    }

    #[test]
    fn enabling_layout_constraint_bounds_dirties_layout_dependents() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![layout, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        let prepared_epoch = instance.prepared_epoch();

        instance.enable_layout_constraint_bounds();

        assert!(instance.layout_constraint_bounds_enabled);
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.prepared_epoch() > prepared_epoch);

        let cache_epoch = instance.cache_epoch();
        instance.enable_layout_constraint_bounds();
        assert_eq!(instance.cache_epoch(), cache_epoch);
    }

    #[test]
    fn retained_layout_owner_keeps_parent_local_coordinates_for_constraint_offsets() {
        let root = synthetic_component_for_type(0, "Artboard");
        let parent = synthetic_component_for_type(1, "LayoutComponent");
        let child = synthetic_component_for_type(2, "LayoutComponent");
        let mut instance = synthetic_instance(vec![root, parent, child], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        let accumulated = BTreeMap::from([
            (
                0,
                RuntimeLayoutBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 300.0,
                    height: 200.0,
                },
            ),
            (
                1,
                RuntimeLayoutBounds {
                    x: 11.0,
                    y: 19.0,
                    width: 100.0,
                    height: 80.0,
                },
            ),
            (
                2,
                RuntimeLayoutBounds {
                    x: 28.0,
                    y: 42.0,
                    width: 40.0,
                    height: 30.0,
                },
            ),
        ]);

        instance.retain_runtime_layout_component_bounds(2, accumulated[&2], Some(&accumulated));
        let layout = instance
            .component(2)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("retained LayoutComponent state");
        assert_eq!(layout.transform_property(TransformProperty::X), Some(17.0));
        assert_eq!(layout.transform_property(TransformProperty::Y), Some(23.0));
    }

    #[test]
    fn layout_world_update_uses_retained_position_and_root_origin() {
        // LayoutComponent::update replaces TransformComponent's authored
        // local matrix with retained Yoga left/top and subtracts the root
        // Artboard normalized origin before multiplying by its parent
        // (`src/layout/layout_component.cpp:82-121`).
        let root = synthetic_component_for_type(0, "Artboard");
        let layout = synthetic_component_for_type(1, "LayoutComponent");
        let mut instance = synthetic_instance(vec![root, layout], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.width = 100.0;
        instance.height = 80.0;
        instance.origin_x = 0.5;
        instance.origin_y = 0.25;
        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
        instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("retained LayoutComponent state")
            .retain_bounds(60.0, 70.0, 40.0, 30.0);

        let layout = instance.component_handle(1).unwrap();
        instance.update_component(layout, ComponentDirt::WORLD_TRANSFORM);

        assert_eq!(
            instance.component(1).unwrap().transform.world_transform,
            Mat2D([2.0, 0.0, 0.0, 3.0, 30.0, 170.0])
        );
    }

    #[test]
    fn layout_ancestry_retains_exact_nearest_first_owner_identity() {
        // C++ `Drawable::isChildOfLayout` walks from the queried Component
        // through concrete parent pointers and compares LayoutComponent
        // identity (`src/drawable.cpp:45-59`). The Rust occurrence retains the
        // same identities instead of the displaced boolean-only mirror.
        let root = synthetic_component_for_type(0, "Artboard");
        let outer = synthetic_component_for_type(1, "LayoutComponent");
        let inner = synthetic_component_for_type(2, "LayoutComponent");
        let shape = synthetic_component_for_type(3, "Shape");
        let mut instance = synthetic_instance(vec![root, outer, inner, shape], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 2);
        crate::components::retain_runtime_component_layout_topology(&mut instance.objects);

        let outer = instance.component_handle(1).expect("outer layout");
        let inner = instance.component_handle(2).expect("inner layout");
        assert_eq!(
            instance.component(3).expect("shape").layout_ancestors,
            vec![inner, outer]
        );
        assert_eq!(
            instance.component(2).expect("inner").layout_ancestors,
            vec![inner, outer],
            "C++ begins the identity walk at `this`"
        );
        assert!(
            instance
                .component(0)
                .expect("root")
                .layout_ancestors
                .is_empty()
        );
    }

    #[test]
    fn nested_layout_constraint_space_refreshes_for_parent_or_child_layout_generation() {
        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboardLayout";
        let mut parent = synthetic_instance(vec![host], vec![0]);

        let mut child_layout = synthetic_component(0, 0);
        child_layout.type_name = "LayoutComponent";
        let mut child =
            synthetic_instance(vec![child_layout, synthetic_component(1, 1)], vec![0, 1]);
        synthetic_add_dependent(&mut child, 0, 1);
        let mut nested = synthetic_nested_artboard_instance(7);
        nested.child = Box::new(child);
        parent.nested_artboards.insert(0, nested);

        let assigned_bounds = RuntimeLayoutBounds {
            x: 4.0,
            y: 5.0,
            width: 120.0,
            height: 80.0,
        };
        let layout_bounds = BTreeMap::from([(0, assigned_bounds)]);
        let first_parent_layout = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: 3,
            layout_epoch: 9,
        };

        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        let first_transfer_key = parent.nested_artboards[&0].layout_data_transfer_key;
        let first_cache_epoch = parent.nested_artboards[&0].child.cache_epoch();

        assert!(!parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0].child.cache_epoch(),
            first_cache_epoch
        );

        // The assigned root writes from the first transfer are already part of
        // the stored generation (the identical apply above stabilized). A
        // later child layout change emulates C++ bubbling
        // `markHostingLayoutDirty` back to the owner of the Yoga node.
        parent
            .nested_artboards
            .get_mut(&0)
            .expect("nested child")
            .child
            .mark_layout_changed();
        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        let after_child_refresh = parent.nested_artboards[&0].child.cache_epoch();
        let after_child_transfer_key = parent.nested_artboards[&0].layout_data_transfer_key;
        assert_ne!(after_child_transfer_key, first_transfer_key);
        assert!(!parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0].child.cache_epoch(),
            after_child_refresh
        );

        let next_parent_layout = RuntimeNestedLayoutBoundsCacheKey {
            layout_epoch: first_parent_layout.layout_epoch + 1,
            ..first_parent_layout
        };
        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            next_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0]
                .layout_data_transfer_key
                .expect("refreshed transfer")
                .parent_layout,
            next_parent_layout
        );
        assert!(parent.nested_artboards[&0].child.cache_epoch() > after_child_refresh);
    }

    #[test]
    fn path_dirt_tracks_geometry_revision_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PATH, false));
        assert!(instance.path_epoch() > initial_path_epoch);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let path_epoch = instance.path_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::PATH, false));
        assert_eq!(instance.path_epoch(), path_epoch);

        assert!(instance.collapse_component(0, true));
        assert!(instance.path_epoch() > path_epoch);
    }

    #[test]
    fn world_transform_dirt_invalidates_world_state_without_rebuilding_paths() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.add_dirt(0, ComponentDirt::WORLD_TRANSFORM, false));

        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn path_epoch_tracks_effect_path_property_changes() {
        let mut trim = synthetic_component(0, 0);
        trim.type_name = "TrimPath";
        let mut instance = synthetic_instance(vec![trim], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "TrimPath", Vec::new()),
        )]);
        let trim_start = property_key_for_name("TrimPath", "start").expect("TrimPath.start");
        let trim_mode = property_key_for_name("TrimPath", "modeValue").expect("TrimPath.modeValue");

        let mut path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, trim_start, 0.25));
        assert!(instance.path_epoch() > path_epoch);

        path_epoch = instance.path_epoch();
        assert!(instance.set_uint_property(0, trim_mode, 2));
        assert!(instance.path_epoch() > path_epoch);

        let mut dash_path = synthetic_component(0, 0);
        dash_path.type_name = "DashPath";
        let mut instance = synthetic_instance(vec![dash_path], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "DashPath", Vec::new()),
        )]);
        let offset_is_percentage = property_key_for_name("DashPath", "offsetIsPercentage")
            .expect("DashPath.offsetIsPercentage");

        path_epoch = instance.path_epoch();
        assert!(instance.set_bool_property(0, offset_is_percentage, true));
        assert!(instance.path_epoch() > path_epoch);

        let mut dash = synthetic_component(0, 0);
        dash.type_name = "Dash";
        let mut instance = synthetic_instance(vec![dash], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Dash", Vec::new()),
        )]);
        let length = property_key_for_name("Dash", "length").expect("Dash.length");

        path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, length, 4.0));
        assert!(instance.path_epoch() > path_epoch);

        let mut feather = synthetic_component(0, 0);
        feather.type_name = "Feather";
        let mut instance = synthetic_instance(vec![feather], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Feather", Vec::new()),
        )]);
        let inner = property_key_for_name("Feather", "inner").expect("Feather.inner");
        let space_value =
            property_key_for_name("Feather", "spaceValue").expect("Feather.spaceValue");

        path_epoch = instance.path_epoch();
        assert!(instance.set_bool_property(0, inner, true));
        assert!(instance.path_epoch() > path_epoch);

        path_epoch = instance.path_epoch();
        assert!(instance.set_uint_property(0, space_value, 1));
        assert!(instance.path_epoch() > path_epoch);
    }

    #[test]
    fn layout_epoch_tracks_layout_dirt_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_layout_epoch = instance.layout_epoch();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.layout_epoch(), initial_layout_epoch);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert!(instance.layout_epoch() > initial_layout_epoch);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let layout_epoch = instance.layout_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert_eq!(instance.layout_epoch(), layout_epoch);
    }

    #[test]
    fn solid_color_changes_keep_prepared_topology_epoch_stable() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();
        let initial_path_epoch = instance.path_epoch();
        let initial_layout_epoch = instance.layout_epoch();
        let initial_paint_revision = instance.solid_color_paint_revision(0);

        assert!(instance.set_keyed_solid_color_property(0, color_key, false, 0xff00_ff00));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert_eq!(instance.layout_epoch(), initial_layout_epoch);
        assert!(instance.solid_color_paint_revision(0) > initial_paint_revision);

        let settled_cache_epoch = instance.cache_epoch();
        let settled_paint_revision = instance.solid_color_paint_revision(0);
        assert!(!instance.set_keyed_solid_color_property(0, color_key, false, 0xff00_ff00));
        assert_eq!(instance.cache_epoch(), settled_cache_epoch);
        assert_eq!(
            instance.solid_color_paint_revision(0),
            settled_paint_revision
        );
    }

    #[test]
    fn solid_color_visibility_changes_invalidate_prepared_topology() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_color_property(0, color_key, 0x00ff_ffff));

        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_input_proxy_value_changes() {
        let mut nested_number = synthetic_component(0, 0);
        nested_number.type_name = "NestedNumber";
        let mut instance = synthetic_instance(vec![nested_number], vec![0]);
        let nested_value =
            property_key_for_name("NestedNumber", "nestedValue").expect("NestedNumber.nestedValue");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        // C++ forwards the virtual setter only to a live SMINumber and leaves
        // the serialized parent field untouched when no child input resolves
        // (`src/animation/nested_number.cpp:24-48`).
        assert!(!instance.set_double_property(0, nested_value, 1.0));
        assert_eq!(instance.double_property(0, nested_value), Some(0.0));
        assert_eq!(instance.cache_epoch(), initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_artboard_animation_knobs() {
        let mut nested_artboard = synthetic_component(0, 0);
        nested_artboard.type_name = "NestedArtboard";
        let mut instance = synthetic_instance(vec![nested_artboard], vec![0]);
        let speed_key = property_key_for_name("NestedArtboard", "speed").expect("speed");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_double_property(0, speed_key, 2.0));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn nested_layout_bounds_cache_tracks_layout_epoch() {
        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboardLayout";
        let mut instance = synthetic_instance(vec![host], vec![0]);
        instance.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(synthetic_instance(
                    vec![synthetic_component(10, 0)],
                    vec![10],
                )),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 1.0,
                quantize: 0.0,
                cumulated_seconds: 0.0,
            },
        );
        instance.nested_artboard_locals.push(0);

        let first_frame = instance.runtime_nested_artboard_layout_bounds_frame();
        let first_bounds = first_frame.bounds.clone();
        assert_eq!(first_frame.key.layout_epoch, instance.layout_epoch());
        assert!(Arc::ptr_eq(&first_bounds, &first_frame.bounds));

        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        let after_paint = instance.runtime_nested_artboard_layout_bounds_frame();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_epoch,
            instance.layout_epoch()
        );
        assert!(Arc::ptr_eq(&first_bounds, &after_paint.bounds));

        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        let after_layout = instance.runtime_nested_artboard_layout_bounds_frame();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_epoch,
            instance.layout_epoch()
        );
        assert!(!Arc::ptr_eq(&first_bounds, &after_layout.bounds));
    }

    #[test]
    fn layout_epoch_tracks_cpp_layout_property_changes() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        let mut text_run = synthetic_component(1, 1);
        text_run.type_name = "TextValueRun";
        let mut solid = synthetic_component(2, 2);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![layout, text_run, solid], vec![0, 1, 2]);

        let fractional_width =
            property_key_for_name("LayoutComponent", "fractionalWidth").expect("fractional width");
        let text = property_key_for_name("TextValueRun", "text").expect("text run text");
        let color = property_key_for_name("SolidColor", "colorValue").expect("solid color");

        let mut layout_epoch = instance.layout_epoch();
        assert!(instance.set_double_property(0, fractional_width, 0.5));
        assert!(instance.layout_epoch() > layout_epoch);

        layout_epoch = instance.layout_epoch();
        assert!(instance.set_string_property(1, text, b"hello".to_vec()));
        assert!(instance.layout_epoch() > layout_epoch);

        layout_epoch = instance.layout_epoch();
        assert!(instance.set_color_property(2, color, 0xff00ff00));
        assert_eq!(instance.layout_epoch(), layout_epoch);
    }

    #[test]
    fn named_root_text_value_run_write_uses_first_local_match_and_ignores_nested_runs() {
        let text_key =
            property_key_for_name("TextValueRun", "text").expect("TextValueRun.text key");
        let mut first = synthetic_component(0, 0);
        first.type_name = "TextValueRun";
        let mut second = synthetic_component(1, 1);
        second.type_name = "TextValueRun";
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        instance.slots[0].name = Some("headline".to_owned());
        instance.slots[1].name = Some("headline".to_owned());
        // Resolution is explicitly local-id ordered even if an embedding's
        // slot enumeration is not already sorted that way.
        instance.slots.reverse();
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("first".to_owned()),
                        raw: b"first".to_vec(),
                    }),
                }],
            )),
            Some(synthetic_runtime_object(
                1,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("second".to_owned()),
                        raw: b"second".to_vec(),
                    }),
                }],
            )),
        ]);

        let mut nested_run = synthetic_component(0, 0);
        nested_run.type_name = "TextValueRun";
        let mut nested = synthetic_instance(vec![nested_run], vec![0]);
        nested.slots[0].name = Some("headline".to_owned());
        nested.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("nested".to_owned()),
                        raw: b"nested".to_vec(),
                    }),
                }],
            ))]);
        instance.nested_artboards.insert(
            9,
            RuntimeNestedArtboardInstance {
                child: Box::new(nested),
                ..synthetic_nested_artboard_instance(9)
            },
        );

        assert_eq!(
            instance.set_root_text_value_run("headline", b"updated".to_vec()),
            Some(true)
        );
        assert_eq!(
            instance.string_property(0, text_key),
            Some(b"updated".as_slice())
        );
        assert_eq!(
            instance.string_property(1, text_key),
            Some(b"second".as_slice())
        );
        assert_eq!(
            instance
                .nested_artboards
                .get(&9)
                .and_then(|nested| nested.child.string_property(0, text_key)),
            Some(b"nested".as_slice())
        );
        assert_eq!(
            instance.set_root_text_value_run("headline", b"updated".to_vec()),
            Some(false)
        );
        assert_eq!(
            instance.set_root_text_value_run("missing", b"ignored".to_vec()),
            None
        );
    }

    #[test]
    fn gradient_property_changes_mark_cpp_dirty_bits() {
        let mut gradient = synthetic_component(0, 0);
        gradient.type_name = "LinearGradient";
        let mut stop = synthetic_component(1, 1);
        stop.type_name = "GradientStop";
        let mut instance = synthetic_instance(vec![gradient, stop], vec![0, 1]);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");
        let start_x_key = property_key_for_name("LinearGradient", "startX").expect("startX key");
        let opacity_key = property_key_for_name("LinearGradient", "opacity").expect("opacity key");
        let stop_color_key =
            property_key_for_name("GradientStop", "colorValue").expect("stop color key");
        let stop_position_key =
            property_key_for_name("GradientStop", "position").expect("stop position key");
        let _ = instance.objects.set_uint_property(1, parent_key, 0);

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, start_x_key, 10.0));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM)
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, opacity_key, 0.5));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT)
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_color_property(1, stop_color_key, 0xff00_ff00));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(1, stop_position_key, 0.25));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );
    }

    #[test]
    fn follow_path_property_changes_dirty_the_constrained_parent_transform() {
        let parent = synthetic_component_for_type(0, "Node");
        let constraint = synthetic_component_for_type(1, "FollowPathConstraint");
        let mut instance = synthetic_instance(vec![parent, constraint], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        let distance_key = property_key_for_name("FollowPathConstraint", "distance")
            .expect("FollowPathConstraint.distance key");
        let orient_key = property_key_for_name("FollowPathConstraint", "orient")
            .expect("FollowPathConstraint.orient key");
        let strength_key =
            property_key_for_name("Constraint", "strength").expect("Constraint.strength key");

        fn assert_parent_transform_dirty(instance: &mut ArtboardInstance, changed: bool) {
            assert!(changed);
            assert!(
                instance
                    .component(0)
                    .unwrap()
                    .dirt
                    .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
            );
            instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        }

        let changed = instance.set_double_property(1, distance_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_bool_property(1, orient_key, false);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_double_property(1, strength_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
    }

    #[test]
    fn list_follow_path_inherited_and_leaf_callbacks_dirty_the_constrained_list() {
        let list = synthetic_component_for_type(0, "ArtboardComponentList");
        let constraint = synthetic_component_for_type(1, "ListFollowPathConstraint");
        let mut instance = synthetic_instance(vec![list, constraint], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        let distance_key = property_key_for_name("FollowPathConstraint", "distance").unwrap();
        let orient_key = property_key_for_name("FollowPathConstraint", "orient").unwrap();
        let distance_end_key =
            property_key_for_name("ListFollowPathConstraint", "distanceEnd").unwrap();
        let distance_offset_key =
            property_key_for_name("ListFollowPathConstraint", "distanceOffset").unwrap();
        let offset_key = property_key_for_name("FollowPathConstraint", "offset").unwrap();

        for change in [
            (distance_key, 0.25),
            (distance_end_key, 0.75),
            (distance_offset_key, 0.1),
        ] {
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
            assert!(instance.set_double_property(1, change.0, change.1));
            assert!(
                instance
                    .component(0)
                    .unwrap()
                    .dirt
                    .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
            );
        }

        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_bool_property(1, orient_key, false));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );

        // Generated FollowPath.offset has an intentional no-op changed
        // callback (`follow_path_constraint_base.hpp:107-110`).
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_bool_property(1, offset_key, true));
        assert!(instance.component(0).unwrap().dirt.is_empty());
    }

    #[test]
    fn ik_strength_and_invert_dirty_the_retained_chain_but_parent_count_is_noop() {
        let root = synthetic_component_for_type(0, "RootBone");
        let middle = synthetic_component_for_type(1, "Bone");
        let tip = synthetic_component_for_type(2, "Bone");
        let constraint = synthetic_component_for_type(3, "IKConstraint");
        let mut instance =
            synthetic_instance(vec![root, middle, tip, constraint], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 2);
        let links = [0usize, 1, 2]
            .into_iter()
            .enumerate()
            .map(|(index, local)| RuntimeIkChainLink {
                index,
                bone: instance.component_handle(local).unwrap(),
                angle: 0.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::IDENTITY,
            })
            .collect();
        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .ik
            .as_mut()
            .unwrap()
            .chain = links;

        let clear_chain = |instance: &mut ArtboardInstance| {
            for local in 0..3 {
                instance.component_mut(local).unwrap().dirt = ComponentDirt::NONE;
            }
        };
        let assert_chain_dirty = |instance: &ArtboardInstance| {
            for local in 0..3 {
                assert!(
                    instance
                        .component(local)
                        .unwrap()
                        .dirt
                        .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
                );
            }
        };

        let strength_key = property_key_for_name("Constraint", "strength").unwrap();
        clear_chain(&mut instance);
        assert!(instance.set_double_property(3, strength_key, 0.5));
        assert_chain_dirty(&instance);

        clear_chain(&mut instance);
        assert!(instance.set_bool_property(
            3,
            crate::constraints::IK_INVERT_DIRECTION_PROPERTY_KEY,
            true,
        ));
        assert_chain_dirty(&instance);

        clear_chain(&mut instance);
        assert!(instance.set_uint_property(
            3,
            crate::constraints::IK_PARENT_BONE_COUNT_PROPERTY_KEY,
            2,
        ));
        for local in 0..3 {
            assert!(instance.component(local).unwrap().dirt.is_empty());
        }
        assert_eq!(
            instance
                .component(3)
                .unwrap()
                .concrete
                .ik
                .as_ref()
                .unwrap()
                .chain
                .len(),
            3,
            "parentBoneCountChanged is a no-op; lifecycle owns chain rebuild"
        );
    }

    #[test]
    fn a5_owned_state_resets_on_occurrence_clone() {
        let follow = synthetic_component_for_type(0, "FollowPathConstraint");
        let list = synthetic_component_for_type(1, "ArtboardComponentList");
        let ik = synthetic_component_for_type(2, "IKConstraint");
        let mut instance = synthetic_instance(vec![follow, list, ik], vec![0, 1, 2]);
        let follow_handle = instance.component_handle(0).unwrap();
        let ik_handle = instance.component_handle(2).unwrap();
        {
            let follow = instance
                .component_mut(0)
                .unwrap()
                .concrete
                .follow_path
                .as_mut()
                .unwrap();
            follow.raw_path.move_to(1.0, 2.0);
            follow.path_measure = crate::draw::RuntimePathMeasure::from_raw_path(&follow.raw_path);
        }
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .constrainable_list
            .as_mut()
            .unwrap()
            .constraints
            .push(follow_handle);
        instance
            .component_mut(2)
            .unwrap()
            .concrete
            .ik
            .as_mut()
            .unwrap()
            .chain
            .push(RuntimeIkChainLink {
                index: 0,
                bone: ik_handle,
                angle: 1.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::IDENTITY,
            });

        let cloned = instance.clone();
        assert!(
            cloned
                .component(0)
                .unwrap()
                .concrete
                .follow_path
                .as_ref()
                .unwrap()
                .raw_path
                .verbs()
                .is_empty()
        );
        assert!(
            cloned
                .component(1)
                .unwrap()
                .concrete
                .constrainable_list
                .as_ref()
                .unwrap()
                .constraints
                .is_empty()
        );
        assert!(
            cloned
                .component(2)
                .unwrap()
                .concrete
                .ik
                .as_ref()
                .unwrap()
                .chain
                .is_empty()
        );
    }

    #[test]
    fn follow_path_measure_rebuilds_on_owner_update_and_preserves_when_shape_is_empty() {
        let root = synthetic_component_for_type(0, "Node");
        let shape = synthetic_component_for_type(1, "Shape");
        let path = synthetic_component_for_type(2, "PointsPath");
        let constraint = synthetic_component_for_type(3, "FollowPathConstraint");
        let empty_shape = synthetic_component_for_type(4, "Shape");
        let mut instance = synthetic_instance(
            vec![root, shape, path, constraint, empty_shape],
            vec![2, 3, 0],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 0);
        synthetic_link_parent(&mut instance, 4, 0);
        let constraint_handle = instance.component_handle(3).unwrap();
        let shape_handle = instance.component_handle(1).unwrap();
        let path_handle = instance.component_handle(2).unwrap();
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .shape
            .as_mut()
            .unwrap()
            .paths
            .push(path_handle);
        {
            let path = instance
                .component_mut(2)
                .unwrap()
                .concrete
                .path
                .as_mut()
                .unwrap();
            path.shape = Some(shape_handle);
        }
        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .constraint
            .as_mut()
            .unwrap()
            .target = Some(shape_handle);
        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 0.0, y: 0.0 },
                crate::draw::RuntimePathCommand::Line { x: 10.0, y: 0.0 },
            ],
            false,
        );

        assert!(crate::constraints::update_follow_path_constraint(
            &mut instance,
            constraint_handle
        ));
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 1);
        assert_eq!(follow.path_measure.length(), 10.0);
        let retained_verb_storage = follow.raw_path.verbs().as_ptr();
        let retained_point_storage = follow.raw_path.points().as_ptr();

        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 2.0, y: 0.0 },
                crate::draw::RuntimePathCommand::Line { x: 14.0, y: 0.0 },
            ],
            false,
        );
        assert!(crate::constraints::update_follow_path_constraint(
            &mut instance,
            constraint_handle
        ));
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 2);
        assert_eq!(follow.path_measure.length(), 12.0);
        assert_eq!(
            follow.raw_path.verbs().as_ptr(),
            retained_verb_storage,
            "FollowPath reuses m_rawPath verb storage across equal-size rewinds (`follow_path_constraint.cpp:137-145`)"
        );
        assert_eq!(
            follow.raw_path.points().as_ptr(),
            retained_point_storage,
            "FollowPath reuses m_rawPath point storage across equal-size rewinds (`follow_path_constraint.cpp:137-145`)"
        );

        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .constraint
            .as_mut()
            .unwrap()
            .target = Some(instance.component_handle(4).unwrap());
        assert!(!crate::constraints::update_follow_path_constraint(
            &mut instance,
            constraint_handle
        ));
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 2);
        assert_eq!(follow.path_measure.length(), 12.0);

        // A clean Artboard pass never reaches the FollowPath update site.
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        for local in 0..5 {
            instance.component_mut(local).unwrap().dirt = ComponentDirt::NONE;
        }
        assert!(!instance.update_components().did_update);
        assert_eq!(
            instance
                .component(3)
                .unwrap()
                .concrete
                .follow_path
                .as_ref()
                .unwrap()
                .measure_rebuilds,
            2
        );
    }

    #[test]
    fn path_deferral_consumes_occurrence_owned_follow_flags_and_rearms_on_dirty() {
        let bytes = synthetic_riv(11_905, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 0)]);
            push_synthetic_object_with_properties(bytes, "Rectangle", |bytes| {
                push_synthetic_uint_property(bytes, "Rectangle", "parentId", 1);
                push_synthetic_f32_property(bytes, "Rectangle", "width", 20.0);
                push_synthetic_f32_property(bytes, "Rectangle", "height", 10.0);
            });
        });
        let mut instance = instance_from_riv(&bytes);
        assert!(instance.update_components().did_update);
        let path = instance.component_handle(2).expect("Rectangle Path handle");
        let shape = instance.component_handle(1).expect("Shape handle");
        let initial_mutation = instance
            .runtime_shapes
            .retained_path_mutation_id_for_test(2)
            .expect("visible first update builds the Path");

        instance.component_at_mut(shape).transform.render_opacity = 0.0;
        assert!(instance.add_component_dirt(path, ComponentDirt::PATH, false));
        assert!(instance.update_components().did_update);
        assert_eq!(
            instance
                .runtime_shapes
                .retained_path_mutation_id_for_test(2),
            Some(initial_mutation),
            "an invisible ordinary Path retains its prior RawPath while deferred (`path.cpp:111-125,350-372`)"
        );
        assert!(
            instance
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );

        instance.component_at_mut(shape).transform.render_opacity = 1.0;
        assert!(instance.add_component_dirt(path, ComponentDirt::WORLD_TRANSFORM, false));
        assert!(
            instance
                .component_at(path)
                .dirt
                .contains(ComponentDirt::PATH),
            "the next Path::onDirty re-adds deferred Path dirt (`path.cpp:336-347`)"
        );
        assert!(instance.update_components().did_update);
        let restored_mutation = instance
            .runtime_shapes
            .retained_path_mutation_id_for_test(2)
            .expect("restored visible Path rebuilds");
        assert_ne!(restored_mutation, initial_mutation);
        assert!(
            !instance
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );

        instance.component_at_mut(shape).transform.render_opacity = 0.0;
        instance
            .component_at(shape)
            .concrete
            .shape
            .as_ref()
            .unwrap()
            .add_flags(crate::components::RuntimeShapeState::FOLLOW_PATH);
        assert!(instance.add_component_dirt(path, ComponentDirt::PATH, false));
        assert!(instance.update_components().did_update);
        assert_ne!(
            instance
                .runtime_shapes
                .retained_path_mutation_id_for_test(2),
            Some(restored_mutation),
            "Shape followPath forbids deferral even at zero opacity (`follow_path_constraint.cpp:149-165`; `path.cpp:111-125`)"
        );

        let cloned = instance.clone();
        assert!(
            !cloned
                .component_at(shape)
                .concrete
                .shape
                .as_ref()
                .unwrap()
                .is_flagged(crate::components::RuntimeShapeState::FOLLOW_PATH),
            "generated clone resets runtime flags before clean-phase producers rebuild them"
        );
        assert!(
            !cloned
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );
    }

    #[test]
    fn artboard_clip_property_updates_draw_cache() {
        let mut artboard = synthetic_component(0, 0);
        artboard.type_name = "Artboard";
        let mut instance = synthetic_instance(vec![artboard], vec![0]);
        let clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip key");

        assert!(instance.clip);
        assert!(instance.set_bool_property(0, clip_key, false));
        assert!(!instance.clip);
        assert!(instance.set_bool_property(0, clip_key, true));
        assert!(instance.clip);
    }

    #[test]
    fn update_components_skips_collapsed_components_without_clearing_dirt() {
        let mut first = synthetic_component(0, 0);
        first.dirt = ComponentDirt::PATH;
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH | ComponentDirt::COLLAPSED;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);

        let report = instance.update_components();

        assert!(report.did_update);
        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.component(0).unwrap().dirt, ComponentDirt::NONE);
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.component(1).unwrap().is_collapsed());
    }

    #[test]
    fn update_components_restarts_when_update_dirties_earlier_graph_order() {
        let first = synthetic_component(0, 0);
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let mut dirtied_earlier = false;

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            if local_id == 1 && !dirtied_earlier {
                dirtied_earlier = true;
                instance.add_dirt(0, ComponentDirt::PATH, false);
            }
        });

        assert_eq!(report.steps, 2);
        assert_eq!(report.updated_locals, vec![1, 0]);
        assert!(!report.max_steps_reached);
    }

    #[test]
    fn update_components_surfaces_cpp_max_pass_guard() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            instance.add_dirt(local_id, ComponentDirt::PATH, false);
        });

        assert_eq!(report.steps, 100);
        assert_eq!(report.updated_locals.len(), 100);
        assert!(report.max_steps_reached);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
    }

    fn synthetic_runtime_object(
        id: u32,
        type_name: &'static str,
        properties: Vec<RuntimeProperty>,
    ) -> RuntimeObject {
        let definition = definition_by_name(type_name).expect("synthetic runtime object type");
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties,
            skipped_properties: Vec::new(),
        }
    }

    #[test]
    fn instance_object_arena_uses_generated_core_registry_setter_families() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Node", Vec::new())),
            Some(synthetic_runtime_object(1, "FileAssetContents", Vec::new())),
        ]);

        assert!(arena.set_double_property(0, node_x_key, 12.5));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_uint_property(0, node_x_key, 12));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_string_property(1, bytes_key, vec![1, 2, 3]));
        assert_eq!(arena.string_property(1, bytes_key), None);
    }

    #[test]
    fn instance_object_arena_keeps_mutable_properties_in_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let source = synthetic_runtime_object(0, "Node", Vec::new());
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![Some(source.clone())]);

        assert!(arena.set_double_property(0, node_x_key, 42.0));

        assert!(source.properties.is_empty());
        assert_eq!(arena.double_property(0, node_x_key), Some(42.0));
    }

    #[test]
    fn instance_object_arena_reads_generated_defaults_and_imported_fields() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let artboard_clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "Node",
                vec![RuntimeProperty {
                    key: node_x_key,
                    name: "x",
                    owner: "Node",
                    value: FieldValue::Double(7.5),
                }],
            )),
            Some(synthetic_runtime_object(1, "Artboard", Vec::new())),
            Some(synthetic_runtime_object(
                2,
                "FileAssetContents",
                vec![RuntimeProperty {
                    key: bytes_key,
                    name: "bytes",
                    owner: "FileAssetContents",
                    value: FieldValue::Bytes(BytesValue::new(vec![1, 2, 3])),
                }],
            )),
        ]);

        assert_eq!(arena.double_property(0, node_x_key), Some(7.5));
        assert_eq!(arena.bool_property(1, artboard_clip_key), Some(true));
        assert_eq!(arena.string_property(2, bytes_key), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn artboard_typed_property_reads_surface_defaults_and_reject_wrong_value_kinds() {
        let opacity_key = property_key_for_name("Shape", "opacity").expect("Shape.opacity");
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Shape", Vec::new())),
            Some(synthetic_runtime_object(1, "SolidColor", Vec::new())),
        ]);

        assert_eq!(instance.double_property(0, opacity_key), Some(1.0));
        assert_eq!(instance.color_property(1, color_key), Some(0xff74_7474));
        assert_eq!(instance.color_property(0, opacity_key), None);
        assert_eq!(instance.double_property(1, color_key), None);
    }

    #[test]
    fn update_transform_reads_generated_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let node_scale_x_key = property_key_for_name("Node", "scaleX").expect("Node.scaleX key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::TRANSFORM;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        assert!(instance.objects.set_double_property_by_name(0, "x", 8.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(0, "scaleX", 2.5)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.double_property(0, node_x_key), Some(8.0));
        assert_eq!(instance.double_property(0, node_scale_x_key), Some(2.5));
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([2.5, 0.0, -0.0, 1.0, 8.0, 0.0])
        );
    }

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        assert!(instance.objects.set_double_property_by_name(1, "x", 2.0));
        assert!(instance.objects.set_double_property_by_name(1, "y", 3.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleX", 4.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleY", 5.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "opacity", 0.25)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![1]);
        let child = instance.component(1).unwrap();
        assert_eq!(
            child.transform.local_transform,
            Mat2D([4.0, 0.0, -0.0, 5.0, 2.0, 3.0])
        );
        assert_eq!(
            child.transform.world_transform,
            child.transform.local_transform
        );
        assert_eq!(child.transform.render_opacity, 0.125);
    }

    #[test]
    fn duplicate_transform_dirt_does_not_redirty_world_dependents() {
        // `TransformComponent::markTransformDirty` returns immediately when
        // Transform is already present; recursive WorldTransform dirt is
        // gated by that first addition (`src/transform_component.cpp:54-61`).
        let mut source = synthetic_component(0, 0);
        source.dirt = ComponentDirt::TRANSFORM;
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.clear_component_dirt(1);

        let source = instance.component_handle(0).expect("source");
        assert!(!instance.mark_transform_dirty_handle(source));
        assert!(
            !instance
                .component(1)
                .expect("dependent")
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
    }

    #[test]
    fn collapse_dirties_constrained_dependents_only_for_transform_owners() {
        // Only TransformComponent::collapse has the constrained-dependent
        // tail, after ContainerComponent child propagation
        // (`src/transform_component.cpp:18-44`). Component/Skin collapse must
        // not acquire that virtual override accidentally.
        for (source_type, should_dirty) in [("Node", true), ("Skin", false)] {
            let source = synthetic_component_for_type(0, source_type);
            let dependent = synthetic_component_for_type(1, "Node");
            let constraint = synthetic_component_for_type(2, "RotationConstraint");
            let mut instance =
                synthetic_instance(vec![source, dependent, constraint], vec![0, 1, 2]);
            let source = instance.component_handle(0).unwrap();
            let dependent = instance.component_handle(1).unwrap();
            let constraint = instance.component_handle(2).unwrap();
            synthetic_add_dependent(&mut instance, 0, 1);
            assert!(instance.objects.add_constraint(dependent, constraint));
            instance.clear_component_dirt(0);
            instance.clear_component_dirt(1);
            instance.clear_component_dirt(2);

            assert!(instance.collapse_component_tree(0, true));
            assert_eq!(
                instance
                    .component_at(dependent)
                    .dirt
                    .contains(ComponentDirt::WORLD_TRANSFORM),
                should_dirty,
                "{source_type} collapse dispatch"
            );
            assert!(instance.component_at(source).is_collapsed());
        }
    }

    #[test]
    fn bone_length_changed_dirties_only_retained_child_bones() {
        // Bone::lengthChanged walks m_ChildBones and calls
        // markTransformDirty on each (`src/bones/bone.cpp:20-26`).
        let parent = synthetic_component_for_type(0, "RootBone");
        let child = synthetic_component_for_type(1, "Bone");
        let unrelated = synthetic_component_for_type(2, "Bone");
        let mut instance = synthetic_instance(vec![parent, child, unrelated], vec![0, 1, 2]);
        let parent = instance.component_handle(0).unwrap();
        let child = instance.component_handle(1).unwrap();
        instance
            .component_at_mut(parent)
            .concrete
            .bone
            .as_mut()
            .unwrap()
            .child_bones
            .push(child);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.clear_component_dirt(2);
        let length = property_key_for_name("Bone", "length").expect("Bone.length");

        assert!(instance.set_double_property(0, length, 42.0));
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert_eq!(instance.component(2).unwrap().dirt, ComponentDirt::NONE);
    }

    #[test]
    fn world_transform_parent_uses_base_child_opacity() {
        // WorldTransformComponent::childOpacity returns authored opacity,
        // while TransformComponent overrides it with settled render opacity
        // (`src/world_transform_component.cpp:8`;
        // `include/rive/transform_component.hpp:40-43`).
        // The pinned schema has no concrete WorldTransformComponent-only
        // subtype, so exercise the abstract base dispatch directly.
        let mut parent = synthetic_component(0, 0);
        parent.type_name = "WorldTransformComponent";
        parent.capabilities = RuntimeComponentCapabilities {
            world_transform: true,
            transform: false,
        };
        parent.transform.render_opacity = 0.125;
        assert_eq!(parent.child_opacity(0.5), 0.5);
    }

    #[test]
    fn node_computed_local_is_lazy_and_singular_parent_falls_back_to_identity() {
        // Node invalidates its distinct computed-local cache before the base
        // world update. Query derives inverse(parent world) * settled world;
        // missing/singular parents yield identity (`src/node.cpp:26-45`).
        let parent = synthetic_component(0, 0);
        let child = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![parent, child], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 3.0]);
        {
            let child = instance.component_mut(1).unwrap();
            child.transform.world_transform = Mat2D([1.0, 0.0, 0.0, 1.0, 25.0, 8.0]);
            child
                .concrete
                .node
                .as_ref()
                .expect("Node state")
                .mark_computed_local_dirty();
        }
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D([1.0, 0.0, 0.0, 1.0, 15.0, 5.0]))
        );

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([0.0, 0.0, 0.0, 0.0, 10.0, 3.0]);
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .expect("Node state")
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );
    }

    #[test]
    fn skin_on_dirty_targets_exact_skinnable_dirt_family() {
        // Skin::onDirty calls one retained Skinnable. PointsPath translates
        // that to Path; Mesh translates it to Vertices
        // (`src/bones/skin.cpp:88-94`,
        // `src/shapes/points_path.cpp:43-52`,
        // `src/shapes/mesh.cpp:84-85`).
        for (type_name, expected, rejected) in [
            ("PointsPath", ComponentDirt::PATH, ComponentDirt::VERTICES),
            ("Mesh", ComponentDirt::VERTICES, ComponentDirt::PATH),
        ] {
            let skinnable = synthetic_component_for_type(0, type_name);
            let skin = synthetic_component_for_type(1, "Skin");
            let mut instance = synthetic_instance(vec![skinnable, skin], vec![1, 0]);
            synthetic_link_parent(&mut instance, 1, 0);
            let skinnable = instance.component_handle(0).unwrap();
            let skin = instance.component_handle(1).unwrap();
            instance
                .component_at_mut(skinnable)
                .concrete
                .skinnable
                .as_mut()
                .unwrap()
                .skin = Some(skin);
            {
                let state = instance
                    .component_at_mut(skin)
                    .concrete
                    .skin
                    .as_mut()
                    .unwrap();
                state.skinnable = Some(skinnable);
            }
            instance.clear_component_dirt(0);
            instance.clear_component_dirt(1);

            assert!(instance.add_component_dirt(skin, ComponentDirt::PAINT, false));
            let dirt = instance.component_at(skinnable).dirt;
            assert!(dirt.contains(expected));
            assert!(!dirt.contains(rejected));
        }
    }

    #[test]
    fn skin_buffer_rebuilds_only_when_skin_owner_updates() {
        // Skin owns `(tendons + 1)` matrices and rebuilds them from the
        // retained Tendon/Bone links in update, not during draw
        // (`src/bones/skin.cpp:38-77`).
        let mut bone = synthetic_component_for_type(0, "RootBone");
        bone.transform.world_transform = Mat2D([1.0, 0.0, 0.0, 1.0, 7.0, 9.0]);
        let tendon = synthetic_component_for_type(1, "Tendon");
        let mut skin = synthetic_component_for_type(2, "Skin");
        skin.dirt = ComponentDirt::SKIN;
        let mut instance = synthetic_instance(vec![bone, tendon, skin], vec![0, 1, 2]);
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        {
            let state = instance
                .component_at_mut(tendon)
                .concrete
                .tendon
                .as_mut()
                .unwrap();
            state.bone = Some(bone);
            state.inverse_bind = Mat2D([1.0, 0.0, 0.0, 1.0, 2.0, 3.0]);
        }
        {
            let state = instance
                .component_at_mut(skin)
                .concrete
                .skin
                .as_mut()
                .unwrap();
            state.tendons.push(tendon);
            state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }

        instance.update_components();
        let state = instance.component_at(skin).concrete.skin.as_ref().unwrap();
        assert_eq!(
            state.bone_transforms,
            vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 12.0])]
        );
        assert_eq!(state.buffer_rebuilds, 1);

        instance.update_components();
        assert_eq!(
            instance
                .component_at(skin)
                .concrete
                .skin
                .as_ref()
                .unwrap()
                .buffer_rebuilds,
            1
        );
    }

    #[test]
    fn weight_deformation_reads_live_packed_occurrence_fields() {
        // Weight decodes four packed bytes from the live generated fields and
        // writes its retained translation without normalization beyond
        // weight/255 (`src/bones/weight.cpp:24-56`;
        // `src/shapes/vertex.cpp:17-23`).
        let path = synthetic_component_for_type(0, "PointsPath");
        let skin = synthetic_component_for_type(1, "Skin");
        let vertex = synthetic_component_for_type(2, "StraightVertex");
        let weight = synthetic_component_for_type(3, "Weight");
        let mut instance = synthetic_instance(vec![path, skin, vertex, weight], vec![1, 0, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        synthetic_link_parent(&mut instance, 3, 2);
        let path = instance.component_handle(0).unwrap();
        let skin = instance.component_handle(1).unwrap();
        let vertex = instance.component_handle(2).unwrap();
        let weight = instance.component_handle(3).unwrap();
        instance
            .component_at_mut(path)
            .concrete
            .skinnable
            .as_mut()
            .unwrap()
            .skin = Some(skin);
        instance
            .component_at_mut(skin)
            .concrete
            .skin
            .as_mut()
            .unwrap()
            .bone_transforms = vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])];
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        let indices = property_key_for_name("Weight", "indices").unwrap();
        let values = property_key_for_name("Weight", "values").unwrap();
        assert_eq!(instance.objects.uint_property(3, indices), Some(1));
        assert!(instance.objects.set_uint_property(3, values, 0));
        assert!(instance.deform_runtime_vertex_weight(2, (2.0, 3.0), None));
        assert_eq!(
            instance.runtime_vertex_weight_state(2).unwrap().translation,
            (0.0, 0.0)
        );

        assert!(instance.objects.set_uint_property(3, values, 255));
        assert!(instance.deform_runtime_vertex_weight(2, (2.0, 3.0), None));
        assert_eq!(
            instance.runtime_vertex_weight_state(2).unwrap().translation,
            (12.0, 23.0)
        );
    }

    #[test]
    fn weight_deformation_matches_clang_contracted_accumulation() {
        // The two active packed influences are the minimal tape.riv
        // counterexample for the one-ulp difference between separate
        // multiply/add and the C++ geometry pipeline's default contraction.
        // Preserve Weight::deform's source loop and clang rounding exactly
        // (`src/bones/weight.cpp:24-55`; `docs/PORTING.md` §4.2).
        let f = f32::from_bits;
        let bones = [
            Mat2D::IDENTITY,
            Mat2D([
                f(1_057_841_340),
                f(3_194_226_667),
                f(1_049_771_165),
                f(1_057_885_778),
                f(1_107_583_000),
                f(1_124_670_380),
            ]),
            Mat2D([
                f(1_057_841_342),
                f(3_194_226_637),
                f(1_049_771_148),
                f(1_057_885_779),
                f(1_106_043_744),
                f(1_125_092_309),
            ]),
        ];
        let point = (f(3_265_177_136), f(1_091_681_984));
        let world = Mat2D([1.0, 0.0, 0.0, 1.0, f(1_135_168_757), f(1_135_822_912)]);

        let deformed =
            deform_point_from_skin(point, 258, 32_385, world, &bones).expect("valid bone indices");
        assert_eq!(
            (deformed.0.to_bits(), deformed.1.to_bits()),
            (1_133_233_234, 1_133_466_484)
        );
    }

    #[test]
    fn cubic_weight_retains_independent_live_in_and_out_translations() {
        // CubicWeight owns independent in/out translations and decodes each
        // packed generated field through Weight::deform
        // (`include/rive/bones/cubic_weight.hpp:9-15`;
        // `src/shapes/cubic_vertex.cpp:16-24`).
        let path = synthetic_component_for_type(0, "PointsPath");
        let skin = synthetic_component_for_type(1, "Skin");
        let vertex = synthetic_component_for_type(2, "CubicDetachedVertex");
        let weight = synthetic_component_for_type(3, "CubicWeight");
        let mut instance = synthetic_instance(vec![path, skin, vertex, weight], vec![1, 0, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        synthetic_link_parent(&mut instance, 3, 2);
        let path = instance.component_handle(0).unwrap();
        let skin = instance.component_handle(1).unwrap();
        let vertex = instance.component_handle(2).unwrap();
        let weight = instance.component_handle(3).unwrap();
        instance
            .component_at_mut(path)
            .concrete
            .skinnable
            .as_mut()
            .unwrap()
            .skin = Some(skin);
        instance
            .component_at_mut(skin)
            .concrete
            .skin
            .as_mut()
            .unwrap()
            .bone_transforms = vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])];
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        for (owner, field) in [
            ("Weight", "values"),
            ("CubicWeight", "inValues"),
            ("CubicWeight", "outValues"),
        ] {
            let key = property_key_for_name(owner, field).unwrap();
            if instance.objects.uint_property(3, key) != Some(255) {
                assert!(instance.objects.set_uint_property(3, key, 255));
            }
        }
        for (owner, field) in [
            ("Weight", "indices"),
            ("CubicWeight", "inIndices"),
            ("CubicWeight", "outIndices"),
        ] {
            let key = property_key_for_name(owner, field).unwrap();
            if instance.objects.uint_property(3, key) != Some(1) {
                assert!(instance.objects.set_uint_property(3, key, 1));
            }
        }

        assert!(instance.deform_runtime_vertex_weight(
            2,
            (2.0, 3.0),
            Some(((0.0, 1.0), (4.0, 5.0))),
        ));
        let state = instance.runtime_vertex_weight_state(2).unwrap();
        assert_eq!(state.translation, (12.0, 23.0));
        assert_eq!(state.in_translation, (10.0, 21.0));
        assert_eq!(state.out_translation, (14.0, 25.0));
    }

    #[test]
    fn clone_resets_every_bone_runtime_relation_and_buffer() {
        // Generated fields clone, but Bone/Skin/Tendon/Skinnable/Weight
        // runtime pointers, buffers, and lazy outputs are defaulted and later
        // rebuilt against clone-owned objects (`artboard.hpp:548-601`;
        // `src/bones/{bone,skin,tendon,weight}.cpp`).
        let bone = synthetic_component_for_type(0, "RootBone");
        let tendon = synthetic_component_for_type(1, "Tendon");
        let skin = synthetic_component_for_type(2, "Skin");
        let vertex = synthetic_component_for_type(3, "CubicDetachedVertex");
        let weight = synthetic_component_for_type(4, "CubicWeight");
        let mut instance = synthetic_instance(
            vec![bone, tendon, skin, vertex, weight],
            vec![0, 1, 2, 3, 4],
        );
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        let vertex = instance.component_handle(3).unwrap();
        let weight = instance.component_handle(4).unwrap();
        instance
            .component_at_mut(bone)
            .concrete
            .bone
            .as_mut()
            .unwrap()
            .child_bones
            .push(bone);
        instance
            .component_at_mut(tendon)
            .concrete
            .tendon
            .as_mut()
            .unwrap()
            .bone = Some(bone);
        {
            let state = instance
                .component_at_mut(skin)
                .concrete
                .skin
                .as_mut()
                .unwrap();
            state.tendons.push(tendon);
            state.skinnable = Some(vertex);
            state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        instance
            .component_at_mut(weight)
            .concrete
            .weight
            .as_mut()
            .unwrap()
            .translation = (7.0, 9.0);

        let cloned = instance.clone();
        assert!(
            cloned
                .component(0)
                .unwrap()
                .concrete
                .bone
                .as_ref()
                .unwrap()
                .child_bones
                .is_empty()
        );
        assert!(
            cloned
                .component(1)
                .unwrap()
                .concrete
                .tendon
                .as_ref()
                .unwrap()
                .bone
                .is_none()
        );
        let cloned_skin = cloned.component(2).unwrap().concrete.skin.as_ref().unwrap();
        assert!(cloned_skin.tendons.is_empty());
        assert!(cloned_skin.skinnable.is_none());
        assert!(cloned_skin.bone_transforms.is_empty());
        assert!(
            cloned
                .component(3)
                .unwrap()
                .concrete
                .vertex
                .as_ref()
                .unwrap()
                .weight
                .is_none()
        );
        assert_eq!(
            cloned
                .component(4)
                .unwrap()
                .concrete
                .weight
                .as_ref()
                .unwrap()
                .translation,
            (0.0, 0.0)
        );
    }

    #[test]
    fn parent_traversal_crosses_mount_once_and_resets_metadata() {
        // ParentTraversal returns current, then advances; the source Artboard
        // root is the call that records a host crossing
        // (`src/parent_traversal.cpp:14-60`).
        use crate::parent_traversal::{ParentTraversal, ParentTraversalFrame};

        let parent_root = synthetic_component(0, 0);
        let parent_ancestor = synthetic_component(1, 1);
        let parent_host = synthetic_component(2, 2);
        let mut parent = synthetic_instance(
            vec![parent_root, parent_ancestor, parent_host],
            vec![0, 1, 2],
        );
        synthetic_link_parent(&mut parent, 1, 0);
        synthetic_link_parent(&mut parent, 2, 1);

        let child_root = synthetic_component(0, 0);
        let child_parent = synthetic_component(1, 1);
        let child_start = synthetic_component(2, 2);
        let mut child =
            synthetic_instance(vec![child_root, child_parent, child_start], vec![0, 1, 2]);
        synthetic_link_parent(&mut child, 1, 0);
        synthetic_link_parent(&mut child, 2, 1);

        let host = parent.component_handle(2).unwrap();
        let start = child.component_handle(2).unwrap();
        let frames = [
            ParentTraversalFrame {
                artboard: &parent,
                host_component_in_parent: None,
            },
            ParentTraversalFrame {
                artboard: &child,
                host_component_in_parent: Some(host),
            },
        ];
        let mut traversal = ParentTraversal::new(&frames, start);
        assert_eq!(
            child.component_local_id(traversal.next().unwrap().component),
            Some(1)
        );
        assert!(!traversal.did_cross_boundary());
        assert_eq!(
            child.component_local_id(traversal.next().unwrap().component),
            Some(0)
        );
        assert!(traversal.did_cross_boundary());
        assert_eq!(traversal.crossing_host(), Some(host));
        assert!(std::ptr::eq(traversal.source_artboard().unwrap(), &child));
        assert!(std::ptr::eq(traversal.current_artboard().unwrap(), &parent));
        assert_eq!(
            parent.component_local_id(traversal.next().unwrap().component),
            Some(1)
        );
        assert!(!traversal.did_cross_boundary());
        assert_eq!(traversal.crossing_host(), None);
        assert!(traversal.source_artboard().is_none());
    }

    #[test]
    fn transform_property_mutation_marks_instance_dirty() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.did_change.set(false);

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(12.0)
        );
        assert_eq!(instance.double_property(0, node_x_key), Some(12.0));
        assert!(
            component
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.did_change());

        assert!(!instance.set_transform_property(0, TransformProperty::X, 12.0));
    }

    #[test]
    fn transform_property_mutation_rejects_missing_dense_local() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut instance = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);

        assert!(!instance.set_transform_property(1, TransformProperty::X, 12.0));
        assert!(!instance.set_transform_property_with_key(
            1,
            TransformProperty::X,
            node_x_key,
            12.0,
        ));
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(0.0)
        );
    }

    #[test]
    fn transform_property_mutation_writes_generated_storage_by_concrete_type() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut vertex = synthetic_component(0, 0);
        vertex.type_name = "StraightVertex";
        let mut instance = synthetic_instance(vec![vertex], vec![0]);

        assert!(instance.set_transform_property(0, TransformProperty::X, 14.0));

        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(14.0)
        );
        assert_eq!(instance.double_property(0, vertex_x_key), Some(14.0));
        assert_eq!(instance.double_property(0, node_x_key), None);
    }

    #[test]
    fn keyed_path_vertex_geometry_mutation_dirties_parent_path() {
        // C++ routes Vertex::xChanged through
        // PathVertex::markGeometryDirty to Path::markPathDirty; the parent
        // PointsPath owns the rebuilt RawPath (`vertex.cpp:14-15`,
        // `path_vertex.cpp:21-30`, `path.cpp:327-334`).
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut path = synthetic_component(0, 0);
        path.type_name = "PointsPath";
        path.capabilities = RuntimeComponentCapabilities::default();
        let mut vertex = synthetic_component(1, 1);
        vertex.type_name = "StraightVertex";
        vertex.capabilities = RuntimeComponentCapabilities::default();
        let mut instance = synthetic_instance(vec![path, vertex], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_keyed_double_property(1, vertex_x_key, 14.0));
        assert!(
            instance
                .component(0)
                .expect("PointsPath component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            !instance
                .component(1)
                .expect("StraightVertex component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
    }

    #[test]
    fn transform_property_mutation_only_recurses_world_transform_to_dependents() {
        let source = synthetic_component(0, 0);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));

        let source = instance.component(0).unwrap();
        assert!(source.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(source.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        let dependent = instance.component(1).unwrap();
        assert!(!dependent.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(dependent.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    }

    #[test]
    fn opacity_mutation_marks_render_opacity_dirty() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::Opacity, 0.35));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.35)
        );
        assert!(component.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!component.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn generic_artboard_opacity_mutation_marks_render_opacity_dirty() {
        let artboard_opacity_key =
            property_key_for_name("Artboard", "opacity").expect("Artboard.opacity key");
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![root], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_double_property(0, artboard_opacity_key, 0.25));

        let root = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.25)
        );
        assert!(root.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!root.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn update_reads_mutated_instance_transform_state() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::X, 9.0));
        assert!(instance.set_transform_property(0, TransformProperty::Y, 4.0));

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([1.0, 0.0, -0.0, 1.0, 9.0, 4.0])
        );
    }

    #[test]
    fn builds_instance_from_graph_fixture() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert_eq!(instance.slots().len(), artboard.local_objects.len());
        assert_eq!(
            instance
                .slots()
                .iter()
                .map(|slot| (slot.local_id, slot.source_global_id, slot.type_name))
                .collect::<Vec<_>>(),
            artboard
                .local_objects
                .iter()
                .map(|object| (object.local_id, object.global_id, object.type_name))
                .collect::<Vec<_>>()
        );
        assert_eq!(instance.components().len(), artboard.components.len());
        let graph_ordered_components = artboard
            .components
            .iter()
            .filter(|component| component.graph_order.is_some())
            .count();
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter(|handle| {
                    matches!(
                        instance.objects.address(**handle),
                        Some(ComponentAddress::Authored(_))
                    )
                })
                .count(),
            graph_ordered_components
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(
            instance
                .components()
                .iter()
                .all(|component| component.dirt == ComponentDirt::FILTHY)
        );
    }

    #[test]
    fn occurrence_schedule_keeps_embedded_path_composer_identity_exact() {
        let bytes = include_bytes!("../../../fixtures/graph/clipping_and_draw_order.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        let (source_local, shape_local, dependent_local) = artboard
            .dependency_nodes
            .iter()
            .enumerate()
            .find_map(|(composer_node, node)| {
                let DependencyNodeKind::PathComposer { shape_local, .. } = node.kind else {
                    return None;
                };
                let source_local = artboard
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .find(|edge| edge.dependent_node == composer_node)
                    .and_then(|edge| artboard.dependency_nodes.get(edge.source_node))
                    .and_then(|node| match node.kind {
                        DependencyNodeKind::Component { local_id, .. } => Some(local_id),
                        _ => None,
                    })?;
                let dependent_local = artboard
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .find(|edge| edge.source_node == composer_node)
                    .and_then(|edge| artboard.dependency_nodes.get(edge.dependent_node))
                    .and_then(|node| match node.kind {
                        DependencyNodeKind::Component { local_id, .. } => Some(local_id),
                        _ => None,
                    })?;
                Some((source_local, shape_local, dependent_local))
            })
            .expect("fixture has Component -> PathComposer -> Component dependency chain");

        let source = instance
            .component_handle(source_local)
            .expect("source handle");
        let shape = instance
            .component_handle(shape_local)
            .expect("shape handle");
        let composer = instance
            .objects
            .path_composer_handle(shape_local)
            .expect("embedded PathComposer handle");
        let dependent = instance
            .component_handle(dependent_local)
            .expect("dependent handle");

        assert_ne!(shape, composer);
        assert_eq!(
            instance.objects.component_local_id(shape),
            instance.objects.component_local_id(composer)
        );
        assert!(matches!(
            instance.objects.address(composer),
            Some(ComponentAddress::PathComposer(_))
        ));
        assert!(
            instance.objects.graph_order(source).unwrap().index()
                < instance.objects.graph_order(composer).unwrap().index()
        );
        assert!(
            instance.objects.graph_order(composer).unwrap().index()
                < instance.objects.graph_order(dependent).unwrap().index()
        );
    }

    #[test]
    fn occurrence_schedule_excludes_draw_order_only_components() {
        let bytes = synthetic_riv(13_103, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 0)]);
            push_synthetic_object(bytes, "DrawTarget", &[("parentId", 0), ("drawableId", 1)]);
            push_synthetic_object(bytes, "DrawRules", &[("parentId", 0), ("drawTargetId", 2)]);
        });
        let instance = instance_from_riv(&bytes);
        let draw_target = instance
            .component_handle(2)
            .expect("DrawTarget remains a live Component occurrence");
        let draw_rules = instance
            .component_handle(3)
            .expect("DrawRules remains a live Component occurrence");

        assert!(
            !instance.objects.dependency_order().contains(&draw_target),
            "DrawTarget inherits empty Component::buildDependencies; its drawable edge belongs only to Artboard draw-order sorting (`src/draw_target.cpp`; `include/rive/component.hpp:50`; `src/artboard.cpp:409-435`)"
        );
        assert!(
            !instance.objects.dependency_order().contains(&draw_rules),
            "DrawRules inherits empty Component::buildDependencies; its target edge belongs only to Artboard draw-order sorting (`src/draw_rules.cpp`; `include/rive/component.hpp:50`; `src/artboard.cpp:409-435`)"
        );
    }

    #[test]
    fn follow_path_node_target_adds_no_path_source_dependency() {
        let bytes = synthetic_riv(13_102, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "FollowPathConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let instance = instance_from_riv(&bytes);
        let target = instance.component_handle(2).expect("ordinary Node target");
        let constraint = instance
            .component_handle(3)
            .expect("FollowPathConstraint handle");
        let constrained_parent = instance
            .component_handle(1)
            .expect("constrained parent handle");

        assert!(
            !(0..instance.objects.dependent_len(target))
                .filter_map(|index| instance.objects.dependent_at(target, index))
                .any(|dependent| dependent == constraint),
            "FollowPath adds a source edge only for Shape/Path targets (`follow_path_constraint.cpp:167-186`)"
        );
        assert!(
            (0..instance.objects.dependent_len(constraint))
                .filter_map(|index| instance.objects.dependent_at(constraint, index))
                .any(|dependent| dependent == constrained_parent)
        );
    }

    #[test]
    fn occurrence_schedule_interleaves_text_helper_before_later_root_child() {
        let bytes = synthetic_riv(9589, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
            push_synthetic_object(bytes, "TextStyle", &[("parentId", 1)]);
            push_synthetic_object(bytes, "TextStyleAxis", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
        });
        let instance = instance_from_riv(&bytes);
        let root = instance.component_handle(0).expect("root handle");
        let text = instance.component_handle(1).expect("Text handle");
        let style = instance.component_handle(2).expect("TextStyle handle");
        let helper = instance
            .objects
            .text_variation_helper_handle(2)
            .expect("TextStyle owns a variation helper");
        let later_child = instance.component_handle(4).expect("later Node handle");

        assert_eq!(
            (0..instance.objects.dependent_len(root))
                .filter_map(|index| instance.objects.dependent_at(root, index))
                .collect::<Vec<_>>(),
            vec![text, helper, later_child],
            "TextStyle invokes helper buildDependencies before its own parent edge, and the later Node runs afterward in object order (`text_style.cpp:128-136`; `text_variation_helper.cpp:7-12`; `artboard.cpp:417-428`)"
        );
        assert_eq!(
            instance.objects.dependency_order(),
            &[root, later_child, helper, text, style],
            "DependencySorter front-inserts completed siblings after visiting retained insertion order (`dependency_sorter.cpp:6-48`)"
        );
    }

    #[test]
    fn clone_rebuilds_component_relations_in_clone_owned_storage() {
        let bytes = include_bytes!("../../../fixtures/graph/clipping_and_draw_order.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");
        let source = instance
            .objects
            .dependency_order()
            .iter()
            .copied()
            .find(|handle| instance.objects.dependent_len(*handle) > 0)
            .expect("fixture has a dependency source");
        let source_dependent = instance
            .objects
            .dependent_at(source, 0)
            .expect("source has a dependent");
        let mut instance = instance;
        let fabricated_bind = DataBindHandle::from_index(usize::MAX);
        assert!(instance.objects.add_collapsable(source, fabricated_bind));
        let source_path = instance
            .objects
            .component_handles()
            .iter()
            .copied()
            .find(|handle| {
                instance
                    .objects
                    .component(*handle)
                    .and_then(|component| component.concrete.path.as_ref())
                    .and_then(|path| path.shape)
                    .is_some()
            })
            .expect("fixture has a Path with a retained Shape owner");
        let source_shape = instance
            .objects
            .component(source_path)
            .and_then(|component| component.concrete.path.as_ref())
            .and_then(|path| path.shape)
            .expect("source Path retained its Shape");
        let source_composer = instance
            .objects
            .component_local_id(source_shape)
            .and_then(|local| instance.objects.path_composer_handle(local))
            .expect("source Shape owns its embedded PathComposer");

        let mut cloned = instance.clone();

        assert!(!std::ptr::eq(
            instance.objects.component(source).unwrap(),
            cloned.objects.component(source).unwrap()
        ));
        assert_eq!(
            cloned.objects.dependent_at(source, 0),
            Some(source_dependent)
        );
        assert_eq!(
            cloned.objects.graph_order(source),
            instance.objects.graph_order(source)
        );
        assert_eq!(
            cloned.objects.collapsable_len(source),
            0,
            "cloneObjectDataBinds rebuilds m_collapsables from cloned authored DataBinds instead of copying the source pointer list (`artboard.cpp:1038-1057`)"
        );
        assert_eq!(
            instance.objects.collapsable_at(source, 0),
            Some(fabricated_bind)
        );
        assert_eq!(
            cloned.objects.dependency_order(),
            instance.objects.dependency_order(),
            "clone rebuild must reproduce the authored occurrence schedule"
        );
        let cloned_shape = cloned
            .objects
            .component(source_path)
            .and_then(|component| component.concrete.path.as_ref())
            .and_then(|path| path.shape)
            .expect("clone Path rebuilt its clone-owned Shape pointer");
        assert_eq!(cloned_shape, source_shape);
        assert!(
            cloned
                .objects
                .component(cloned_shape)
                .and_then(|component| component.concrete.shape.as_ref())
                .is_some_and(|shape| shape.paths.contains(&source_path))
        );
        assert_eq!(
            cloned
                .objects
                .component_local_id(cloned_shape)
                .and_then(|local| cloned.objects.path_composer_handle(local)),
            Some(source_composer),
            "Path reaches the clone-owned embedded composer only through its retained Shape (`path.cpp:76-96`; `follow_path_constraint.cpp:175-186`)"
        );
        assert!(!std::ptr::eq(
            instance.objects.component(source_shape).unwrap(),
            cloned.objects.component(cloned_shape).unwrap()
        ));

        cloned
            .objects
            .component_mut(source)
            .unwrap()
            .dependents
            .clear();
        cloned.objects.component_mut(source).unwrap().dirt = ComponentDirt::NONE;

        assert_eq!(cloned.objects.dependent_len(source), 0);
        assert!(instance.objects.dependent_len(source) > 0);
        assert_ne!(
            cloned.objects.component(source).unwrap().dirt,
            instance.objects.component(source).unwrap().dirt
        );
    }

    #[test]
    fn clone_rebuilds_parent_links_from_clone_owned_generated_parent_id() {
        let bytes = synthetic_riv(9590, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
        });
        let mut instance = instance_from_riv(&bytes);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");

        assert_eq!(instance.component_parent_local(3), Some(1));
        assert!(instance.set_uint_property(3, parent_key, 2));
        assert_eq!(
            instance.component_parent_local(3),
            Some(1),
            "Component.parentIdChanged is intentionally a no-op on the live occurrence"
        );

        let cloned = instance.clone();
        assert_eq!(
            cloned.component_parent_local(3),
            Some(2),
            "fresh clone onAddedDirty resolves the copied generated parentId (`src/component.cpp:19-29`)"
        );
        assert_eq!(instance.component_parent_local(3), Some(1));
    }

    #[test]
    fn targeted_constraint_retains_live_target_and_clone_reresolves_generated_target_id() {
        let bytes = synthetic_riv(9_594, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "RotationConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let mut instance = instance_from_riv(&bytes);
        let owner = instance.component_handle(1).expect("constraint owner");
        let first_target = instance.component_handle(2).expect("first target");
        let second_target = instance.component_handle(3).expect("second target");
        let constraint = instance.component_handle(4).expect("constraint");

        assert_eq!(instance.objects.constraint_len(owner), 1);
        assert_eq!(instance.objects.constraint_at(owner, 0), Some(constraint));
        assert_eq!(
            instance
                .component_at(constraint)
                .concrete
                .constraint
                .expect("retained constraint state")
                .target,
            Some(first_target)
        );
        assert_eq!(
            instance.objects.dependent_at(first_target, 0),
            Some(owner),
            "TargetedConstraint::buildDependencies adds target -> constrained parent (`src/constraints/targeted_constraint.cpp:42-49`)"
        );

        let target_key =
            property_key_for_name("TargetedConstraint", "targetId").expect("targetId key");
        instance.clear_component_dirt(1);
        assert!(instance.set_uint_property(4, target_key, 3));
        assert_eq!(
            instance
                .component_at(constraint)
                .concrete
                .constraint
                .expect("retained constraint state")
                .target,
            Some(first_target),
            "TargetedConstraintBase::targetIdChanged is intentionally empty; a live occurrence does not retarget (`generated/constraints/targeted_constraint_base.hpp`)"
        );
        assert!(
            !instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "targetIdChanged must not dirty the constrained parent"
        );

        let cloned = instance.clone();
        assert_eq!(
            cloned
                .component_at(constraint)
                .concrete
                .constraint
                .expect("clone retained constraint state")
                .target,
            Some(second_target),
            "fresh clone onAddedDirty resolves the copied generated targetId (`src/constraints/targeted_constraint.cpp:23-39`)"
        );
        assert_eq!(cloned.objects.constraint_at(owner, 0), Some(constraint));
        assert_eq!(cloned.objects.dependent_at(second_target, 0), Some(owner));
        assert_eq!(instance.objects.dependent_at(first_target, 0), Some(owner));

        instance.clear_component_dirt(1);
        instance.clear_component_dirt(4);
        assert!(instance.add_dirt(4, ComponentDirt::PATH, false));
        assert!(
            instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "Constraint::onDirty marks its retained parent for every non-empty accumulated dirt (`src/constraints/constraint.cpp:23-29`)"
        );

        instance.clear_component_dirt(1);
        let strength_key = property_key_for_name("Constraint", "strength").expect("strength key");
        assert!(instance.set_double_property(4, strength_key, 0.5));
        assert!(
            instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "ConstraintBase::strengthChanged dirties the retained constrained parent"
        );
    }

    #[test]
    fn targeted_constraint_target_validation_preserves_optional_and_required_contracts() {
        let optional = synthetic_riv(9_595, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "RotationConstraint", &[("parentId", 1)]);
        });
        let optional = instance_from_riv(&optional);
        let optional_constraint = optional.component_handle(2).expect("optional constraint");
        assert_eq!(
            optional
                .component_at(optional_constraint)
                .concrete
                .constraint
                .expect("constraint state")
                .target,
            None
        );

        let required = synthetic_riv(9_596, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "DistanceConstraint", &[("parentId", 1)]);
        });
        let file = read_runtime_file(&required).expect("required fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("required fixture graphs");
        let required = ArtboardInstance::from_graph(&file, &graphs.artboards[0])
            .expect("invalid required-target constraint is filtered before initialize");
        assert!(
            required
                .components()
                .iter()
                .all(|component| component.type_name != "DistanceConstraint"),
            "Artboard::validateObjects removes invalid Components from m_Objects before \
             initialize; MissingObject from onAddedDirty is non-fatal (`artboard.cpp:204-245, \
             264-288`; `targeted_constraint.cpp:7-38`)"
        );

        let wrong_type = synthetic_riv(9_597, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Skin", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "DistanceConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let file = read_runtime_file(&wrong_type).expect("wrong-type fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("wrong-type fixture graphs");
        let wrong_type = ArtboardInstance::from_graph(&file, &graphs.artboards[0])
            .expect("wrong-type target constraint is filtered before initialize");
        assert!(
            wrong_type
                .components()
                .iter()
                .all(|component| component.type_name != "DistanceConstraint"),
            "TargetedConstraint::validate rejects a non-Transform target and the Artboard removes \
             that invalid object before lifecycle dispatch"
        );
    }

    #[test]
    fn transform_constraint_registration_is_an_unconditional_ordered_push() {
        let owner = synthetic_component_for_type(0, "Node");
        let first = synthetic_component_for_type(1, "RotationConstraint");
        let second = synthetic_component_for_type(2, "ScaleConstraint");
        let mut instance = synthetic_instance(vec![owner, first, second], vec![0, 1, 2]);
        let owner = instance.component_handle(0).unwrap();
        let first = instance.component_handle(1).unwrap();
        let second = instance.component_handle(2).unwrap();

        assert!(instance.objects.add_constraint(owner, first));
        assert!(instance.objects.add_constraint(owner, second));
        assert!(instance.objects.add_constraint(owner, first));
        assert_eq!(
            (0..instance.objects.constraint_len(owner))
                .map(|index| instance.objects.constraint_at(owner, index).unwrap())
                .collect::<Vec<_>>(),
            vec![first, second, first],
            "TransformComponent::addConstraint unconditionally push_backs in call order (`src/transform_component.cpp:123-126`)"
        );
    }

    #[test]
    fn clone_relinks_text_variation_helper_to_clone_owned_text_parent() {
        let bytes = synthetic_riv(9591, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
            push_synthetic_object(bytes, "TextStyle", &[("parentId", 1)]);
            push_synthetic_object(bytes, "TextStyleAxis", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
        });
        let mut instance = instance_from_riv(&bytes);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");
        let first_text = instance.component_handle(1).expect("first Text handle");
        let second_text = instance.component_handle(4).expect("second Text handle");

        assert_eq!(
            instance.objects.text_variation_helper_text_handle(2),
            Some(first_text)
        );
        assert!(instance.set_uint_property(2, parent_key, 4));

        let cloned = instance.clone();
        assert_eq!(cloned.component_parent_local(2), Some(4));
        assert_eq!(
            cloned.objects.text_variation_helper_text_handle(2),
            Some(second_text),
            "TextStyle::onAddedClean refreshes m_text before creating the clone-owned helper (`src/text/text_style.cpp:45-70`)"
        );
        let helper = cloned
            .objects
            .text_variation_helper_handle(2)
            .expect("helper handle");
        assert_eq!(
            cloned.objects.dependent_at(helper, 0),
            Some(second_text),
            "TextVariationHelper::buildDependencies reads the rebuilt TextStyle parent (`src/text/text_variation_helper.cpp:7-12`)"
        );
        assert_eq!(
            instance.objects.text_variation_helper_text_handle(2),
            Some(first_text)
        );
    }

    #[test]
    fn data_bind_collapsables_link_in_import_order_and_rebuild_on_clone_initialize() {
        let bytes = include_bytes!("../../../fixtures/flow/data_binding_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");
        let artboard_index = file
            .artboards()
            .iter()
            .position(|candidate| candidate.id == artboard.global_id)
            .expect("graph artboard maps to runtime file");

        let mut expected = BTreeMap::<ComponentHandle, Vec<DataBindHandle>>::new();
        for (data_bind_index, data_bind) in file
            .artboard_data_binds(artboard_index)
            .into_iter()
            .enumerate()
        {
            let Some(target_local_id) = data_bind.target_local_id else {
                continue;
            };
            let Some(component) = instance.objects.component_handle(target_local_id) else {
                continue;
            };
            expected
                .entry(component)
                .or_default()
                .push(DataBindHandle::from_index(data_bind_index));
        }
        assert!(
            !expected.is_empty(),
            "fixture must exercise Component targets"
        );
        for (component, data_binds) in &expected {
            assert_eq!(
                (0..instance.objects.collapsable_len(*component))
                    .filter_map(|index| instance.objects.collapsable_at(*component, index))
                    .collect::<Vec<_>>(),
                *data_binds
            );
        }

        let mut cloned = instance.clone();
        for (component, data_binds) in expected {
            assert_eq!(
                (0..cloned.objects.collapsable_len(component))
                    .filter_map(|index| cloned.objects.collapsable_at(component, index))
                    .collect::<Vec<_>>(),
                data_binds
            );
        }
        let _ = cloned.advance_artboard_data_binds();
    }

    #[test]
    fn unattached_import_only_paths_do_not_rearm_runtime_traversal() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let mut instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert!(instance.update_components().did_update);
        assert!(
            !instance.update_components().did_update,
            "C++ leaves unattached import-only components out of the rooted runtime schedule; their cold Path owners must not re-arm Artboard dirt"
        );
    }

    #[test]
    fn construction_seeds_file_owned_external_fonts_on_the_root_instance() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let font_bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        let external_fonts = BTreeMap::from([(7, Arc::clone(&font_bytes))]);

        let instance = ArtboardInstance::from_graph_with_artboards_and_external_fonts(
            &file,
            artboard,
            &graph.artboards,
            &external_fonts,
        )
        .expect("instance builds with file-owned fonts");

        assert_eq!(instance.external_font_asset_bytes(7), Some(&*font_bytes));
        assert_eq!(
            instance
                .build_context
                .as_ref()
                .and_then(|context| context.external_font_assets.get(&7))
                .map(AsRef::as_ref),
            Some(&*font_bytes)
        );
    }

    #[test]
    fn replacing_file_owned_fonts_updates_existing_nested_children() {
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        instance
            .nested_artboards
            .insert(7, synthetic_nested_artboard_instance(0));
        let font_bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        let external_fonts = BTreeMap::from([(7, Arc::clone(&font_bytes))]);

        instance.replace_external_font_asset_snapshot(&external_fonts);

        let nested = instance
            .nested_artboards
            .get(&7)
            .expect("nested child exists");
        assert_eq!(
            nested.child.external_font_asset_bytes(7),
            Some(&*font_bytes)
        );
        assert!(Arc::ptr_eq(
            &instance.external_font_assets,
            &nested.child.external_font_assets
        ));
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn schema_type_key(type_name: &str) -> u16 {
        definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int
    }

    fn schema_property_key(type_name: &str, property_name: &str) -> u16 {
        property_key_for_name(type_name, property_name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
    }

    fn push_synthetic_object(bytes: &mut Vec<u8>, type_name: &str, properties: &[(&str, u64)]) {
        push_synthetic_object_with_properties(bytes, type_name, |bytes| {
            for (property_name, value) in properties {
                push_synthetic_uint_property(bytes, type_name, property_name, *value);
            }
        });
    }

    fn push_synthetic_object_with_properties(
        bytes: &mut Vec<u8>,
        type_name: &str,
        properties: impl FnOnce(&mut Vec<u8>),
    ) {
        push_var_uint(bytes, u64::from(schema_type_key(type_name)));
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_synthetic_uint_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: u64,
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        push_var_uint(bytes, value);
    }

    fn push_synthetic_f32_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: f32,
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_synthetic_bytes_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: &[u8],
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn synthetic_owned_view_model_action_riv(file_id: u64, listener_action: bool) -> Vec<u8> {
        synthetic_owned_view_model_action_riv_with_options(
            file_id,
            listener_action,
            false,
            false,
            false,
        )
    }

    fn synthetic_owned_view_model_action_riv_with_options(
        file_id: u64,
        listener_action: bool,
        cross_model_trigger_action: bool,
        listener_cascade: bool,
        unrelated_two_way_bind: bool,
    ) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "FontAsset", &[("assetId", 17)]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyList", |bytes| {
                push_synthetic_bytes_property(bytes, "ViewModelPropertyList", "name", b"items");
            });
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
                push_synthetic_bytes_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "name",
                    b"child",
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "viewModelReferenceId",
                    1,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
                push_synthetic_bytes_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "name",
                    b"other_child",
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "viewModelReferenceId",
                    2,
                );
            });
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyTrigger", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyAssetFont", &[]);
            // A same-shaped global model lets listener tests distinguish an
            // authored slot identity from its compatible cross-model occupant.
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyTrigger", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyAssetFont", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    0,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceTrigger",
                    "viewModelPropertyId",
                    2,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceAssetFont", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "viewModelPropertyId",
                    3,
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "propertyValue",
                    0,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    1,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 2)]);
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    0,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    1,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceTrigger",
                    "viewModelPropertyId",
                    2,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceAssetFont", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "viewModelPropertyId",
                    3,
                );
            });
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);

            if listener_action {
                push_synthetic_object(bytes, "StateMachineNumber", &[]);
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 1);
                push_var_uint(&mut listener_path, 0);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                    push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                    push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 7.0);
                });
                push_owned_number_change_action(bytes, 0);
                if cross_model_trigger_action {
                    push_owned_trigger_change_action(bytes, 2, 2);
                    push_owned_number_change_action_for(bytes, 2, 1, 64.0, 0);
                }
                if listener_cascade {
                    let mut cascade_path = Vec::new();
                    push_var_uint(&mut cascade_path, 1);
                    push_var_uint(&mut cascade_path, 1);
                    push_synthetic_object_with_properties(
                        bytes,
                        "StateMachineListenerSingle",
                        |bytes| {
                            push_synthetic_uint_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "targetId",
                                0,
                            );
                            push_synthetic_uint_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "listenerTypeValue",
                                11,
                            );
                            push_synthetic_bytes_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "viewModelPathIds",
                                &cascade_path,
                            );
                        },
                    );
                    push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                        push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                        push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 9.0);
                    });
                }
            }

            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            if !listener_action {
                const STATE_AT_START: u64 = 2 << 1;
                push_owned_number_change_action(bytes, STATE_AT_START);
                if cross_model_trigger_action {
                    push_owned_trigger_change_action_with_flags(bytes, 2, 2, STATE_AT_START);
                }
            }
            push_owned_font_bind(bytes, unrelated_two_way_bind.then_some(1 << 1));
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn synthetic_owned_view_model_listener_chain_riv(
        file_id: u64,
        listener_count: usize,
        close_cycle: bool,
    ) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            for _ in 0..=listener_count {
                push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            }
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            for property_index in 0..=listener_count {
                push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                    push_synthetic_uint_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        property_index as u64,
                    );
                    push_synthetic_f32_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        0.0,
                    );
                });
            }
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);
            for property_index in 0..listener_count {
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 0);
                push_var_uint(&mut listener_path, property_index as u64);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_owned_number_change_action_for(
                    bytes,
                    0,
                    property_index.saturating_add(1) as u64,
                    1.0,
                    0,
                );
            }
            if close_cycle {
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 0);
                push_var_uint(&mut listener_path, listener_count as u64);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_owned_number_change_action_for(bytes, 0, 0, 2.0, 0);
            }
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn synthetic_owned_view_model_listener_live_cycle_riv(file_id: u64) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            for property_index in 0..2 {
                push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                    push_synthetic_uint_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        property_index,
                    );
                    push_synthetic_f32_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        0.0,
                    );
                });
            }
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);

            // This listener order forms a permanent three-phase cycle:
            // (A=1, B=1) -> (A=0, B=0) -> (A=1, B=0).
            push_owned_number_listener_change(bytes, 0, 1, 1.0);
            push_owned_number_listener_change(bytes, 1, 0, 0.0);
            push_owned_number_listener_change(bytes, 0, 0, 1.0);
            push_owned_number_listener_change(bytes, 1, 1, 0.0);

            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn push_owned_number_listener_change(
        bytes: &mut Vec<u8>,
        source_property_index: u64,
        target_property_index: u64,
        value: f32,
    ) {
        let mut listener_path = Vec::new();
        push_var_uint(&mut listener_path, 0);
        push_var_uint(&mut listener_path, source_property_index);
        push_synthetic_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_synthetic_uint_property(bytes, "StateMachineListenerSingle", "targetId", 0);
            push_synthetic_uint_property(
                bytes,
                "StateMachineListenerSingle",
                "listenerTypeValue",
                11,
            );
            push_synthetic_bytes_property(
                bytes,
                "StateMachineListenerSingle",
                "viewModelPathIds",
                &listener_path,
            );
        });
        push_owned_number_change_action_for(bytes, 0, target_property_index, value, 0);
    }

    fn push_owned_number_change_action(bytes: &mut Vec<u8>, flags: u64) {
        push_owned_number_change_action_for(bytes, 1, 1, 42.0, flags);
    }

    fn push_owned_number_change_action_for(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
        value: f32,
        flags: u64,
    ) {
        push_synthetic_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
            push_synthetic_f32_property(bytes, "BindablePropertyNumber", "propertyValue", value);
        });
        let mut output_path = Vec::new();
        push_var_uint(&mut output_path, view_model_index);
        push_var_uint(&mut output_path, property_index);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyNumber",
                    "propertyValue",
                )),
            );
            push_synthetic_uint_property(bytes, "DataBindContext", "flags", 1);
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &output_path);
        });
        push_synthetic_object(bytes, "ListenerViewModelChange", &[("flags", flags)]);
    }

    fn push_owned_trigger_change_action(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
    ) {
        push_owned_trigger_change_action_with_flags(bytes, view_model_index, property_index, 0);
    }

    fn push_owned_trigger_change_action_with_flags(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
        flags: u64,
    ) {
        push_synthetic_object(bytes, "BindablePropertyTrigger", &[("propertyValue", 1)]);
        let mut output_path = Vec::new();
        push_var_uint(&mut output_path, view_model_index);
        push_var_uint(&mut output_path, property_index);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyTrigger",
                    "propertyValue",
                )),
            );
            push_synthetic_uint_property(bytes, "DataBindContext", "flags", 1);
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &output_path);
        });
        push_synthetic_object(bytes, "ListenerViewModelChange", &[("flags", flags)]);
    }

    fn push_owned_font_bind(bytes: &mut Vec<u8>, flags: Option<u64>) {
        push_synthetic_object(
            bytes,
            "BindablePropertyAsset",
            &[("propertyValue", u64::from(u32::MAX))],
        );
        let mut source_path = Vec::new();
        push_var_uint(&mut source_path, 1);
        push_var_uint(&mut source_path, 3);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyAsset",
                    "propertyValue",
                )),
            );
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path);
            if let Some(flags) = flags {
                push_synthetic_uint_property(bytes, "DataBindContext", "flags", flags);
            }
        });
    }

    fn owned_view_model_action_fixture(
        file_id: u64,
        listener_action: bool,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes = synthetic_owned_view_model_action_riv(file_id, listener_action);
        let file = read_runtime_file(&bytes).expect("owned ViewModel action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let runtime_state_machine = artboard.state_machine(0).expect("fixture machine graph");
        assert_eq!(runtime_state_machine.bindable_numbers.len(), 1);
        if listener_action {
            assert_eq!(runtime_state_machine.listeners.len(), 1);
            assert_eq!(
                runtime_state_machine.listeners[0]
                    .view_model_path
                    .as_ref()
                    .and_then(|path| path.absolute_source_path()),
                Some((1, [0].as_slice()))
            );
            assert_eq!(runtime_state_machine.listeners[0].listener_actions.len(), 2);
        }
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    fn owned_view_model_action_fixture_with_unrelated_two_way_bind(
        file_id: u64,
        listener_action: bool,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes = synthetic_owned_view_model_action_riv_with_options(
            file_id,
            listener_action,
            false,
            false,
            true,
        );
        let file = read_runtime_file(&bytes).expect("two-way listener fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    #[test]
    fn component_list_occurrence_ignores_scalar_dirt_but_consumes_structural_rebind() {
        let (file, _, _) = owned_view_model_action_fixture(9713, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row context"),
        );
        let row = RuntimeComponentListItemInstance {
            child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines: Vec::new(),
            context_rebind_sink: {
                let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                context.add_rebind_dependent(&sink);
                sink
            },
            draw_index_sink: None,
            context: context.clone(),
            occurrence_identity: 1,
            logical_index: 0,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            render_cache_revision: 1,
        };
        assert!(row.context_is_current(&context));

        assert!(context.borrow_mut().set_number_by_property_path(&[1], 42.0));
        assert!(row.context_is_current(&context));

        row.context_rebind_sink
            .add_dirt(crate::view_model_cell::RuntimeCellDirt::BINDINGS);
        assert!(!row.context_is_current(&context));
        row.consume_context_rebind_dirt();
        assert!(row.context_is_current(&context));
    }

    fn owned_view_model_action_fixture_with_cross_model_trigger(
        file_id: u64,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(file_id, true, true, false, false);
        let file = read_runtime_file(&bytes).expect("owned ViewModel action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let runtime_state_machine = artboard.state_machine(0).expect("fixture machine graph");
        assert_eq!(runtime_state_machine.listeners.len(), 1);
        assert_eq!(runtime_state_machine.listeners[0].listener_actions.len(), 4);
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    fn owned_view_model_listener_cascade_fixture(
        file_id: u64,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(file_id, true, false, true, false);
        let file = read_runtime_file(&bytes).expect("listener cascade fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    #[test]
    fn immutable_owned_view_model_bind_dispatches_without_mutating_the_borrowed_context() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9680, true);
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context(&context));
        assert!(context.set_number_by_property_index(0, 1.0));
        state_machine.bind_owned_view_model_context(&context);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "rebinding must not execute a queued listener action"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "the documented immutable low-level bind still dispatches non-context listener actions"
        );
        assert_eq!(context.number_value_by_property_path(&[1]), Some(0.0));

        assert!(context.set_number_by_property_index(0, 2.0));
        state_machine.bind_owned_view_model_context_mut(&mut context);
        assert_eq!(context.number_value_by_property_path(&[1]), Some(0.0));
        artboard.advance_state_machine_instances_with_nested_and_owned_view_model_context(
            std::slice::from_mut(&mut state_machine),
            0.0,
            &mut context,
        );
        assert_eq!(context.number_value_by_property_path(&[1]), Some(42.0));
    }

    #[test]
    fn compatibility_context_chain_listener_dispatches_on_the_next_frame() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9682, true);
        let mut root = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a nested owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context_chain(&file, &root, &[&[1]]));
        assert!(root.set_number_by_property_path(&[1, 0], 1.0));
        state_machine.bind_owned_view_model_context_chain(&file, &root, &[&[1]]);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "context-chain rebinding must only retain the listener cell"
        );

        // C++ reports cell dirt immediately but drains listener actions at
        // the next new-frame `applyEvents`, before layer advance
        // (`state_machine_instance.cpp:1374-1380,2320-2335,2555-2565`).
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0)
        );
        assert_eq!(
            root.number_value_by_property_path(&[1, 1]),
            Some(0.0),
            "an immutable context-chain bind cannot receive ViewModel writes"
        );
    }

    #[test]
    fn compatibility_context_chain_converter_retains_the_nested_operand_cell() {
        let (file, _, _) = owned_view_model_action_fixture(9683, true);
        let mut root = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a nested owned ViewModel context");
        assert!(root.set_number_by_property_path(&[1, 0], 4.0));
        let mut converter = RuntimeDataBindGraphConverter::OperationViewModel {
            operation_type: 2,
            operation_value: 0.0,
            default_operation_value: 0.0,
            source_path: Some(vec![1, 0]),
            retained_operation_value: None,
        };

        assert!(
            crate::data_bind_graph::runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                &mut converter,
                &root,
                &[&[1]],
            )
        );
        let RuntimeDataBindGraphConverter::OperationViewModel {
            retained_operation_value: Some(retained),
            ..
        } = &converter
        else {
            panic!("nested operation operand must retain its exact cell")
        };
        assert!(retained.ptr_eq(&root.cell_by_property_path(&[1, 0]).expect("nested cell")));
        assert_eq!(
            crate::data_bind_graph::runtime_data_bind_graph_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(3.0),
            ),
            Some(RuntimeDataBindGraphValue::Number(12.0))
        );
    }

    #[test]
    fn compatibility_mutable_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9689);
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context_mut(&mut context));
        assert!(context.set_number_by_property_index(0, 1.0));
        state_machine.bind_owned_view_model_context_mut(&mut context);

        // C++ applies the report present at new-frame start, then loops the
        // ViewModel write's chained report to completion before layer advance
        // (`state_machine_instance.cpp:2320-2343,2555-2565`).
        artboard.advance_state_machine_instances_with_nested_and_owned_view_model_context(
            std::slice::from_mut(&mut state_machine),
            0.0,
            &mut context,
        );
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "the chained listener must finish in the same applyEvents frame",
        );
    }

    #[test]
    fn composite_owned_view_model_bind_dispatches_view_model_listeners() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9684, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        let context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "the composite artboard context must dispatch its ViewModel listener"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "listener ViewModel writes must reach the retained composite main context"
        );
    }

    #[test]
    fn composite_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9700);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        let context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "applyEvents must drain listener A then its chained listener B before layer advance",
        );
    }

    #[test]
    fn retained_handle_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9701);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "retained listener reports must preserve the chained FIFO order",
        );
    }

    #[test]
    fn retained_data_context_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9702);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "the retained DataContext must drain the same applyEvents queue",
        );
    }

    #[test]
    fn retained_state_action_does_not_rebind_an_unrelated_two_way_data_bind() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_unrelated_two_way_bind(9730, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        let bind_count = state_machine.owned_data_bind_context_bind_count();
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the state-entry ListenerViewModelChange must still reach its exact source",
        );
        assert_eq!(
            state_machine.owned_data_bind_context_bind_count(),
            bind_count,
            "an exact listener write must not reconcile the fixture's unrelated two-way bind",
        );
    }

    #[test]
    fn retained_listener_report_does_not_rebind_an_unrelated_two_way_data_bind() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_unrelated_two_way_bind(9731, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        let bind_count = state_machine.owned_data_bind_context_bind_count();
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the queued ListenerViewModelChange must still reach its exact source",
        );
        assert_eq!(
            state_machine.owned_data_bind_context_bind_count(),
            bind_count,
            "a queued exact listener write must not reconcile the fixture's unrelated two-way bind",
        );
    }

    #[test]
    fn retained_data_context_listener_reads_the_live_bindable_occurrence() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9732, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(state_machine.set_bindable_number_for_data_bind(0, 9.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(9.0),
            "ListenerViewModelChange must read the mutable cloned BindableProperty at perform time, not its imported default",
        );
    }

    #[test]
    fn retained_data_context_listener_queues_each_mutation_until_next_frame() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9720, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        // Binding the DataContext registers the listener condition as a
        // dependent on the retained cell it reads (#RB-1 e4).
        assert!(state_machine.bind_owned_view_model_handle(&context));
        let condition_cell = context
            .borrow()
            .cell_by_property_path(&[0])
            .expect("condition property has a retained cell");
        let bound_cell = state_machine
            .view_model_listener_condition_cell(0)
            .expect("DataContext bind migrates the scalar condition");
        assert!(
            bound_cell.ptr_eq(&condition_cell),
            "the listener must observe the SAME retained cell the context owns"
        );

        // A slot write reports the listener immediately, but C++ performs its
        // actions only from next-frame applyEvents
        // (`state_machine_instance.cpp:2320-2335,3021-3025`).
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert_eq!(
            state_machine.pending_listener_view_model_report_count(),
            1,
            "the cell cascade must append one listener report"
        );
        state_machine.bind_owned_view_model_handle(&context);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "rebinding must not execute a queued listener action"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "next-frame applyEvents must dispatch the listener actions"
        );
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );

        // C++ deliberately preserves duplicates instead of collapsing a
        // transient 1→2→1 into a net-equal observed copy.
        assert!(context.borrow_mut().set_number_by_property_path(&[1], 0.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 2.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert_eq!(
            state_machine.pending_listener_view_model_report_count(),
            2,
            "both genuine mutations must remain queued in order"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the transient reports must execute instead of disappearing behind a net diff"
        );
    }

    #[test]
    fn retained_data_context_listener_apply_events_cap_leaves_batch_101_pending() {
        const LISTENER_CAP: usize = 100;
        let bytes = synthetic_owned_view_model_listener_chain_riv(9705, LISTENER_CAP + 1, false);
        let file = read_runtime_file(&bytes).expect("listener boundary fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );
        let data_context = RuntimeOwnedDataContext::from_root_handle(context.clone());

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP),
            Some(1.0),
        );
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP + 1),
            Some(0.0),
            "the applyEvents batch cap must stop before listener 101",
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 1);

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP + 1),
            Some(1.0),
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
    }

    #[test]
    fn retained_data_context_listener_cycle_settles_without_replaying() {
        let bytes = synthetic_owned_view_model_listener_chain_riv(9706, 2, true);
        let file = read_runtime_file(&bytes).expect("listener cycle fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );
        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_index(0),
            Some(2.0)
        );
        assert_eq!(
            context.borrow().number_value_by_property_index(1),
            Some(1.0)
        );
        assert_eq!(
            context.borrow().number_value_by_property_index(2),
            Some(1.0)
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
    }

    #[test]
    fn retained_data_context_listener_live_cycle_stays_pending_at_apply_events_cap() {
        let bytes = synthetic_owned_view_model_listener_live_cycle_riv(9707);
        let file = read_runtime_file(&bytes).expect("listener live-cycle fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert!(state_machine.has_pending_listener_view_model_reports());

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert!(state_machine.has_pending_listener_view_model_reports());
    }

    #[test]
    fn retained_scoped_context_refresh_dispatches_listener_actions_to_the_scope() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9688, true);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");

        assert!(state_machine.bind_owned_view_model_context_handle(&scoped));
        assert!(root.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "automatic retained-handle refresh must observe the scoped listener source"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "automatic retained-handle refresh must route listener writes back into the scope"
        );
    }

    #[test]
    fn composite_listener_preserves_authored_view_model_identity() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9689, true);
        let same_shaped_main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a same-shaped non-global ViewModel"),
        );
        let authored_global = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the listener's authored global ViewModel"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(same_shaped_main.clone());
        assert!(context.set_global_slot_handle(&file, 1, authored_global.clone()));
        assert!(
            same_shaped_main
                .borrow_mut()
                .set_trigger_by_property_index(2, 9)
        );
        assert!(
            authored_global
                .borrow_mut()
                .set_trigger_by_property_index(2, 3)
        );

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(
            authored_global
                .borrow_mut()
                .set_number_by_property_index(0, 1.0)
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "a same-shaped main model must not mask a listener authored against global slot 1"
        );
        assert_eq!(
            authored_global.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
        assert_eq!(
            same_shaped_main
                .borrow()
                .number_value_by_property_path(&[1]),
            Some(0.0),
            "listener observation and writes must retain their authored ViewModel identity"
        );
    }

    #[test]
    fn listener_write_rejects_cross_model_global_slot_occupant() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9690, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a main ViewModel context"),
        );
        let override_instance = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model override"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(context.set_global_slot_handle(&file, 1, override_instance.clone()));
        assert!(
            override_instance
                .borrow_mut()
                .set_trigger_by_property_index(2, 3)
        );
        assert!(
            override_instance
                .borrow_mut()
                .set_font_asset_index_by_property_index(3, 7)
        );

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert_eq!(
            state_machine.bindable_asset_value_for_data_bind(1),
            Some(RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX),
            "font synchronization must reject the same wrong-model occupant as the data-bind graph"
        );
        assert!(
            override_instance
                .borrow_mut()
                .set_number_by_property_index(0, 1.0)
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "C++ DataContext rejects the wrong-model occupant before listener dispatch (data_context.cpp:397-506)"
        );
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the authored source remains unresolved against a wrong-model slot occupant"
        );
        assert_eq!(
            override_instance
                .borrow()
                .number_value_by_property_path(&[1]),
            Some(0.0),
            "listener writes must not be redirected through the slot key"
        );
    }

    #[test]
    fn cross_model_listener_trigger_does_not_fire_default_view_model_transition_trigger() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_cross_model_trigger(9694);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the listener's default ViewModel"),
        );
        let cross_model = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has the compatible cross-model trigger target"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 2, cross_model.clone()));

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert_eq!(
            state_machine.default_view_model_trigger_source_value_for_data_bind(1),
            Some(0),
            "cross-model trigger source must be represented in the bound graph"
        );
        assert_eq!(
            state_machine.bindable_trigger_value_for_data_bind(1),
            Some(1),
            "listener trigger bindable retains its authored action value"
        );
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            cross_model.borrow().trigger_value_by_property_path(&[2]),
            Some(1),
            "the listener action must still fire its declared cross-model trigger target"
        );
        assert_eq!(
            cross_model.borrow().number_value_by_property_path(&[1]),
            Some(64.0),
            "a non-default global number action must retain its schema-backed source and reach the declared slot"
        );
    }

    #[test]
    fn switching_from_retained_handle_to_composite_clears_stale_refresh_source() {
        let (file, _artboard, mut state_machine) = owned_view_model_action_fixture(9696, false);
        let stale = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the original retained ViewModel"),
        );
        let replacement = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the replacement composite ViewModel"),
        );
        assert!(stale.borrow_mut().set_number_by_property_index(1, 5.0));
        assert!(
            replacement
                .borrow_mut()
                .set_number_by_property_index(1, 8.0)
        );

        assert!(state_machine.bind_owned_view_model_handle(&stale));
        let contexts = RuntimeOwnedViewModelContext::from_main_handle(replacement);
        assert!(state_machine.bind_owned_view_model_contexts(&contexts));
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(8.0)
        );

        assert!(stale.borrow_mut().set_number_by_property_index(1, 9.0));
        let _ = state_machine.advance_data_context();
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(8.0),
            "advance_data_context must not resurrect the previously retained single handle after a composite bind"
        );
    }

    #[test]
    fn retained_composite_does_not_route_authored_path_through_cross_model_slot() {
        let (file, mut artboard, state_machine) = owned_view_model_action_fixture(9697, false);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a distinct main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));
        let mut state_machines = vec![state_machine];

        assert!(state_machines[0].bind_owned_view_model_contexts(&contexts));
        assert!(artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "slot keys only place globals; C++ lookup compares the actual occupant viewModelId (data_context.cpp:397-506)"
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0);
        assert_eq!(
            state_machines[0].default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the unresolved authored source must remain at its default"
        );
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the one-shot state action must not replay on the unchanged second advance"
        );
    }

    #[test]
    fn retained_state_action_rejects_slot_without_matching_actual_model() {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(9704, false, true, false, false);
        let file = read_runtime_file(&bytes).expect("state trigger action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the default ViewModel"),
        );
        let slot_two_occupant = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has a same-type global-slot occupant"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 2, slot_two_occupant.clone()));
        let mut state_machines = vec![state_machine];

        assert!(state_machines[0].bind_owned_view_model_contexts(&context));
        assert!(artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0));
        assert_eq!(
            slot_two_occupant
                .borrow()
                .trigger_value_by_property_path(&[2]),
            Some(0),
            "same-model locals are resolved in DataContext order, not by slot key (data_context.cpp:397-506)",
        );
        assert_eq!(main.borrow().trigger_value_by_property_path(&[2]), Some(0));
    }

    #[test]
    fn artboard_created_machine_rejects_cross_model_global_occupant() {
        let (file, mut artboard, _) = owned_view_model_action_fixture(9698, false);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a distinct main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));
        let _ = artboard.bind_owned_view_model_artboard_contexts(&file, &contexts);
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("bound artboard creates its state machine");

        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "artboard-created machines inherit actual-id DataContext resolution (data_context.cpp:397-506)",
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the wrong-model global occupant must remain unresolved",
        );
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the one-shot state action must not replay during alias refresh",
        );
    }

    #[test]
    fn retained_scoped_context_routes_state_entry_action_into_scope() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9699, false);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");

        assert!(state_machine.bind_owned_view_model_context_handle(&scoped));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "scheduled state actions must resolve through the retained scope path",
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1]),
            None,
            "the scoped write must not be redirected to the root object",
        );
    }

    #[test]
    fn scoped_data_context_bind_dispatches_view_model_listeners() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9685, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, main.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");
        let data_context = RuntimeOwnedDataContext::from_context_handle(&scoped);

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(main.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "scoped DataContexts must dispatch their ViewModel listener"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "listener ViewModel writes must reach the retained scoped path"
        );
    }

    #[test]
    fn later_local_data_context_instance_owns_listener_observation_and_writes() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9686, true);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has nested DataContext instances"),
        );
        let invalid_first = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![2])
            .expect("fixture has a same-shaped wrong-model child");
        let resolved_later = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture has the matching child");
        let data_context = RuntimeOwnedDataContext::with_local_context_handles(
            [invalid_first, resolved_later],
            None,
        );

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(root.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "listener observation must fall through an invalid first local instance"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "listener writes must follow the data-bind source into the later local instance"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[2, 1]),
            Some(0.0),
            "the unresolved first local instance must remain untouched"
        );
    }

    #[test]
    fn composite_context_listener_falls_through_main_to_global_slot() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9687, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a main ViewModel context"),
        );
        let global = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has a global ViewModel context"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 1, global.clone()));

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(global.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "listener observation must follow composite main-to-global ordering"
        );
        assert_eq!(
            global.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "listener writes must reach the global slot that resolved the source"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1, 1]),
            Some(0.0),
            "the composite main context must not receive the global source write"
        );
    }

    #[test]
    fn component_list_advance_writes_state_actions_to_the_item_context() {
        let (file, mut child, mut state_machine) = owned_view_model_action_fixture(9681, false);
        child.update_pass();
        let context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        let mut root_context = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a root owned ViewModel context");
        let list_source = root_context
            .list_source_handle_by_property_name("items")
            .expect("fixture root exposes its item list");
        assert_eq!(
            root_context.replace_list_items_by_source_handle(&list_source, vec![context.clone()]),
            Some(true)
        );
        let list = root_context
            .list_handle_by_property_path(list_source.path())
            .expect("fixture root retains its item list");
        let row = list
            .item_entries()
            .into_iter()
            .next()
            .expect("fixture list has one retained row");
        state_machine.bind_owned_view_model_handle(&row.instance);

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.set_component_list_source(1, Some(list.clone()));
        parent.component_list_state_mut(1).unwrap().items =
            vec![RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    row.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: row.instance,
                occurrence_identity: row.occurrence_identity,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: row.occurrence_identity,
            }];

        let cache_epoch = parent.cache_epoch();
        let prepared_epoch = parent.prepared_epoch();
        let retained_path_generation = parent.path_epoch();
        let layout_epoch = parent.layout_epoch();
        assert!(parent.advance_nested_artboards(0.0));
        let context = &parent.component_list_items(1).unwrap()[0].context;
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
        assert_eq!(
            list.items()[0].borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the retained list source must observe the item-owned write"
        );
        // C++ only dirties the component-list host when the mounted child
        // retains Components dirt after its advance. This fixture's scalar is
        // unprojected, so the row write stays local
        // (`artboard_component_list.cpp:827-885`, especially 870-881).
        assert_eq!(parent.cache_epoch(), cache_epoch);
        assert_eq!(parent.prepared_epoch(), prepared_epoch);
        assert_eq!(parent.path_epoch(), retained_path_generation);
        assert_eq!(parent.layout_epoch(), layout_epoch);
    }

    #[test]
    fn component_list_machine_rejects_inherited_cross_model_global_slot() {
        let (file, child, state_machine) = owned_view_model_action_fixture(9703, false);
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a row that does not occupy declared slot 1"),
        );
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has the parent main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.component_list_state_mut(1).unwrap().items =
            vec![RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    row.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: row.clone(),
                occurrence_identity: 1,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: 1,
            }];
        let _ = parent.bind_owned_view_model_artboard_contexts(&file, &contexts);

        assert!(parent.advance_nested_artboards(0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "row parent fallback still resolves by actual viewModelId, never the inherited slot key (data_context.cpp:397-506)",
        );
        assert_eq!(
            row.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "a same-shaped row of another ViewModel type must not steal the declared global action",
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = parent.advance_nested_artboards(0.0);
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the retained inherited alias must refresh without replaying the one-shot state action",
        );
    }

    #[test]
    fn component_list_reverse_writes_target_the_exact_repeated_occurrence() {
        let (file, child, mut state_machine) = owned_view_model_action_fixture(9682, false);
        let context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        let mut root_context = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a root owned ViewModel context");
        let list_source = root_context
            .list_source_handle_by_property_name("items")
            .expect("fixture root exposes its item list");
        assert_eq!(
            root_context.replace_list_items_by_source_handle(
                &list_source,
                vec![context.clone(), context.clone()],
            ),
            Some(true)
        );
        let list = root_context
            .list_handle_by_property_path(list_source.path())
            .expect("fixture root retains its item list");
        let rows = list.item_entries();
        let first = rows[0].clone();
        let second = rows[1].clone();
        state_machine.bind_owned_view_model_handle(&second.instance);

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.set_component_list_source(1, Some(list.clone()));
        parent.component_list_state_mut(1).unwrap().items = vec![
            RuntimeComponentListItemInstance {
                child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: Vec::new(),
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    first.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: first.instance,
                occurrence_identity: first.occurrence_identity,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: first.occurrence_identity,
            },
            RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    second.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: second.instance,
                occurrence_identity: second.occurrence_identity,
                logical_index: 1,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: second.occurrence_identity,
            },
        ];

        assert!(parent.advance_nested_artboards(0.0));
        let source_items = list.items();
        assert_eq!(
            source_items[0].borrow().number_value_by_property_path(&[1]),
            Some(0.0)
        );
        assert_eq!(
            source_items[1].borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
    }

    fn synthetic_riv(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, file_id);
        push_var_uint(&mut bytes, 0);
        object_stream(&mut bytes);
        bytes
    }

    fn instance_from_riv(bytes: &[u8]) -> ArtboardInstance {
        let file = read_runtime_file(bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");
        ArtboardInstance::from_graph(&file, artboard).expect("instance builds")
    }

    #[test]
    fn nested_state_machine_rejects_artboard_list_map_rule_parent() {
        let bytes = synthetic_riv(9599, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "ArtboardListMapRule", &[("parentId", 0)]);
            push_synthetic_object(bytes, "NestedStateMachine", &[("parentId", 1)]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");

        let error = match ArtboardInstance::from_graph(&file, artboard) {
            Ok(_) => panic!("non-container Component parent must be rejected"),
            Err(error) => error,
        };
        assert_eq!(
            error.to_string(),
            "Component NestedStateMachine local 2 parent local 1 type ArtboardListMapRule is not a ContainerComponent"
        );
    }

    fn assert_collapsed(instance: &ArtboardInstance, local_id: usize, collapsed: bool) {
        assert_eq!(
            instance
                .component(local_id)
                .unwrap_or_else(|| panic!("missing component {local_id}"))
                .is_collapsed(),
            collapsed,
            "component {local_id} collapse mismatch"
        );
    }

    #[test]
    fn instantiating_an_artboard_without_solos_skips_solo_mapping_analysis() {
        let bytes = synthetic_riv(9600, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 1)]);
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 1, false);
        assert_collapsed(&instance, 2, false);
        assert_eq!(solo_mapping_work(), SoloMappingWork::default());
    }

    #[test]
    fn imported_solo_mapping_preserves_a_null_slot_before_the_active_child() {
        let bytes = synthetic_riv(9604, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: abstract runtime objects become indexed null slots in
            // C++ Artboard::objects(). They must not compact later local ids.
            push_synthetic_object(bytes, "BindableProperty", &[]);
            // Local 2: solo; local 4 is active across the null slot.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 4)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 2)]);
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 3, true);
        assert_collapsed(&instance, 4, false);
        let solo = instance
            .component(2)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("Solo occurrence owns its retained state");
        assert_eq!(solo.cpp_local_ids, [3, 4]);
        assert_eq!(
            solo_mapping_work(),
            SoloMappingWork {
                analyses: 1,
                batch_queries: 1,
                visited_slots: 5,
            }
        );
    }

    fn imported_solo_mapping_work(child_count: usize) -> SoloMappingWork {
        let bytes = synthetic_riv(9605 + child_count as u64, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            for _ in 0..child_count {
                push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            }
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);
        assert_eq!(
            instance
                .components()
                .iter()
                .filter(|component| component.concrete.solo.is_some())
                .count(),
            1
        );
        solo_mapping_work()
    }

    #[test]
    fn solo_mapping_analysis_is_one_batched_linear_pass() {
        let small_child_count = 8;
        let large_child_count = 64;

        let small = imported_solo_mapping_work(small_child_count);
        let large = imported_solo_mapping_work(large_child_count);

        assert_eq!(small.analyses, 1);
        assert_eq!(large.analyses, 1);
        assert_eq!(small.batch_queries, 1);
        assert_eq!(large.batch_queries, 1);
        assert_eq!(small.visited_slots, small_child_count + 2);
        assert_eq!(large.visited_slots, large_child_count + 2);
    }

    #[test]
    fn imported_solos_retain_children_on_each_occurrence_without_behavior_drift() {
        let bytes = synthetic_riv(9670, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1 with active/inactive children 2 and 3.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 4 with active/inactive children 5 and 6.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 5)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
        });

        reset_solo_mapping_work();
        let mut instance = instance_from_riv(&bytes);

        let first = instance
            .component(1)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("first Solo occurrence");
        let second = instance
            .component(4)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("second Solo occurrence");
        assert_eq!(first.cpp_local_ids, [2, 3]);
        assert_eq!(second.cpp_local_ids, [5, 6]);
        assert_eq!(
            solo_mapping_work(),
            SoloMappingWork {
                analyses: 1,
                batch_queries: 1,
                visited_slots: 7,
            }
        );

        assert_collapsed(&instance, 2, false);
        assert_collapsed(&instance, 3, true);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, true);

        assert!(instance.set_solo_active_child_by_index(1, 1.0));
        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, true);

        assert!(instance.set_solo_active_child_by_index(4, 1.0));
        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 5, true);
        assert_collapsed(&instance, 6, false);
    }

    // Regression for the M8 audit finding: apply_initial_solo_collapses only
    // flagged DIRECT solo children, so Solo -> Group -> Shape left the Shape
    // un-collapsed (and drawing) on a fresh instance without a state machine.
    // C++ Solo::onAddedClean recurses the full subtree (src/solo.cpp).
    #[test]
    fn initial_solo_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9601, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: solo with the first group active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            // Local 2/3: active branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 2)]);
            // Local 4/5: statically-inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 4)]);
        });
        let instance = instance_from_riv(&bytes);
        assert_collapsed(&instance, 2, false);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 5, true);
        let inactive_composer = instance
            .objects
            .path_composer_handle(5)
            .expect("inactive Shape owns its PathComposer");
        assert!(
            instance
                .objects
                .component(inactive_composer)
                .expect("inactive PathComposer remains live")
                .is_collapsed(),
            "Shape::collapse must keep its embedded PathComposer collapsed"
        );
    }

    // Regression for the M8 audit finding: collapse propagation from a
    // display:none layout recursed only into Artboard|LayoutComponent
    // children, so display:none -> Node -> Shape still drew. C++
    // LayoutComponent::propagateCollapse recurses through
    // ContainerComponent::collapse (src/layout_component.cpp).
    #[test]
    fn initial_display_none_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9602, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: hidden layout; local 2: its style with display:none.
            push_synthetic_object(bytes, "LayoutComponent", &[("parentId", 0), ("styleId", 2)]);
            push_synthetic_object(bytes, "LayoutComponentStyle", &[("displayValue", 1)]);
            // Local 3/4: plain-node chain under the hidden layout.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 3)]);
        });
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 3, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 4, true);
    }

    // Regression for the M8 audit finding: collapse_component_tree_with_ancestor
    // blindly un-collapsed descendants, clobbering a nested solo's
    // re-collapsed inactive children. C++ Solo::collapse skips the blind
    // container child walk (src/solo.cpp).
    #[test]
    fn solo_switch_preserves_nested_solo_inactive_collapse() {
        let bytes = synthetic_riv(9603, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: outer solo, group A (local 2) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 3: inactive group B holding a nested solo.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 4: inner solo, group C (local 5) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 3), ("activeComponentId", 5)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 5)]);
            // Local 7/8: inner solo's inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 7)]);
        });
        let mut instance = instance_from_riv(&bytes);

        // Fresh instance: the whole inactive outer branch is collapsed.
        for local_id in 3..=8 {
            assert_collapsed(&instance, local_id, true);
        }

        // Switch the outer solo to group B (child index 1).
        assert!(instance.set_solo_active_child_by_index(1, 1.0));

        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, false);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, false);
        // The nested solo's inactive branch must stay collapsed; the blind
        // descendant walk un-collapsed it before the fix.
        assert_collapsed(&instance, 7, true);
        assert_collapsed(&instance, 8, true);
    }

    #[test]
    fn component_list_children_select_the_cpp_default_state_machine_index() {
        assert_eq!(component_list_default_state_machine_index(Some(1), 3), 1);
        assert_eq!(component_list_default_state_machine_index(None, 3), 0);
        assert_eq!(component_list_default_state_machine_index(Some(3), 3), 0);
        assert_eq!(
            component_list_default_state_machine_index(Some(u64::MAX), 3),
            0
        );
    }

    #[test]
    fn state_machine_frame_settles_deep_nested_render_opacity() {
        let typed_component = |local_id: usize, graph_order: usize, type_name: &'static str| {
            let mut component = synthetic_component(local_id, graph_order);
            component.type_name = type_name;
            component.transform_property_keys =
                crate::components::TransformPropertyKeys::for_type(type_name);
            component
        };

        let mut leaf_root = typed_component(0, 0, "Artboard");
        leaf_root.transform.render_opacity = 0.0;
        let mut leaf = synthetic_instance(vec![leaf_root], vec![0]);
        let opacity_key = property_key_for_name("Artboard", "opacity").expect("opacity key");
        assert!(leaf.set_double_property(0, opacity_key, 0.0));
        leaf.clear_component_dirt(0);
        leaf.set_artboard_dirt_for_test(ComponentDirt::NONE);

        let mut middle_root = typed_component(0, 0, "Artboard");
        middle_root.transform.render_opacity = 1.0;
        let mut middle_host = typed_component(1, 1, "NestedArtboard");
        middle_host.dirt = ComponentDirt::RENDER_OPACITY;
        let mut middle = synthetic_instance(vec![middle_root, middle_host], vec![1]);
        synthetic_link_parent(&mut middle, 1, 0);
        let mut leaf_mount = synthetic_nested_artboard_instance(2);
        leaf_mount.child = Box::new(leaf);
        middle.nested_artboards.insert(1, leaf_mount);
        middle.nested_artboard_locals.push(1);
        middle.set_artboard_dirt_for_test(ComponentDirt::COMPONENTS);

        let mut root_component = typed_component(0, 0, "Artboard");
        root_component.transform.render_opacity = 1.0;
        let mut root_host = typed_component(1, 1, "NestedArtboard");
        root_host.transform.render_opacity = 1.0;
        root_host.dirt = ComponentDirt::COMPONENTS;
        let mut root = synthetic_instance(vec![root_component, root_host], vec![1]);
        synthetic_link_parent(&mut root, 1, 0);
        let mut middle_mount = synthetic_nested_artboard_instance(1);
        middle_mount.child = Box::new(middle);
        root.nested_artboards.insert(1, middle_mount);
        root.nested_artboard_locals.push(1);

        root.update_pass();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert!(leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));

        root.settle_state_machine_update_passes();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 1.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));
    }

    #[test]
    fn component_list_row_state_machine_uses_parent_focus_domain_for_all_input_channels() {
        let bytes = synthetic_riv(9703, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyList", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceList",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceListItem",
                &[("viewModelId", 1), ("viewModelInstanceId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "ArtboardComponentList", |bytes| {
                push_synthetic_uint_property(bytes, "ArtboardComponentList", "parentId", 0);
                push_synthetic_f32_property(bytes, "ArtboardComponentList", "opacity", 1.0);
            });
            // The parent StateMachine owns the shared FocusManager before the
            // row is mounted.
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "Node", |bytes| {
                push_synthetic_uint_property(bytes, "Node", "parentId", 0);
                push_synthetic_f32_property(bytes, "Node", "opacity", 1.0);
            });
            push_synthetic_object(bytes, "FocusData", &[("parentId", 1), ("focusFlags", 7)]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineBool", &[]);
            push_synthetic_object(bytes, "StateMachineListener", &[("targetId", 1)]);
            push_synthetic_object(
                bytes,
                "ListenerInputTypeKeyboard",
                &[("listenerTypeValue", RuntimeListenerType::Keyboard as u64)],
            );
            push_synthetic_object(
                bytes,
                "ListenerInputTypeText",
                &[("listenerTypeValue", RuntimeListenerType::TextInput as u64)],
            );
            push_synthetic_object(
                bytes,
                "ListenerInputTypeGamepad",
                &[("listenerTypeValue", RuntimeListenerType::Gamepad as u64)],
            );
            // Toggle once for each delivered channel so order/duplication is
            // observable without relying on a handled return.
            push_synthetic_object(bytes, "ListenerBoolChange", &[("inputId", 0), ("value", 2)]);
        });
        let file = read_runtime_file(&bytes).expect("component-list focus fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list focus graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");
        // The ordinary runtime constructs focus topology after the initial
        // component update has published render opacity. This low-level
        // fixture bypasses the facade initialization, so mirror that C++
        // ordering explicitly.
        parent.update_components();
        let mut parent_machine = parent
            .state_machine_instance(0)
            .expect("parent focus-manager owner");

        let list_local_id = graph.artboards[0].component_lists[0].local_id;
        let row_context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("component-list row context"),
        );
        assert!(parent.sync_component_list_items(&file, list_local_id, vec![row_context],));

        // Refresh the parent's retained projection after the dynamic row was
        // linked, then focus through the row machine's shared manager handle.
        parent_machine.sync_focus_for_test(&parent);
        let row_target_local = graph.artboards[1]
            .components
            .iter()
            .find(|component| component.type_name == "Node")
            .expect("row focus target")
            .local_id;
        {
            let row = parent
                .component_list_items_mut(list_local_id)
                .and_then(|items| items.first_mut())
                .expect("mounted row");
            assert!(
                row.state_machines[0].set_focus_target_for_test(row_target_local),
                "C++ setExternalFocusManager makes the row target visible in the parent domain"
            );
        }

        assert!(!parent_machine.key_input(&mut parent, 65, 0, true, false));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true)
        );
        assert!(!parent_machine.text_input(&mut parent, "owned"));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(false)
        );
        assert!(!parent_machine.gamepad_dispatch(
            &mut parent,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true),
            "key, text, and gamepad each reach the same list-row occurrence exactly once"
        );
    }

    #[test]
    fn component_list_mount_settles_context_without_advancing_the_row_state_machine() {
        let bytes = synthetic_riv(9702, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyList", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceList",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceListItem",
                &[("viewModelId", 1), ("viewModelInstanceId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "ArtboardComponentList", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        });
        let file = read_runtime_file(&bytes).expect("component-list mount fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list fixture graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");

        let list_local_id = graph.artboards[0].component_lists[0].local_id;
        let row_context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("component-list row context"),
        );
        assert!(parent.sync_component_list_items(&file, list_local_id, vec![row_context.clone()],));
        let mounted = parent
            .component_list_items(list_local_id)
            .and_then(|items| items.first())
            .expect("mounted component-list row");
        assert!(mounted.context.ptr_eq(&row_context));
        assert!(
            mounted
                .child
                .owned_view_model_context()
                .and_then(RuntimeOwnedViewModelContext::main_handle)
                .is_some_and(|context| context.ptr_eq(&row_context)),
            "the mounted child must publicly expose its occurrence-scoped row context"
        );
        assert_eq!(
            mounted
                .state_machines
                .first()
                .expect("row default state machine")
                .changed_state_count(),
            0,
            "mount links and settles the row context but leaves state advancement to the normal list pass"
        );

        parent.advance_nested_artboards(0.0);
        let advanced = parent.component_list_items(list_local_id).unwrap()[0]
            .state_machines
            .first()
            .expect("row default state machine");
        assert_eq!(advanced.changed_state_count(), 1);
    }
}
