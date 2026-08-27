use std::{
    collections::{HashMap, HashSet},
    ptr::NonNull,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::{
        linear_animation::LinearAnimation, linear_animation_instance::LinearAnimationInstance,
        state_machine::StateMachine, state_machine_instance::StateMachineInstance,
    },
    artboard_component_list::ArtboardComponentList,
    artboard_host::ArtboardHost,
    audio::audio_engine::AudioEngine,
    component::Component,
    component_dirt::ComponentDirt,
    core::{Core, binary_reader::BinaryReader},
    core_context::CoreContext,
    data_bind::{
        data_bind::DataBind,
        data_bind_container::DataBindContainer,
        data_context::{DataContext, ViewModelInstance as DataContextViewModelInstance},
    },
    draw_rules::DrawRules,
    draw_target::DrawTarget,
    draw_target_placement::DrawTargetPlacement,
    drawable::Drawable,
    factory::Factory,
    file::File,
    generated::artboard_base::{ArtboardBase, ArtboardBaseCallbacks},
    hit_info::HitInfo,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    input::{focus_manager::FocusManager, focus_node::FocusNodeRef},
    joystick::Joystick,
    layout::{layout_component::LayoutComponent, layout_data::LayoutData},
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    renderer::{RenderPath, Renderer},
    resetting_component::ResettingComponent,
    scripted::scripted_object::ScriptedObject,
    semantic::{
        semantic_manager::SemanticManager,
        semantic_node::{SemanticNode, SemanticNodeRef},
    },
    shapes::{clipping_shape::ClippingShape, paint::shape_paint::ShapePaint, shape::Shape},
    status_code::StatusCode,
    text::text_value_run::TextValueRun,
    viewmodel::viewmodel_instance::ViewModelInstance,
};

#[cfg(feature = "rive_tools")]
pub type ArtboardCallback = fn(*mut ());
#[cfg(feature = "rive_tools")]
pub type TestBoundsCallback = fn(*mut (), f32, f32, bool) -> u8;
#[cfg(feature = "rive_tools")]
pub type IsAncestorCallback = fn(*mut (), u16) -> u8;
#[cfg(feature = "rive_tools")]
pub type RootTransformCallback = fn(*mut (), f32, f32, bool) -> f32;

static FRAME_ID: AtomicU64 = AtomicU64::new(0);

pub struct Artboard {
    pub base: ArtboardBase,
    objects: Vec<Option<NonNull<Core>>>,
    invalid_objects: Vec<Option<NonNull<Core>>>,
    animations: Vec<*mut LinearAnimation>,
    state_machines: Vec<*mut StateMachine>,
    dependency_order: Vec<*mut Component>,
    drawables: Vec<*mut Drawable>,
    clipping_shapes: Vec<*mut ClippingShape>,
    draw_targets: Vec<*mut DrawTarget>,
    nested_artboards: Vec<*mut NestedArtboard>,
    component_lists: Vec<*mut ArtboardComponentList>,
    artboard_hosts: Vec<*mut dyn ArtboardHost>,
    joysticks: Vec<*mut Joystick>,
    resettables: Vec<*mut dyn ResettingComponent>,
    scripted_objects: Vec<*mut ScriptedObject>,
    advancing_components: Vec<*mut dyn AdvancingComponent>,
    data_bind_container: DataBindContainer,
    data_context: Option<Rc<DataContext>>,
    #[cfg(feature = "rive_scripting")]
    scripting_vm: Option<*mut ()>,
    joysticks_apply_before_update: bool,
    dirt_depth: u32,
    dirt: ComponentDirt,
    factory: Option<NonNull<Factory>>,
    first_drawable: Option<NonNull<Drawable>>,
    is_instance: bool,
    frame_origin: bool,
    dirty_layout: HashSet<*mut LayoutComponent>,
    is_cleaning_dirty_layouts: bool,
    owned_inherited_interpolator: Option<
        Box<crate::mechanical_port::source::animation::keyframe_interpolator::KeyFrameInterpolator>,
    >,
    original_width: f32,
    original_height: f32,
    updates_own_layout: bool,
    host_transform_marked_dirty: bool,
    did_change: bool,
    host: Option<*mut dyn ArtboardHost>,
    active_focus_manager: Option<NonNull<FocusManager>>,
    active_semantic_manager: Option<NonNull<SemanticManager>>,
    semantic_boundary_node: Option<SemanticNodeRef>,
    #[cfg(feature = "rive_tools")]
    external_parent_focus_node: Option<FocusNodeRef>,
    draw_order_change_counter: u8,
    #[cfg(feature = "rive_tools")]
    artboard_id: u16,
    artboard_source: Option<NonNull<Artboard>>,
    #[cfg(feature = "external-rive_audio-engine")]
    audio_engine: Option<Rc<AudioEngine>>,
    volume: f32,
    host_opacity: f32,
    #[cfg(feature = "rive_tools")]
    layout_changed_callback: Option<ArtboardCallback>,
    #[cfg(feature = "rive_tools")]
    layout_dirty_callback: Option<ArtboardCallback>,
    #[cfg(feature = "rive_tools")]
    transform_dirty_callback: Option<ArtboardCallback>,
    #[cfg(feature = "rive_tools")]
    test_bounds_callback: Option<TestBoundsCallback>,
    #[cfg(feature = "rive_tools")]
    is_ancestor_callback: Option<IsAncestorCallback>,
    #[cfg(feature = "rive_tools")]
    root_transform_callback: Option<RootTransformCallback>,
    #[cfg(feature = "rive_tools")]
    pub callback_user_data: *mut (),
}

impl Default for Artboard {
    fn default() -> Self {
        let mut base = ArtboardBase::default();
        base.base.set_clip(true);
        Self {
            base,
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
            #[cfg(feature = "rive_scripting")]
            scripting_vm: None,
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
            #[cfg(feature = "rive_tools")]
            external_parent_focus_node: None,
            draw_order_change_counter: 0,
            #[cfg(feature = "rive_tools")]
            artboard_id: 0,
            artboard_source: None,
            #[cfg(feature = "external-rive_audio-engine")]
            audio_engine: None,
            volume: 1.0,
            host_opacity: 1.0,
            #[cfg(feature = "rive_tools")]
            layout_changed_callback: None,
            #[cfg(feature = "rive_tools")]
            layout_dirty_callback: None,
            #[cfg(feature = "rive_tools")]
            transform_dirty_callback: None,
            #[cfg(feature = "rive_tools")]
            test_bounds_callback: None,
            #[cfg(feature = "rive_tools")]
            is_ancestor_callback: None,
            #[cfg(feature = "rive_tools")]
            root_transform_callback: None,
            #[cfg(feature = "rive_tools")]
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
        #[cfg(feature = "rive_tools")]
        {
            artboard.callback_user_data = (&mut *artboard as *mut Artboard).cast();
        }
        artboard
    }

    #[cfg(feature = "testing")]
    pub fn with_factory(factory: &mut Factory) -> Self {
        let mut artboard = Self::default();
        artboard.factory = Some(NonNull::from(factory));
        artboard.base.base.set_clip(true);
        artboard
    }

    pub fn frame_id() -> u64 {
        FRAME_ID.load(Ordering::Relaxed)
    }

    #[cfg(any(test, feature = "rive_tools"))]
    pub fn inc_frame_id() {
        FRAME_ID.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_active_focus_manager(&mut self, manager: Option<NonNull<FocusManager>>) {
        self.active_focus_manager = manager;
    }

    pub fn focus_manager(&self) -> Option<&FocusManager> {
        self.active_focus_manager
            .map(|manager| unsafe { manager.as_ref() })
    }

    pub fn set_active_semantic_manager(&mut self, manager: Option<NonNull<SemanticManager>>) {
        self.active_semantic_manager = manager;
    }

    pub fn semantic_manager(&self) -> Option<&SemanticManager> {
        self.active_semantic_manager
            .map(|manager| unsafe { manager.as_ref() })
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

    pub fn objects(&self) -> &[Option<NonNull<Core>>] {
        &self.objects
    }

    pub fn nested_artboards(&self) -> Vec<*mut NestedArtboard> {
        self.nested_artboards.clone()
    }

    pub fn artboard_component_lists(&self) -> Vec<*mut ArtboardComponentList> {
        self.component_lists.clone()
    }

    pub fn data_context(&self) -> Option<Rc<DataContext>> {
        self.data_context.clone()
    }

    #[cfg(feature = "rive_scripting")]
    pub fn set_scripting_vm(&mut self, value: Option<*mut ()>) {
        self.scripting_vm = value;
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

    pub fn first_animation(&self) -> Option<&LinearAnimation> {
        self.animation_at(0)
    }

    pub fn first_state_machine(&self) -> Option<&StateMachine> {
        self.state_machine_at(0)
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

    pub fn first_drawable(&self) -> Option<&Drawable> {
        self.first_drawable
            .map(|drawable| unsafe { drawable.as_ref() })
    }

    pub fn owned_inherited_interpolator(
        &mut self,
    ) -> &mut Option<
        Box<crate::mechanical_port::source::animation::keyframe_interpolator::KeyFrameInterpolator>,
    > {
        &mut self.owned_inherited_interpolator
    }

    pub fn factory(&self) -> Option<&Factory> {
        self.factory.map(|factory| unsafe { factory.as_ref() })
    }

    pub fn artboard_source(&self) -> &Artboard {
        if self.is_instance {
            self.artboard_source
                .map(|source| unsafe { source.as_ref() })
                .unwrap_or(self)
        } else {
            self
        }
    }

    pub fn set_artboard_source(&mut self, artboard: Option<NonNull<Artboard>>) {
        self.artboard_source = artboard;
    }

    #[cfg(feature = "rive_tools")]
    pub fn set_artboard_id(&mut self, id: u16) {
        self.artboard_id = id;
    }

    #[cfg(feature = "rive_tools")]
    pub fn artboard_id(&self) -> u16 {
        self.artboard_id
    }

    pub fn added_to_host(&mut self) {
        self.base.base.set_just_added_to_host(true);
    }

    pub(crate) fn add_object(&mut self, object: Option<NonNull<Core>>) {
        self.objects.push(object);
    }

    pub(crate) fn add_animation(&mut self, object: *mut LinearAnimation) {
        self.animations.push(object);
    }

    pub(crate) fn add_state_machine(&mut self, object: *mut StateMachine) {
        self.state_machines.push(object);
    }

    pub fn add_scripted_object(&mut self, object: *mut ScriptedObject) {
        self.scripted_objects.push(object);
    }

    pub fn validate_objects(&mut self) -> bool {
        let size = self.objects.len();
        let mut valid = vec![false; size];
        for _cycle in 0..100 {
            let mut changed = false;
            for (i, validity) in valid.iter_mut().enumerate().take(size).skip(1) {
                let Some(mut object) = self.objects[i] else {
                    continue;
                };
                let was_valid = *validity;
                let is_valid = unsafe { object.as_mut() }.validate(self);
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
                    self.invalid_objects.push(self.objects[i]);
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
        #[cfg(feature = "rive_layout")]
        self.mark_layout_dirty(self as *mut Artboard as *mut LayoutComponent);

        for object in self.objects.clone().into_iter().flatten() {
            let code = unsafe { &mut *object.as_ptr() }.on_added_dirty(self);
            if !can_continue(code) {
                return code;
            }
        }

        if !self.is_instance {
            for animation in self.animations.clone() {
                let code = unsafe { &mut *animation }.on_added_dirty(self);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in self.state_machines.clone() {
                let code = unsafe { &mut *state_machine }.on_added_dirty(self);
                if !can_continue(code) {
                    return code;
                }
            }
            if self.animations.is_empty() && self.state_machines.is_empty() {
                let mut state_machine = Box::new(StateMachine::default());
                state_machine.set_name("Auto Generated State Machine".into());
                self.state_machines.push(Box::into_raw(state_machine));
            }
        }

        let mut component_draw_rules = HashMap::<*mut Core, *mut DrawRules>::new();
        for object in self.objects.clone().into_iter().flatten() {
            let object = unsafe { &mut *object.as_ptr() };
            let code = object.on_added_clean(self);
            if !can_continue(code) {
                return code;
            }
            if let Some(component) = object.as_component_mut() {
                if let Some(resettable) =
                    crate::mechanical_port::source::resetting_component::from(component)
                {
                    self.resettables.push(resettable);
                }
            }
            if let Some(rules) = object.as_draw_rules_mut() {
                if let Some(component) = self.resolve_ptr(rules.base.parent_id()) {
                    component_draw_rules.insert(component, rules);
                } else {
                    eprintln!(
                        "Artboard::initialize - Draw rule targets missing component width id {}",
                        rules.base.parent_id()
                    );
                }
            } else if let Some(nested) = object.as_nested_artboard_mut() {
                self.nested_artboards.push(nested);
                self.artboard_hosts.push(nested as *mut dyn ArtboardHost);
            } else if let Some(list) = object.as_artboard_component_list_mut() {
                self.component_lists.push(list);
                self.artboard_hosts.push(list as *mut dyn ArtboardHost);
            } else if let Some(joystick) = object.as_joystick_mut() {
                if !joystick.can_apply_before_update() {
                    self.joysticks_apply_before_update = false;
                }
                joystick.add_dependents(self);
                self.joysticks.push(joystick);
            }
            if let Some(advancing) = <dyn AdvancingComponent>::from(object) {
                self.advancing_components.push(advancing);
            }
        }

        if !self.is_instance {
            for animation in self.animations.clone() {
                let code = unsafe { &mut *animation }.on_added_clean(self);
                if !can_continue(code) {
                    return code;
                }
            }
            for state_machine in self.state_machines.clone() {
                let code = unsafe { &mut *state_machine }.on_added_clean(self);
                if !can_continue(code) {
                    return code;
                }
            }
        }

        for object in self.objects.clone().into_iter().flatten() {
            let object = unsafe { &mut *object.as_ptr() };
            if let Some(component) = object.as_component_mut() {
                component.build_dependencies();
            }
            if let Some(drawable) = object.as_drawable_mut()
                && !std::ptr::eq(
                    drawable as *mut Drawable,
                    self as *mut Artboard as *mut Drawable,
                )
            {
                self.drawables.push(drawable);
                if drawable.is_foreground_layout_drawable() {
                    let parent = drawable.base.base.base.base.base.parent_ptr();
                    let mut index = self.drawables.len() - 1;
                    while index >= 1 {
                        let swapping = self.drawables[index - 1];
                        self.drawables.swap(index - 1, index);
                        if swapping.cast::<Component>() == parent {
                            break;
                        }
                        index -= 1;
                    }
                }
                let mut parent = Some(drawable as *mut Drawable as *mut Core);
                while let Some(current) = parent {
                    if let Some(rules) = component_draw_rules.get(&current) {
                        drawable.flattened_draw_rules = Some(*rules);
                        break;
                    }
                    parent = unsafe { &mut *current }.parent_core_ptr();
                }
            } else if let Some(clipping_shape) = object.as_clipping_shape_mut() {
                self.clipping_shapes.push(clipping_shape);
            }
        }

        let mut layouts = Vec::<*mut LayoutComponent>::new();
        let mut i = 0;
        while i < self.drawables.len() {
            let drawable = self.drawables[i];
            let mut current_layout = layouts.last().copied();
            let in_current_layout = current_layout
                .is_none_or(|layout| unsafe { &mut *drawable }.is_child_of_layout(layout));
            if current_layout.is_some() && !in_current_layout {
                loop {
                    let layout = current_layout.unwrap();
                    self.drawables.insert(i, unsafe { &mut *layout }.proxy());
                    i += 1;
                    layouts.pop();
                    current_layout = layouts.last().copied();
                    if current_layout.is_none()
                        || unsafe { &mut *drawable }.is_child_of_layout(current_layout.unwrap())
                    {
                        break;
                    }
                }
            }
            if let Some(layout) = unsafe { &mut *drawable }.as_layout_component_mut() {
                layouts.push(layout);
            }
            i += 1;
        }
        while let Some(layout) = layouts.pop() {
            self.drawables.push(unsafe { &mut *layout }.proxy());
        }

        self.sort_dependencies();
        let rules_list: Vec<*mut DrawRules> = self
            .objects
            .iter()
            .flatten()
            .filter_map(|object| component_draw_rules.get(&object.as_ptr()).copied())
            .collect();
        let mut root = DrawTarget::default();
        for rules in rules_list {
            for child in unsafe { &mut *rules }.base.base.base.base.children() {
                let target = child.as_draw_target_mut().unwrap();
                root.add_dependent(target);
                if let Some(drawable) = target.drawable_mut()
                    && let Some(dependent_rules) = drawable.flattened_draw_rules
                {
                    for object in self.objects.iter_mut().flatten() {
                        if let Some(dependent_target) =
                            unsafe { object.as_mut() }.as_draw_target_mut()
                            && dependent_target.parent_ptr() == Some(dependent_rules)
                        {
                            dependent_target.add_dependent(target);
                        }
                    }
                }
            }
        }
        let mut draw_target_order = Vec::<*mut Component>::new();
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default()
            .sort(&mut root, &mut draw_target_order);
        self.draw_targets.extend(
            draw_target_order
                .into_iter()
                .skip(1)
                .map(|target| target.cast::<DrawTarget>()),
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
        for target in self.draw_targets.iter().copied() {
            unsafe {
                (*target).first = None;
                (*target).last = None;
            }
        }

        self.first_drawable = None;
        let mut last_drawable = None::<NonNull<Drawable>>;
        for drawable in self.drawables.iter().copied() {
            let drawable_ref = unsafe { &mut *drawable };
            let active_target = drawable_ref
                .flattened_draw_rules
                .and_then(|rules| unsafe { (&mut *rules).active_target_mut_ptr() });
            if let Some(target) = active_target {
                let target = unsafe { &mut *target };
                if target.first.is_none() {
                    target.first = NonNull::new(drawable);
                    target.last = NonNull::new(drawable);
                    drawable_ref.prev = None;
                    drawable_ref.next = None;
                } else {
                    let last = target.last.unwrap();
                    unsafe { (*last.as_ptr()).next = NonNull::new(drawable) };
                    drawable_ref.prev = Some(last.as_ptr());
                    target.last = NonNull::new(drawable);
                    drawable_ref.next = None;
                }
            } else {
                drawable_ref.prev = last_drawable.map(NonNull::as_ptr);
                drawable_ref.next = None;
                if let Some(last) = last_drawable {
                    unsafe { (*last.as_ptr()).next = NonNull::new(drawable) };
                    last_drawable = NonNull::new(drawable);
                } else {
                    let pointer = NonNull::new(drawable);
                    last_drawable = pointer;
                    self.first_drawable = pointer;
                }
            }
        }

        for rule in self.draw_targets.iter().copied() {
            let rule = unsafe { &mut *rule };
            let Some(first) = rule.first else {
                continue;
            };
            let last = rule.last.unwrap();
            let target_drawable = rule.drawable_mut().unwrap();
            match rule.placement() {
                DrawTargetPlacement::Before => {
                    if let Some(previous) = target_drawable.prev {
                        unsafe { (*previous).next = Some(first) };
                        unsafe { (*first.as_ptr()).prev = Some(previous) };
                    }
                    if self.first_drawable.is_some_and(|value| {
                        std::ptr::eq(value.as_ptr(), target_drawable as *mut Drawable)
                    }) {
                        self.first_drawable = Some(first);
                    }
                    target_drawable.prev = Some(last.as_ptr());
                    unsafe { (*last.as_ptr()).next = NonNull::from_mut(target_drawable).into() };
                }
                DrawTargetPlacement::After => {
                    if let Some(next) = target_drawable.next {
                        unsafe {
                            (*next.as_ptr()).prev = Some(last.as_ptr());
                            (*last.as_ptr()).next = Some(next);
                        }
                    }
                    if last_drawable.is_some_and(|value| {
                        std::ptr::eq(value.as_ptr(), target_drawable as *mut Drawable)
                    }) {
                        last_drawable = Some(last);
                    }
                    target_drawable.next = Some(first);
                    unsafe { (*first.as_ptr()).prev = Some(target_drawable) };
                }
            }
        }

        self.first_drawable = last_drawable;
        for clipping_shape in self.clipping_shapes.iter().copied() {
            unsafe { &mut *clipping_shape }.reset_drawables();
        }

        let mut current_drawable = self.first_drawable;
        let mut next_drawable = None::<NonNull<Drawable>>;
        let mut clipping_stack = Vec::<*mut ClippingShape>::new();
        while let Some(mut current) = current_drawable {
            let current_ref = unsafe { current.as_mut() };
            current_ref.set_needs_save_operation(true);
            let drawable_clipping_shapes = current_ref.clipping_shapes().to_vec();
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
                    let clipping_shape = unsafe { &mut *clipping_stack[i] };
                    let proxy = clipping_shape.create_proxy_drawable(Box::new(
                        crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeEnd::new(
                            clipping_shape,
                        ),
                    ));
                    let proxy_drawable = proxy.drawable_mut();
                    if let Some(mut next) = next_drawable {
                        proxy_drawable.next = Some(next);
                        unsafe { next.as_mut() }.prev = Some(proxy_drawable);
                    } else {
                        eprintln!("Error - adding clip end as first operation");
                    }
                    proxy_drawable.prev = Some(current.as_ptr());
                    current_ref.next = NonNull::new(proxy_drawable);
                    next_drawable = NonNull::new(proxy_drawable);
                    if i == removing_index || i == 0 {
                        break;
                    }
                    i -= 1;
                }
                clipping_stack.truncate(removing_index);
            }
            for clipping_shape in drawable_clipping_shapes {
                if !clipping_stack.contains(&clipping_shape) {
                    let clipping = unsafe { &mut *clipping_shape };
                    let proxy = clipping.create_proxy_drawable(Box::new(
                        crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeStart::new(
                            clipping,
                        ),
                    ));
                    let proxy_drawable = proxy.drawable_mut();
                    if let Some(mut next) = next_drawable {
                        proxy_drawable.next = Some(next);
                        unsafe { next.as_mut() }.prev = Some(proxy_drawable);
                    } else {
                        self.first_drawable = NonNull::new(proxy_drawable);
                    }
                    proxy_drawable.prev = Some(current.as_ptr());
                    current_ref.next = NonNull::new(proxy_drawable);
                    next_drawable = NonNull::new(proxy_drawable);
                    clipping_stack.push(clipping_shape);
                }
            }
            next_drawable = Some(current);
            current_drawable = current_ref.prev.and_then(NonNull::new);
        }
        if !clipping_stack.is_empty() {
            for i in (0..clipping_stack.len()).rev() {
                let clipping = unsafe { &mut *clipping_stack[i] };
                let proxy = clipping.create_proxy_drawable(Box::new(
                    crate::mechanical_port::source::shapes::clipping_shape::ClippingShapeEnd::new(
                        clipping,
                    ),
                ));
                let proxy_drawable = proxy.drawable_mut();
                if let Some(mut next) = next_drawable {
                    unsafe { next.as_mut() }.prev = Some(proxy_drawable);
                    proxy_drawable.next = Some(next);
                }
                proxy_drawable.prev = None;
                next_drawable = NonNull::new(proxy_drawable);
            }
        }
        self.clear_redundant_operations();
    }

    fn clear_redundant_operations(&mut self) {
        let mut current_drawable = self.first_drawable;
        let mut previous_applied_save = false;
        let mut applied_clipping_save_operations = Vec::<bool>::new();
        while let Some(mut current) = current_drawable {
            let drawable = unsafe { current.as_mut() };
            drawable.set_needs_save_operation(true);
            if previous_applied_save {
                if drawable.is_clip_start() {
                    applied_clipping_save_operations.push(false);
                    drawable.set_needs_save_operation(false);
                } else if drawable.is_clip_end() {
                    let applied = applied_clipping_save_operations.pop().unwrap();
                    drawable.set_needs_save_operation(applied);
                } else {
                    let next = drawable.prev.unwrap();
                    if unsafe { &*next }.is_clip_end() {
                        drawable.set_needs_save_operation(false);
                    }
                }
            } else if drawable.is_clip_start() {
                applied_clipping_save_operations.push(true);
            } else if drawable.is_clip_end() {
                let applied = applied_clipping_save_operations.pop().unwrap();
                drawable.set_needs_save_operation(applied);
            }
            previous_applied_save =
                drawable.is_clip_start() && (drawable.will_clip() || previous_applied_save);
            current_drawable = drawable.prev.and_then(NonNull::new);
        }
        assert!(applied_clipping_save_operations.is_empty());
    }

    fn sort_dependencies(&mut self) {
        self.dependency_order.clear();
        crate::mechanical_port::source::dependency_sorter::DependencySorter::default()
            .sort(self, &mut self.dependency_order);
        for (graph_order, component) in self.dependency_order.iter().copied().enumerate() {
            unsafe { &mut *component }.set_graph_order(graph_order as u32);
        }
        self.dirt |= ComponentDirt::COMPONENTS;
    }

    fn init_scripted_objects(&mut self) {
        if self.is_instance {
            for object in self.scripted_objects.iter().copied() {
                let object = unsafe { &mut *object };
                if let Some(script_asset) = object.script_asset_mut() {
                    if !object.user_lua_init_done() {
                        script_asset.init_scripted_object(object);
                    }
                    object.hydrate_script_inputs();
                }
            }
        }
    }

    pub fn poll_async_work(&mut self) {
        crate::mechanical_port::source::r#async::work_pool::rive_poll_async_work();
    }

    pub fn draw_canvases(&mut self) {
        #[cfg(feature = "rive_scripting")]
        if let Some(vm) = self.scripting_vm {
            let state = unsafe { (&mut *vm).state() };
            if !state.is_null() {
                let context = unsafe { crate::mechanical_port::source::lua::thread_data(state) };
                let _phase =
                    crate::mechanical_port::source::lua::ScopedCanvasDrawingPhase::new(context);
                self.internal_draw_canvases();
                return;
            }
        }
        self.internal_draw_canvases();
    }

    pub fn advance_scripted_view_models(&mut self) {
        #[cfg(feature = "rive_scripting")]
        if let Some(vm) = self.scripting_vm
            && let Some(context) = unsafe { (&mut *vm).context_mut() }
        {
            context.advance_detached_view_models();
        }
    }

    pub fn internal_draw_canvases(&mut self) {
        for object in self.scripted_objects.iter().copied() {
            unsafe { &mut *object }.script_draw_canvas();
        }
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            for i in 0..host.artboard_count() as i32 {
                if let Some(nested) = host.artboard_instance(i) {
                    nested.internal_draw_canvases();
                }
            }
        }
    }

    #[cfg(feature = "rive_scripting")]
    pub fn find_draw_canvas_luau_state(&self) -> Option<*mut ()> {
        for object in self.scripted_objects.iter().copied() {
            let object = unsafe { &*object };
            if object.draws_canvas() {
                return Some(object.state());
            }
        }
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            for i in 0..host.artboard_count() as i32 {
                if let Some(nested) = host.artboard_instance(i)
                    && let Some(state) = nested.find_draw_canvas_luau_state()
                {
                    return Some(state);
                }
            }
        }
        None
    }

    pub fn resolve_ptr(&self, id: u32) -> Option<*mut Core> {
        self.objects.get(id as usize)?.map(NonNull::as_ptr)
    }

    pub fn id_of(&self, object: *mut Core) -> u32 {
        self.objects
            .iter()
            .position(|candidate| candidate.is_some_and(|candidate| candidate.as_ptr() == object))
            .map_or(0, |index| index as u32)
    }

    pub fn on_component_dirty(&mut self, component: &Component) {
        self.did_change = true;
        self.dirt |= ComponentDirt::COMPONENTS;
        if component.graph_order() < self.dirt_depth {
            self.dirt_depth = component.graph_order();
        }
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {
        self.dirt |= ComponentDirt::COMPONENTS;
    }

    #[cfg(feature = "rive_layout")]
    pub fn propagate_size(&mut self) {
        self.add_dirt(ComponentDirt::PATH, false);
        if self.shares_layout_with_host() {
            unsafe { &mut *self.host.unwrap() }.mark_host_transform_dirty();
        }
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.layout_changed_callback {
            callback(self.callback_user_data);
        }
    }

    fn shares_layout_with_host(&self) -> bool {
        self.host
            .is_some_and(|host| unsafe { (&*host).is_layout_provider() })
    }

    pub fn set_host(&mut self, host: Option<*mut dyn ArtboardHost>) {
        self.added_to_host();
        self.host = host;
        #[cfg(feature = "rive_layout")]
        if self.shares_layout_with_host()
            && let Some(parent) = self.parent_artboard()
        {
            let this = self as *mut Artboard as *mut LayoutComponent;
            parent.mark_layout_dirty(this);
            parent.sync_layout_children();
        }
    }

    pub fn host(&self) -> Option<*mut dyn ArtboardHost> {
        self.host
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

    fn parent_artboard(&self) -> Option<&mut Artboard> {
        self.host
            .map(|host| unsafe { (&mut *host).parent_artboard() })
    }

    pub fn layout_width(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.base.base.layout().width();
        }
        #[cfg(not(feature = "rive_layout"))]
        self.width()
    }

    pub fn layout_height(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.base.base.layout().height();
        }
        #[cfg(not(feature = "rive_layout"))]
        self.height()
    }

    pub fn layout_x(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.base.base.layout().left();
        }
        #[cfg(not(feature = "rive_layout"))]
        0.0
    }

    pub fn layout_y(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.base.base.layout().top();
        }
        #[cfg(not(feature = "rive_layout"))]
        0.0
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
        #[cfg(feature = "rive_layout")]
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

    pub fn add_dirty_data_bind(&mut self, data_bind: *mut DataBind) {
        let component = unsafe { &mut *data_bind }
            .target_mut()
            .as_component_mut()
            .unwrap();
        self.on_component_dirty(component);
        self.data_bind_container.add_dirty_data_bind(data_bind);
    }

    pub fn update_data_binds(&mut self, apply_target_to_source: bool) {
        for host in self.artboard_hosts.iter().copied() {
            unsafe { &mut *host }.update_data_binds();
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
                let component = unsafe { &mut *self.dependency_order[i] };
                self.dirt_depth = i as u32;
                let dirt = component.dirt();
                if dirt == ComponentDirt::NONE || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }
                component.set_dirt(ComponentDirt::NONE);
                component.update(dirt);
                if self.dirt_depth < i as u32 {
                    break;
                }
            }
            step += 1;
        }
        true
    }

    pub fn take_layout_data(&mut self) -> Option<*mut LayoutData> {
        #[cfg(feature = "rive_layout")]
        {
            self.updates_own_layout = false;
            return self.base.base.take_layout_data();
        }
        #[cfg(not(feature = "rive_layout"))]
        None
    }

    pub fn clean_layout(&mut self, layout_component: *mut LayoutComponent) {
        assert!(!self.is_cleaning_dirty_layouts);
        if self.is_cleaning_dirty_layouts {
            eprintln!("Artboard::cleanLayout - trying to remove a dirty layout during clean pass!");
            return;
        }
        self.dirty_layout.remove(&layout_component);
        if std::ptr::eq(
            layout_component,
            self as *mut Artboard as *mut LayoutComponent,
        ) && let Some(parent) = self.parent_artboard()
        {
            parent.clean_layout(layout_component);
        }
    }

    pub fn mark_layout_dirty(&mut self, layout_component: *mut LayoutComponent) {
        assert!(!self.is_cleaning_dirty_layouts);
        if self.is_cleaning_dirty_layouts {
            eprintln!(
                "Artboard::markLayoutDirty - trying to mark a layout dirty during clean pass!"
            );
            return;
        }
        #[cfg(feature = "rive_tools")]
        if self.dirty_layout.is_empty()
            && let Some(callback) = self.layout_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.dirty_layout.insert(layout_component);
        if self.is_instance {
            if self.shares_layout_with_host() {
                if let Some(host) = self.host {
                    unsafe { &mut *host }
                        .mark_hosting_layout_dirty(self as *mut Artboard as *mut ArtboardInstance);
                }
            } else {
                self.mark_host_transform_dirty();
            }
        }
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    pub fn mark_host_transform_dirty(&mut self) {
        #[cfg(feature = "rive_tools")]
        if !self.host_transform_marked_dirty
            && let Some(callback) = self.transform_dirty_callback
        {
            callback(self.callback_user_data);
        }
        self.host_transform_marked_dirty = true;
        if let Some(host) = self.host {
            unsafe { &mut *host }.mark_host_transform_dirty();
        }
    }

    pub fn sync_style_changes_with_update(&mut self, force_update: bool) {
        #[cfg(feature = "rive_layout")]
        if self.sync_style_changes() && (self.updates_own_layout || force_update) {
            self.calculate_layout();
            self.base.base.update_layout_bounds(true);
        }
    }

    pub fn sync_style_changes(&mut self) -> bool {
        let mut updated = false;
        self.is_cleaning_dirty_layouts = true;
        #[cfg(feature = "rive_layout")]
        if !self.dirty_layout.is_empty() {
            for layout in self.dirty_layout.iter().copied() {
                if layout.is_null() {
                    continue;
                }
                let layout = unsafe { &mut *layout };
                if let Some(artboard) = layout.as_artboard_mut() {
                    if std::ptr::eq(artboard, self) {
                        artboard.base.base.sync_style();
                    } else if !artboard.updates_own_layout() {
                        artboard.sync_style_changes();
                    }
                } else {
                    layout.sync_style();
                }
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
            for joystick in self.joysticks.iter().copied() {
                unsafe { &mut *joystick }.apply(self);
            }
        }
        if self.update_components() {
            did_update = true;
        }
        if !self.joysticks_apply_before_update {
            for joystick in self.joysticks.iter().copied() {
                if !unsafe { &*joystick }.can_apply_before_update() {
                    self.update_data_binds(true);
                    if self.update_components() {
                        did_update = true;
                    }
                }
                unsafe { &mut *joystick }.apply(self);
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
        for advancing in self.advancing_components.iter().copied() {
            if unsafe { &mut *advancing }.advance_component(elapsed_seconds, flags) {
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
        for resettable in self.resettables.iter().copied() {
            unsafe { &mut *resettable }.reset();
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

    pub fn hit_test(&mut self, info: &mut HitInfo, transform: &Mat2D) -> Option<&mut Core> {
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
        let mut last = self.first_drawable;
        while let Some(drawable) = last {
            if let Some(previous) = unsafe { drawable.as_ref() }.prev {
                last = NonNull::new(previous);
            } else {
                break;
            }
        }
        let mut drawable = last;
        while let Some(mut pointer) = drawable {
            let drawable_ref = unsafe { pointer.as_mut() };
            drawable = drawable_ref.next.and_then(NonNull::new);
            if drawable_ref.is_hidden() {
                continue;
            }
            if let Some(core) = drawable_ref.hit_test(info, &matrix) {
                return Some(core);
            }
        }
        None
    }

    pub fn root_transform(&mut self, point: Vec2D) -> Vec2D {
        if let Some(host) = self.host {
            let local = if self.has_self_transform() {
                self.self_transform() * point
            } else {
                point
            };
            return unsafe { &*host }
                .host_transform_point(&local, self as *mut Artboard as *mut ArtboardInstance);
        }
        #[cfg(feature = "rive_tools")]
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
            let host = self.host.unwrap();
            if !unsafe { &mut *host }.hit_test_host(
                position,
                skip_on_unclipped,
                self as *mut Artboard as *mut ArtboardInstance,
            ) {
                return false;
            }
        }
        #[cfg(feature = "rive_tools")]
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

    pub fn draw(&mut self, renderer: &mut dyn Renderer) {
        FRAME_ID.fetch_add(1, Ordering::Relaxed);
        self.draw_canvases();
        self.draw_internal(renderer);
    }

    pub fn draw_internal(&mut self, renderer: &mut dyn Renderer) {
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
            renderer.transform(&transform);
        }
        if has_self {
            renderer.transform(&self.self_transform());
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
        let mut pending_clip_operations = Vec::<*mut Drawable>::new();
        let mut drawable = self.first_drawable;
        while let Some(mut pointer) = drawable {
            let drawable_ref = unsafe { pointer.as_mut() };
            drawable = drawable_ref.prev.and_then(NonNull::new);
            let previous_clips = empty_clips;
            empty_clips += drawable_ref.empty_clip_count();
            if !drawable_ref.will_draw() || empty_clips != previous_clips || empty_clips > 0 {
                continue;
            }
            if drawable_ref.is_clip_start() {
                pending_clip_operations.push(drawable_ref);
                continue;
            } else if !pending_clip_operations.is_empty() {
                if drawable_ref.is_clip_end() {
                    pending_clip_operations.pop();
                    continue;
                }
                for pending in pending_clip_operations.drain(..) {
                    unsafe { &mut *pending }.draw(renderer);
                }
            }
            drawable_ref.draw(renderer);
        }
        if save {
            renderer.restore();
        }
    }

    pub fn add_to_render_path(&mut self, path: &mut RenderPath, transform: &Mat2D) {
        let mut drawable = self.first_drawable;
        while let Some(mut pointer) = drawable {
            let drawable_ref = unsafe { pointer.as_mut() };
            drawable = drawable_ref.prev.and_then(NonNull::new);
            if drawable_ref.is_hidden() {
                continue;
            }
            if let Some(shape) = drawable_ref.as_shape_mut() {
                shape.add_to_render_path(path, transform);
            }
        }
    }

    pub fn add_to_raw_path(&mut self, path: &mut RawPath, transform: Option<&Mat2D>) {
        let mut drawable = self.first_drawable;
        while let Some(mut pointer) = drawable {
            let drawable_ref = unsafe { pointer.as_mut() };
            drawable = drawable_ref.prev.and_then(NonNull::new);
            if drawable_ref.is_hidden() {
                continue;
            }
            if let Some(shape) = drawable_ref.as_shape_mut() {
                shape.add_to_raw_path(path, transform);
            }
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
        if self.objects.iter().flatten().any(|object| unsafe {
            object.as_ref().core_type()
                == crate::mechanical_port::source::generated::audio_event_base::AudioEventBase::TYPE_KEY
        }) {
            return true;
        }
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            for i in 0..host.artboard_count() as i32 {
                if host.artboard_instance(i).is_some_and(Artboard::has_audio) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_animation_translucent(&self, animation: &LinearAnimation) -> bool {
        for keyed_object in animation.keyed_objects() {
            let pointer = self.resolve_ptr(keyed_object.object_id());
            for paint in self.base.base.shape_paints() {
                if pointer.is_some_and(|pointer| {
                    std::ptr::eq(
                        pointer.cast::<ShapePaint>(),
                        paint as *const ShapePaint as *mut ShapePaint,
                    )
                }) {
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
        self.animation_at(index)
            .map_or_else(String::new, |animation| animation.name().to_owned())
    }

    pub fn state_machine_name_at(&self, index: usize) -> String {
        self.state_machine_at(index)
            .map_or_else(String::new, |machine| machine.name().to_owned())
    }

    pub fn animation_named(&self, name: &str) -> Option<&LinearAnimation> {
        self.animations
            .iter()
            .copied()
            .map(|animation| unsafe { &*animation })
            .find(|animation| animation.name() == name)
    }

    pub fn animation_at(&self, index: usize) -> Option<&LinearAnimation> {
        self.animations
            .get(index)
            .copied()
            .map(|animation| unsafe { &*animation })
    }

    pub fn state_machine_named(&self, name: &str) -> Option<&StateMachine> {
        self.state_machines
            .iter()
            .copied()
            .map(|machine| unsafe { &*machine })
            .find(|machine| machine.name() == name)
    }

    pub fn state_machine_at(&self, index: usize) -> Option<&StateMachine> {
        self.state_machines
            .get(index)
            .copied()
            .map(|machine| unsafe { &*machine })
    }

    pub fn default_state_machine_index(&self) -> i32 {
        let index = self.base.default_state_machine_id() as usize;
        if index >= self.state_machines.len() {
            -1
        } else {
            index as i32
        }
    }

    pub fn nested_artboard(&self, name: &str) -> Option<&mut NestedArtboard> {
        self.nested_artboards
            .iter()
            .copied()
            .map(|nested| unsafe { &mut *nested })
            .find(|nested| nested.name() == name)
    }

    pub fn nested_artboard_at_path(&self, path: &str) -> Option<&mut NestedArtboard> {
        let (artboard_name, rest) = path.split_once('/').unwrap_or((path, ""));
        if artboard_name.is_empty() {
            return None;
        }
        let nested = self.nested_artboard(artboard_name)?;
        if rest.is_empty() {
            Some(nested)
        } else {
            nested.artboard_instance()?.nested_artboard_at_path(rest)
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
        let Some(backboard_importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::backboard::Backboard::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let result = self.base.base.import(import_stack);
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
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            for i in 0..host.artboard_count() as i32 {
                if let Some(artboard) = host.artboard_instance(i) {
                    artboard.set_volume(value);
                }
            }
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

    #[cfg(feature = "rive_tools")]
    pub fn on_layout_changed(&mut self, callback: Option<ArtboardCallback>) {
        self.layout_changed_callback = callback;
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_layout_dirty(&mut self, callback: Option<ArtboardCallback>) {
        self.layout_dirty_callback = callback;
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_transform_dirty(&mut self, callback: Option<ArtboardCallback>) {
        self.transform_dirty_callback = callback;
        self.add_dirt(ComponentDirt::COMPONENTS, false);
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_test_bounds(&mut self, callback: Option<TestBoundsCallback>) {
        self.test_bounds_callback = callback;
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_is_ancestor(&mut self, callback: Option<IsAncestorCallback>) {
        self.is_ancestor_callback = callback;
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_root_transform(&mut self, callback: Option<RootTransformCallback>) {
        self.root_transform_callback = callback;
    }

    #[cfg(feature = "external-rive_audio-engine")]
    pub fn audio_engine(&self) -> Option<Rc<AudioEngine>> {
        self.audio_engine.clone()
    }

    #[cfg(feature = "external-rive_audio-engine")]
    pub fn set_audio_engine(&mut self, audio_engine: Option<Rc<AudioEngine>>) {
        self.audio_engine = audio_engine.clone();
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            for i in 0..host.artboard_count() as i32 {
                if let Some(artboard) = host.artboard_instance(i) {
                    artboard.set_audio_engine(audio_engine.clone());
                }
            }
        }
    }

    pub fn is_ancestor(&mut self, artboard: Option<&Artboard>) -> bool {
        if artboard.is_some_and(|artboard| {
            self.artboard_source().artboard_source == artboard.artboard_source().artboard_source
        }) {
            return true;
        }
        if let Some(parent) = self.parent_artboard() {
            return parent.is_ancestor(artboard);
        }
        #[cfg(feature = "rive_tools")]
        if let (Some(callback), Some(artboard)) = (self.is_ancestor_callback, artboard)
            && callback(self.callback_user_data, artboard.artboard_id()) == 1
        {
            return true;
        }
        false
    }

    pub fn changed(&mut self) {
        if !self.did_change {
            self.did_change = true;
            if let Some(parent) = self.parent_artboard() {
                parent.changed();
            }
        }
    }

    fn has_parent_focus_data(
        focus_data: &crate::mechanical_port::source::focus_data::FocusData,
    ) -> bool {
        let mut current = focus_data.parent_ptr();
        while let Some(parent) = current {
            let parent = unsafe { &mut *parent };
            if let Some(node) = parent.as_node_mut() {
                for child in node.children() {
                    if child.is_focus_data()
                        && !std::ptr::eq(
                            child as *const Component,
                            focus_data as *const _ as *const Component,
                        )
                    {
                        return true;
                    }
                }
            }
            current = parent.parent_ptr();
        }
        false
    }

    pub fn root_focus_data_count(&self) -> usize {
        self.objects
            .iter()
            .flatten()
            .filter_map(|object| unsafe { object.as_ref().as_focus_data() })
            .filter(|focus_data| !Self::has_parent_focus_data(focus_data))
            .count()
    }

    pub fn root_focus_data_at(
        &self,
        index: usize,
    ) -> Option<&mut crate::mechanical_port::source::focus_data::FocusData> {
        self.objects
            .iter()
            .flatten()
            .filter_map(|object| unsafe { object.as_ref().as_focus_data_mut() })
            .filter(|focus_data| !Self::has_parent_focus_data(focus_data))
            .nth(index)
    }

    fn build_focus_tree_visit(
        focus_manager: &mut FocusManager,
        component: *mut Component,
        focus_node: Option<FocusNodeRef>,
    ) {
        if component.is_null() {
            return;
        }
        let component = unsafe { &mut *component };
        if let Some(nested_host) = component.as_nested_artboard_mut() {
            let mut rewired = false;
            for animation in nested_host.nested_animations() {
                if let Some(nested_state_machine) = animation.as_nested_state_machine_mut() {
                    if let Some(instance) = nested_state_machine.state_machine_instance_mut()
                        && !instance.focus_manager_is(focus_manager)
                    {
                        instance.set_external_focus_manager(focus_manager);
                        rewired = true;
                    }
                }
            }
            nested_host.sync_nested_focus_tree(focus_node.clone(), true, rewired);
        } else if let Some(list) = component.as_artboard_component_list_mut() {
            list.ensure_list_scope_focus_node(focus_manager, focus_node.clone());
        }
        if let Some(container) = component.as_container_component_mut() {
            let direct_focus_data = container
                .children()
                .iter()
                .copied()
                .filter_map(|child| unsafe { &mut *child }.as_focus_data_mut())
                .next();
            let recurse_with = if let Some(focus_data) = direct_focus_data {
                let node = focus_data.focus_node();
                focus_manager.add_child(focus_node.clone(), node.clone());
                Some(node)
            } else {
                focus_node
            };
            for child in container.children().iter().copied() {
                if child.is_null() || unsafe { &*child }.is_focus_data() {
                    continue;
                }
                Self::build_focus_tree_visit(focus_manager, child, recurse_with.clone());
            }
        }
    }

    pub fn build_focus_tree(
        &mut self,
        focus_manager: Option<&mut FocusManager>,
        parent_focus_node: Option<FocusNodeRef>,
    ) {
        let Some(focus_manager) = focus_manager else {
            return;
        };
        self.active_focus_manager = Some(NonNull::from(&mut *focus_manager));
        #[cfg(feature = "rive_tools")]
        if let Some(parent) = parent_focus_node.clone() {
            self.external_parent_focus_node = Some(parent);
        }
        #[cfg(feature = "rive_tools")]
        let effective_parent =
            parent_focus_node.or_else(|| self.external_parent_focus_node.clone());
        #[cfg(not(feature = "rive_tools"))]
        let effective_parent = parent_focus_node;
        Self::build_focus_tree_visit(
            focus_manager,
            self as *mut Artboard as *mut Component,
            effective_parent,
        );
    }

    pub fn build_focus_tree_from_parent(&mut self, parent: Option<FocusNodeRef>) {
        let Some(parent) = parent else {
            return;
        };
        let Some(manager) = parent.borrow().manager() else {
            return;
        };
        self.build_focus_tree(Some(unsafe { &mut *manager }), Some(parent));
    }

    pub fn cleanup_focus_tree(&mut self) {
        let Some(mut manager) = self.active_focus_manager else {
            return;
        };
        for object in self.objects.iter().flatten() {
            if let Some(focus_data) = unsafe { object.as_ref().as_focus_data_mut() }
                && let Some(node) = focus_data.existing_focus_node()
            {
                let node_manager = node.borrow().manager();
                let attached_without_manager =
                    node_manager.is_none() && node.borrow().parent().is_some();
                if node_manager.is_some_and(|candidate| std::ptr::eq(candidate, manager.as_ptr()))
                    || attached_without_manager
                {
                    unsafe { manager.as_mut() }.remove_child(node);
                }
            }
        }
        for nested_host in self.nested_artboards.iter().copied() {
            if let Some(nested) = unsafe { &mut *nested_host }.artboard_instance_at(0)
                && nested
                    .active_focus_manager
                    .is_some_and(|nested_manager| nested_manager == manager)
            {
                nested.cleanup_focus_tree();
            }
        }
        for list in self.component_lists.iter().copied() {
            let list = unsafe { &mut *list };
            for i in 0..list.artboard_count() as i32 {
                if let Some(nested) = list.artboard_instance(i)
                    && nested
                        .active_focus_manager
                        .is_some_and(|nested_manager| nested_manager == manager)
                {
                    nested.cleanup_focus_tree();
                }
            }
        }
        for list in self.component_lists.iter().copied() {
            unsafe { &mut *list }.remove_list_scope_focus_node();
        }
        self.active_focus_manager = None;
    }

    #[cfg(feature = "rive_tools")]
    pub fn set_external_parent_focus_node(&mut self, node: Option<FocusNodeRef>) {
        self.external_parent_focus_node = node;
    }

    #[cfg(feature = "rive_tools")]
    pub fn external_parent_focus_node(&self) -> Option<FocusNodeRef> {
        self.external_parent_focus_node.clone()
    }

    #[cfg(feature = "rive_tools")]
    pub fn collapse_single(&mut self, value: bool) {
        self.base.base.as_component_mut().collapse(value);
    }

    pub fn build_semantic_tree(
        &mut self,
        semantic_manager: Option<&mut SemanticManager>,
        parent_semantic_node: Option<SemanticNodeRef>,
    ) {
        let Some(semantic_manager) = semantic_manager else {
            return;
        };
        self.active_semantic_manager = Some(NonNull::from(&mut *semantic_manager));
        let mut effective_parent = parent_semantic_node.clone();
        if self.host.is_some() {
            if self.semantic_boundary_node.is_none() {
                let boundary = SemanticNode::new(0);
                {
                    let mut boundary_mut = boundary.borrow_mut();
                    boundary_mut.is_boundary_node = true;
                    boundary_mut.boundary_artboard = Some(self as *mut Artboard as usize);
                }
                self.semantic_boundary_node = Some(boundary);
            }
            let boundary = self.semantic_boundary_node.clone().unwrap();
            semantic_manager.add_child(parent_semantic_node, boundary.clone());
            self.mark_semantic_boundary_transform_dirty();
            effective_parent = Some(boundary);
        }

        for object in self.objects.iter().flatten() {
            if let Some(semantic_data) = unsafe { object.as_ref().as_semantic_data_mut() } {
                let parent = semantic_data
                    .find_parent_semantic_data()
                    .map(|data| data.semantic_node())
                    .or_else(|| effective_parent.clone());
                semantic_manager.add_child(parent, semantic_data.semantic_node());
                semantic_data.sync_semantic_tree_visibility();
            }
        }

        for nested_host in self.nested_artboards.iter().copied() {
            let nested_host = unsafe { &mut *nested_host };
            let parent = crate::mechanical_port::source::semantic::semantic_data::SemanticData::find_closest_semantic_node(
                nested_host,
            )
            .or_else(|| effective_parent.clone());
            if let Some(nested) = nested_host.artboard_instance_at(0)
                && !nested.semantic_manager_is(semantic_manager)
            {
                nested.cleanup_semantic_tree();
                nested.build_semantic_tree(Some(semantic_manager), parent);
            }
        }
        for list in self.component_lists.iter().copied() {
            let list = unsafe { &mut *list };
            let parent = crate::mechanical_port::source::semantic::semantic_data::SemanticData::find_closest_semantic_node(
                list,
            )
            .or_else(|| effective_parent.clone());
            for i in 0..list.artboard_count() as i32 {
                if let Some(nested) = list.artboard_instance(i)
                    && !nested.semantic_manager_is(semantic_manager)
                {
                    nested.cleanup_semantic_tree();
                    nested.build_semantic_tree(Some(semantic_manager), parent.clone());
                }
            }
        }
    }

    fn semantic_manager_is(&self, manager: &SemanticManager) -> bool {
        self.active_semantic_manager
            .is_some_and(|current| std::ptr::eq(current.as_ptr(), manager))
    }

    pub fn cleanup_semantic_tree(&mut self) {
        let Some(mut manager) = self.active_semantic_manager else {
            return;
        };
        for nested_host in self.nested_artboards.iter().copied() {
            if let Some(nested) = unsafe { &mut *nested_host }.artboard_instance_at(0)
                && nested.active_semantic_manager == Some(manager)
            {
                nested.cleanup_semantic_tree();
            }
        }
        for list in self.component_lists.iter().copied() {
            let list = unsafe { &mut *list };
            for i in 0..list.artboard_count() as i32 {
                if let Some(nested) = list.artboard_instance(i)
                    && nested.active_semantic_manager == Some(manager)
                {
                    nested.cleanup_semantic_tree();
                }
            }
        }
        for object in self.objects.iter().flatten() {
            if let Some(semantic_data) = unsafe { object.as_ref().as_semantic_data_mut() }
                && let Some(node) = semantic_data.existing_semantic_node()
                && node
                    .borrow()
                    .manager()
                    .is_some_and(|candidate| std::ptr::eq(candidate, manager.as_ptr()))
            {
                unsafe { manager.as_mut() }.remove_child(node);
            }
        }
        if let Some(boundary) = self.semantic_boundary_node.take()
            && boundary
                .borrow()
                .manager()
                .is_some_and(|candidate| std::ptr::eq(candidate, manager.as_ptr()))
        {
            unsafe { manager.as_mut() }.remove_child(boundary);
        }
        self.active_semantic_manager = None;
    }

    fn collapse_boundary_subtree(node: &SemanticNodeRef, value: bool) {
        let children = node.borrow().children().to_vec();
        for child in children {
            if let Some(semantic_data) = child.borrow().semantic_data_ptr() {
                let semantic_data = unsafe { &mut *semantic_data };
                if semantic_data.is_collapsed() != value {
                    semantic_data.collapse(value);
                }
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
            if let Some(semantic_data) = unsafe { object.as_ref().as_semantic_data_mut() }
                && semantic_data.is_collapsed() != value
            {
                semantic_data.collapse(value);
            }
        }
    }

    pub fn mark_semantic_boundary_transform_dirty(&mut self) {
        if let (Some(boundary), Some(mut manager)) = (
            self.semantic_boundary_node.as_ref(),
            self.active_semantic_manager,
        ) {
            unsafe { manager.as_mut() }.mark_boundary_dirty(boundary.borrow().id());
        }
    }

    fn clone_object_data_binds(
        &self,
        object: *const Core,
        clone: *mut Core,
        artboard: &mut Artboard,
    ) {
        for data_bind in self.data_bind_container.data_binds() {
            let data_bind = unsafe { &mut **data_bind };
            if data_bind.target_ptr() == object {
                let mut data_bind_clone = data_bind.clone_data_bind();
                data_bind_clone.set_target(clone);
                data_bind_clone.set_file(data_bind.file());
                data_bind_clone.initialize();
                if let Some(converter) = data_bind.converter() {
                    data_bind_clone.set_converter(Some(converter.clone_converter()));
                }
                artboard
                    .data_bind_container
                    .add_data_bind(Box::into_raw(data_bind_clone));
            }
        }
    }

    fn build_data_context(&mut self, _value: Rc<DataContext>) {}

    pub fn internal_data_context(&mut self, value: Rc<DataContext>) {
        self.data_context = Some(value.clone());
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            let instance = value.get_view_model_instance(&host.data_bind_path());
            if let Some(instance) = instance {
                host.bind_view_model_instance(instance, value.clone());
            } else {
                host.internal_data_context(value.clone());
            }
        }
        self.data_bind_container
            .bind_data_binds_from_context(Rc::as_ptr(&value) as *mut ());
        self.data_bind_container.sort_data_binds();
        for scripted_object in self.scripted_objects.iter().copied() {
            unsafe { &mut *scripted_object }.set_data_context(value.clone());
        }
        self.init_scripted_objects();
    }

    pub fn set_data_context(&mut self, value: Rc<DataContext>) {
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
        for host in self.artboard_hosts.iter().copied() {
            let host = unsafe { &mut *host };
            let value = context
                .get_view_model_instance(&host.data_bind_path())
                .or_else(|| context.main_view_model_instance());
            if let Some(value) = value {
                host.relink_data_context(value);
            }
        }
    }

    pub fn rebuild_data_bind(&mut self, data_bind: &mut DataBind) {
        if let Some(context_bind) = data_bind.as_data_bind_context_mut() {
            let pointer = self
                .data_context
                .as_ref()
                .map_or(std::ptr::null_mut(), |value| Rc::as_ptr(value) as *mut ());
            context_bind.bind_from_context(pointer);
        }
    }

    pub fn unbind(&mut self) {
        self.clear_data_context();
        self.data_bind_container.unbind_data_binds();
        for host in self.artboard_hosts.iter().copied() {
            unsafe { &mut *host }.unbind();
        }
    }

    pub fn clear_data_context(&mut self) {
        if let Some(mut context) = self.data_context.take() {
            if let Some(context) = Rc::get_mut(&mut context) {
                context.remove_dependent_container(self as *mut Artboard);
            }
        }
        for host in self.artboard_hosts.iter().copied() {
            unsafe { &mut *host }.clear_data_context();
        }
        for scripted_object in self.scripted_objects.iter().copied() {
            unsafe { &mut *scripted_object }.reset_lua_init();
        }
    }

    pub fn bind_view_model_instance(
        &mut self,
        view_model_instance: Option<Rc<dyn DataContextViewModelInstance>>,
    ) {
        self.bind_view_model_instance_with_parent(view_model_instance, None);
    }

    pub fn bind_view_model_instance_with_parent(
        &mut self,
        view_model_instance: Option<Rc<dyn DataContextViewModelInstance>>,
        parent: Option<Rc<DataContext>>,
    ) {
        let Some(instance) = view_model_instance else {
            self.unbind();
            return;
        };
        self.set_view_model_instance(Some(instance));
        if let (Some(parent), Some(context)) = (parent, self.data_context.as_mut()) {
            Rc::get_mut(context).unwrap().set_parent(Some(parent));
        }
        self.bind();
    }

    pub fn set_view_model_instance(
        &mut self,
        view_model_instance: Option<Rc<dyn DataContextViewModelInstance>>,
    ) {
        let Some(instance) = view_model_instance else {
            return;
        };
        if self.data_context.is_none() {
            let mut context = Rc::new(DataContext::new(Some(instance)));
            Rc::get_mut(&mut context)
                .unwrap()
                .add_dependent_container(self as *mut Artboard);
            self.data_context = Some(context);
            return;
        }
        Rc::get_mut(self.data_context.as_mut().unwrap())
            .unwrap()
            .set_main_view_model_instance(Some(instance));
    }

    pub fn bind_view_model_instances(
        &mut self,
        instances: Vec<Rc<dyn DataContextViewModelInstance>>,
        parent: Option<Rc<DataContext>>,
    ) {
        if instances.is_empty() {
            self.unbind();
            return;
        }
        self.clear_data_context();
        let mut context = Rc::new(DataContext::from_instances(instances));
        let context_mut = Rc::get_mut(&mut context).unwrap();
        context_mut.add_dependent_container(self as *mut Artboard);
        context_mut.set_parent(parent);
        self.internal_data_context(context);
    }

    pub fn bind(&mut self) {
        if let Some(context) = self.data_context.clone() {
            self.internal_data_context(context);
        }
    }

    pub fn global_view_model_instance(
        &self,
        name: &str,
    ) -> Option<Rc<dyn DataContextViewModelInstance>> {
        let context = self.data_context.as_ref()?;
        let file = self.artboard_file()?;
        context.instance_for_slot(file.view_model_id(name))
    }

    pub fn set_global_view_model_instance(
        &mut self,
        name: &str,
        instance: Option<Rc<dyn DataContextViewModelInstance>>,
    ) -> bool {
        let Some(file) = self.artboard_file() else {
            return false;
        };
        let slot_key = file.view_model_id(name);
        if slot_key >= file.view_model_count() {
            return false;
        }
        let Some(slot_view_model) = file.view_model(slot_key) else {
            return false;
        };
        if slot_view_model.view_model_type()
            != crate::mechanical_port::source::view_model_type::ViewModelType::Global
        {
            return false;
        }
        if self.data_context.is_none() {
            if instance.is_none() {
                return true;
            }
            let mut context = Rc::new(DataContext::new(None));
            Rc::get_mut(&mut context)
                .unwrap()
                .add_dependent_container(self as *mut Artboard);
            self.data_context = Some(context);
        }
        Rc::get_mut(self.data_context.as_mut().unwrap())
            .unwrap()
            .set_view_model_instance_for_slot(slot_key, instance);
        true
    }

    pub fn find<T>(&self, name: &str) -> Option<&mut T> {
        self.objects.iter().flatten().find_map(|object| unsafe {
            object
                .as_ref()
                .downcast_mut::<T>()
                .filter(|candidate| candidate.name() == name)
        })
    }

    pub fn count<T>(&self) -> usize {
        self.objects
            .iter()
            .flatten()
            .filter(|object| unsafe { object.as_ref().is::<T>() })
            .count()
    }

    pub fn object_at<T>(&self, index: usize) -> Option<&mut T> {
        self.objects
            .iter()
            .flatten()
            .filter_map(|object| unsafe { object.as_ref().downcast_mut::<T>() })
            .nth(index)
    }

    pub fn object_index(&self, component: *mut Core) -> i32 {
        self.objects
            .iter()
            .position(|object| object.is_some_and(|object| object.as_ptr() == component))
            .map_or(-1, |index| index as i32)
    }

    pub fn find_all<T>(&self) -> Vec<&mut T> {
        self.objects
            .iter()
            .flatten()
            .filter_map(|object| unsafe { object.as_ref().downcast_mut::<T>() })
            .collect()
    }

    pub fn instance(&self) -> Option<Box<ArtboardInstance>> {
        let mut clone = Box::new(ArtboardInstance::default());
        clone.base.base.copy(&self.base, &mut clone.base);
        clone.base.factory = self.factory;
        clone.base.frame_origin = self.frame_origin;
        clone.base.data_context = self.data_context.clone();
        clone.base.is_instance = true;
        clone.base.original_width = self.original_width;
        clone.base.original_height = self.original_height;
        #[cfg(feature = "rive_tools")]
        {
            clone.base.artboard_id = self.artboard_id;
        }
        clone.base.artboard_source = if self.is_instance {
            self.artboard_source
        } else {
            NonNull::new(self as *const Artboard as *mut Artboard)
        };
        self.clone_object_data_binds(
            self as *const Artboard as *const Core,
            &mut clone.base as *mut Artboard as *mut Core,
            &mut clone.base,
        );
        clone
            .base
            .objects
            .push(NonNull::new(&mut clone.base as *mut Artboard as *mut Core));
        for object in self.objects.iter().skip(1) {
            let cloned = object.and_then(|object| unsafe { object.as_ref().clone_object() });
            clone.base.objects.push(cloned);
            if let (Some(object), Some(cloned)) = (object, cloned) {
                self.clone_object_data_binds(object.as_ptr(), cloned.as_ptr(), &mut clone.base);
            }
        }
        clone
            .base
            .animations
            .extend(self.animations.iter().copied());
        clone
            .base
            .state_machines
            .extend(self.state_machines.iter().copied());
        if clone.base.initialize() != StatusCode::Ok {
            return None;
        }
        assert!(clone.base.is_instance());
        Some(clone)
    }

    fn artboard_file(&self) -> Option<Rc<File>> {
        None
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
    fn resolve(&self, id: u32) -> Option<&mut Core> {
        self.resolve_ptr(id).map(|pointer| unsafe { &mut *pointer })
    }
}

impl crate::mechanical_port::source::data_bind::data_context::DependentContainer for Artboard {}

impl AdvancingComponent for Artboard {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        self.advance_internal(elapsed_seconds, flags)
    }
}

impl Drop for Artboard {
    fn drop(&mut self) {
        // Focus cleanup is deliberately explicit. A StateMachineInstance may
        // already have destroyed the manager before its artboard is dropped.
        #[cfg(feature = "rive_audio")]
        {
            #[cfg(feature = "external-rive_audio-engine")]
            let engine = self.audio_engine.clone();
            #[cfg(not(feature = "external-rive_audio-engine"))]
            let engine = AudioEngine::runtime_engine(false);
            if let Some(engine) = engine {
                engine.stop(self);
            }
        }
        self.unbind();

        let mut vm_objects = HashSet::<*mut Core>::new();
        let mut deferred_vmi_unrefs = HashSet::<*mut ViewModelInstance>::new();
        for object in self
            .objects
            .iter()
            .chain(&self.invalid_objects)
            .flatten()
            .copied()
        {
            let pointer = object.as_ptr();
            if std::ptr::eq(pointer, self as *mut Artboard as *mut Core) {
                continue;
            }
            let object_ref = unsafe { &mut *pointer };
            if let Some(instance) = object_ref.as_view_model_instance_mut() {
                vm_objects.insert(pointer);
                deferred_vmi_unrefs.insert(instance);
            } else if object_ref.is_view_model_instance_value() {
                vm_objects.insert(pointer);
            }
        }
        for object in self.objects.drain(..).flatten() {
            let pointer = object.as_ptr();
            if std::ptr::eq(pointer, self as *mut Artboard as *mut Core)
                || vm_objects.contains(&pointer)
            {
                continue;
            }
            unsafe { Core::delete_object(pointer) };
        }
        for object in self.invalid_objects.drain(..).flatten() {
            if !vm_objects.contains(&object.as_ptr()) {
                unsafe { Core::delete_object(object.as_ptr()) };
            }
        }
        for instance in deferred_vmi_unrefs {
            unsafe { &mut *instance }.unref();
        }
        self.data_bind_container.delete_data_binds();
        if !self.is_instance {
            for animation in self.animations.drain(..) {
                unsafe { drop(Box::from_raw(animation)) };
            }
            for state_machine in self.state_machines.drain(..) {
                unsafe { drop(Box::from_raw(state_machine)) };
            }
        }
        self.dirty_layout.clear();
    }
}

pub struct ArtboardInstance {
    pub base: Artboard,
    file: Option<Rc<File>>,
}

impl Default for ArtboardInstance {
    fn default() -> Self {
        Self {
            base: Artboard::default(),
            file: None,
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

pub enum Scene {
    StateMachine(Box<StateMachineInstance>),
    LinearAnimation(Box<LinearAnimationInstance>),
}

impl ArtboardInstance {
    pub fn set_file(&mut self, file: Option<Rc<File>>) {
        self.file = file;
    }

    pub fn file(&self) -> Option<Rc<File>> {
        self.file.clone()
    }

    fn artboard_file(&self) -> Option<Rc<File>> {
        self.file.clone()
    }

    pub fn animation_at(&mut self, index: usize) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.base.animation_at(index)?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self as *mut ArtboardInstance as *mut (),
        )))
    }

    pub fn animation_named(&mut self, name: &str) -> Option<Box<LinearAnimationInstance>> {
        let animation = self.base.animation_named(name)?;
        Some(Box::new(LinearAnimationInstance::new(
            animation,
            self as *mut ArtboardInstance as *mut (),
        )))
    }

    pub fn state_machine_at(&mut self, index: usize) -> Option<Box<StateMachineInstance>> {
        let state_machine = self.base.state_machine_at(index)?;
        let mut instance = Box::new(StateMachineInstance::new(state_machine, self));
        if let Some(context) = self.base.data_context() {
            instance.inherit_data_context(context);
        }
        Some(instance)
    }

    pub fn state_machine_named(&mut self, name: &str) -> Option<Box<StateMachineInstance>> {
        let state_machine = self.base.state_machine_named(name)?;
        let mut instance = Box::new(StateMachineInstance::new(state_machine, self));
        if let Some(context) = self.base.data_context() {
            instance.inherit_data_context(context);
        }
        Some(instance)
    }

    pub fn default_state_machine(&mut self) -> Option<Box<StateMachineInstance>> {
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

    pub fn input(
        &mut self,
        name: &str,
        path: &str,
    ) -> Option<*mut crate::mechanical_port::source::state_machine::SmiInput> {
        self.named_input(name, path)
    }

    fn named_input<T>(&mut self, name: &str, path: &str) -> Option<*mut T> {
        if path.is_empty() {
            return None;
        }
        let nested = self.base.nested_artboard_at_path(path)?;
        let input = nested.input(name)?;
        input.input().map(|input| input.cast::<T>())
    }

    pub fn get_bool(
        &mut self,
        name: &str,
        path: &str,
    ) -> Option<*mut crate::mechanical_port::source::state_machine::SmiBool> {
        self.named_input(name, path)
    }

    pub fn get_number(
        &mut self,
        name: &str,
        path: &str,
    ) -> Option<*mut crate::mechanical_port::source::state_machine::SmiNumber> {
        self.named_input(name, path)
    }

    pub fn get_trigger(
        &mut self,
        name: &str,
        path: &str,
    ) -> Option<*mut crate::mechanical_port::source::state_machine::SmiTrigger> {
        self.named_input(name, path)
    }

    pub fn get_text_run(&mut self, name: &str, path: &str) -> Option<&mut TextValueRun> {
        if path.is_empty() {
            return None;
        }
        self.base
            .nested_artboard_at_path(path)?
            .artboard_instance()?
            .find::<TextValueRun>(name)
    }
}
