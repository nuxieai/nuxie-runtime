use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::{
        listener_invocation::ListenerInvocation, nested_state_machine::NestedStateMachine,
    },
    artboard::{
        Artboard, ArtboardInstance, RuntimeArtboardInstanceHandle,
        RuntimeArtboardInstanceWeakHandle,
    },
    artboard_host::ArtboardHost,
    artboard_referencer::{ArtboardReferencer, ArtboardReferencerBehavior, CoreArtboardReferencer},
    component_dirt::ComponentDirt,
    component_origin::ComponentOrigin,
    core::{Core, CoreHandle},
    core_context::CoreContext,
    data_bind::data_context::RuntimeDataContextHandle,
    data_bind_path_referencer::DataBindPathReferencer,
    file::RuntimeFileWeakHandle,
    focus_data::FocusData,
    generated::nested_artboard_base::{NestedArtboardBase, NestedArtboardBaseCallbacks},
    hit_info::HitInfo,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    input::{
        focus_manager::RuntimeFocusManagerHandle,
        focus_node::{FocusNode, FocusNodeRef},
        focusable::{Focusable, Key, KeyModifiers},
    },
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard_host_flags::NestedArtboardHostFlags,
    renderer::Renderer,
    status_code::StatusCode,
    view_model_type::ViewModelType,
};

fn build_vmi_list(primary: Option<CoreHandle>, globals: &[CoreHandle]) -> Vec<CoreHandle> {
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
    instance: Option<RuntimeArtboardInstanceHandle>,
    bound_nested_state_machine: Option<CoreHandle>,
    nested_animations: Vec<CoreHandle>,
    file: RuntimeFileWeakHandle,
    view_model_instance: Option<CoreHandle>,
    data_context: Option<RuntimeDataContextHandle>,
    active_view_model_instance: Option<CoreHandle>,
    global_view_model_instances: Vec<CoreHandle>,
    focus_scope: Option<FocusNodeRef>,
    focus_scope_manager: Option<RuntimeFocusManagerHandle>,
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
            file: RuntimeFileWeakHandle::default(),
            view_model_instance: None,
            data_context: None,
            active_view_model_instance: None,
            global_view_model_instances: Vec::new(),
            focus_scope: None,
            focus_scope_manager: None,
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
        for animation in &self.nested_animations {
            animation.with_mut(|animation| {
                animation.nested_animation_release_dependencies();
            });
        }
        if let Some(state_machine) = &self.bound_nested_state_machine {
            state_machine.with_mut(|state_machine| {
                state_machine.nested_animation_release_dependencies();
            });
        }

        self.view_model_instance = None;
        self.active_view_model_instance = None;
        self.owns_active_vmi = false;
        self.global_view_model_instances.clear();

        // The structural scope persists across swaps, but not host teardown.
        if let Some(scope) = self.focus_scope.take() {
            if let Some(manager) = self.focus_scope_manager.take() {
                manager.with_focus_manager_mut(|manager| manager.remove_child(&scope));
            }
        }
    }
}

impl NestedArtboard {
    pub(crate) fn take_artboard_instance(&mut self) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance.take()
    }

    pub(crate) fn restore_artboard_instance(&mut self, instance: RuntimeArtboardInstanceHandle) {
        debug_assert!(self.instance.is_none());
        self.instance = Some(instance);
    }

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
        let Some(parent_artboard) = self.parent_artboard_handle() else {
            return;
        };
        let Some(parent_focus_manager) = parent_artboard
            .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
            .flatten()
        else {
            return;
        };

        self.register_focus_scope(
            parent_focus_manager.clone(),
            fallback_parent.clone(),
            place_scope,
        );

        let Some(nested_instance) = self.artboard_instance_handle(0) else {
            return;
        };
        let already_shared = nested_instance.with_artboard(|instance| {
            instance
                .focus_manager_handle()
                .is_some_and(|manager| manager.ptr_eq(&parent_focus_manager))
        });
        if !force_rebuild && already_shared {
            return;
        }

        nested_instance.cleanup_focus_tree();
        let parent = self.focus_scope.clone().or(fallback_parent);
        nested_instance.build_focus_tree(Some(parent_focus_manager), parent);
    }

    pub fn sync_nested_focus_tree_default(&mut self, fallback_parent: Option<FocusNodeRef>) {
        self.sync_nested_focus_tree(fallback_parent, false, true);
    }

    pub fn sync_nested_focus_tree_occurrence(
        owner: &CoreHandle,
        fallback_parent: Option<FocusNodeRef>,
        place_scope: bool,
        force_rebuild: bool,
    ) {
        let parent_artboard = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .and_then(Self::parent_artboard_handle)
            })
            .flatten();
        let Some(parent_focus_manager) = parent_artboard.and_then(|parent| {
            parent
                .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
                .flatten()
        }) else {
            return;
        };
        owner
            .with_mut(|owner| {
                owner
                    .as_nested_artboard_mut()
                    .expect("NestedArtboard owner")
                    .register_focus_scope(
                        parent_focus_manager.clone(),
                        fallback_parent.clone(),
                        place_scope,
                    )
            })
            .expect("live NestedArtboard");
        let Some(nested_instance) = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .and_then(Self::artboard_instance_default)
            })
            .flatten()
        else {
            return;
        };
        let already_shared = nested_instance.with_artboard(|instance| {
            instance
                .focus_manager_handle()
                .is_some_and(|manager| manager.ptr_eq(&parent_focus_manager))
        });
        if !force_rebuild && already_shared {
            return;
        }
        nested_instance.cleanup_focus_tree();
        let scope = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .expect("NestedArtboard owner")
                    .focus_scope
                    .clone()
            })
            .expect("live NestedArtboard");
        nested_instance.build_focus_tree(Some(parent_focus_manager), scope.or(fallback_parent));
    }

    pub fn clone_core(&self) -> Box<NestedArtboard> {
        let mut nested_artboard = Box::new(NestedArtboard::default());
        // NestedArtboardBase::clone copies the generated base before this
        // owner restores its host-specific state.
        let mut base = std::mem::take(&mut nested_artboard.base);
        base.copy(self, &mut *nested_artboard);
        nested_artboard.base = base;
        nested_artboard.file = self.file.clone();
        if self.is_artboard_data_bound() {
            nested_artboard.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);
        }
        let Some(referenced) = self
            .instance
            .as_ref()
            .map(|instance| instance.core_handle())
            .or_else(|| self.artboard_referencer.referenced_artboard())
        else {
            return nested_artboard;
        };
        if let Some(instance) = Artboard::nested_instance_from_handle(&referenced) {
            nested_artboard.referenced_artboard_instance(instance);
        }
        nested_artboard
    }

    fn nest(&mut self, artboard: CoreHandle) {
        self.artboard_referencer
            .set_referenced_artboard(Some(artboard.clone()));
        // Upstream nest only records authored source artboards at import. A
        // supplied instance is mounted directly; cloning belongs to clone().
        let Some(instance) = artboard.runtime_artboard_instance() else {
            return;
        };
        self.nest_instance(instance);
    }

    fn nest_instance(&mut self, instance: RuntimeArtboardInstanceHandle) {
        self.artboard_referencer
            .set_referenced_artboard(Some(instance.core_handle()));
        let host = crate::mechanical_port::source::core::CoreObject::core(self).handle();
        let opacity = self.render_opacity();
        let parent = self.parent_artboard_handle();
        instance.with_artboard_mut(|instance| {
            instance.set_frame_origin(false);
            instance.set_host_opacity(opacity);
            let volume = instance.volume();
            instance.set_volume(volume);
        });
        self.instance = Some(instance.clone());
        instance.with_artboard_mut(|instance| instance.set_host_with_parent(host, parent));
        self.apply_origin_override();
    }

    fn nest_instance_occurrence(owner: &CoreHandle, instance: RuntimeArtboardInstanceHandle) {
        owner.with_mut(|owner| {
            owner
                .as_nested_artboard_mut()
                .expect("NestedArtboard host")
                .artboard_referencer
                .set_referenced_artboard(Some(instance.core_handle()));
        });
        instance.with_artboard_mut(|instance| instance.set_frame_origin(false));
        let opacity = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .expect("NestedArtboard host")
                    .render_opacity()
            })
            .expect("live NestedArtboard");
        instance.with_artboard_mut(|instance| {
            instance.set_host_opacity(opacity);
            let volume = instance.volume();
            instance.set_volume(volume);
        });
        // The source destroys the outgoing owned instance and takes ownership
        // of the new one before host() can enumerate this provider's nodes.
        let previous = owner
            .with_mut(|owner| {
                owner
                    .as_nested_artboard_mut()
                    .expect("NestedArtboard host")
                    .instance
                    .take()
            })
            .expect("live NestedArtboard");
        drop(previous);
        let parent = owner
            .with_mut(|owner| {
                let nested = owner.as_nested_artboard_mut().expect("NestedArtboard host");
                nested.instance = Some(instance.clone());
                nested.parent_artboard_handle()
            })
            .expect("live NestedArtboard");
        Artboard::set_host_occurrence(&instance.core_handle(), Some(owner.clone()), parent);
        Self::apply_origin_override_occurrence(owner);
    }

    fn origin_override_child(&self) -> Option<CoreHandle> {
        self.children().into_iter().find(|child| {
            child.is_type_of(crate::mechanical_port::source::generated::component_origin_base::ComponentOriginBase::TYPE_KEY)
        })
    }

    // nest() already owns the host. Pass that same owner to the child's dirty
    // callback, just as markLayoutDirty does for an already-borrowed host.
    fn apply_origin_override(&mut self) {
        if self.instance.is_none() {
            return;
        }
        let Some(origin) = self.origin_override_child() else {
            return;
        };
        let origin_x = origin
            .with_downcast::<ComponentOrigin, _>(|origin| origin.base.origin_x())
            .unwrap();
        let instance = self
            .instance
            .clone()
            .expect("mounted origin override owner");
        instance
            .with_artboard_mut(|instance| instance.set_origin_x_with_borrowed_host(origin_x, self));
        // The source reads originY after originX's setter and its callbacks.
        let origin_y = origin
            .with_downcast::<ComponentOrigin, _>(|origin| origin.base.origin_y())
            .unwrap();
        let instance = self
            .instance
            .clone()
            .expect("mounted origin override owner");
        instance
            .with_artboard_mut(|instance| instance.set_origin_y_with_borrowed_host(origin_y, self));
    }

    pub fn apply_origin_override_occurrence(owner: &CoreHandle) {
        let origin = owner
            .with(|owner| {
                let nested = owner.as_nested_artboard().expect("NestedArtboard owner");
                nested.instance.as_ref()?;
                nested.origin_override_child()
            })
            .flatten();
        let Some(origin) = origin else {
            return;
        };
        let origin_x = origin
            .with_downcast::<ComponentOrigin, _>(|origin| origin.base.origin_x())
            .unwrap();
        let instance = owner
            .with(|owner| owner.as_nested_artboard().unwrap().instance.clone())
            .flatten()
            .expect("mounted origin override owner");
        instance.with_artboard_mut(|instance| instance.set_origin_x(origin_x));
        let origin_y = origin
            .with_downcast::<ComponentOrigin, _>(|origin| origin.base.origin_y())
            .unwrap();
        let instance = owner
            .with(|owner| owner.as_nested_artboard().unwrap().instance.clone())
            .flatten()
            .expect("mounted origin override owner");
        instance.with_artboard_mut(|instance| instance.set_origin_y(origin_y));
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
        let Some(instance) = self.instance.clone() else {
            return;
        };
        let primary = self.active_view_model_instance.clone();
        let list = build_vmi_list(primary, &self.global_view_model_instances);
        instance.bind_view_model_instances(list, self.data_context.clone());
        let nested_data_context = instance.data_context();
        for animation in &self.nested_animations {
            animation.with_downcast_mut::<NestedStateMachine, _>(|state_machine| {
                if let Some(context) = nested_data_context.clone() {
                    state_machine.data_context(context);
                }
            });
        }
    }

    fn find_stateful_child_vmi(&self) -> Option<CoreHandle> {
        for child in self.children() {
            if child
                .with(|child| child.as_view_model_instance().is_some())
                .unwrap_or(false)
            {
                return Some(child.clone());
            }
        }
        None
    }

    fn set_active_view_model_instance(&mut self, instance: Option<CoreHandle>, owns: bool) {
        if self.active_view_model_instance == instance {
            return;
        }
        self.active_view_model_instance = instance;
        self.owns_active_vmi = owns;
    }

    fn clear_nested_animations(&mut self) {
        for animation in &self.nested_animations {
            animation.with_mut(|animation| {
                animation.nested_animation_release_dependencies();
            });
        }
        self.nested_animations.clear();
    }

    pub fn update_artboard(&mut self, view_model_instance_artboard: Option<CoreHandle>) {
        if let Some((artboard, instance)) =
            self.prepare_artboard_update(view_model_instance_artboard.clone())
        {
            self.referenced_artboard_instance(instance);
            self.finish_artboard_update(artboard, view_model_instance_artboard);
        }
    }

    pub(crate) fn update_artboard_occurrence(owner: &CoreHandle, value: Option<CoreHandle>) {
        let prepared = owner
            .with_mut(|owner| {
                owner
                    .as_nested_artboard_mut()
                    .expect("NestedArtboard host")
                    .prepare_artboard_update(value.clone())
            })
            .expect("live NestedArtboard");
        if let Some((artboard, instance)) = prepared {
            Self::nest_instance_occurrence(owner, instance);
            owner.with_mut(|owner| {
                let nested = owner.as_nested_artboard_mut().expect("NestedArtboard host");
                nested.try_schedule_bind_stateful();
                nested.finish_artboard_update(artboard, value);
            });
        }
    }

    fn prepare_artboard_update(
        &mut self,
        view_model_instance_artboard: Option<CoreHandle>,
    ) -> Option<(CoreHandle, RuntimeArtboardInstanceHandle)> {
        self.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);

        let explicit_null = view_model_instance_artboard
            .as_ref()
            .is_some_and(|property| {
                property
                    .with(|property| {
                        property
                            .as_view_model_instance_artboard()
                            .is_some_and(|property| {
                                property.asset().is_none()
                                    && property.base.property_value() == u32::MAX
                            })
                    })
                    .unwrap_or(false)
            });
        let parent = self.parent_artboard_handle();
        let artboard = if explicit_null {
            None
        } else {
            ArtboardReferencer::find_artboard(
                view_model_instance_artboard.clone(),
                parent,
                Some(self.file.clone()),
            )
        };
        if !explicit_null && artboard.is_none() {
            return None;
        }

        if let Some(instance) = self.artboard_instance_handle(0) {
            instance.cleanup_focus_tree();
        }
        self.clear_data_context();
        self.clear_nested_animations();
        self.bound_nested_state_machine = None;

        if explicit_null {
            if self.instance.is_none() {
                if let Some(referenced) = self.artboard_referencer.referenced_artboard() {
                    referenced.with_downcast_mut::<Artboard, _>(|artboard| {
                        artboard.set_host_handle(None)
                    });
                }
            }
            self.artboard_referencer.set_referenced_artboard(None);
            self.instance = None;
            self.set_active_view_model_instance(None, false);
            return None;
        }

        if let Some(artboard) = artboard {
            let artboard_instance = Artboard::nested_instance_from_handle(&artboard);
            let state_machine_count = artboard
                .with_downcast::<Artboard, _>(Artboard::state_machine_count)
                .unwrap_or_default();
            if state_machine_count > 0
                && let Some(owner) =
                    crate::mechanical_port::source::core::CoreObject::core(self).handle()
                && let Some(nested_state_machine) =
                    owner.insert_sibling(NestedStateMachine::default())
            {
                crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(
                    &nested_state_machine,
                    crate::mechanical_port::source::generated::nested_animation_base::NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY as i32,
                    0,
                );
                if let Some(instance) = artboard_instance.as_ref() {
                    crate::mechanical_port::source::animation::nested_animation::initialize_animation(&nested_state_machine, instance.downgrade());
                }
                self.add_nested_animation_handle(nested_state_machine.clone());
                self.bound_nested_state_machine = Some(nested_state_machine);
            }
            return artboard_instance.map(|instance| (artboard, instance));
        }
        None
    }

    fn finish_artboard_update(
        &mut self,
        artboard: CoreHandle,
        view_model_instance_artboard: Option<CoreHandle>,
    ) {
        if self.base.is_stateful() {
            let stateful_child = self.find_stateful_child_vmi();
            let artboard_view_model_id = artboard
                .with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id())
                .unwrap_or_default();
            let same_view_model = |instance: &CoreHandle| {
                instance
                    .with(|instance| {
                        instance.as_view_model_instance().is_some_and(|instance| {
                            instance.base.view_model_id() == artboard_view_model_id
                        })
                    })
                    .unwrap_or(false)
            };
            if stateful_child.as_ref().is_some_and(&same_view_model) {
                self.set_active_view_model_instance(stateful_child, false);
            } else if self.owns_active_vmi
                && self
                    .active_view_model_instance
                    .as_ref()
                    .is_some_and(&same_view_model)
            {
                // Reuse the already-owned binding for the same ViewModel.
            } else {
                let bound = self
                    .file
                    .with_file_mut(|file| {
                        let model = file.view_model(artboard_view_model_id as usize)?;
                        file.create_default_view_model_instance(model)
                    })
                    .flatten();
                self.set_active_view_model_instance(bound.clone(), bound.is_some());
                if let Some(bound) = bound {
                    self.file.complete_view_model_properties(&bound);
                }
            }
        } else {
            self.set_active_view_model_instance(None, false);
        }

        let bound = view_model_instance_artboard.as_ref().and_then(|property| {
            property
                .with(|property| {
                    property
                        .as_view_model_instance_artboard()
                        .and_then(|property| property.bound_view_model_instance())
                })
                .flatten()
        });
        if let Some(bound) = bound {
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

        let parent_focus_manager = self.parent_artboard_handle().and_then(|parent| {
            parent
                .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
                .flatten()
        });
        if let (Some(parent_focus_manager), Some(state_machine)) = (
            parent_focus_manager,
            self.bound_nested_state_machine.as_ref(),
        ) {
            state_machine.with_downcast_mut::<NestedStateMachine, _>(|state_machine| {
                if let Some(instance) = state_machine.state_machine_instance() {
                    instance.with_instance_mut(|instance| {
                        instance.set_external_focus_manager_handle(parent_focus_manager)
                    });
                }
            });
        }
        let fallback = FocusData::find_closest_focus_node_from_parent(self.parent_handle());
        self.sync_nested_focus_tree(fallback, false, true);
    }

    fn detect_artboard_data_binding(&mut self) {
        if self.is_artboard_data_bound() {
            return;
        }
        let Some(parent) = self.parent_artboard_handle() else {
            return;
        };
        let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        let data_binds = parent
            .with_downcast::<Artboard, _>(Artboard::data_bind_handles)
            .unwrap_or_default();
        for data_bind in data_binds {
            let matches = data_bind
                .with(|data_bind| {
                    let data_bind = data_bind.as_data_bind().expect("DataBind-derived owner");
                    data_bind.target().as_ref() == Some(&owner)
                        && data_bind.base.property_key()
                            == NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY as u32
                })
                .unwrap_or(false);
            if matches {
                self.set_host_flag(NestedArtboardHostFlags::ARTBOARD_DATA_BOUND);
                return;
            }
        }
    }

    fn register_focus_scope(
        &mut self,
        focus_manager: RuntimeFocusManagerHandle,
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
        if !place
            && self
                .focus_scope_manager
                .as_ref()
                .is_some_and(|current| current.ptr_eq(&focus_manager))
        {
            return;
        }
        if let Some(previous) = self.focus_scope_manager.replace(focus_manager.clone()) {
            previous.with_focus_manager_mut(|manager| manager.remove_child(&scope));
        }
        focus_manager.with_focus_manager_mut(|manager| manager.add_child(parent_node, scope, None));
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.needs_save_operation() {
            renderer.save();
        }
        renderer.transform(nuxie_render_api::Mat2D(*self.world_transform().values()));
        if let Some(instance) = &self.instance {
            instance.draw_internal(renderer);
        }
        if self.needs_save_operation() {
            renderer.restore();
        }
    }

    pub fn will_draw(&self) -> bool {
        self.base.base.will_draw() && self.artboard_referencer.referenced_artboard().is_some()
    }

    pub fn hit_test(&mut self, hit_info: &mut HitInfo, transform: &Mat2D) -> Option<CoreHandle> {
        let instance = self.instance.clone()?;
        let mounted = crate::mechanical_port::source::core::CoreObject::core(self)
            .handle()
            .expect("arena-owned NestedArtboard");
        hit_info.mounts.push(mounted);
        let mounted_translation = instance.with_artboard(|instance| make_translate(&instance.base));
        let mounted_transform = *transform * self.world_transform() * mounted_translation;
        if let Some(component) = instance
            .with_artboard_mut(|instance| instance.hit_test_handle(hit_info, &mounted_transform))
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
        _artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> bool {
        let mounted_position = self.world_transform() * *position;
        self.parent().is_some_and(|parent| {
            parent
                .with_mut(|parent| {
                    parent.component_hit_test_point(&mounted_position, skip_on_unclipped, false)
                })
                .flatten()
                .unwrap_or(false)
        })
    }

    pub fn host_transform_point(
        &self,
        point: &Vec2D,
        _artboard_instance: RuntimeArtboardInstanceWeakHandle,
    ) -> Vec2D {
        let local = Vec2D::transform_mat2d(*point, &self.world_transform());
        self.parent_artboard_handle()
            .and_then(|artboard| {
                artboard.with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(local))
            })
            .unwrap_or(local)
    }

    pub fn world_transform_for_artboard(
        &self,
        _artboard_instance: RuntimeArtboardInstanceWeakHandle,
    ) -> Mat2D {
        self.world_transform()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        let Some(backboard_importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return StatusCode::MissingObject;
        };
        backboard_importer.add_artboard_referencer(this);
        self.base.base.import(import_stack)
    }

    pub fn add_nested_animation_handle(&mut self, nested_animation: CoreHandle) {
        self.nested_animations.push(nested_animation);
    }

    pub fn on_added_clean_occurrence(
        owner: &CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let has_instance = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .expect("NestedArtboard owner")
                    .instance
                    .is_some()
            })
            .expect("live NestedArtboard");
        if has_instance {
            let animations = owner
                .with(|owner| {
                    owner
                        .as_nested_artboard()
                        .expect("NestedArtboard owner")
                        .nested_animations
                        .clone()
                })
                .expect("live NestedArtboard");
            for animation in animations {
                let instance = owner
                    .with(|owner| {
                        owner
                            .as_nested_artboard()
                            .expect("NestedArtboard owner")
                            .instance
                            .clone()
                    })
                    .expect("live NestedArtboard")
                    .expect("mounted nested instance");
                crate::mechanical_port::source::animation::nested_animation::initialize_animation(
                    &animation,
                    instance.downgrade(),
                );
            }
            let (instance, parent) = owner
                .with(|owner| {
                    let nested = owner.as_nested_artboard().expect("NestedArtboard owner");
                    (
                        nested.instance.clone().expect("mounted nested instance"),
                        nested.parent_artboard_handle(),
                    )
                })
                .expect("live NestedArtboard");
            // host() synchronously rebuilds the parent's layout children, which
            // visits this same NestedArtboard and the newly hosted child.
            Artboard::set_host_occurrence(&instance.core_handle(), Some(owner.clone()), parent);
            Self::apply_origin_override_occurrence(owner);
        }
        let children = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .expect("NestedArtboard owner")
                    .children()
                    .to_vec()
            })
            .expect("live NestedArtboard");
        for child in children {
            if !child.is_type_of(crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY) { continue; }
            let view_model = child
                .with(|child| {
                    child
                        .as_view_model_instance()
                        .and_then(|instance| instance.get_view_model())
                })
                .flatten();
            let view_model_type = view_model
                .and_then(|view_model| {
                    view_model
                        .with(|view_model| {
                            view_model
                                .as_view_model()
                                .map(|view_model| view_model.base.view_model_type())
                        })
                        .flatten()
                })
                .unwrap_or(ViewModelType::Standard as u32);
            let file = owner
                .with(|owner| {
                    owner
                        .as_nested_artboard()
                        .expect("NestedArtboard owner")
                        .file
                        .clone()
                })
                .expect("live NestedArtboard");
            file.complete_view_model_properties(&child);
            owner
                .with_mut(|owner| {
                    let nested = owner
                        .as_nested_artboard_mut()
                        .expect("NestedArtboard owner");
                    if view_model_type == ViewModelType::Global as u32 {
                        nested.global_view_model_instances.push(child);
                    } else if nested.active_view_model_instance.is_none() {
                        nested.active_view_model_instance = Some(child);
                        nested.owns_active_vmi = false;
                    }
                })
                .expect("live NestedArtboard");
        }
        owner
            .with_mut(|owner| {
                let nested = owner
                    .as_nested_artboard_mut()
                    .expect("NestedArtboard owner");
                nested.try_schedule_bind_stateful();
                nested.detect_artboard_data_binding();
            })
            .expect("live NestedArtboard");
        owner
            .with_mut(|owner| owner.on_added_clean(context))
            .expect("live NestedArtboard")
    }

    pub(crate) fn on_added_clean_after_animation_initialization(
        &mut self,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        self.base.base.on_added_clean(context)
    }

    pub(crate) fn update_after_transform_occurrence(owner: &CoreHandle, value: ComponentDirt) {
        let Some((referenced, instance)) = owner
            .with(|owner| {
                let nested = owner.as_nested_artboard()?;
                Some((nested.source_artboard()?, nested.instance.clone()))
            })
            .flatten()
        else {
            return;
        };
        if value.contains(ComponentDirt::WORLD_TRANSFORM)
            && let Some(instance) = instance
        {
            instance
                .with_artboard_mut(|instance| instance.mark_semantic_boundary_transform_dirty());
        }
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            let opacity = owner
                .with(|owner| {
                    owner
                        .as_nested_artboard()
                        .expect("NestedArtboard owner")
                        .render_opacity()
                })
                .expect("live NestedArtboard");
            referenced
                .with_downcast_mut::<Artboard, _>(|artboard| artboard.set_host_opacity(opacity));
        }
        let needs_update = value.contains(ComponentDirt::COMPONENTS)
            || (owner
                .with(|owner| {
                    owner
                        .as_nested_artboard()
                        .expect("NestedArtboard owner")
                        .base
                        .is_paused()
                })
                .expect("live NestedArtboard")
                && referenced
                    .with_downcast::<Artboard, _>(Artboard::has_component_dirt)
                    .expect("referenced Artboard"));
        if needs_update {
            // Child layout may synchronously dirty this host. Keep the host's
            // slot available, as the upstream pointer call does.
            Artboard::update_pass_handle(&referenced, false);
        }
    }

    pub(crate) fn collapse_after_super_occurrence(owner: &CoreHandle, value: bool) {
        let instance = owner
            .with(|object| {
                object
                    .as_nested_artboard()
                    .expect("NestedArtboard collapse owner")
                    .instance
                    .clone()
            })
            .expect("live NestedArtboard collapse owner");
        if let Some(instance) = instance {
            instance.collapse_semantic_boundary(value);
        }
    }

    pub fn has_nested_state_machines(&self) -> bool {
        self.nested_animations.iter().any(|animation| {
            animation
                .is_type_of(crate::mechanical_port::source::generated::animation::nested_state_machine_base::NestedStateMachineBase::TYPE_KEY)
        })
    }

    pub fn nested_animations(&self) -> &[CoreHandle] {
        &self.nested_animations
    }

    pub fn nested_artboard(&self, name: &str) -> Option<CoreHandle> {
        self.instance.as_ref()?.with_artboard(|instance| {
            instance.nested_artboards().into_iter().find(|nested| {
                nested
                    .with(|object| {
                        object
                            .as_nested_artboard()
                            .is_some_and(|nested| nested.base.base.name() == name)
                    })
                    .unwrap_or(false)
            })
        })
    }

    pub fn state_machine(&self, name: &str) -> Option<CoreHandle> {
        self.nested_animations.iter().find_map(|animation| {
            animation
                .with_downcast::<NestedStateMachine, _>(|state_machine| {
                    state_machine.base.base.name() == name
                })
                .unwrap_or(false)
                .then(|| animation.clone())
        })
    }

    pub fn input(&self, name: &str) -> Option<CoreHandle> {
        self.input_for_state_machine(name, "")
    }

    pub fn input_for_state_machine(
        &self,
        name: &str,
        state_machine_name: &str,
    ) -> Option<CoreHandle> {
        if !state_machine_name.is_empty() {
            return self.state_machine(state_machine_name).and_then(|machine| {
                machine
                    .with_downcast::<NestedStateMachine, _>(|machine| {
                        (0..machine.input_count()).find_map(|index| {
                            (machine.input_name(index as u32).as_deref() == Some(name))
                                .then(|| machine.input(index))
                                .flatten()
                        })
                    })
                    .flatten()
            });
        }
        self.nested_animations.iter().find_map(|animation| {
            animation
                .with_downcast::<NestedStateMachine, _>(|machine| {
                    (0..machine.input_count()).find_map(|index| {
                        (machine.input_name(index as u32).as_deref() == Some(name))
                            .then(|| machine.input(index))
                            .flatten()
                    })
                })
                .flatten()
        })
    }

    pub fn world_to_local(&self, world: Vec2D, local: &mut Vec2D) -> bool {
        if self.artboard_referencer.referenced_artboard().is_none() {
            return false;
        }
        let mut inverse = Mat2D::default();
        if !self.world_transform().invert(&mut inverse) {
            return false;
        }
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
            maximum_width.min(self.instance.as_ref().map_or(0.0, |instance| {
                instance.with_artboard(|instance| instance.width())
            })),
            maximum_height.min(self.instance.as_ref().map_or(0.0, |instance| {
                instance.with_artboard(|instance| instance.height())
            })),
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

    pub fn copy_data_bind_path_ids(&mut self, object: &NestedArtboard) {
        self.data_bind_path_referencer
            .copy_data_bind_path(&object.data_bind_path_referencer);
    }

    pub fn internal_data_context(&mut self, value: Option<RuntimeDataContextHandle>) {
        self.data_context = value.clone();
        self.view_model_instance = None;
        let Some(instance) = self.instance.clone() else {
            return;
        };
        if self.try_schedule_bind_stateful() {
            return;
        }
        if !self.global_view_model_instances.is_empty() {
            let list = build_vmi_list(None, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, value);
            let nested_context = instance.data_context();
            self.set_nested_state_machine_context(nested_context);
            return;
        }

        match value.clone() {
            Some(context) => {
                instance.internal_data_context(context);
            }
            None => instance.clear_data_context(),
        }
        self.set_nested_state_machine_context(value);
    }

    fn set_nested_state_machine_context(&mut self, value: Option<RuntimeDataContextHandle>) {
        for animation in &self.nested_animations {
            animation.with_downcast_mut::<NestedStateMachine, _>(|state_machine| {
                match value.clone() {
                    Some(context) => state_machine.data_context(context),
                    None => state_machine.clear_data_context(),
                }
            });
        }
    }

    pub fn relink_data_context(&mut self, view_model_instance: Option<CoreHandle>) {
        self.view_model_instance = view_model_instance.clone();
        if self.base.is_stateful() {
            return;
        }
        let Some(instance) = self.instance.as_ref() else {
            return;
        };
        if let Some(context) = instance.data_context() {
            if context.with_context(|context| context.main_view_model_instance())
                != view_model_instance
            {
                context.with_context_mut(|context| {
                    context.set_view_model_instance(view_model_instance)
                });
            }
        }
        instance.relink_data_context();
    }

    pub fn clear_data_context(&mut self) {
        let Some(instance) = self.instance.as_ref() else {
            return;
        };
        instance.clear_data_context();
        self.set_nested_state_machine_context(None);
    }

    pub fn unbind(&mut self) {
        if let Some(instance) = self.instance.as_ref() {
            instance.unbind();
        }
    }

    pub fn update_data_binds(&mut self) {
        if !self.base.is_paused() {
            if let Some(instance) = self.instance.as_ref() {
                instance.update_data_binds(true);
            }
        }
    }

    pub(crate) fn update_data_binds_occurrence(owner: &CoreHandle) {
        let instance = owner
            .with(|owner| {
                let nested = owner.as_nested_artboard().expect("NestedArtboard host");
                nested.instance.clone().filter(|_| !nested.base.is_paused())
            })
            .flatten();
        if let Some(instance) = instance {
            instance.update_data_binds(true);
        }
    }

    pub fn bind_view_model_instance(
        &mut self,
        view_model_instance: Option<CoreHandle>,
        parent: Option<RuntimeDataContextHandle>,
    ) {
        self.data_context = parent.clone();
        self.view_model_instance = view_model_instance.clone();
        let Some(instance) = self.instance.clone() else {
            return;
        };

        if let Some(active) = self.active_view_model_instance.clone() {
            let primary = Some(active);
            let list = build_vmi_list(primary, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, parent);
            let nested_context = instance.data_context();
            self.set_nested_state_machine_context(nested_context);
            return;
        }

        if view_model_instance.is_some() || !self.global_view_model_instances.is_empty() {
            let list = build_vmi_list(view_model_instance, &self.global_view_model_instances);
            instance.bind_view_model_instances(list, parent);
            let nested_context = instance.data_context();
            self.set_nested_state_machine_context(nested_context);
            return;
        }

        match parent.clone() {
            Some(context) => {
                instance.internal_data_context(context);
            }
            None => instance.clear_data_context(),
        }
        self.set_nested_state_machine_context(parent);
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

    pub fn advance_component_occurrence(
        owner: &CoreHandle,
        elapsed_seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        let stopped = owner
            .with(|owner| {
                let owner = owner.as_nested_artboard().expect("NestedArtboard owner");
                owner.artboard_referencer.referenced_artboard().is_none()
                    || owner.is_collapsed()
                    || owner.base.is_paused()
            })
            .expect("live NestedArtboard owner");
        if stopped {
            return false;
        }
        owner.with_mut(|owner| {
            let owner = owner
                .as_nested_artboard_mut()
                .expect("NestedArtboard owner");
            if owner.has_host_flag(NestedArtboardHostFlags::PENDING_STATEFUL_BINDING) {
                owner.bind_stateful();
            }
        });
        let mut keep_going = false;
        let advance_nested =
            flags.0 & AdvanceFlags::ADVANCE_NESTED.0 == AdvanceFlags::ADVANCE_NESTED.0;
        let (local_elapsed_seconds, quantize) = owner
            .with_mut(|owner| {
                let owner = owner
                    .as_nested_artboard_mut()
                    .expect("NestedArtboard owner");
                (
                    owner.calculate_local_elapsed_seconds(elapsed_seconds),
                    owner.base.quantize(),
                )
            })
            .expect("live NestedArtboard owner");
        let new_frame = flags.0 & AdvanceFlags::NEW_FRAME.0 == AdvanceFlags::NEW_FRAME.0;
        if local_elapsed_seconds == 0.0 && quantize >= 0.0 && new_frame {
            return true;
        }
        if advance_nested {
            let animation_count = owner
                .with(|owner| {
                    owner
                        .as_nested_artboard()
                        .expect("NestedArtboard owner")
                        .nested_animations
                        .len()
                })
                .expect("live NestedArtboard owner");
            for index in 0..animation_count {
                let animation = owner
                    .with(|owner| {
                        owner
                            .as_nested_artboard()
                            .expect("NestedArtboard owner")
                            .nested_animations
                            .get(index)
                            .cloned()
                    })
                    .flatten()
                    .expect("nested animation array remains stable during advance");
                if !new_frame {
                    let changed = animation.with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                        .flatten().is_some_and(|machine| machine.with_instance_mut(
                            crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance::try_change_state));
                    if changed
                        && crate::mechanical_port::source::generated::core_registry::nested_animation_advance_handle(
                            &animation, local_elapsed_seconds, new_frame).unwrap_or(false)
                    {
                        keep_going = true;
                    }
                } else if crate::mechanical_port::source::generated::core_registry::nested_animation_advance_handle(
                    &animation, local_elapsed_seconds, new_frame).unwrap_or(false)
                {
                    keep_going = true;
                }
            }
        }

        let advancing_flags = AdvanceFlags(flags.0 & !AdvanceFlags::IS_ROOT.0);
        let instance = owner
            .with(|owner| {
                owner
                    .as_nested_artboard()
                    .expect("NestedArtboard owner")
                    .instance
                    .clone()
            })
            .flatten();
        if let Some(instance) = instance {
            if instance.advance_internal(local_elapsed_seconds, advancing_flags) {
                keep_going = true;
            }
            if instance.with_artboard(|instance| instance.has_component_dirt()) {
                owner.with_mut(|owner| {
                    owner
                        .as_nested_artboard_mut()
                        .expect("NestedArtboard owner")
                        .add_dirt(ComponentDirt::COMPONENTS)
                });
            }
        }
        keep_going
    }

    pub fn reset_impl(&mut self) {
        if let Some(instance) = self.instance.as_ref() {
            instance.with_artboard_mut(|instance| instance.reset());
        }
        if let Some(active) = self.active_view_model_instance.as_ref() {
            active.with_mut(|active| {
                if let Some(active) = active.as_view_model_instance_mut() {
                    active.advanced();
                }
            });
        }
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    pub fn set_file(&mut self, value: RuntimeFileWeakHandle) {
        self.file = value;
    }

    pub fn referenced_artboard_id(&self) -> i32 {
        self.base.artboard_id() as i32
    }

    pub fn referenced_artboard(&mut self, artboard: Option<CoreHandle>) {
        let artboard = artboard.expect("NestedArtboard requires a referenced artboard");
        self.nest(artboard);
        self.try_schedule_bind_stateful();
    }

    pub fn referenced_artboard_instance(&mut self, instance: RuntimeArtboardInstanceHandle) {
        self.nest_instance(instance);
        self.try_schedule_bind_stateful();
    }

    pub fn artboard_count(&self) -> usize {
        1
    }

    pub fn type_(&self) -> i32 {
        self.base.core_type() as i32
    }

    pub fn artboard_instance_handle(&self, _index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance.clone()
    }

    pub fn artboard_instance_default(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance_handle(0)
    }

    pub fn source_artboard(&self) -> Option<CoreHandle> {
        self.artboard_referencer.referenced_artboard()
    }

    pub fn parent_artboard_handle(&self) -> Option<CoreHandle> {
        self.base.base.artboard_handle()
    }

    pub fn mark_host_transform_dirty(&mut self) {
        self.mark_transform_dirty();
    }

    pub fn host_component(&mut self) -> Option<CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
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

    pub fn gamepad_dispatch(&mut self, _invocation: &ListenerInvocation) -> bool {
        false
    }

    pub fn focused(&mut self) {}

    pub fn blurred(&mut self) {}

    pub fn focusable_artboard(&self) -> Option<CoreHandle> {
        self.parent_artboard_handle()
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

    fn parent(&self) -> Option<CoreHandle> {
        self.base.base.parent_handle()
    }

    fn children(&self) -> Vec<CoreHandle> {
        self.base.base.base.children().to_vec()
    }

    fn render_opacity(&self) -> f32 {
        self.base.base.render_opacity()
    }

    fn world_transform(&self) -> Mat2D {
        *self.base.base.world_transform()
    }

    fn needs_save_operation(&self) -> bool {
        self.base.base.needs_save_operation()
    }

    fn add_dirt(&mut self, dirt: ComponentDirt) {
        self.base.base.base.add_dirt(dirt, false);
    }

    fn is_collapsed(&self) -> bool {
        self.base.base.base.is_collapsed()
    }

    fn mark_transform_dirty(&mut self) {
        self.base.base.mark_transform_dirty();
    }
}

impl crate::mechanical_port::source::resetting_component::ResettingComponent for NestedArtboard {
    fn reset(&mut self) {
        self.reset_impl();
    }
}

impl ArtboardHost for NestedArtboard {
    fn data_bind_path_referencer(&self) -> &DataBindPathReferencer {
        &self.data_bind_path_referencer
    }

    fn artboard_count(&self) -> usize {
        NestedArtboard::artboard_count(self)
    }

    fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance_handle(index)
    }

    fn internal_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        NestedArtboard::internal_data_context(self, Some(data_context));
    }

    fn bind_view_model_instance(
        &mut self,
        view_model_instance: CoreHandle,
        parent: RuntimeDataContextHandle,
    ) {
        NestedArtboard::bind_view_model_instance(self, Some(view_model_instance), Some(parent));
    }

    fn clear_data_context(&mut self) {
        NestedArtboard::clear_data_context(self);
    }

    fn unbind(&mut self) {
        NestedArtboard::unbind(self);
    }

    fn update_data_binds(&mut self) {
        NestedArtboard::update_data_binds(self);
    }

    fn mark_hosting_layout_dirty(&mut self, _artboard: RuntimeArtboardInstanceWeakHandle) {
        // The base NestedArtboard is not a layout provider. The pinned virtual
        // is intentionally empty here; NestedArtboardLayout supplies the
        // concrete layout-host behavior.
    }

    fn parent_artboard(&self) -> Option<CoreHandle> {
        self.parent_artboard_handle()
    }

    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> bool {
        NestedArtboard::hit_test_host(self, position, skip_on_unclipped, artboard)
    }

    fn host_transform_point(
        &self,
        position: &Vec2D,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Vec2D {
        NestedArtboard::host_transform_point(self, position, artboard)
    }

    fn world_transform_for_artboard(&self, artboard: RuntimeArtboardInstanceWeakHandle) -> Mat2D {
        NestedArtboard::world_transform_for_artboard(self, artboard)
    }

    fn mark_host_transform_dirty(&mut self) {
        NestedArtboard::mark_host_transform_dirty(self);
    }

    fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        NestedArtboard::set_file(self, value.unwrap_or_default());
    }

    fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.upgrade().map(|_| self.file.clone())
    }

    fn host_component(&self) -> Option<CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
    }

    fn relink_data_context(&mut self, view_model_instance: Option<CoreHandle>) {
        NestedArtboard::relink_data_context(self, view_model_instance);
    }

    fn type_(&self) -> i32 {
        NestedArtboard::type_(self)
    }
}

impl ArtboardReferencerBehavior for NestedArtboard {
    fn artboard_referencer(&self) -> &ArtboardReferencer {
        &self.artboard_referencer
    }

    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer {
        &mut self.artboard_referencer
    }

    fn update_artboard(&mut self, view_model_instance_artboard: Option<CoreHandle>) {
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
    fn focusable_artboard(&self) -> Option<CoreHandle> {
        NestedArtboard::focusable_artboard(self)
    }

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

    fn gamepad_dispatch(
        &mut self,
        _invocation: &crate::mechanical_port::source::animation::listener_invocation::ListenerInvocation,
        _out_dispatched_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
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

    fn copy_data_bind_path_ids(&mut self, object: &NestedArtboard) {
        NestedArtboard::copy_data_bind_path_ids(self, object);
    }
}

fn make_translate(artboard: &Artboard) -> Mat2D {
    Mat2D::from_translate(
        -artboard.base.origin_x() * artboard.base.width(),
        -artboard.base.origin_y() * artboard.base.height(),
    )
}

impl std::ops::Deref for NestedArtboard {
    type Target = NestedArtboardBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedArtboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
