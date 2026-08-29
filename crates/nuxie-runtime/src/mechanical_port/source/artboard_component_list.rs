use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::property_recorder::PropertyRecorder,
    animation::state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
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
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
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
    renderer::Renderer,
    resetting_component::ResettingComponent,
    semantic::semantic_data::SemanticData,
    transform_component::TransformComponent,
    viewmodel::{
        symbol_type::SymbolType,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
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
    data_bind_path_referencer:
        crate::mechanical_port::source::data_bind_path_referencer::DataBindPathReferencer,
    list_items: Vec<CoreHandle>,
    old_items: Vec<CoreHandle>,
    artboards_map: HashMap<u32, CoreHandle>,
    artboard_instances_map: HashMap<CoreHandle, RuntimeArtboardInstanceHandle>,
    state_machines_map: HashMap<CoreHandle, RuntimeStateMachineInstanceHandle>,
    resource_pool: HashMap<CoreHandle, Vec<Option<RuntimeArtboardInstanceHandle>>>,
    state_machines_pool: HashMap<CoreHandle, Vec<RuntimeStateMachineInstanceHandle>>,
    property_recorders_map: HashMap<CoreHandle, Rc<RefCell<PropertyRecorder>>>,
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
            data_bind_path_referencer: Default::default(),
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

    pub(crate) fn collapse_after_super_occurrence(owner: &CoreHandle, value: bool) {
        let mut index = 0;
        loop {
            let row = owner
                .with_downcast::<Self, _>(|list| {
                    (index < list.artboard_count()).then(|| list.artboard_instance(index as i32))
                })
                .expect("live ArtboardComponentList collapse owner");
            let Some(row) = row else { break };
            if let Some(artboard) = row {
                artboard.collapse_semantic_boundary(value);
            }
            index += 1;
        }
    }

    pub fn clear(&mut self) {
        for artboard in self.artboard_instances_map.values() {
            artboard.cleanup_semantic_tree();
        }
        self.clear_draw_index_listeners();
        self.invalidate_ordered_list_indices_cache();
        for artboard in self.artboard_instances_map.values() {
            artboard.cleanup_focus_tree();
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

    pub fn layout_node(
        &self,
        index: i32,
    ) -> Option<crate::mechanical_port::source::layout::layout_node_provider::LayoutNodeKey> {
        let artboard = self.artboard_instance(index)?;
        artboard.with_artboard_mut(|artboard| {
            artboard.take_layout_data();
            artboard.layout_node_key(0)
        })
    }

    pub fn list_item(&self, index: i32) -> Option<CoreHandle> {
        if index >= 0 && (index as usize) < self.list_items.len() {
            return Some(self.list_items[index as usize].clone());
        }
        None
    }

    pub fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance_with_virtualization(index, self.virtualization_enabled())
    }

    fn artboard_instance_with_virtualization(
        &self,
        index: i32,
        virtualized: bool,
    ) -> Option<RuntimeArtboardInstanceHandle> {
        if !virtualized {
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
                LayoutComponent::set_parent_is_row_with_host_occurrence(
                    &artboard.core_handle(),
                    parent_is_row,
                    self,
                );
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

    pub(crate) fn finish_layout_bounds_occurrence(owner: &CoreHandle) {
        let scroll = owner
            .with_downcast_mut::<Self, _>(Self::compute_layout_size_before_constraint)
            .expect("live ArtboardComponentList");
        if let Some(scroll) = scroll {
            ScrollConstraint::constrain_virtualized_occurrence(&scroll, true);
        }
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
                LayoutComponent::cascade_layout_style_occurrence(
                    &artboard.core_handle(),
                    inherited_interpolation,
                    inherited_interpolator.clone(),
                    inherited_interpolation_time,
                    direction,
                );
            }
        }
        false
    }

    pub fn sync_style_changes(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                if artboard.sync_style_changes() {
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
            .with_downcast::<ViewModelInstance, _>(|instance| instance.base.view_model_id())?;
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
                .with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id())
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
        Artboard::nested_instance_from_handle(&self.find_artboard(&list_item)?)
    }

    fn create_state_machine_instance_occurrence(
        owner: &CoreHandle,
        artboard: &RuntimeArtboardInstanceHandle,
    ) -> Option<RuntimeStateMachineInstanceHandle> {
        let instance = artboard.default_state_machine_handle()?;
        Self::link_state_machine_to_artboard_occurrence(owner, &instance, artboard);
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
        for index in 0..self.list_row_focus_nodes.len() {
            let Some(node) = self.list_row_focus_nodes[index].clone() else {
                continue;
            };
            let focus_manager = node.borrow().manager();
            if let Some(focus_manager) = focus_manager {
                focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
            }
            self.list_row_focus_nodes[index] = None;
        }
        self.list_row_focus_nodes.clear();
        let Some(node) = self.list_scope_focus_node.clone() else {
            return;
        };
        let focus_manager = node.borrow().manager();
        if let Some(focus_manager) = focus_manager {
            focus_manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
        }
        self.list_scope_focus_node = None;
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
            FocusNode::remove_from_parent(row);
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
    if artboard.with_artboard(|artboard| artboard.root_focus_data_count()) > 0 {
        return true;
    }
    let nested = artboard.with_artboard(|artboard| artboard.nested_artboards());
    for host in nested {
        let is_data_bound = host
            .with(|host| host.nested_artboard_is_data_bound())
            .flatten()
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
        .with_artboard(|artboard| artboard.artboard_component_lists())
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
                    let manager = row.borrow().manager();
                    if let Some(manager) = manager {
                        manager.with_focus_manager_mut(|manager| manager.remove_child(&row));
                    }
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
                let manager = unmapped.borrow().manager();
                if let Some(manager) = manager {
                    manager.with_focus_manager_mut(|manager| manager.remove_child(&unmapped));
                }
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
                if instance.with_artboard(|instance| instance.focus_manager().is_some()) {
                    instance.cleanup_focus_tree();
                }
                instance.build_focus_tree(Some(focus_manager.clone()), row);
            }
        }
    }

    fn link_state_machine_to_artboard_occurrence(
        owner: &CoreHandle,
        state_machine_instance: &RuntimeStateMachineInstanceHandle,
        artboard_instance: &RuntimeArtboardInstanceHandle,
    ) {
        let data_context = artboard_instance.with_artboard(|artboard| artboard.base.data_context());
        if let Some(data_context) = data_context {
            let container = state_machine_instance.with_instance_mut(|state_machine_instance| {
                state_machine_instance.set_data_context_handle(data_context);
                state_machine_instance.data_bind_container.clone()
            });
            container.update_data_binds(false);
        }
        let parent = owner
            .with_downcast::<Self, _>(|owner| owner.component().artboard_handle())
            .flatten();
        let focus_manager = parent
            .as_ref()
            .and_then(|parent| parent.with_downcast::<Artboard, _>(Artboard::focus_manager_handle))
            .flatten();
        if let Some(focus_manager) = focus_manager {
            state_machine_instance.with_instance_mut(|machine| {
                machine.set_external_focus_manager_handle(focus_manager)
            });
        }
        let semantic_manager = parent
            .as_ref()
            .and_then(|parent| {
                parent.with_downcast::<Artboard, _>(Artboard::semantic_manager_handle)
            })
            .flatten();
        if let Some(semantic_manager) = semantic_manager {
            let parent_node = SemanticData::find_closest_semantic_node_handle(Some(owner.clone()));
            state_machine_instance.with_instance_mut(|machine| {
                machine.set_external_semantic_manager_handle(semantic_manager, parent_node);
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

    pub fn update_list_occurrence(owner: &CoreHandle, list: &[CoreHandle]) {
        let Some((previous_list_items, previous_row_nodes)) = owner
            .with_downcast_mut::<Self, _>(|owner| owner.begin_list_change(list))
            .expect("live ArtboardComponentList")
        else {
            return;
        };
        let mut index = 0;
        while index
            < owner
                .with_downcast::<Self, _>(|owner| owner.list_items.len())
                .expect("live ArtboardComponentList")
        {
            let create = owner
                .with_downcast_mut::<Self, _>(|owner| owner.prepare_list_item(index))
                .expect("live ArtboardComponentList");
            if create {
                Self::create_artboard_at_occurrence(owner, index as i32, false);
            }
            index += 1;
        }
        Self::finish_layout_bounds_occurrence(owner);
        Self::sync_layout_children_occurrence(owner);
        owner
            .with_downcast_mut::<Self, _>(|owner| {
                owner.finish_list_after_layout_children(&previous_list_items, &previous_row_nodes);
            })
            .expect("live ArtboardComponentList");
    }

    fn begin_list_change(
        &mut self,
        list: &[CoreHandle],
    ) -> Option<(Vec<CoreHandle>, Vec<Option<FocusNodeRef>>)> {
        if Self::lists_are_equal(Some(&self.list_items), Some(list)) {
            return None;
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
        Some((previous_list_items, previous_row_nodes))
    }

    fn prepare_list_item(&mut self, index: usize) -> bool {
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
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(
                    &symbol,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_symbol_list_index_base::ViewModelInstanceSymbolListIndexBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
                    index as u32,
                );
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
                return true;
            } else {
                self.artboard_instances_by_index[index] =
                    self.artboard_instances_map.get(&item).cloned();
                self.state_machines_by_index[index] = self.state_machines_map.get(&item).cloned();
            }
        }
        false
    }

    fn finish_list_after_layout_children(
        &mut self,
        previous_list_items: &[CoreHandle],
        previous_row_nodes: &[Option<FocusNodeRef>],
    ) {
        self.mark_layout_node_dirty(false);
        self.transform_mut().mark_world_transform_dirty();
        self.component_mut()
            .add_dirt(ComponentDirt::COMPONENTS, false);
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
                previous_list_items,
                previous_row_nodes,
            );
        }
    }

    fn sync_layout_children_occurrence(owner: &CoreHandle) {
        let parent = owner
            .with_downcast::<Self, _>(Self::layout_parent_handle)
            .expect("live ArtboardComponentList");
        if let Some(parent) = parent {
            LayoutComponent::sync_layout_children_occurrence(&parent);
        }
    }

    pub fn advance_component_occurrence(
        owner: &CoreHandle,
        elapsed_seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        let inactive = owner
            .with_downcast::<Self, _>(|owner| {
                owner.artboard_count() == 0 || owner.component().is_collapsed()
            })
            .expect("live ArtboardComponentList");
        if inactive {
            return false;
        }
        let mut keep_going = false;
        let advance_nested = flags.contains(AdvanceFlags::ADVANCE_NESTED);
        let new_frame = flags.contains(AdvanceFlags::NEW_FRAME);
        let advancing_flags = flags & !AdvanceFlags::IS_ROOT;
        let mut index = 0;
        while index
            < owner
                .with_downcast::<Self, _>(Self::artboard_count)
                .expect("live ArtboardComponentList") as i32
        {
            if advance_nested {
                let state_machine = owner
                    .with_downcast::<Self, _>(|owner| owner.state_machine_instance(index))
                    .flatten();
                if let Some(state_machine) = state_machine {
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
            let artboard = owner
                .with_downcast::<Self, _>(|owner| owner.artboard_instance(index))
                .flatten();
            if let Some(artboard) = artboard {
                if artboard.advance_internal(elapsed_seconds, advancing_flags) {
                    keep_going = true;
                }
                if artboard.with_artboard(|artboard| artboard.has_component_dirt()) {
                    owner.with_downcast_mut::<Self, _>(|owner| {
                        owner
                            .component_mut()
                            .add_dirt(ComponentDirt::COMPONENTS, false);
                    });
                }
            }
            index += 1;
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
                artboard.with_artboard_mut(|artboard| artboard.reset());
            }
        }
    }

    pub fn layout_bounds(&self) -> Aabb {
        Aabb::new(0.0, 0.0, self.layout_size.x, self.layout_size.y)
    }

    pub fn layout_bounds_for_node(&self, index: usize) -> Aabb {
        self.layout_bounds_for_node_with_scroll(index, None)
    }

    pub(crate) fn layout_bounds_for_node_with_scroll(
        &self,
        index: usize,
        scroll: Option<&ScrollConstraint>,
    ) -> Aabb {
        let virtualized = self.virtualization_enabled_with_scroll(scroll);
        if virtualized {
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
            if let Some(artboard) =
                self.artboard_instance_with_virtualization(index as i32, virtualized)
            {
                return artboard.with_artboard(|artboard| artboard.layout_bounds());
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
                if let Some(parent) = self.component().artboard_handle() {
                    Artboard::mark_layout_dirty_occurrence(
                        &parent,
                        artboard_instance.core_handle(),
                        None,
                    );
                }
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
                value.with_downcast::<ViewModelInstanceNumber, _>(|number| {
                    number.base.property_value()
                })
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
                        artboard.draw_internal(renderer);
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
                    artboard.draw_internal(renderer);
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
                    parent
                        .with(|parent| {
                            parent
                                .as_layout_component()
                                .map(|layout| *layout.world_transform())
                        })
                        .flatten()
                })
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        let local = transform * offset;
        self.component()
            .with_artboard_mut(|artboard| artboard.root_transform(local))
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
            .filter(|parent| parent.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY));
        if let Some(parent_layout) = parent_layout {
            let bounds = self.layout_bounds();
            if let Some(transform) = parent_layout
                .with(|parent| {
                    parent.as_layout_component().map(|layout| {
                        *layout.world_transform()
                            * Mat2D::from_translate(bounds.left(), bounds.top())
                    })
                })
                .flatten()
            {
                return transform * Mat2D::from_translate(position.x, position.y);
            }
        }
        let transform = if self.virtualization_enabled_ref() {
            self.component()
                .parent_handle()
                .and_then(|parent| {
                    parent
                        .with(|parent| {
                            parent
                                .as_layout_component()
                                .map(|layout| *layout.world_transform())
                        })
                        .flatten()
                })
                .unwrap_or_else(Mat2D::identity)
        } else {
            *self.transform().world_transform()
        };
        transform * Mat2D::from_translate(position.x, position.y)
    }

    pub(crate) fn update_after_transform_occurrence(owner: &CoreHandle, value: ComponentDirt) {
        if owner
            .with_downcast::<Self, _>(Self::artboard_count)
            .expect("live ArtboardComponentList")
            == 0
        {
            return;
        }
        if Component::has_dirt_in(value, ComponentDirt::WORLD_TRANSFORM) {
            let mut index = 0;
            loop {
                let next = owner
                    .with_downcast::<Self, _>(|list| {
                        (index < list.artboard_count() as i32)
                            .then(|| list.artboard_instance(index))
                    })
                    .expect("live ArtboardComponentList");
                let Some(artboard) = next else {
                    break;
                };
                if let Some(artboard) = artboard {
                    artboard.with_artboard_mut(|artboard| {
                        artboard.mark_semantic_boundary_transform_dirty()
                    });
                }
                index += 1;
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::RENDER_OPACITY) {
            let mut index = 0;
            loop {
                let next = owner
                    .with_downcast::<Self, _>(|list| {
                        (index < list.artboard_count() as i32)
                            .then(|| list.artboard_instance(index))
                    })
                    .expect("live ArtboardComponentList");
                let Some(artboard) = next else {
                    break;
                };
                if let Some(artboard) = artboard {
                    let opacity = owner
                        .with_downcast::<Self, _>(|list| list.transform().render_opacity())
                        .expect("live ArtboardComponentList");
                    crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_double_handle(
                        &artboard.core_handle(),
                        crate::mechanical_port::source::generated::world_transform_component_base::WorldTransformComponentBase::OPACITY_PROPERTY_KEY as i32,
                        opacity,
                    );
                }
                index += 1;
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::COMPONENTS) {
            let mut index = 0;
            loop {
                let next = owner
                    .with_downcast::<Self, _>(|list| {
                        (index < list.artboard_count() as i32)
                            .then(|| list.artboard_instance(index))
                    })
                    .expect("live ArtboardComponentList");
                let Some(artboard) = next else {
                    break;
                };
                if let Some(artboard) = artboard {
                    artboard.update_pass(false);
                }
                index += 1;
            }
        }
    }

    pub(crate) fn update_world_transform_before_super(&mut self) {
        self.update_artboards_world_transform();
    }

    pub(crate) fn layout_constraint_handles(&self) -> Vec<CoreHandle> {
        self.provider_state.layout_constraints().to_vec()
    }

    pub(crate) fn active_list_constraint_handles(&self) -> Vec<CoreHandle> {
        if self.virtualization_enabled() {
            Vec::new()
        } else {
            self.constrainable_list_state.list_constraints.clone()
        }
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

    pub fn internal_data_context(&mut self, value: RuntimeDataContextHandle) {
        for artboard in self.artboard_instances_map.values() {
            if let Some(data_context) = artboard.data_context() {
                data_context.with_context_mut(|context| context.set_parent(Some(value.clone())));
                artboard.internal_data_context(data_context);
            }
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
                let container = state_machine
                    .with_instance(|state_machine| state_machine.data_bind_container.clone());
                container.update_data_binds(false);
            }
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.update_data_binds(true);
            }
        }
    }

    pub(crate) fn update_data_binds_occurrence(owner: &CoreHandle) {
        let mut index = 0;
        while index
            < owner
                .with_downcast::<Self, _>(Self::artboard_count)
                .expect("ArtboardComponentList host")
        {
            let machine = owner
                .with_downcast::<Self, _>(|list| list.state_machine_instance(index as i32))
                .flatten();
            if let Some(machine) = machine {
                let container =
                    machine.with_instance(|machine| machine.data_bind_container.clone());
                container.update_data_binds(false);
            }
            let artboard = owner
                .with_downcast::<Self, _>(|list| list.artboard_instance(index as i32))
                .flatten();
            if let Some(artboard) = artboard {
                artboard.update_data_binds(true);
            }
            index += 1;
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

    fn create_artboard_for_index(
        &mut self,
        index: i32,
    ) -> Option<(RuntimeArtboardInstanceHandle, Option<CoreHandle>)> {
        let item = self.list_item(index)?;
        let artboard = self.create_artboard(item.clone())?;
        let artboard_override = self.artboard_override_for_item(item);
        Some((artboard, artboard_override))
    }

    pub fn create_artboard_at_occurrence(owner: &CoreHandle, index: i32, force_layout_sync: bool) {
        let artboard = owner
            .with_downcast_mut::<Self, _>(|owner| owner.create_artboard_for_index(index))
            .expect("live ArtboardComponentList");
        if let Some((artboard, artboard_override)) = artboard {
            if let Some(artboard_override) = artboard_override {
                // addArtboard reads the parent list's current main axis while
                // applying width and height. Release this same list first;
                // registration and both overrides still precede addArtboardAt.
                artboard_override
                    .with_downcast_mut::<ArtboardComponentListOverride, _>(|override_| {
                        override_.add_artboard(&artboard);
                    })
                    .expect("selected ArtboardComponentListOverride remains live");
            }
            Self::add_artboard_at_occurrence(owner, artboard, index, force_layout_sync);
        }
    }

    pub fn add_artboard_at_occurrence(
        owner: &CoreHandle,
        artboard: RuntimeArtboardInstanceHandle,
        index: i32,
        force_layout_sync: bool,
    ) {
        let item = owner
            .with_downcast_mut::<Self, _>(|owner| owner.begin_add_artboard_at(&artboard, index))
            .expect("live ArtboardComponentList");
        let Some(item) = item else {
            return;
        };
        Self::bind_artboard_occurrence(owner, &artboard, &item);
        let parent = owner
            .with_downcast::<Self, _>(Self::parent_artboard)
            .expect("live ArtboardComponentList");
        Artboard::set_host_occurrence(&artboard.core_handle(), Some(owner.clone()), parent);
        artboard.with_artboard_mut(|artboard| artboard.set_frame_origin(false));
        owner
            .with_downcast_mut::<Self, _>(|owner| {
                let parent_is_row = owner.main_axis_is_row();
                LayoutComponent::set_parent_is_row_with_host_occurrence(
                    &artboard.core_handle(),
                    parent_is_row,
                    owner,
                );
            })
            .expect("live ArtboardComponentList");
        if force_layout_sync {
            Self::sync_layout_children_occurrence(owner);
        }
        Self::finish_add_artboard_at_occurrence(owner, artboard, index, item);
    }

    fn begin_add_artboard_at(
        &mut self,
        artboard: &RuntimeArtboardInstanceHandle,
        index: i32,
    ) -> Option<CoreHandle> {
        let item = self.list_item(index)?;
        self.artboard_instances_map
            .insert(item.clone(), artboard.clone());
        Some(item)
    }

    fn finish_add_artboard_at_occurrence(
        owner: &CoreHandle,
        artboard: RuntimeArtboardInstanceHandle,
        index: i32,
        item: CoreHandle,
    ) {
        let source_artboard = owner
            .with_downcast_mut::<Self, _>(|owner| owner.find_artboard(&item))
            .flatten();
        let pooled = source_artboard.as_ref().and_then(|source| {
            owner
                .with_downcast_mut::<Self, _>(|owner| {
                    owner
                        .state_machines_pool
                        .entry(source.clone())
                        .or_default()
                        .last()
                        .cloned()
                })
                .flatten()
        });
        let state_machine_instance = if let Some(state_machine) = pooled {
            state_machine.with_instance_mut(StateMachineInstance::reset_state);
            let source = source_artboard.as_ref().expect("pooled machine source");
            Self::apply_recorders_to_state_machine_occurrence(owner, &state_machine, source);
            owner.with_downcast_mut::<Self, _>(|owner| {
                owner
                    .state_machines_map
                    .insert(item.clone(), state_machine.clone());
            });
            Self::link_state_machine_to_artboard_occurrence(owner, &state_machine, &artboard);
            owner.with_downcast_mut::<Self, _>(|owner| {
                owner
                    .state_machines_pool
                    .get_mut(source)
                    .expect("source machine pool")
                    .pop();
            });
            Some(state_machine)
        } else {
            let machine = Self::create_state_machine_instance_occurrence(owner, &artboard);
            owner.with_downcast_mut::<Self, _>(|owner| {
                if let Some(machine) = &machine {
                    owner
                        .state_machines_map
                        .insert(item.clone(), machine.clone());
                }
            });
            machine
        };
        owner.with_downcast_mut::<Self, _>(|owner| {
            if !owner.virtualization_enabled() {
                if index as usize >= owner.artboard_instances_by_index.len() {
                    owner
                        .artboard_instances_by_index
                        .resize(index as usize + 1, None);
                    owner
                        .state_machines_by_index
                        .resize(index as usize + 1, None);
                }
                owner.artboard_instances_by_index[index as usize] = Some(artboard);
                owner.state_machines_by_index[index as usize] = state_machine_instance;
            }
        });
    }

    fn bind_artboard_occurrence(
        owner: &CoreHandle,
        artboard_instance: &RuntimeArtboardInstanceHandle,
        list_item: &CoreHandle,
    ) {
        let parent = owner
            .with_downcast::<Self, _>(|owner| owner.component().artboard_handle())
            .flatten();
        let data_context = parent.and_then(|parent| {
            parent
                .with_downcast::<Artboard, _>(Artboard::data_context)
                .flatten()
        });
        let view_model_instance = list_item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten();
        if let Some(view_model_instance) = view_model_instance {
            artboard_instance
                .bind_view_model_instance_with_parent(Some(view_model_instance), data_context);
            artboard_instance.update_data_binds(true);
            owner.with_downcast_mut::<Self, _>(Self::invalidate_ordered_list_indices_cache);
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
            artboard.cleanup_semantic_tree();
            artboard.cleanup_focus_tree();
            self.clear_artboard_override(artboard);
        }
        self.state_machines_map.remove(&item);
        self.artboard_instances_map.remove(&item);
    }

    fn create_artboard_recorders(&mut self, artboard: CoreHandle) {
        if !self.property_recorders_map.contains_key(&artboard) {
            let mut recorder = PropertyRecorder::default();
            recorder.record_artboard(&artboard);
            let nested_sources = artboard
                .with_downcast::<Artboard, _>(|artboard| {
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
            self.property_recorders_map
                .insert(artboard, Rc::new(RefCell::new(recorder)));
            for nested_artboard in nested_sources {
                self.create_artboard_recorders(nested_artboard);
            }
        }
    }

    fn apply_recorders_to_artboard_occurrence(
        owner: &CoreHandle,
        artboard: &RuntimeArtboardInstanceHandle,
        source_artboard: &CoreHandle,
    ) {
        let recorder = owner
            .with_downcast::<Self, _>(|owner| {
                owner.property_recorders_map.get(source_artboard).cloned()
            })
            .expect("live ArtboardComponentList");
        if let Some(recorder) = recorder {
            // Restored properties synchronously notify this list through their
            // hosted artboard. Retain the actual recorder without holding ACL.
            recorder.borrow_mut().apply_artboard(artboard);
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
            Self::apply_recorders_to_artboard_occurrence(owner, &nested_instance, &nested_source);
        }
    }

    fn apply_recorders_to_state_machine_occurrence(
        owner: &CoreHandle,
        state_machine_instance: &RuntimeStateMachineInstanceHandle,
        source_artboard: &CoreHandle,
    ) {
        let recorder = owner
            .with_downcast::<Self, _>(|owner| {
                owner.property_recorders_map.get(source_artboard).cloned()
            })
            .expect("live ArtboardComponentList");
        if let Some(recorder) = recorder {
            recorder
                .borrow_mut()
                .apply_state_machine(state_machine_instance);
        }
    }
}

impl ArtboardComponentList {
    pub fn add_virtualizable_occurrence(owner: &CoreHandle, index: i32) {
        let prepared = owner
            .with_downcast_mut::<Self, _>(|owner| {
                let list_item = owner.list_item(index)?;
                let artboard = owner.find_artboard(&list_item)?;
                owner.create_artboard_recorders(artboard.clone());
                let pooled = owner
                    .resource_pool
                    .entry(artboard.clone())
                    .or_default()
                    .last()
                    .cloned();
                let pooled =
                    pooled.map(|pooled| pooled.expect("resource pool entry not already moved"));
                Some((artboard, pooled))
            })
            .expect("live ArtboardComponentList");
        let Some((artboard, pooled)) = prepared else {
            return;
        };
        if let Some(pooled_artboard) = pooled {
            Self::apply_recorders_to_artboard_occurrence(owner, &pooled_artboard, &artboard);
            let pooled_artboard = owner
                .with_downcast_mut::<Self, _>(|owner| {
                    owner
                        .resource_pool
                        .get_mut(&artboard)
                        .expect("source resource pool")
                        .last_mut()
                        .expect("source pooled artboard")
                        .take()
                        .expect("source pooled artboard not already moved")
                })
                .expect("live ArtboardComponentList");
            Self::add_artboard_at_occurrence(owner, pooled_artboard, index, true);
            owner.with_downcast_mut::<Self, _>(|owner| {
                owner
                    .resource_pool
                    .get_mut(&artboard)
                    .expect("source resource pool")
                    .pop();
            });
        } else {
            Self::create_artboard_at_occurrence(owner, index, true);
        }
        let parent = owner
            .with_downcast_mut::<Self, _>(|owner| {
                owner
                    .component_mut()
                    .add_dirt(ComponentDirt::COMPONENTS, false);
                owner.layout_parent_handle()
            })
            .expect("live ArtboardComponentList");
        if let Some(parent) = parent {
            LayoutComponent::mark_layout_style_dirty_occurrence(&parent);
        }
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
                    .push(Some(artboard_instance));
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
        self.virtualization_enabled_with_scroll(None)
    }

    fn virtualization_enabled_with_scroll(
        &self,
        borrowed_scroll: Option<&ScrollConstraint>,
    ) -> bool {
        self.scroll_constraint_handle().is_some_and(|constraint| {
            if let Some(scroll) =
                borrowed_scroll.filter(|scroll| scroll.handle().as_ref() == Some(&constraint))
            {
                return scroll.base.virtualize();
            }
            constraint
                .with_downcast::<ScrollConstraint, _>(|constraint| constraint.base.virtualize())
                .expect("live ScrollConstraint")
        })
    }

    pub fn virtualization_enabled(&self) -> bool {
        self.virtualization_enabled_ref()
    }

    pub fn scroll_constraint_handle(&self) -> Option<CoreHandle> {
        self.provider_state
            .layout_constraints()
            .iter()
            .find(|constraint| {
                constraint.is_type_of(crate::mechanical_port::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase::TYPE_KEY)
            })
            .cloned()
    }

    fn compute_layout_bounds(&mut self) {
        if let Some(scroll) = self.compute_layout_size_before_constraint() {
            ScrollConstraint::constrain_virtualized_occurrence(&scroll, true);
        }
    }

    fn compute_layout_size_before_constraint(&mut self) -> Option<CoreHandle> {
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
            return self.scroll_constraint_handle();
        }
        None
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

    fn artboard_override_for_item(&mut self, list_item: CoreHandle) -> Option<CoreHandle> {
        let Some(view_model_instance) = list_item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten()
        else {
            return None;
        };
        let Some(view_model_id) = view_model_instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.base.view_model_id())
        else {
            return None;
        };
        let Some(artboards) = self
            .file
            .as_ref()
            .and_then(|file| file.with_file(File::artboards))
        else {
            return None;
        };
        let mut artboard_index = -1i32;
        for artboard in &artboards {
            artboard_index += 1;
            if artboard.with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id())
                == Some(view_model_id)
            {
                break;
            }
        }
        if artboard_index < 0 && artboard_index as usize >= artboards.len() {
            return None;
        }
        let mut artboard_override = None;
        let children = self.container_mut().children().to_vec();
        for child in children {
            let candidate_id =
                child.with_downcast::<ArtboardComponentListOverride, _>(|candidate| {
                    candidate.base.artboard_id()
                });
            if let Some(candidate_id) = candidate_id {
                if candidate_id == u32::MAX {
                    artboard_override = Some(child.clone());
                } else if candidate_id == artboard_index as u32 {
                    artboard_override = Some(child);
                    break;
                }
            }
        }
        artboard_override
    }

    fn clear_artboard_override(&mut self, artboard_instance: &RuntimeArtboardInstanceHandle) {
        let children = self.container_mut().children().to_vec();
        for child in children {
            child.with_downcast_mut::<ArtboardComponentListOverride, _>(|override_| {
                override_.remove_artboard(artboard_instance);
            });
        }
    }

    pub fn main_axis_is_row(&self) -> bool {
        self.main_axis_is_row_ref()
    }

    fn main_axis_is_row_ref(&self) -> bool {
        self.layout_parent_handle()
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_layout_component()
                            .map(LayoutComponent::main_axis_is_row)
                    })
                    .flatten()
            })
            .unwrap_or(true)
    }

    pub fn layout_parent_handle(&self) -> Option<CoreHandle> {
        self.component()
            .parent_handle()
            .filter(|parent| parent.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY))
    }

    fn layout_parent_ref<R>(&self, use_parent: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.layout_parent_handle()?
            .with(|parent| parent.as_layout_component().map(use_parent))?
    }

    fn layout_parent_mut<R>(
        &self,
        use_parent: impl FnOnce(&mut LayoutComponent) -> R,
    ) -> Option<R> {
        self.layout_parent_handle()?
            .with_mut(|parent| parent.as_layout_component_mut().map(use_parent))?
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
        self.artboard_map_rules.insert(
            rule.base.view_model_id() as i32,
            rule.base.artboard_id() as i32,
        );
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
    fn data_bind_path_referencer(
        &self,
    ) -> &crate::mechanical_port::source::data_bind_path_referencer::DataBindPathReferencer {
        &self.data_bind_path_referencer
    }

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
