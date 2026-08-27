use std::{ptr::NonNull, rc::Rc};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::{
        listener_invocation::ListenerInvocation, nested_animation::NestedAnimation,
        nested_state_machine::NestedStateMachine,
    },
    artboard::{Artboard, ArtboardInstance},
    artboard_referencer::{ArtboardReferencer, ArtboardReferencerBehavior, CoreArtboardReferencer},
    component::Component,
    component_dirt::ComponentDirt,
    component_origin::ComponentOrigin,
    core::Core,
    data_bind::{data_bind::DataBind, data_context::DataContext},
    data_bind_path_referencer::DataBindPathReferencer,
    file::File,
    focus_data::FocusData,
    generated::nested_artboard_base::{NestedArtboardBase, NestedArtboardBaseCallbacks},
    hit_info::HitInfo,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    input::{
        focus_manager::FocusManager,
        focus_node::{FocusNode, FocusNodeRef},
        focusable::{Focusable, Key, KeyModifiers},
    },
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard_host_flags::NestedArtboardHostFlags,
    refcnt::{Rcp, ref_rcp},
    renderer::Renderer,
    scripted::scripted_drawable::ScriptedDrawable,
    status_code::StatusCode,
    view_model_type::ViewModelType,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
    },
};

fn build_vmi_list(
    primary: Option<Rcp<ViewModelInstance>>,
    globals: &[Rcp<ViewModelInstance>],
) -> Vec<Rcp<ViewModelInstance>> {
    let mut list = Vec::with_capacity(usize::from(primary.is_some()) + globals.len());
    if let Some(primary) = primary {
        list.push(primary);
    }
    for global in globals {
        list.push(global.clone());
    }
    list
}

pub struct NestedArtboard {
    pub base: NestedArtboardBase,
    pub artboard_referencer: ArtboardReferencer,
    pub data_bind_path_referencer: DataBindPathReferencer,
    instance: Option<Box<ArtboardInstance>>,
    bound_nested_state_machine: Option<Box<NestedStateMachine>>,
    nested_animations: Vec<NonNull<NestedAnimation>>,
    file: Option<NonNull<File>>,
    view_model_instance: Option<Rcp<ViewModelInstance>>,
    data_context: Option<Rc<DataContext>>,
    active_view_model_instance: Option<NonNull<ViewModelInstance>>,
    global_view_model_instances: Vec<Rcp<ViewModelInstance>>,
    focus_scope: Option<FocusNodeRef>,
    cumulated_seconds: f32,
    owns_active_vmi: bool,
    host_flags: NestedArtboardHostFlags,
}

impl Default for NestedArtboard {
    fn default() -> Self {
        Self {
            base: NestedArtboardBase::default(),
            artboard_referencer: ArtboardReferencer::default(),
            data_bind_path_referencer: DataBindPathReferencer::default(),
            instance: None,
            bound_nested_state_machine: None,
            nested_animations: Vec::new(),
            file: None,
            view_model_instance: None,
            data_context: None,
            active_view_model_instance: None,
            global_view_model_instances: Vec::new(),
            focus_scope: None,
            cumulated_seconds: 0.0,
            owns_active_vmi: false,
            host_flags: NestedArtboardHostFlags::NONE,
        }
    }
}

impl Drop for NestedArtboard {
    fn drop(&mut self) {
        // Release dependencies before the mounted instance is destroyed. The
        // nested animations' state-machine instances refer to that instance.
        for animation in &mut self.nested_animations {
            unsafe { animation.as_mut() }.release_dependencies();
        }
        if let Some(state_machine) = &mut self.bound_nested_state_machine {
            state_machine.release_dependencies();
        }

        self.view_model_instance = None;
        // The active stateful child is borrowed. A dynamically created bound
        // instance is owned here and must release its explicit reference.
        if self.owns_active_vmi {
            if let Some(mut instance) = self.active_view_model_instance {
                unsafe { instance.as_mut() }.base.unref();
            }
        }
        self.active_view_model_instance = None;
        self.owns_active_vmi = false;
        self.global_view_model_instances.clear();

        // The structural scope persists across swaps, but not host teardown.
        if let Some(scope) = self.focus_scope.take() {
            if let Some(manager) = scope.borrow().manager() {
                manager.borrow_mut().remove_child(scope.clone());
            }
        }
    }
}

impl NestedArtboard {
    pub const TYPE_KEY: u16 = NestedArtboardBase::TYPE_KEY;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_artboard_data_bound(&self) -> bool {
        self.has_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND)
    }

    pub fn sync_nested_focus_tree(
        &mut self,
        fallback_parent: Option<FocusNodeRef>,
        place_scope: bool,
        force_rebuild: bool,
    ) {
        let Some(mut parent_artboard) = self.artboard() else {
            return;
        };
        let Some(parent_focus_manager) = (unsafe { parent_artboard.as_mut() }).focus_manager()
        else {
            return;
        };

        self.register_focus_scope(parent_focus_manager, fallback_parent.clone(), place_scope);

        let Some(nested_instance) = self.artboard_instance_ptr(0) else {
            return;
        };
        let already_shared =
            unsafe { nested_instance.as_ref() }.focus_manager() == Some(parent_focus_manager);
        if !force_rebuild && already_shared {
            return;
        }

        unsafe { nested_instance.as_mut() }.cleanup_focus_tree();
        let parent = self.focus_scope.clone().or(fallback_parent);
        unsafe { nested_instance.as_mut() }.build_focus_tree(parent_focus_manager, parent);
    }

    pub fn sync_nested_focus_tree_default(&mut self, fallback_parent: Option<FocusNodeRef>) {
        self.sync_nested_focus_tree(fallback_parent, false, true);
    }

    pub fn clone_core(&self) -> Box<NestedArtboard> {
        let mut nested_artboard = Box::new(NestedArtboard::default());
        // NestedArtboardBase::clone copies the generated base before this
        // owner restores its host-specific state.
        nested_artboard.base.copy_from_source(&self.base);
        nested_artboard.file = self.file;
        if self.is_artboard_data_bound() {
            nested_artboard.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);
        }
        let Some(referenced) = self.artboard_referencer.referenced_artboard() else {
            return nested_artboard;
        };
        let instance = unsafe { referenced.as_ref() }.instance();
        nested_artboard.referenced_artboard(instance.map(NonNull::from));
        nested_artboard
    }

    fn nest(&mut self, artboard: NonNull<Artboard>) {
        self.artboard_referencer
            .set_referenced_artboard(Some(artboard));
        if !unsafe { artboard.as_ref() }.is_instance() {
            // Import only marks the source artboard; it is instanced later.
            return;
        }

        unsafe { artboard.as_mut() }.set_frame_origin(false);
        unsafe { artboard.as_mut() }.set_host_opacity(self.render_opacity());
        let volume = unsafe { artboard.as_ref() }.volume();
        unsafe { artboard.as_mut() }.set_volume(volume);
        self.instance = None;
        if unsafe { artboard.as_ref() }.is_instance() {
            self.instance = Some(unsafe { Box::from_raw(artboard.as_ptr().cast()) });
        }
        unsafe { artboard.as_mut() }.set_host(Some(NonNull::from(&mut *self)));
        self.apply_origin_override();
    }

    pub fn apply_origin_override(&mut self) {
        let Some(mut referenced) = self.artboard_referencer.referenced_artboard() else {
            return;
        };
        if !unsafe { referenced.as_ref() }.is_instance() {
            return;
        }
        for child in self.children() {
            let Some(origin) = unsafe { child.as_ref() }.as_component_origin() else {
                continue;
            };
            unsafe { referenced.as_mut() }.set_origin_x(unsafe { origin.as_ref() }.base.origin_x());
            unsafe { referenced.as_mut() }.set_origin_y(unsafe { origin.as_ref() }.base.origin_y());
            return;
        }
    }

    fn try_schedule_bind_stateful(&mut self) -> bool {
        if self.active_view_model_instance.is_some() && self.instance.is_some() {
            self.set_host_flag(NestedArtboardHostFlags::PENDING_STATEFUL_BINDING);
            return true;
        }
        false
    }

    fn bind_stateful(&mut self) {
        self.clear_host_flag(NestedArtboardHostFlags::PENDING_STATEFUL_BINDING);
        let Some(instance) = self.instance.as_deref_mut() else {
            return;
        };
        let primary = self
            .active_view_model_instance
            .map(|instance| unsafe { ref_rcp(instance.as_ptr()) });
        let list = build_vmi_list(primary, &self.global_view_model_instances);
        instance.bind_view_model_instances(list, self.data_context.clone());
        let nested_data_context = instance.data_context();
        for animation in &mut self.nested_animations {
            if let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            {
                state_machine.set_data_context(nested_data_context.clone());
            }
        }
    }

    fn find_stateful_child_vmi(&self) -> Option<NonNull<ViewModelInstance>> {
        for child in self.children() {
            if let Some(instance) = unsafe { child.as_ref() }.as_view_model_instance() {
                return Some(instance);
            }
        }
        None
    }

    fn set_active_view_model_instance(
        &mut self,
        instance: Option<NonNull<ViewModelInstance>>,
        owns: bool,
    ) {
        if self.active_view_model_instance == instance {
            return;
        }
        if self.owns_active_vmi {
            if let Some(mut current) = self.active_view_model_instance {
                unsafe { current.as_mut() }.base.unref();
            }
        }
        self.active_view_model_instance = instance;
        self.owns_active_vmi = owns;
    }

    fn clear_nested_animations(&mut self) {
        for animation in &mut self.nested_animations {
            unsafe { animation.as_mut() }.release_dependencies();
        }
        self.nested_animations.clear();
    }

    pub fn update_artboard(
        &mut self,
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
    ) {
        self.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);

        let explicit_null = view_model_instance_artboard.is_some_and(|property| unsafe {
            property.as_ref().asset().is_none() && property.as_ref().base.property_value() == -1
        });
        let parent = self.artboard();
        let artboard = if explicit_null {
            None
        } else {
            ArtboardReferencer::find_artboard(view_model_instance_artboard, parent, self.file)
        };
        if !explicit_null && artboard.is_none() {
            return;
        }

        if let Some(instance) = self.artboard_instance_ptr(0) {
            unsafe { instance.as_mut() }.cleanup_focus_tree();
        }
        self.clear_data_context();
        self.clear_nested_animations();
        self.bound_nested_state_machine = None;

        if explicit_null {
            if self.artboard_referencer.referenced_artboard().is_some() && self.instance.is_none() {
                let mut referenced = self.artboard_referencer.referenced_artboard().unwrap();
                unsafe { referenced.as_mut() }.set_host(None);
            }
            self.artboard_referencer.set_referenced_artboard(None);
            self.instance = None;
            self.set_active_view_model_instance(None, false);
            return;
        }

        if let Some(mut artboard) = artboard {
            let mut artboard_instance = unsafe { artboard.as_mut() }.instance();
            if unsafe { artboard.as_ref() }.state_machine_count() > 0 {
                let mut nested_state_machine = Box::<NestedStateMachine>::default();
                nested_state_machine.base.set_animation_id(0);
                nested_state_machine
                    .initialize_animation(artboard_instance.as_deref_mut().map(NonNull::from));
                let pointer = NonNull::from(nested_state_machine.as_mut()).cast();
                self.add_nested_animation(pointer);
                self.bound_nested_state_machine = Some(nested_state_machine);
            }
            let instance_pointer = artboard_instance
                .take()
                .map(Box::into_raw)
                .and_then(NonNull::new)
                .map(NonNull::cast);
            self.referenced_artboard(instance_pointer);

            if self.base.is_stateful() {
                let stateful_child = self.find_stateful_child_vmi();
                if stateful_child.is_some_and(|child| unsafe {
                    child.as_ref().base.view_model_id() == artboard.as_ref().base.view_model_id()
                }) {
                    self.set_active_view_model_instance(stateful_child, false);
                } else if self.owns_active_vmi
                    && self
                        .active_view_model_instance
                        .is_some_and(|active| unsafe {
                            active.as_ref().base.view_model_id()
                                == artboard.as_ref().base.view_model_id()
                        })
                {
                    // Reuse the already-owned binding for the same ViewModel.
                } else {
                    let view_model = self.file.and_then(|file| unsafe {
                        file.as_ref()
                            .view_model(artboard.as_ref().base.view_model_id())
                    });
                    let bound = match (self.file, view_model) {
                        (Some(mut file), Some(view_model)) => unsafe {
                            file.as_mut().create_default_view_model_instance(view_model)
                        },
                        _ => None,
                    };
                    let raw = bound
                        .map(|mut instance| NonNull::new(instance.release()))
                        .flatten();
                    self.set_active_view_model_instance(raw, raw.is_some());
                    if let (Some(mut file), Some(raw)) = (self.file, raw) {
                        unsafe { file.as_mut() }.complete_view_model_properties(raw);
                    }
                }
            } else {
                self.set_active_view_model_instance(None, false);
            }

            let property = view_model_instance_artboard.unwrap();
            if let Some(bound) = unsafe { property.as_ref() }.bound_view_model_instance() {
                self.bind_view_model_instance(Some(bound), self.data_context.clone());
            } else if self.try_schedule_bind_stateful() {
                self.bind_stateful();
            } else if self.data_context.is_some() && self.view_model_instance.is_none() {
                self.internal_data_context(self.data_context.clone());
            } else if self.view_model_instance.is_some() {
                self.bind_view_model_instance(
                    self.view_model_instance.clone(),
                    self.data_context.clone(),
                );
            }
            // Upstream marks the host fully dirty after a runtime swap.
            self.add_dirt(ComponentDirt::FILTHY);

            let parent_focus_manager = self
                .artboard()
                .and_then(|parent| unsafe { parent.as_ref() }.focus_manager());
            if let (Some(parent_focus_manager), Some(state_machine)) = (
                parent_focus_manager,
                self.bound_nested_state_machine.as_deref_mut(),
            ) {
                if let Some(instance) = state_machine.state_machine_instance() {
                    if unsafe { instance.as_ref() }.focus_manager() != Some(parent_focus_manager) {
                        unsafe { instance.as_mut() }
                            .set_external_focus_manager(parent_focus_manager);
                    }
                }
            }
            let fallback = FocusData::find_closest_focus_node(NonNull::from(&mut *self).cast());
            self.sync_nested_focus_tree(fallback, false, true);
        }
    }

    fn detect_artboard_data_binding(&mut self) {
        if self.is_artboard_data_bound() {
            return;
        }
        let Some(parent) = self.artboard() else {
            return;
        };
        for data_bind in unsafe { parent.as_ref() }.data_binds() {
            let data_bind = unsafe { data_bind.as_ref() };
            if data_bind.target() == Some(NonNull::from(&mut *self).cast())
                && data_bind.base.property_key() == NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY
            {
                self.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);
                return;
            }
        }
    }

    fn register_focus_scope(
        &mut self,
        focus_manager: NonNull<FocusManager>,
        parent_node: Option<FocusNodeRef>,
        place: bool,
    ) {
        if !self.is_artboard_data_bound() {
            return;
        }
        if self.focus_scope.is_none() {
            self.focus_scope = Some(FocusNode::make_structural_scope());
        }
        let scope = self.focus_scope.as_ref().unwrap().clone();
        if !place && scope.borrow().manager() == Some(focus_manager) {
            return;
        }
        unsafe { focus_manager.as_mut() }.add_child(parent_node, scope);
    }

    pub fn draw(&mut self, renderer: &mut dyn Renderer) {
        if self.needs_save_operation() {
            renderer.save();
        }
        renderer.transform(&self.world_transform());
        let referenced = self.artboard_referencer.referenced_artboard().unwrap();
        unsafe { referenced.as_mut() }.draw_internal(renderer);
        if self.needs_save_operation() {
            renderer.restore();
        }
    }

    pub fn will_draw(&self) -> bool {
        self.base.base.will_draw() && self.artboard_referencer.referenced_artboard().is_some()
    }

    pub fn hit_test(&mut self, hit_info: &mut HitInfo, transform: &Mat2D) -> Option<NonNull<Core>> {
        let referenced = self.artboard_referencer.referenced_artboard()?;
        hit_info.mounts.push(NonNull::from(&mut *self).cast());
        let mounted_translation = make_translate(unsafe { referenced.as_ref() });
        let mounted_transform = *transform * self.world_transform() * mounted_translation;
        if let Some(component) =
            unsafe { referenced.as_mut() }.hit_test(hit_info, &mounted_transform)
        {
            return Some(component);
        }
        hit_info.mounts.pop();
        None
    }

    pub fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        _artboard: Option<NonNull<ArtboardInstance>>,
    ) -> bool {
        let mounted_position = self.world_transform() * *position;
        self.parent().is_some_and(|parent| {
            unsafe { parent.as_ref() }.hit_test_point(mounted_position, skip_on_unclipped, false)
        })
    }

    pub fn host_transform_point(
        &self,
        point: &Vec2D,
        _artboard_instance: Option<NonNull<ArtboardInstance>>,
    ) -> Vec2D {
        let local = Vec2D::transform_mat2d(*point, self.world_transform());
        self.artboard().map_or(local, |artboard| {
            unsafe { artboard.as_ref() }.root_transform(local)
        })
    }

    pub fn world_transform_for_artboard(
        &self,
        _artboard_instance: Option<NonNull<ArtboardInstance>>,
    ) -> Mat2D {
        self.world_transform()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        let Some(backboard_importer) =
            import_stack.latest::<BackboardImporter>(BackboardImporter::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        backboard_importer.add_artboard_referencer(NonNull::from(&mut *self).cast());
        self.base.base.import(import_stack)
    }

    pub fn add_nested_animation(&mut self, nested_animation: NonNull<NestedAnimation>) {
        self.nested_animations.push(nested_animation);
    }

    pub fn on_added_clean(&mut self, context: NonNull<Core>) -> StatusCode {
        debug_assert!(
            self.artboard_referencer.referenced_artboard().is_none()
                || self.artboard_referencer.referenced_artboard()
                    == self
                        .instance
                        .as_deref_mut()
                        .map(NonNull::from)
                        .map(NonNull::cast)
        );

        if let Some(instance) = self.instance.as_deref_mut() {
            let instance_pointer = NonNull::from(&mut *instance);
            for animation in &mut self.nested_animations {
                unsafe { animation.as_mut() }.initialize_animation(Some(instance_pointer));
            }
            unsafe { instance_pointer.cast::<Artboard>().as_mut() }
                .set_host(Some(NonNull::from(&mut *self)));
            self.apply_origin_override();
        }

        for child in self.children() {
            let Some(mut instance) = unsafe { child.as_ref() }.as_view_model_instance() else {
                continue;
            };
            let view_model = unsafe { instance.as_ref() }.get_view_model();
            let view_model_type = view_model
                .and_then(|view_model| {
                    ViewModelType::from_u32(unsafe { view_model.as_ref().base.view_model_type() })
                })
                .unwrap_or(ViewModelType::Standard);
            unsafe { self.file.unwrap().as_mut() }.complete_view_model_properties(instance);
            if view_model_type == ViewModelType::Global {
                self.global_view_model_instances
                    .push(unsafe { ref_rcp(instance.as_ptr()) });
            } else if self.active_view_model_instance.is_none() {
                self.active_view_model_instance = Some(instance);
                self.owns_active_vmi = false;
            }
        }
        self.try_schedule_bind_stateful();
        self.detect_artboard_data_binding();

        self.base.base.on_added_clean(context)
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        let Some(mut referenced) = self.artboard_referencer.referenced_artboard() else {
            return;
        };
        if value.contains(ComponentDirt::WORLD_TRANSFORM) {
            if let Some(instance) = self.instance.as_deref_mut() {
                instance.mark_semantic_boundary_transform_dirty();
            }
        }
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            unsafe { referenced.as_mut() }.set_host_opacity(self.render_opacity());
        }
        if value.contains(ComponentDirt::COMPONENTS)
            || (self.base.is_paused()
                && unsafe { referenced.as_ref() }.has_dirt(ComponentDirt::COMPONENTS))
        {
            unsafe { referenced.as_mut() }.update_pass(false);
        }
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.collapse(value) {
            return false;
        }
        let Some(instance) = self.instance.as_deref_mut() else {
            return true;
        };
        instance.collapse_semantic_boundary(value);
        true
    }

    pub fn has_nested_state_machines(&self) -> bool {
        self.nested_animations
            .iter()
            .any(|animation| unsafe { animation.as_ref().as_nested_state_machine().is_some() })
    }

    pub fn nested_animations(&mut self) -> &mut [NonNull<NestedAnimation>] {
        &mut self.nested_animations
    }

    pub fn nested_artboard(&mut self, name: &str) -> Option<NonNull<NestedArtboard>> {
        self.instance.as_deref_mut()?.nested_artboard(name)
    }

    pub fn state_machine(&mut self, name: &str) -> Option<NonNull<NestedStateMachine>> {
        for animation in &mut self.nested_animations {
            let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            else {
                continue;
            };
            if state_machine.base.base.name() == name {
                return Some(NonNull::from(state_machine));
            }
        }
        None
    }

    pub fn input(&mut self, name: &str) -> Option<NonNull<Core>> {
        self.input_for_state_machine(name, "")
    }

    pub fn input_for_state_machine(
        &mut self,
        name: &str,
        state_machine_name: &str,
    ) -> Option<NonNull<Core>> {
        if !state_machine_name.is_empty() {
            return self
                .state_machine(state_machine_name)
                .and_then(|machine| unsafe { machine.as_mut() }.input(name));
        }
        for animation in &mut self.nested_animations {
            let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            else {
                continue;
            };
            if let Some(input) = state_machine.input(name) {
                return Some(input);
            }
        }
        None
    }

    pub fn world_to_local(&self, world: Vec2D, local: &mut Vec2D) -> bool {
        if self.artboard_referencer.referenced_artboard().is_none() {
            return false;
        }
        let Some(inverse) = self.world_transform().invert() else {
            return false;
        };
        *local = inverse * world;
        true
    }

    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        let maximum_width = if width_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            width
        };
        let maximum_height = if height_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            height
        };
        Vec2D::new(
            maximum_width.min(
                self.instance
                    .as_deref()
                    .map_or(0.0, ArtboardInstance::width),
            ),
            maximum_height.min(
                self.instance
                    .as_deref()
                    .map_or(0.0, ArtboardInstance::height),
            ),
        )
    }

    pub fn control_size(
        &mut self,
        _size: Vec2D,
        _width_scale_type: LayoutScaleType,
        _height_scale_type: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
    }

    pub fn decode_data_bind_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_data_bind_path_ids(&mut self, object: &NestedArtboardBase) {
        if let Some(nested) = object.base.as_nested_artboard() {
            self.data_bind_path_referencer.copy_data_bind_path(
                unsafe { nested.as_ref() }
                    .data_bind_path_referencer
                    .data_bind_path(),
            );
        }
    }

    pub fn internal_data_context(&mut self, value: Option<Rc<DataContext>>) {
        self.data_context = value.clone();
        self.view_model_instance = None;
        let Some(instance) = self.instance.as_deref_mut() else {
            return;
        };
        if self.try_schedule_bind_stateful() {
            return;
        }
        if !self.global_view_model_instances.is_empty() {
            let list = build_vmi_list(None, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, value);
            let nested_context = instance.data_context();
            for animation in &mut self.nested_animations {
                if let Some(state_machine) =
                    unsafe { animation.as_mut() }.as_nested_state_machine_mut()
                {
                    state_machine.set_data_context(nested_context.clone());
                }
            }
            return;
        }

        instance.internal_data_context(value.clone());
        for animation in &mut self.nested_animations {
            if let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            {
                state_machine.set_data_context(value.clone());
            }
        }
    }

    pub fn relink_data_context(&mut self, view_model_instance: Rcp<ViewModelInstance>) {
        self.view_model_instance = Some(view_model_instance.clone());
        if self.base.is_stateful() {
            return;
        }
        let Some(instance) = self.instance.as_deref_mut() else {
            return;
        };
        if let Some(data_context) = instance.data_context() {
            if !data_context
                .main_view_model_instance()
                .is_some_and(|current| current.get() == view_model_instance.get())
            {
                data_context.set_view_model_instance(view_model_instance);
            }
        }
        instance.relink_data_context();
    }

    pub fn clear_data_context(&mut self) {
        let Some(instance) = self.instance.as_deref_mut() else {
            return;
        };
        instance.clear_data_context();
        for animation in &mut self.nested_animations {
            if let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            {
                state_machine.clear_data_context();
            }
        }
    }

    pub fn unbind(&mut self) {
        if let Some(instance) = self.instance.as_deref_mut() {
            instance.unbind();
        }
    }

    pub fn update_data_binds(&mut self) {
        if !self.base.is_paused() {
            if let Some(instance) = self.instance.as_deref_mut() {
                instance.update_data_binds();
            }
        }
    }

    pub fn bind_view_model_instance(
        &mut self,
        view_model_instance: Option<Rcp<ViewModelInstance>>,
        parent: Option<Rc<DataContext>>,
    ) {
        self.data_context = parent.clone();
        self.view_model_instance = view_model_instance.clone();
        let Some(instance) = self.instance.as_deref_mut() else {
            return;
        };

        if let Some(active) = self.active_view_model_instance {
            let primary = Some(unsafe { ref_rcp(active.as_ptr()) });
            let list = build_vmi_list(primary, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, parent);
            let nested_context = instance.data_context();
            for animation in &mut self.nested_animations {
                if let Some(state_machine) =
                    unsafe { animation.as_mut() }.as_nested_state_machine_mut()
                {
                    state_machine.set_data_context(nested_context.clone());
                }
            }
            return;
        }

        if view_model_instance.is_some() || !self.global_view_model_instances.is_empty() {
            let list = build_vmi_list(view_model_instance, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, parent);
            let nested_context = instance.data_context();
            for animation in &mut self.nested_animations {
                if let Some(state_machine) =
                    unsafe { animation.as_mut() }.as_nested_state_machine_mut()
                {
                    state_machine.set_data_context(nested_context.clone());
                }
            }
            return;
        }

        instance.internal_data_context(parent.clone());
        for animation in &mut self.nested_animations {
            if let Some(state_machine) = unsafe { animation.as_mut() }.as_nested_state_machine_mut()
            {
                state_machine.set_data_context(parent.clone());
            }
        }
    }

    pub fn calculate_local_elapsed_seconds(&mut self, elapsed_seconds: f32) -> f32 {
        let mut local_elapsed_seconds = elapsed_seconds
            * if self.base.speed() >= 0.0 {
                self.base.speed()
            } else {
                1.0
            };
        if self.base.quantize() >= 0.0 {
            self.cumulated_seconds += local_elapsed_seconds;
            let quantized_seconds = 1.0 / self.base.quantize();
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

    pub fn advance_component_impl(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if self.artboard_referencer.referenced_artboard().is_none()
            || self.is_collapsed()
            || self.base.is_paused()
        {
            return false;
        }
        if self.has_host_flag(NestedArtboardHostFlags::PENDING_STATEFUL_BINDING) {
            self.bind_stateful();
        }
        let mut keep_going = false;
        let advance_nested =
            flags.0 & AdvanceFlags::ADVANCE_NESTED.0 == AdvanceFlags::ADVANCE_NESTED.0;
        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(elapsed_seconds);
        let new_frame = flags.0 & AdvanceFlags::NEW_FRAME.0 == AdvanceFlags::NEW_FRAME.0;
        if local_elapsed_seconds == 0.0 && self.base.quantize() >= 0.0 && new_frame {
            return true;
        }
        if advance_nested {
            for animation in &mut self.nested_animations {
                let animation = unsafe { animation.as_mut() };
                if !new_frame {
                    if let Some(state_machine) = animation.as_nested_state_machine_mut() {
                        if state_machine.try_change_state()
                            && animation.advance(local_elapsed_seconds, new_frame)
                        {
                            keep_going = true;
                        }
                    }
                } else if animation.advance(local_elapsed_seconds, new_frame) {
                    keep_going = true;
                }
            }
        }

        let advancing_flags = AdvanceFlags(flags.0 & !AdvanceFlags::IS_ROOT.0);
        let mut referenced = self.artboard_referencer.referenced_artboard().unwrap();
        if unsafe { referenced.as_mut() }.advance_internal(local_elapsed_seconds, advancing_flags) {
            keep_going = true;
        }
        if unsafe { referenced.as_ref() }.has_dirt(ComponentDirt::COMPONENTS) {
            self.add_dirt(ComponentDirt::COMPONENTS);
        }
        keep_going
    }

    pub fn reset_impl(&mut self) {
        if let Some(mut referenced) = self.artboard_referencer.referenced_artboard() {
            unsafe { referenced.as_mut() }.reset();
        }
        if let Some(mut active) = self.active_view_model_instance {
            unsafe { active.as_mut() }.advanced();
        }
    }

    pub fn file(&self) -> Option<NonNull<File>> {
        self.file
    }

    pub fn set_file(&mut self, value: Option<NonNull<File>>) {
        self.file = value;
    }

    pub fn referenced_artboard_id(&self) -> i32 {
        self.base.artboard_id() as i32
    }

    pub fn referenced_artboard(&mut self, artboard: Option<NonNull<Artboard>>) {
        let artboard = artboard.expect("NestedArtboard requires a referenced artboard");
        self.artboard_referencer
            .set_referenced_artboard(Some(artboard));
        self.nest(artboard);
        self.try_schedule_bind_stateful();
    }

    pub fn artboard_count(&self) -> usize {
        1
    }

    pub fn type_(&self) -> i32 {
        self.base.core_type() as i32
    }

    pub fn artboard_instance_ptr(&mut self, _index: i32) -> Option<NonNull<ArtboardInstance>> {
        self.instance.as_deref_mut().map(NonNull::from)
    }

    pub fn artboard_instance_default(&mut self) -> Option<NonNull<ArtboardInstance>> {
        self.artboard_instance_ptr(0)
    }

    pub fn advance_component_default(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_component_impl(
            elapsed_seconds,
            AdvanceFlags(AdvanceFlags::ANIMATE.0 | AdvanceFlags::NEW_FRAME.0),
        )
    }

    pub fn source_artboard(&self) -> Option<NonNull<Artboard>> {
        self.artboard_referencer.referenced_artboard()
    }

    pub fn parent_artboard(&self) -> Option<NonNull<Artboard>> {
        self.artboard()
    }

    pub fn mark_host_transform_dirty(&mut self) {
        self.mark_transform_dirty();
    }

    pub fn host_component(&mut self) -> NonNull<Component> {
        NonNull::from(&mut *self).cast()
    }

    pub fn key_input(
        &mut self,
        _key: Key,
        _modifiers: KeyModifiers,
        _is_pressed: bool,
        _is_repeat: bool,
    ) -> bool {
        false
    }

    pub fn text_input(&mut self, _text: &str) -> bool {
        false
    }

    pub fn gamepad_dispatch(
        &mut self,
        _invocation: &ListenerInvocation,
        _scripted_drawable: Option<&mut Option<NonNull<ScriptedDrawable>>>,
    ) -> bool {
        false
    }

    pub fn focused(&mut self) {}

    pub fn blurred(&mut self) {}

    pub fn focusable_artboard(&self) -> Option<NonNull<Artboard>> {
        self.artboard()
    }

    fn set_host_flag(&mut self, flag: NestedArtboardHostFlags) {
        self.host_flags.0 |= flag.0;
    }

    fn clear_host_flag(&mut self, flag: NestedArtboardHostFlags) {
        self.host_flags.0 &= !flag.0;
    }

    fn has_host_flag(&self, flag: NestedArtboardHostFlags) -> bool {
        self.host_flags.0 & flag.0 == flag.0
    }

    fn artboard(&self) -> Option<NonNull<Artboard>> {
        self.base.base.base.artboard()
    }

    fn parent(&self) -> Option<NonNull<Component>> {
        self.base.base.base.parent()
    }

    fn children(&self) -> Vec<NonNull<Core>> {
        self.base.base.base.children().to_vec()
    }

    fn render_opacity(&self) -> f32 {
        self.base.base.render_opacity()
    }

    fn world_transform(&self) -> Mat2D {
        self.base.base.world_transform()
    }

    fn needs_save_operation(&self) -> bool {
        self.base.base.needs_save_operation()
    }

    fn add_dirt(&mut self, dirt: ComponentDirt) {
        self.base.base.base.add_dirt(dirt);
    }

    fn is_collapsed(&self) -> bool {
        self.base.base.base.is_collapsed()
    }

    fn mark_transform_dirty(&mut self) {
        self.base.base.mark_transform_dirty();
    }
}

impl AdvancingComponent for NestedArtboard {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        self.advance_component_impl(elapsed_seconds, flags)
    }
}

impl crate::mechanical_port::source::resetting_component::ResettingComponent for NestedArtboard {
    fn reset(&mut self) {
        self.reset_impl();
    }
}

impl ArtboardReferencerBehavior for NestedArtboard {
    fn artboard_referencer(&self) -> &ArtboardReferencer {
        &self.artboard_referencer
    }

    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer {
        &mut self.artboard_referencer
    }

    fn update_artboard(
        &mut self,
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
    ) {
        NestedArtboard::update_artboard(self, view_model_instance_artboard);
    }

    fn referenced_artboard_id(&self) -> i32 {
        NestedArtboard::referenced_artboard_id(self)
    }
}

impl CoreArtboardReferencer for NestedArtboard {
    fn core(&mut self) -> &mut Core {
        &mut self.base.base.base
    }

    fn core_type(&self) -> u16 {
        self.base.core_type()
    }
}

impl Focusable for NestedArtboard {
    fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        NestedArtboard::key_input(self, key, modifiers, is_pressed, is_repeat)
    }

    fn text_input(&mut self, text: &str) -> bool {
        NestedArtboard::text_input(self, text)
    }

    fn gamepad_dispatch(&mut self, _invocation: &dyn core::any::Any) -> bool {
        false
    }

    fn focused(&mut self) {
        NestedArtboard::focused(self);
    }

    fn blurred(&mut self) {
        NestedArtboard::blurred(self);
    }
}

impl NestedArtboardBaseCallbacks for NestedArtboard {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.base.notify_property_changed(property_key);
    }

    fn decode_data_bind_path_ids(&mut self, value: &[u8]) {
        NestedArtboard::decode_data_bind_path_ids(self, value);
    }

    fn copy_data_bind_path_ids(&mut self, object: &NestedArtboardBase) {
        NestedArtboard::copy_data_bind_path_ids(self, object);
    }
}

fn make_translate(artboard: &Artboard) -> Mat2D {
    Mat2D::from_translate(Vec2D::new(
        -artboard.base.origin_x() * artboard.base.width(),
        -artboard.base.origin_y() * artboard.base.height(),
    ))
}
