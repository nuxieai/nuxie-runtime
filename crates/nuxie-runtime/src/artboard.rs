use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result};
use nuxie_binary::RuntimeFile;
#[cfg(test)]
use nuxie_graph::ResettingComponentKind;
use nuxie_graph::{AdvancingComponentKind, ArtboardGraph, DependencyNode, DependencyNodeKind};
use nuxie_render_api::Factory as RenderFactory;
use nuxie_schema::definition_by_name;

use crate::animation::{
    LinearAnimationInstance, RuntimeInterpolator, RuntimeKeyedCallback, RuntimeLinearAnimation,
    RuntimeLinearAnimationHandle, build_linear_animations,
};
use crate::artboard_data_bind::{
    RuntimeArtboardAuthoredDataBindStates, RuntimeArtboardContextSourceValue,
    RuntimeArtboardConverterPropertyBindingInstance, RuntimeArtboardDataBindSourceQueues,
    RuntimeArtboardDataBindTargetQueues, RuntimeArtboardFormulaTokenBindingStates,
    RuntimeArtboardImageAssetBindingInstance, RuntimeArtboardLayoutComputedBindingInstance,
    RuntimeArtboardListBindingInstance, RuntimeArtboardNestedHostBindingInstance,
    RuntimeArtboardNumericSourceBindingInstance, RuntimeArtboardPropertyBindingInstance,
    RuntimeArtboardRetainedSubordinateConverterOperands, RuntimeArtboardSoloBindingInstance,
    RuntimeArtboardSoloSourceBindingInstance, RuntimeArtboardTextListBindingInstance,
    RuntimeNestedChildContextUpdate, RuntimeOwnedDataContext,
    apply_artboard_name_based_color_data_bind_defaults, build_artboard_authored_data_bind_states,
    build_artboard_converter_property_bindings, build_artboard_default_view_model_values,
    build_artboard_formula_token_bindings, build_artboard_image_asset_bindings,
    build_artboard_layout_computed_bindings, build_artboard_list_bindings,
    build_artboard_nested_host_bindings, build_artboard_numeric_source_bindings,
    build_artboard_property_bindings, build_artboard_solo_bindings,
    build_artboard_solo_source_bindings, build_artboard_text_list_bindings,
    build_nested_host_data_bind_source_local_slots, build_nested_host_data_bind_source_locals,
    build_nested_host_view_model_instance_locals,
    reunite_artboard_shared_data_bind_converter_states,
};
use crate::audio_event::RuntimeAudioEventPlayback;
use crate::components::{
    ComponentDirt, ComponentHandle, Mat2D, RuntimeComponent, RuntimeConstrainableListState,
    RuntimeIkChainLink, RuntimeSkinnableKind, TransformComponents, TransformProperty,
    UpdateComponentsReport, retain_runtime_component_layout_topology,
    retain_runtime_layout_component_styles, retain_runtime_solos,
    retain_runtime_text_input_scroll_constraints,
};
use crate::constraints::scrolling::scroll_virtualizer::component_list_virtualization;
use crate::constraints::{
    apply_scroll_offset_changed, retain_runtime_scroll_constraints, runtime_scroll_double_property,
    set_runtime_scroll_double_property,
};
use crate::custom_property_container::{
    RuntimeArtboardCustomPropertyBindingInstance, build_artboard_custom_property_bindings,
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
use crate::joystick::{RuntimeJoystick, build_runtime_joysticks};
use crate::objects::{ComponentAddress, InstanceObjectArena, InstanceSlot};
use crate::properties::{
    RuntimeArtboardDimensions, artboard_index_for_graph,
    layout_component_style_display_value_property_key, property_key_for_name,
    solid_color_value_property_key, solo_active_component_id_property_key,
    transform_property_for_key,
};
use crate::scene::select_default_state_machine;
use crate::script_asset::{RuntimeScriptImplementedMethods, RuntimeScriptedObjectOccurrence};
use crate::scripted_interpolator::RuntimeScriptedInterpolatorState;
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
use crate::{
    RuntimeScriptedInterpolatorDiagnostic, RuntimeScriptedInterpolatorFactory,
    ScriptInterpolatorMethod,
};

mod advancing_component;
#[path = "artboard/artboard_component_list.rs"]
mod artboard_component_list;
pub(crate) use advancing_component::RuntimeAdvancingComponent;
#[path = "artboard/bones/bone.rs"]
mod bone;
#[path = "artboard/nested_artboard.rs"]
mod nested_artboard;
#[path = "artboard/nested_artboard_layout.rs"]
mod nested_artboard_layout;
#[path = "artboard/node.rs"]
mod node;
#[path = "artboard/animation/property_recorder.rs"]
mod property_recorder;
mod resetting_component;
pub(crate) use resetting_component::RuntimeResettingComponent;
#[path = "artboard/shapes/paint/solid_color.rs"]
mod solid_color;
#[path = "artboard/text/text_style.rs"]
mod text_style;
use text_style::RuntimeTextStyleFeatureOption;
#[path = "artboard/text/text_input.rs"]
mod text_input;
#[path = "artboard/text/text_value_run.rs"]
mod text_value_run;
#[path = "artboard/text/text_variation_helper.rs"]
mod text_variation_helper;
#[path = "artboard/transform_component.rs"]
mod transform_component;
#[path = "artboard/virtualizing_component.rs"]
mod virtualizing_component;

// C++ `Artboard::sm_frameId` is global across artboards and advances at each
// public `Artboard::draw` entry. Scripted paths use it to distinguish a second
// rebuild in the current frame from a reusable rebuild in a later frame.
static ARTBOARD_DRAW_FRAME_ID: AtomicU64 = AtomicU64::new(0);

#[doc(hidden)]
pub fn artboard_draw_frame_id() -> u64 {
    ARTBOARD_DRAW_FRAME_ID.load(Ordering::Relaxed)
}

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

/// Rejection from attaching host-supplied bytes to one external `FontAsset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalFontAssetError {
    UnknownAsset { asset_id: u32 },
    WrongAssetKind { asset_id: u32, actual: &'static str },
    InvalidFont { asset_id: u32 },
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

/// Test-owned equivalent of the C++ `PersistentDirtProbeComponent`.
///
/// It participates in the ordinary Artboard advance/update phases; it does
/// not reproduce the settlement loop or call its policy directly.
#[cfg(test)]
#[derive(Debug, Clone, Default)]
struct PersistentDirtComponentFixture {
    local_id: usize,
    advance_count: usize,
    update_count: usize,
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
    audio_event_playback: RuntimeAudioEventPlayback,
    /// Transient layout clones share the mounted occurrence's playback owner
    /// but must not run its destructor-side stop hook.
    audio_lifecycle_armed: bool,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) clip: bool,
    /// C++ `Artboard::m_hostOpacity`: opacity imposed by a mounted
    /// `NestedArtboard`, kept separate from the artboard's authored opacity.
    pub(crate) host_opacity: f32,
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
    #[cfg(test)]
    persistent_dirt_component_fixture: Option<PersistentDirtComponentFixture>,
    #[cfg(test)]
    update_pass_data_bind_call_count: usize,
    /// C++ `Artboard::m_Resettables`, retained in authored object order.
    pub(crate) resetting_components: Vec<RuntimeResettingComponent>,
    /// C++ `Artboard::m_ComponentLists`, retained in authored object order.
    /// Handles address the concrete list occurrences in `objects`; steady
    /// paths never rescan Components to rediscover this family
    /// (`src/artboard.cpp:330-395`).
    component_lists: Vec<ComponentHandle>,
    component_list_resource_pools: RuntimeComponentListResourcePools,
    pub(crate) joysticks_apply_before_update: bool,
    pub(crate) linear_animations: Arc<Vec<RuntimeLinearAnimation>>,
    shared_scripted_interpolators: RefCell<RuntimeScriptedInterpolatorState>,
    scripted_interpolator_factories: BTreeMap<u32, RuntimeScriptedInterpolatorFactory>,
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
    /// State machines detached from one actively advancing nested occurrence.
    ///
    /// The occurrence itself must leave `nested_artboards` so Rust can hand
    /// the parent and child to the callback-major policy simultaneously.
    /// Keeping its state-machine owners here preserves same-host nested-input
    /// listener actions while each callback completes through the parent.
    pub(crate) active_nested_state_machines: BTreeMap<usize, StateMachineInstance>,
    pub(crate) nested_artboard_locals: Vec<usize>,
    newly_uncollapsed_nested_artboards: BTreeSet<usize>,
    pub(crate) graph_global_id: u32,
    pub(crate) profile_name: String,
    pub(crate) profile_path: Vec<crate::ProfilePathSegment>,
    build_context: Option<RuntimeArtboardBuildContext>,
    pub(crate) nested_context_source_tree_cache: Cell<Option<(u64, bool)>>,
    nested_layout_bounds: Option<RuntimeNestedLayoutBoundsFrame>,
    previous_nested_layout_transfers: BTreeMap<
        usize,
        (
            RuntimeNestedLayoutDataTransferKey,
            Option<Arc<BTreeMap<usize, RuntimeLayoutBounds>>>,
        ),
    >,
    consumed_mounted_layout_hosts: BTreeSet<usize>,
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
    /// `ListPath::m_vertexListeners` projected into the cloned Artboard
    /// occurrence. Rows own their synthetic vertices and exact cell
    /// subscriptions; drawing only borrows the ordered projection.
    runtime_list_paths: RefCell<Vec<crate::shapes::list_path::RuntimeListPathState>>,
    pub(crate) artboard_context_source_values_scratch: Vec<RuntimeArtboardContextSourceValue>,
    pub(crate) artboard_nested_child_context_updates_scratch: Vec<RuntimeNestedChildContextUpdate>,
    /// C++ nested artboards retain authored view-model instances by pointer,
    /// so clean frames do not reconcile detached copies. Rust only needs the
    /// full ordered reconciliation after a source value or context changes.
    pub(crate) stateful_nested_view_model_contexts_dirty: bool,
    /// Authored `ViewModelInstanceValue` locals changed since the last
    /// reconciliation. These writes are newer than the detached retained
    /// cell, even when that cell still carries an unacknowledged child write.
    pub(crate) stateful_nested_view_model_dirty_locals: BTreeSet<usize>,
    pub(crate) image_asset_overrides: BTreeMap<usize, Option<u32>>,
    pub(crate) image_render_overrides: BTreeMap<usize, crate::RuntimeViewModelImage>,
    text_style_font_overrides: BTreeMap<usize, RuntimeFontAssetValue>,
    text_style_feature_options: RefCell<BTreeMap<usize, RuntimeTextStyleFeatureOption>>,
    text_variation_modifier_tags: RefCell<BTreeMap<usize, (u64, u32)>>,
    pub(crate) runtime_images: crate::draw::image::RuntimeImageList,
    external_font_assets: Arc<BTreeMap<u32, Arc<[u8]>>>,
    pub(crate) runtime_font_assets: Arc<crate::RuntimeFontAssetOwners>,
    pub(crate) runtime_font_asset_snapshots: BTreeMap<u32, Arc<[u8]>>,
    pub(crate) runtime_font_asset_referencer:
        Rc<crate::font_asset::RuntimeFontAssetReferencerQueue>,
    /// C++ File/ImageAsset ownership projected into the runtime occurrence
    /// tree. Every clone retains the same file-owned owner list; Images borrow
    /// RenderImage from it and never from a facade scene cache.
    pub(crate) runtime_image_assets:
        RefCell<Option<Arc<crate::draw::image_asset::RuntimeImageAssetOwners>>>,
    /// Occurrence-local callback sink registered with every retained
    /// ImageAsset owner, mirroring `FileAssetReferencer` registration.
    pub(crate) runtime_image_asset_referencer:
        Rc<crate::draw::image_asset::RuntimeImageAssetReferencerQueue>,
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
    pub(crate) layout_revision: u64,
    text_shape_revision: u64,
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
    /// Component occurrences that consumed C++ semantic-bounds dirt since the
    /// last semantic-tree synchronization. `SemanticData` is a dependent of
    /// its geometry owner, so either local can appear here
    /// (`semantic_data.cpp:258-293`).
    semantic_bounds_dirty_locals: BTreeSet<usize>,
    /// `Artboard::takeLayoutData()` transferred this occurrence's root Yoga
    /// node to a hosting layout. Child-local solves still retain descendants,
    /// but must not overwrite the parent-owned root result.
    pub(crate) layout_node_owned_by_host: bool,
    pub(crate) suppress_mounted_component_list_layout_updates: bool,
    pub(crate) layout_constraint_bounds_enabled: bool,
    pub(crate) layout_constraint_bounds: Option<Arc<BTreeMap<usize, RuntimeLayoutBounds>>>,
    solved_layout_bounds: Option<Arc<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

impl Clone for ArtboardInstance {
    fn clone(&self) -> Self {
        let instance_identity = self.instance_identity.clone();
        let mut cloned = Self {
            audio_event_playback: self
                .audio_event_playback
                .cold_clone(crate::AudioArtboardId(instance_identity.0)),
            instance_identity,
            audio_lifecycle_armed: true,
            width: self.width,
            height: self.height,
            origin_x: self.origin_x,
            origin_y: self.origin_y,
            clip: self.clip,
            host_opacity: self.host_opacity,
            frame_origin: self.frame_origin.clone(),
            frame_id: self.frame_id.clone(),
            slots: self.slots.clone(),
            objects: self.objects.clone(),
            joysticks: self.joysticks.clone(),
            advancing_components: self.advancing_components.clone(),
            #[cfg(test)]
            persistent_dirt_component_fixture: self.persistent_dirt_component_fixture.clone(),
            #[cfg(test)]
            update_pass_data_bind_call_count: 0,
            resetting_components: self.resetting_components.clone(),
            component_lists: self.component_lists.clone(),
            component_list_resource_pools: RuntimeComponentListResourcePools::default(),
            joysticks_apply_before_update: self.joysticks_apply_before_update,
            linear_animations: self.linear_animations.clone(),
            shared_scripted_interpolators: RefCell::new(RuntimeScriptedInterpolatorState::default()),
            scripted_interpolator_factories: self.scripted_interpolator_factories.clone(),
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
            active_nested_state_machines: BTreeMap::new(),
            nested_artboard_locals: self.nested_artboard_locals.clone(),
            newly_uncollapsed_nested_artboards: self.newly_uncollapsed_nested_artboards.clone(),
            graph_global_id: self.graph_global_id,
            profile_name: self.profile_name.clone(),
            profile_path: self.profile_path.clone(),
            build_context: self.build_context.clone(),
            nested_context_source_tree_cache: self.nested_context_source_tree_cache.clone(),
            nested_layout_bounds: self.nested_layout_bounds.clone(),
            previous_nested_layout_transfers: BTreeMap::new(),
            consumed_mounted_layout_hosts: BTreeSet::new(),
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
            runtime_list_paths: RefCell::new(
                self.runtime_list_paths
                    .borrow()
                    .iter()
                    .map(crate::shapes::list_path::RuntimeListPathState::cold_clone)
                    .collect(),
            ),
            artboard_context_source_values_scratch: self
                .artboard_context_source_values_scratch
                .clone(),
            artboard_nested_child_context_updates_scratch: self
                .artboard_nested_child_context_updates_scratch
                .clone(),
            stateful_nested_view_model_contexts_dirty: self
                .stateful_nested_view_model_contexts_dirty,
            stateful_nested_view_model_dirty_locals: self
                .stateful_nested_view_model_dirty_locals
                .clone(),
            image_asset_overrides: self.image_asset_overrides.clone(),
            image_render_overrides: self.image_render_overrides.clone(),
            text_style_font_overrides: self.text_style_font_overrides.clone(),
            // C++ clones generated feature fields, then builds a fresh
            // occurrence-local optioned-font cache during clean/add.
            text_style_feature_options: RefCell::new(BTreeMap::new()),
            text_variation_modifier_tags: RefCell::new(BTreeMap::new()),
            runtime_images: self.runtime_images.clone(),
            external_font_assets: self.external_font_assets.clone(),
            runtime_font_assets: Arc::clone(&self.runtime_font_assets),
            runtime_font_asset_snapshots: self.runtime_font_asset_snapshots.clone(),
            runtime_font_asset_referencer: Rc::new(Default::default()),
            runtime_image_assets: self.runtime_image_assets.clone(),
            runtime_image_asset_referencer: Rc::new(Default::default()),
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
            layout_revision: self.layout_revision,
            text_shape_revision: self.text_shape_revision,
            text_affecting_locals: self.text_affecting_locals.clone(),
            solid_color_paint_revisions: self.solid_color_paint_revisions.clone(),
            runtime_drawables: self.runtime_drawables.clone(),
            runtime_shapes: self.runtime_shapes.clone(),
            runtime_clipping_shapes: self.runtime_clipping_shapes.clone(),
            runtime_meshes: self.runtime_meshes.clone(),
            did_change: self.did_change.clone(),
            semantic_bounds_dirty_locals: BTreeSet::new(),
            layout_node_owned_by_host: self.layout_node_owned_by_host,
            suppress_mounted_component_list_layout_updates: self
                .suppress_mounted_component_list_layout_updates,
            layout_constraint_bounds_enabled: self.layout_constraint_bounds_enabled,
            layout_constraint_bounds: self.layout_constraint_bounds.clone(),
            solved_layout_bounds: self.solved_layout_bounds.clone(),
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
            cloned.joysticks =
                build_runtime_joysticks(&graph, &cloned.objects, cloned.linear_animations.as_ref());
            cloned.joysticks_apply_before_update = cloned
                .joysticks
                .iter()
                .all(RuntimeJoystick::can_apply_before_update);
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
        cloned.initialize_root_layout_bounds();
        cloned.initialize_text_inputs();

        // Generated C++ clones start with `ComponentDirt::Filthy` and clear
        // custom DataBind flags. Re-run authored Solo/Layout collapse only
        // after the clone-owned Component ↔ DataBind links are rebuilt.
        cloned.rederive_initial_component_collapse();
        let image_assets = cloned.runtime_image_assets.borrow().clone();
        if let Some(owners) = image_assets {
            cloned.attach_runtime_image_assets_tree(owners);
        }
        cloned.refresh_runtime_font_asset_referencers();
        cloned
    }
}

impl Drop for ArtboardInstance {
    fn drop(&mut self) {
        if self.audio_lifecycle_armed {
            self.audio_event_playback.stop_artboard();
        }
    }
}

include!("nested_artboard.rs");

include!("artboard_component_list.rs");
include!("artboard_list_map_rule.rs");
include!("artboard_referencer.rs");
include!("bindable_artboard.rs");
include!("nested_artboard_layout.rs");
include!("nested_artboard_leaf.rs");
include!("component_origin.rs");
include!("bones/weight.rs");
include!("profiler/rive_profile.rs");

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

/// Result of one root frame-component advance for a state-machine scene:
/// `notified` triggers the pinned zero-time follow-up advance; `changed`
/// feeds the `advanceAndApply` keep-going composition.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeFrameComponentsAdvance {
    pub notified: bool,
    pub changed: bool,
}

/// Probe-facing snapshot of one mounted nested remap animation occurrence.
#[cfg(feature = "tools")]
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeNestedRemapAnimationReport {
    pub host_local_id: usize,
    pub local_id: usize,
    pub animation_time: f32,
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
    runtime_font_assets: Arc<crate::RuntimeFontAssetOwners>,
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
            // ClippingShape is a sibling renderer owner mounted below Text,
            // not part of Text's shaped/style content. Its generated
            // callbacks dirty the retained clipping path only
            // (`clipping_shape.cpp:117-173`).
            if slots.get(current_local).and_then(|slot| slot.type_name) == Some("ClippingShape") {
                break;
            }
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

fn dependency_edge_targets_component(
    edge: &nuxie_graph::DependencyNodeEdge,
    nodes: &[DependencyNode],
    kind: nuxie_graph::DependencyKind,
    local_id: usize,
) -> bool {
    edge.kind == kind
        && nodes.get(edge.dependent_node).is_some_and(|node| {
            matches!(
                &node.kind,
                DependencyNodeKind::Component {
                    local_id: dependent_local,
                    ..
                } if *dependent_local == local_id
            )
        })
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

impl ArtboardInstance {
    /// Thin borrow-model hook for C++
    /// `Artboard::clearDataContext` as called by the StateMachineInstance bind
    /// family. Substantive bind ordering remains owned by
    /// `state_machine_instance.rs`; this releases the artboard/host retained
    /// identities and invalidates scripted lifetimes before the immediately
    /// following internal bind.
    pub(crate) fn clear_data_context_for_state_machine_bind(&mut self) {
        self.artboard_owned_view_model_context = None;
        self.artboard_owned_data_context = None;
        self.artboard_owned_view_model_handle = None;
        self.artboard_owned_view_model_rebind_sink =
            crate::view_model_cell::RuntimeCellDirtSink::new();
        self.stateful_nested_view_model_contexts_dirty = true;
        let scripted_occurrences = self
            .script_instances_by_global
            .iter()
            .map(|(global_id, occurrence)| (*global_id, occurrence.instance()))
            .collect::<Vec<_>>();
        for (global_id, instance) in scripted_occurrences {
            instance.borrow_mut().invalidate_for_init_retry();
            self.set_script_owner_lifecycle(global_id, false, false);
        }
        for nested in self.nested_artboards.values_mut() {
            for animation in &mut nested.animations {
                if let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation {
                    occurrence.clear_data_context();
                }
            }
            nested.child.clear_data_context_for_state_machine_bind();
        }
    }

    /// Thin counterpart of `Artboard::unbind` for
    /// `StateMachineInstance::bindViewModelInstance(nullptr)`.
    pub(crate) fn unbind_for_state_machine_view_model_clear(&mut self, file: Option<&RuntimeFile>) {
        self.clear_data_context_for_state_machine_bind();
        if let Some(file) = file {
            let empty = RuntimeOwnedDataContext::default();
            self.bind_owned_view_model_artboard_data_context(file, &empty, true, true);
        }
        self.clear_data_context_for_state_machine_bind();
    }

    /// Artboard-only relink delegation used by
    /// `StateMachineInstance::relinkDataContext`.
    pub(crate) fn relink_data_context_for_state_machine(&mut self, file: &RuntimeFile) -> bool {
        let Some(data_context) = self.artboard_owned_data_context.clone() else {
            return false;
        };
        self.bind_owned_view_model_artboard_data_context(file, &data_context, false, true)
    }

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
        self.solved_layout_bounds = None;
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
        self.solved_layout_bounds = source.solved_layout_bounds.clone();
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
        self.audio_event_playback
            .replace_with_transient_view_of(&source.audio_event_playback);
        self.audio_lifecycle_armed = false;
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
                        .then(crate::constraints::targeted_constraint::targeted_constraint_target_id_property_key)
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
                // Cycle guard: this is the unbounded shape-parent walk from
                // `Path::onAddedClean` that hangs C++ on a malformed parent
                // cycle. We deliberately DIVERGE and terminate via the
                // DependencySorter visited-set idiom (see
                // runtime_layout_ancestors in components.rs); a cycle with no
                // Shape fails as if the chain ended. Unreachable on any valid
                // file.
                let mut visited = BTreeSet::new();
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
                    if !visited.insert(candidate) {
                        anyhow::bail!("Path is missing its owning Shape");
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
                    // Cycle guard: parentBoneCount is file-controlled, so a
                    // malformed bone parent cycle would revisit a chain bone
                    // (C++ registers the duplicate peer constraint; Rust's
                    // uniqueness assert would fire). Terminate as if the
                    // chain ended -- reverse_chain doubles as the visited
                    // set. Unreachable on any valid file.
                    if reverse_chain.contains(&parent) {
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

            // LinearGradient::buildDependencies and Feather::buildDependencies
            // run at their authored object slots. Both attach the concrete
            // component directly to the owning Shape/PathComposer (or other
            // ShapePaintContainer) path builder. These insertion points are
            // observable because DependencySorter visits dependents in their
            // retained insertion order. In particular, a gradient authored
            // before an inner feather must remain ahead of that feather
            // (`linear_gradient.cpp:32-61`, `feather.cpp:77-89`).
            let authored_path_dependency = match component.type_name {
                "LinearGradient" | "RadialGradient" => {
                    Some(nuxie_graph::DependencyKind::LinearGradientPaintContainer)
                }
                "Feather" => Some(nuxie_graph::DependencyKind::FeatherPathBuilder),
                _ => None,
            };
            if let Some(kind) = authored_path_dependency {
                for (edge_index, edge) in graph
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .enumerate()
                {
                    if dependency_edge_targets_component(
                        edge,
                        &graph.dependency_nodes,
                        kind,
                        component.local_id,
                    ) {
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

        // `TransformComponent::onAddedClean` retains its typed parent and
        // immediately calls `parent->addDependent(this)`. Keep both halves of
        // that construction invariant together: parent-composed world opacity
        // and transforms cannot be dirtied reliably if either relationship is
        // missing (`src/transform_component.cpp:14-23`).
        for component in &graph.components {
            let handle = objects
                .component_handle(component.local_id)
                .expect("authored Transform invariant requires its retained handle");
            let retained = objects
                .component(handle)
                .expect("authored Transform invariant requires its retained component");
            if !retained.capabilities.transform {
                continue;
            }
            let Some(parent) = retained.parent else {
                continue;
            };
            if !objects
                .component(parent)
                .is_some_and(|parent| parent.capabilities.world_transform)
            {
                continue;
            }
            assert_eq!(
                retained.parent_transform,
                Some(parent),
                "authored Transform child must retain its typed parent_transform",
            );
            assert!(
                (0..objects.dependent_len(parent))
                    .filter_map(|index| objects.dependent_at(parent, index))
                    .any(|dependent| dependent == handle),
                "authored Transform child must remain in parent.dependents",
            );
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
            runtime_font_assets: Arc::new(crate::RuntimeFontAssetOwners::from_runtime(file)),
        };
        Self::from_graph_inner(
            file,
            graph,
            &artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
            Vec::new(),
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
            runtime_font_assets: Arc::new(
                crate::RuntimeFontAssetOwners::from_runtime_with_external_fonts(
                    file,
                    external_font_assets,
                ),
            ),
        };
        Self::from_graph_inner(
            file,
            graph,
            artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
            Vec::new(),
        )
    }

    fn from_graph_inner(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        visiting: &mut BTreeSet<u32>,
        build_context: Option<RuntimeArtboardBuildContext>,
        layout_constraint_bounds_enabled: bool,
        profile_path: Vec<crate::ProfilePathSegment>,
    ) -> Result<Self> {
        let external_font_assets = build_context
            .as_ref()
            .map(|context| Arc::clone(&context.external_font_assets))
            .unwrap_or_default();
        let runtime_font_assets = build_context
            .as_ref()
            .map(|context| Arc::clone(&context.runtime_font_assets))
            .unwrap_or_else(|| Arc::new(crate::RuntimeFontAssetOwners::from_runtime(file)));
        let runtime_font_asset_snapshots = file
            .file_assets()
            .into_iter()
            .filter(|asset| asset.type_name == "FontAsset")
            .filter_map(|asset| {
                runtime_font_assets
                    .get(asset.id)
                    .map(|bytes| (asset.id, bytes))
            })
            .collect();
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
        let joysticks = build_runtime_joysticks(graph, &objects, &linear_animations);
        let joysticks_apply_before_update = joysticks
            .iter()
            .all(RuntimeJoystick::can_apply_before_update);
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
        let artboard_nested_host_bindings = build_artboard_nested_host_bindings(file, graph);
        let artboard_data_bind_target_queues = RuntimeArtboardDataBindTargetQueues::new(
            &artboard_property_bindings,
            &artboard_image_asset_bindings,
            &artboard_converter_property_bindings,
            &artboard_nested_host_bindings,
            &artboard_list_bindings,
        );
        let artboard_solo_bindings = build_artboard_solo_bindings(file, graph);
        let artboard_solo_source_bindings = build_artboard_solo_source_bindings(file, graph);
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
                &profile_path,
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
        let instance_identity = RuntimeArtboardInstanceIdentity::next();
        let audio_event_playback = RuntimeAudioEventPlayback::new(
            crate::AudioArtboardId(instance_identity.0),
            file,
            &slots,
            Arc::new(crate::RuntimeAudioAssetOwners::from_runtime(file)),
        );
        let mut instance = Self {
            instance_identity,
            audio_event_playback,
            audio_lifecycle_armed: true,
            width: dimensions.width,
            height: dimensions.height,
            origin_x: dimensions.origin_x,
            origin_y: dimensions.origin_y,
            clip: dimensions.clip,
            host_opacity: 1.0,
            frame_origin: Cell::new(true),
            frame_id: Cell::new(0),
            slots,
            objects,
            joysticks,
            advancing_components,
            #[cfg(test)]
            persistent_dirt_component_fixture: None,
            #[cfg(test)]
            update_pass_data_bind_call_count: 0,
            resetting_components,
            component_lists,
            component_list_resource_pools: RuntimeComponentListResourcePools::default(),
            joysticks_apply_before_update,
            linear_animations: Arc::new(linear_animations),
            shared_scripted_interpolators: RefCell::new(RuntimeScriptedInterpolatorState::default()),
            scripted_interpolator_factories: BTreeMap::new(),
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
            active_nested_state_machines: BTreeMap::new(),
            nested_artboard_locals,
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: graph.global_id,
            profile_name: graph.name.clone().unwrap_or_default(),
            profile_path,
            build_context,
            nested_context_source_tree_cache: Cell::new(None),
            nested_layout_bounds: None,
            previous_nested_layout_transfers: BTreeMap::new(),
            consumed_mounted_layout_hosts: BTreeSet::new(),
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
            runtime_list_paths: RefCell::new(
                graph
                    .components
                    .iter()
                    .filter(|component| component.type_name == "ListPath")
                    .map(|component| {
                        crate::shapes::list_path::RuntimeListPathState::new(component.local_id)
                    })
                    .collect(),
            ),
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            stateful_nested_view_model_contexts_dirty: true,
            stateful_nested_view_model_dirty_locals: BTreeSet::new(),
            image_asset_overrides: BTreeMap::new(),
            image_render_overrides: BTreeMap::new(),
            text_style_font_overrides: BTreeMap::new(),
            text_style_feature_options: RefCell::new(BTreeMap::new()),
            text_variation_modifier_tags: RefCell::new(BTreeMap::new()),
            runtime_images: crate::draw::image::RuntimeImageList::from_graph(file, graph),
            external_font_assets,
            runtime_font_assets,
            runtime_font_asset_snapshots,
            runtime_font_asset_referencer: Rc::new(Default::default()),
            runtime_image_assets: RefCell::new(None),
            runtime_image_asset_referencer: Rc::new(Default::default()),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            geometry_state: RefCell::new(crate::draw::RuntimeGeometryState::default()),
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_revision: 1,
            text_shape_revision: 1,
            text_affecting_locals,
            solid_color_paint_revisions,
            runtime_drawables,
            runtime_shapes: RuntimeShapeList::from_graph(graph),
            runtime_clipping_shapes: RuntimeClippingShapeList::from_graph(graph),
            runtime_meshes: crate::draw::RuntimeMeshList::from_graph(graph),
            did_change: Cell::new(true),
            semantic_bounds_dirty_locals: BTreeSet::new(),
            layout_node_owned_by_host: false,
            suppress_mounted_component_list_layout_updates: false,
            layout_constraint_bounds_enabled,
            layout_constraint_bounds: None,
            solved_layout_bounds: None,
        };
        instance.initialize_root_layout_bounds();
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
        instance.initialize_text_inputs();
        instance.register_runtime_font_asset_referencers(file, graph);
        let nested_host_locals = instance.nested_artboard_locals.clone();
        for host_local_id in nested_host_locals {
            instance.sync_nested_artboard_root_opacity(host_local_id);
        }

        Ok(instance)
    }

    fn initialize_root_layout_bounds(&self) {
        let Some(layout) = self
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
        else {
            return;
        };
        // Artboard::initialize seeds its root Layout from the authored
        // dimensions before any pointer hit test or layout solve
        // (`src/artboard.cpp:264-273`). Occurrence cloning reruns that same
        // initialize lifecycle.
        layout.retain_bounds(0.0, 0.0, self.width, self.height);
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

    pub(crate) fn runtime_font_asset_bytes(&self, asset_global: u32) -> Option<&[u8]> {
        self.runtime_font_asset_snapshots
            .get(&asset_global)
            .map(AsRef::as_ref)
    }

    fn register_runtime_font_asset_referencers(&self, file: &RuntimeFile, graph: &ArtboardGraph) {
        let styles = graph.local_objects.iter().filter_map(|local| {
            let object = file.object(local.global_id as usize)?;
            if !matches!(object.type_name, "TextStyle" | "TextStylePaint") {
                return None;
            }
            let asset_index = object
                .uint_property("fontAssetId")
                .and_then(|index| usize::try_from(index).ok())?;
            let asset = file.file_asset(asset_index)?;
            (asset.type_name == "FontAsset").then_some((asset.id, local.local_id))
        });
        self.runtime_font_asset_referencer.replace_styles(styles);
        self.runtime_font_assets
            .register_referencer(&self.runtime_font_asset_referencer);
    }

    fn refresh_runtime_font_asset_referencers(&self) {
        let Some((file, graph)) = self.build_context.as_ref().and_then(|context| {
            let graph_index = context
                .artboard_index_by_global
                .get(usize::try_from(self.graph_global_id).ok()?)
                .copied()
                .flatten()?;
            Some((context.file.as_ref(), context.artboards.get(graph_index)?))
        }) else {
            return;
        };
        self.register_runtime_font_asset_referencers(file, graph);
    }

    /// Attach the file-owned ImageAsset/FontAsset/AudioAsset owner set to this complete
    /// occurrence tree and to contexts that may materialize children later.
    pub fn attach_runtime_file_asset_owners(&mut self, owners: &crate::RuntimeFileAssetOwners) {
        if let Some(images) = owners.loader_image_assets() {
            self.attach_runtime_image_assets_tree(images);
        }
        self.attach_runtime_font_assets_tree(owners.font_assets());
        self.attach_runtime_audio_assets_tree(owners.audio_assets());
    }

    fn attach_runtime_audio_assets_tree(&mut self, owners: Arc<crate::RuntimeAudioAssetOwners>) {
        self.audio_event_playback.set_assets(Arc::clone(&owners));
        for nested in self.nested_artboards.values_mut() {
            nested
                .child
                .attach_runtime_audio_assets_tree(Arc::clone(&owners));
        }
        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for local_id in list_locals {
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child
                        .attach_runtime_audio_assets_tree(Arc::clone(&owners));
                }
            }
        }
    }

    fn attach_runtime_font_assets_tree(&mut self, owners: Arc<crate::RuntimeFontAssetOwners>) {
        self.runtime_font_assets = Arc::clone(&owners);
        self.runtime_font_asset_snapshots.clear();
        if let Some(context) = self.build_context.as_ref() {
            for asset in context.file.file_assets() {
                if asset.type_name == "FontAsset"
                    && let Some(bytes) = owners.get(asset.id)
                {
                    self.runtime_font_asset_snapshots.insert(asset.id, bytes);
                }
            }
        }
        if let Some(context) = self.build_context.as_mut() {
            context.runtime_font_assets = Arc::clone(&owners);
        }
        self.refresh_runtime_font_asset_referencers();
        for nested in self.nested_artboards.values_mut() {
            nested
                .child
                .attach_runtime_font_assets_tree(Arc::clone(&owners));
        }
        for list_index in 0..self.component_list_count() {
            let Some(local_id) = self.component_list_local_at(list_index) else {
                continue;
            };
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child
                        .attach_runtime_font_assets_tree(Arc::clone(&owners));
                }
            }
        }
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

    /// Drain VM-thread async completions for every retained scripted
    /// occurrence, including parked owners, and recurse through occurrence
    /// children. Multiple instances may share one VM; later polls are then
    /// harmless no-ops.
    pub(crate) fn poll_script_async_work_tree(&mut self) -> Result<bool, ScriptError> {
        let handles = self
            .script_instances_by_global
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut changed = false;
        for handle in handles {
            changed |= handle.borrow_mut().poll_async_work()?;
        }
        for nested in self.nested_artboards.values_mut() {
            changed |= nested.child.poll_script_async_work_tree()?;
        }
        for list_index in 0..self.component_list_count() {
            let Some(local_id) = self.component_list_local_at(list_index) else {
                continue;
            };
            let Some(items) = self.component_list_items_mut(local_id) else {
                continue;
            };
            for item in items {
                changed |= item.child.poll_script_async_work_tree()?;
            }
        }
        Ok(changed)
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
        let mut first_error = None;
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
            match self.advance_script_component_with(entry, seconds, &mut host, &mut call_advance) {
                Ok(advanced) => did_advance |= advanced,
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
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
        // result rearms the same owner; false parks it until wakeAdvance.
        // ScriptedObject::scriptAdvance also converts a protected-call error
        // to false, so surfacing Rust's typed error must not rearm the owner
        // (`scripted_object.cpp:178-203`;
        // `scripted_drawable.cpp:376-397`;
        // `scripted_path_effect.cpp:111-130`).
        self.set_script_owner_advance_active_handle(component, false);
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let result = call_advance(
            handle.borrow_mut().as_mut(),
            &[ScriptValue::Number(f64::from(seconds))],
            host,
        )?;
        if result != ScriptValue::Bool(true) {
            return Ok(false);
        }
        self.set_script_owner_advance_active_handle(component, true);
        if entry.kind == AdvancingComponentKind::ScriptedPathEffect {
            // The scripting facade exposes an advance result but has no
            // separate `markNeedsUpdate` callback. A true path-effect advance
            // therefore publishes the ScriptUpdate that C++ consumes at the
            // effect's dependency slot before ShapePaint rebuilds its retained
            // EffectPath (`scripted_path_effect.cpp:111-132,199-207`).
            self.set_script_owner_update_pending(component, true);
            self.add_component_dirt(component, ComponentDirt::SCRIPT_UPDATE, false);
        } else {
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

    #[doc(hidden)]
    pub fn set_script_input_for_global_if_changed(
        &mut self,
        global_id: u32,
        name: &str,
        value: ScriptValue,
    ) -> Result<bool, ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        if handle.borrow_mut().get_input(name)? == value {
            return Ok(false);
        }
        self.set_script_input_for_global(global_id, name, value)?;
        Ok(true)
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

    pub(crate) fn reconcile_runtime_list_path(
        &mut self,
        file: &RuntimeFile,
        path_local: usize,
        items: Vec<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let rows = items.into_iter().map(Some).collect::<Vec<_>>();
        let reconciled = {
            let states = self.runtime_list_paths.get_mut();
            let Some(state) = states
                .iter_mut()
                .find(|state| state.path_local() == path_local)
            else {
                return false;
            };
            state.reconcile(file, Some(&rows)).is_ok()
        };
        // C++ marks Path dirt even for an identity-only reconciliation. The
        // occurrence-wide epoch covers composer/bounds/length caches while
        // the component dirt follows the ordinary Path dependency schedule.
        self.mark_runtime_list_path_dirty(path_local);
        reconciled
    }

    pub(crate) fn reject_runtime_list_path_input(
        &mut self,
        path_local: usize,
        error: crate::shapes::list_path::RuntimeListPathInputError,
    ) -> bool {
        let rejected = {
            let states = self.runtime_list_paths.get_mut();
            let Some(state) = states
                .iter_mut()
                .find(|state| state.path_local() == path_local)
            else {
                return false;
            };
            state.reject_invalid(error).is_err()
        };
        self.mark_runtime_list_path_dirty(path_local);
        rejected
    }

    pub(crate) fn flush_runtime_list_path_changes(&mut self) -> bool {
        let dirty_paths = self
            .runtime_list_paths
            .get_mut()
            .iter_mut()
            .filter_map(|state| (state.flush_live_changes() != 0).then_some(state.path_local()))
            .collect::<Vec<_>>();
        if dirty_paths.is_empty() {
            return false;
        }
        for path_local in dirty_paths {
            self.mark_runtime_list_path_dirty(path_local);
        }
        true
    }

    fn mark_runtime_list_path_dirty(&mut self, path_local: usize) {
        if !self.add_dirt(path_local, ComponentDirt::PATH, false)
            && let Some(component) = self.component(path_local)
        {
            component.bump_path_revision();
        }
        // `Path::markPathDirty` explicitly calls the containing Shape's
        // `pathChanged`; the synthetic vertices have no parent through which
        // ordinary vertex dirt could reach that render-path cache.
        let mut parent = self
            .component(path_local)
            .and_then(|component| component.parent);
        while let Some(handle) = parent {
            let Some(component) = self.objects.component(handle) else {
                break;
            };
            if component.type_name == "Shape" {
                component.bump_path_revision();
                break;
            }
            parent = component.parent;
        }
        self.mark_path_changed();
    }

    pub(crate) fn runtime_list_path_vertices(
        &self,
        path_local: usize,
    ) -> Option<Vec<nuxie_graph::PathVertexNode>> {
        self.runtime_list_paths
            .borrow_mut()
            .iter_mut()
            .find(|state| state.path_local() == path_local)
            .map(crate::shapes::list_path::RuntimeListPathState::projected_vertices)
    }

    pub fn runtime_list_path_debug_report(
        &self,
        path_local: usize,
    ) -> Option<crate::shapes::list_path::RuntimeListPathDebugReport> {
        self.runtime_list_paths
            .borrow()
            .iter()
            .find(|state| state.path_local() == path_local)
            .map(crate::shapes::list_path::RuntimeListPathState::debug_report)
    }

    pub fn components(&self) -> RuntimeComponents<'_> {
        RuntimeComponents {
            arena: &self.objects,
        }
    }

    /// Whether this concrete artboard occurrence or a mounted descendant
    /// contains an AudioEvent.
    pub fn has_audio(&self) -> bool {
        if self
            .components()
            .iter()
            .any(|component| component.type_name == "AudioEvent")
        {
            return true;
        }
        if self
            .nested_artboards
            .values()
            .any(|nested| nested.child.has_audio())
        {
            return true;
        }
        self.component_list_locals().any(|local_id| {
            self.component_list_items(local_id)
                .is_some_and(|items| items.iter().any(|item| item.child.has_audio()))
        })
    }

    /// Headless engine selected for this Artboard occurrence.
    pub fn audio_engine(&self) -> Option<crate::AudioEngine> {
        self.audio_event_playback.engine()
    }

    /// Install an external/headless engine on this complete Artboard tree.
    pub fn set_audio_engine(&mut self, engine: Option<crate::AudioEngine>) {
        self.audio_event_playback.set_engine(engine.clone());
        for nested in self.nested_artboards.values_mut() {
            nested.child.set_audio_engine(engine.clone());
        }
        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for local_id in list_locals {
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child.set_audio_engine(engine.clone());
                }
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.audio_event_playback.volume()
    }

    /// Set the Artboard audio multiplier and propagate it to mounted children.
    pub fn set_volume(&mut self, volume: f32) {
        self.audio_event_playback.set_volume(volume);
        for nested in self.nested_artboards.values_mut() {
            nested.child.set_volume(volume);
        }
        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for local_id in list_locals {
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child.set_volume(volume);
                }
            }
        }
    }

    /// Invoke the concrete event's public C++ `AudioEvent::play` counterpart.
    pub fn play_audio_event(&self, event_local_id: usize) -> Option<crate::AudioSound> {
        self.audio_event_playback.play(event_local_id)
    }

    pub(crate) fn audio_event_playback(&self) -> RuntimeAudioEventPlayback {
        self.audio_event_playback.clone()
    }

    fn inherit_audio_configuration_from(&mut self, source: &RuntimeAudioEventPlayback) {
        self.audio_event_playback.inherit_configuration_from(source);
        for nested in self.nested_artboards.values_mut() {
            nested.child.inherit_audio_configuration_from(source);
        }
        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for local_id in list_locals {
            if let Some(items) = self.component_list_items_mut(local_id) {
                for item in items {
                    item.child.inherit_audio_configuration_from(source);
                }
            }
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
        let world = self
            .runtime_graph()
            .map(|graph| self.runtime_component_world_transform(owner.local_id, graph))
            .unwrap_or(owner.transform.world_transform);
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
        if let Some(focus) = self.external_focus_domain.as_ref() {
            focus.rebuild_after_structure_change(self);
        }
    }

    fn refresh_retained_focusables_for_property(&self, local_id: usize, property_key: u16) {
        if let Some(focus) = self.external_focus_domain.as_ref() {
            focus.refresh_after_property_change(self, local_id, property_key);
        }
    }

    fn refresh_retained_focus_data(&self, focus_data_local: usize, root_transform: Mat2D) {
        if let Some(focus) = self.external_focus_domain.as_ref() {
            focus.refresh_focus_data_update(self, focus_data_local, root_transform);
        }
    }

    fn refresh_retained_focus_visibility(&self) {
        if let Some(focus) = self.external_focus_domain.as_ref() {
            focus.refresh_visibility_change(self);
        }
    }

    pub(crate) fn runtime_graph(&self) -> Option<&ArtboardGraph> {
        self.runtime_graph_for_global(self.graph_global_id)
    }

    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn debug_static_text_layout_report(
        &self,
        text_local: usize,
    ) -> Option<crate::RuntimeTextLayoutDebugReport> {
        crate::text::static_text_layout_debug_report(
            self.runtime_file()?,
            self.runtime_graph()?,
            self,
            text_local,
            None,
        )
    }

    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn debug_static_text_target_report(
        &self,
        text_local: usize,
    ) -> Option<Vec<crate::RuntimeTextTargetModifierDebugReport>> {
        Some(crate::text::static_text_target_debug_report(
            self.runtime_file()?,
            self.runtime_graph()?,
            text_local,
        ))
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

    /// Attach the compiled protocol factory resolved by a
    /// `ScriptedInterpolator.scriptAssetId` occurrence.
    #[doc(hidden)]
    pub fn set_scripted_interpolator_factory(
        &mut self,
        interpolator_global_id: u32,
        factory: RuntimeScriptedInterpolatorFactory,
    ) {
        self.scripted_interpolator_factories
            .insert(interpolator_global_id, factory);
    }

    #[doc(hidden)]
    pub fn has_scripted_interpolator_factory(&self, interpolator_global_id: u32) -> bool {
        self.scripted_interpolator_factories
            .contains_key(&interpolator_global_id)
    }

    pub(crate) fn scripted_interpolator_factory(
        &self,
        interpolator_global_id: u32,
    ) -> Option<RuntimeScriptedInterpolatorFactory> {
        self.scripted_interpolator_factories
            .get(&interpolator_global_id)
            .cloned()
    }

    pub(crate) fn evaluate_shared_scripted_interpolator(
        &self,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        arguments: &[f32],
        fallback: f32,
    ) -> f32 {
        let factory = self.scripted_interpolator_factory(interpolator_global_id);
        self.shared_scripted_interpolators.borrow_mut().evaluate(
            Some(self),
            factory.as_ref(),
            interpolator_global_id,
            key_frame_global_id,
            interpolator_global_id,
            method,
            arguments,
            fallback,
        )
    }

    /// Script initialization/callback failures from definition-level
    /// `LinearAnimation::apply`, which uses the shared authored interpolator.
    pub fn shared_scripted_interpolator_diagnostics(
        &self,
    ) -> Vec<RuntimeScriptedInterpolatorDiagnostic> {
        self.shared_scripted_interpolators.borrow().diagnostics()
    }

    /// Resolve the concrete artboard occurrence DataContext used by a lazily
    /// cloned ScriptedInterpolator table.
    #[doc(hidden)]
    pub fn scripted_interpolator_data_context_view_models(
        &self,
        file: &RuntimeFile,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> (Option<ScriptViewModel>, Vec<Option<ScriptViewModel>>) {
        if let Some(data_context) = self.artboard_owned_data_context.as_ref() {
            let mut contexts = data_context.main_context_slots(file).into_iter();
            if let Some(main) = contexts.next() {
                let main = main.and_then(|context| {
                    crate::script_view_model_from_owned_context(file, &context)
                });
                let parents = contexts
                    .map(|context| {
                        context.and_then(|context| {
                            crate::script_view_model_from_owned_context(file, &context)
                        })
                    })
                    .collect();
                return (main, parents);
            }
            return (None, Vec::new());
        }
        if let Some(context) = self.artboard_owned_view_model_handle.as_ref() {
            return (
                crate::script_view_model_from_owned_context(file, context),
                Vec::new(),
            );
        }
        if let Some(context) = self.artboard_owned_view_model_context.as_ref() {
            if let Some(main) = context.main_handle() {
                return (crate::script_view_model_from_owned(file, main), Vec::new());
            }
            return (None, Vec::new());
        }
        (
            fallback_root.and_then(|root| crate::script_view_model_from_owned(file, root)),
            Vec::new(),
        )
    }

    /// Root handle matching the DataContext above, used to hydrate scalar
    /// ScriptInput values before the first interpolation callback.
    #[doc(hidden)]
    pub fn scripted_interpolator_root_view_model(
        &self,
        file: &RuntimeFile,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> Option<RuntimeOwnedViewModelHandle> {
        if let Some(data_context) = self.artboard_owned_data_context.as_ref() {
            return data_context
                .main_context_chain(file)
                .into_iter()
                .next()
                .map(|context| context.root_handle());
        }
        if let Some(context) = self.artboard_owned_view_model_handle.as_ref() {
            return Some(context.root_handle());
        }
        if let Some(context) = self.artboard_owned_view_model_context.as_ref() {
            return context.main_handle().cloned();
        }
        fallback_root.cloned()
    }

    pub(crate) fn scripted_interpolator_owned_data_context(
        &self,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> RuntimeOwnedDataContext {
        if let Some(data_context) = self.artboard_owned_data_context.as_ref() {
            return data_context.clone();
        }
        if let Some(context) = self.artboard_owned_view_model_handle.as_ref() {
            return RuntimeOwnedDataContext::from_context_handle(context);
        }
        if let Some(context) = self.artboard_owned_view_model_context.as_ref() {
            return RuntimeOwnedDataContext::from_owned_context(context);
        }
        fallback_root
            .cloned()
            .map(RuntimeOwnedDataContext::from_root_handle)
            .unwrap_or_default()
    }

    pub fn state_machine(&self, index: usize) -> Option<&RuntimeStateMachine> {
        self.state_machines.get(index)
    }

    pub fn state_machines(&self) -> &[RuntimeStateMachine] {
        self.state_machines.as_slice()
    }

    /// Pinned C++ `defaultStateMachineIndex`: an explicit authored ordinal is
    /// valid only while it addresses this Artboard's state-machine table.
    pub fn default_state_machine_index(&self) -> Option<usize> {
        select_default_state_machine(
            property_key_for_name("Artboard", "defaultStateMachineId")
                .and_then(|key| self.uint_property(0, key)),
            self.state_machines.len(),
        )
    }

    /// Pinned C++ `defaultStateMachine`: unlike `defaultScene`, this never
    /// falls back to state machine zero.
    pub fn default_state_machine(&self) -> Option<&RuntimeStateMachine> {
        self.state_machine(self.default_state_machine_index()?)
    }

    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        let root_layout = self.component_handle(0);
        let controlled_text = self
            .components()
            .iter()
            .filter_map(|component| {
                (component.type_name == "Text"
                    && root_layout
                        .is_some_and(|layout| component.layout_ancestors.contains(&layout)))
                .then_some(component.local_id)
            })
            .collect::<Vec<_>>();
        for text_local in controlled_text {
            // A hosting-layout resize dirties TextShape but does not run
            // `Text::clearRenderStyles`; TextStylePaint keeps and rewinds its
            // existing opacity paths. Classify the pending rebuild before
            // publishing Path dirt so pooled component-list rows reuse those
            // concrete RenderPath owners (`layout_component.cpp:1116-1124`,
            // `text.cpp:1209-1230`).
            self.runtime_drawables
                .mark_text_resource_dirty_for_local(text_local);
            crate::text_owner::mark_shape_dirty_without_layout(self, text_local);
        }
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

    /// Return one layout component's retained solved border box.
    ///
    /// This reads the exact occurrence-owned result retained during layout
    /// settlement. It does not run or approximate a layout solve. The `x` and
    /// `y` values are in artboard-local coordinates; width and height are the
    /// solved dimensions exposed by pinned C++
    /// `LayoutComponent::layoutBounds/layoutWidth/layoutHeight`.
    pub fn layout_bounds(&self, local_id: usize) -> Option<RuntimeLayoutBounds> {
        self.solved_layout_bounds
            .as_deref()?
            .get(&local_id)
            .copied()
    }

    /// LayoutComponent::worldBounds in this Artboard's coordinate space.
    ///
    /// Pinned C++ `LayoutComponent::localBounds` reads the current `m_layout`
    /// frame, not the newly solved animation target retained separately by
    /// Rust. Keep semantic bounds on that live frame so `SemanticData` can
    /// apply its exact bounds-delta gate.
    pub(crate) fn layout_world_bounds(&self, local_id: usize) -> Option<(f32, f32, f32, f32)> {
        let component = self.component(local_id)?;
        if component.type_name != "LayoutComponent" {
            return None;
        }
        let (_, _, width, height) = component.concrete.layout.as_ref()?.current_bounds();
        let transform = component.transform.world_transform;
        let corners = [
            transform.transform_point(0.0, 0.0),
            transform.transform_point(width, 0.0),
            transform.transform_point(0.0, height),
            transform.transform_point(width, height),
        ];
        let (mut min_x, mut min_y) = corners[0];
        let (mut max_x, mut max_y) = corners[0];
        for (x, y) in corners.into_iter().skip(1) {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        Some((min_x, min_y, max_x, max_y))
    }

    /// Layout-derived world transform with live ancestor ScrollConstraint
    /// transforms applied. The draw cache computes the same combination;
    /// semantic/focus providers need it without creating renderer state.
    pub(crate) fn runtime_component_world_transform_with_scroll(&self, local_id: usize) -> Mat2D {
        let world = self
            .runtime_graph()
            .map(|graph| self.runtime_component_world_transform(local_id, graph))
            .or_else(|| {
                self.component(local_id)
                    .map(|component| component.transform.world_transform)
            })
            .unwrap_or(Mat2D::IDENTITY);
        self.runtime_component_apply_ancestor_scroll(local_id, world)
    }

    /// Fold the live ancestor ScrollConstraint scroll transforms for one
    /// component occurrence onto `world`, in the exact order the semantic
    /// provider and draw cache compose them.
    fn runtime_component_apply_ancestor_scroll(&self, local_id: usize, mut world: Mat2D) -> Mat2D {
        let mut current = Some(local_id);
        while let Some(ancestor_local) = current {
            if let Some(component) = self.component(ancestor_local) {
                for constraint in &component.constraints {
                    if let Some(scroll_transform) = self
                        .objects
                        .component(*constraint)
                        .and_then(|component| component.concrete.scroll.as_ref())
                        .map(|scroll| scroll.scroll_transform)
                    {
                        world = world.multiply(scroll_transform);
                    }
                }
            }
            current = self.component_parent_local(ancestor_local);
        }
        world
    }

    /// Public seam over the layout-derived world transform with live ancestor
    /// ScrollConstraint transforms applied — the combination the semantic
    /// provider and draw cache already compose internally.
    ///
    /// A ScrollConstraint owned by the queried component itself does not
    /// displace it: pinned C++ constrains the content's layout children,
    /// never the content (`scroll_constraint.cpp:215-230` at
    /// `4ac7b32798da0482e441ef09304dc3b480ed3ee5`), so the fold starts at
    /// the parent.
    ///
    /// Settles pending update dirt first so a scroll offset written this
    /// frame is reflected in the same read.
    ///
    /// Returns `None` for an unknown component occurrence.
    pub fn component_world_transform_with_scroll(&mut self, local_id: usize) -> Option<Mat2D> {
        self.update_pass();
        self.component(local_id)?;
        let world = self
            .runtime_graph()
            .map(|graph| self.runtime_component_world_transform(local_id, graph))
            .or_else(|| {
                self.component(local_id)
                    .map(|component| component.transform.world_transform)
            })
            .unwrap_or(Mat2D::IDENTITY);
        Some(match self.component_parent_local(local_id) {
            Some(parent_local) => self.runtime_component_apply_ancestor_scroll(parent_local, world),
            None => world,
        })
    }

    /// `layout_bounds` mapped through the live ancestor ScrollConstraint
    /// scroll transforms, composed exactly as
    /// [`Self::component_world_transform_with_scroll`] composes them.
    /// Identical to `layout_bounds` when no ancestor ScrollConstraint is
    /// live.
    ///
    /// The settled layout rect itself never reflects scroll in pinned C++:
    /// `ScrollConstraint::offsetY` only marks the content world transform
    /// dirty and `constrainChild` composes a world translate, leaving
    /// `layoutBounds()` untouched (`scroll_constraint.cpp:182-230` at
    /// `4ac7b32798da0482e441ef09304dc3b480ed3ee5`). Callers that need the
    /// settled box in scrolled space compose here instead of mutating the
    /// solve.
    ///
    /// A ScrollConstraint owned by the queried component itself does not
    /// displace it: pinned C++ constrains the content's layout children,
    /// never the content (`scroll_constraint.cpp:215-230`), so the fold
    /// starts at the parent.
    ///
    /// Settles pending update dirt first so a scroll offset written this
    /// frame is reflected in the same read.
    pub fn scrolled_layout_bounds(&mut self, local_id: usize) -> Option<RuntimeLayoutBounds> {
        self.update_pass();
        let layout = self.layout_bounds(local_id)?;
        // Pinned C++ post-multiplies the scroll translate onto the child's
        // world transform (`constrainChild`: `worldTransform *
        // m_scrollTransform`, `scroll_constraint.cpp:215-230`), so the
        // world-space displacement is the world linear applied to the local
        // scroll offset — under a rotated or scaled ancestor the settled box
        // shifts along the transformed axis, not the artboard axis. The
        // displacement is the translation delta between the layout-derived
        // world with and without ancestor scroll; at a zero offset it
        // vanishes and this read equals `layout_bounds` exactly.
        let world = self
            .runtime_graph()
            .map(|graph| self.runtime_component_world_transform(local_id, graph))
            .or_else(|| {
                self.component(local_id)
                    .map(|component| component.transform.world_transform)
            })
            .unwrap_or(Mat2D::IDENTITY);
        let with_scroll = match self.component_parent_local(local_id) {
            Some(parent_local) => self.runtime_component_apply_ancestor_scroll(parent_local, world),
            None => world,
        };
        Some(RuntimeLayoutBounds {
            x: layout.x + (with_scroll.0[4] - world.0[4]),
            y: layout.y + (with_scroll.0[5] - world.0[5]),
            width: layout.width,
            height: layout.height,
        })
    }

    /// Run the `FocusData::scrollIntoView` ancestor walk for one mounted
    /// focus occurrence. The owner identity disambiguates repeated nested
    /// artboards whose authored local ids are identical.
    pub(crate) fn scroll_focus_target_into_view(
        &mut self,
        owner_identity: u64,
        target_local: usize,
    ) -> bool {
        self.scroll_focus_target_path(owner_identity, target_local)
            .is_some_and(|path| path.changed)
    }

    fn scroll_focus_target_path(
        &mut self,
        owner_identity: u64,
        target_local: usize,
    ) -> Option<RuntimeFocusScrollPath> {
        if self.instance_identity() == owner_identity {
            let bounds = self.layout_world_bounds(target_local).or_else(|| {
                self.object_world_bounds(target_local)
                    .map(|bounds| (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y))
            })?;
            let changed = self.scroll_bounds_along_ancestors(target_local, bounds);
            return Some(RuntimeFocusScrollPath { bounds, changed });
        }

        let nested_hosts = self.nested_artboards.keys().copied().collect::<Vec<_>>();
        for host_local in nested_hosts {
            let host_world = self.runtime_component_world_transform_with_scroll(host_local);
            let child_path = {
                let nested = self.nested_artboards.get_mut(&host_local)?;
                let child_transform = nested.child.mounted_root_transform(host_world);
                nested
                    .child
                    .scroll_focus_target_path(owner_identity, target_local)
                    .map(|path| (path, child_transform))
            };
            if let Some((child_path, child_transform)) = child_path {
                let bounds = transform_focus_bounds(child_transform, child_path.bounds);
                let changed =
                    child_path.changed | self.scroll_bounds_along_ancestors(host_local, bounds);
                return Some(RuntimeFocusScrollPath { bounds, changed });
            }
        }

        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        let list_roots = self.runtime_component_list_child_root_transforms(Mat2D::IDENTITY);
        for list_local in list_locals {
            let item_count = self.component_list_items(list_local).map_or(0, <[_]>::len);
            for item_index in 0..item_count {
                let child_transform = list_roots
                    .get(&list_local)
                    .and_then(|roots| roots.get(item_index))
                    .copied()
                    .unwrap_or(Mat2D::IDENTITY);
                let child_path = self
                    .component_list_items_mut(list_local)
                    .and_then(|items| items.get_mut(item_index))
                    .and_then(|item| {
                        item.child
                            .scroll_focus_target_path(owner_identity, target_local)
                    });
                if let Some(child_path) = child_path {
                    let bounds = transform_focus_bounds(child_transform, child_path.bounds);
                    let changed =
                        child_path.changed | self.scroll_bounds_along_ancestors(list_local, bounds);
                    return Some(RuntimeFocusScrollPath { bounds, changed });
                }
            }
        }
        None
    }

    fn scroll_bounds_along_ancestors(
        &mut self,
        start_local: usize,
        bounds: (f32, f32, f32, f32),
    ) -> bool {
        let mut constraints = Vec::new();
        let mut current = Some(start_local);
        while let Some(local_id) = current {
            if let Some(component) = self.component(local_id) {
                constraints.extend(component.constraints.iter().copied().filter(|constraint| {
                    self.objects
                        .component(*constraint)
                        .is_some_and(|component| component.concrete.scroll.is_some())
                }));
            }
            current = self.component_parent_local(local_id);
        }
        constraints.into_iter().fold(false, |changed, constraint| {
            crate::constraints::scroll_constraint_to_show_bounds(self, constraint, bounds) | changed
        })
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

    fn after_color_property_set(
        &mut self,
        local_id: usize,
        property_key: u16,
        previous: Option<u32>,
        value: u32,
    ) -> bool {
        // Generated setters run the concrete callback before notifying
        // listeners (`gradient_stop_base.hpp`, `solid_color_base.hpp`).
        let mut owner_callback_handled = false;
        self.apply_color_property_changed(local_id, property_key, &mut owner_callback_handled);
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("SolidColor")
            && solid_color_value_property_key() == Some(property_key)
        {
            // See `SolidColor::renderOpacityChanged`: the renderer-visible
            // paint changes in place without invalidating prepared paths.
            self.did_change.set(true);
        } else {
            self.mark_changed_unless_view_model_instance(local_id);
        }
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
            // The retained paint is shared with the parent renderer
            // occurrence, so neither local nor tree preparation is dirtied.
        } else {
            self.mark_runtime_shape_property_changed(local_id);
        }
        if !owner_callback_handled {
            self.mark_prepared_changed_for_color_property(local_id, property_key, previous, value);
        }
        true
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
        // Generated setter order is backing field, concrete callback, then
        // property notification.
        let mut owner_callback_handled = false;
        self.apply_bool_property_changed(
            local_id,
            property_key,
            value,
            &mut owner_callback_handled,
        );
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed();
        self.mark_text_changed_for_local(local_id);
        if !owner_callback_handled {
            self.mark_prepared_changed_for_property(local_id, property_key);
        }
        self.mark_runtime_shape_property_changed(local_id);
        self.refresh_retained_focusables_for_property(local_id, property_key);
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
        self.set_image_override(local_id, asset_global, None)
    }

    pub(crate) fn set_image_render_override(
        &mut self,
        local_id: usize,
        image: Option<crate::RuntimeViewModelImage>,
    ) -> bool {
        self.set_image_override(local_id, None, image)
    }

    fn set_image_override(
        &mut self,
        local_id: usize,
        asset_global: Option<u32>,
        image: Option<crate::RuntimeViewModelImage>,
    ) -> bool {
        let same_asset = self.image_asset_overrides.get(&local_id) == Some(&asset_global);
        let same_image = match (self.image_render_overrides.get(&local_id), image.as_ref()) {
            (Some(current), Some(next)) => current.ptr_eq(next),
            (None, None) => true,
            _ => false,
        };
        if same_asset && same_image {
            return false;
        }
        self.image_asset_overrides.insert(local_id, asset_global);
        match image.as_ref() {
            Some(image) => {
                self.image_render_overrides.insert(local_id, image.clone());
            }
            None => {
                self.image_render_overrides.remove(&local_id);
            }
        }
        let dimensions = image
            .and_then(|image| image.render_image())
            .or_else(|| {
                asset_global.and_then(|asset_global| self.runtime_render_image(asset_global))
            })
            .map(|image| (image.width(), image.height()));
        self.runtime_images
            .set_asset(local_id, asset_global, dimensions);
        self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
        self.mark_changed();
        self.mark_prepared_changed();
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
        self.runtime_images
            .public_scale(local_id, property_key)
            .or_else(|| runtime_scroll_double_property(self, local_id, property_key))
            .or_else(|| self.objects.double_property(local_id, property_key))
    }

    #[doc(hidden)]
    pub fn debug_uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.uint_property(local_id, property_key)
    }

    #[doc(hidden)]
    pub fn debug_component_dirt(&self, local_id: usize) -> Option<ComponentDirt> {
        self.component(local_id).map(|component| component.dirt)
    }

    #[doc(hidden)]
    pub fn debug_layout_forced_size(&self, local_id: usize) -> Option<(Option<f32>, Option<f32>)> {
        self.component(local_id)?
            .concrete
            .layout
            .as_ref()
            .map(|layout| layout.forced_size())
    }

    #[doc(hidden)]
    pub fn debug_set_layout_forced_size(
        &mut self,
        local_id: usize,
        width: f32,
        height: f32,
    ) -> bool {
        let Some(layout) = self
            .component(local_id)
            .and_then(|component| component.concrete.layout.as_ref())
        else {
            return false;
        };
        let previous = layout.forced_size();
        if previous == (Some(width), Some(height)) {
            return false;
        }
        layout.forced_width(width);
        layout.forced_height(height);
        self.add_dirt(local_id, ComponentDirt::LAYOUT_STYLE, false);
        true
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
        if self.runtime_images.has_public_scale(local_id, property_key)
            && self.double_property(local_id, property_key) == Some(value)
        {
            return false;
        }
        let object_changed = self
            .objects
            .set_double_property(local_id, property_key, value);
        let image_scale_changed = self
            .runtime_images
            .mark_public_scale_written(local_id, property_key);
        if !object_changed && !image_scale_changed {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    pub(crate) fn set_int_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: i32,
    ) -> bool {
        let changed = self.objects.set_int_property(local_id, property_key, value);
        if changed {
            self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
            self.mark_changed_unless_view_model_instance(local_id);
            self.mark_prepared_changed_for_property(local_id, property_key);
            self.mark_layout_changed();
        }
        changed
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
        let mut owner_callback_handled = false;
        self.apply_double_property_changed(
            local_id,
            property_key,
            value,
            &mut owner_callback_handled,
        );
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed_unless_view_model_instance(local_id);
        self.mark_text_changed_for_local(local_id);
        if !owner_callback_handled {
            self.mark_prepared_changed_for_property(local_id, property_key);
        }
        self.mark_runtime_shape_property_changed(local_id);
        self.refresh_retained_focusables_for_property(local_id, property_key);
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
        let mut owner_callback_handled = false;
        self.apply_uint_property_changed(local_id, property_key, &mut owner_callback_handled);
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed_unless_view_model_instance(local_id);
        self.mark_text_changed_for_local(local_id);
        if !owner_callback_handled {
            self.mark_prepared_changed_for_property(local_id, property_key);
        }
        self.mark_runtime_shape_property_changed(local_id);
        self.refresh_retained_focusables_for_property(local_id, property_key);
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
        // Generated C++ setters invoke their concrete `*Changed()` callback
        // before notifying Core observers.
        self.apply_string_property_changed(local_id, property_key);
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
        self.mark_changed_unless_view_model_instance(local_id);
        self.mark_text_changed_for_local(local_id);
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.refresh_retained_focusables_for_property(local_id, property_key);
        true
    }

    #[doc(hidden)]
    pub fn debug_string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        self.string_property(local_id, property_key)
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
        LinearAnimationInstance::new(
            RuntimeLinearAnimationHandle::new(index),
            Arc::clone(&self.linear_animations),
            Arc::clone(&self.empty_linear_animation),
            speed_multiplier,
        )
    }

    pub fn advance_linear_animation_instance(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
    ) -> bool {
        instance.advance(elapsed_seconds)
    }

    pub fn advance_linear_animation_instance_with_events(
        &mut self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let (mut changed, keyed_callbacks) = {
            let Some(animation) = instance.retained_definition() else {
                return false;
            };
            if !animation.has_keyed_callbacks {
                return instance.advance(elapsed_seconds);
            }
            let mut keyed_callbacks = Vec::new();
            let changed = instance.advance_with_events(
                elapsed_seconds,
                reported_events,
                &mut keyed_callbacks,
            );
            (changed, keyed_callbacks)
        };
        for callback in keyed_callbacks {
            changed |= self.apply_keyed_callback(callback);
            if let Some(report) = crate::event::trigger_event(
                self,
                callback.target_local_id,
                callback.seconds_delay,
                None,
            ) {
                reported_events.push(report);
            }
        }
        changed
    }

    pub(crate) fn advance_linear_animation_instance_with_callback_sink(
        &mut self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        callback_sink: &mut dyn FnMut(
            &mut ArtboardInstance,
            Option<StateMachineReportedEvent>,
        ) -> bool,
    ) -> bool {
        let Some(animation) = instance.retained_definition() else {
            return false;
        };
        if !animation.has_keyed_callbacks {
            return instance.advance(elapsed_seconds);
        }

        let mut changed = false;
        let mut apply_and_deliver =
            |callback: RuntimeKeyedCallback, _event: Option<StateMachineReportedEvent>| {
                changed |= self.apply_keyed_callback(callback);
                let event = crate::event::trigger_event(
                    self,
                    callback.target_local_id,
                    callback.seconds_delay,
                    None,
                );
                changed |= callback_sink(self, event);
            };
        let keep_going =
            instance.advance_with_callback_sink(elapsed_seconds, &mut apply_and_deliver);
        changed | keep_going
    }

    pub fn apply_linear_animation_instance(
        &mut self,
        instance: &LinearAnimationInstance,
        mix: f32,
    ) -> bool {
        instance.apply(self, mix)
    }

    pub fn linear_animation_instance_keep_going(&self, instance: &LinearAnimationInstance) -> bool {
        instance.keep_going()
    }

    pub(crate) fn linear_animation_instance_definition<'a>(
        &self,
        instance: &'a LinearAnimationInstance,
    ) -> Option<&'a RuntimeLinearAnimation> {
        instance.retained_definition()
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

    /// Stage or clear one file-global view-model slot on this artboard. A
    /// valid clear against an artboard with no context is an allocation-free
    /// success, matching upstream `Artboard::setGlobalViewModelInstance`.
    #[doc(hidden)]
    pub(crate) fn set_global_view_model_instance(
        &mut self,
        file: &RuntimeFile,
        name: &str,
        instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let mut validation = RuntimeOwnedViewModelContext::default();
        let valid = match instance.as_ref() {
            Some(instance) => validation.set_global_named_handle(file, name, instance.clone()),
            None => validation.unset_global_named(file, name),
        };
        if !valid {
            return false;
        }
        if instance.is_none() && self.artboard_owned_view_model_context.is_none() {
            return true;
        }
        let context = self
            .artboard_owned_view_model_context
            .get_or_insert_with(RuntimeOwnedViewModelContext::default);
        match instance {
            Some(instance) => context.set_global_named_handle(file, name, instance),
            None => context.unset_global_named(file, name),
        }
    }

    #[doc(hidden)]
    pub(crate) fn global_view_model_instance(
        &self,
        file: &RuntimeFile,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelHandle> {
        let slot = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))?;
        self.artboard_owned_view_model_context
            .as_ref()?
            .global_slot_handle(slot)
            .cloned()
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
        instance.advance_on_artboard(self, elapsed_seconds, true, owned_context)
    }

    pub(crate) fn advance_state_machine_instance_after_state_probe(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        instance.advance_on_artboard(self, elapsed_seconds, false, None)
    }

    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn advance_state_machine_instance_after_state_probe_for_tools(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_state_machine_instance_after_state_probe(instance, elapsed_seconds)
    }

    pub(crate) fn try_change_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
    ) -> bool {
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

    pub(crate) fn state_machine_definition_owner(
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
        StateMachineInstance::advance_artboard_frame_components_with(
            self,
            instances,
            elapsed_seconds,
            None,
            |artboard, elapsed_seconds, nested_event_dispatch| {
                let mut script_mode = RuntimeScriptAdvanceMode::Disabled;
                artboard.advance_retained_components_collect_events_with_scripts(
                    elapsed_seconds,
                    true,
                    &mut script_mode,
                    None,
                    Some(nested_event_dispatch),
                )
            },
        )
        .expect("disabled script dispatch cannot fail")
    }

    pub fn advance_state_machine_instances_with_nested_and_owned_view_model_context(
        &mut self,
        instances: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        StateMachineInstance::advance_artboard_frame_components_with(
            self,
            instances,
            elapsed_seconds,
            Some(context),
            |artboard, elapsed_seconds, nested_event_dispatch| {
                let mut script_mode = RuntimeScriptAdvanceMode::Disabled;
                artboard.advance_retained_components_collect_events_with_scripts(
                    elapsed_seconds,
                    true,
                    &mut script_mode,
                    None,
                    Some(nested_event_dispatch),
                )
            },
        )
        .expect("disabled script dispatch cannot fail")
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

    /// Report mounted remap occurrence time for pinned-C++ differentials.
    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn runtime_nested_remap_animation_reports(&self) -> Vec<RuntimeNestedRemapAnimationReport> {
        self.nested_artboards
            .iter()
            .flat_map(|(host_local_id, nested)| {
                nested
                    .animations
                    .iter()
                    .filter_map(|animation| match animation {
                        RuntimeNestedAnimationInstance::Remap {
                            local_id,
                            animation,
                            ..
                        } => Some(RuntimeNestedRemapAnimationReport {
                            host_local_id: *host_local_id,
                            local_id: *local_id,
                            animation_time: animation.time(),
                        }),
                        RuntimeNestedAnimationInstance::Simple { .. }
                        | RuntimeNestedAnimationInstance::StateMachine(_) => None,
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
        StateMachineInstance::dispatch_nested_event_sources_with(
            self,
            state_machine,
            |artboard, nested_event_dispatch| {
                let _ = artboard
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        true,
                        &mut RuntimeScriptAdvanceMode::Disabled,
                        None,
                        Some(nested_event_dispatch),
                    )
                    .expect("disabled script dispatch cannot fail");
                Ok(())
            },
        )
        .expect("nested-artboard collection cannot dispatch scripts")
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
            None,
        )
    }

    /// Root C++ `Artboard::advance` settlement boundary.
    ///
    /// Like C++, this polls completed async work before any scripted or
    /// retained advancing component runs.
    pub fn advance(&mut self, elapsed_seconds: f32) -> Result<bool, ScriptError> {
        crate::scene::advance(self, elapsed_seconds)
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
            None,
        )
    }

    pub fn advance_frame_components_with_state_machine(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> Result<bool, ScriptError> {
        self.advance_frame_components_with_state_machine_report(elapsed_seconds, state_machine)
            .map(|report| report.notified)
    }

    /// Like [`Self::advance_frame_components_with_state_machine`], but also
    /// reports whether the component advance itself changed anything. The
    /// pinned `advanceAndApply` folds that result into its keep-going return
    /// (`state_machine_instance.cpp:2614-2620`), including the quantized
    /// nested-artboard force-true (`nested_artboard.cpp:983-986`); callers
    /// composing the facade bool need it alongside the notify trigger.
    pub fn advance_frame_components_with_state_machine_report(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> Result<RuntimeFrameComponentsAdvance, ScriptError> {
        let mut components_changed = false;
        let notified = StateMachineInstance::dispatch_nested_event_sources_with(
            self,
            state_machine,
            |artboard, nested_event_dispatch| {
                let mut script_mode = RuntimeScriptAdvanceMode::HostOnly;
                artboard
                    .advance_frame_components_collect_events_with_mode(
                        elapsed_seconds,
                        &mut script_mode,
                        None,
                        Some(nested_event_dispatch),
                    )
                    .map(|changed| {
                        components_changed = changed;
                    })
            },
        )?;
        Ok(RuntimeFrameComponentsAdvance {
            notified,
            changed: components_changed || notified,
        })
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
        StateMachineInstance::advance_artboard_frame_components(
            self,
            state_machines,
            elapsed_seconds,
        )
    }

    pub fn advance_frame_components_with_state_machines_and_factory(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        StateMachineInstance::advance_artboard_frame_components_with_factory(
            self,
            state_machines,
            elapsed_seconds,
            factory,
        )
    }

    pub(crate) fn advance_components_after_root_state_machines(
        &mut self,
        elapsed_seconds: f32,
        nested_event_dispatch: &mut dyn FnMut(
            &mut ArtboardInstance,
            usize,
            &[StateMachineReportedEvent],
        ) -> bool,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::HostOnly;
        self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            &mut script_mode,
            None,
            Some(nested_event_dispatch),
        )
    }

    pub(crate) fn advance_components_after_root_state_machines_with_factory(
        &mut self,
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
        nested_event_dispatch: &mut dyn FnMut(
            &mut ArtboardInstance,
            usize,
            &[StateMachineReportedEvent],
        ) -> bool,
    ) -> Result<bool, ScriptError> {
        let mut script_mode = RuntimeScriptAdvanceMode::Factory(factory);
        self.advance_frame_components_collect_events_with_mode(
            elapsed_seconds,
            &mut script_mode,
            None,
            Some(nested_event_dispatch),
        )
    }

    fn advance_frame_components_collect_events_with_mode(
        &mut self,
        elapsed_seconds: f32,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        nested_event_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
    ) -> Result<bool, ScriptError> {
        // Stateful VMI children are shared pointers in C++. Apply the host's
        // newly keyed values to Rust's detached occurrence before any nested
        // state machine consumes the child DataContext this frame
        // (`src/nested_artboard.cpp:156-185`).
        let mut changed = self.sync_stateful_nested_view_model_contexts();
        let retained_result = self.advance_retained_components_collect_events_with_scripts(
            elapsed_seconds,
            true,
            script_mode,
            nested_events,
            nested_event_dispatch,
        );
        changed |= retained_result.as_ref().copied().unwrap_or(false);
        changed |= self.advance_artboard_data_binds_with_elapsed(elapsed_seconds);
        if let Err(error) = retained_result {
            return Err(error);
        }
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
            None,
        )
        .expect("disabled script dispatch cannot fail")
    }

    fn advance_retained_components_collect_events_with_scripts(
        &mut self,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        mut nested_event_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        let mut first_script_error = None;
        if self.advancing_components.is_empty() {
            return Ok(changed);
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
        let mut script_host = NoopScriptHost;
        for index in 0..self.advancing_components.len() {
            let entry = self.advancing_components[index];
            changed |= match entry.kind {
                AdvancingComponentKind::NestedArtboard => {
                    let host_local = entry.local_id;
                    let has_dispatch = nested_event_dispatch.is_some();
                    let mut dispatch_source =
                        |artboard: &mut ArtboardInstance,
                         host_local: usize,
                         events: &[StateMachineReportedEvent]| {
                            nested_event_dispatch
                                .as_mut()
                                .is_some_and(|dispatch| (**dispatch)(artboard, host_local, events))
                        };
                    let advance_result = self.advance_nested_artboard_entry(
                        host_local,
                        elapsed_seconds,
                        new_frame,
                        script_mode,
                        layout_frame.as_ref(),
                        &mut initial_layout_paint_evaluations,
                        nested_events.as_deref_mut(),
                        has_dispatch.then_some(
                            &mut dispatch_source
                                as &mut dyn FnMut(
                                    &mut ArtboardInstance,
                                    usize,
                                    &[StateMachineReportedEvent],
                                ) -> bool,
                        ),
                    );
                    let published = self.publish_nested_view_model_context_mutations(host_local);
                    match advance_result {
                        Ok(advanced) => advanced | published,
                        Err(error) => {
                            first_script_error.get_or_insert(error);
                            published
                        }
                    }
                }
                AdvancingComponentKind::ArtboardComponentList => {
                    match self.advance_component_list_entry(
                        entry.local_id,
                        elapsed_seconds,
                        new_frame,
                        script_mode,
                    ) {
                        Ok(advanced) => advanced,
                        Err(error) => {
                            first_script_error.get_or_insert(error);
                            false
                        }
                    }
                }
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
                    match self.advance_script_component_with(
                        entry,
                        elapsed_seconds,
                        &mut script_host,
                        &mut |instance, args, host| script_mode.call(instance, args, host),
                    ) {
                        Ok(advanced) => advanced,
                        Err(error) => {
                            first_script_error.get_or_insert(error);
                            false
                        }
                    }
                }
                // Root Artboard participates in the C++ advancing interface
                // only when a concrete root owner adds work; ordinary roots
                // have no additional advance body.
                AdvancingComponentKind::Artboard => {
                    #[cfg(test)]
                    {
                        self.advance_persistent_dirt_component_fixture(entry.local_id)
                    }
                    #[cfg(not(test))]
                    {
                        false
                    }
                }
                AdvancingComponentKind::ScriptedDrawable
                | AdvancingComponentKind::ScriptedLayout
                | AdvancingComponentKind::ScriptedPathEffect => false,
            };
        }
        if let Some(error) = first_script_error {
            return Err(error);
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
        if advance.size_changed || (advance.layout_changed && !advance.keep_going) {
            self.propagate_scripted_layout_size(entry.local_id);
        }
        if advance.layout_changed {
            // `LayoutComponent::applyInterpolation` writes `m_layout`, then
            // propagates size and marks the exact owner world-transform dirty
            // (`src/layout_component.cpp:1329-1401`).
            self.add_dirt(entry.local_id, ComponentDirt::WORLD_TRANSFORM, true);
            self.layout_revision = self.layout_revision.wrapping_add(1);
            self.runtime_drawables
                .mark_layout_resource_dirty_for_local(entry.local_id);
        }
        advance.keep_going
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

    fn detach_active_nested_state_machines(&mut self, nested: &mut RuntimeNestedArtboardInstance) {
        debug_assert!(self.active_nested_state_machines.is_empty());
        for animation in &mut nested.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            let local_id = occurrence.local_id();
            if let Some(state_machine) = occurrence.take_state_machine() {
                let previous = self
                    .active_nested_state_machines
                    .insert(local_id, state_machine);
                debug_assert!(previous.is_none());
            }
        }
    }

    fn restore_active_nested_state_machines(&mut self, nested: &mut RuntimeNestedArtboardInstance) {
        for animation in &mut nested.animations {
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                continue;
            };
            if let Some(state_machine) = self
                .active_nested_state_machines
                .remove(&occurrence.local_id())
            {
                occurrence.restore_state_machine(state_machine);
            }
        }
        debug_assert!(self.active_nested_state_machines.is_empty());
    }

    fn advance_nested_artboard_entry(
        &mut self,
        host_local: usize,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
        layout_frame: Option<&RuntimeNestedLayoutBoundsFrame>,
        initial_layout_paint_evaluations: &mut BTreeMap<usize, ArtboardInstance>,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        mut nested_event_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
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

        let (keep_going, first_script_error) = if new_frame {
            let Some(mut nested) = self.nested_artboards.remove(&host_local) else {
                return Ok(false);
            };
            self.detach_active_nested_state_machines(&mut nested);
            let result = match nested.begin_advance(elapsed_seconds) {
                Err(changed) => Ok(changed),
                Ok(local_elapsed_seconds) => (|| {
                    let animation_count = nested.animations.len();
                    let mut changed = false;
                    for animation_index in 0..animation_count {
                        changed |= match nested_event_dispatch.as_mut() {
                            Some(dispatch) => {
                                StateMachineInstance::advance_nested_animation_owner_with(
                                    self,
                                    &mut nested,
                                    host_local,
                                    animation_index,
                                    local_elapsed_seconds,
                                    nested_events.as_deref_mut(),
                                    Some(&mut **dispatch),
                                )?
                            }
                            None => StateMachineInstance::advance_nested_animation_owner_with(
                                self,
                                &mut nested,
                                host_local,
                                animation_index,
                                local_elapsed_seconds,
                                nested_events.as_deref_mut(),
                                None,
                            )?,
                        };
                    }
                    changed |= match nested_event_dispatch.as_mut() {
                        Some(dispatch) => nested.advance_after_animation_owners(
                            self,
                            host_local,
                            local_elapsed_seconds,
                            script_mode,
                            nested_events.as_deref_mut(),
                            Some(&mut **dispatch),
                        )?,
                        None => nested.advance_after_animation_owners(
                            self,
                            host_local,
                            local_elapsed_seconds,
                            script_mode,
                            nested_events.as_deref_mut(),
                            None,
                        )?,
                    };
                    Ok(changed)
                })(),
            };
            self.restore_active_nested_state_machines(&mut nested);
            self.nested_artboards.insert(host_local, nested);
            match result {
                Ok(changed) => (changed, None),
                Err(error) => (false, Some(error)),
            }
        } else {
            (
                self.nested_artboards
                    .get_mut(&host_local)
                    .is_some_and(RuntimeNestedArtboardInstance::advance_outer_update),
                None,
            )
        };
        let child_dirty = self
            .nested_artboards
            .get(&host_local)
            .is_some_and(|nested| nested.child.has_dirt(ComponentDirt::COMPONENTS));
        if child_dirty {
            self.add_dirt(host_local, ComponentDirt::COMPONENTS, false);
        }
        if let Some(error) = first_script_error {
            return Err(error);
        }
        Ok(keep_going)
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

    pub(crate) fn has_self_transform(&self) -> bool {
        let authored = self.authored_transform(0);
        authored.rotation != 0.0 || authored.scale_x != 1.0 || authored.scale_y != 1.0
    }

    pub(crate) fn self_transform(&self) -> Mat2D {
        let authored = self.authored_transform(0);
        let mut transform = Mat2D::from_rotation(authored.rotation);
        transform.scale_by_values(authored.scale_x, authored.scale_y);
        transform
    }

    pub(crate) fn mounted_root_transform(&self, host_transform: Mat2D) -> Mat2D {
        host_transform.multiply(self.self_transform())
    }

    pub(crate) fn child_opacity(&self) -> f32 {
        self.component(0)
            .map(|component| component.transform.render_opacity * self.host_opacity)
            .unwrap_or(self.host_opacity)
    }

    pub(crate) fn set_host_opacity(&mut self, opacity: f32) -> bool {
        if self.host_opacity == opacity {
            return false;
        }
        self.host_opacity = opacity;
        self.add_dirt(0, ComponentDirt::RENDER_OPACITY, true);
        true
    }

    pub fn frame_id(&self) -> u64 {
        self.frame_id.get()
    }

    pub(crate) fn begin_draw_frame(&self) {
        ARTBOARD_DRAW_FRAME_ID.fetch_add(1, Ordering::Relaxed);
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

    pub(crate) fn layout_revision(&self) -> u64 {
        self.layout_revision
    }

    pub(crate) fn mark_text_shape_changed(&mut self) {
        self.text_shape_revision = self.text_shape_revision.wrapping_add(1);
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

    fn mark_changed_unless_view_model_instance(&mut self, local_id: usize) {
        if !self
            .slot(local_id)
            .and_then(|slot| slot.type_name)
            .is_some_and(|type_name| type_name.starts_with("ViewModelInstance"))
        {
            self.mark_changed();
        }
    }

    fn mark_stateful_nested_view_model_contexts_dirty_for_local(&mut self, local_id: usize) {
        if self
            .slot(local_id)
            .and_then(|slot| slot.type_name)
            .is_some_and(|type_name| type_name.starts_with("ViewModelInstance"))
        {
            self.stateful_nested_view_model_contexts_dirty = true;
            self.stateful_nested_view_model_dirty_locals
                .insert(local_id);
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
        let previous_bounds = self.layout_constraint_bounds.clone();
        let next_bounds = self.runtime_graph().and_then(|graph| {
            self.runtime_taffy_layout_bounds(graph, self.runtime_file())
                .map(Arc::new)
        });
        let resized_parametric_paths = self
            .runtime_graph()
            .map(|graph| {
                graph
                    .paths
                    .iter()
                    .filter(|path| path.parametric.is_some())
                    .filter_map(|path| {
                        let control_size =
                            |bounds: Option<&Arc<BTreeMap<usize, RuntimeLayoutBounds>>>| {
                                bounds.and_then(|bounds| {
                                    self.runtime_layout_control_size_for_path(
                                        path.local_id,
                                        bounds.as_ref(),
                                    )
                                    .map(|bounds| (bounds.width, bounds.height))
                                })
                            };
                        (control_size(previous_bounds.as_ref())
                            != control_size(next_bounds.as_ref()))
                        .then_some(path.local_id)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let resized_layout_components = self
            .components()
            .iter()
            .filter(|component| component.type_name == "LayoutComponent")
            .filter_map(|component| {
                let bounds_for = |bounds: Option<&Arc<BTreeMap<usize, RuntimeLayoutBounds>>>| {
                    bounds.and_then(|bounds| bounds.get(&component.local_id).copied())
                };
                (bounds_for(previous_bounds.as_ref()) != bounds_for(next_bounds.as_ref()))
                    .then_some(component.local_id)
            })
            .collect::<Vec<_>>();
        self.solved_layout_bounds = next_bounds.clone();
        self.layout_constraint_bounds = next_bounds;
        if let (Some(bounds), Some(graph)) = (
            self.layout_constraint_bounds.clone(),
            self.runtime_graph().cloned(),
        ) {
            self.control_runtime_layout_images(&graph, bounds.as_ref());
            self.control_runtime_layout_joysticks(&graph, bounds.as_ref());
        }
        for path_local in resized_parametric_paths {
            // C++ LayoutComponent::propagateSizeToChildren calls
            // ParametricPath::controlSize. That setter writes the solved
            // dimensions and raises both WorldTransform and Path dirt before
            // dependency settlement (`layout_component.cpp:934-967`,
            // `shapes/parametric_path.cpp:24-33`).
            self.add_dirt(
                path_local,
                ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH,
                false,
            );
        }
        self.enqueue_artboard_parametric_layout_control_sources();
        for local_id in resized_layout_components {
            self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub(crate) fn mark_layout_changed(&mut self) {
        self.layout_revision = self.layout_revision.wrapping_add(1);
        self.mark_prepared_changed();
    }

    /// Direct `LayoutComponent::markLayoutNodeDirty` publication. The revision
    /// is only a derived fence for the retained layout solve; unrelated paint
    /// preparation is not dirtied.
    pub(crate) fn mark_layout_node_changed(&mut self, local_id: usize) -> bool {
        let owner_changed = self
            .component(local_id)
            .and_then(|component| component.concrete.layout.as_ref())
            .is_some_and(|layout| layout.mark_layout_node_dirty());
        if !owner_changed {
            return false;
        }
        self.layout_revision = self.layout_revision.wrapping_add(1);
        self.mark_components_dirty();
        true
    }

    pub(crate) fn mark_layout_component_children_dirty(&mut self, local_id: usize) -> bool {
        let Some(parent) = self.component_handle(local_id) else {
            return false;
        };
        let child_count = self.component_child_len(parent);
        let mut changed = false;
        for index in 0..child_count {
            let Some(child) = self.component_child_at(parent, index) else {
                continue;
            };
            let Some(child_local) = self.component_local_id(child) else {
                continue;
            };
            if self
                .component(child_local)
                .is_some_and(|component| component.concrete.layout.is_some())
            {
                changed |= self.mark_layout_node_changed(child_local);
            }
        }
        changed
    }

    pub(crate) fn mark_component_list_override_changed(&mut self, override_local: usize) -> bool {
        let Some(list_local) = self.component_parent_local(override_local) else {
            return false;
        };
        if self
            .component(list_local)
            .is_none_or(|component| component.type_name != "ArtboardComponentList")
        {
            return false;
        }
        let artboard_id = property_key_for_name("ArtboardComponentListOverride", "artboardId")
            .and_then(|key| self.uint_property(override_local, key))
            .unwrap_or(u64::from(u32::MAX));
        let matching_items = self
            .component_list_items(list_local)
            .map(|items| {
                items
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| {
                        let item_artboard_index = item
                            .child
                            .runtime_file()
                            .zip(item.child.runtime_graph())
                            .and_then(|(file, graph)| artboard_index_for_graph(file, graph));
                        (artboard_id == u64::from(u32::MAX)
                            || usize::try_from(artboard_id).ok() == item_artboard_index)
                            .then_some(index)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if matching_items.is_empty() {
            // C++ override callbacks only walk their retained `m_artboards`.
            // With no matching mounted occurrence there is no hosted layout
            // to invalidate and no dirt bubbles to the list owner
            // (`artboard_component_list_override.cpp:7-44`).
            return false;
        }
        for index in matching_items {
            if let Some(item) = self
                .component_list_items_mut(list_local)
                .and_then(|items| items.get_mut(index))
            {
                item.child.mark_layout_node_changed(0);
            }
        }
        self.add_dirt(list_local, ComponentDirt::LAYOUT_STYLE, false)
            | self.mark_layout_node_changed(list_local)
    }

    pub(crate) fn mark_path_changed(&mut self) {
        self.path_epoch = self.path_epoch.wrapping_add(1);
        self.mark_prepared_changed();
    }

    fn mark_runtime_shape_property_changed(&mut self, local_id: usize) {
        self.runtime_shapes.mark_property_changed(local_id);
    }

    /// Direct `StrokeEffect::invalidateEffectFromLocal` callback. The concrete
    /// owner rewinds only its provider-local EffectPaths, then invalidates
    /// downstream effects through its retained parent EffectsContainer.
    pub(crate) fn invalidate_runtime_stroke_effect_from_local(&mut self, local_id: usize) -> bool {
        let paint_locals = self.runtime_shapes.invalidate_effect_from_local(local_id);
        // `StrokeEffect::invalidateEffectFromLocal` first rewinds its concrete
        // EffectPaths, then `ShapePaint::invalidateEffects` publishes Path
        // dirt on each owning paint (`stroke_effect.cpp:13-25`,
        // `shape_paint.cpp:193-205`). An unattached effect has no fallback
        // global epoch publication.
        paint_locals
            .into_iter()
            .fold(false, |changed, paint_local| {
                self.add_dirt(paint_local, ComponentDirt::PATH, false) | changed
            })
    }

    pub(crate) fn mark_points_path_skin_dirty(&mut self, path_local: usize) -> bool {
        let Some(path) = self.component_handle(path_local) else {
            return false;
        };
        let skin = self
            .objects
            .component(path)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .and_then(|skinnable| skinnable.skin);
        skin.is_some_and(|skin| self.add_component_dirt(skin, ComponentDirt::SKIN, false))
    }

    fn mark_text_changed(&mut self) {
        self.runtime_drawables.mark_text_resources_dirty();
    }

    fn mark_text_changed_for_local(&mut self, local_id: usize) {
        if !self
            .text_affecting_locals
            .get(local_id)
            .copied()
            .unwrap_or(false)
        {
            return;
        }
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return;
        };
        let mut text_local = local_id;
        let mut remaining = self.slots.len().saturating_add(1);
        while remaining != 0 {
            remaining -= 1;
            if matches!(
                self.slot(text_local).and_then(|slot| slot.type_name),
                Some("Text" | "TextInput")
            ) {
                // The concrete Text callback rebuilds only this Text's render
                // styles. In particular, TextValueRun setters do not
                // invalidate sibling Text occurrences
                // (`text_value_run.cpp:90-113`, `text.cpp:534-543`).
                if self
                    .runtime_shapes
                    .text_style_paint_container_for_component(local_id)
                    .is_some_and(|container_local| container_local != local_id)
                {
                    // ShapePaint mutators under TextStylePaint change the
                    // retained paint frame, not the glyph paths. FL-E7 routes
                    // the container's Paint dirt back to this Text during the
                    // dependency update, matching C++'s ordinary ShapePaint
                    // lifecycle without rewinding unchanged opacity paths.
                    self.runtime_drawables
                        .mark_text_resource_dirty_for_local(text_local);
                } else {
                    self.runtime_drawables
                        .mark_text_render_styles_dirty_for_local(text_local);
                }
                return;
            }
            let Some(parent_local) = self
                .objects
                .uint_property(text_local, parent_key)
                .and_then(|parent| usize::try_from(parent).ok())
            else {
                return;
            };
            if parent_local == text_local || parent_local >= self.slots.len() {
                return;
            }
            text_local = parent_local;
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

        self.runtime_meshes
            .mark_component_dirt(local_id, accumulated);
        if accumulated.contains(ComponentDirt::LAYOUT_STYLE) {
            self.mark_layout_changed();
        }
        if component_dirt_affects_path_epoch(accumulated) {
            if let Some(component) = self.component(local_id) {
                component.bump_path_revision();
            }
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
        self.settle_runtime_image_asset_updates();
        self.settle_runtime_font_asset_updates();
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

    pub(crate) fn take_semantic_bounds_dirty_locals(&mut self) -> BTreeSet<usize> {
        std::mem::take(&mut self.semantic_bounds_dirty_locals)
    }

    #[doc(hidden)]
    pub fn debug_update_pass_with_root_transform(&mut self, root_transform: Mat2D) -> bool {
        if let Some(root) = self.objects.root() {
            if let Some(component) = self.objects.component_mut(root) {
                // `Artboard::mutableWorldTransform` is retained owner state;
                // unlike a TransformComponent, the Artboard update does not
                // rebuild this matrix from authored x/y/scale properties.
                component.transform.world_transform = root_transform;
            }
            self.add_component_dirt(root, ComponentDirt::WORLD_TRANSFORM, true);
        }
        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        self.update_pass_with_script_mode(&mut script_mode, root_transform)
    }

    fn update_data_binds_for_update_pass(&mut self, root_transform: Mat2D) {
        #[cfg(test)]
        {
            self.update_pass_data_bind_call_count += 1;
        }
        self.update_nested_artboard_data_binds_from_hosts(root_transform);
        self.advance_artboard_data_binds_with_root_transform(root_transform, 0.0);
    }

    fn update_pass_with_script_mode(
        &mut self,
        script_mode: &mut RuntimeScriptUpdateMode<'_>,
        root_transform: Mat2D,
    ) -> bool {
        let image_assets_did_update = self.settle_runtime_image_asset_updates();
        let font_assets_did_update = self.settle_runtime_font_asset_updates();
        // Mirrors C++ src/artboard.cpp Artboard::updatePass: data binds run
        // before components, with artboard-host children publishing first.
        self.update_data_binds_for_update_pass(root_transform);
        // C++ transfers a NestedArtboardLayout's Yoga node after the first
        // child-recursive data-bind pass, then reuses that node whenever the
        // parent Yoga graph reports a new layout. The transfer key keeps later
        // precise child-local writes from causing a second solve in the same
        // outer update while still refreshing genuine parent assignments.
        let mut did_update = image_assets_did_update
            | font_assets_did_update
            | self.apply_nested_artboard_layout_bounds_after_parent_solve();
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
                if !self.joysticks[joystick_index].can_apply_before_update() {
                    self.update_data_binds_for_update_pass(root_transform);
                }
                if !self.joysticks[joystick_index].can_apply_before_update()
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
                did_update |= self.apply_runtime_joystick_at(joystick_index);
            }
            self.update_data_binds_for_update_pass(root_transform);
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
            // C++ keeps only the enumerated computed-target exceptions in the
            // persisting list. Poll that list explicitly after component
            // settlement; pushed Core properties remain queue-driven.
            self.update_data_binds_for_update_pass(root_transform);
        }
        let has_unsettled_component_list_rows =
            self.component_list_locals().into_iter().any(|local_id| {
                self.component_list_items(local_id).is_some_and(|items| {
                    items
                        .iter()
                        .any(|item| item.settled_layout_size.get().is_none())
                })
            });
        if (!self.suppress_mounted_component_list_layout_updates && did_update)
            || has_unsettled_component_list_rows
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

    #[cfg(test)]
    fn advance_persistent_dirt_component_fixture(&mut self, local_id: usize) -> bool {
        let Some(fixture) = self.persistent_dirt_component_fixture.as_mut() else {
            return false;
        };
        if fixture.local_id != local_id {
            return false;
        }
        fixture.advance_count += 1;
        let _ = self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, false);
        true
    }

    #[cfg(test)]
    fn update_persistent_dirt_component_fixture(&mut self, local_id: usize) {
        if let Some(fixture) = self.persistent_dirt_component_fixture.as_mut()
            && fixture.local_id == local_id
        {
            fixture.update_count += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn install_persistent_dirt_component_fixture(&mut self) {
        let root = self
            .objects
            .root()
            .expect("fixture Artboard root component");
        let root_local_id = self
            .objects
            .component(root)
            .expect("fixture Artboard root occurrence")
            .local_id;
        if !self
            .objects
            .dependency_order()
            .iter()
            .any(|&component| component == root)
        {
            let mut dependency_order = self.objects.dependency_order().to_vec();
            dependency_order.push(root);
            self.objects.set_dependency_order(dependency_order);
        }
        if !self.advancing_components.iter().any(|entry| {
            entry.local_id == root_local_id && entry.kind == AdvancingComponentKind::Artboard
        }) {
            self.advancing_components.push(RuntimeAdvancingComponent {
                local_id: root_local_id,
                object: self
                    .objects
                    .object_handle(root_local_id)
                    .expect("fixture Artboard root object"),
                component: Some(root),
                kind: AdvancingComponentKind::Artboard,
            });
        }
        self.persistent_dirt_component_fixture = Some(PersistentDirtComponentFixture {
            local_id: root_local_id,
            advance_count: 0,
            update_count: 0,
        });
    }

    #[cfg(test)]
    pub(crate) fn persistent_dirt_component_fixture_receipt(&self) -> (usize, usize, bool) {
        let fixture = self
            .persistent_dirt_component_fixture
            .as_ref()
            .expect("persistent-dirt component fixture must be installed");
        (
            fixture.advance_count,
            fixture.update_count,
            self.has_dirt(ComponentDirt::COMPONENTS),
        )
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
        StateMachineInstance::settle_artboard_update_passes(
            self,
            state_machines,
            reset_root_view_models,
            |artboard| artboard.update_pass_with_script_mode(script_mode, Mat2D::IDENTITY),
        )
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
    pub(crate) fn advance_outer_update_components_for_state_machine_settlement(&mut self) -> bool {
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
            && !dirt.contains(ComponentDirt::WORLD_TRANSFORM)
        {
            return false;
        }
        let mut changed = false;
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            changed |= self.sync_nested_artboard_root_opacity(host_local_id);
        }
        let paused_child_has_component_dirt = self
            .nested_artboards
            .get(&host_local_id)
            .is_some_and(|nested| {
                nested.is_paused && nested.child.has_dirt(ComponentDirt::COMPONENTS)
            });
        if dirt.contains(ComponentDirt::COMPONENTS) || paused_child_has_component_dirt {
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
            let host_world = self
                .component(host_local_id)
                .map(|component| component.transform.world_transform)
                .unwrap_or(Mat2D::IDENTITY);
            if let Some(nested) = self.nested_artboards.get_mut(&host_local_id) {
                let child_root_transform = nested
                    .child
                    .mounted_root_transform(root_transform.multiply(host_world));
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
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM)
            && self.runtime_object_type_name(host_local_id) == Some("NestedArtboardLeaf")
            && property_key_for_name("NestedArtboardLeaf", "fit")
                .and_then(|key| self.uint_property(host_local_id, key))
                == Some(7)
        {
            let child_dimensions = self
                .nested_artboards
                .get(&host_local_id)
                .map(|nested| nested.child.artboard_dimensions());
            let frame_dimensions = self
                .component_parent_local(host_local_id)
                .filter(|parent_local| {
                    self.runtime_object_type_name(*parent_local) == Some("LayoutComponent")
                })
                .and_then(|parent_local| self.layout_bounds(parent_local))
                .map(|bounds| (bounds.width, bounds.height))
                .or(child_dimensions);
            let child_root_transform = root_transform.multiply(
                self.component(host_local_id)
                    .map(|component| component.transform.world_transform)
                    .unwrap_or(Mat2D::IDENTITY),
            );
            if let (Some((width, height)), Some(nested)) = (
                frame_dimensions,
                self.nested_artboards.get_mut(&host_local_id),
            ) && nested.child.set_artboard_dimensions(width, height)
            {
                // The mounted instance already advanced; use one non-root
                // child update pass to reflow now or this frame would draw
                // the old layout at the new bounds.
                changed |= nested
                    .child
                    .update_pass_with_script_mode(script_mode, child_root_transform);
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
        let Some(host_opacity) = self.component(host_local_id).map(|component| {
            if component.transform.render_opacity == 0.0 && !component.dirt.is_empty() {
                // During clone construction the retained runtime opacity
                // has not run its first dependency update. C++
                // NestedArtboard::onAdded observes the authored host
                // opacity at this boundary, not Rust's zero-initialized
                // derived cache.
                self.authored_transform(host_local_id).opacity
            } else {
                component.transform.render_opacity
            }
        }) else {
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
        // Every retained Text render owner dirt transition corresponds to
        // C++ Text Path dirt. Enroll exactly those occurrences before the
        // clean-frame guard so update, never draw, owns reconstruction.
        for text_local in self.runtime_drawables.dirty_text_locals() {
            self.add_dirt(text_local, ComponentDirt::PATH, false);
        }
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
        self.solved_layout_bounds = layout_bounds.clone();
        if let Some(layout_bounds) = layout_bounds.as_deref() {
            for (&local_id, &bounds) in layout_bounds {
                if local_id == 0 && self.layout_node_owned_by_host {
                    continue;
                }
                self.retain_runtime_layout_component_bounds(local_id, bounds, Some(layout_bounds));
            }
            // The solve above can dirty a Text owner after the pre-guard
            // enrollment. Mirror `propagateSizeToChildren -> controlSize` by
            // enrolling that exact second set before component traversal.
            for text_local in self.runtime_drawables.dirty_text_locals() {
                self.add_dirt(text_local, ComponentDirt::PATH, false);
            }
            if let Some((graphs, graph_index)) = graph_owner.as_ref() {
                self.control_runtime_layout_images(&graphs[*graph_index], layout_bounds);
                self.control_runtime_layout_joysticks(&graphs[*graph_index], layout_bounds);
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
                if !(dirt & (ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH)).is_empty() {
                    self.semantic_bounds_dirty_locals.insert(local_id);
                }
                #[cfg(test)]
                self.update_persistent_dirt_component_fixture(local_id);
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
                        if dirt.contains(ComponentDirt::WORLD_TRANSFORM)
                            && self
                                .component(local_id)
                                .is_some_and(|component| component.type_name == "FocusData")
                        {
                            self.refresh_retained_focus_data(local_id, root_transform);
                        }
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
                            if self
                                .component(local_id)
                                .is_some_and(|component| component.type_name == "TextInput")
                            {
                                if !(dirt
                                    & (ComponentDirt::TEXT_SHAPE
                                        | ComponentDirt::WORLD_TRANSFORM
                                        | ComponentDirt::LAYOUT_STYLE))
                                    .is_empty()
                                {
                                    self.refresh_text_input_geometry(local_id);
                                }
                                if !(dirt
                                    & (ComponentDirt::TEXT_SHAPE
                                        | ComponentDirt::WORLD_TRANSFORM
                                        | ComponentDirt::LAYOUT_STYLE
                                        | ComponentDirt::PAINT))
                                    .is_empty()
                                {
                                    self.adjust_text_input_scroll_to_caret(local_id);
                                }
                            }
                            if self.update_runtime_text_render_styles(
                                local_id,
                                dirt,
                                &graphs[*graph_index],
                                layout_bounds.as_deref(),
                            ) && !(dirt & (ComponentDirt::TEXT_SHAPE | ComponentDirt::PAINT))
                                .is_empty()
                            {
                                // `Text::buildRenderStyles` publishes this
                                // post-bounds Node -> LayoutComponent ->
                                // Artboard callback from the mutable update
                                // phase (`text.cpp:534-826,1130-1213`).
                                crate::layout_node_provider::mark_layout_node_dirty(self, local_id);
                            }
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
                        crate::text::update_text_variation_helper(self, text, dirt);
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

    pub(crate) fn mark_components_dirty(&mut self) -> bool {
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
            crate::constraints::follow_path_constraint::update_follow_path_constraint(
                self,
                component_handle,
            );
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
                crate::constraints::constrainable_list::apply_list_constraints(
                    self,
                    component_handle,
                );
                crate::constraints::constraint::apply_constraints(self, component_handle);

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
            crate::constraints::constrainable_list::apply_list_constraints(self, component_handle);
            crate::constraints::constraint::apply_constraints(self, component_handle);
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
                    Some(if component.type_name == "Artboard" {
                        component.child_opacity(authored_opacity) * self.host_opacity
                    } else {
                        component.child_opacity(authored_opacity)
                    })
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
        self.update_runtime_joystick(component_handle, dirt);
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
            if self.joysticks[joystick_index].can_apply_before_update() == can_apply_before_update {
                changed |= self.apply_runtime_joystick_at(joystick_index);
            }
        }
        changed
    }

    pub(crate) fn apply_uint_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        owner_callback_handled: &mut bool,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        let owner_callback =
            crate::shapes::uint_property_changed(self, local_id, type_name, property_key);
        let owner_callback = owner_callback.or_else(|| {
            crate::layout_component_style::uint_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::layout_component::uint_property_changed(self, local_id, type_name, property_key)
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::artboard_component_list_override::uint_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::text_owner::uint_property_changed(self, local_id, type_name, property_key)
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::text::text_style_axis_uint_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::text::text_modifier_group_uint_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::text_value_run_owner::uint_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::draw_rules::uint_property_changed(self, local_id, type_name, property_key)
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::draw_target::uint_property_changed(self, local_id, type_name, property_key)
        });
        let owner_callback = owner_callback.or_else(|| {
            nested_artboard_leaf_uint_property_changed(self, local_id, type_name, property_key)
        });
        *owner_callback_handled = owner_callback.is_some();
        let owner_changed = owner_callback.unwrap_or(false);
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("Image")
            && let Some(value) = self.uint_property(local_id, property_key)
        {
            // Generated Image fields update in place. Pinned Image does not
            // rerun `updateImageScale` for fit writes; the next controlSize,
            // setMesh, or assetUpdated callback consumes the new value.
            self.runtime_images
                .apply_uint_property(local_id, property_key, value);
        }
        let mut changed = self
            .component(local_id)
            .and_then(|component| component.concrete.constraint)
            .is_some_and(|constraint| {
                crate::constraints::constraint::constraint_uint_change_marks_parent_dirty(
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
        changed | owner_changed
    }

    pub(crate) fn apply_bool_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
        owner_callback_handled: &mut bool,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        let owner_callback =
            crate::shapes::bool_property_changed(self, local_id, type_name, property_key);
        let owner_callback = owner_callback.or_else(|| {
            crate::layout_component_style::bool_property_changed(
                self,
                local_id,
                type_name,
                property_key,
            )
        });
        let owner_callback = owner_callback.or_else(|| {
            crate::layout_component::bool_property_changed(self, local_id, type_name, property_key)
        });
        let owner_callback = owner_callback.or_else(|| {
            (type_name == Some("TextInput")
                && property_key_for_name("TextInput", "multiline") == Some(property_key))
            .then(|| self.text_input_multiline_changed(local_id))
        });
        *owner_callback_handled = owner_callback.is_some();
        let owner_changed = owner_callback.unwrap_or(false);
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
                    return owner_changed;
                }
                self.clip = value;
                true
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
            _ => owner_changed,
        }
    }

    pub(crate) fn apply_string_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if type_name == Some("TextInput")
            && property_key_for_name("TextInput", "text") == Some(property_key)
        {
            return self.text_input_property_changed(local_id);
        }
        crate::text_value_run_owner::string_property_changed(
            self,
            local_id,
            type_name,
            property_key,
        )
        .unwrap_or(false)
    }

    pub(crate) fn apply_color_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        owner_callback_handled: &mut bool,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        let owner_callback =
            crate::shapes::color_property_changed(self, local_id, type_name, property_key);
        *owner_callback_handled = owner_callback.is_some();
        owner_callback.unwrap_or(false)
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
        owner_callback_handled: &mut bool,
    ) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        let owner_callback =
            crate::shapes::double_property_changed(self, local_id, type_name, property_key)
                .or_else(|| {
                    crate::joystick::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::layout_component_style::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::layout_component::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::artboard_component_list_override::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text_style_owner::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text_style_paint_owner::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text::text_style_axis_double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text::text_variation_modifier_double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text::text_modifier_group_double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    crate::text_owner::double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    component_origin_double_property_changed(
                        self,
                        local_id,
                        type_name,
                        property_key,
                    )
                })
                .or_else(|| {
                    (type_name == Some("TextInput")
                        && property_key_for_name("TextInput", "selectionRadius")
                            == Some(property_key))
                    .then(|| self.text_input_selection_radius_changed(local_id))
                });
        *owner_callback_handled = owner_callback.is_some();
        let owner_changed = owner_callback.unwrap_or(false);
        if type_name == Some("Image") {
            // Same generated-field ownership as C++ ImageBase. These setters
            // intentionally do not call updateImageScale themselves.
            self.runtime_images
                .apply_double_property(local_id, property_key, value);
        }
        if type_name == Some("NSlicedNode")
            && let Some(changed) =
                crate::draw::n_sliced_node::size_changed(self, local_id, property_key)
        {
            return changed;
        }
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
                crate::constraints::constraint::constraint_is_ik_strength_property(
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
                crate::constraints::constraint::constraint_double_change_marks_parent_dirty(
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
            Some("MeshVertex")
                if property_key_for_name("Vertex", "x") == Some(property_key)
                    || property_key_for_name("Vertex", "y") == Some(property_key) =>
            {
                crate::draw::mesh_vertex::geometry_changed(self, local_id)
            }
            Some("AxisX" | "AxisY")
                if property_key_for_name("Axis", "offset") == Some(property_key) =>
            {
                crate::draw::axis::offset_changed(self, local_id)
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
                // Computed scroll setters retain/resolve their intent in
                // `set_runtime_scroll_double_property`. Only a resulting
                // scrollOffset write invokes C++ `offsetX/Y` and dirties the
                // content transform; an unresolved intent or an unchanged
                // resolved offset is clean (`scroll_constraint.cpp:182-199,
                // 590-665`).
                apply_scroll_offset_changed(self, local_id, property_key, value).unwrap_or(false)
            }
            _ => owner_changed,
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

    pub(crate) fn mark_parent_gradient_dirty(
        &mut self,
        stop_local_id: usize,
        dirt: ComponentDirt,
    ) -> bool {
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
        self.add_dirt(gradient_local_id, dirt, false)
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

    fn mark_nested_artboard_layout_changed(&mut self, local_id: usize) -> bool {
        let local_changed = self
            .component(local_id)
            .is_some_and(|component| component.concrete.layout.is_some())
            && self.mark_layout_node_changed(local_id);
        crate::layout_node_provider::mark_layout_node_dirty(self, local_id) | local_changed
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
            // Keep the owned occurrence alive through the hosting artboard's
            // layout cleanup. This is the Rust ownership twin of leaving
            // `m_host` attached while C++ destroys `m_Instance`: teardown may
            // use that host to unregister pending layout work.
            let removed = self.nested_artboards.remove(&local_id);
            let changed = removed.is_some();
            if changed {
                self.remove_nested_artboard_local(local_id);
                self.mark_nested_structure_changed();
                self.mark_nested_artboard_layout_changed(local_id);
                self.stateful_nested_view_model_contexts_dirty = true;
                self.mark_changed();
                self.mark_prepared_changed();
            }
            drop(removed);
            return changed;
        }
        let Some(mut nested) = self.runtime_nested_artboard_instance_for_id(local_id, value) else {
            return false;
        };
        if !bindable_artboard_requires_replacement(
            self.nested_artboards
                .get(&local_id)
                .map(|existing| existing.child.graph_global_id),
            nested.child.graph_global_id,
            force,
        ) {
            return false;
        }
        if let Some(existing) = self.nested_artboards.get(&local_id) {
            nested.reuse_owned_stateful_view_model_context(existing);
        }
        if let Some(parent_focus) = self.external_focus_domain.as_ref() {
            nested.install_external_focus_domain(parent_focus);
        }
        nested.render_cache_revision = self.nested_artboards.get(&local_id).map_or(0, |existing| {
            if existing.child.graph_global_id == nested.child.graph_global_id {
                existing.render_cache_revision.saturating_add(1)
            } else {
                0
            }
        });
        // Pinned C++ completes replacement VMI selection and binds the mounted
        // child plus its NestedStateMachine before the replacement operation
        // returns (`src/nested_artboard.cpp:228-350`). Keep the detached Rust
        // occurrence private until that replacement binding is complete too.
        if let Some(file) = self.runtime_file_arc() {
            self.rebind_owned_view_model_context_after_nested_artboard_swap(
                &file,
                local_id,
                &mut nested,
            );
        }
        self.nested_artboards.insert(local_id, nested);
        self.insert_nested_artboard_local(local_id);
        self.mark_nested_structure_changed();
        self.mark_nested_artboard_layout_changed(local_id);
        self.stateful_nested_view_model_contexts_dirty = true;
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
        let (data_bind_path_ids, data_bind_path_is_relative) = self
            .slot(host_local_id)
            .and_then(|host| context.file.object(host.source_global_id as usize))
            .map(|host_object| referencer_data_bind_path(&context.file, host_object))
            .unwrap_or((None, false));
        let mut visiting = BTreeSet::new();
        visiting.insert(self.graph_global_id);
        let mut nested = build_runtime_nested_artboard_instance(
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
            &self.profile_path,
        )
        .ok()?;
        nested
            .child
            .inherit_audio_configuration_from(&self.audio_event_playback);
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
        self.active_nested_state_machines
            .get(&state_machine_local_id)
            .or_else(|| {
                self.nested_artboards
                    .state_machine(state_machine_local_id)
                    .and_then(RuntimeNestedStateMachineInstance::state_machine)
            })
    }

    pub(crate) fn nested_state_machine_mut(
        &mut self,
        state_machine_local_id: usize,
    ) -> Option<&mut StateMachineInstance> {
        if self
            .active_nested_state_machines
            .contains_key(&state_machine_local_id)
        {
            return self
                .active_nested_state_machines
                .get_mut(&state_machine_local_id);
        }
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

    pub(crate) fn advance_nested_remap_animation(&mut self, remap_local_id: usize) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.advance_remap(remap_local_id))
    }

    pub(crate) fn collapse_component_tree(&mut self, local_id: usize, collapsed: bool) -> bool {
        let changed = self.collapse_component_tree_with_ancestor(local_id, collapsed, false);
        if changed {
            self.refresh_retained_focus_visibility();
        }
        changed
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

#[derive(Debug, Clone, Copy)]
struct RuntimeFocusScrollPath {
    bounds: (f32, f32, f32, f32),
    changed: bool,
}

fn transform_focus_bounds(transform: Mat2D, bounds: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    // Pinned FocusData::scrollIntoView maps exactly the AABB minimum and
    // maximum while crossing an Artboard boundary. SemanticProvider uses the
    // separate four-corner rootTransformAABB path.
    let min = transform.transform_point(bounds.0, bounds.1);
    let max = transform.transform_point(bounds.2, bounds.3);
    (min.0, min.1, max.0, max.1)
}

impl ArtboardInstance {
    pub(crate) fn install_external_focus_domain(
        &mut self,
        parent_focus: &crate::focus::RuntimeFocusTree,
    ) {
        let next = parent_focus.external_for_owner(self.instance_identity());
        if let Some(current) = self.external_focus_domain.take() {
            if current.shares_manager(&next) {
                self.external_focus_domain = Some(current);
            } else {
                current.cleanup_focus_tree(self);
                // `NestedArtboard::syncNestedFocusTree` re-homes under the
                // recorded placement as the final write after a manager
                // switch (`src/nested_artboard.cpp:369-376`).
                next.sync_mounted_focus_tree(self);
                self.external_focus_domain = Some(next);
            }
        } else {
            next.sync_mounted_focus_tree(self);
            self.external_focus_domain = Some(next);
        }
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
        item.child
            .reset_retained_components_for_state_machine_settlement();
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
}

fn component_list_profile_path(
    parent_path: &[crate::ProfilePathSegment],
    child_name: &str,
    component_list_name: &str,
    logical_index: usize,
) -> Vec<crate::ProfilePathSegment> {
    let mut path = parent_path.to_vec();
    path.push(crate::ProfilePathSegment::nested_artboard(child_name));
    path.push(crate::ProfilePathSegment::component_list(
        component_list_name,
        i32::try_from(logical_index).unwrap_or(i32::MAX),
    ));
    path
}

fn build_runtime_nested_artboard_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    visiting: &mut BTreeSet<u32>,
    build_context: Option<RuntimeArtboardBuildContext>,
    parent_profile_path: &[crate::ProfilePathSegment],
) -> Result<RuntimeNestedArtboards> {
    if artboards.is_empty() {
        return Ok(RuntimeNestedArtboards::default());
    }

    let mut nested_artboards = RuntimeNestedArtboards::default();
    for host in &graph.nested_artboards {
        if !is_nested_artboard_occurrence_type(host.type_name) {
            continue;
        }

        let Some(host_object) = file.object(host.global_id as usize) else {
            continue;
        };
        let (data_bind_path_ids, data_bind_path_is_relative) =
            referencer_data_bind_path(file, host_object);
        let Some(child_graph) =
            resolved_artboard_graph_for_referencer(file, artboards, host_object)
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
            parent_profile_path,
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
    parent_profile_path: &[crate::ProfilePathSegment],
) -> Result<RuntimeNestedArtboardInstance> {
    let mut profile_path = parent_profile_path.to_vec();
    profile_path.push(crate::ProfilePathSegment::nested_artboard(
        child_graph.name.clone().unwrap_or_default(),
    ));
    let host_name = parent_slots
        .iter()
        .find(|slot| slot.local_id == host_local_id)
        .and_then(|slot| slot.name.clone())
        .unwrap_or_default();
    profile_path.push(crate::ProfilePathSegment::nested_artboard(host_name));
    let mut child = Box::new(ArtboardInstance::from_graph_inner(
        file,
        child_graph,
        artboards,
        visiting,
        build_context,
        false,
        profile_path,
    )?);
    apply_nested_artboard_origin_override(parent_objects, host_local_id, &mut child);
    child.set_frame_origin(false);
    child.added_to_host();
    child.bind_default_view_model_artboard_list_context(file);
    if !child_has_state_machine_data_binds(file, child_graph) {
        child.clear_default_text_property_context();
    }
    // C++ initializes the authored nested animation occurrences before
    // `onAddedClean` discovers the active stateful VMI
    // (`src/nested_artboard.cpp:570-620`). The occurrence is not allowed to
    // consume its default context during construction; the child-first bind
    // below supplies the context once the complete occurrence exists.
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
    // onAddedClean retains the first authored standard child VMI as the
    // active local root; tryScheduleBindStateful then binds that exact pointer
    // whenever the mounted instance is present (`nested_artboard.cpp:567-621,
    // 156-180`). `isStateful` controls replacement/default creation, not this
    // authored-child bind.
    let stateful_view_model_instance_local = child_view_model_index
        .and_then(|view_model_index| u32::try_from(view_model_index).ok())
        .and_then(|view_model_id| {
            data_bind_view_model_instance_locals_by_id
                .get(&view_model_id)
                .copied()
        });
    let stateful_view_model_context = if let Some(local_id) = stateful_view_model_instance_local {
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
    } else if is_stateful {
        child_view_model_index.and_then(|view_model_index| {
            RuntimeOwnedViewModelInstance::from_instance(file, view_model_index, 0)
                .or_else(|| RuntimeOwnedViewModelInstance::new(file, view_model_index))
        })
    } else {
        None
    }
    .map(RuntimeOwnedViewModelHandle::new);
    let stateful_global_view_model_contexts: BTreeMap<usize, RuntimeOwnedViewModelHandle> =
        data_bind_view_model_instance_locals_by_id
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
                Some((view_model_index, RuntimeOwnedViewModelHandle::new(context)))
            })
            .collect();
    let mut local_handles = Vec::new();
    if let Some(context) = stateful_view_model_context.clone() {
        local_handles.push(context);
    }
    local_handles.extend(stateful_global_view_model_contexts.values().cloned());
    let initial_data_context = if local_handles.is_empty() {
        child.artboard_owned_data_context.clone()
    } else {
        Some(RuntimeOwnedDataContext::with_local_handles(
            local_handles,
            None,
        ))
    };
    let data_bind_source_locals_by_path = build_nested_host_data_bind_source_locals(
        parent_slots,
        parent_objects,
        host_local_id,
        &data_bind_view_model_instance_locals_by_id,
        &child,
    );
    let (data_bind_property_source_locals, data_bind_image_source_locals) =
        build_nested_host_data_bind_source_local_slots(&child, &data_bind_source_locals_by_path);
    let mut nested = RuntimeNestedArtboardInstance {
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
    };
    if let Some(data_context) = initial_data_context.as_ref() {
        // `NestedArtboard::bindStateful` binds the active local/global VMI
        // list to the mounted child before its NestedStateMachine consumes the
        // child's DataContext (`src/nested_artboard.cpp:156-185`).
        nested.bind_owned_view_model_occurrence_data_context(file, data_context, true);
    }
    Ok(nested)
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
    !(dirt & (ComponentDirt::PATH | ComponentDirt::VERTICES | ComponentDirt::N_SLICER)).is_empty()
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
    include!("artboard/tests.rs");
}
