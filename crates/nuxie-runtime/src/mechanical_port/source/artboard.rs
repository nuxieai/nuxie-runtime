use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::{
        keyed_object::{KeyedObject, KeyedObjectContext},
        linear_animation::{LinearAnimation, LinearAnimationArtboard},
        linear_animation_instance::LinearAnimationInstance,
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
    generated::artboard_base::{ArtboardBase, ArtboardBaseCallbacks},
    generated::core_registry::{CoreCapabilities, CoreRegistry},
    hit_info::HitInfo,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    input::{focus_manager::RuntimeFocusManagerHandle, focus_node::FocusNodeRef},
    joystick::Joystick,
    layout::layout_component::LayoutComponent,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    renderer::{RenderPath, Renderer},
    resetting_component::ResettingComponent,
    semantic::{
        semantic_data::SemanticData,
        semantic_manager::RuntimeSemanticManagerHandle,
        semantic_node::{SemanticNode, SemanticNodeRef},
        semantic_snapshot::Bounds,
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

static FRAME_ID: AtomicU64 = AtomicU64::new(0);

pub struct Artboard {
    pub base: ArtboardBase,
    core_arena: CoreArena,
    objects: Vec<Option<CoreHandle>>,
    invalid_objects: Vec<Option<CoreHandle>>,
    animations: Vec<CoreHandle>,
    state_machines: Vec<CoreHandle>,
    dependency_order: Vec<CoreHandle>,
    drawables: Vec<RuntimeDrawableOccurrence>,
    clipping_shapes: Vec<CoreHandle>,
    draw_targets: Vec<CoreHandle>,
    nested_artboards: Vec<CoreHandle>,
    component_lists: Vec<CoreHandle>,
    artboard_hosts: Vec<CoreHandle>,
    joysticks: Vec<CoreHandle>,
    resettables: Vec<CoreHandle>,
    scripted_objects: Vec<CoreHandle>,
    advancing_components: Vec<CoreHandle>,
    data_bind_container: DataBindContainer,
    data_context: Option<RuntimeDataContextHandle>,
    scripting_vm: Option<RuntimeScriptingVmHandle>,
    file: RuntimeFileWeakHandle,
    joysticks_apply_before_update: bool,
    dirt_depth: u32,
    dirt: ComponentDirt,
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
    did_change: bool,
    host: Option<CoreHandle>,
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
            dirt_depth: 0,
            dirt: ComponentDirt::FILTHY,
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
            did_change: true,
            host: None,
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

    #[cfg(test)]
    pub fn with_factory(factory: RuntimeFactoryHandle) -> Self {
        let mut artboard = Self::default();
        artboard.factory = Some(factory);
        artboard.base.base.set_clip(true);
        artboard
    }

    pub fn frame_id() -> u64 {
        FRAME_ID.load(Ordering::Relaxed)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn inc_frame_id() {
        FRAME_ID.fetch_add(1, Ordering::Relaxed);
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
        self.base.base.as_component_mut()
    }

    pub fn updates_own_layout(&self) -> bool {
        self.updates_own_layout
    }

    pub fn did_change(&self) -> bool {
        self.did_change
    }

    pub fn core_arena(&self) -> &CoreArena {
        &self.core_arena
    }

    pub fn set_core_arena(&mut self, arena: CoreArena) {
        self.core_arena = arena;
    }

    pub fn objects(&self) -> &[Option<CoreHandle>] {
        &self.objects
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
        self.artboard_source
            .clone()
            .or_else(|| crate::mechanical_port::source::core::CoreObject::core(self).handle())
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
        self.base.base.set_just_added_to_host(true);
    }

    pub(crate) fn add_object(&mut self, object: Option<CoreHandle>) {
        self.objects.push(object);
    }

    pub(crate) fn add_animation(&mut self, object: CoreHandle) {
        self.animations.push(object);
    }

    pub(crate) fn add_state_machine(&mut self, object: CoreHandle) {
        self.state_machines.push(object);
    }

    pub(crate) fn add_data_bind(&mut self, object: CoreHandle) {
        self.data_bind_container.add_data_bind(object);
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

    pub fn initialize(&mut self) -> StatusCode {
        self.base
            .base
            .set_layout(0.0, 0.0, self.width(), self.height());
        if let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            self.mark_layout_dirty(this);
        }

        for object in self.objects.clone().into_iter().flatten() {
            let code = object
                .with_mut(|object| object.on_added_dirty(self))
                .unwrap_or(StatusCode::MissingObject);
            if !can_continue(code) {
                return code;
            }
        }

        if !self.is_instance {
            for animation in self.animations.clone() {
                let code = animation
                    .with_mut(|animation| animation.on_added_dirty(self))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in self.state_machines.clone() {
                let code = state_machine
                    .with_mut(|state_machine| state_machine.on_added_dirty(self))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            if self.animations.is_empty() && self.state_machines.is_empty() {
                let Some(owner) =
                    crate::mechanical_port::source::core::CoreObject::core(self).handle()
                else {
                    return StatusCode::MissingObject;
                };
                let mut state_machine = StateMachine::default();
                state_machine.set_name("Auto Generated State Machine".into());
                let Some(state_machine) = owner.insert_sibling(state_machine) else {
                    return StatusCode::MissingObject;
                };
                self.state_machines.push(state_machine);
            }
        }

        let mut component_draw_rules = HashMap::<CoreHandle, CoreHandle>::new();
        for object in self.objects.clone().into_iter().flatten() {
            let code = object
                .with_mut(|object| object.on_added_clean(self))
                .unwrap_or(StatusCode::MissingObject);
            if !can_continue(code) {
                return code;
            }
            if object
                .with(|object| object.is_resetting_component())
                .unwrap_or(false)
            {
                self.resettables.push(object.clone());
            }
            if object.is_type_of(crate::mechanical_port::source::generated::draw_rules_base::DrawRulesBase::TYPE_KEY) {
                let parent_id = object
                    .with_downcast::<DrawRules, _>(|rules| rules.base.parent_id())
                    .unwrap_or(u32::MAX);
                if let Some(component) = self.resolve_handle(parent_id) {
                    component_draw_rules.insert(component, object.clone());
                } else {
                    eprintln!(
                        "Artboard::initialize - Draw rule targets missing component width id {}",
                        parent_id
                    );
                }
            } else if object.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY) {
                self.nested_artboards.push(object.clone());
                self.artboard_hosts.push(object.clone());
            } else if object.is_type_of(crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY) {
                self.component_lists.push(object.clone());
                self.artboard_hosts.push(object.clone());
            } else if object.is_type_of(crate::mechanical_port::source::generated::joystick_base::JoystickBase::TYPE_KEY) {
                let can_apply_before = object
                    .with_downcast_mut::<Joystick, _>(|joystick| {
                        let can_apply = joystick.can_apply_before_update();
                        joystick.add_dependents(self);
                        can_apply
                    })
                    .unwrap_or(true);
                if !can_apply_before {
                    self.joysticks_apply_before_update = false;
                }
                self.joysticks.push(object.clone());
            }
            if object
                .with(|object| object.is_advancing_component())
                .unwrap_or(false)
            {
                self.advancing_components.push(object);
            }
        }

        if !self.is_instance {
            for animation in self.animations.clone() {
                let code = animation
                    .with_mut(|animation| animation.on_added_clean(self))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in self.state_machines.clone() {
                let code = state_machine
                    .with_mut(|state_machine| state_machine.on_added_clean(self))
                    .unwrap_or(StatusCode::MissingObject);
                if !can_continue(code) {
                    return code;
                }
            }
        }

        for object in self.objects.clone().into_iter().flatten() {
            object.with_mut(|object| {
                object.component_build_dependencies();
            });
            let is_drawable = object
                .with(|object| object.as_drawable().is_some())
                .unwrap_or(false);
            if is_drawable
                && crate::mechanical_port::source::core::CoreObject::core(self)
                    .handle()
                    .as_ref()
                    != Some(&object)
            {
                self.drawables
                    .push(RuntimeDrawableOccurrence::authored(object.clone()));
                if object.is_type_of(
                    crate::mechanical_port::source::generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase::TYPE_KEY,
                ) {
                    let parent = object
                        .with(|object| object.component_parent_handle())
                        .flatten();
                    let mut index = self.drawables.len() - 1;
                    while index >= 1 {
                        let swapping = self.drawables[index - 1].authored_handle();
                        self.drawables.swap(index - 1, index);
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
                self.clipping_shapes.push(object);
            }
        }

        let mut layouts = Vec::<CoreHandle>::new();
        let mut i = 0;
        while i < self.drawables.len() {
            let drawable = self.drawables[i].clone();
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
                        self.drawables.insert(i, proxy);
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
                self.drawables.push(proxy);
            }
        }

        self.sort_dependencies();
        let rules_list: Vec<CoreHandle> = self
            .objects
            .iter()
            .flatten()
            .filter_map(|object| component_draw_rules.get(object).cloned())
            .collect();
        let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return StatusCode::MissingObject;
        };
        let Some(root) = owner.insert_sibling(DrawTarget::default()) else {
            return StatusCode::MissingObject;
        };
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
                root.with_mut(|root| root.component_add_dependent(target.clone()));
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
                    for dependent_target in self.objects.iter().flatten() {
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
        let mut draw_target_order = Vec::<CoreHandle>::new();
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default()
            .sort(root.clone(), &mut draw_target_order);
        self.core_arena.remove(&root);
        self.draw_targets.extend(
            draw_target_order
                .into_iter()
                .filter(|target| target != &root)
                .filter(|target| {
                    target.is_type_of(
                        crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
                    )
                }),
        );
        self.init_scripted_objects();
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
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default()
            .sort(root, &mut self.dependency_order);
        for (graph_order, component) in self.dependency_order.iter().cloned().enumerate() {
            component.with_mut(|component| {
                component.component_set_graph_order(graph_order as u32);
            });
        }
        self.dirt |= ComponentDirt::COMPONENTS;
    }

    fn init_scripted_objects(&mut self) {
        if self.is_instance {
            for object in self.scripted_objects.clone() {
                object.with_mut(|object_owner| {
                    let Some(object) = object_owner.as_scripted_object_mut() else {
                        return;
                    };
                    let Some(script_asset) = object.script_asset() else {
                        return;
                    };
                    if !object.user_lua_init_done() {
                        script_asset.with_downcast_mut::<
                            crate::mechanical_port::source::assets::script_asset::ScriptAsset,
                            _,
                        >(|script_asset| script_asset.init_scripted_object(object));
                    }
                    object.hydrate_script_inputs();
                });
            }
        }
    }

    pub fn poll_async_work(&mut self) {
        crate::mechanical_port::source::r#async::work_pool::rive_poll_async_work();
    }

    pub fn draw_canvases(&mut self) {
        self.internal_draw_canvases();
    }

    pub fn advance_scripted_view_models(&mut self) {
        if let Some(vm) = &self.scripting_vm {
            vm.with_vm_mut(|vm| {
                vm.advance_detached_view_models();
            });
        }
    }

    pub fn internal_draw_canvases(&mut self) {
        for object in self.scripted_objects.clone() {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.script_draw_canvas();
                }
            });
        }
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                let Some(host) = host.as_artboard_host_mut() else {
                    return;
                };
                for i in 0..host.artboard_count() as i32 {
                    if let Some(nested) = host.artboard_instance(i) {
                        nested.with_artboard_mut(ArtboardInstance::internal_draw_canvases);
                    }
                }
            });
        }
    }

    pub fn find_draw_canvas_luau_state(&self) -> Option<*mut ()> {
        for object in &self.scripted_objects {
            let state = object
                .with(|object| {
                    object
                        .as_scripted_object()
                        .and_then(|object| object.draws_canvas().then(|| object.state()))
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
                            nested.with_artboard_mut(ArtboardInstance::find_draw_canvas_luau_state)
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
        self.did_change = true;
        self.dirt |= ComponentDirt::COMPONENTS;
        if graph_order < self.dirt_depth {
            self.dirt_depth = graph_order;
        }
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {
        self.dirt |= ComponentDirt::COMPONENTS;
    }

    pub fn has_component_dirt(&self) -> bool {
        self.dirt.contains(ComponentDirt::COMPONENTS)
    }

    pub fn propagate_size(&mut self) {
        self.add_dirt(ComponentDirt::PATH, false);
        if self.shares_layout_with_host() {
            if let Some(host) = &self.host {
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

    fn shares_layout_with_host(&self) -> bool {
        self.host.as_ref().is_some_and(|host| {
            host.with(|host| {
                host.as_artboard_host()
                    .is_some_and(ArtboardHost::is_layout_provider)
            })
            .unwrap_or(false)
        })
    }

    pub fn set_host(&mut self, host: Option<CoreHandle>) {
        self.added_to_host();
        self.host = host;
        let this = crate::mechanical_port::source::core::CoreObject::core(self).handle();
        if self.shares_layout_with_host()
            && let Some(this) = this
        {
            if let Some(parent) = self.parent_artboard() {
                parent.with_downcast_mut::<Artboard, _>(|parent| {
                    parent.mark_layout_dirty(this);
                    parent.sync_layout_children();
                });
            }
        }
    }

    pub fn set_host_handle(&mut self, host: Option<CoreHandle>) {
        self.set_host(host);
    }

    pub fn host(&self) -> Option<CoreHandle> {
        self.host.clone()
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
        self.host.as_ref().and_then(|host| {
            host.with(|host| host.as_artboard_host()?.parent_artboard())
                .flatten()
        })
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

    fn update_render_path(&mut self) {
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
        self.base.base.local_path_mut().rewind();
        self.base.base.local_path_mut().add_rect(background);
        self.base.base.world_path_mut().rewind();
        self.base.base.world_path_mut().add_rect(clip);
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        if value.contains(ComponentDirt::DRAW_ORDER) {
            self.sort_draw_order();
        }
        if value.contains(ComponentDirt::CLIPPING) {
            self.clear_redundant_operations();
        }
        if value.contains(ComponentDirt::LAYOUT_STYLE) {
            let cascade_changed = self.base.base.cascade_layout_style(
                self.base.base.interpolation(),
                self.base.base.interpolator(),
                self.base.base.interpolation_time(),
                self.base.base.actual_direction(),
            );
            self.sync_style_changes_with_update(cascade_changed);
        }
        self.host_transform_marked_dirty = false;
    }

    pub fn add_dirty_data_bind(&mut self, data_bind: CoreHandle) {
        let target = data_bind
            .with_downcast::<DataBind, _>(DataBind::target)
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

    pub fn update_data_binds(&mut self, apply_target_to_source: bool) {
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.update_data_binds();
                }
            });
        }
        self.data_bind_container
            .update_data_binds(apply_target_to_source);
    }

    pub fn update_data_binds_default(&mut self) {
        self.update_data_binds(true);
    }

    pub fn update_components(&mut self) -> bool {
        if !self.dirt.contains(ComponentDirt::COMPONENTS) {
            return false;
        }
        let max_steps = 100;
        let mut step = 0;
        let count = self.dependency_order.len();
        while self.dirt.contains(ComponentDirt::COMPONENTS) && step < max_steps {
            self.dirt = ComponentDirt(self.dirt.0 & !ComponentDirt::COMPONENTS.0);
            for i in 0..count {
                let component = self.dependency_order[i].clone();
                self.dirt_depth = i as u32;
                let dirt = component
                    .with(|component| component.component_dirt())
                    .flatten()
                    .unwrap_or(ComponentDirt::NONE);
                if dirt == ComponentDirt::NONE || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }
                let constraints = component
                    .with_mut(|component| {
                        component.component_set_dirt(ComponentDirt::NONE);
                        component.component_update(dirt);
                        component.transform_component_constraint_handles()
                    })
                    .unwrap_or_default();
                crate::mechanical_port::source::transform_component::TransformComponent::apply_constraints(
                    component,
                    constraints,
                );
                if self.dirt_depth < i as u32 {
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

    pub fn mark_layout_dirty(&mut self, layout_component: CoreHandle) {
        assert!(!self.is_cleaning_dirty_layouts);
        if self.is_cleaning_dirty_layouts {
            eprintln!(
                "Artboard::markLayoutDirty - trying to mark a layout dirty during clean pass!"
            );
            return;
        }
        #[cfg(feature = "tools")]
        if self.dirty_layout.is_empty()
            && let Some(callback) = self.layout_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.dirty_layout.insert(layout_component);
        if self.is_instance {
            if self.shares_layout_with_host() {
                if let Some(host) = self.host.clone() {
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

    pub fn mark_host_transform_dirty(&mut self) {
        #[cfg(feature = "tools")]
        if !self.host_transform_marked_dirty
            && let Some(callback) = self.transform_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.host_transform_marked_dirty = true;
        if let Some(host) = self.host.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.mark_host_transform_dirty();
                }
            });
        }
    }

    pub fn sync_style_changes_with_update(&mut self, force_update: bool) {
        if self.sync_style_changes() && (self.updates_own_layout || force_update) {
            self.calculate_layout();
            self.base.base.update_layout_bounds(true);
        }
    }

    pub fn sync_style_changes(&mut self) -> bool {
        let mut updated = false;
        self.is_cleaning_dirty_layouts = true;
        if !self.dirty_layout.is_empty() {
            let this = crate::mechanical_port::source::core::CoreObject::core(self).handle();
            for layout in self.dirty_layout.iter().cloned() {
                if this.as_ref() == Some(&layout) {
                    self.base.base.sync_style();
                    continue;
                }
                layout.with_mut(|layout| {
                    if let Some(artboard) = layout.as_artboard_mut() {
                        if !artboard.updates_own_layout() {
                            artboard.sync_style_changes();
                        }
                    } else if let Some(layout) = layout.as_layout_component_mut() {
                        layout.sync_style();
                    }
                });
            }
            self.dirty_layout.clear();
            updated = true;
        }
        self.is_cleaning_dirty_layouts = false;
        updated
    }

    pub fn calculate_layout(&mut self) {
        self.base.base.calculate_layout_internal(f32::NAN, f32::NAN);
    }

    pub fn update_pass(&mut self, _is_root: bool) -> bool {
        self.update_data_binds(true);
        let mut did_update = false;
        self.sync_style_changes_with_update(false);
        self.host_transform_marked_dirty = false;
        if self.joysticks_apply_before_update {
            for joystick in self.joysticks.clone() {
                joystick.with_downcast_mut::<Joystick, _>(|joystick| joystick.apply(self));
            }
        }
        if self.update_components() {
            did_update = true;
        }
        if !self.joysticks_apply_before_update {
            for joystick in self.joysticks.clone() {
                if !joystick
                    .with_downcast::<Joystick, _>(Joystick::can_apply_before_update)
                    .unwrap_or(false)
                {
                    self.update_data_binds(true);
                    if self.update_components() {
                        did_update = true;
                    }
                }
                joystick.with_downcast_mut::<Joystick, _>(|joystick| joystick.apply(self));
            }
            self.update_data_binds(true);
            if self.update_components() {
                did_update = true;
            }
        }
        if did_update {
            self.update_data_binds(true);
        }
        did_update
    }

    pub fn advance_internal(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        let mut did_update = false;
        for advancing in self.advancing_components.clone() {
            if advancing
                .with_mut(|advancing| {
                    advancing
                        .advancing_component_advance(elapsed_seconds, flags)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            {
                did_update = true;
            }
        }
        if self.data_bind_container.advance_data_binds(elapsed_seconds) {
            did_update = true;
        }
        did_update
    }

    pub fn advance_internal_default(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_internal(
            elapsed_seconds,
            AdvanceFlags(
                AdvanceFlags::ADVANCE_NESTED.0
                    | AdvanceFlags::ANIMATE.0
                    | AdvanceFlags::NEW_FRAME.0,
            ),
        )
    }

    pub fn reset(&mut self) {
        if self.resettables.is_empty() {
            return;
        }
        for resettable in self.resettables.clone() {
            resettable.with_mut(|resettable| {
                resettable.resetting_component_reset();
            });
        }
    }

    pub fn advance(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        self.poll_async_work();
        let advancing_flags = AdvanceFlags(flags.0 | AdvanceFlags::IS_ROOT.0);
        let mut did_update = self.advance_internal(elapsed_seconds, advancing_flags);
        if self.update_pass(true) {
            did_update = true;
        }
        did_update || self.dirt.contains(ComponentDirt::COMPONENTS)
    }

    pub fn advance_default(&mut self, elapsed_seconds: f32) -> bool {
        self.advance(
            elapsed_seconds,
            AdvanceFlags(
                AdvanceFlags::ADVANCE_NESTED.0
                    | AdvanceFlags::ANIMATE.0
                    | AdvanceFlags::NEW_FRAME.0,
            ),
        )
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
        if let Some(host) = self.host.clone() {
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
        if self.host.is_some() && self.is_instance {
            let host = self.host.clone().unwrap();
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
        self.base
            .base
            .hit_test_point(position, skip_on_unclipped, is_primary_hit)
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        FRAME_ID.fetch_add(1, Ordering::Relaxed);
        self.draw_canvases();
        self.draw_internal(renderer);
    }

    pub fn draw_internal(&mut self, renderer: &mut Renderer) {
        self.did_change = false;
        if self.child_opacity() == 0.0 {
            return;
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
        if self.clip() {
            let path = self.base.base.local_path_mut().render_path(self);
            renderer.clip_path(path);
        }
        let world_transform = self.world_transform();
        for paint in self.base.base.shape_paints_mut() {
            if !paint.should_draw() {
                continue;
            }
            let Some(path) = paint.pick_path(self) else {
                continue;
            };
            paint.draw(renderer, path, world_transform);
        }
        let mut empty_clips = 0;
        let mut pending_clip_operations = Vec::<RuntimeDrawableOccurrence>::new();
        let mut drawable = self.first_drawable.clone();
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
        if save {
            renderer.restore();
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
        self.add_dirt(ComponentDirt::PATH | ComponentDirt::COMPONENTS, false);
        self.mark_host_transform_dirty();
    }

    pub fn origin_y_changed(&mut self) {
        self.add_dirt(ComponentDirt::PATH | ComponentDirt::COMPONENTS, false);
        self.mark_host_transform_dirty();
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
            self.x(),
            self.y(),
            self.base.base.layout().width(),
            self.base.base.layout().height(),
        )
    }

    pub fn is_translucent(&self) -> bool {
        for paint in self.base.base.shape_paints() {
            if !paint.is_translucent() {
                return false;
            }
        }
        true
    }

    pub fn has_audio(&mut self) -> bool {
        if self.objects.iter().flatten().any(|object| {
            object.core_type()
                == crate::mechanical_port::source::generated::audio_event_base::AudioEventBase::TYPE_KEY
        }) {
            return true;
        }
        for host in self.artboard_hosts.clone() {
            let has_audio = host
                .with_mut(|host| {
                    let host = host.as_artboard_host_mut()?;
                    Some((0..host.artboard_count() as i32).any(|index| {
                        host.artboard_instance(index)
                            .is_some_and(|instance| instance.with_artboard_mut(Artboard::has_audio))
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
            let object = self.resolve_handle(keyed_object.object_id());
            for paint in self.base.base.shape_paints() {
                if object.as_ref() == Some(paint) {
                    return true;
                }
            }
        }
        self.is_translucent()
    }

    pub fn is_animation_instance_translucent(&self, instance: &LinearAnimationInstance) -> bool {
        self.is_animation_translucent(instance.animation_definition().as_linear_animation())
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
                    .with_downcast::<NestedArtboard, _>(|nested| nested.name() == name)
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
                .with_downcast::<NestedArtboard, _>(|nested| nested.artboard_instance_handle(0))
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
        let result = self.base.deserialize(property_key, reader, self);
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
        let Some(backboard_importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::backboard::Backboard::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        debug_assert!(self.objects.is_empty());
        self.add_object(Some(artboard.clone()));
        let result = self
            .base
            .base
            .as_component_mut()
            .base
            .base
            .import(import_stack);
        if result == StatusCode::Ok {
            backboard_importer.add_artboard(artboard);
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
        self.base.base.world_path_mut()
    }

    #[cfg(test)]
    pub fn background_path(
        &mut self,
    ) -> &mut crate::mechanical_port::source::shapes::paint::shape_paint_path::ShapePaintPath {
        self.base.base.local_path_mut()
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
        if !self.did_change {
            self.did_change = true;
            if let Some(parent) = self.parent_artboard() {
                parent.with_downcast_mut::<Artboard, _>(Artboard::changed);
            }
        }
    }

    fn has_parent_focus_data(focus_data: &CoreHandle) -> bool {
        let mut current = focus_data
            .with(|focus_data| focus_data.component_parent_handle())
            .flatten();
        while let Some(parent) = current {
            let contains_other_focus = parent
                .with(|parent| {
                    parent.as_container_component().is_some_and(|container| {
                        container.children().iter().any(|child| {
                            child != focus_data
                                && child
                                    .with(|child| child.as_focus_data().is_some())
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
                    .with(|object| object.as_focus_data().is_some())
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
                    .with(|object| object.as_focus_data().is_some())
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
            .with(|component| component.nested_animations())
            .flatten();
        if let Some(animations) = nested_animations {
            let mut rewired = false;
            for animation in animations {
                animation.with_downcast_mut::<NestedStateMachine, _>(|nested_state_machine| {
                    if let Some(instance) = nested_state_machine.state_machine_instance() {
                        instance.with_instance_mut(|instance| {
                            instance.set_external_focus_manager_handle(focus_manager.clone())
                        });
                        rewired = true;
                    }
                });
            }
            component.with_downcast_mut::<NestedArtboard, _>(|nested_host| {
                nested_host.sync_nested_focus_tree(focus_node.clone(), true, rewired)
            });
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
        let direct_focus_data = children.iter().find(|child| {
            child
                .with(|child| child.as_focus_data().is_some())
                .unwrap_or(false)
        });
        let recurse_with = direct_focus_data
            .and_then(|focus_data| {
                focus_data
                    .with_mut(|focus_data| {
                        focus_data
                            .as_focus_data_mut()
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
            if child
                .with(|child| child.as_focus_data().is_some())
                .unwrap_or(false)
            {
                continue;
            }
            Self::build_focus_tree_visit(focus_manager, child, recurse_with.clone());
        }
    }

    pub fn build_focus_tree(
        &mut self,
        focus_manager: Option<RuntimeFocusManagerHandle>,
        parent_focus_node: Option<FocusNodeRef>,
    ) {
        let Some(focus_manager) = focus_manager else {
            return;
        };
        self.active_focus_manager = Some(focus_manager.clone());
        #[cfg(feature = "tools")]
        if let Some(parent) = parent_focus_node.clone() {
            self.external_parent_focus_node = Some(parent);
        }
        #[cfg(feature = "tools")]
        let effective_parent =
            parent_focus_node.or_else(|| self.external_parent_focus_node.clone());
        #[cfg(not(feature = "tools"))]
        let effective_parent = parent_focus_node;
        if let Some(root) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            Self::build_focus_tree_visit(&focus_manager, root, effective_parent);
        }
    }

    pub fn build_focus_tree_from_parent(&mut self, parent: Option<FocusNodeRef>) {
        let Some(parent) = parent else {
            return;
        };
        let Some(manager) = self.active_focus_manager.clone() else {
            return;
        };
        self.build_focus_tree(Some(manager), Some(parent));
    }

    pub fn cleanup_focus_tree(&mut self) {
        let Some(manager) = self.active_focus_manager.clone() else {
            return;
        };
        for object in self.objects.iter().flatten() {
            object.with_mut(|object| {
                if let Some(focus_data) = object.as_focus_data_mut()
                    && let Some(node) = focus_data.existing_focus_node()
                {
                    manager.with_focus_manager_mut(|manager| manager.remove_child(node));
                }
            });
        }
        for nested_host in self.nested_artboards.clone() {
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                nested.with_artboard_mut(|nested| {
                    if nested
                        .active_focus_manager
                        .as_ref()
                        .is_some_and(|nested_manager| nested_manager.ptr_eq(&manager))
                    {
                        nested.cleanup_focus_tree();
                    }
                });
            }
        }
        for list in self.component_lists.clone() {
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
                nested.with_artboard_mut(|nested| {
                    if nested
                        .active_focus_manager
                        .as_ref()
                        .is_some_and(|nested_manager| nested_manager.ptr_eq(&manager))
                    {
                        nested.cleanup_focus_tree();
                    }
                });
            }
        }
        for list in self.component_lists.clone() {
            list.with_downcast_mut::<ArtboardComponentList, _>(
                ArtboardComponentList::remove_list_scope_focus_node,
            );
        }
        self.active_focus_manager = None;
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
        self.base.base.as_component_mut().collapse(value);
    }

    pub fn build_semantic_tree(
        &mut self,
        semantic_manager: Option<RuntimeSemanticManagerHandle>,
        parent_semantic_node: Option<SemanticNodeRef>,
    ) {
        let Some(semantic_manager) = semantic_manager else {
            return;
        };
        self.active_semantic_manager = Some(semantic_manager.clone());
        let mut effective_parent = parent_semantic_node.clone();
        if self.host.is_some() {
            if self.semantic_boundary_node.is_none() {
                let boundary = SemanticNode::new(0);
                {
                    let mut boundary_mut = boundary.borrow_mut();
                    boundary_mut.is_boundary_node = true;
                    boundary_mut.boundary_artboard =
                        crate::mechanical_port::source::core::CoreObject::core(self).handle();
                }
                self.semantic_boundary_node = Some(boundary);
            }
            let boundary = self.semantic_boundary_node.clone().unwrap();
            semantic_manager.with_semantic_manager_mut(|manager| {
                manager.add_child(parent_semantic_node, boundary.clone())
            });
            self.mark_semantic_boundary_transform_dirty();
            effective_parent = Some(boundary);
        }

        for object in self.objects.iter().flatten() {
            let parent = object
                .with(|object| object.component_parent_handle())
                .flatten()
                .and_then(|parent| SemanticData::find_closest_semantic_node_handle(Some(parent)))
                .or_else(|| effective_parent.clone());
            let manager_ref = semantic_manager.semantic_manager_ref();
            object.with_mut(|candidate| {
                let core_owner = candidate.component_parent_handle();
                let Some(semantic_data) = candidate.as_semantic_data_mut() else {
                    return;
                };
                let node = semantic_data.semantic_node(
                    false,
                    core_owner,
                    object.clone(),
                    Bounds::default(),
                );
                semantic_data.attach(manager_ref, parent.clone());
                semantic_data.sync_semantic_tree_visibility(false, false, false, parent);
                debug_assert!(
                    semantic_data
                        .existing_semantic_node()
                        .is_some_and(|n| Rc::ptr_eq(&n, &node))
                );
            });
        }

        for nested_host in self.nested_artboards.clone() {
            let parent = SemanticData::find_closest_semantic_node_handle(Some(nested_host.clone()))
                .or_else(|| effective_parent.clone());
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                nested.with_artboard_mut(|nested| {
                    if !nested.semantic_manager_is(&semantic_manager) {
                        nested.cleanup_semantic_tree();
                        nested.build_semantic_tree(Some(semantic_manager.clone()), parent);
                    }
                });
            }
        }
        for list in self.component_lists.clone() {
            let parent = SemanticData::find_closest_semantic_node_handle(Some(list.clone()))
                .or_else(|| effective_parent.clone());
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
                nested.with_artboard_mut(|nested| {
                    if !nested.semantic_manager_is(&semantic_manager) {
                        nested.cleanup_semantic_tree();
                        nested.build_semantic_tree(Some(semantic_manager.clone()), parent.clone());
                    }
                });
            }
        }
    }

    fn semantic_manager_is(&self, manager: &RuntimeSemanticManagerHandle) -> bool {
        self.active_semantic_manager
            .as_ref()
            .is_some_and(|current| current.ptr_eq(manager))
    }

    pub fn cleanup_semantic_tree(&mut self) {
        let Some(manager) = self.active_semantic_manager.clone() else {
            return;
        };
        for nested_host in self.nested_artboards.clone() {
            let nested = nested_host
                .with(|nested| nested.nested_artboard_instance_handle())
                .flatten();
            if let Some(nested) = nested {
                nested.with_artboard_mut(|nested| {
                    if nested.semantic_manager_is(&manager) {
                        nested.cleanup_semantic_tree();
                    }
                });
            }
        }
        for list in self.component_lists.clone() {
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
                nested.with_artboard_mut(|nested| {
                    if nested.semantic_manager_is(&manager) {
                        nested.cleanup_semantic_tree();
                    }
                });
            }
        }
        for object in self.objects.iter().flatten() {
            object.with_mut(|object| {
                if let Some(semantic_data) = object.as_semantic_data_mut()
                    && let Some(node) = semantic_data.existing_semantic_node()
                    && semantic_data.manager_is(&manager.semantic_manager_ref())
                {
                    manager.with_semantic_manager_mut(|manager| manager.remove_child(&node));
                }
            });
        }
        if let Some(boundary) = self.semantic_boundary_node.take() {
            manager.with_semantic_manager_mut(|manager| manager.remove_child(&boundary));
        }
        self.active_semantic_manager = None;
    }

    fn collapse_boundary_subtree(node: &SemanticNodeRef, value: bool) {
        let children = node.borrow().children().to_vec();
        for child in children {
            let semantic_data = child.borrow().semantic_data.clone();
            if let Some(semantic_data) = semantic_data {
                semantic_data.with_mut(|semantic_data| {
                    if let Some(semantic_data) = semantic_data.as_semantic_data_mut()
                        && semantic_data.is_collapsed() != value
                    {
                        semantic_data.collapse(value, child.borrow().parent());
                    }
                });
            }
            Self::collapse_boundary_subtree(&child, value);
        }
    }

    pub fn collapse_semantic_boundary(&mut self, value: bool) {
        if self.active_semantic_manager.is_none() {
            return;
        }
        if value {
            if let Some(boundary) = self.semantic_boundary_node.as_ref() {
                Self::collapse_boundary_subtree(boundary, true);
            } else {
                self.collapse_semantic_objects(value);
            }
        } else {
            self.collapse_semantic_objects(value);
            self.mark_semantic_boundary_transform_dirty();
        }
    }

    fn collapse_semantic_objects(&mut self, value: bool) {
        for object in self.objects.iter().flatten() {
            object.with_mut(|object| {
                if let Some(semantic_data) = object.as_semantic_data_mut()
                    && semantic_data.is_collapsed() != value
                {
                    semantic_data.collapse(value, None);
                }
            });
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
        &self,
        object: &CoreHandle,
        clone: CoreHandle,
        artboard: &mut Artboard,
    ) {
        for data_bind_handle in self.data_bind_container.data_binds() {
            let matches = data_bind_handle
                .with(|data_bind| {
                    data_bind
                        .as_data_bind()
                        .is_some_and(|data_bind| data_bind.target().as_ref() == Some(object))
                })
                .unwrap_or(false);
            if !matches {
                continue;
            }
            let Some(data_bind_clone) = data_bind_handle
                .with(|data_bind| data_bind.clone_boxed())
                .flatten()
            else {
                continue;
            };
            let clone_handle = artboard.core_arena.insert_boxed(data_bind_clone);
            let (file, converter) = data_bind_handle
                .with(|data_bind| {
                    let data_bind = data_bind.as_data_bind()?;
                    Some((data_bind.file(), data_bind.converter()))
                })
                .flatten()
                .unwrap_or_default();
            let converter_clone = converter
                .and_then(|converter| {
                    converter
                        .with(|converter| converter.clone_boxed())
                        .flatten()
                })
                .map(|converter| artboard.core_arena.insert_boxed(converter));
            clone_handle.with_mut(|data_bind| {
                if let Some(data_bind) = data_bind.as_data_bind_mut() {
                    data_bind.set_target(Some(clone));
                    data_bind.set_file(file);
                    data_bind.initialize();
                    data_bind.set_converter(converter_clone);
                }
            });
            artboard.data_bind_container.add_data_bind(clone_handle);
        }
    }

    pub fn internal_data_context(&mut self, value: RuntimeDataContextHandle) {
        self.data_context = Some(value.clone());
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.internal_data_context(value.clone());
                }
            });
        }
        self.data_bind_container
            .bind_data_binds_from_context(value.clone());
        self.data_bind_container.sort_data_binds();
        for scripted_object in self.scripted_objects.clone() {
            scripted_object.with_mut(|scripted_object| {
                if let Some(scripted_object) = scripted_object.as_scripted_object_mut() {
                    scripted_object.set_data_context(Some(value.clone()));
                }
            });
        }
        self.init_scripted_objects();
    }

    pub fn set_data_context(&mut self, value: RuntimeDataContextHandle) {
        self.internal_data_context(value);
    }

    pub fn rebind(&mut self) {
        if let Some(context) = self.data_context.clone() {
            self.internal_data_context(context);
        }
    }

    pub fn relink_data_context(&mut self) {
        let Some(context) = self.data_context.clone() else {
            return;
        };
        for host in self.artboard_hosts.clone() {
            let value = context.with_context(DataContext::main_view_model_instance);
            if let Some(value) = value {
                host.with_mut(|host| {
                    if let Some(host) = host.as_artboard_host_mut() {
                        host.relink_data_context(value);
                    }
                });
            }
        }
    }

    pub fn rebuild_data_bind(&mut self, data_bind: &mut DataBind) {
        if let Some(context_bind) = data_bind.as_data_bind_context_mut() {
            context_bind.bind_from_context(self.data_context.clone());
        }
    }

    pub fn unbind(&mut self) {
        self.clear_data_context();
        self.data_bind_container.unbind_data_binds();
        for host in self.artboard_hosts.clone() {
            host.with_mut(|host| {
                if let Some(host) = host.as_artboard_host_mut() {
                    host.unbind();
                }
            });
        }
    }

    pub fn clear_data_context(&mut self) {
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
                    scripted_object.set_data_context(None);
                }
            });
        }
    }

    pub fn bind_view_model_instance(&mut self, view_model_instance: Option<CoreHandle>) {
        self.bind_view_model_instance_with_parent(view_model_instance, None);
    }

    pub fn bind_view_model_instance_with_parent(
        &mut self,
        view_model_instance: Option<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        let Some(instance) = view_model_instance else {
            self.unbind();
            return;
        };
        self.set_view_model_instance(Some(instance));
        if let (Some(parent), Some(context)) = (parent, self.data_context.as_ref()) {
            context.with_context_mut(|context| context.set_parent(Some(parent)));
        }
        self.bind();
    }

    pub fn bind_view_model_instance_handle_with_parent(
        &mut self,
        view_model_instance: CoreHandle,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        self.bind_view_model_instance_with_parent(Some(view_model_instance), parent);
    }

    pub fn set_view_model_instance(&mut self, view_model_instance: Option<CoreHandle>) {
        let Some(instance) = view_model_instance else {
            return;
        };
        if self.data_context.is_none() {
            let context = RuntimeDataContextHandle::new(DataContext::new(Some(instance)));
            if let Some(owner) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
            {
                context.with_context_mut(|context| context.add_dependent_container(owner));
            }
            self.data_context = Some(context);
            return;
        }
        self.data_context
            .as_ref()
            .unwrap()
            .with_context_mut(|context| context.set_main_view_model_instance(Some(instance)));
    }

    pub fn bind_view_model_instances(
        &mut self,
        instances: Vec<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        if instances.is_empty() {
            self.unbind();
            return;
        }
        self.clear_data_context();
        let context = RuntimeDataContextHandle::new(DataContext::from_instances(instances));
        context.with_context_mut(|context| {
            if let Some(owner) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
            {
                context.add_dependent_container(owner);
            }
            context.set_parent(parent);
        });
        self.internal_data_context(context);
    }

    pub fn bind(&mut self) {
        if let Some(context) = self.data_context.clone() {
            self.internal_data_context(context);
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
                |view_model| {
                    crate::mechanical_port::source::view_model_type::ViewModelType::from_u32(
                        view_model.base.view_model_type(),
                    )
                },
            )
            .flatten()
            != Some(crate::mechanical_port::source::view_model_type::ViewModelType::Global)
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

    pub fn find_handle<T: 'static>(&self, name: &str) -> Option<CoreHandle> {
        self.objects.iter().flatten().find_map(|object| {
            object.with_downcast::<T, _>(|_| ()).is_some().then_some(())?;
            object.with_mut(|candidate| {
                (candidate.get_string(
                    crate::mechanical_port::source::generated::core_registry::CoreField::ComponentName,
                ) == name)
                    .then(|| object.clone())
            })?
        })
    }

    pub fn count<T: 'static>(&self) -> usize {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.with_downcast::<T, _>(|_| ()).is_some())
            .count()
    }

    pub fn object_handle_at<T: 'static>(&self, index: usize) -> Option<CoreHandle> {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.with_downcast::<T, _>(|_| ()).is_some())
            .nth(index)
            .cloned()
    }

    pub fn object_index(&self, component: &CoreHandle) -> i32 {
        self.objects
            .iter()
            .position(|object| object.as_ref().is_some_and(|object| object == component))
            .map_or(-1, |index| index as i32)
    }

    pub fn find_all_handles<T: 'static>(&self) -> Vec<CoreHandle> {
        self.objects
            .iter()
            .flatten()
            .filter(|object| object.with_downcast::<T, _>(|_| ()).is_some())
            .cloned()
            .collect()
    }

    pub fn instance(&self) -> Option<Box<ArtboardInstance>> {
        let mut clone = Box::new(ArtboardInstance::default());
        clone.base.base.copy(&self.base, &mut clone.base);
        clone.base.factory = self.factory.clone();
        clone.base.file = self.file.clone();
        clone.base.scripting_vm = self.scripting_vm.clone();
        clone.base.frame_origin = self.frame_origin;
        clone.base.data_context = self.data_context.clone();
        clone.base.is_instance = true;
        clone.base.original_width = self.original_width;
        clone.base.original_height = self.original_height;
        clone.base.core_arena = self.core_arena.clone();
        #[cfg(feature = "tools")]
        {
            clone.base.artboard_id = self.artboard_id;
        }
        clone.base.artboard_source = if self.is_instance {
            self.artboard_source.clone()
        } else {
            crate::mechanical_port::source::core::CoreObject::core(self).handle()
        };
        clone.base.objects.push(None);
        for object in self.objects.iter().skip(1) {
            clone
                .base
                .objects
                .push(object.as_ref().and_then(CoreHandle::clone_occurrence));
        }
        clone
            .base
            .animations
            .extend(self.animations.iter().cloned());
        clone
            .base
            .state_machines
            .extend(self.state_machines.iter().cloned());
        if clone.base.initialize() != StatusCode::Ok {
            return None;
        }
        assert!(clone.base.is_instance());
        Some(clone)
    }

    pub fn instance_handle(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance()
            .map(|instance| RuntimeArtboardInstanceHandle::new(*instance))
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
        self.base.base.world_transform()
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        self.base.base.as_component_mut().add_dirt(value, recurse)
    }

    pub fn can_have_overrides(&self) -> bool {
        true
    }

    pub fn update_world_transform(&mut self) {}
}

impl ArtboardBaseCallbacks for Artboard {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .as_component_mut()
            .base
            .base
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
        self.advance_internal(elapsed_seconds, flags)
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
        self.unbind();

        self.data_bind_container.delete_data_binds();
        self.objects.clear();
        self.invalid_objects.clear();
        self.animations.clear();
        self.state_machines.clear();
        self.dirty_layout.clear();
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
        handle.0.borrow_mut().base.runtime_self = handle.downgrade();
        handle
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
        let artboard = self.0.upgrade()?;
        Some(Rc::as_ptr(&artboard) as usize)
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

    pub fn state_machine_instance_handle(
        &mut self,
        index: usize,
    ) -> Option<RuntimeStateMachineInstanceHandle> {
        let machine = self.base.state_machine_handle_at(index)?;
        let instance = StateMachineInstance::new(machine, self.runtime_weak_handle());
        if let Some(context) = self.base.data_context() {
            instance.with_instance_mut(|instance| instance.inherit_data_context_handle(context));
        }
        Some(instance)
    }

    pub fn default_state_machine_handle(&mut self) -> Option<RuntimeStateMachineInstanceHandle> {
        let index = self.base.default_state_machine_index();
        (index >= 0)
            .then(|| self.state_machine_instance_handle(index as usize))
            .flatten()
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

    pub fn state_machine_at(&mut self, index: usize) -> Option<RuntimeStateMachineInstanceHandle> {
        self.state_machine_instance_handle(index)
    }

    pub fn state_machine_named(&mut self, name: &str) -> Option<RuntimeStateMachineInstanceHandle> {
        let machine = self.base.state_machine_named(name)?;
        let index = self
            .base
            .state_machine_handles()
            .iter()
            .position(|candidate| candidate == &machine)?;
        self.state_machine_instance_handle(index)
    }

    pub fn default_state_machine(&mut self) -> Option<RuntimeStateMachineInstanceHandle> {
        let index = self.base.default_state_machine_index();
        (index >= 0)
            .then(|| self.state_machine_at(index as usize))
            .flatten()
    }

    pub fn default_scene(&mut self) -> Option<Scene> {
        if let Some(instance) = self.default_state_machine() {
            return Some(Scene::StateMachine(instance));
        }
        if let Some(instance) = self.state_machine_at(0) {
            return Some(Scene::StateMachine(instance));
        }
        self.animation_at(0).map(Scene::LinearAnimation)
    }

    pub fn input(&self, name: &str, path: &str) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        let nested = self.base.nested_artboard_at_path(path)?;
        nested
            .with_downcast::<NestedArtboard, _>(|nested| nested.input(name))
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
