use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::{
        keyframe_interpolator::KeyFrameInterpolator, state_machine::StateMachineInstance,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{
        Artboard, ArtboardInstance, RuntimeArtboardInstanceHandle,
        RuntimeArtboardInstanceWeakHandle,
    },
    artboard_host::ArtboardHost,
    artboard_list_map_rule::ArtboardListMapRule,
    component::Component,
    component_dirt::ComponentDirt,
    constraints::{
        constrainable_list::{ConstrainableList, ConstrainableListState},
        scrolling::scroll_constraint::ScrollConstraint,
    },
    core::CoreHandle,
    data_bind::{
        data_bind_list_item_consumer::DataBindListItemConsumer,
        data_context::{DataContext, RuntimeDataContextHandle},
    },
    dirtyable::Dirtyable,
    file::{File, RuntimeFileWeakHandle},
    generated::artboard_component_list_base::ArtboardComponentListBaseCallbacks,
    hit_info::HitInfo,
    input::{
        focus_manager::RuntimeFocusManagerHandle,
        focus_node::{FocusNode, FocusNodeRef},
    },
    layout::{
        artboard_component_list_override::ArtboardComponentListOverride,
        layout_enums::{LayoutDirection, LayoutStyleInterpolation},
        layout_node_provider::{LayoutNodeProvider, LayoutNodeProviderState},
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    property_recorder::PropertyRecorder,
    renderer::Renderer,
    resetting_component::ResettingComponent,
    semantic::semantic_data::SemanticData,
    transform_component::TransformComponent,
    viewmodel::{
        symbol_type::SymbolType,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_value::{ValueDependentHandle, ViewModelInstanceValue},
        viewmodel_value_dependent::ViewModelValueDependent,
    },
    virtualizing_component::VirtualizingComponent,
};

pub use crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase;

pub struct ArtboardListDrawIndexDependent {
    list: CoreHandle,
    value: CoreHandle,
    identity: Option<ValueDependentHandle>,
}

impl ArtboardListDrawIndexDependent {
    pub fn new(list: CoreHandle, value: CoreHandle) -> Rc<RefCell<dyn ViewModelValueDependent>> {
        let dependent = Rc::new(RefCell::new(Self {
            list,
            value: value.clone(),
            identity: None,
        }));
        let erased: Rc<RefCell<dyn ViewModelValueDependent>> = dependent.clone();
        let identity = ValueDependentHandle::runtime(&erased);
        dependent.borrow_mut().identity = Some(identity.clone());
        value.with_downcast_mut::<ViewModelInstanceValue, _>(|value| {
            value.add_dependent(identity);
        });
        erased
    }

    pub fn clear(&mut self) {
        if let Some(identity) = self.identity.take() {
            self.value
                .with_downcast_mut::<ViewModelInstanceValue, _>(|value| {
                    value.remove_dependent(&identity);
                });
        }
    }
}

impl Drop for ArtboardListDrawIndexDependent {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Dirtyable for ArtboardListDrawIndexDependent {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.list
            .with_downcast_mut::<ArtboardComponentList, _>(|list| {
                list.invalidate_ordered_list_indices_cache();
                list.component_mut()
                    .add_dirt(ComponentDirt::COMPONENTS, false);
            });
    }
}

impl ViewModelValueDependent for ArtboardListDrawIndexDependent {
    fn relink_data_bind(&mut self) {}
}

pub struct ArtboardComponentList {
    pub base: ArtboardComponentListBase,
    list_items: Vec<CoreHandle>,
    old_items: Vec<CoreHandle>,
    artboards_map: HashMap<u32, CoreHandle>,
    artboard_instances_map: HashMap<CoreHandle, RuntimeArtboardInstanceHandle>,
    state_machines_map: HashMap<CoreHandle, RuntimeStateMachineInstanceHandle>,
    resource_pool: HashMap<CoreHandle, Vec<RuntimeArtboardInstanceHandle>>,
    state_machines_pool: HashMap<CoreHandle, Vec<RuntimeStateMachineInstanceHandle>>,
    property_recorders_map: HashMap<CoreHandle, Box<PropertyRecorder>>,
    artboard_transforms: HashMap<CoreHandle, Mat2D>,
    artboard_instances_by_index: Vec<Option<RuntimeArtboardInstanceHandle>>,
    state_machines_by_index: Vec<Option<RuntimeStateMachineInstanceHandle>>,
    file: Option<RuntimeFileWeakHandle>,
    artboard_sizes: Vec<Vec2D>,
    layout_size: Vec2D,
    visible_start_index: i32,
    visible_end_index: i32,
    artboard_map_rules: HashMap<i32, i32>,
    list_scope_focus_node: Option<FocusNodeRef>,
    list_row_focus_nodes: Vec<Option<FocusNodeRef>>,
    should_reset_instances: bool,
    list_uses_draw_index_sort: bool,
    ordered_list_indices_cache_valid: bool,
    cached_ordered_list_indices: Vec<i32>,
    draw_index_dependents: HashMap<CoreHandle, Rc<RefCell<dyn ViewModelValueDependent>>>,
    provider_state: LayoutNodeProviderState,
    constrainable_list_state: ConstrainableListState,
}

impl Default for ArtboardComponentList {
    fn default() -> Self {
        Self {
            base: ArtboardComponentListBase::default(),
            list_items: Vec::new(),
            old_items: Vec::new(),
            artboards_map: HashMap::new(),
            artboard_instances_map: HashMap::new(),
            state_machines_map: HashMap::new(),
            resource_pool: HashMap::new(),
            state_machines_pool: HashMap::new(),
            property_recorders_map: HashMap::new(),
            artboard_transforms: HashMap::new(),
            artboard_instances_by_index: Vec::new(),
            state_machines_by_index: Vec::new(),
            file: None,
            artboard_sizes: Vec::new(),
            layout_size: Vec2D::default(),
            visible_start_index: -1,
            visible_end_index: -1,
            artboard_map_rules: HashMap::new(),
            list_scope_focus_node: None,
            list_row_focus_nodes: Vec::new(),
            should_reset_instances: false,
            list_uses_draw_index_sort: false,
            ordered_list_indices_cache_valid: false,
            cached_ordered_list_indices: Vec::new(),
            draw_index_dependents: HashMap::new(),
            provider_state: LayoutNodeProviderState::default(),
            constrainable_list_state: ConstrainableListState::default(),
        }
    }
}

impl Drop for ArtboardComponentList {
    fn drop(&mut self) {
        self.clear();
    }
}

impl ArtboardComponentList {
    pub const TYPE_KEY: u16 = ArtboardComponentListBase::TYPE_KEY;

    fn drawable(&self) -> &crate::mechanical_port::source::drawable::Drawable {
        &self.base.base
    }

    fn drawable_mut(&mut self) -> &mut crate::mechanical_port::source::drawable::Drawable {
        &mut self.base.base
    }

    fn transform(&self) -> &TransformComponent {
        &self.base.base.base.base.base.base
    }

    fn transform_mut(&mut self) -> &mut TransformComponent {
        &mut self.base.base.base.base.base.base
    }

    fn container(
        &self,
    ) -> &crate::mechanical_port::source::container_component::ContainerComponent {
        &self.transform().base.base.base.base
    }

    fn container_mut(
        &mut self,
    ) -> &mut crate::mechanical_port::source::container_component::ContainerComponent {
        &mut self.transform_mut().base.base.base.base
    }

    fn component(&self) -> &Component {
        &self.transform().base.base.base.base.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.transform_mut().base.base.base.base.base.base
    }

    pub fn core_type(&self) -> u16 {
        self.base.core_type()
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.transform_mut().collapse(value) {
            return false;
        }
        self.collapse_after_super(value);
        true
    }

    pub(crate) fn collapse_after_super(&mut self, value: bool) {
        for index in 0..self.artboard_count() {
            if let Some(nested_artboard) = self.artboard_instance(index as i32) {
                nested_artboard
                    .with_artboard_mut(|artboard| artboard.collapse_semantic_boundary(value));
            }
        }
    }

    pub fn clear(&mut self) {
        for artboard in self.artboard_instances_map.values() {
            artboard.with_artboard_mut(|artboard| artboard.cleanup_semantic_tree());
        }
        self.clear_draw_index_listeners();
        self.invalidate_ordered_list_indices_cache();
        for artboard in self.artboard_instances_map.values() {
            artboard.with_artboard_mut(|artboard| artboard.cleanup_focus_tree());
        }
        self.remove_list_scope_focus_node();
        self.list_row_focus_nodes.clear();
        self.state_machines_map.clear();
        self.artboard_instances_by_index.clear();
        self.state_machines_by_index.clear();
        self.artboard_instances_map.clear();
        self.list_items.clear();
        self.artboards_map.clear();
        self.resource_pool.clear();
        self.state_machines_pool.clear();
    }

    pub fn artboard_count(&self) -> usize {
        self.list_items.len()
    }

    pub fn list_item(&self, index: i32) -> Option<CoreHandle> {
        if index >= 0 && (index as usize) < self.list_items.len() {
            return Some(self.list_items[index as usize].clone());
        }
        None
    }

    pub fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        if !self.virtualization_enabled() {
            return (index >= 0)
                .then(|| self.artboard_instances_by_index.get(index as usize))
                .flatten()
                .cloned()
                .flatten();
        }
        if index >= 0 && (index as usize) < self.list_items.len() {
            let item = self.list_items[index as usize].clone();
            return self.artboard_instances_map.get(&item).cloned();
        }
        None
    }

    pub fn index_of_artboard_instance(&self, instance: &RuntimeArtboardInstanceHandle) -> i32 {
        for (index, item) in self.list_items.iter().enumerate() {
            if self
                .artboard_instances_map
                .get(item)
                .is_some_and(|artboard| artboard.downgrade().ptr_eq(&instance.downgrade()))
            {
                return index as i32;
            }
        }
        -1
    }

    pub fn state_machine_instance(&self, index: i32) -> Option<RuntimeStateMachineInstanceHandle> {
        if !self.virtualization_enabled() {
            return (index >= 0)
                .then(|| self.state_machines_by_index.get(index as usize))
                .flatten()
                .cloned()
                .flatten();
        }
        if index >= 0 && (index as usize) < self.list_items.len() {
            let item = self.list_items[index as usize].clone();
            return self.state_machines_map.get(&item).cloned();
        }
        None
    }

    pub fn mark_layout_node_dirty(&mut self, _should_force_update_layout_bounds: bool) {
        let parent_is_row = self.main_axis_is_row();
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.with_artboard_mut(|artboard| artboard.parent_is_row(parent_is_row));
            }
        }
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                let bounds = artboard.with_artboard_mut(|artboard| {
                    artboard.update_layout_bounds(animate);
                    artboard.layout_bounds()
                });
                self.set_item_size(Vec2D::new(bounds.width(), bounds.height()), index);
            }
        }
        self.compute_layout_bounds();
    }

    pub fn cascade_layout_style(
        &mut self,
        inherited_interpolation: LayoutStyleInterpolation,
        inherited_interpolator: Option<CoreHandle>,
        inherited_interpolation_time: f32,
        direction: LayoutDirection,
    ) -> bool {
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                if let Some(interpolator) = inherited_interpolator.as_ref() {
                    interpolator.with_downcast_mut::<KeyFrameInterpolator, _>(|interpolator| {
                        artboard.with_artboard_mut(|artboard| {
                            artboard.cascade_layout_style(
                                inherited_interpolation,
                                Some(interpolator),
                                inherited_interpolation_time,
                                direction,
                            );
                        });
                    });
                } else {
                    artboard.with_artboard_mut(|artboard| {
                        artboard.cascade_layout_style(
                            inherited_interpolation,
                            None,
                            inherited_interpolation_time,
                            direction,
                        );
                    });
                }
            }
        }
        false
    }

    pub fn sync_style_changes(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                if artboard.with_artboard_mut(Artboard::sync_style_changes) {
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn find_artboard(&mut self, list_item: &CoreHandle) -> Option<CoreHandle> {
        let view_model_id = list_item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten()?
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::view_model_id)?;
        if let Some(artboard) = self.artboards_map.get(&view_model_id) {
            return Some(artboard.clone());
        }
        let artboards = self.file.as_ref()?.with_file(File::artboards)?;
        if let Some(artboard_index) = self.artboard_map_rules.get(&(view_model_id as i32)) {
            if *artboard_index >= 0 && (*artboard_index as usize) < artboards.len() {
                let artboard = artboards[*artboard_index as usize].clone();
                self.artboards_map.insert(view_model_id, artboard.clone());
                return Some(artboard);
            }
        }
        for artboard in artboards {
            if artboard
                .with_downcast::<Artboard, _>(Artboard::view_model_id)
                .is_some_and(|id| id == view_model_id)
            {
                self.artboards_map.insert(view_model_id, artboard.clone());
                return Some(artboard);
            }
        }
        None
    }

    fn dispose_list_item(&mut self, list_item: &CoreHandle) {
        self.remove_artboard(list_item.clone());
    }

    fn create_artboard(&mut self, list_item: CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
        self.find_artboard(&list_item)?
            .with_downcast::<Artboard, _>(Artboard::instance_handle)
            .flatten()
    }

    fn create_state_machine_instance(
        &mut self,
        artboard: &RuntimeArtboardInstanceHandle,
    ) -> Option<RuntimeStateMachineInstanceHandle> {
        let instance =
            artboard.with_artboard_mut(ArtboardInstance::default_state_machine_handle)?;
        self.link_state_machine_to_artboard(&instance, artboard);
        Some(instance)
    }

    pub fn ensure_list_scope_focus_node(
        &mut self,
        focus_manager: RuntimeFocusManagerHandle,
        host_parent: Option<FocusNodeRef>,
    ) {
        if self.list_scope_focus_node.is_none() {
            let node = FocusNode::make_structural_scope();
            node.borrow_mut().name = "ArtboardComponentListScope".to_owned();
            self.list_scope_focus_node = Some(node);
        }
        focus_manager.with_focus_manager_mut(|manager| {
            manager.add_child(
                host_parent,
                self.list_scope_focus_node.clone().unwrap(),
                None,
            );
        });
        self.sync_list_row_nodes_with_list(focus_manager);
    }

    pub fn list_scope_focus_node(&self) -> Option<FocusNodeRef> {
        self.list_scope_focus_node.clone()
    }

    pub fn remove_list_scope_focus_node(&mut self) {
        let rows = std::mem::take(&mut self.list_row_focus_nodes);
        let scope = self.list_scope_focus_node.take();
        let focus_manager = self
            .component()
            .with_artboard(Artboard::focus_manager_handle)
            .flatten();
        for row in rows {
            let Some(node) = row else {
                continue;
            };
            if let Some(parent) = node.borrow().parent() {
                FocusNode::remove_child(&parent, &node);
            } else if let Some(focus_manager) = focus_manager.as_ref() {
                focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
            }
        }
        let Some(node) = scope else {
            return;
        };
        if let Some(parent) = node.borrow().parent() {
            FocusNode::remove_child(&parent, &node);
        } else if let Some(focus_manager) = focus_manager {
            focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
        }
    }

    fn make_list_row_focus_node(&self) -> FocusNodeRef {
        let node = FocusNode::make_structural_scope();
        node.borrow_mut().name = "ArtboardComponentListRow".to_owned();
        node
    }

    fn reparent_list_rows_in_scope(&mut self, focus_manager: &RuntimeFocusManagerHandle) {
        let Some(scope) = self.list_scope_focus_node.clone() else {
            return;
        };
        for row in self.list_row_focus_nodes.iter().flatten() {
            if let Some(parent) = row.borrow().parent() {
                FocusNode::remove_child(&parent, row);
            }
        }
        focus_manager.with_focus_manager_mut(|manager| {
            for (index, row) in self.list_row_focus_nodes.iter().enumerate() {
                if let Some(row) = row {
                    manager.add_child(Some(scope.clone()), row.clone(), Some(index));
                }
            }
        });
    }
}

fn artboard_has_focus_content(artboard: &RuntimeArtboardInstanceHandle) -> bool {
    if artboard.with_artboard(Artboard::root_focus_data_count) > 0 {
        return true;
    }
    let nested = artboard.with_artboard(Artboard::nested_artboards);
    for host in nested {
        let is_data_bound = host
            .with(|host| host.nested_artboard_is_data_bound())
            .unwrap_or(false);
        if is_data_bound {
            return true;
        }
        let instance = host
            .with(|host| host.nested_artboard_instance_handle())
            .flatten();
        if instance.as_ref().is_some_and(artboard_has_focus_content) {
            return true;
        }
    }
    artboard
        .with_artboard(Artboard::artboard_component_lists)
        .into_iter()
        .any(|list| list.is_alive())
}

impl ArtboardComponentList {
    fn list_item_needs_build_under_row(
        &self,
        parent_focus_manager: &RuntimeFocusManagerHandle,
        instance: &RuntimeArtboardInstanceHandle,
        row: Option<FocusNodeRef>,
    ) -> bool {
        let Some(row) = row else {
            return false;
        };
        if instance.with_artboard(|instance| {
            instance
                .focus_manager()
                .is_none_or(|manager| !manager.ptr_eq(parent_focus_manager))
        }) {
            return true;
        }
        if row.borrow().children().is_empty() && artboard_has_focus_content(instance) {
            return true;
        }
        false
    }

    fn sync_list_row_nodes_with_list(&mut self, focus_manager: RuntimeFocusManagerHandle) {
        if self.list_items.is_empty() {
            while let Some(row) = self.list_row_focus_nodes.pop() {
                if let Some(row) = row {
                    focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&row));
                }
            }
            return;
        }
        let list_copy = self.list_items.clone();
        let row_copy = self.list_row_focus_nodes.clone();
        self.sync_list_row_nodes_with_previous(focus_manager, &list_copy, &row_copy);
    }

    fn sync_list_row_nodes_with_previous(
        &mut self,
        focus_manager: RuntimeFocusManagerHandle,
        previous_list_items: &[CoreHandle],
        previous_row_nodes: &[Option<FocusNodeRef>],
    ) {
        if self.list_scope_focus_node.is_none() {
            return;
        }
        let count = self.list_items.len();
        if count == 0 {
            self.list_row_focus_nodes.clear();
            return;
        }
        let mut new_rows = vec![None; count];
        let mut previous_rows = previous_row_nodes.to_vec();
        for index in 0..count {
            for previous_index in 0..previous_list_items.len().min(previous_rows.len()) {
                if self.list_items[index] == previous_list_items[previous_index] {
                    new_rows[index] = previous_rows[previous_index].take();
                    break;
                }
            }
        }
        for previous_index in 0..previous_rows.len().min(previous_list_items.len()) {
            let Some(unmapped) = previous_rows[previous_index].take() else {
                continue;
            };
            let mut in_new = false;
            for index in 0..count {
                if self.list_items[index] == previous_list_items[previous_index] {
                    in_new = true;
                    break;
                }
            }
            if !in_new {
                focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&unmapped));
            }
        }
        self.list_row_focus_nodes = new_rows;
        for index in 0..count {
            if self.list_row_focus_nodes[index].is_none() {
                self.list_row_focus_nodes[index] = Some(self.make_list_row_focus_node());
            }
        }
        self.reparent_list_rows_in_scope(&focus_manager);
        for index in 0..count {
            let Some(instance) = self.artboard_instance(index as i32) else {
                continue;
            };
            let row = self.list_row_focus_nodes[index].clone();
            if row.is_none() {
                continue;
            }
            if let Some(state_machine) = self.state_machine_instance(index as i32) {
                state_machine.with_instance_mut(|state_machine| {
                    let needs_parent = !state_machine.focus_manager().ptr_eq(&focus_manager);
                    if needs_parent {
                        state_machine.set_external_focus_manager(Some(focus_manager.clone()));
                    }
                });
            }
            if self.list_item_needs_build_under_row(&focus_manager, &instance, row.clone()) {
                instance.with_artboard_mut(|instance| {
                    if instance.focus_manager().is_some() {
                        instance.cleanup_focus_tree();
                    }
                    instance.build_focus_tree(Some(focus_manager.clone()), row);
                });
            }
        }
    }

    fn link_state_machine_to_artboard(
        &mut self,
        state_machine_instance: &RuntimeStateMachineInstanceHandle,
        artboard_instance: &RuntimeArtboardInstanceHandle,
    ) {
        let data_context = artboard_instance.with_artboard(|artboard| artboard.base.data_context());
        state_machine_instance.with_instance_mut(|state_machine_instance| {
            if let Some(data_context) = data_context {
                state_machine_instance.set_data_context_handle(data_context);
                state_machine_instance.update_data_binds(false);
            }
        });
        let parent_node = SemanticData::find_closest_semantic_node_handle(self.host_component());
        let managers = self.component().with_artboard(|parent_artboard| {
            (
                parent_artboard.focus_manager_handle(),
                parent_artboard.semantic_manager_handle(),
            )
        });
        if let Some((focus_manager, semantic_manager)) = managers {
            state_machine_instance.with_instance_mut(|state_machine_instance| {
                if let Some(focus_manager) = focus_manager {
                    state_machine_instance.set_external_focus_manager_handle(focus_manager);
                }
                if let Some(semantic_manager) = semantic_manager {
                    state_machine_instance
                        .set_external_semantic_manager_handle(semantic_manager, parent_node);
                }
            });
        }
    }

    fn lists_are_equal(list: Option<&[CoreHandle]>, compared: Option<&[CoreHandle]>) -> bool {
        let (Some(list), Some(compared)) = (list, compared) else {
            return false;
        };
        if list.len() != compared.len() {
            return false;
        }
        for (index, item) in list.iter().enumerate() {
            if item != &compared[index] {
                return false;
            }
        }
        true
    }

    pub fn update_list(&mut self, list: &[CoreHandle]) {
        if Self::lists_are_equal(Some(&self.list_items), Some(list)) {
            return;
        }
        let previous_list_items = self.list_items.clone();
        let previous_row_nodes = self.list_row_focus_nodes.clone();
        self.old_items.clear();
        self.old_items.extend(self.list_items.iter().cloned());
        self.list_items.clear();
        self.list_items.extend(list.iter().cloned());
        self.invalidate_ordered_list_indices_cache();
        self.artboard_sizes.clear();
        self.artboard_instances_by_index.clear();
        self.state_machines_by_index.clear();
        if !self.virtualization_enabled() {
            self.artboard_instances_by_index
                .resize(self.list_items.len(), None);
            self.state_machines_by_index
                .resize(self.list_items.len(), None);
        }
        self.layout_parent_mut(LayoutComponent::clear_layout_children);
        for item in self.old_items.clone() {
            if !self.list_items.contains(&item) {
                self.dispose_list_item(&item);
            }
        }
        for index in 0..self.list_items.len() {
            let item = self.list_items[index].clone();
            let view_model_instance = item
                .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
                .flatten();
            if let Some(symbol) = view_model_instance.as_ref().and_then(|instance| {
                instance
                    .with_downcast::<ViewModelInstance, _>(|instance| {
                        instance.property_value_for_symbol(SymbolType::ItemIndex)
                    })
                    .flatten()
            }) {
                symbol.with_downcast_mut::<ViewModelInstanceSymbolListIndex, _>(|symbol| {
                    symbol.set_property_value(index as u32);
                });
            }
            if let Some(artboard) = self.find_artboard(&item) {
                if let Some(size) = artboard.with_downcast::<Artboard, _>(|artboard| {
                    Vec2D::new(artboard.width(), artboard.height())
                }) {
                    self.artboard_sizes.push(size);
                }
            }
            if !self.virtualization_enabled() {
                if !self.artboard_instances_map.contains_key(&item) {
                    self.create_artboard_at(index as i32, false);
                } else {
                    self.artboard_instances_by_index[index] =
                        self.artboard_instances_map.get(&item).cloned();
                    self.state_machines_by_index[index] =
                        self.state_machines_map.get(&item).cloned();
                }
            }
        }
        self.compute_layout_bounds();
        self.sync_layout_children();
        self.mark_layout_node_dirty(false);
        self.transform_mut().mark_world_transform_dirty();
        self.component_mut()
            .add_dirt(ComponentDirt::COMPONENTS, true);
        self.recompute_list_uses_draw_index_sort();
        self.sync_draw_index_listeners();
        let focus_manager = self
            .component()
            .with_artboard(Artboard::focus_manager_handle)
            .flatten();
        if let Some(focus_manager) = focus_manager.filter(|_| self.list_scope_focus_node.is_some())
        {
            self.sync_list_row_nodes_with_previous(
                focus_manager,
                &previous_list_items,
                &previous_row_nodes,
            );
        }
    }

    pub fn sync_layout_children(&mut self) {
        self.layout_parent_mut(LayoutComponent::sync_layout_children);
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if self.artboard_count() == 0 || self.component().is_collapsed() {
            return false;
        }
        let mut keep_going = false;
        let advance_nested = flags.contains(AdvanceFlags::ADVANCE_NESTED);
        let new_frame = flags.contains(AdvanceFlags::NEW_FRAME);
        let advancing_flags = flags & !AdvanceFlags::IS_ROOT;
        for index in 0..self.artboard_count() as i32 {
            if advance_nested {
                if let Some(state_machine) = self.state_machine_instance(index) {
                    if state_machine.with_instance_mut(|state_machine| {
                        if !new_frame {
                            state_machine.try_change_state()
                                && state_machine.advance(elapsed_seconds, new_frame)
                        } else {
                            state_machine.advance(elapsed_seconds, new_frame)
                        }
                    }) {
                        keep_going = true;
                    }
                }
            }
            if let Some(artboard) = self.artboard_instance(index) {
                let (advanced, has_dirt) = artboard.with_artboard_mut(|artboard| {
                    (
                        artboard.advance_internal(elapsed_seconds, advancing_flags),
                        artboard.base.has_dirt(ComponentDirt::COMPONENTS),
                    )
                });
                if advanced {
                    keep_going = true;
                }
                if has_dirt {
                    self.component_mut()
                        .add_dirt(ComponentDirt::COMPONENTS, true);
                }
            }
        }
        keep_going
    }

    pub fn reset(&mut self) {
        for item in self.list_items.clone() {
            if self.should_reset_instances {
                let view_model_instance = item
                    .with_downcast::<ViewModelInstanceListItem, _>(|item| {
                        item.view_model_instance()
                    })
                    .flatten();
                if let Some(view_model_instance) = view_model_instance.as_ref() {
                    view_model_instance
                        .with_downcast_mut::<ViewModelInstance, _>(ViewModelInstance::advanced);
                }
                if let Some(artboard) = self.artboard_instances_map.get(&item) {
                    let bound_instance = artboard.with_artboard(|artboard| {
                        artboard.base.data_context().and_then(|context| {
                            context.with_context(DataContext::main_view_model_instance)
                        })
                    });
                    if let Some(bound_instance) =
                        bound_instance.filter(|bound| Some(bound) != view_model_instance.as_ref())
                    {
                        bound_instance
                            .with_downcast_mut::<ViewModelInstance, _>(ViewModelInstance::advanced);
                    }
                }
            }
            if let Some(artboard) = self.artboard_instances_map.get(&item) {
                artboard.with_artboard_mut(Artboard::reset);
            }
        }
    }

    pub fn layout_bounds(&self) -> Aabb {
        Aabb::new(0.0, 0.0, self.layout_size.x, self.layout_size.y)
    }

    pub fn layout_bounds_for_node(&self, index: usize) -> Aabb {
        if self.virtualization_enabled_ref() {
            let real_index = index % self.list_items.len();
            let gap = self.gap_ref();
            let mut running_size = 0.0;
            let is_horizontal = self.main_axis_is_row_ref();
            for item_index in 0..real_index {
                let size = self.artboard_sizes[item_index];
                running_size += if is_horizontal { size.x } else { size.y } + gap;
            }
            let item_size = self.artboard_sizes[real_index];
            let left = if is_horizontal { running_size } else { 0.0 };
            let top = if is_horizontal { 0.0 } else { running_size };
            return Aabb::new(left, top, left + item_size.x, top + item_size.y);
        }
        if index < self.num_layout_nodes() {
            if let Some(artboard) = self.artboard_instance(index as i32) {
                return artboard.with_artboard(Artboard::layout_bounds);
            }
        }
        Aabb::default()
    }

    pub fn mark_hosting_layout_dirty(&mut self, artboard_instance: &RuntimeArtboardInstanceHandle) {
        for index in 0..self.artboard_count() as i32 {
            let Some(artboard) = self.artboard_instance(index) else {
                continue;
            };
            if artboard.downgrade().ptr_eq(&artboard_instance.downgrade()) {
                self.component().with_artboard_mut(|parent| {
                    artboard_instance.with_artboard_mut(|artboard| {
                        parent.mark_layout_dirty(artboard);
                    });
                });
                break;
            }
        }
        self.transform_mut().mark_world_transform_dirty();
    }

    pub fn will_draw(&self) -> bool {
        self.drawable().will_draw() && !self.list_items.is_empty()
    }

    pub fn invalidate_ordered_list_indices_cache(&mut self) {
        self.ordered_list_indices_cache_valid = false;
    }

    fn recompute_list_uses_draw_index_sort(&mut self) {
        let previous = self.list_uses_draw_index_sort;
        self.list_uses_draw_index_sort = false;
        for item in &self.list_items {
            let Some(instance) = item
                .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
                .flatten()
            else {
                continue;
            };
            let has_draw_index = instance
                .with_downcast::<ViewModelInstance, _>(|instance| instance.get_view_model())
                .flatten()
                .and_then(|view_model| {
                    view_model.with(|view_model| {
                        view_model.as_view_model().and_then(|view_model| {
                            view_model.property_for_symbol(SymbolType::DrawIndex)
                        })
                    })
                })
                .flatten()
                .is_some();
            if has_draw_index {
                self.list_uses_draw_index_sort = true;
                if previous != self.list_uses_draw_index_sort {
                    self.invalidate_ordered_list_indices_cache();
                }
                return;
            }
        }
        if previous != self.list_uses_draw_index_sort {
            self.invalidate_ordered_list_indices_cache();
        }
    }

    fn list_item_draw_index(&self, index: i32) -> f32 {
        if index < 0 || index as usize >= self.list_items.len() {
            return 0.0;
        }
        let Some(instance) = self.list_items[index as usize]
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten()
        else {
            return 0.0;
        };
        let has_draw_index = instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.get_view_model())
            .flatten()
            .and_then(|view_model| {
                view_model.with(|view_model| {
                    view_model.as_view_model().and_then(|view_model| {
                        view_model.property_for_symbol(SymbolType::DrawIndex)
                    })
                })
            })
            .flatten()
            .is_some();
        if !has_draw_index {
            return 0.0;
        }
        let value = instance
            .with_downcast::<ViewModelInstance, _>(|instance| {
                instance.property_value_for_symbol(SymbolType::DrawIndex)
            })
            .flatten();
        if let Some(value) = value
            .and_then(|value| {
                value.with_downcast::<ViewModelInstanceNumber, _>(|number| number.property_value())
            })
            .filter(|value| value.is_finite())
        {
            return value;
        }
        0.0
    }

    fn clear_draw_index_listeners(&mut self) {
        self.draw_index_dependents.clear();
    }

    fn remove_draw_index_listener_for_item(&mut self, item: &CoreHandle) {
        self.draw_index_dependents.remove(item);
    }

    fn sync_draw_index_listeners(&mut self) {
        self.clear_draw_index_listeners();
        if !self.list_uses_draw_index_sort {
            return;
        }
        let Some(list) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        for item in self.list_items.clone() {
            let Some(instance) = item
                .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
                .flatten()
            else {
                continue;
            };
            let Some(value) = instance
                .with_downcast::<ViewModelInstance, _>(|instance| {
                    instance.property_value_for_symbol(SymbolType::DrawIndex)
                })
                .flatten()
            else {
                continue;
            };
            self.draw_index_dependents.insert(
                item,
                ArtboardListDrawIndexDependent::new(list.clone(), value),
            );
        }
    }

    pub fn ensure_ordered_list_indices(&mut self) {
        let count = self.list_items.len() as i32;
        if count == 0 {
            self.ordered_list_indices_cache_valid = false;
            self.cached_ordered_list_indices.clear();
            return;
        }
        if self.ordered_list_indices_cache_valid {
            return;
        }
        self.cached_ordered_list_indices.clear();
        let use_virtual_window = self.virtualization_enabled()
            && self.visible_start_index >= 0
            && self.visible_end_index >= 0;
        if use_virtual_window {
            let start_index = self.visible_start_index % count;
            let end_index = self.visible_end_index % count;
            let mut index = start_index;
            loop {
                self.cached_ordered_list_indices.push(index);
                if index == end_index {
                    break;
                }
                index = (index + 1) % count;
            }
        } else {
            self.cached_ordered_list_indices.reserve(count as usize);
            for index in 0..count {
                self.cached_ordered_list_indices.push(index);
            }
        }
        if self.list_uses_draw_index_sort {
            let mut indices = std::mem::take(&mut self.cached_ordered_list_indices);
            indices.sort_by(|left, right| {
                let left_value = self.list_item_draw_index(*left);
                let right_value = self.list_item_draw_index(*right);
                left_value
                    .partial_cmp(&right_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| left.cmp(right))
            });
            self.cached_ordered_list_indices = indices;
        }
        self.ordered_list_indices_cache_valid = true;
    }

    pub fn ordered_list_indices(&mut self) -> &[i32] {
        self.ensure_ordered_list_indices();
        &self.cached_ordered_list_indices
    }
}

impl ArtboardComponentList {
    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.drawable().needs_save_operation() {
            renderer.save();
        }
        if self.virtualization_enabled() {
            self.layout_parent_ref(|parent| {
                renderer.transform(nuxie_render_api::Mat2D(*parent.world_transform().values()));
            });
            if self.visible_start_index != -1 && self.visible_end_index != -1 {
                let indices = self.ordered_list_indices().to_vec();
                for index in indices {
                    if let (Some(artboard), Some(item)) =
                        (self.artboard_instance(index), self.list_item(index))
                    {
                        renderer.save();
                        let transform = self.artboard_transforms[&item];
                        renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
                        artboard.with_artboard_mut(|artboard| artboard.draw_internal(renderer));
                        renderer.restore();
                    }
                }
            }
        } else {
            let transform = *self.transform().world_transform();
            renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
            let indices = self.ordered_list_indices().to_vec();
            for index in indices {
                if let (Some(artboard), Some(item)) =
                    (self.artboard_instance(index), self.list_item(index))
                {
                    renderer.save();
                    let transform = self.artboard_transforms[&item];
                    renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
                    artboard.with_artboard_mut(|artboard| artboard.draw_internal(renderer));
                    renderer.restore();
                }
            }
        }
        if self.drawable().needs_save_operation() {
            renderer.restore();
        }
    }

    pub fn hit_test(&mut self, _hit_info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }

    pub fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: &RuntimeArtboardInstanceHandle,
    ) -> bool {
        let bounds = self.artboard_position(artboard);
        let offset = Vec2D::new(bounds.x + position.x, bounds.y + position.y);
        let transform = if self.virtualization_enabled() {
            self.layout_parent_ref(|parent| *parent.world_transform())
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        self.component()
            .with_parent(|parent| {
                parent.hit_test_point(&(transform * offset), skip_on_unclipped, false)
            })
            .unwrap_or(false)
    }

    pub fn host_transform_point(
        &self,
        vector: &Vec2D,
        artboard_instance: &RuntimeArtboardInstanceHandle,
    ) -> Vec2D {
        let bounds = self.artboard_transform(artboard_instance);
        let offset = Vec2D::new(bounds[4] + vector.x, bounds[5] + vector.y);
        let transform = if self.virtualization_enabled_ref() {
            self.component()
                .parent_handle()
                .and_then(|parent| {
                    parent.with_downcast::<LayoutComponent, _>(|parent| *parent.world_transform())
                })
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        let local = transform * offset;
        self.component()
            .with_artboard(|artboard| artboard.root_transform(local))
            .unwrap_or(local)
    }

    pub fn world_transform_for_artboard(
        &self,
        artboard_instance: &RuntimeArtboardInstanceHandle,
    ) -> Mat2D {
        let offset = self.artboard_transform(artboard_instance);
        let position = Vec2D::new(offset[4], offset[5]);
        let parent_layout = self
            .component()
            .parent_handle()
            .filter(|parent| parent.with_downcast::<LayoutComponent, _>(|_| ()).is_some());
        if let Some(parent_layout) = parent_layout {
            let bounds = self.layout_bounds();
            if let Some(transform) = parent_layout.with_downcast::<LayoutComponent, _>(|parent| {
                *parent.world_transform() * Mat2D::from_translate(bounds.left(), bounds.top())
            }) {
                return transform * Mat2D::from_translate(position.x, position.y);
            }
        }
        let transform = if self.virtualization_enabled_ref() {
            self.component()
                .parent_handle()
                .and_then(|parent| {
                    parent.with_downcast::<LayoutComponent, _>(|parent| *parent.world_transform())
                })
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        transform * Mat2D::from_translate(position.x, position.y)
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.transform_mut().update(value);
        if self.artboard_count() == 0 {
            return;
        }
        if Component::has_dirt_in(value, ComponentDirt::WORLD_TRANSFORM) {
            for index in 0..self.artboard_count() as i32 {
                if let Some(artboard) = self.artboard_instance(index) {
                    artboard.with_artboard_mut(Artboard::mark_semantic_boundary_transform_dirty);
                }
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::RENDER_OPACITY) {
            let opacity = self.transform().render_opacity();
            for index in 0..self.artboard_count() as i32 {
                if let Some(artboard) = self.artboard_instance(index) {
                    artboard.with_artboard_mut(|artboard| artboard.opacity(opacity));
                }
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::COMPONENTS) {
            for index in 0..self.artboard_count() as i32 {
                if let Some(artboard) = self.artboard_instance(index) {
                    artboard.with_artboard_mut(|artboard| artboard.update_pass(false));
                }
            }
        }
    }

    pub fn update_world_transform(&mut self) {
        self.update_artboards_world_transform();
        self.transform_mut().update_world_transform();
    }

    fn update_artboards_world_transform(&mut self) {
        let count = self.list_items.len();
        if count == 0 {
            return;
        }
        if !self.virtualization_enabled() {
            let use_layout = self.layout_parent_handle().is_some();
            for index in 0..count {
                if let (Some(artboard), Some(item)) = (
                    self.artboard_instance(index as i32),
                    self.list_item(index as i32),
                ) {
                    let (bounds, origin) = artboard.with_artboard(|artboard| {
                        (
                            if use_layout {
                                artboard.layout_bounds()
                            } else {
                                artboard.world_bounds()
                            },
                            if use_layout {
                                artboard.origin()
                            } else {
                                Vec2D::default()
                            },
                        )
                    });
                    self.artboard_transforms.insert(
                        item,
                        Mat2D::from_translate(bounds.left() - origin.x, bounds.top() - origin.y),
                    );
                }
            }
        }
    }

    pub fn update_constraints(&mut self) {
        let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle()
        else {
            return;
        };
        let layout_constraints = self.provider_state.layout_constraints().to_vec();
        for parent_constraint in layout_constraints {
            parent_constraint.with_mut(|constraint| {
                constraint.layout_constraint_constrain_child(owner.clone());
            });
        }
        if !self.constrainable_list_state.list_constraints.is_empty()
            && !self.virtualization_enabled()
        {
            let list_constraints = self.constrainable_list_state.list_constraints.clone();
            for list_constraint in list_constraints {
                list_constraint.with_mut(|constraint| {
                    constraint.list_constraint_constrain_list(owner.clone());
                });
            }
        }
        let constraints = self.transform().constraints().to_vec();
        for constraint in constraints {
            if constraint.with(|constraint| constraint.as_list_constraint().is_some()) == Some(true)
            {
                continue;
            }
            constraint.with_mut(|constraint| {
                constraint.constraint_apply(owner.clone());
            });
        }
    }

    pub fn internal_data_context(&mut self, value: RuntimeDataContextHandle) {
        for artboard in self.artboard_instances_map.values() {
            artboard.with_artboard_mut(|artboard| {
                if let Some(data_context) = artboard.base.data_context() {
                    data_context.with_context_mut(|context| {
                        context.set_parent(Some(value.clone()));
                    });
                    artboard.internal_data_context(data_context);
                }
            });
        }
        for state_machine in self.state_machines_map.values() {
            state_machine.with_instance_mut(|state_machine| {
                if let Some(data_context) = state_machine.data_context_handle() {
                    data_context.with_context_mut(|context| {
                        context.set_parent(Some(value.clone()));
                    });
                    state_machine.internal_data_context_handle(data_context);
                }
            });
        }
    }

    pub fn bind_view_model_instance(
        &mut self,
        _view_model_instance: CoreHandle,
        _parent: RuntimeDataContextHandle,
    ) {
    }

    pub fn clear_data_context(&mut self) {}

    pub fn unbind(&mut self) {
        self.clear();
    }

    pub fn update_data_binds(&mut self) {
        for index in 0..self.artboard_count() as i32 {
            if let Some(state_machine) = self.state_machine_instance(index) {
                state_machine.with_instance_mut(|state_machine| {
                    state_machine.update_data_binds(false);
                });
            }
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.with_artboard_mut(Artboard::update_data_binds_default);
            }
        }
    }

    fn artboard_transform(&self, artboard: &RuntimeArtboardInstanceHandle) -> Mat2D {
        let index = self.index_of_artboard_instance(artboard);
        self.list_item(index)
            .and_then(|item| self.artboard_transforms.get(&item).copied())
            .unwrap_or_else(Mat2D::identity)
    }

    fn artboard_position(&self, artboard: &RuntimeArtboardInstanceHandle) -> Vec2D {
        let matrix = self.artboard_transform(artboard);
        Vec2D::new(matrix[4], matrix[5])
    }

    pub fn world_to_local(&mut self, world: Vec2D, local: &mut Vec2D, index: i32) -> bool {
        let Some(artboard) = self.artboard_instance(index) else {
            return false;
        };
        let offset = self.artboard_position(&artboard);
        let transform = if self.virtualization_enabled() {
            self.layout_parent_ref(|parent| *parent.world_transform())
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        let artboard_transform = transform * Mat2D::from_translate(offset.x, offset.y);
        let mut to_mounted_artboard = Mat2D::identity();
        if !artboard_transform.invert(&mut to_mounted_artboard) {
            return false;
        }
        *local = to_mounted_artboard * world;
        true
    }

    pub fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        self.file = value;
    }

    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }

    pub fn clone_core(&self) -> Self {
        let mut cloned = self.base.clone_into(&mut ArtboardComponentList::default());
        cloned.set_file(self.file());
        cloned
    }

    pub fn create_artboard_at(&mut self, index: i32, force_layout_sync: bool) {
        if let Some(item) = self.list_item(index) {
            if let Some(artboard) = self.create_artboard(item.clone()) {
                self.attach_artboard_override(&artboard, item);
                self.add_artboard_at(artboard, index, force_layout_sync);
            }
        }
    }

    pub fn add_artboard_at(
        &mut self,
        artboard: RuntimeArtboardInstanceHandle,
        index: i32,
        force_layout_sync: bool,
    ) {
        let Some(item) = self.list_item(index) else {
            return;
        };
        self.artboard_instances_map
            .insert(item.clone(), artboard.clone());
        self.bind_artboard(&artboard, item.clone());
        let parent_is_row = self.main_axis_is_row();
        let host = crate::mechanical_port::source::core::CoreObject::core(self).handle();
        artboard.with_artboard_mut(|artboard| {
            artboard.set_host_handle(host);
            artboard.frame_origin(false);
            artboard.parent_is_row(parent_is_row);
        });
        if force_layout_sync {
            self.sync_layout_children();
        }
        let mut state_machine_instance = None;
        let source_artboard = self.find_artboard(&item);
        if let Some(source_artboard) = source_artboard.as_ref() {
            let pool = self
                .state_machines_pool
                .entry(source_artboard.clone())
                .or_default();
            if let Some(state_machine) = pool.pop() {
                state_machine.with_instance_mut(StateMachineInstance::reset_state);
                self.apply_recorders_to_state_machine(&state_machine, source_artboard);
                state_machine_instance = Some(state_machine.clone());
                self.state_machines_map
                    .insert(item.clone(), state_machine.clone());
                self.link_state_machine_to_artboard(&state_machine, &artboard);
            }
        }
        if state_machine_instance.is_none() {
            let state_machine = self.create_state_machine_instance(&artboard);
            state_machine_instance.clone_from(&state_machine);
            if let Some(state_machine) = state_machine {
                self.state_machines_map.insert(item.clone(), state_machine);
            }
        }
        if !self.virtualization_enabled() {
            if index as usize >= self.artboard_instances_by_index.len() {
                self.artboard_instances_by_index
                    .resize(index as usize + 1, None);
                self.state_machines_by_index
                    .resize(index as usize + 1, None);
            }
            self.artboard_instances_by_index[index as usize] = Some(artboard);
            self.state_machines_by_index[index as usize] = state_machine_instance;
        }
    }

    fn bind_artboard(
        &mut self,
        artboard_instance: &RuntimeArtboardInstanceHandle,
        list_item: CoreHandle,
    ) {
        let data_context = self
            .component()
            .with_artboard(Artboard::data_context)
            .flatten();
        let view_model_instance = list_item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten();
        if let Some(view_model_instance) = view_model_instance {
            artboard_instance.with_artboard_mut(|artboard_instance| {
                artboard_instance
                    .bind_view_model_instance_handle_with_parent(view_model_instance, data_context);
                artboard_instance.update_data_binds_default();
            });
            self.invalidate_ordered_list_indices_cache();
        }
    }

    pub fn remove_artboard_at(&mut self, index: i32) {
        if !self.virtualization_enabled()
            && index >= 0
            && (index as usize) < self.artboard_instances_by_index.len()
        {
            self.artboard_instances_by_index[index as usize] = None;
            self.state_machines_by_index[index as usize] = None;
        }
        if let Some(item) = self.list_item(index) {
            self.remove_artboard(item);
        }
    }

    pub fn remove_artboard(&mut self, item: CoreHandle) {
        self.invalidate_ordered_list_indices_cache();
        self.remove_draw_index_listener_for_item(&item);
        let artboard = self.artboard_instances_map.get(&item).cloned();
        if let Some(artboard) = artboard.as_ref() {
            artboard.with_artboard_mut(|artboard| {
                artboard.cleanup_semantic_tree();
                artboard.cleanup_focus_tree();
            });
            self.clear_artboard_override(artboard);
        }
        self.state_machines_map.remove(&item);
        self.artboard_instances_map.remove(&item);
    }

    fn create_artboard_recorders(&mut self, artboard: CoreHandle) {
        if !self.property_recorders_map.contains_key(&artboard) {
            let mut recorder = Box::new(PropertyRecorder::default());
            let nested_sources = artboard
                .with_downcast::<Artboard, _>(|artboard| {
                    recorder.record_artboard(artboard);
                    artboard
                        .nested_artboards()
                        .into_iter()
                        .filter_map(|nested| {
                            nested
                                .with(|nested| nested.nested_artboard_source_handle())
                                .flatten()
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            self.property_recorders_map.insert(artboard, recorder);
            for nested_artboard in nested_sources {
                self.create_artboard_recorders(nested_artboard);
            }
        }
    }

    fn apply_recorders_to_artboard(
        &self,
        artboard: &RuntimeArtboardInstanceHandle,
        source_artboard: &CoreHandle,
    ) {
        if let Some(recorder) = self.property_recorders_map.get(&source_artboard) {
            artboard.with_artboard_mut(|artboard| recorder.apply_artboard(artboard));
        }
        let nested_instances = artboard.with_artboard(|artboard| {
            artboard
                .nested_artboards()
                .into_iter()
                .filter_map(|nested| {
                    nested.with(|nested| {
                        Some((
                            nested.nested_artboard_instance_handle()?,
                            nested.nested_artboard_source_handle()?,
                        ))
                    })?
                })
                .collect::<Vec<_>>()
        });
        for (nested_instance, nested_source) in nested_instances {
            self.apply_recorders_to_artboard(&nested_instance, &nested_source);
        }
    }

    fn apply_recorders_to_state_machine(
        &self,
        state_machine_instance: &RuntimeStateMachineInstanceHandle,
        source_artboard: &CoreHandle,
    ) {
        if let Some(recorder) = self.property_recorders_map.get(source_artboard) {
            state_machine_instance.with_instance_mut(|state_machine_instance| {
                recorder.apply_state_machine(state_machine_instance);
            });
        }
    }
}

impl ArtboardComponentList {
    pub fn add_virtualizable(&mut self, index: i32) {
        let Some(list_item) = self.list_item(index) else {
            return;
        };
        let Some(artboard) = self.find_artboard(&list_item) else {
            return;
        };
        self.create_artboard_recorders(artboard.clone());
        let pooled = self
            .resource_pool
            .entry(artboard.clone())
            .or_default()
            .pop();
        if let Some(pooled_artboard) = pooled {
            self.apply_recorders_to_artboard(&pooled_artboard, &artboard);
            self.add_artboard_at(pooled_artboard, index, true);
        } else {
            self.create_artboard_at(index, true);
        }
        self.component_mut()
            .add_dirt(ComponentDirt::COMPONENTS, true);
        self.layout_parent_mut(LayoutComponent::mark_layout_style_dirty);
    }

    pub fn virtualizable_changed(&mut self) {
        let focus_manager = self
            .component()
            .with_artboard(Artboard::focus_manager_handle)
            .flatten();
        if let Some(focus_manager) = focus_manager.filter(|_| self.list_scope_focus_node.is_some())
        {
            self.sync_list_row_nodes_with_list(focus_manager);
        }
    }

    pub fn remove_virtualizable(&mut self, index: i32) {
        if let Some(list_item) = self.list_item(index) {
            let artboard = self.find_artboard(&list_item);
            let artboard_instance = self.artboard_instances_map.remove(&list_item);
            if let (Some(artboard), Some(artboard_instance)) =
                (artboard.as_ref(), artboard_instance)
            {
                self.resource_pool
                    .entry(artboard.clone())
                    .or_default()
                    .push(artboard_instance);
            }
            if let Some(state_machine) = self.state_machines_map.remove(&list_item) {
                if let Some(artboard) = artboard {
                    self.state_machines_pool
                        .entry(artboard)
                        .or_default()
                        .push(state_machine);
                }
            }
        }
        self.remove_artboard_at(index);
    }

    pub fn set_virtualizable_position(&mut self, index: i32, position: Vec2D) {
        let use_layout = self.layout_parent_handle().is_some();
        if let Some(artboard) = self.artboard_instance(index) {
            let origin = artboard.with_artboard(|artboard| {
                if use_layout {
                    artboard.origin()
                } else {
                    Vec2D::default()
                }
            });
            if let Some(item) = self.list_item(index) {
                self.artboard_transforms.insert(
                    item,
                    Mat2D::from_translate(position.x - origin.x, position.y - origin.y),
                );
            }
        }
    }

    fn virtualization_enabled_ref(&self) -> bool {
        self.scroll_constraint_handle()
            .and_then(|constraint| {
                constraint.with_downcast::<ScrollConstraint, _>(ScrollConstraint::virtualize)
            })
            .unwrap_or(false)
    }

    pub fn virtualization_enabled(&self) -> bool {
        self.virtualization_enabled_ref()
    }

    pub fn scroll_constraint_handle(&self) -> Option<CoreHandle> {
        self.provider_state
            .layout_constraints()
            .iter()
            .find(|constraint| {
                constraint
                    .with(|constraint| constraint.as_scroll_constraint().is_some())
                    .unwrap_or(false)
            })
            .cloned()
    }

    fn compute_layout_bounds(&mut self) {
        if self.virtualization_enabled() {
            let gap = self.gap();
            let mut running_width: f32 = 0.0;
            let mut running_height: f32 = 0.0;
            let is_horizontal = self.main_axis_is_row();
            for (index, size) in self.artboard_sizes.iter().copied().enumerate() {
                let real_gap = if index == self.artboard_sizes.len() - 1 {
                    0.0
                } else {
                    gap
                };
                if is_horizontal {
                    running_width += size.x + real_gap;
                    running_height = running_height.max(size.y);
                } else {
                    running_width = running_width.max(size.x);
                    running_height += size.y + real_gap;
                }
            }
            self.layout_size = Vec2D::new(running_width, running_height);
            if let Some(scroll) = self.scroll_constraint_handle() {
                scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                    scroll.constrain_virtualized(true);
                });
            }
        }
    }

    pub fn size(&self) -> Vec2D {
        self.layout_size
    }

    pub fn item_size(&self, index: i32) -> Vec2D {
        if index >= 0 && (index as usize) < self.artboard_sizes.len() {
            self.artboard_sizes[index as usize]
        } else {
            Vec2D::default()
        }
    }

    pub fn set_item_size(&mut self, size: Vec2D, index: i32) {
        if index >= 0 && (index as usize) < self.artboard_sizes.len() {
            self.artboard_sizes[index as usize] = size;
        }
    }

    pub fn gap(&mut self) -> f32 {
        self.gap_ref()
    }

    fn gap_ref(&self) -> f32 {
        let is_row = self.main_axis_is_row_ref();
        self.layout_parent_ref(|parent| {
            if is_row {
                parent.gap_horizontal()
            } else {
                parent.gap_vertical()
            }
        })
        .unwrap_or(0.0)
    }

    fn attach_artboard_override(
        &mut self,
        instance: &RuntimeArtboardInstanceHandle,
        list_item: CoreHandle,
    ) {
        let Some(view_model_instance) = list_item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten()
        else {
            return;
        };
        let Some(view_model_id) = view_model_instance
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::view_model_id)
        else {
            return;
        };
        let Some(artboards) = self
            .file
            .as_ref()
            .and_then(|file| file.with_file(File::artboards))
        else {
            return;
        };
        let mut artboard_index = -1;
        for artboard in &artboards {
            artboard_index += 1;
            if artboard.with_downcast::<Artboard, _>(Artboard::view_model_id) == Some(view_model_id)
            {
                break;
            }
        }
        if artboard_index < 0 && artboard_index as usize >= artboards.len() {
            return;
        }
        let mut artboard_override = None;
        let children = self.container_mut().children().to_vec();
        for child in children {
            let candidate_id =
                child.with_downcast::<ArtboardComponentListOverride, _>(|candidate| {
                    candidate.base.artboard_id()
                });
            if let Some(candidate_id) = candidate_id {
                if candidate_id == -1 {
                    artboard_override = Some(child.clone());
                } else if candidate_id == artboard_index {
                    artboard_override = Some(child);
                    break;
                }
            }
        }
        if let Some(artboard_override) = artboard_override {
            artboard_override.with_downcast_mut::<ArtboardComponentListOverride, _>(|override_| {
                instance.with_artboard_mut(|instance| override_.add_artboard(instance));
            });
        }
    }

    fn clear_artboard_override(&mut self, artboard_instance: &RuntimeArtboardInstanceHandle) {
        let children = self.container_mut().children().to_vec();
        for child in children {
            child.with_downcast_mut::<ArtboardComponentListOverride, _>(|override_| {
                artboard_instance.with_artboard_mut(|instance| override_.remove_artboard(instance));
            });
        }
    }

    pub fn main_axis_is_row(&self) -> bool {
        self.main_axis_is_row_ref()
    }

    fn main_axis_is_row_ref(&self) -> bool {
        self.layout_parent_handle()
            .and_then(|parent| {
                parent.with_downcast::<LayoutComponent, _>(LayoutComponent::main_axis_is_row)
            })
            .unwrap_or(true)
    }

    pub fn layout_parent_handle(&self) -> Option<CoreHandle> {
        self.component()
            .parent_handle()
            .filter(|parent| parent.with_downcast::<LayoutComponent, _>(|_| ()).is_some())
    }

    fn layout_parent_ref<R>(&self, use_parent: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.layout_parent_handle()?
            .with_downcast::<LayoutComponent, _>(use_parent)
    }

    fn layout_parent_mut<R>(
        &self,
        use_parent: impl FnOnce(&mut LayoutComponent) -> R,
    ) -> Option<R> {
        self.layout_parent_handle()?
            .with_downcast_mut::<LayoutComponent, _>(use_parent)
    }

    pub fn list_transform(&self) -> &Mat2D {
        self.transform().world_transform()
    }

    pub fn for_each_list_item_transform(&mut self, mut use_transform: impl FnMut(&mut Mat2D)) {
        for item in &self.list_items {
            if let Some(transform) = self.artboard_transforms.get_mut(item) {
                use_transform(transform);
            }
        }
    }

    pub fn add_map_rule(&mut self, rule: &ArtboardListMapRule) {
        self.artboard_map_rules
            .insert(rule.base.view_model_id(), rule.base.artboard_id());
    }

    pub fn set_visible_indices(&mut self, start: i32, end: i32) {
        self.visible_start_index = start;
        self.visible_end_index = end;
        self.invalidate_ordered_list_indices_cache();
    }

    pub fn should_reset_instances(&mut self, value: bool) {
        self.should_reset_instances = value;
    }

    pub fn num_layout_nodes(&self) -> usize {
        self.list_items.len()
    }

    pub fn item_count(&self) -> i32 {
        self.list_items.len() as i32
    }

    pub fn item(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance(index)
    }

    pub fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        Some(self.transform_mut())
    }

    pub fn transform_component(&self) -> Option<&TransformComponent> {
        Some(self.transform())
    }

    pub fn parent_artboard(&self) -> Option<CoreHandle> {
        self.component().artboard_handle()
    }

    pub fn mark_host_transform_dirty(&mut self) {
        self.transform_mut().mark_transform_dirty();
    }

    pub fn host_component(&self) -> Option<CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
    }

    pub fn is_layout_provider(&self) -> bool {
        true
    }

    pub fn type_(&self) -> i32 {
        self.core_type() as i32
    }
}

impl AdvancingComponent for ArtboardComponentList {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        ArtboardComponentList::advance_component(self, elapsed_seconds, flags)
    }
}

impl ResettingComponent for ArtboardComponentList {
    fn reset(&mut self) {
        ArtboardComponentList::reset(self);
    }
}

impl LayoutNodeProvider for ArtboardComponentList {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider_state
    }

    fn provider_handle(&self) -> Option<CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
    }

    fn owner_handle(&self) -> Option<CoreHandle> {
        self.provider_handle()
    }

    fn layout_bounds(&self) -> Aabb {
        ArtboardComponentList::layout_bounds(self)
    }

    fn layout_bounds_for_node(&self, index: usize) -> Aabb {
        ArtboardComponentList::layout_bounds_for_node(self, index)
    }

    fn sync_style_changes(&mut self) -> bool {
        ArtboardComponentList::sync_style_changes(self)
    }

    fn update_layout_bounds(&mut self, animate: bool) {
        ArtboardComponentList::update_layout_bounds(self, animate);
    }

    fn mark_layout_node_dirty(&mut self, should_force_update_layout_bounds: bool) {
        ArtboardComponentList::mark_layout_node_dirty(self, should_force_update_layout_bounds);
    }

    fn num_layout_nodes(&self) -> usize {
        ArtboardComponentList::num_layout_nodes(self)
    }

    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        ArtboardComponentList::cascade_layout_style(
            self,
            interpolation,
            interpolator,
            time,
            direction,
        )
    }
}

impl ConstrainableList for ArtboardComponentList {
    fn constrainable_list_state(&mut self) -> &mut ConstrainableListState {
        &mut self.constrainable_list_state
    }

    fn list_transform(&self) -> &Mat2D {
        ArtboardComponentList::list_transform(self)
    }

    fn for_each_list_item_transform(&mut self, use_transform: &mut dyn FnMut(&mut Mat2D)) {
        ArtboardComponentList::for_each_list_item_transform(self, use_transform);
    }
}

impl VirtualizingComponent for ArtboardComponentList {
    fn virtualization_enabled(&self) -> bool {
        self.virtualization_enabled_ref()
    }

    fn item_count(&self) -> i32 {
        ArtboardComponentList::item_count(self)
    }

    fn item(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        ArtboardComponentList::item(self, index)
    }

    fn size(&self) -> Vec2D {
        ArtboardComponentList::size(self)
    }

    fn item_size(&self, index: i32) -> Vec2D {
        ArtboardComponentList::item_size(self, index)
    }

    fn set_item_size(&mut self, size: Vec2D, index: i32) {
        ArtboardComponentList::set_item_size(self, size, index);
    }

    fn add_virtualizable(&mut self, index: i32) {
        ArtboardComponentList::add_virtualizable(self, index);
    }

    fn virtualizable_changed(&mut self) {
        ArtboardComponentList::virtualizable_changed(self);
    }

    fn remove_virtualizable(&mut self, index: i32) {
        ArtboardComponentList::remove_virtualizable(self, index);
    }

    fn set_visible_indices(&mut self, start: i32, end: i32) {
        ArtboardComponentList::set_visible_indices(self, start, end);
    }

    fn set_virtualizable_position(&mut self, index: i32, position: Vec2D) {
        ArtboardComponentList::set_virtualizable_position(self, index, position);
    }
}

impl ArtboardHost for ArtboardComponentList {
    fn artboard_count(&self) -> usize {
        ArtboardComponentList::artboard_count(self)
    }

    fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        ArtboardComponentList::artboard_instance(self, index)
    }

    fn internal_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        ArtboardComponentList::internal_data_context(self, data_context);
    }

    fn bind_view_model_instance(
        &mut self,
        view_model_instance: CoreHandle,
        parent: RuntimeDataContextHandle,
    ) {
        ArtboardComponentList::bind_view_model_instance(self, view_model_instance, parent);
    }

    fn clear_data_context(&mut self) {
        ArtboardComponentList::clear_data_context(self);
    }

    fn unbind(&mut self) {
        ArtboardComponentList::unbind(self);
    }

    fn update_data_binds(&mut self) {
        ArtboardComponentList::update_data_binds(self);
    }

    fn mark_hosting_layout_dirty(&mut self, artboard_instance: RuntimeArtboardInstanceWeakHandle) {
        if let Some(artboard_instance) = artboard_instance.upgrade() {
            ArtboardComponentList::mark_hosting_layout_dirty(self, &artboard_instance);
        }
    }

    fn parent_artboard(&self) -> Option<CoreHandle> {
        ArtboardComponentList::parent_artboard(self)
    }

    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> bool {
        artboard.upgrade().is_some_and(|artboard| {
            ArtboardComponentList::hit_test_host(self, position, skip_on_unclipped, &artboard)
        })
    }

    fn host_transform_point(
        &self,
        position: &Vec2D,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Vec2D {
        artboard.upgrade().map_or(*position, |artboard| {
            ArtboardComponentList::host_transform_point(self, position, &artboard)
        })
    }

    fn world_transform_for_artboard(&self, artboard: RuntimeArtboardInstanceWeakHandle) -> Mat2D {
        artboard.upgrade().map_or_else(Mat2D::identity, |artboard| {
            ArtboardComponentList::world_transform_for_artboard(self, &artboard)
        })
    }

    fn mark_host_transform_dirty(&mut self) {
        ArtboardComponentList::mark_host_transform_dirty(self);
    }

    fn is_layout_provider(&self) -> bool {
        true
    }

    fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        ArtboardComponentList::set_file(self, value);
    }

    fn file(&self) -> Option<RuntimeFileWeakHandle> {
        ArtboardComponentList::file(self)
    }

    fn host_component(&self) -> Option<CoreHandle> {
        ArtboardComponentList::host_component(self)
    }

    fn type_(&self) -> i32 {
        ArtboardComponentList::type_(self)
    }
}

impl DataBindListItemConsumer for ArtboardComponentList {
    fn update_list(&mut self, list: &[CoreHandle]) {
        ArtboardComponentList::update_list(self, list);
    }
}

impl ArtboardComponentListBaseCallbacks for ArtboardComponentList {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.component_mut()
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl std::ops::Deref for ArtboardComponentList {
    type Target = ArtboardComponentListBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ArtboardComponentList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
