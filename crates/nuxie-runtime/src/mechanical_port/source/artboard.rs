use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::{AdvancingComponent, AdvancingComponentHandle},
    animation::{
        keyed_object::{KeyedObject, KeyedObjectContext},
        linear_animation::{LinearAnimation, LinearAnimationArtboard},
        linear_animation_instance::LinearAnimationInstance,
        nested_state_machine::NestedStateMachine,
        state_machine::StateMachine,
        state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
    },
    artboard_component_list::ArtboardComponentList,
    artboard_host::ArtboardHost,
    audio::audio_engine::AudioEngineRef,
    component::Component,
    component_dirt::ComponentDirt,
    core::field_types::core_callback_type::{CallbackContext, CallbackData},
    core::{CoreArena, CoreHandle, binary_reader::BinaryReader},
    core_context::CoreContext,
    data_bind::{
        data_bind::DataBind,
        data_bind_container::DataBindContainer,
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    draw_rules::DrawRules,
    draw_target::DrawTarget,
    draw_target_placement::DrawTargetPlacement,
    drawable::{Drawable, RuntimeDrawableOccurrence},
    factory::RuntimeFactoryHandle,
    file::RuntimeFileWeakHandle,
    focus_data::FocusData,
    generated::core_registry::{CoreCapabilities, CoreRegistry},
    generated::{
        artboard_base::{ArtboardBase, ArtboardBaseCallbacks},
        layout_component_base::LayoutComponentBase,
    },
    hit_info::HitInfo,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    input::{focus_manager::RuntimeFocusManagerHandle, focus_node::FocusNodeRef},
    joystick::Joystick,
    layout::layout_style_applier::LayoutParentStyleSnapshot,
    layout_component::LayoutComponent,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    math::{aabb::Aabb, mat2d::Mat2D, path_types::PathDirection, raw_path::RawPath, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    renderer::{RenderPath, Renderer},
    resetting_component::ResettingComponent,
    semantic::{
        semantic_data::SemanticData,
        semantic_manager::RuntimeSemanticManagerHandle,
        semantic_node::{SemanticNode, SemanticNodeRef},
    },
    shapes::{clipping_shape::ClippingShape, paint::shape_paint::ShapePaint, shape::Shape},
    status_code::StatusCode,
    text::text_value_run::TextValueRun,
    viewmodel::viewmodel_instance::ViewModelInstance,
};

#[cfg(feature = "tools")]
pub type ArtboardCallback = fn(*mut ());
#[cfg(feature = "tools")]
pub type TestBoundsCallback = fn(*mut (), f32, f32, bool) -> u8;
#[cfg(feature = "tools")]
pub type IsAncestorCallback = fn(*mut (), u16) -> u8;
#[cfg(feature = "tools")]
pub type RootTransformCallback = fn(*mut (), f32, f32, bool) -> f32;

struct ArtboardDirtyState {
    depth: Cell<u32>,
    dirt: Cell<ComponentDirt>,
    did_change: Cell<bool>,
    host: RefCell<Option<ArtboardHostAttachment>>,
}

/// One mount relation, shared with the dirty state rather than duplicated in
/// Artboard. Each native host's parentArtboard is its Component::artboard,
/// fixed at onAddedDirty; mounting/onAddedClean installs that exact identity.
struct ArtboardHostAttachment {
    host: CoreHandle,
    parent_artboard: Option<CoreHandle>,
}
/// The actual Artboard dirty fields live separately from its mutable geometry,
/// so synchronous property callbacks can dirty the root currently being set.
#[derive(Clone)]
pub struct RuntimeArtboardDirtyHandle(Rc<ArtboardDirtyState>);
impl Default for RuntimeArtboardDirtyHandle {
    fn default() -> Self {
        Self(Rc::new(ArtboardDirtyState {
            depth: Cell::new(0),
            dirt: Cell::new(ComponentDirt::FILTHY),
            did_change: Cell::new(true),
            host: RefCell::new(None),
        }))
    }
}
impl RuntimeArtboardDirtyHandle {
    pub fn changed(&self) {
        // Artboard::changed: guard, set, then notify the actual parent. Neither
        // the geometry root nor its host needs to be borrowed for this callback.
        if self.0.did_change.replace(true) {
            return;
        }
        let parent = self
            .0
            .host
            .borrow()
            .as_ref()
            .and_then(|attachment| attachment.parent_artboard.clone());
        if let Some(parent) = parent {
            if let Some(dirty) = parent.artboard_dirty_handle() {
                dirty.changed();
            }
        }
    }

    pub fn on_component_dirty_at(&self, graph_order: u32) {
        self.0.did_change.set(true);
        self.mark_components_dirty();
        if graph_order < self.0.depth.get() {
            self.0.depth.set(graph_order);
        }
    }
    pub fn mark_components_dirty(&self) {
        self.0
            .dirt
            .set(self.0.dirt.get() | ComponentDirt::COMPONENTS);
    }
    pub fn has_component_dirt(&self) -> bool {
        self.0.dirt.get().contains(ComponentDirt::COMPONENTS)
    }
}

pub struct Artboard {
    pub base: ArtboardBase,
    core_arena: CoreArena,
    definition_owner: Option<CoreArena>,
    objects: Vec<Option<CoreHandle>>,
    invalid_objects: Vec<Option<CoreHandle>>,
    animations: Vec<CoreHandle>,
    state_machines: Vec<CoreHandle>,
    dependency_order: Vec<crate::mechanical_port::source::component::ComponentOccurrenceHandle>,
    drawables: Vec<RuntimeDrawableOccurrence>,
    clipping_shapes: Vec<CoreHandle>,
    draw_targets: Vec<CoreHandle>,
    nested_artboards: Vec<CoreHandle>,
    component_lists: Vec<CoreHandle>,
    artboard_hosts: Vec<CoreHandle>,
    joysticks: Vec<CoreHandle>,
    resettables: Vec<CoreHandle>,
    scripted_objects: Vec<CoreHandle>,
    advancing_components: Vec<AdvancingComponentHandle>,
    pub(crate) data_bind_container: DataBindContainer,
    data_context: Option<RuntimeDataContextHandle>,
    scripting_vm: Option<RuntimeScriptingVmHandle>,
    file: RuntimeFileWeakHandle,
    joysticks_apply_before_update: bool,
    dirty_state: RuntimeArtboardDirtyHandle,
    factory: Option<RuntimeFactoryHandle>,
    first_drawable: Option<RuntimeDrawableOccurrence>,
    is_instance: bool,
    frame_origin: bool,
    dirty_layout: HashSet<CoreHandle>,
    is_cleaning_dirty_layouts: bool,
    owned_inherited_interpolator: Option<
        Box<crate::mechanical_port::source::animation::keyframe_interpolator::KeyFrameInterpolator>,
    >,
    original_width: f32,
    original_height: f32,
    updates_own_layout: bool,
    host_transform_marked_dirty: bool,
    active_focus_manager: Option<RuntimeFocusManagerHandle>,
    active_semantic_manager: Option<RuntimeSemanticManagerHandle>,
    semantic_boundary_node: Option<SemanticNodeRef>,
    #[cfg(feature = "tools")]
    external_parent_focus_node: Option<FocusNodeRef>,
    draw_order_change_counter: u8,
    #[cfg(feature = "tools")]
    artboard_id: u16,
    artboard_source: Option<CoreHandle>,
    runtime_self: RuntimeArtboardInstanceWeakHandle,
    audio_engine: Option<AudioEngineRef>,
    volume: f32,
    host_opacity: f32,
    #[cfg(feature = "tools")]
    layout_changed_callback: Option<ArtboardCallback>,
    #[cfg(feature = "tools")]
    layout_dirty_callback: Option<ArtboardCallback>,
    #[cfg(feature = "tools")]
    transform_dirty_callback: Option<ArtboardCallback>,
    #[cfg(feature = "tools")]
    test_bounds_callback: Option<TestBoundsCallback>,
    #[cfg(feature = "tools")]
    is_ancestor_callback: Option<IsAncestorCallback>,
    #[cfg(feature = "tools")]
    root_transform_callback: Option<RootTransformCallback>,
    #[cfg(feature = "tools")]
    pub callback_user_data: *mut (),
}

impl Default for Artboard {
    fn default() -> Self {
        let mut base = ArtboardBase::default();
        base.base.set_clip(true);
        Self {
            base,
            core_arena: CoreArena::default(),
            definition_owner: None,
            objects: Vec::new(),
            invalid_objects: Vec::new(),
            animations: Vec::new(),
            state_machines: Vec::new(),
            dependency_order: Vec::new(),
            drawables: Vec::new(),
            clipping_shapes: Vec::new(),
            draw_targets: Vec::new(),
            nested_artboards: Vec::new(),
            component_lists: Vec::new(),
            artboard_hosts: Vec::new(),
            joysticks: Vec::new(),
            resettables: Vec::new(),
            scripted_objects: Vec::new(),
            advancing_components: Vec::new(),
            data_bind_container: DataBindContainer::default(),
            data_context: None,
            scripting_vm: None,
            file: RuntimeFileWeakHandle::default(),
            joysticks_apply_before_update: true,
            dirty_state: RuntimeArtboardDirtyHandle::default(),
            factory: None,
            first_drawable: None,
            is_instance: false,
            frame_origin: true,
            dirty_layout: HashSet::new(),
            is_cleaning_dirty_layouts: false,
            owned_inherited_interpolator: None,
            original_width: 0.0,
            original_height: 0.0,
            updates_own_layout: true,
            host_transform_marked_dirty: false,
            active_focus_manager: None,
            active_semantic_manager: None,
            semantic_boundary_node: None,
            #[cfg(feature = "tools")]
            external_parent_focus_node: None,
            draw_order_change_counter: 0,
            #[cfg(feature = "tools")]
            artboard_id: 0,
            artboard_source: None,
            runtime_self: RuntimeArtboardInstanceWeakHandle::default(),
            audio_engine: None,
            volume: 1.0,
            host_opacity: 1.0,
            #[cfg(feature = "tools")]
            layout_changed_callback: None,
            #[cfg(feature = "tools")]
            layout_dirty_callback: None,
            #[cfg(feature = "tools")]
            transform_dirty_callback: None,
            #[cfg(feature = "tools")]
            test_bounds_callback: None,
            #[cfg(feature = "tools")]
            is_ancestor_callback: None,
            #[cfg(feature = "tools")]
            root_transform_callback: None,
            #[cfg(feature = "tools")]
            callback_user_data: std::ptr::null_mut(),
        }
    }
}

fn can_continue(code: StatusCode) -> bool {
    code != StatusCode::InvalidObject
}

// Initialization resolves the fixed imported object table without borrowing
// the Artboard whose virtual lifecycle method is currently running.
struct ArtboardObjectContext {
    arena: CoreArena,
    objects: Vec<Option<CoreHandle>>,
}
impl CoreContext for ArtboardObjectContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize).cloned().flatten()
    }
}
impl KeyedObjectContext for ArtboardObjectContext {
    fn resolves_object(&self, id: u32) -> bool {
        self.resolve_handle(id).is_some()
    }
    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
        self.resolve_handle(id)
    }
    fn object_supports_property(&self, id: u32, key: u32) -> bool {
        self.resolve_handle(id)
            .and_then(|object| {
                object.with(|object| CoreRegistry::object_supports_property(object, key))
            })
            .unwrap_or(false)
    }
    fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool {
        object
            .with_mut(|object| {
                object
                    .overrides_keyed_interpolation(key as i32)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}
impl LinearAnimationArtboard for ArtboardObjectContext {
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn crate::mechanical_port::source::animation::interpolating_keyframe::KeyFrameValueContext>,
    ) {
        object.with_downcast_mut::<KeyedObject, _>(|object| object.apply(self, time, mix, context));
    }
}

// Runtime callbacks receive the live Artboard in pinned C++. Resolve its
// object table on demand instead of copying every handle before each callback.
// The root borrow is released before the resolved occurrence is invoked, so
// keyed properties and joysticks remain free to synchronously dirty the same
// Artboard.
struct RuntimeArtboardObjectContext {
    arena: CoreArena,
    root: CoreHandle,
}

impl CoreContext for RuntimeArtboardObjectContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }

    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.root
            .with_downcast::<Artboard, _>(|artboard| artboard.resolve_handle(id))
            .flatten()
    }
}

impl KeyedObjectContext for RuntimeArtboardObjectContext {
    fn resolves_object(&self, id: u32) -> bool {
        self.resolve_handle(id).is_some()
    }

    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
        self.resolve_handle(id)
    }

    fn object_supports_property(&self, id: u32, key: u32) -> bool {
        self.resolve_handle(id)
            .and_then(|object| {
                object.with(|object| CoreRegistry::object_supports_property(object, key))
            })
            .unwrap_or(false)
    }

    fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool {
        object
            .with_mut(|object| {
                object
                    .overrides_keyed_interpolation(key as i32)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

impl LinearAnimationArtboard for RuntimeArtboardObjectContext {
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn crate::mechanical_port::source::animation::interpolating_keyframe::KeyFrameValueContext>,
    ) {
        object.with_downcast_mut::<KeyedObject, _>(|object| object.apply(self, time, mix, context));
    }
}

impl Artboard {
    pub fn new() -> Box<Self> {
        let mut artboard = Box::new(Self::default());
        #[cfg(feature = "tools")]
        {
            // SAFETY: the tools callback ABI retains this address only for the lifetime of the
            // boxed Artboard allocated above; Artboard is never moved out of that Box.
            artboard.callback_user_data = (&mut *artboard as *mut Artboard).cast();
        }
        artboard
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn with_factory(factory: RuntimeFactoryHandle) -> Self {
        let mut artboard = Self::default();
        artboard.factory = Some(factory);
        artboard.base.base.set_clip(true);
        artboard
    }

    pub fn frame_id() -> u64 {
        nuxie_render_api::artboard_draw_frame_id()
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn inc_frame_id() {
        nuxie_render_api::increment_artboard_draw_frame_id();
    }

    pub fn set_active_focus_manager(&mut self, manager: Option<RuntimeFocusManagerHandle>) {
        self.active_focus_manager = manager;
    }

    pub fn focus_manager(&self) -> Option<RuntimeFocusManagerHandle> {
        self.active_focus_manager.clone()
    }

    pub fn focus_manager_handle(&self) -> Option<RuntimeFocusManagerHandle> {
        self.focus_manager()
    }

    pub fn set_active_semantic_manager(&mut self, manager: Option<RuntimeSemanticManagerHandle>) {
        self.active_semantic_manager = manager;
    }

    pub fn semantic_manager(&self) -> Option<RuntimeSemanticManagerHandle> {
        self.active_semantic_manager.clone()
    }

    pub fn semantic_manager_handle(&self) -> Option<RuntimeSemanticManagerHandle> {
        self.semantic_manager()
    }

    pub fn semantic_boundary_node(&self) -> Option<SemanticNodeRef> {
        self.semantic_boundary_node.clone()
    }

    pub fn shape_world_transform(&self) -> Mat2D {
        self.world_transform()
    }

    pub fn virtualizable_component(&mut self) -> &mut Component {
        self.base
            .base
            .as_component_mut()
            .expect("Artboard Component")
    }

    pub fn updates_own_layout(&self) -> bool {
        self.updates_own_layout
    }
    pub fn take_layout_data(
        &mut self,
    ) -> &mut crate::mechanical_port::source::layout::layout_data::LayoutData {
        self.updates_own_layout = false;
        &mut self.base.base.layout_data
    }

    pub fn did_change(&self) -> bool {
        self.dirty_state.0.did_change.get()
    }
    pub fn dirty_handle(&self) -> RuntimeArtboardDirtyHandle {
        self.dirty_state.clone()
    }

    pub fn core_arena(&self) -> &CoreArena {
        &self.core_arena
    }

    pub fn set_core_arena(&mut self, arena: CoreArena) {
        self.core_arena = arena.weak_handle();
    }

    pub fn objects(&self) -> &[Option<CoreHandle>] {
        &self.objects
    }

    /// Tool observation of the actual scheduled owners, including runtime helpers.
    #[cfg(feature = "tools")]
    pub fn dependency_order(
        &self,
    ) -> &[crate::mechanical_port::source::component::ComponentOccurrenceHandle] {
        &self.dependency_order
    }

    pub fn objects_typed<T: crate::mechanical_port::source::core::CoreType>(
        &self,
    ) -> crate::mechanical_port::source::typed_children::TypedChildren<'_, T> {
        crate::mechanical_port::source::typed_children::TypedChildren::new(&self.objects)
    }

    pub fn animation_handles(&self) -> &[CoreHandle] {
        &self.animations
    }

    pub fn state_machine_handles(&self) -> &[CoreHandle] {
        &self.state_machines
    }

    pub fn nested_artboards(&self) -> Vec<CoreHandle> {
        self.nested_artboards.clone()
    }

    pub fn artboard_component_lists(&self) -> Vec<CoreHandle> {
        self.component_lists.clone()
    }

    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.data_context.clone()
    }

    pub fn data_bind_handles(&self) -> Vec<CoreHandle> {
        self.data_bind_container.data_binds()
    }

    pub fn add_data_bind(&mut self, bind: CoreHandle) {
        self.data_bind_container.add_data_bind(bind);
    }

    pub fn set_scripting_vm(&mut self, value: Option<RuntimeScriptingVmHandle>) {
        self.scripting_vm = value;
    }

    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = file;
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    pub fn original_width(&self) -> f32 {
        self.original_width
    }

    pub fn original_height(&self) -> f32 {
        self.original_height
    }

    pub fn reset_size(&mut self) {
        self.set_width(self.original_width);
        self.set_height(self.original_height);
    }

    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn state_machine_count(&self) -> usize {
        self.state_machines.len()
    }

    pub fn first_animation(&self) -> Option<CoreHandle> {
        self.animation_handle_at(0)
    }

    pub fn first_state_machine(&self) -> Option<CoreHandle> {
        self.state_machine_handle_at(0)
    }

    pub fn is_instance(&self) -> bool {
        self.is_instance
    }

    pub fn frame_origin(&self) -> bool {
        self.frame_origin
    }

    pub fn host_opacity(&self) -> f32 {
        self.host_opacity
    }

    pub fn child_opacity(&self) -> f32 {
        self.render_opacity() * self.host_opacity
    }

    pub fn has_self_transform(&self) -> bool {
        self.rotation() != 0.0 || self.scale_x() != 1.0 || self.scale_y() != 1.0
    }

    pub fn self_transform(&self) -> Mat2D {
        let mut transform = Mat2D::from_rotation(self.rotation());
        transform.scale_by_values(self.scale_x(), self.scale_y());
        transform
    }

    pub fn draw_order_change_counter(&self) -> u8 {
        self.draw_order_change_counter
    }

    pub fn first_drawable(&self) -> Option<RuntimeDrawableOccurrence> {
        self.first_drawable.clone()
    }

    pub fn owned_inherited_interpolator(
        &mut self,
    ) -> &mut Option<
        Box<crate::mechanical_port::source::animation::keyframe_interpolator::KeyFrameInterpolator>,
    > {
        &mut self.owned_inherited_interpolator
    }

    pub fn factory(&self) -> Option<RuntimeFactoryHandle> {
        self.factory.clone()
    }

    pub fn set_factory(&mut self, factory: RuntimeFactoryHandle) {
        self.factory = Some(factory);
    }

    pub fn artboard_source_handle(&self) -> Option<CoreHandle> {
        if self.is_instance {
            self.artboard_source.clone()
        } else {
            crate::mechanical_port::source::core::CoreObject::core(self).handle()
        }
    }

    pub fn runtime_weak_handle(&self) -> RuntimeArtboardInstanceWeakHandle {
        self.runtime_self.clone()
    }

    pub fn set_artboard_source(&mut self, artboard: Option<CoreHandle>) {
        self.artboard_source = artboard;
    }

    #[cfg(feature = "tools")]
    pub fn set_artboard_id(&mut self, id: u16) {
        self.artboard_id = id;
    }

    #[cfg(feature = "tools")]
    pub fn artboard_id(&self) -> u16 {
        self.artboard_id
    }

    pub fn added_to_host(&mut self) {
        self.base.base.just_added_to_host = true;
    }

    // TESTING exposes addObject in the pinned header. This is only the live
    // object-list append: callers still perform the explicit source lifecycle.
    #[cfg(feature = "tools")]
    pub fn add_object(&mut self, object: Option<CoreHandle>) {
        self.objects.push(object);
    }

    #[cfg(not(feature = "tools"))]
    pub(crate) fn add_object(&mut self, object: Option<CoreHandle>) {
        self.objects.push(object);
    }

    pub(crate) fn add_animation(&mut self, object: CoreHandle) {
        self.animations.push(object);
    }

    pub(crate) fn add_state_machine(&mut self, object: CoreHandle) {
        self.state_machines.push(object);
    }

    pub fn add_scripted_object(&mut self, object: CoreHandle) {
        self.scripted_objects.push(object);
    }

    pub fn validate_objects(&mut self) -> bool {
        let size = self.objects.len();
        let mut valid = vec![false; size];
        for _cycle in 0..100 {
            let mut changed = false;
            for (i, validity) in valid.iter_mut().enumerate().take(size).skip(1) {
                let Some(object) = self.objects[i].clone() else {
                    continue;
                };
                let was_valid = *validity;
                let is_valid = object
                    .with_mut(|object| object.validate(self))
                    .unwrap_or(false);
                if was_valid != is_valid {
                    changed = true;
                    *validity = is_valid;
                }
            }
            if changed {
                for (i, is_valid) in valid.iter().copied().enumerate().take(size).skip(1) {
                    if is_valid {
                        continue;
                    }
                    self.invalid_objects.push(self.objects[i].clone());
                    self.objects[i] = None;
                }
            } else {
                break;
            }
        }
        true
    }

    pub fn initialize_handle(root: &CoreHandle) -> StatusCode {
        let Some((arena, objects, animations, state_machines, is_instance)) = root
            .with_downcast::<Artboard, _>(|artboard| {
                (
                    artboard.core_arena.clone(),
                    artboard.objects.clone(),
                    artboard.animations.clone(),
                    artboard.state_machines.clone(),
                    artboard.is_instance,
                )
            })
        else {
            return StatusCode::MissingObject;
        };
        let mut context = ArtboardObjectContext {
            arena,
            objects: objects.clone(),
        };
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard
                .base
                .base
                .set_layout(0.0, 0.0, artboard.width(), artboard.height());
            artboard.mark_layout_dirty(root.clone());
        });
        let mut drawables = Vec::new();
        let mut clipping_shapes = Vec::new();

        for object in objects.clone().into_iter().flatten() {
            let code = object
                .with_mut(|object| object.on_added_dirty(&mut context))
                .unwrap_or(StatusCode::MissingObject);
            if !can_continue(code) {
                return code;
            }
        }

        if !is_instance {
            for animation in animations.clone() {
                let code = animation
                    .with_downcast_mut::<LinearAnimation, _>(|animation| {
                        animation.on_added_dirty(&mut context)
                    })
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in state_machines.clone() {
                let code = state_machine
                    .with_mut(|state_machine| state_machine.on_added_dirty(&mut context))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            if animations.is_empty() && state_machines.is_empty() {
                let owner = root;
                let mut state_machine = StateMachine::default();
                state_machine.set_name("Auto Generated State Machine".into());
                let Some(state_machine) = owner.insert_sibling(state_machine) else {
                    return StatusCode::MissingObject;
                };
                root.with_downcast_mut::<Artboard, _>(|artboard| {
                    artboard.state_machines.push(state_machine)
                });
            }
        }

        let mut component_draw_rules = HashMap::<CoreHandle, CoreHandle>::new();
        for object in objects.clone().into_iter().flatten() {
            let code = if &object == root {
                let code = LayoutComponent::on_added_clean_occurrence(root, &mut context);
                if code == StatusCode::Ok {
                    root.with_downcast_mut::<Artboard, _>(|artboard| {
                        artboard.base.base.set_x(0.0);
                        artboard.base.base.set_y(0.0);
                    });
                }
                code
            } else if object.is_type_of(NestedArtboard::TYPE_KEY) {
                NestedArtboard::on_added_clean_occurrence(&object, &mut context)
            } else if object.is_type_of(crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::TYPE_KEY) {
                crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::on_added_clean_occurrence(&object, &mut context)
            } else {
                object
                    .with_mut(|object| object.on_added_clean(&mut context))
                    .unwrap_or(StatusCode::MissingObject)
            };
            if !can_continue(code) {
                return code;
            }
            if object
                .with(|object| object.is_resetting_component())
                .unwrap_or(false)
            {
                root.with_downcast_mut::<Artboard, _>(|artboard| {
                    artboard.resettables.push(object.clone())
                });
            }
            if object.is_type_of(crate::mechanical_port::source::generated::draw_rules_base::DrawRulesBase::TYPE_KEY) {
                let parent_id = object
                    .with_downcast::<DrawRules, _>(|rules| rules.base.parent_id())
                    .unwrap_or(u32::MAX);
                if let Some(component) = context.resolve_handle(parent_id) {
                    component_draw_rules.insert(component, object.clone());
                } else {
                    eprintln!(
                        "Artboard::initialize - Draw rule targets missing component width id {}",
                        parent_id
                    );
                }
            } else if object.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY) {
                root.with_downcast_mut::<Artboard, _>(|artboard| artboard.nested_artboards.push(object.clone()));
                root.with_downcast_mut::<Artboard, _>(|artboard| artboard.artboard_hosts.push(object.clone()));
            } else if object.is_type_of(crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY) {
                root.with_downcast_mut::<Artboard, _>(|artboard| artboard.component_lists.push(object.clone()));
                root.with_downcast_mut::<Artboard, _>(|artboard| artboard.artboard_hosts.push(object.clone()));
            } else if object.is_type_of(crate::mechanical_port::source::generated::joystick_base::JoystickBase::TYPE_KEY) {
                let can_apply_before = object
                    .with_downcast_mut::<Joystick, _>(|joystick| {
                        let can_apply = joystick.can_apply_before_update();
                        joystick.add_dependents(&context);
                        can_apply
                    })
                    .unwrap_or(true);
                if !can_apply_before {
                    root.with_downcast_mut::<Artboard, _>(|artboard| artboard.joysticks_apply_before_update = false);
                }
                root.with_downcast_mut::<Artboard, _>(|artboard| artboard.joysticks.push(object.clone()));
            }
            if object
                .with(|object| object.is_advancing_component())
                .unwrap_or(false)
            {
                let advancing_component = AdvancingComponentHandle::classified(&object);
                root.with_downcast_mut::<Artboard, _>(|artboard| {
                    artboard.advancing_components.push(advancing_component)
                });
            }
        }

        if !is_instance {
            for animation in animations.clone() {
                let code = animation
                    .with_downcast_mut::<LinearAnimation, _>(|animation| {
                        animation.on_added_clean(&mut context)
                    })
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in state_machines.clone() {
                let code = state_machine
                    .with_mut(|state_machine| state_machine.on_added_clean(&mut context))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
        }

        for object in objects.clone().into_iter().flatten() {
            object.with_mut(|object| {
                object.component_build_dependencies();
            });
            let is_drawable = object
                .with(|object| object.as_drawable().is_some())
                .unwrap_or(false);
            if is_drawable && root != &object {
                drawables.push(RuntimeDrawableOccurrence::authored(object.clone()));
                if object.is_type_of(
                    crate::mechanical_port::source::generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase::TYPE_KEY,
                ) {
                    let parent = object
                        .with(|object| object.component_parent_handle())
                        .flatten();
                    let mut index = drawables.len() - 1;
                    while index >= 1 {
                        let swapping = drawables[index - 1].authored_handle();
                        drawables.swap(index - 1, index);
                        if parent == swapping {
                            break;
                        }
                        index -= 1;
                    }
                }
                let mut current = Some(object.clone());
                let mut flattened = None;
                while let Some(component) = current {
                    if let Some(rules) = component_draw_rules.get(&component) {
                        flattened = Some(rules.clone());
                        break;
                    }
                    current = component
                        .with(|component| component.component_parent_handle())
                        .flatten();
                }
                object.with_mut(|object| {
                    if let Some(drawable) = object.as_drawable_mut() {
                        drawable.flattened_draw_rules = flattened;
                    }
                });
            } else if object
                .with(|object| object.as_clipping_shape().is_some())
                .unwrap_or(false)
            {
                clipping_shapes.push(object);
            }
        }

        let mut layouts = Vec::<CoreHandle>::new();
        let mut i = 0;
        while i < drawables.len() {
            let drawable = drawables[i].clone();
            let mut current_layout = layouts.last().cloned();
            let in_current_layout = current_layout.as_ref().is_none_or(|layout| {
                drawable
                    .with(|drawable| drawable.is_child_of_layout(layout))
                    .unwrap_or(false)
            });
            if current_layout.is_some() && !in_current_layout {
                loop {
                    let layout = current_layout.take().unwrap();
                    let proxy = layout
                        .with_mut(|layout| {
                            layout
                                .as_layout_component_mut()
                                .and_then(LayoutComponent::proxy)
                        })
                        .flatten();
                    if let Some(proxy) = proxy {
                        drawables.insert(i, proxy);
                    }
                    i += 1;
                    layouts.pop();
                    current_layout = layouts.last().cloned();
                    if current_layout.is_none()
                        || current_layout.as_ref().is_some_and(|layout| {
                            drawable
                                .with(|drawable| drawable.is_child_of_layout(layout))
                                .unwrap_or(false)
                        })
                    {
                        break;
                    }
                }
            }
            if let Some(layout) = drawable.authored_handle().filter(|layout| {
                layout
                    .with(|layout| layout.as_layout_component().is_some())
                    .unwrap_or(false)
            }) {
                layouts.push(layout);
            }
            i += 1;
        }
        while let Some(layout) = layouts.pop() {
            if let Some(proxy) = layout
                .with_mut(|layout| {
                    layout
                        .as_layout_component_mut()
                        .and_then(LayoutComponent::proxy)
                })
                .flatten()
            {
                drawables.push(proxy);
            }
        }

        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.drawables = drawables;
            artboard.clipping_shapes = clipping_shapes;
            artboard.sort_dependencies();
        });
        let rules_list: Vec<CoreHandle> = objects
            .iter()
            .flatten()
            .filter_map(|object| component_draw_rules.get(object).cloned())
            .collect();
        let mut draw_target_roots = Vec::new();
        for rules in rules_list {
            let children = rules
                .with(|rules| {
                    rules
                        .as_container_component()
                        .map(|rules| rules.children().to_vec())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            for target in children {
                if !target.is_type_of(
                    crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
                ) {
                    continue;
                }
                if !draw_target_roots.contains(&target) {
                    draw_target_roots.push(target.clone());
                }
                let dependent_rules = target
                    .with_downcast::<DrawTarget, _>(DrawTarget::drawable)
                    .flatten()
                    .and_then(|drawable| {
                        drawable
                            .with(|drawable| {
                                drawable
                                    .as_drawable()
                                    .and_then(|drawable| drawable.flattened_draw_rules.clone())
                            })
                            .flatten()
                    });
                if let Some(dependent_rules) = dependent_rules {
                    for dependent_target in objects.iter().flatten() {
                        if dependent_target.is_type_of(
                            crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
                        ) && dependent_target
                            .with(|target| target.component_parent_handle())
                            .flatten()
                            .as_ref()
                            == Some(&dependent_rules)
                        {
                            dependent_target.with_mut(|dependent_target| {
                                dependent_target.component_add_dependent(target.clone())
                            });
                        }
                    }
                }
            }
        }
        let mut draw_target_order = Vec::new();
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default().sort_roots(
            draw_target_roots.into_iter().map(Into::into).collect(),
            &mut draw_target_order,
        );
        root.with_downcast_mut::<Artboard, _>(|artboard| artboard.draw_targets.extend(
            draw_target_order
                .into_iter()
                .filter_map(|target| target.authored().cloned())
                .filter(|target| {
                    target.is_type_of(
                        crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
                    )
                }),
        ));
        Self::init_scripted_objects_handle(root);
        StatusCode::Ok
    }

    fn sort_draw_order(&mut self) {
        self.draw_order_change_counter = if self.draw_order_change_counter == u8::MAX {
            0
        } else {
            self.draw_order_change_counter + 1
        };
        for target in &self.draw_targets {
            target.with_downcast_mut::<DrawTarget, _>(|target| {
                target.first = None;
                target.last = None;
            });
        }

        self.first_drawable = None;
        let mut last_drawable = None::<RuntimeDrawableOccurrence>;
        for drawable in self.drawables.iter().cloned() {
            let active_target = drawable
                .with(|drawable| drawable.flattened_draw_rules.clone())
                .flatten()
                .and_then(|rules| {
                    rules
                        .with_downcast::<DrawRules, _>(DrawRules::active_target)
                        .flatten()
                });
            if let Some(target_handle) = active_target {
                let target_last = target_handle
                    .with_downcast::<DrawTarget, _>(DrawTarget::last)
                    .flatten();
                if let Some(target_last) = target_last {
                    target_last.with_mut(|last| {
                        last.next = Some(drawable.downgrade());
                    });
                    drawable.with_mut(|drawable_base| {
                        drawable_base.prev = Some(target_last.downgrade());
                        drawable_base.next = None;
                    });
                    target_handle.with_downcast_mut::<DrawTarget, _>(|target| {
                        target.last = Some(drawable.clone());
                    });
                } else {
                    drawable.with_mut(|drawable_base| {
                        drawable_base.prev = None;
                        drawable_base.next = None;
                    });
                    target_handle.with_downcast_mut::<DrawTarget, _>(|target| {
                        target.first = Some(drawable.clone());
                        target.last = Some(drawable.clone());
                    });
                }
            } else {
                drawable.with_mut(|drawable_base| {
                    drawable_base.prev = last_drawable.as_ref().map(|last| last.downgrade());
                    drawable_base.next = None;
                });
                if let Some(last) = last_drawable.as_ref() {
                    last.with_mut(|last_base| {
                        last_base.next = Some(drawable.downgrade());
                    });
                } else {
                    self.first_drawable = Some(drawable.clone());
                }
                last_drawable = Some(drawable);
            }
        }

        for rule_handle in &self.draw_targets {
            let Some((first, last, target_drawable, placement)) = rule_handle
                .with_downcast::<DrawTarget, _>(|rule| {
                    Some((
                        rule.first()?,
                        rule.last()?,
                        RuntimeDrawableOccurrence::authored(rule.drawable()?),
                        rule.placement(),
                    ))
                })
                .flatten()
            else {
                continue;
            };
            match placement {
                DrawTargetPlacement::Before => {
                    let previous = target_drawable.with(Drawable::prev_drawable).flatten();
                    if let Some(previous) = previous {
                        previous.with_mut(|previous| {
                            previous.next = Some(first.downgrade());
                        });
                        first.with_mut(|first| {
                            first.prev = Some(previous.downgrade());
                        });
                    }
                    if self
                        .first_drawable
                        .as_ref()
                        .is_some_and(|value| value.ptr_eq(&target_drawable))
                    {
                        self.first_drawable = Some(first.clone());
                    }
                    target_drawable.with_mut(|target| {
                        target.prev = Some(last.downgrade());
                    });
                    last.with_mut(|last| {
                        last.next = Some(target_drawable.downgrade());
                    });
                }
                DrawTargetPlacement::After => {
                    let next = target_drawable.with(Drawable::next_drawable).flatten();
                    if let Some(next) = next {
                        next.with_mut(|next| {
                            next.prev = Some(last.downgrade());
                        });
                        last.with_mut(|last| {
                            last.next = Some(next.downgrade());
                        });
                    }
                    if last_drawable
                        .as_ref()
                        .is_some_and(|value| value.ptr_eq(&target_drawable))
                    {
                        last_drawable = Some(last.clone());
                    }
                    target_drawable.with_mut(|target| {
                        target.next = Some(first.downgrade());
                    });
                    first.with_mut(|first| {
                        first.prev = Some(target_drawable.downgrade());
                    });
                }
            }
        }

        self.first_drawable = last_drawable;
        for clipping_shape in &self.clipping_shapes {
            clipping_shape.with_downcast_mut::<ClippingShape, _>(ClippingShape::reset_drawables);
        }

        let create_clipping_proxy = |clipping_shape: &CoreHandle, is_start: bool| {
            let mut operation: Box<
                dyn crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeOperation,
            > = if is_start {
                Box::new(crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeStart::default())
            } else {
                Box::new(crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeEnd::default())
            };
            operation.set_clipping_shape(clipping_shape.clone());
            clipping_shape
                .with_downcast_mut::<ClippingShape, _>(|shape| {
                    shape.create_proxy_drawable(operation)
                })
                .flatten()
        };

        let mut current_drawable = self.first_drawable.clone();
        let mut next_drawable = None::<RuntimeDrawableOccurrence>;
        let mut clipping_stack = Vec::<CoreHandle>::new();
        while let Some(current) = current_drawable {
            let drawable_clipping_shapes = current
                .with_mut(|current| {
                    current.set_needs_save_operation(true);
                    current.clipping_shapes().to_vec()
                })
                .unwrap_or_default();
            let mut removing_index = clipping_stack.len();
            for (i, clipping) in clipping_stack.iter().enumerate() {
                if !drawable_clipping_shapes.contains(clipping) {
                    removing_index = i;
                    break;
                }
            }
            if !clipping_stack.is_empty() && removing_index < clipping_stack.len() {
                let mut i = clipping_stack.len() - 1;
                loop {
                    let clipping_shape = &clipping_stack[i];
                    let Some(proxy) = create_clipping_proxy(clipping_shape, false) else {
                        break;
                    };
                    if let Some(next) = next_drawable.as_ref() {
                        proxy.with_mut(|proxy| {
                            proxy.next = Some(next.downgrade());
                        });
                        next.with_mut(|next| {
                            next.prev = Some(proxy.downgrade());
                        });
                    } else {
                        eprintln!("Error - adding clip end as first operation");
                    }
                    proxy.with_mut(|proxy| {
                        proxy.prev = Some(current.downgrade());
                    });
                    current.with_mut(|current| {
                        current.next = Some(proxy.downgrade());
                    });
                    next_drawable = Some(proxy);
                    if i == removing_index || i == 0 {
                        break;
                    }
                    i -= 1;
                }
                clipping_stack.truncate(removing_index);
            }
            for clipping_shape in drawable_clipping_shapes {
                if !clipping_stack.contains(&clipping_shape) {
                    let Some(proxy) = create_clipping_proxy(&clipping_shape, true) else {
                        continue;
                    };
                    if let Some(next) = next_drawable.as_ref() {
                        proxy.with_mut(|proxy| {
                            proxy.next = Some(next.downgrade());
                        });
                        next.with_mut(|next| {
                            next.prev = Some(proxy.downgrade());
                        });
                    } else {
                        self.first_drawable = Some(proxy.clone());
                    }
                    proxy.with_mut(|proxy| {
                        proxy.prev = Some(current.downgrade());
                    });
                    current.with_mut(|current| {
                        current.next = Some(proxy.downgrade());
                    });
                    next_drawable = Some(proxy);
                    clipping_stack.push(clipping_shape);
                }
            }
            next_drawable = Some(current.clone());
            current_drawable = current.with(Drawable::prev_drawable).flatten();
        }
        if !clipping_stack.is_empty() {
            for i in (0..clipping_stack.len()).rev() {
                let Some(proxy) = create_clipping_proxy(&clipping_stack[i], false) else {
                    continue;
                };
                if let Some(next) = next_drawable.as_ref() {
                    next.with_mut(|next| {
                        next.prev = Some(proxy.downgrade());
                    });
                    proxy.with_mut(|proxy| {
                        proxy.next = Some(next.downgrade());
                    });
                }
                proxy.with_mut(|proxy| proxy.prev = None);
                next_drawable = Some(proxy);
            }
        }
        self.clear_redundant_operations();
    }

    fn clear_redundant_operations(&mut self) {
        let mut current_drawable = self.first_drawable.clone();
        let mut previous_applied_save = false;
        let mut applied_clipping_save_operations = Vec::<bool>::new();
        while let Some(current) = current_drawable {
            let previous = current
                .with_mut(|drawable| {
                    drawable.set_needs_save_operation(true);
                    drawable.prev_drawable()
                })
                .expect("draw-order occurrence always resolves");
            let is_clip_start = current.is_clip_start();
            let is_clip_end = current.is_clip_end();
            let will_clip = current.will_clip();
            if previous_applied_save {
                if is_clip_start {
                    applied_clipping_save_operations.push(false);
                    current.with_mut(|drawable| drawable.set_needs_save_operation(false));
                } else if is_clip_end {
                    let applied = applied_clipping_save_operations
                        .pop()
                        .expect("clip end has matching clip start");
                    current.with_mut(|drawable| drawable.set_needs_save_operation(applied));
                } else if previous
                    .as_ref()
                    .is_some_and(|previous| previous.is_clip_end())
                {
                    current.with_mut(|drawable| drawable.set_needs_save_operation(false));
                }
            } else if is_clip_start {
                applied_clipping_save_operations.push(true);
            } else if is_clip_end {
                let applied = applied_clipping_save_operations
                    .pop()
                    .expect("clip end has matching clip start");
                current.with_mut(|drawable| drawable.set_needs_save_operation(applied));
            }
            previous_applied_save = is_clip_start && (will_clip || previous_applied_save);
            current_drawable = previous;
        }
        assert!(applied_clipping_save_operations.is_empty());
    }

    fn sort_dependencies(&mut self) {
        self.dependency_order.clear();
        let Some(root) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        let dependents = crate::mechanical_port::source::generated::core_registry::CoreCapabilities::as_component(self)
            .expect("Artboard Component").dependents_snapshot();
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default()
            .sort_with_root_dependents(root.clone().into(), dependents, &mut self.dependency_order);
        for (graph_order, component) in self.dependency_order.clone().into_iter().enumerate() {
            if component.authored() == Some(&root) {
                crate::mechanical_port::source::generated::core_registry::CoreCapabilities::component_set_graph_order(self, graph_order as u32);
                continue;
            }
            component.with_component_mut(|component| {
                component.set_graph_order(graph_order as u32);
            });
        }
        self.dirty_state.mark_components_dirty();
    }

    pub(crate) fn init_scripted_objects_handle(root: &CoreHandle) {
        use crate::mechanical_port::source::scripted::scripted_object::{
            ScriptUpdateRequestHost, ScriptedObject,
        };
        let Some(objects) = root
            .with_downcast::<Artboard, _>(|artboard| {
                artboard
                    .is_instance
                    .then(|| artboard.scripted_objects.clone())
            })
            .flatten()
        else {
            return;
        };
        for object in objects {
            let Some(needs_init) = object
                .with(|owner| {
                    let scripted = owner.as_scripted_object()?;
                    scripted.script_asset()?;
                    Some(!scripted.user_lua_init_done())
                })
                .flatten()
            else {
                continue;
            };
            let properties = ScriptedObject::custom_properties(&object);
            let mut host = ScriptUpdateRequestHost::default();
            if needs_init {
                ScriptedObject::initialize_occurrence(&object, &properties, &mut host);
            }
            ScriptedObject::hydrate_occurrence(&object, &properties, &mut host);
            if host.take_requested() {
                ScriptedObject::apply_update_request(&object);
            }
        }
    }

    pub fn poll_async_work(&mut self) {
        crate::mechanical_port::source::r#async::work_pool::rive_poll_async_work();
        if let Some(vm) = &self.scripting_vm {
            let _ = vm.poll_async_work();
        }
    }

    pub fn poll_async_work_handle(root: &CoreHandle) {
        crate::mechanical_port::source::r#async::work_pool::rive_poll_async_work();
        let vm = root
            .with_downcast::<Artboard, _>(|artboard| artboard.scripting_vm.clone())
            .flatten();
        if let Some(vm) = vm {
            let _ = vm.poll_async_work();
        }
    }

    pub fn advance_scripted_view_models(&mut self) {
        if let Some(vm) = &self.scripting_vm {
            vm.with_vm_mut(|vm| {
                vm.advance_detached_view_models();
            });
        }
    }

    pub fn advance_scripted_view_models_handle(root: &CoreHandle) -> bool {
        let vm = root
            .with_downcast::<Artboard, _>(|artboard| artboard.scripting_vm.clone())
            .flatten();
        vm.is_some_and(|vm| vm.with_vm_mut(|vm| vm.advance_detached_view_models()))
    }

    pub fn internal_draw_canvases_handle(root: &CoreHandle) {
        let (object_count, host_count, factory) = root
            .with_downcast::<Artboard, _>(|artboard| {
                (
                    artboard.scripted_objects.len(),
                    artboard.artboard_hosts.len(),
                    artboard.factory(),
                )
            })
            .expect("live Artboard canvas pass");
        if let Some(factory) = factory {
            for index in 0..object_count {
                let object = root
                    .with_downcast::<Artboard, _>(|artboard| {
                        artboard.scripted_objects.get(index).cloned()
                    })
                    .flatten()
                    .expect("scripted object array remains stable during canvas drawing");
                factory.with_factory_mut(|factory| crate::mechanical_port::source::scripted::scripted_object::ScriptedObject::draw_canvas_occurrence(&object, factory));
            }
        }
        for host_index in 0..host_count {
            let host = root
                .with_downcast::<Artboard, _>(|artboard| {
                    artboard.artboard_hosts.get(host_index).cloned()
                })
                .flatten()
                .expect("Artboard host array remains stable during canvas drawing");
            let nested_count = host
                .with(|host| host.as_artboard_host().map(ArtboardHost::artboard_count))
                .flatten()
                .unwrap_or(0);
            for nested_index in 0..nested_count {
                let instance = host
                    .with_mut(|host| {
                        host.as_artboard_host_mut()?
                            .artboard_instance(nested_index as i32)
                    })
                    .flatten();
                if let Some(instance) = instance {
                    instance.internal_draw_canvases();
                }
            }
        }
    }

    pub fn find_draw_canvas_luau_state(&self) -> Option<RuntimeScriptingVmHandle> {
        for object in &self.scripted_objects {
            let state = object
                .with(|object| {
                    object.as_scripted_object().and_then(|object| {
                        object
                            .draws_canvas()
                            .then(|| object.scripting_vm())
                            .flatten()
                    })
                })
                .flatten();
            if state.is_some() {
                return state;
            }
        }
        for host in &self.artboard_hosts {
            let state = host
                .with_mut(|host| {
                    let host = host.as_artboard_host_mut()?;
                    (0..host.artboard_count() as i32).find_map(|index| {
                        host.artboard_instance(index).and_then(|nested| {
                            nested.with_artboard(|nested| nested.find_draw_canvas_luau_state())
                        })
                    })
                })
                .flatten();
            if state.is_some() {
                return state;
            }
        }
        None
    }

    pub fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize)?.clone()
    }

    pub fn id_of(&self, object: &CoreHandle) -> u32 {
        self.objects
            .iter()
            .position(|candidate| {
                candidate
                    .as_ref()
                    .is_some_and(|candidate| candidate == object)
            })
            .map_or(0, |index| index as u32)
    }

    pub fn on_component_dirty(&mut self, component: &Component) {
        self.on_component_dirty_at(component.graph_order());
    }

    pub fn on_component_dirty_at(&mut self, graph_order: u32) {
        self.dirty_state.on_component_dirty_at(graph_order);
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {
        self.dirty_state.mark_components_dirty();
    }

    pub fn has_component_dirt(&self) -> bool {
        self.dirty_state.has_component_dirt()
    }

    pub fn propagate_size(&mut self) {
        self.add_dirt(ComponentDirt::PATH, false);
        if self.shares_layout_with_host() {
            if let Some(host) = self.host() {
                host.with_mut(|host| {
                    if let Some(host) = host.as_artboard_host_mut() {
                        host.mark_host_transform_dirty();
                    }
                });
            }
        }
        #[cfg(feature = "tools")]
        if let Some(callback) = self.layout_changed_callback {
            callback(self.callback_user_data);
        }
    }

    pub fn propagate_size_handle(root: &CoreHandle) {
        root.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
        let host = root
            .with_downcast::<Artboard, _>(|artboard| {
                if artboard.shares_layout_with_host() {
                    artboard.host()
                } else {
                    None
                }
            })
            .flatten();
        if let Some(host) = host {
            host.with_mut(|object| {
                object
                    .as_artboard_host_mut()
                    .expect("Artboard retained its actual host")
                    .mark_host_transform_dirty();
            });
        }
        #[cfg(feature = "tools")]
        if let Some((callback, user_data)) = root
            .with_downcast::<Artboard, _>(|artboard| {
                artboard
                    .layout_changed_callback
                    .map(|callback| (callback, artboard.callback_user_data))
            })
            .flatten()
        {
            callback(user_data);
        }
    }

    fn shares_layout_with_host(&self) -> bool {
        self.host().is_some_and(|host| {
            // The pinned virtual has a constant false default and only these
            // two constant-true overrides. Its result depends on type, not
            // owner state; a host may already be borrowed while attaching us.
            host.is_type_of(crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY)
                || host.is_type_of(crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY)
        })
    }

    pub fn set_host(&mut self, host: Option<CoreHandle>) {
        let parent = host.as_ref().and_then(|host| {
            host.with(|host| host.as_artboard_host()?.parent_artboard())
                .flatten()
        });
        self.set_host_with_parent(host, parent);
    }

    /// A host calling while already mutably borrowed supplies its own actual
    /// parentArtboard, avoiding a second borrow to query that same pointer.
    pub(crate) fn set_host_with_parent(
        &mut self,
        host: Option<CoreHandle>,
        parent_artboard: Option<CoreHandle>,
    ) {
        let parent = self.set_host_state(host, parent_artboard);
        if let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            Self::sync_layout_after_host_attachment(&this, parent);
        }
    }

    /// The host and child are both reachable during parent layout traversal.
    /// End the child mutation before running that synchronous source callback.
    pub(crate) fn set_host_occurrence(
        this: &CoreHandle,
        host: Option<CoreHandle>,
        parent_artboard: Option<CoreHandle>,
    ) {
        let parent = this
            .with_mut(|object| {
                object
                    .as_artboard_mut()
                    .expect("Artboard")
                    .set_host_state(host, parent_artboard)
            })
            .expect("live Artboard");
        Self::sync_layout_after_host_attachment(this, parent);
    }

    fn set_host_state(
        &mut self,
        host: Option<CoreHandle>,
        parent_artboard: Option<CoreHandle>,
    ) -> Option<CoreHandle> {
        self.added_to_host();
        *self.dirty_state.0.host.borrow_mut() = host.map(|host| ArtboardHostAttachment {
            host,
            parent_artboard,
        });
        self.shares_layout_with_host()
            .then(|| self.parent_artboard())
            .flatten()
    }

    fn sync_layout_after_host_attachment(this: &CoreHandle, parent: Option<CoreHandle>) {
        if let Some(parent) = parent {
            // markLayoutDirty can synchronously notify the parent's own host,
            // which reads this same parent instance. Release its owner first.
            Self::mark_layout_dirty_occurrence(&parent, this.clone(), None);
            LayoutComponent::sync_layout_children_occurrence(&parent);
        }
    }

    pub fn set_host_handle(&mut self, host: Option<CoreHandle>) {
        self.set_host(host);
    }

    pub fn host(&self) -> Option<CoreHandle> {
        self.dirty_state
            .0
            .host
            .borrow()
            .as_ref()
            .map(|attachment| attachment.host.clone())
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.base.base.set_x(0.0);
        self.base.base.set_y(0.0);
        StatusCode::Ok
    }

    fn parent_artboard(&self) -> Option<CoreHandle> {
        self.dirty_state
            .0
            .host
            .borrow()
            .as_ref()
            .and_then(|attachment| attachment.parent_artboard.clone())
    }

    pub fn layout_width(&self) -> f32 {
        self.base.base.layout().width()
    }

    pub fn layout_height(&self) -> f32 {
        self.base.base.layout().height()
    }

    pub fn layout_x(&self) -> f32 {
        self.base.base.layout().left()
    }

    pub fn layout_y(&self) -> f32 {
        self.base.base.layout().top()
    }

    pub(crate) fn update_render_path(&mut self) {
        let background = Aabb::from_ltwh(
            -self.layout_width() * self.origin_x(),
            -self.layout_height() * self.origin_y(),
            self.layout_width(),
            self.layout_height(),
        );
        let clip = if self.frame_origin {
            Aabb::new(0.0, 0.0, self.layout_width(), self.layout_height())
        } else {
            background
        };
        self.base.base.local_path().rewind();
        self.base
            .base
            .local_path()
            .add_rect(background, PathDirection::Clockwise);
        self.base.base.world_path().rewind();
        self.base
            .base
            .world_path()
            .add_rect(clip, PathDirection::Clockwise);
    }

    pub(crate) fn update_after_layout_super_handle(root: &CoreHandle, value: ComponentDirt) {
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            if value.contains(ComponentDirt::DRAW_ORDER) {
                artboard.sort_draw_order();
            }
            if value.contains(ComponentDirt::CLIPPING) {
                artboard.clear_redundant_operations();
            }
        });
        if value.contains(ComponentDirt::LAYOUT_STYLE) {
            let (interpolation, interpolator, time, direction) = root
                .with_downcast::<Artboard, _>(|artboard| {
                    (
                        artboard.base.base.interpolation(),
                        artboard.base.base.interpolator(),
                        artboard.base.base.interpolation_time(),
                        artboard.base.base.actual_direction(),
                    )
                })
                .expect("live Artboard layout tail");
            let cascade_changed = LayoutComponent::cascade_layout_style_occurrence(
                root,
                interpolation,
                interpolator,
                time,
                direction,
            );
            Self::sync_style_changes_with_update_handle(root, cascade_changed);
        }
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.host_transform_marked_dirty = false
        });
    }

    pub fn add_dirty_data_bind(&mut self, data_bind: CoreHandle) {
        let target = data_bind
            .with(|bind| {
                bind.as_data_bind()
                    .expect("DataBind-derived owner")
                    .target()
            })
            .flatten();
        if let Some(target) = target {
            target.with(|target| {
                if let Some(component) = target.as_component() {
                    self.on_component_dirty(component);
                }
            });
        }
        self.data_bind_container.add_dirty_data_bind(data_bind);
    }

    pub fn update_data_binds_handle(root: &CoreHandle, apply_target_to_source: bool) {
        let hosts = root
            .with_downcast::<Artboard, _>(|artboard| artboard.artboard_hosts.clone())
            .expect("live Artboard");
        for host in hosts {
            if host.is_type_of(NestedArtboard::TYPE_KEY) {
                NestedArtboard::update_data_binds_occurrence(&host);
            } else {
                ArtboardComponentList::update_data_binds_occurrence(&host);
            }
        }
        crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(root.clone()).update_data_binds(apply_target_to_source);
    }

    pub fn advance_data_binds_handle(root: &CoreHandle, elapsed: f32) -> bool {
        crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(root.clone()).advance_data_binds(elapsed)
    }

    pub fn update_components_handle(root: &CoreHandle) -> bool {
        let Some((dirty, count)) = root.with_downcast::<Artboard, _>(|artboard| {
            (
                artboard.has_component_dirt(),
                artboard.dependency_order.len(),
            )
        }) else {
            return false;
        };
        if !dirty {
            return false;
        }
        let mut step = 0;
        while step < 100
            && root
                .with_downcast::<Artboard, _>(Artboard::has_component_dirt)
                .unwrap_or(false)
        {
            root.with_downcast_mut::<Artboard, _>(|artboard| {
                artboard
                    .dirty_state
                    .0
                    .dirt
                    .set(artboard.dirty_state.0.dirt.get() & !ComponentDirt::COMPONENTS)
            });
            for i in 0..count {
                let component = root
                    .with_downcast_mut::<Artboard, _>(|artboard| {
                        artboard.dirty_state.0.depth.set(i as u32);
                        artboard.dependency_order[i].clone()
                    })
                    .expect("live Artboard dependency walk");
                let dirt = component
                    .with_component(|component| component.dirt())
                    .expect("live component in dependency graph");
                if dirt == ComponentDirt::NONE || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }
                component.with_component_mut(|component| component.set_dirt(ComponentDirt::NONE));
                component.update(dirt);
                if root
                    .with_downcast::<Artboard, _>(|artboard| {
                        artboard.dirty_state.0.depth.get() < i as u32
                    })
                    .unwrap_or(false)
                {
                    break;
                }
            }
            step += 1;
        }
        true
    }

    pub fn clean_layout(&mut self, layout_component: &CoreHandle) {
        assert!(!self.is_cleaning_dirty_layouts);
        if self.is_cleaning_dirty_layouts {
            eprintln!("Artboard::cleanLayout - trying to remove a dirty layout during clean pass!");
            return;
        }
        self.dirty_layout.remove(layout_component);
        if crate::mechanical_port::source::core::CoreObject::core(self)
            .handle()
            .as_ref()
            == Some(layout_component)
            && let Some(parent) = self.parent_artboard()
        {
            parent.with_downcast_mut::<Artboard, _>(|parent| parent.clean_layout(layout_component));
        }
    }

    fn begin_layout_dirty(&mut self, layout_component: CoreHandle) -> bool {
        assert!(!self.is_cleaning_dirty_layouts);
        if self.is_cleaning_dirty_layouts {
            eprintln!(
                "Artboard::markLayoutDirty - trying to mark a layout dirty during clean pass!"
            );
            return false;
        }
        #[cfg(feature = "tools")]
        if self.dirty_layout.is_empty()
            && let Some(callback) = self.layout_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.dirty_layout.insert(layout_component);
        true
    }

    pub fn mark_layout_dirty(&mut self, layout_component: CoreHandle) {
        if !self.begin_layout_dirty(layout_component) {
            return;
        }
        if self.is_instance {
            if self.shares_layout_with_host() {
                if let Some(host) = self.host() {
                    let runtime = self.runtime_self.clone();
                    host.with_mut(|host| {
                        if let Some(host) = host.as_artboard_host_mut() {
                            host.mark_hosting_layout_dirty(runtime);
                        }
                    });
                }
            } else {
                self.mark_host_transform_dirty();
            }
        }
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    /// Preserve markLayoutDirty's synchronous host callback without retaining
    /// the child Artboard borrow across it. List setup already owns its host;
    /// that exact owner is passed through rather than borrowing its slot again.
    pub(crate) fn mark_layout_dirty_occurrence(
        root: &CoreHandle,
        layout_component: CoreHandle,
        borrowed_host: Option<&mut dyn ArtboardHost>,
    ) {
        let state = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.begin_layout_dirty(layout_component).then(|| {
                    (
                        artboard.is_instance,
                        artboard.shares_layout_with_host(),
                        artboard.host(),
                        artboard.runtime_self.clone(),
                    )
                })
            })
            .expect("live layout Artboard");
        let Some((is_instance, shares_layout, host, runtime)) = state else {
            return;
        };
        if is_instance {
            if shares_layout {
                if let Some(host) = host {
                    if let Some(borrowed_host) = borrowed_host {
                        assert_eq!(borrowed_host.host_component().as_ref(), Some(&host));
                        borrowed_host.mark_hosting_layout_dirty(runtime);
                    } else {
                        host.with_mut(|host| {
                            host.as_artboard_host_mut()
                                .expect("mounted Artboard host")
                                .mark_hosting_layout_dirty(runtime);
                        });
                    }
                }
            } else {
                let host = root
                    .with_downcast_mut::<Artboard, _>(Artboard::begin_host_transform_dirty)
                    .expect("live layout Artboard");
                if let Some(host) = host {
                    if let Some(borrowed_host) = borrowed_host {
                        assert_eq!(borrowed_host.host_component().as_ref(), Some(&host));
                        borrowed_host.mark_host_transform_dirty();
                    } else {
                        host.with_mut(|host| {
                            host.as_artboard_host_mut()
                                .expect("mounted Artboard host")
                                .mark_host_transform_dirty();
                        });
                    }
                }
            }
        }
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.add_dirt(ComponentDirt::COMPONENTS, false)
        });
    }

    fn begin_host_transform_dirty(&mut self) -> Option<CoreHandle> {
        #[cfg(feature = "tools")]
        if !self.host_transform_marked_dirty
            && let Some(callback) = self.transform_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.host_transform_marked_dirty = true;
        self.host()
    }

    pub fn mark_host_transform_dirty(&mut self) {
        if let Some(host) = self.begin_host_transform_dirty() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.mark_host_transform_dirty();
                }
            });
        }
    }

    pub fn sync_style_changes_with_update_handle(root: &CoreHandle, force_update: bool) {
        if Self::sync_style_changes_handle(root)
            && (force_update
                || root
                    .with_downcast::<Artboard, _>(Artboard::updates_own_layout)
                    .unwrap_or(false))
        {
            LayoutComponent::calculate_layout_occurrence(root, f32::NAN, f32::NAN);
            LayoutComponent::update_layout_bounds_occurrence(root, true);
        }
    }

    pub fn sync_style_changes_handle(root: &CoreHandle) -> bool {
        Self::sync_style_changes_with_parent_style_handle(root, None)
    }

    pub(crate) fn sync_style_changes_with_parent_style_handle(
        root: &CoreHandle,
        parent_style: Option<&LayoutParentStyleSnapshot>,
    ) -> bool {
        let dirty = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.is_cleaning_dirty_layouts = true;
                artboard.dirty_layout.iter().cloned().collect::<Vec<_>>()
            })
            .expect("live Artboard style pass");
        let updated = !dirty.is_empty();
        for layout in dirty {
            if &layout == root {
                LayoutComponent::sync_style_with_parent_style_occurrence(root, parent_style);
            } else if let Some(updates_own) =
                layout.with_downcast::<Artboard, _>(Artboard::updates_own_layout)
            {
                if !updates_own {
                    Self::sync_style_changes_with_parent_style_handle(&layout, parent_style);
                }
            } else {
                LayoutComponent::sync_style_with_parent_style_occurrence(&layout, parent_style);
            }
        }
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.dirty_layout.clear();
            artboard.is_cleaning_dirty_layouts = false;
        });
        updated
    }

    pub fn update_pass_handle(root: &CoreHandle, _is_root: bool) -> bool {
        Self::update_data_binds_handle(root, true);
        Self::sync_style_changes_with_update_handle(root, false);
        let (before, joysticks) = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.host_transform_marked_dirty = false;
                let before = artboard.joysticks_apply_before_update;
                let joysticks =
                    (!artboard.joysticks.is_empty()).then(|| artboard.joysticks.clone());
                (before, joysticks)
            })
            .expect("live Artboard update pass");
        let mut joystick_state = joysticks.map(|joysticks| {
            let arena = root
                .retain_arena()
                .expect("live Artboard update pass retains its object arena");
            (
                joysticks,
                RuntimeArtboardObjectContext {
                    arena,
                    root: root.clone(),
                },
            )
        });
        if before && let Some((joysticks, context)) = joystick_state.as_mut() {
            for joystick in joysticks {
                joystick.with_downcast::<Joystick, _>(|joystick| joystick.apply(context));
            }
        }
        let mut did_update = Self::update_components_handle(root);
        if !before {
            if let Some((joysticks, context)) = joystick_state.as_mut() {
                for joystick in joysticks {
                    if !joystick
                        .with_downcast::<Joystick, _>(Joystick::can_apply_before_update)
                        .unwrap_or(false)
                    {
                        Self::update_data_binds_handle(root, true);
                        did_update |= Self::update_components_handle(root);
                    }
                    joystick.with_downcast::<Joystick, _>(|joystick| joystick.apply(context));
                }
            }
            Self::update_data_binds_handle(root, true);
            did_update |= Self::update_components_handle(root);
        }
        if did_update {
            Self::update_data_binds_handle(root, true);
        }
        did_update
    }

    pub fn advance_internal_handle(
        root: &CoreHandle,
        elapsed_seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        let advancing_count = root
            .with_downcast::<Artboard, _>(|artboard| artboard.advancing_components.len())
            .expect("live Artboard advance");
        let mut did_update = false;
        for index in 0..advancing_count {
            let mut component = root
                .with_downcast::<Artboard, _>(|artboard| {
                    artboard.advancing_components.get(index).cloned()
                })
                .flatten()
                .expect("advancing component array remains stable during advance");
            did_update |= component.advance_component(elapsed_seconds, flags);
        }
        did_update | Self::advance_data_binds_handle(root, elapsed_seconds)
    }

    pub fn reset(&mut self) {
        if self.resettables.is_empty() {
            return;
        }
        let resettable_count = self.resettables.len();
        for index in 0..resettable_count {
            let resettable = self.resettables[index].clone();
            resettable.with_mut(|resettable| {
                resettable.resetting_component_reset();
            });
        }
    }

    pub fn reset_handle(root: &CoreHandle) {
        let resettable_count = root
            .with_downcast::<Artboard, _>(|artboard| artboard.resettables.len())
            .expect("live Artboard reset");
        for index in 0..resettable_count {
            let resettable = root
                .with_downcast::<Artboard, _>(|artboard| artboard.resettables.get(index).cloned())
                .flatten()
                .expect("resettable array remains stable during reset");
            resettable.with_mut(|resettable| resettable.resetting_component_reset());
        }
    }

    pub fn advance_handle(root: &CoreHandle, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        Self::poll_async_work_handle(root);
        let advancing_flags = AdvanceFlags(flags.0 | AdvanceFlags::IS_ROOT.0);
        let mut did_update = Self::advance_internal_handle(root, elapsed_seconds, advancing_flags);
        if Self::update_pass_handle(root, true) {
            did_update = true;
        }
        did_update
            || root
                .with_downcast::<Artboard, _>(Artboard::has_component_dirt)
                .unwrap_or(false)
    }

    pub fn hit_test(&mut self, info: &mut HitInfo, transform: &Mat2D) -> Option<CoreHandle> {
        let mut matrix = *transform;
        if self.frame_origin {
            matrix *= Mat2D::from_translate(
                self.layout_width() * self.origin_x(),
                self.layout_height() * self.origin_y(),
            );
        }
        if self.has_self_transform() {
            matrix *= self.self_transform();
        }
        let mut last = self.first_drawable.clone();
        while let Some(previous) = last
            .as_ref()
            .and_then(|drawable| drawable.with(Drawable::prev_drawable))
            .flatten()
        {
            last = Some(previous);
        }
        let mut drawable = last;
        while let Some(current) = drawable {
            drawable = current.with(Drawable::next_drawable).flatten();
            if current.is_hidden() {
                continue;
            }
            if let Some(core) = current.hit_test(info, &matrix) {
                return Some(core);
            }
        }
        None
    }

    pub fn hit_test_handle(&mut self, info: &mut HitInfo, transform: &Mat2D) -> Option<CoreHandle> {
        self.hit_test(info, transform)
    }

    pub fn root_transform(&mut self, point: Vec2D) -> Vec2D {
        if let Some(host) = self.host() {
            let local = if self.has_self_transform() {
                self.self_transform() * point
            } else {
                point
            };
            let runtime = self.runtime_self.clone();
            if let Some(transformed) = host
                .with(|host| {
                    host.as_artboard_host()
                        .map(|host| host.host_transform_point(&local, runtime))
                })
                .flatten()
            {
                return transformed;
            }
        }
        #[cfg(feature = "tools")]
        if let Some(callback) = self.root_transform_callback {
            let local = if self.has_self_transform() {
                self.self_transform() * point
            } else {
                point
            };
            return Vec2D::new(
                callback(self.callback_user_data, local.x, local.y, true),
                callback(self.callback_user_data, local.x, local.y, false),
            );
        }
        point
    }

    pub fn hit_test_point(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        if self.host().is_some() && self.is_instance {
            let host = self.host().unwrap();
            let runtime = self.runtime_self.clone();
            let hit = host
                .with_mut(|host| {
                    host.as_artboard_host_mut().is_some_and(|host| {
                        host.hit_test_host(position, skip_on_unclipped, runtime)
                    })
                })
                .unwrap_or(false);
            if !hit {
                return false;
            }
        }
        #[cfg(feature = "tools")]
        if let Some(callback) = self.test_bounds_callback {
            if callback(
                self.callback_user_data,
                position.x,
                position.y,
                skip_on_unclipped,
            ) == 0
            {
                return false;
            }
        }
        let origin = Vec2D::new(
            self.origin_x() * self.layout_width(),
            self.origin_y() * self.layout_height(),
        );
        self.base.base.hit_test_point_with_origin(
            position,
            skip_on_unclipped,
            is_primary_hit,
            Some(origin),
        )
    }

    pub fn draw_handle(root: &CoreHandle, renderer: &mut Renderer) {
        nuxie_render_api::increment_artboard_draw_frame_id();
        Self::internal_draw_canvases_handle(root);
        Self::draw_internal_handle(root, renderer);
    }

    pub fn draw_internal_handle(root: &CoreHandle, renderer: &mut Renderer) {
        let Some((save, first_drawable)) = root
            .with_downcast_mut::<Artboard, _>(|artboard| artboard.draw_background(renderer))
            .flatten()
        else {
            return;
        };
        Self::draw_drawables(renderer, first_drawable);
        if save {
            renderer.restore();
        }
    }

    fn draw_background(
        &mut self,
        renderer: &mut Renderer,
    ) -> Option<(bool, Option<RuntimeDrawableOccurrence>)> {
        self.dirty_state.0.did_change.set(false);
        if self.child_opacity() == 0.0 {
            return None;
        }
        let has_self = self.has_self_transform();
        let save = self.clip() || self.frame_origin || has_self;
        if save {
            renderer.save();
        }
        if self.frame_origin {
            let transform = Mat2D::from_translate(
                self.layout_width() * self.origin_x(),
                self.layout_height() * self.origin_y(),
            );
            renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
        }
        if has_self {
            renderer.transform(nuxie_render_api::Mat2D(*self.self_transform().values()));
        }
        let factory = self.factory().expect("Artboard renderer factory");
        if self.clip() {
            let path = self.base.base.local_path().render_path(&factory);
            renderer.clip_path(path);
        }
        let world_transform = self.world_transform();
        let mut paint_index = 0;
        while let Some(paint) = self
            .base
            .base
            .shape_paint_container()
            .shape_paints()
            .get(paint_index)
            .cloned()
        {
            paint_index += 1;
            paint.with_mut(|paint| {
                let Some(behavior) = paint.as_shape_paint_behavior_mut() else { return; };
                if !behavior.should_draw() { return; }
                let kind = behavior.pick_path_kind();
                let fill_rule = behavior.fill_rule();
                let path = match kind {
                    crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaintPathKind::Local => self.base.base.local_path(),
                    crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaintPathKind::LocalClockwise => self.base.base.local_clockwise_path(),
                    crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaintPathKind::World => self.base.base.world_path(),
                };
                behavior.shape_paint_mut().draw_with_factory(renderer, path, world_transform, false, None, true, fill_rule, &factory);
            });
        }
        Some((save, self.first_drawable.clone()))
    }

    fn draw_drawables(renderer: &mut Renderer, first_drawable: Option<RuntimeDrawableOccurrence>) {
        let mut empty_clips = 0;
        let mut pending_clip_operations = Vec::<RuntimeDrawableOccurrence>::new();
        let mut drawable = first_drawable;
        while let Some(current) = drawable {
            drawable = current.with(Drawable::prev_drawable).flatten();
            let previous_clips = empty_clips;
            empty_clips += current.empty_clip_count();
            if !current.will_draw() || empty_clips != previous_clips || empty_clips > 0 {
                continue;
            }
            if current.is_clip_start() {
                pending_clip_operations.push(current);
                continue;
            } else if !pending_clip_operations.is_empty() {
                if current.is_clip_end() {
                    pending_clip_operations.pop();
                    continue;
                }
                for pending in pending_clip_operations.drain(..) {
                    pending.draw(renderer);
                }
            }
            current.draw(renderer);
        }
    }

    pub fn add_to_render_path(&mut self, path: &mut RenderPath, transform: &Mat2D) {
        let mut drawable = self.first_drawable.clone();
        while let Some(current) = drawable {
            drawable = current.with(Drawable::prev_drawable).flatten();
            if current.is_hidden() {
                continue;
            }
            current.add_to_render_path(path, transform);
        }
    }

    pub fn add_to_raw_path(&mut self, path: &mut RawPath, transform: Option<&Mat2D>) {
        let mut drawable = self.first_drawable.clone();
        while let Some(current) = drawable {
            drawable = current.with(Drawable::prev_drawable).flatten();
            if current.is_hidden() {
                continue;
            }
            current.add_to_raw_path(path, transform);
        }
    }
    pub fn add_to_raw_path_handle(
        root: &CoreHandle,
        path: &mut RawPath,
        transform: Option<&Mat2D>,
    ) {
        let mut drawable = root
            .with_downcast::<Artboard, _>(|artboard| artboard.first_drawable.clone())
            .flatten();
        while let Some(current) = drawable {
            drawable = current.with(Drawable::prev_drawable).flatten();
            if current.is_hidden() {
                continue;
            }
            current.add_to_raw_path(path, transform);
        }
    }

    pub fn origin(&self) -> Vec2D {
        if self.frame_origin {
            Vec2D::new(0.0, 0.0)
        } else {
            Vec2D::new(
                -self.layout_width() * self.origin_x(),
                -self.layout_height() * self.origin_y(),
            )
        }
    }

    pub fn x_changed(&mut self) {
        self.base.base.x_changed();
        self.mark_host_transform_dirty();
    }

    pub fn y_changed(&mut self) {
        self.base.base.y_changed();
        self.mark_host_transform_dirty();
    }

    pub fn origin_x_changed(&mut self) {
        self.origin_changed_with_host(None);
    }

    pub fn origin_y_changed(&mut self) {
        self.origin_changed_with_host(None);
    }

    fn origin_changed_with_host(&mut self, borrowed_host: Option<&mut dyn ArtboardHost>) {
        self.add_dirt(ComponentDirt::PATH | ComponentDirt::COMPONENTS, false);
        if let Some(borrowed_host) = borrowed_host {
            if let Some(host) = self.begin_host_transform_dirty() {
                assert_eq!(borrowed_host.host_component().as_ref(), Some(&host));
                borrowed_host.mark_host_transform_dirty();
            }
        } else {
            self.mark_host_transform_dirty();
        }
    }

    pub fn bounds(&self) -> Aabb {
        if self.frame_origin {
            Aabb::new(0.0, 0.0, self.layout_width(), self.layout_height())
        } else {
            Aabb::from_ltwh(
                -self.layout_width() * self.origin_x(),
                -self.layout_height() * self.origin_y(),
                self.layout_width(),
                self.layout_height(),
            )
        }
    }

    pub fn world_bounds(&self) -> Aabb {
        Aabb::from_ltwh(
            self.base.base.base.base.base.base.base.x(),
            self.base.base.base.base.base.base.base.y(),
            self.base.base.layout().width(),
            self.base.base.layout().height(),
        )
    }

    pub fn is_translucent(&self) -> bool {
        for paint in self.base.base.shape_paint_container().shape_paints() {
            if !paint
                .with(|paint| {
                    paint
                        .as_shape_paint_behavior()
                        .expect("Artboard ShapePaint")
                        .is_translucent()
                })
                .expect("Artboard retains its paint")
            {
                return false;
            }
        }
        true
    }

    pub fn has_audio(&mut self) -> bool {
        if self.objects.iter().flatten().any(|object| {
            object.core_type()
                == Some(crate::mechanical_port::source::generated::audio_event_base::AudioEventBase::TYPE_KEY)
        }) {
            return true;
        }
        for host in self.artboard_hosts.clone() {
            let has_audio = host
                .with_mut(|host| {
                    let host = host.as_artboard_host_mut()?;
                    Some((0..host.artboard_count() as i32).any(|index| {
                        host.artboard_instance(index).is_some_and(|instance| {
                            instance.with_artboard_mut(|instance| instance.has_audio())
                        })
                    }))
                })
                .flatten()
                .unwrap_or(false);
            if has_audio {
                return true;
            }
        }
        false
    }

    pub fn is_animation_translucent(&self, animation: &LinearAnimation) -> bool {
        for keyed_object in animation.keyed_objects() {
            let object_id = keyed_object
                .with_downcast::<KeyedObject, _>(|keyed| keyed.object_id())
                .expect("LinearAnimation retains its keyed objects");
            let object = self.resolve_handle(object_id);
            for paint in self.base.base.shape_paint_container().shape_paints() {
                if object.as_ref() == Some(paint) {
                    return true;
                }
            }
        }
        self.is_translucent()
    }

    pub fn is_animation_instance_translucent(&self, instance: &LinearAnimationInstance) -> bool {
        instance.with_animation(|animation| self.is_animation_translucent(animation))
    }

    pub fn animation_name_at(&self, index: usize) -> String {
        self.animation_handle_at(index)
            .and_then(|animation| {
                animation
                    .with_downcast::<LinearAnimation, _>(|animation| animation.name().to_owned())
            })
            .unwrap_or_default()
    }

    pub fn state_machine_name_at(&self, index: usize) -> String {
        self.state_machine_handle_at(index)
            .and_then(|machine| {
                machine.with_downcast::<StateMachine, _>(|machine| machine.name().to_owned())
            })
            .unwrap_or_default()
    }

    pub fn animation_named(&self, name: &str) -> Option<CoreHandle> {
        self.animations
            .iter()
            .find(|animation| {
                animation
                    .with_downcast::<LinearAnimation, _>(|animation| animation.name() == name)
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn animation_handle_at(&self, index: usize) -> Option<CoreHandle> {
        self.animations.get(index).cloned()
    }

    pub fn state_machine_named(&self, name: &str) -> Option<CoreHandle> {
        self.state_machines
            .iter()
            .find(|machine| {
                machine
                    .with_downcast::<StateMachine, _>(|machine| machine.name() == name)
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn state_machine_handle_at(&self, index: usize) -> Option<CoreHandle> {
        self.state_machines.get(index).cloned()
    }

    pub fn default_state_machine_index(&self) -> i32 {
        let index = self.base.default_state_machine_id() as usize;
        if index >= self.state_machines.len() {
            -1
        } else {
            index as i32
        }
    }

    pub fn nested_artboard_handle(&self, name: &str) -> Option<CoreHandle> {
        self.nested_artboards
            .iter()
            .find(|nested| {
                nested
                    .with(|object| {
                        object
                            .as_nested_artboard()
                            .is_some_and(|nested| nested.name() == name)
                    })
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn nested_artboard_at_path(&self, path: &str) -> Option<CoreHandle> {
        let (artboard_name, rest) = path.split_once('/').unwrap_or((path, ""));
        if artboard_name.is_empty() {
            return None;
        }
        let nested = self.nested_artboard_handle(artboard_name)?;
        if rest.is_empty() {
            Some(nested)
        } else {
            let instance = nested
                .with(|object| object.as_nested_artboard()?.artboard_instance_handle(0))
                .flatten()?;
            instance.with_artboard(|instance| instance.nested_artboard_at_path(rest))
        }
    }

    pub fn set_frame_origin(&mut self, value: bool) {
        if value == self.frame_origin {
            return;
        }
        self.frame_origin = value;
        self.add_dirt(ComponentDirt::PATH, false);
    }

    pub fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool {
        let mut base = std::mem::take(&mut self.base);
        let result = base.deserialize(property_key, reader, self);
        self.base = base;
        match property_key {
            crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::WIDTH_PROPERTY_KEY => {
                self.original_width = self.width();
            }
            crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::HEIGHT_PROPERTY_KEY => {
                self.original_height = self.height();
            }
            _ => {}
        }
        result
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(artboard) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return StatusCode::MissingObject;
        };
        if import_stack.latest_backboard_importer().is_none() {
            return StatusCode::MissingObject;
        }
        // Component::import's Artboard branch operates on this already-borrowed
        // root directly, then continues to Core::import.
        debug_assert!(self.objects.is_empty());
        self.add_object(Some(artboard.clone()));
        let result =
            crate::mechanical_port::source::core::CoreObject::core_mut(self).import(import_stack);
        let backboard_importer = import_stack
            .latest_backboard_importer()
            .expect("Core import preserves the backboard importer");
        if result == StatusCode::Ok {
            backboard_importer.add_artboard(self);
        } else {
            backboard_importer.add_missing_artboard();
        }
        result
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume = value;
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                let Some(host) = host.as_artboard_host_mut() else {
                    return;
                };
                for i in 0..host.artboard_count() as i32 {
                    if let Some(artboard) = host.artboard_instance(i) {
                        artboard.with_artboard_mut(|artboard| artboard.set_volume(value));
                    }
                }
            });
        }
    }

    pub fn set_host_opacity(&mut self, value: f32) {
        if self.host_opacity == value {
            return;
        }
        self.host_opacity = value;
        self.add_dirt(ComponentDirt::RENDER_OPACITY, true);
    }

    #[cfg(test)]
    pub fn clip_path(
        &mut self,
    ) -> &mut crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath {
        self.base.base.world_path()
    }

    #[cfg(test)]
    pub fn background_path(
        &mut self,
    ) -> &mut crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath {
        self.base.base.local_path()
    }

    #[cfg(feature = "tools")]
    pub fn on_layout_changed(&mut self, callback: Option<ArtboardCallback>) {
        self.layout_changed_callback = callback;
    }

    #[cfg(feature = "tools")]
    pub fn on_layout_dirty(&mut self, callback: Option<ArtboardCallback>) {
        self.layout_dirty_callback = callback;
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    #[cfg(feature = "tools")]
    pub fn on_transform_dirty(&mut self, callback: Option<ArtboardCallback>) {
        self.transform_dirty_callback = callback;
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    #[cfg(feature = "tools")]
    pub fn on_test_bounds(&mut self, callback: Option<TestBoundsCallback>) {
        self.test_bounds_callback = callback;
    }

    #[cfg(feature = "tools")]
    pub fn on_is_ancestor(&mut self, callback: Option<IsAncestorCallback>) {
        self.is_ancestor_callback = callback;
    }

    #[cfg(feature = "tools")]
    pub fn on_root_transform(&mut self, callback: Option<RootTransformCallback>) {
        self.root_transform_callback = callback;
    }

    pub fn audio_engine(&self) -> Option<AudioEngineRef> {
        self.audio_engine.clone()
    }

    pub fn audio_engine_handle(&self) -> Option<AudioEngineRef> {
        self.audio_engine()
    }

    pub fn set_audio_engine(&mut self, audio_engine: Option<AudioEngineRef>) {
        self.audio_engine = audio_engine.clone();
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                let Some(host) = host.as_artboard_host_mut() else {
                    return;
                };
                for i in 0..host.artboard_count() as i32 {
                    if let Some(artboard) = host.artboard_instance(i) {
                        artboard.with_artboard_mut(|artboard| {
                            artboard.set_audio_engine(audio_engine.clone())
                        });
                    }
                }
            });
        }
    }

    pub fn is_ancestor(&mut self, artboard: Option<CoreHandle>) -> bool {
        let candidate_source = artboard.as_ref().and_then(|artboard| {
            artboard
                .with_downcast::<Artboard, _>(Artboard::artboard_source_handle)
                .flatten()
        });
        if candidate_source.is_some() && candidate_source == self.artboard_source_handle() {
            return true;
        }
        if let Some(parent) = self.parent_artboard() {
            return parent
                .with_downcast_mut::<Artboard, _>(|parent| parent.is_ancestor(artboard.clone()))
                .unwrap_or(false);
        }
        #[cfg(feature = "tools")]
        if let (Some(callback), Some(artboard)) = (self.is_ancestor_callback, artboard)
            && artboard
                .with_downcast::<Artboard, _>(|artboard| {
                    callback(self.callback_user_data, artboard.artboard_id()) == 1
                })
                .unwrap_or(false)
        {
            return true;
        }
        false
    }

    pub fn changed(&mut self) {
        self.dirty_state.changed();
    }

    fn has_parent_focus_data(focus_data: &CoreHandle) -> bool {
        let mut current = focus_data
            .with(|focus_data| focus_data.component_parent_handle())
            .flatten();
        while let Some(parent) = current {
            let contains_other_focus = parent
                .with(|parent| {
                    parent.as_node().is_some_and(|node| {
                        node.children().iter().any(|child| {
                            child != focus_data
                                && child
                                    .with(|child| {
                                        child.as_any().downcast_ref::<FocusData>().is_some()
                                    })
                                    .unwrap_or(false)
                        })
                    })
                })
                .unwrap_or(false);
            if contains_other_focus {
                return true;
            }
            current = parent
                .with(|parent| parent.component_parent_handle())
                .flatten();
        }
        false
    }

    pub fn root_focus_data_count(&self) -> usize {
        self.objects
            .iter()
            .flatten()
            .filter(|object| {
                object
                    .with(|object| object.as_any().downcast_ref::<FocusData>().is_some())
                    .unwrap_or(false)
                    && !Self::has_parent_focus_data(object)
            })
            .count()
    }

    pub fn root_focus_data_at(&self, index: usize) -> Option<CoreHandle> {
        self.objects
            .iter()
            .flatten()
            .filter(|object| {
                object
                    .with(|object| object.as_any().downcast_ref::<FocusData>().is_some())
                    .unwrap_or(false)
                    && !Self::has_parent_focus_data(object)
            })
            .nth(index)
            .cloned()
    }

    fn build_focus_tree_visit(
        focus_manager: &RuntimeFocusManagerHandle,
        component: CoreHandle,
        focus_node: Option<FocusNodeRef>,
    ) {
        let nested_animations = component
            .with(|component| {
                component
                    .as_nested_artboard()
                    .map(|nested| nested.nested_animations().to_vec())
            })
            .flatten();
        if let Some(animations) = nested_animations {
            let mut rewired = false;
            for animation in animations {
                animation.with_downcast_mut::<NestedStateMachine, _>(|nested_state_machine| {
                    if let Some(instance) = nested_state_machine.state_machine_instance() {
                        instance.with_instance_mut(|instance| {
                            if !instance.focus_manager().ptr_eq(focus_manager) {
                                instance.set_external_focus_manager_handle(focus_manager.clone());
                                rewired = true;
                            }
                        });
                    }
                });
            }
            NestedArtboard::sync_nested_focus_tree_occurrence(
                &component,
                focus_node.clone(),
                true,
                rewired,
            );
        } else {
            component.with_downcast_mut::<ArtboardComponentList, _>(|list| {
                list.ensure_list_scope_focus_node(focus_manager.clone(), focus_node.clone());
            });
        }
        let children = component
            .with(|component| {
                component
                    .as_container_component()
                    .map(|container| container.children().to_vec())
            })
            .flatten()
            .unwrap_or_default();
        Self::build_focus_tree_children(focus_manager, children, focus_node);
    }

    fn build_focus_tree_children(
        focus_manager: &RuntimeFocusManagerHandle,
        children: Vec<CoreHandle>,
        focus_node: Option<FocusNodeRef>,
    ) {
        let direct_focus_data = children
            .iter()
            .find(|child| child.is_type_of(FocusData::TYPE_KEY));
        let recurse_with = direct_focus_data
            .and_then(|focus_data| {
                focus_data
                    .with_mut(|focus_data| {
                        focus_data
                            .as_any_mut()
                            .downcast_mut::<FocusData>()
                            .map(|focus_data| focus_data.focus_node())
                    })
                    .flatten()
            })
            .map(|node| {
                focus_manager.with_focus_manager_mut(|manager| {
                    manager.add_child(focus_node.clone(), node.clone(), None)
                });
                node
            })
            .or(focus_node);
        for child in children {
            if child.is_type_of(FocusData::TYPE_KEY) {
                continue;
            }
            Self::build_focus_tree_visit(focus_manager, child, recurse_with.clone());
        }
    }

    pub fn build_focus_tree_handle(
        root: &CoreHandle,
        focus_manager: Option<RuntimeFocusManagerHandle>,
        parent_focus_node: Option<FocusNodeRef>,
    ) {
        let Some(focus_manager) = focus_manager else {
            return;
        };
        let effective_parent = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.active_focus_manager = Some(focus_manager.clone());
                #[cfg(feature = "tools")]
                {
                    if let Some(parent) = parent_focus_node.clone() {
                        artboard.external_parent_focus_node = Some(parent);
                    }
                    parent_focus_node.or_else(|| artboard.external_parent_focus_node.clone())
                }
                #[cfg(not(feature = "tools"))]
                parent_focus_node
            })
            .expect("focus tree root is a live Artboard");
        // Release the root before visiting children: their synchronous focus
        // callbacks consult this same Artboard's active manager.
        Self::build_focus_tree_visit(&focus_manager, root.clone(), effective_parent);
    }

    pub fn build_focus_tree_from_parent_handle(root: &CoreHandle, parent: Option<FocusNodeRef>) {
        let Some(parent) = parent else {
            return;
        };
        let Some(manager) = parent.borrow().manager() else {
            return;
        };
        Self::build_focus_tree_handle(root, Some(manager), Some(parent));
    }

    pub fn cleanup_focus_tree_handle(root: &CoreHandle) {
        let Some((manager, objects, nested_artboards, component_lists)) = root
            .with_downcast::<Artboard, _>(|artboard| {
                Some((
                    artboard.active_focus_manager.clone()?,
                    artboard.objects.clone(),
                    artboard.nested_artboards.clone(),
                    artboard.component_lists.clone(),
                ))
            })
            .flatten()
        else {
            return;
        };
        for object in objects.iter().flatten() {
            if !object.is_type_of(FocusData::TYPE_KEY) {
                continue;
            }
            let node = object
                .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
                .expect("FocusData type key resolves to FocusData");
            let should_remove = {
                let node = node.borrow();
                match node.manager() {
                    Some(owner) => owner.ptr_eq(&manager),
                    None => node.parent().is_some(),
                }
            };
            if should_remove {
                // removeChild may synchronously blur this very FocusData.
                manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
            }
        }
        for nested_host in nested_artboards {
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                let shares_manager = nested.with_artboard(|nested| {
                    nested
                        .active_focus_manager
                        .as_ref()
                        .is_some_and(|nested_manager| nested_manager.ptr_eq(&manager))
                });
                if shares_manager {
                    nested.cleanup_focus_tree();
                }
            }
        }
        for list in &component_lists {
            let instances = list
                .with(|list| {
                    let host = list.as_artboard_host()?;
                    Some(
                        (0..host.artboard_count() as i32)
                            .filter_map(|index| host.artboard_instance(index))
                            .collect::<Vec<_>>(),
                    )
                })
                .flatten()
                .unwrap_or_default();
            for nested in instances {
                let shares_manager = nested.with_artboard(|nested| {
                    nested
                        .active_focus_manager
                        .as_ref()
                        .is_some_and(|nested_manager| nested_manager.ptr_eq(&manager))
                });
                if shares_manager {
                    nested.cleanup_focus_tree();
                }
            }
        }
        for list in component_lists {
            list.with_downcast_mut::<ArtboardComponentList, _>(
                ArtboardComponentList::remove_list_scope_focus_node,
            );
        }
        root.with_downcast_mut::<Artboard, _>(|artboard| artboard.active_focus_manager = None);
    }

    #[cfg(feature = "tools")]
    pub fn set_external_parent_focus_node(&mut self, node: Option<FocusNodeRef>) {
        self.external_parent_focus_node = node;
    }

    #[cfg(feature = "tools")]
    pub fn external_parent_focus_node(&self) -> Option<FocusNodeRef> {
        self.external_parent_focus_node.clone()
    }

    #[cfg(feature = "tools")]
    pub fn collapse_single(&mut self, value: bool) {
        self.base
            .base
            .as_component_mut()
            .expect("Artboard inherits Component")
            .collapse(value);
    }

    pub fn build_semantic_tree_handle(
        root: &CoreHandle,
        semantic_manager: Option<RuntimeSemanticManagerHandle>,
        parent_semantic_node: Option<SemanticNodeRef>,
    ) {
        let Some(semantic_manager) = semantic_manager else {
            return;
        };
        let boundary = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.active_semantic_manager = Some(semantic_manager.clone());
                if artboard.host().is_none() {
                    return None;
                }
                if artboard.semantic_boundary_node.is_none() {
                    let boundary = SemanticNode::new(0);
                    {
                        let mut node = boundary.borrow_mut();
                        node.is_boundary_node = true;
                        node.boundary_artboard = Some(root.clone());
                    }
                    artboard.semantic_boundary_node = Some(boundary);
                }
                artboard.semantic_boundary_node.clone()
            })
            .expect("live semantic Artboard");
        let mut effective_parent = parent_semantic_node.clone();
        if let Some(boundary) = boundary {
            semantic_manager.add_child(parent_semantic_node, boundary.clone());
            root.with_downcast_mut::<Artboard, _>(Artboard::mark_semantic_boundary_transform_dirty);
            effective_parent = Some(boundary);
        }

        let objects = root
            .with_downcast::<Artboard, _>(|artboard| artboard.objects.clone())
            .expect("live semantic Artboard");
        for object in objects.into_iter().flatten() {
            if !object.is_type_of(crate::mechanical_port::source::generated::semantic::semantic_data_base::SemanticDataBase::TYPE_KEY) {
                continue;
            }
            object.with_mut(|candidate| {
                let semantic_data = candidate.as_semantic_data_mut().expect("SemanticData");
                let node = semantic_data
                    .register_with_manager(semantic_manager.clone(), effective_parent.clone());
                debug_assert!(
                    semantic_data
                        .existing_semantic_node()
                        .is_some_and(|n| Rc::ptr_eq(&n, &node))
                );
            });
        }

        let nested_artboards = root
            .with_downcast::<Artboard, _>(|artboard| artboard.nested_artboards.clone())
            .expect("live semantic Artboard");
        for nested_host in nested_artboards {
            let parent = SemanticData::find_closest_semantic_node_handle(Some(nested_host.clone()))
                .or_else(|| effective_parent.clone());
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                if !nested.with_artboard(|nested| nested.semantic_manager_is(&semantic_manager)) {
                    nested.cleanup_semantic_tree();
                    nested.build_semantic_tree(Some(semantic_manager.clone()), parent);
                }
            }
        }
        let lists = root
            .with_downcast::<Artboard, _>(|artboard| artboard.component_lists.clone())
            .expect("live semantic Artboard");
        for list in lists {
            let parent = SemanticData::find_closest_semantic_node_handle(Some(list.clone()))
                .or_else(|| effective_parent.clone());
            let mut index = 0;
            loop {
                let count = list
                    .with(|list| {
                        list.as_artboard_host()
                            .expect("component list host")
                            .artboard_count()
                    })
                    .expect("live component list");
                if index >= count {
                    break;
                }
                let nested = list
                    .with(|list| {
                        list.as_artboard_host()
                            .expect("component list host")
                            .artboard_instance(index as i32)
                    })
                    .flatten();
                if let Some(nested) = nested {
                    if !nested.with_artboard(|nested| nested.semantic_manager_is(&semantic_manager))
                    {
                        nested.cleanup_semantic_tree();
                        nested.build_semantic_tree(Some(semantic_manager.clone()), parent.clone());
                    }
                }
                index += 1;
            }
        }
    }

    fn semantic_manager_is(&self, manager: &RuntimeSemanticManagerHandle) -> bool {
        self.active_semantic_manager
            .as_ref()
            .is_some_and(|current| current.ptr_eq(manager))
    }

    pub fn cleanup_semantic_tree_handle(root: &CoreHandle) {
        let Some(manager) = root
            .with_downcast::<Artboard, _>(|artboard| artboard.active_semantic_manager.clone())
            .flatten()
        else {
            return;
        };
        let nested_artboards = root
            .with_downcast::<Artboard, _>(|artboard| artboard.nested_artboards.clone())
            .expect("live semantic Artboard");
        for nested_host in nested_artboards {
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                if nested.with_artboard(|nested| nested.semantic_manager_is(&manager)) {
                    nested.cleanup_semantic_tree();
                }
            }
        }
        let lists = root
            .with_downcast::<Artboard, _>(|artboard| artboard.component_lists.clone())
            .expect("live semantic Artboard");
        for list in lists {
            let mut index = 0;
            loop {
                let count = list
                    .with(|list| {
                        list.as_artboard_host()
                            .expect("component list host")
                            .artboard_count()
                    })
                    .expect("live component list");
                if index >= count {
                    break;
                }
                let nested = list
                    .with(|list| {
                        list.as_artboard_host()
                            .expect("component list host")
                            .artboard_instance(index as i32)
                    })
                    .flatten();
                if let Some(nested) = nested {
                    if nested.with_artboard(|nested| nested.semantic_manager_is(&manager)) {
                        nested.cleanup_semantic_tree();
                    }
                }
                index += 1;
            }
        }
        let objects = root
            .with_downcast::<Artboard, _>(|artboard| artboard.objects.clone())
            .expect("live semantic Artboard");
        for object in objects.into_iter().flatten() {
            if object.is_type_of(crate::mechanical_port::source::generated::semantic::semantic_data_base::SemanticDataBase::TYPE_KEY) {
                object.with_mut(|object| object.as_semantic_data_mut().expect("SemanticData").detach_if_managed_by(&manager));
            }
        }
        let boundary = root
            .with_downcast::<Artboard, _>(|artboard| artboard.semantic_boundary_node.clone())
            .flatten();
        if let Some(boundary) = boundary {
            let managed_here = boundary
                .borrow()
                .manager()
                .as_ref()
                .is_some_and(|owner| owner.ptr_eq(&manager));
            if managed_here {
                manager.remove_child(&boundary);
            }
        }
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.semantic_boundary_node = None;
            artboard.active_semantic_manager = None;
        });
    }

    fn collapse_boundary_subtree(node: &SemanticNodeRef, value: bool) {
        let children = node.borrow().children().to_vec();
        for child in children {
            let semantic_data = child.borrow().semantic_data.clone();
            if let Some(semantic_data) = semantic_data {
                semantic_data.with_mut(|semantic_data| {
                    if semantic_data
                        .as_semantic_data()
                        .is_some_and(|semantic_data| semantic_data.is_collapsed() != value)
                    {
                        semantic_data.component_collapse(value);
                    }
                });
            }
            Self::collapse_boundary_subtree(&child, value);
        }
    }

    pub fn collapse_semantic_boundary_handle(root: &CoreHandle, value: bool) {
        let (managed, boundary) = root
            .with_downcast::<Artboard, _>(|artboard| {
                (
                    artboard.active_semantic_manager.is_some(),
                    artboard.semantic_boundary_node.clone(),
                )
            })
            .expect("live semantic Artboard");
        if !managed {
            return;
        }
        if value && boundary.is_some() {
            Self::collapse_boundary_subtree(boundary.as_ref().unwrap(), true);
        } else {
            let objects = root
                .with_downcast::<Artboard, _>(|artboard| artboard.objects.clone())
                .expect("live semantic Artboard");
            for object in objects.into_iter().flatten() {
                if object.is_type_of(crate::mechanical_port::source::generated::semantic::semantic_data_base::SemanticDataBase::TYPE_KEY) {
                    object.with_mut(|object| {
                        if object.as_semantic_data().expect("SemanticData").is_collapsed()
                            != value
                        {
                            object.component_collapse(value);
                        }
                    });
                }
            }
        }
        if !value {
            root.with_downcast_mut::<Artboard, _>(Artboard::mark_semantic_boundary_transform_dirty);
        }
    }

    pub fn mark_semantic_boundary_transform_dirty(&mut self) {
        if let (Some(boundary), Some(manager)) = (
            self.semantic_boundary_node.as_ref(),
            self.active_semantic_manager.as_ref(),
        ) {
            manager.with_semantic_manager_mut(|manager| {
                manager.mark_boundary_dirty(boundary.borrow().id())
            });
        }
    }

    fn clone_object_data_binds(
        data_binds: &[CoreHandle],
        object: Option<&CoreHandle>,
        clone: Option<CoreHandle>,
        arena: &CoreArena,
        container: &DataBindContainer,
    ) -> Option<()> {
        for data_bind_handle in data_binds {
            let matches = data_bind_handle
                .with(|data_bind| {
                    data_bind
                        .as_data_bind()
                        .is_some_and(|data_bind| data_bind.target().as_ref() == object)
                })
                .unwrap_or(false);
            if !matches {
                continue;
            }
            let clone_handle = data_bind_handle.clone_occurrence_into(arena)?;
            let (file, converter) = data_bind_handle
                .with(|data_bind| {
                    let data_bind = data_bind.as_data_bind()?;
                    Some((data_bind.file(), data_bind.converter()))
                })
                .flatten()
                .unwrap_or_default();
            clone_handle.with_mut(|data_bind| {
                if let Some(data_bind) = data_bind.as_data_bind_mut() {
                    data_bind.set_target(clone.clone());
                    data_bind.set_file(file);
                    data_bind.initialize();
                }
            });
            if let Some(converter) = converter {
                let converter_clone = converter.clone_occurrence_into(arena)?;
                clone_handle.with_mut(|data_bind| {
                    data_bind
                        .as_data_bind_mut()
                        .unwrap()
                        .set_converter(Some(converter_clone));
                });
            }
            container.add_data_bind(clone_handle);
        }
        Some(())
    }

    pub fn internal_data_context_handle(root: &CoreHandle, value: RuntimeDataContextHandle) {
        let hosts = root
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.data_context = Some(value.clone());
                artboard.artboard_hosts.clone()
            })
            .expect("live Artboard");
        for host in hosts {
            let instance = host
                .with(|owner| {
                    owner
                        .as_artboard_host()
                        .expect("ArtboardHost owner")
                        .data_bind_path_referencer()
                        .with_data_bind_path_mut(|path| {
                            value.with_context(|context| context.get_instance_from_path(Some(path)))
                        })
                        .flatten()
                })
                .flatten();
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    if let Some(instance) = instance {
                        host.bind_view_model_instance(instance, value.clone());
                    } else {
                        host.internal_data_context(value.clone());
                    }
                }
            });
        }
        let container = crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(root.clone());
        container.bind_data_binds_from_context(value.clone());
        container.sort_data_binds();
        let objects = root
            .with_downcast::<Artboard, _>(|artboard| artboard.scripted_objects.clone())
            .expect("live Artboard");
        for object in objects {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.set_data_context(Some(value.clone()));
                }
            });
        }
        Self::init_scripted_objects_handle(root);
    }

    pub fn bind_handle(root: &CoreHandle) {
        if let Some(context) = root
            .with_downcast::<Artboard, _>(Artboard::data_context)
            .flatten()
        {
            Self::internal_data_context_handle(root, context);
        }
    }

    pub fn clear_data_context_handle(root: &CoreHandle) {
        let context = root
            .with_downcast::<Artboard, _>(Artboard::data_context)
            .flatten();
        if let Some(context) = context {
            context.with_context_mut(|context| context.remove_dependent_container(root));
            root.with_downcast_mut::<Artboard, _>(|artboard| artboard.data_context = None);
        }
        let (hosts, objects) = root
            .with_downcast::<Artboard, _>(|artboard| {
                (
                    artboard.artboard_hosts.clone(),
                    artboard.scripted_objects.clone(),
                )
            })
            .expect("live Artboard");
        for host in hosts {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.clear_data_context();
                }
            });
        }
        for object in objects {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.reset_lua_init();
                }
            });
        }
    }

    pub fn unbind_handle(root: &CoreHandle) {
        Self::clear_data_context_handle(root);
        crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainerOwner::Authored(root.clone()).unbind_data_binds();
        let hosts = root
            .with_downcast::<Artboard, _>(|artboard| artboard.artboard_hosts.clone())
            .expect("live Artboard");
        for host in hosts {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.unbind();
                }
            });
        }
    }

    pub fn bind_view_model_instance_handle(
        root: &CoreHandle,
        instance: Option<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        let Some(instance) = instance else {
            Self::unbind_handle(root);
            return;
        };
        Self::set_view_model_instance_handle(root, instance);
        let context = root
            .with_downcast::<Artboard, _>(Artboard::data_context)
            .flatten()
            .expect("set_view_model_instance creates a context");
        if let Some(parent) = parent {
            context.with_context_mut(|context| context.set_parent(Some(parent)));
        }
        Self::internal_data_context_handle(root, context);
    }

    pub fn set_view_model_instance_handle(root: &CoreHandle, instance: CoreHandle) {
        let context = root
            .with_downcast::<Artboard, _>(Artboard::data_context)
            .flatten();
        if let Some(context) = context {
            context
                .with_context_mut(|context| context.set_main_view_model_instance(Some(instance)));
        } else {
            let context = RuntimeDataContextHandle::new(DataContext::new(Some(instance)));
            context.with_context_mut(|context| context.add_dependent_container(root.clone()));
            root.with_downcast_mut::<Artboard, _>(|artboard| artboard.data_context = Some(context));
        }
    }

    pub fn bind_view_model_instances_handle(
        root: &CoreHandle,
        instances: Vec<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        if instances.is_empty() {
            Self::unbind_handle(root);
            return;
        }
        Self::clear_data_context_handle(root);
        let context = RuntimeDataContextHandle::new(DataContext::from_instances(
            instances.into_iter().map(Some).collect(),
        ));
        context.with_context_mut(|context| {
            context.add_dependent_container(root.clone());
            context.set_parent(parent);
        });
        Self::internal_data_context_handle(root, context);
    }

    pub fn relink_data_context_handle(root: &CoreHandle) {
        let Some(context) = root
            .with_downcast::<Artboard, _>(Artboard::data_context)
            .flatten()
        else {
            return;
        };
        let hosts = root
            .with_downcast::<Artboard, _>(|artboard| artboard.artboard_hosts.clone())
            .expect("live Artboard");
        for host in hosts {
            let resolved = host
                .with(|owner| {
                    owner
                        .as_artboard_host()
                        .expect("ArtboardHost owner")
                        .data_bind_path_referencer()
                        .with_data_bind_path_mut(|path| {
                            context
                                .with_context(|context| context.get_instance_from_path(Some(path)))
                        })
                        .flatten()
                })
                .flatten();
            let value =
                resolved.or_else(|| context.with_context(DataContext::main_view_model_instance));
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.relink_data_context(value);
                }
            });
        }
    }

    fn unbind_for_drop(&mut self) {
        self.clear_data_context_for_drop();
        self.data_bind_container.unbind_data_binds();
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.unbind();
                }
            });
        }
    }

    fn clear_data_context_for_drop(&mut self) {
        if let Some(context) = self.data_context.take()
            && let Some(owner) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
        {
            context.with_context_mut(|context| {
                context.remove_dependent_container(&owner);
            });
        }
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.clear_data_context();
                }
            });
        }
        for scripted_object in self.scripted_objects.clone() {
            scripted_object.with_mut(|scripted_object| {
                if let Some(scripted_object) = scripted_object.as_scripted_object_mut() {
                    scripted_object.reset_lua_init();
                }
            });
        }
    }

    pub fn global_view_model_instance(&self, name: &str) -> Option<CoreHandle> {
        let context = self.data_context.as_ref()?;
        let file = self.artboard_file()?;
        let slot = file.with_file(|file| file.view_model_id(name))?;
        context.with_context(|context| context.instance_for_slot(slot))
    }

    pub fn set_global_view_model_instance(
        &mut self,
        name: &str,
        instance: Option<CoreHandle>,
    ) -> bool {
        let Some(file) = self.artboard_file() else {
            return false;
        };
        let Some((slot_key, count, slot_view_model)) = file.with_file(|file| {
            let slot_key = file.view_model_id(name);
            (
                slot_key,
                file.view_model_count(),
                file.view_model(slot_key as usize),
            )
        }) else {
            return false;
        };
        if slot_key >= count as u32 {
            return false;
        }
        let Some(slot_view_model) = slot_view_model else {
            return false;
        };
        if slot_view_model
            .with_downcast::<crate::mechanical_port::source::viewmodel::viewmodel::ViewModel, _>(
                |view_model| view_model.base.view_model_type(),
            )
            != Some(crate::mechanical_port::source::view_model_type::ViewModelType::Global as u32)
        {
            return false;
        }
        if self.data_context.is_none() {
            if instance.is_none() {
                return true;
            }
            let context = RuntimeDataContextHandle::new(DataContext::new(None));
            if let Some(owner) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
            {
                context.with_context_mut(|context| context.add_dependent_container(owner));
            }
            self.data_context = Some(context);
        }
        self.data_context
            .as_ref()
            .unwrap()
            .with_context_mut(|context| {
                context.set_view_model_instance_for_slot(slot_key, instance)
            });
        true
    }

    pub fn find_handle<T: crate::mechanical_port::source::core::CoreType>(
        &self,
        name: &str,
    ) -> Option<CoreHandle> {
        let own_handle = crate::mechanical_port::source::core::CoreObject::core(self).handle();
        self.objects.iter().flatten().find_map(|object| {
            if !object.is_type_of(T::TYPE_KEY) {
                return None;
            }
            let matches = if own_handle.as_ref() == Some(object) {
                self.name() == name
            } else {
                object.with(|candidate| {
                    candidate
                        .as_component()
                        .is_some_and(|component| component.name() == name)
                })?
            };
            matches.then(|| object.clone())
        })
    }

    pub fn count<T: crate::mechanical_port::source::core::CoreType>(&self) -> usize {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(T::TYPE_KEY))
            .count()
    }

    pub fn object_handle_at<T: crate::mechanical_port::source::core::CoreType>(
        &self,
        index: usize,
    ) -> Option<CoreHandle> {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(T::TYPE_KEY))
            .nth(index)
            .cloned()
    }

    pub fn object_index(&self, component: &CoreHandle) -> i32 {
        self.objects
            .iter()
            .position(|object| object.as_ref().is_some_and(|object| object == component))
            .map_or(-1, |index| index as i32)
    }

    pub fn find_all_handles<T: crate::mechanical_port::source::core::CoreType>(
        &self,
    ) -> Vec<CoreHandle> {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(T::TYPE_KEY))
            .cloned()
            .collect()
    }

    fn clone_instance_definition(&self) -> Box<ArtboardInstance> {
        let mut clone = Box::new(ArtboardInstance::default());
        let mut base = std::mem::take(&mut clone.base.base);
        base.copy(&self.base, &mut clone.base);
        clone.base.base = base;
        clone.base.factory = self.factory.clone();
        clone.base.file = self.file.clone();
        clone.base.scripting_vm = self.scripting_vm.clone();
        clone.base.frame_origin = self.frame_origin;
        clone.base.data_context = self.data_context.clone();
        clone.base.is_instance = true;
        clone.base.original_width = self.original_width;
        clone.base.original_height = self.original_height;
        clone.base.definition_owner = Some(if let Some(source) = self.artboard_source.as_ref() {
            source
                .retain_arena()
                .expect("live Artboard source definition")
        } else {
            self.core_arena.strong_handle()
        });
        #[cfg(feature = "tools")]
        {
            clone.base.artboard_id = self.artboard_id;
        }
        clone.base.artboard_source = if self.is_instance {
            self.artboard_source.clone()
        } else {
            crate::mechanical_port::source::core::CoreObject::core(self).handle()
        };
        assert!(clone.base.is_instance());
        clone
    }

    pub fn instance_from_handle(source: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
        Self::instance_from_handle_internal(source, true)
    }

    /// An embedded instance's source definitions are owned by its containing
    /// File/Artboard. Retaining that same arena from the child would create a
    /// parent -> NestedArtboard -> child -> parent ownership cycle.
    pub fn nested_instance_from_handle(
        source: &CoreHandle,
    ) -> Option<RuntimeArtboardInstanceHandle> {
        Self::instance_from_handle_internal(source, false)
    }

    fn instance_from_handle_internal(
        source: &CoreHandle,
        retain_definitions: bool,
    ) -> Option<RuntimeArtboardInstanceHandle> {
        let (mut definition, objects, data_binds, animations, state_machines) = source
            .with_downcast::<Artboard, _>(|source| {
                (
                    source.clone_instance_definition(),
                    source.objects.clone(),
                    source.data_bind_container.data_binds(),
                    source.animations.clone(),
                    source.state_machines.clone(),
                )
            })?;
        if !retain_definitions {
            definition.base.definition_owner = None;
        }
        let instance = RuntimeArtboardInstanceHandle::new(*definition);
        let root = instance.core_handle();
        let (arena, container) = instance.with_artboard(|instance| {
            (
                instance.base.core_arena.clone(),
                instance.base.data_bind_container.clone(),
            )
        });
        // The root exists before bind initialization so observers and collapsables
        // attach to the actual instance, never a temporary copied Artboard.
        Self::clone_object_data_binds(
            &data_binds,
            Some(source),
            Some(root.clone()),
            &arena,
            &container,
        )?;
        for object in objects.iter().skip(1) {
            let clone = match object {
                Some(object) => Some(object.clone_occurrence_into(&arena)?),
                None => None,
            };
            instance.with_artboard_mut(|instance| instance.base.objects.push(clone.clone()));
            Self::clone_object_data_binds(&data_binds, object.as_ref(), clone, &arena, &container)?;
        }
        instance.with_artboard_mut(|instance| {
            instance.base.animations = animations;
            instance.base.state_machines = state_machines;
        });
        (Self::initialize_handle(&root) == StatusCode::Ok).then_some(instance)
    }

    fn artboard_file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.upgrade().map(|_| self.file.clone())
    }

    pub fn width(&self) -> f32 {
        self.base.base.width()
    }

    pub fn height(&self) -> f32 {
        self.base.base.height()
    }

    pub fn set_width(&mut self, value: f32) {
        self.base.base.set_width(value);
    }

    pub fn set_height(&mut self, value: f32) {
        self.base.base.set_height(value);
    }

    pub fn origin_x(&self) -> f32 {
        self.base.origin_x()
    }

    pub fn origin_y(&self) -> f32 {
        self.base.origin_y()
    }

    pub fn set_origin_x(&mut self, value: f32) {
        self.set_origin_x_with_host(value, None);
    }

    pub(crate) fn set_origin_x_with_borrowed_host(
        &mut self,
        value: f32,
        host: &mut dyn ArtboardHost,
    ) {
        self.set_origin_x_with_host(value, Some(host));
    }

    fn set_origin_x_with_host(&mut self, value: f32, host: Option<&mut dyn ArtboardHost>) {
        if self.base.set_origin_x_value(value) {
            self.origin_changed_with_host(host);
            crate::mechanical_port::source::core::CoreObject::core_mut(self)
                .notify_property_changed(ArtboardBase::ORIGIN_X_PROPERTY_KEY);
        }
    }

    pub fn set_origin_y(&mut self, value: f32) {
        self.set_origin_y_with_host(value, None);
    }

    pub(crate) fn set_origin_y_with_borrowed_host(
        &mut self,
        value: f32,
        host: &mut dyn ArtboardHost,
    ) {
        self.set_origin_y_with_host(value, Some(host));
    }

    fn set_origin_y_with_host(&mut self, value: f32, host: Option<&mut dyn ArtboardHost>) {
        if self.base.set_origin_y_value(value) {
            self.origin_changed_with_host(host);
            crate::mechanical_port::source::core::CoreObject::core_mut(self)
                .notify_property_changed(ArtboardBase::ORIGIN_Y_PROPERTY_KEY);
        }
    }

    pub fn x(&self) -> f32 {
        self.base.base.x()
    }

    pub fn y(&self) -> f32 {
        self.base.base.y()
    }

    pub fn rotation(&self) -> f32 {
        self.base.base.rotation()
    }

    pub fn scale_x(&self) -> f32 {
        self.base.base.scale_x()
    }

    pub fn scale_y(&self) -> f32 {
        self.base.base.scale_y()
    }

    pub fn clip(&self) -> bool {
        self.base.base.clip()
    }

    pub fn render_opacity(&self) -> f32 {
        self.base.base.render_opacity()
    }

    pub fn world_transform(&self) -> Mat2D {
        *self.base.base.world_transform()
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        CoreCapabilities::component_add_dirt(self, value, recurse)
    }

    pub fn can_have_overrides(&self) -> bool {
        true
    }

    pub fn update_world_transform(&mut self) {}
}

impl ArtboardBaseCallbacks for Artboard {
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::CoreObject::core_mut(self)
            .notify_property_changed(property_key);
    }

    fn origin_x_changed(&mut self) {
        Artboard::origin_x_changed(self);
    }

    fn origin_y_changed(&mut self) {
        Artboard::origin_y_changed(self);
    }
}

impl CoreContext for Artboard {
    fn core_arena(&self) -> &CoreArena {
        &self.core_arena
    }

    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        Artboard::resolve_handle(self, id)
    }
}

impl KeyedObjectContext for Artboard {
    fn resolves_object(&self, id: u32) -> bool {
        self.resolve_handle(id).is_some()
    }

    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
        self.resolve_handle(id)
    }

    fn object_supports_property(&self, id: u32, key: u32) -> bool {
        self.resolve_handle(id)
            .and_then(|object| {
                object.with(|object| CoreRegistry::object_supports_property(object, key))
            })
            .unwrap_or(false)
    }

    fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool {
        object
            .with_mut(|object| {
                CoreCapabilities::overrides_keyed_interpolation(object, key as i32).unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

impl crate::mechanical_port::source::animation::animation_reset_factory::ResetArtboard
    for Artboard
{
    fn resolves(&self, object_id: u32) -> bool {
        self.resolve_handle(object_id).is_some()
    }

    fn double_value(&self, object_id: u32, property_key: u32) -> f32 {
        self.resolve_handle(object_id)
            .and_then(|object| CoreRegistry::get_double_handle(&object, property_key as i32))
            .unwrap_or_default()
    }

    fn color_value(&self, object_id: u32, property_key: u32) -> u32 {
        self.resolve_handle(object_id)
            .and_then(|object| CoreRegistry::get_color_handle(&object, property_key as i32))
            .unwrap_or_default() as u32
    }
}

impl crate::mechanical_port::source::animation::animation_reset::AnimationResetTarget for Artboard {
    fn resolves(&self, object_id: u32) -> bool {
        self.resolve_handle(object_id).is_some()
    }

    fn property_field_id(property_key: u32) -> u32 {
        CoreRegistry::property_field_id(property_key as i32) as u32
    }

    fn set_double(&mut self, object_id: u32, property_key: u32, value: f32) -> bool {
        self.resolve_handle(object_id).is_some_and(|object| {
            CoreRegistry::set_double_handle(&object, property_key as i32, value)
        })
    }

    fn set_color(&mut self, object_id: u32, property_key: u32, value: u32) -> bool {
        self.resolve_handle(object_id).is_some_and(|object| {
            CoreRegistry::set_color_handle(&object, property_key as i32, value as i32)
        })
    }
}

impl AdvancingComponent for Artboard {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        self.base.base.advance_component(elapsed_seconds, flags)
    }
}

impl Drop for Artboard {
    fn drop(&mut self) {
        // Focus cleanup is deliberately explicit. A StateMachineInstance may
        // already have destroyed the manager before its artboard is dropped.
        if let (Some(engine), Some(identity)) = (
            self.audio_engine.as_ref(),
            self.runtime_self.audio_identity(),
        ) {
            engine.stop_artboard(identity);
        }
        self.unbind_for_drop();

        self.data_bind_container.delete_data_binds();
        if self.is_instance {
            for object in self
                .objects
                .iter()
                .skip(1)
                .chain(self.invalid_objects.iter())
                .flatten()
            {
                self.core_arena.remove(object);
            }
            if let Some(root) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
            {
                self.core_arena.retire_runtime_artboard(&root);
            }
        }
        self.objects.clear();
        self.invalid_objects.clear();
        self.animations.clear();
        self.state_machines.clear();
        self.dirty_layout.clear();
    }
}

impl RuntimeArtboardInstanceHandle {
    pub(crate) fn from_retained(instance: Rc<RefCell<ArtboardInstance>>) -> Self {
        Self(instance)
    }

    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.with_artboard(|instance| instance.base.data_context())
    }
    pub fn internal_data_context(&self, context: RuntimeDataContextHandle) {
        Artboard::internal_data_context_handle(&self.core_handle(), context);
    }
    pub fn set_data_context(&self, context: RuntimeDataContextHandle) {
        self.internal_data_context(context);
    }
    pub fn clear_data_context(&self) {
        Artboard::clear_data_context_handle(&self.core_handle());
    }
    pub fn unbind(&self) {
        Artboard::unbind_handle(&self.core_handle());
    }
    pub fn bind(&self) {
        Artboard::bind_handle(&self.core_handle());
    }
    pub fn relink_data_context(&self) {
        Artboard::relink_data_context_handle(&self.core_handle());
    }
    pub fn bind_view_model_instance(&self, instance: Option<CoreHandle>) {
        self.bind_view_model_instance_with_parent(instance, None);
    }
    pub fn bind_view_model_instance_with_parent(
        &self,
        instance: Option<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        Artboard::bind_view_model_instance_handle(&self.core_handle(), instance, parent);
    }
    pub fn bind_view_model_instances(
        &self,
        instances: Vec<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        Artboard::bind_view_model_instances_handle(&self.core_handle(), instances, parent);
    }
    pub fn update_data_binds(&self, apply_target_to_source: bool) {
        Artboard::update_data_binds_handle(&self.core_handle(), apply_target_to_source);
    }
}

impl std::ops::Deref for Artboard {
    type Target = ArtboardBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Artboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct ArtboardInstance {
    pub base: Artboard,
}

/// Shared identity for one instantiated Artboard runtime occurrence.
///
/// Runtime consumers retain this handle (or its weak counterpart) and borrow
/// the occurrence only for the duration of a closure. This keeps animation
/// instances and other helpers from retaining pointers into a movable `Box`.
#[derive(Clone)]
pub struct RuntimeArtboardInstanceHandle(Rc<RefCell<ArtboardInstance>>);

#[derive(Clone, Default)]
pub struct RuntimeArtboardInstanceWeakHandle(Weak<RefCell<ArtboardInstance>>);

impl RuntimeArtboardInstanceHandle {
    pub fn new(mut artboard: ArtboardInstance) -> Self {
        artboard.base.runtime_self = RuntimeArtboardInstanceWeakHandle::default();
        let handle = Self(Rc::new(RefCell::new(artboard)));
        let arena = handle.0.borrow().base.core_arena.clone();
        let root = arena.insert_runtime_artboard(Rc::downgrade(&handle.0));
        {
            let mut instance = handle.0.borrow_mut();
            instance.base.runtime_self = handle.downgrade();
            crate::mechanical_port::source::core::CoreObject::set_core_handle(
                &mut instance.base,
                root.clone(),
            );
            if instance.base.objects.is_empty() {
                instance.base.objects.push(Some(root.clone()));
            } else {
                instance.base.objects[0] = Some(root.clone());
            }
            instance.base.data_bind_container.set_owner(root);
        }
        handle
    }

    pub fn core_handle(&self) -> CoreHandle {
        self.with_artboard(|instance| {
            crate::mechanical_port::source::core::CoreObject::core(&instance.base)
                .handle()
                .expect("runtime Artboard root registered before use")
        })
    }

    /// Apply the pinned ArtboardInstance width/height setters without retaining
    /// the Rust occurrence borrow across their synchronous layout callbacks.
    pub fn set_size(&self, width: f32, height: f32) {
        let root = self.core_handle();
        LayoutComponent::set_dimension_occurrence(
            &root,
            LayoutComponentBase::WIDTH_PROPERTY_KEY,
            width,
        );
        LayoutComponent::set_dimension_occurrence(
            &root,
            LayoutComponentBase::HEIGHT_PROPERTY_KEY,
            height,
        );
    }

    pub fn reset_size(&self) {
        let (width, height) = self.with_artboard(|artboard| {
            (
                artboard.base.original_width(),
                artboard.base.original_height(),
            )
        });
        self.set_size(width, height);
    }
    pub fn instance(&self) -> Option<Self> {
        Artboard::instance_from_handle(&self.core_handle())
    }

    pub fn advance_internal(&self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        Artboard::advance_internal_handle(&self.core_handle(), elapsed_seconds, flags)
    }
    pub fn apply_linear_animation(
        &self,
        animation: &mut LinearAnimation,
        time: f32,
        mix: f32,
        context: Option<&dyn crate::mechanical_port::source::animation::interpolating_keyframe::KeyFrameValueContext>,
    ) {
        let root = self.core_handle();
        let arena = root
            .retain_arena()
            .expect("live Artboard animation retains its object arena");
        let mut target = RuntimeArtboardObjectContext { arena, root };
        animation.apply(&mut target, time, mix, context);
    }
    pub fn state_machine_instance_handle(
        &self,
        index: usize,
    ) -> Option<RuntimeStateMachineInstanceHandle> {
        let (machine, context) = self.with_artboard(|artboard| {
            (
                artboard.base.state_machine_handle_at(index),
                artboard.base.data_context(),
            )
        });
        let instance = StateMachineInstance::new(machine?, self.downgrade());
        if let Some(context) = context {
            instance.with_instance_mut(|instance| instance.inherit_data_context_handle(context));
        }
        Some(instance)
    }
    pub fn default_state_machine_handle(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        let index = self.with_artboard(|artboard| artboard.base.default_state_machine_index());
        if index < 0 {
            None
        } else {
            self.state_machine_instance_handle(index as usize)
        }
    }
    pub fn animation_at(&self, index: usize) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.with_artboard(|artboard| artboard.base.animation_handle_at(index))?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self.downgrade(),
            1.0,
        )))
    }
    pub fn animation_named(&self, name: &str) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.with_artboard(|artboard| artboard.base.animation_named(name))?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self.downgrade(),
            1.0,
        )))
    }
    pub fn update_pass(&self, is_root: bool) -> bool {
        Artboard::update_pass_handle(&self.core_handle(), is_root)
    }
    pub fn build_focus_tree(
        &self,
        manager: Option<RuntimeFocusManagerHandle>,
        parent: Option<FocusNodeRef>,
    ) {
        Artboard::build_focus_tree_handle(&self.core_handle(), manager, parent);
    }
    pub fn cleanup_focus_tree(&self) {
        Artboard::cleanup_focus_tree_handle(&self.core_handle());
    }
    pub fn build_semantic_tree(
        &self,
        manager: Option<RuntimeSemanticManagerHandle>,
        parent: Option<SemanticNodeRef>,
    ) {
        Artboard::build_semantic_tree_handle(&self.core_handle(), manager, parent);
    }
    pub fn cleanup_semantic_tree(&self) {
        Artboard::cleanup_semantic_tree_handle(&self.core_handle());
    }
    pub fn collapse_semantic_boundary(&self, value: bool) {
        Artboard::collapse_semantic_boundary_handle(&self.core_handle(), value);
    }
    pub fn advance(&self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        Artboard::advance_handle(&self.core_handle(), elapsed_seconds, flags)
    }
    pub fn advance_default(&self, elapsed_seconds: f32) -> bool {
        self.advance(
            elapsed_seconds,
            AdvanceFlags(
                AdvanceFlags::ADVANCE_NESTED.0
                    | AdvanceFlags::ANIMATE.0
                    | AdvanceFlags::NEW_FRAME.0,
            ),
        )
    }
    pub fn draw(&self, renderer: &mut Renderer) {
        Artboard::draw_handle(&self.core_handle(), renderer);
    }
    pub fn draw_internal(&self, renderer: &mut Renderer) {
        Artboard::draw_internal_handle(&self.core_handle(), renderer);
    }
    pub fn internal_draw_canvases(&self) {
        Artboard::internal_draw_canvases_handle(&self.core_handle());
    }
    pub fn sync_style_changes(&self) -> bool {
        Artboard::sync_style_changes_handle(&self.core_handle())
    }

    pub fn downgrade(&self) -> RuntimeArtboardInstanceWeakHandle {
        RuntimeArtboardInstanceWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_artboard<R>(&self, f: impl FnOnce(&ArtboardInstance) -> R) -> R {
        f(&self.0.borrow())
    }

    pub fn with_artboard_mut<R>(&self, f: impl FnOnce(&mut ArtboardInstance) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

impl RuntimeArtboardInstanceWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.0.upgrade().map(RuntimeArtboardInstanceHandle)
    }

    pub fn with_artboard<R>(&self, f: impl FnOnce(&ArtboardInstance) -> R) -> Option<R> {
        self.upgrade().map(|artboard| artboard.with_artboard(f))
    }

    pub fn with_artboard_mut<R>(&self, f: impl FnOnce(&mut ArtboardInstance) -> R) -> Option<R> {
        self.upgrade().map(|artboard| artboard.with_artboard_mut(f))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }

    pub fn audio_identity(&self) -> Option<usize> {
        // AudioEngine uses this only as an identity, like C++ Artboard*. The
        // final strong reference is already gone during Artboard::drop, but
        // its weak allocation still retains the same address for stop(this).
        (!Weak::ptr_eq(&self.0, &Weak::new())).then(|| self.0.as_ptr() as usize)
    }
}

impl Default for ArtboardInstance {
    fn default() -> Self {
        Self {
            base: Artboard::default(),
        }
    }
}

impl std::ops::Deref for ArtboardInstance {
    type Target = Artboard;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ArtboardInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl crate::mechanical_port::source::virtualizing_component::Virtualizable for Artboard {
    fn virtualizable_component(&mut self) -> &mut Component {
        Artboard::virtualizable_component(self)
    }

    fn layout_x(&self) -> f32 {
        Artboard::layout_x(self)
    }

    fn layout_y(&self) -> f32 {
        Artboard::layout_y(self)
    }
}

impl crate::mechanical_port::source::virtualizing_component::Virtualizable for ArtboardInstance {
    fn virtualizable_component(&mut self) -> &mut Component {
        self.base.virtualizable_component()
    }

    fn layout_x(&self) -> f32 {
        self.base.layout_x()
    }

    fn layout_y(&self) -> f32 {
        self.base.layout_y()
    }
}

impl crate::mechanical_port::source::animation::animation_reset_factory::ResetArtboard
    for ArtboardInstance
{
    fn resolves(&self, object_id: u32) -> bool {
        crate::mechanical_port::source::animation::animation_reset_factory::ResetArtboard::resolves(
            &self.base, object_id,
        )
    }

    fn double_value(&self, object_id: u32, property_key: u32) -> f32 {
        crate::mechanical_port::source::animation::animation_reset_factory::ResetArtboard::double_value(
            &self.base,
            object_id,
            property_key,
        )
    }

    fn color_value(&self, object_id: u32, property_key: u32) -> u32 {
        crate::mechanical_port::source::animation::animation_reset_factory::ResetArtboard::color_value(
            &self.base,
            object_id,
            property_key,
        )
    }
}

impl crate::mechanical_port::source::animation::animation_reset::AnimationResetTarget
    for RuntimeArtboardInstanceHandle
{
    fn resolves(&self, object_id: u32) -> bool {
        self.with_artboard(|artboard| artboard.resolve_handle(object_id).is_some())
    }

    fn property_field_id(property_key: u32) -> u32 {
        CoreRegistry::property_field_id(property_key as i32) as u32
    }

    fn set_double(&mut self, object_id: u32, property_key: u32, value: f32) -> bool {
        let object = self.with_artboard(|artboard| artboard.resolve_handle(object_id));
        object.is_some_and(|object| {
            CoreRegistry::set_double_handle(&object, property_key as i32, value)
        })
    }

    fn set_color(&mut self, object_id: u32, property_key: u32, value: u32) -> bool {
        let object = self.with_artboard(|artboard| artboard.resolve_handle(object_id));
        object.is_some_and(|object| {
            CoreRegistry::set_color_handle(&object, property_key as i32, value as i32)
        })
    }
}

impl crate::mechanical_port::source::animation::animation_reset::AnimationResetTarget
    for ArtboardInstance
{
    fn resolves(&self, object_id: u32) -> bool {
        crate::mechanical_port::source::animation::animation_reset::AnimationResetTarget::resolves(
            &self.base, object_id,
        )
    }

    fn property_field_id(property_key: u32) -> u32 {
        <Artboard as crate::mechanical_port::source::animation::animation_reset::AnimationResetTarget>::property_field_id(property_key)
    }

    fn set_double(&mut self, object_id: u32, property_key: u32, value: f32) -> bool {
        self.base.set_double(object_id, property_key, value)
    }

    fn set_color(&mut self, object_id: u32, property_key: u32, value: u32) -> bool {
        self.base.set_color(object_id, property_key, value)
    }
}

pub enum Scene {
    StateMachine(RuntimeStateMachineInstanceHandle),
    LinearAnimation(Box<LinearAnimationInstance>),
}

impl ArtboardInstance {
    pub fn runtime_handle(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.base.runtime_self.upgrade()
    }

    pub fn runtime_weak_handle(&self) -> RuntimeArtboardInstanceWeakHandle {
        self.base.runtime_self.clone()
    }

    pub fn report_keyed_callback(
        &mut self,
        object_id: u32,
        property_key: u32,
        elapsed_seconds: f32,
        context: &mut dyn CallbackContext,
    ) -> bool {
        let Some(target) = self.base.resolve_handle(object_id) else {
            return false;
        };
        CoreRegistry::set_callback_handle(
            &target,
            property_key as i32,
            CallbackData::new(Some(context), elapsed_seconds),
        )
    }

    pub fn remove_data_bind(&mut self, bind: CoreHandle) -> bool {
        self.base.data_bind_container.remove_data_bind(bind.clone());
        self.base.core_arena.remove(&bind).is_some()
    }

    pub fn add_data_bind(&mut self, bind: CoreHandle) {
        self.base.add_data_bind(bind);
    }

    pub fn remove_runtime_object(&mut self, object: CoreHandle) -> bool {
        self.base.core_arena.remove(&object).is_some()
    }

    pub fn set_file(&mut self, file: Option<RuntimeFileWeakHandle>) {
        self.base.file = file.unwrap_or_default();
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.base.file.clone()
    }

    fn artboard_file(&self) -> Option<RuntimeFileWeakHandle> {
        self.base.artboard_file()
    }

    pub fn animation_at(&mut self, index: usize) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.base.animation_handle_at(index)?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self.runtime_weak_handle(),
            1.0,
        )))
    }

    pub fn animation_named(&mut self, name: &str) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.base.animation_named(name)?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self.runtime_weak_handle(),
            1.0,
        )))
    }

    pub fn input(&self, name: &str, path: &str) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        let nested = self.base.nested_artboard_at_path(path)?;
        nested
            .with(|object| object.as_nested_artboard()?.input(name))
            .flatten()
    }

    pub fn get_bool(&self, name: &str, path: &str) -> Option<CoreHandle> {
        let input = self.input(name, path)?;
        input
            .with_downcast::<crate::mechanical_port::source::animation::state_machine_bool::StateMachineBool, _>(|_| ())
            .is_some()
            .then_some(input)
    }

    pub fn get_number(&self, name: &str, path: &str) -> Option<CoreHandle> {
        let input = self.input(name, path)?;
        input
            .with_downcast::<crate::mechanical_port::source::animation::state_machine_number::StateMachineNumber, _>(|_| ())
            .is_some()
            .then_some(input)
    }

    pub fn get_trigger(&self, name: &str, path: &str) -> Option<CoreHandle> {
        let input = self.input(name, path)?;
        input
            .with_downcast::<crate::mechanical_port::source::animation::state_machine_trigger::StateMachineTrigger, _>(|_| ())
            .is_some()
            .then_some(input)
    }

    pub fn get_text_run(&self, name: &str, path: &str) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        let nested = self.base.nested_artboard_at_path(path)?;
        let instance = nested
            .with(|nested| nested.nested_artboard_instance_handle())
            .flatten()?;
        instance.with_artboard(|artboard| artboard.base.find_handle::<TextValueRun>(name))
    }
}

impl RuntimeArtboardInstanceHandle {
    pub fn state_machine_at(&self, index: usize) -> Option<RuntimeStateMachineInstanceHandle> {
        self.state_machine_instance_handle(index)
    }

    pub fn state_machine_named(&self, name: &str) -> Option<RuntimeStateMachineInstanceHandle> {
        let index = self.with_artboard(|artboard| {
            let machine = artboard.base.state_machine_named(name)?;
            artboard
                .base
                .state_machine_handles()
                .iter()
                .position(|candidate| candidate == &machine)
        })?;
        self.state_machine_instance_handle(index)
    }

    pub fn default_state_machine(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.default_state_machine_handle()
    }

    pub fn default_scene(&self) -> Option<Scene> {
        if let Some(instance) = self.default_state_machine() {
            return Some(Scene::StateMachine(instance));
        }
        if let Some(instance) = self.state_machine_at(0) {
            return Some(Scene::StateMachine(instance));
        }
        self.animation_at(0).map(Scene::LinearAnimation)
    }
}

impl LinearAnimationArtboard for Artboard {
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn crate::mechanical_port::source::animation::interpolating_keyframe::KeyFrameValueContext>,
    ) {
        object.with_downcast_mut::<KeyedObject, _>(|object| {
            object.apply(self, time, mix, context);
        });
    }
}

impl LinearAnimationArtboard for ArtboardInstance {
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn crate::mechanical_port::source::animation::interpolating_keyframe::KeyFrameValueContext>,
    ) {
        object.with_downcast_mut::<KeyedObject, _>(|object| {
            object.apply(&mut self.base, time, mix, context);
        });
    }
}
