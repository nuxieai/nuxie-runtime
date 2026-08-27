use std::{collections::HashMap, ffi::c_void, ptr::NonNull, rc::Rc};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::{keyframe_interpolator::KeyFrameInterpolator, state_machine::StateMachineInstance},
    artboard::{Artboard, ArtboardInstance},
    artboard_host::ArtboardHost,
    artboard_list_map_rule::ArtboardListMapRule,
    component::Component,
    component_dirt::ComponentDirt,
    constraints::{
        constrainable_list::{ConstrainableList, ConstrainableListState},
        list_constraint::ListConstraint,
        scrolling::scroll_constraint::ScrollConstraint,
    },
    core::Core,
    data_bind::{
        data_bind_list_item_consumer::DataBindListItemConsumer, data_context::DataContext,
    },
    dirtyable::Dirtyable,
    file::File,
    generated::artboard_component_list_base::ArtboardComponentListBaseCallbacks,
    hit_info::HitInfo,
    input::{
        focus_manager::FocusManager,
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
    refcnt::{Rcp, ref_rcp},
    renderer::Renderer,
    resetting_component::ResettingComponent,
    semantic::semantic_data::SemanticData,
    transform_component::TransformComponent,
    viewmodel::{
        symbol_type::SymbolType, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_value::ViewModelInstanceValue,
        viewmodel_value_dependent::ViewModelValueDependent,
    },
    virtualizing_component::{Virtualizable, VirtualizingComponent},
};

pub use crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase;

pub struct ArtboardListDrawIndexDependent {
    list: *mut ArtboardComponentList,
    value: Rcp<ViewModelInstanceValue>,
}

impl ArtboardListDrawIndexDependent {
    pub fn new(list: *mut ArtboardComponentList, value: *mut ViewModelInstanceValue) -> Box<Self> {
        let mut dependent = Box::new(Self {
            list,
            value: unsafe { ref_rcp(value) },
        });
        unsafe {
            (*value).add_dependent(NonNull::from(
                dependent.as_mut() as &mut dyn ViewModelValueDependent
            ));
        }
        dependent
    }

    pub fn clear(&mut self) {
        if !self.value.get().is_null() {
            unsafe {
                (*self.value.get())
                    .remove_dependent(NonNull::from(self as &mut dyn ViewModelValueDependent));
            }
            self.value.reset(None);
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
        if !self.list.is_null() {
            unsafe {
                (*self.list).invalidate_ordered_list_indices_cache();
                (*self.list)
                    .component_mut()
                    .add_dirt(ComponentDirt::COMPONENTS, false);
            }
        }
    }
}

impl ViewModelValueDependent for ArtboardListDrawIndexDependent {
    fn relink_data_bind(&mut self) {}
}

pub struct ArtboardComponentList {
    pub base: ArtboardComponentListBase,
    list_items: Vec<Rcp<ViewModelInstanceListItem>>,
    old_items: Vec<Rcp<ViewModelInstanceListItem>>,
    artboards_map: HashMap<u32, *mut Artboard>,
    artboard_instances_map: HashMap<Rcp<ViewModelInstanceListItem>, Box<ArtboardInstance>>,
    state_machines_map: HashMap<Rcp<ViewModelInstanceListItem>, Box<StateMachineInstance>>,
    resource_pool: HashMap<*mut Artboard, Vec<Box<ArtboardInstance>>>,
    state_machines_pool: HashMap<*mut Artboard, Vec<Box<StateMachineInstance>>>,
    property_recorders_map: HashMap<*const Artboard, Box<PropertyRecorder>>,
    artboard_transforms: HashMap<*mut ArtboardInstance, Mat2D>,
    artboard_instances_by_index: Vec<*mut ArtboardInstance>,
    state_machines_by_index: Vec<*mut StateMachineInstance>,
    file: *mut File,
    artboard_sizes: Vec<Vec2D>,
    layout_size: Vec2D,
    visible_start_index: i32,
    visible_end_index: i32,
    artboard_overrides_map: HashMap<*mut ArtboardInstance, *mut ArtboardComponentListOverride>,
    artboard_map_rules: HashMap<i32, i32>,
    list_scope_focus_node: Option<FocusNodeRef>,
    list_row_focus_nodes: Vec<Option<FocusNodeRef>>,
    should_reset_instances: bool,
    list_uses_draw_index_sort: bool,
    ordered_list_indices_cache_valid: bool,
    cached_ordered_list_indices: Vec<i32>,
    draw_index_dependents:
        HashMap<Rcp<ViewModelInstanceListItem>, Box<ArtboardListDrawIndexDependent>>,
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
            file: std::ptr::null_mut(),
            artboard_sizes: Vec::new(),
            layout_size: Vec2D::default(),
            visible_start_index: -1,
            visible_end_index: -1,
            artboard_overrides_map: HashMap::new(),
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
        for index in 0..self.artboard_count() {
            if let Some(nested_artboard) = self.artboard_instance(index as i32) {
                nested_artboard.collapse_semantic_boundary(value);
            }
        }
        true
    }

    pub fn clear(&mut self) {
        for artboard in self.artboard_instances_map.values_mut() {
            artboard.cleanup_semantic_tree();
        }
        self.clear_draw_index_listeners();
        self.invalidate_ordered_list_indices_cache();
        for artboard in self.artboard_instances_map.values_mut() {
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
        self.artboard_overrides_map.clear();
    }

    pub fn artboard_count(&self) -> usize {
        self.list_items.len()
    }

    pub fn list_item(&self, index: i32) -> Option<Rcp<ViewModelInstanceListItem>> {
        if index >= 0 && (index as usize) < self.list_items.len() {
            return Some(self.list_items[index as usize].clone());
        }
        None
    }

    pub fn artboard_instance(&mut self, index: i32) -> Option<&mut ArtboardInstance> {
        if !self.virtualization_enabled() {
            if index >= 0 && (index as usize) < self.artboard_instances_by_index.len() {
                return unsafe { self.artboard_instances_by_index[index as usize].as_mut() };
            }
            return None;
        }
        if index >= 0 && (index as usize) < self.list_items.len() {
            let item = self.list_items[index as usize].clone();
            return self.artboard_instances_map.get_mut(&item).map(Box::as_mut);
        }
        None
    }

    pub fn index_of_artboard_instance(&self, instance: *mut ArtboardInstance) -> i32 {
        if instance.is_null() {
            return -1;
        }
        for (index, item) in self.list_items.iter().enumerate() {
            if self
                .artboard_instances_map
                .get(item)
                .is_some_and(|artboard| std::ptr::eq(artboard.as_ref(), unsafe { &*instance }))
            {
                return index as i32;
            }
        }
        -1
    }

    pub fn state_machine_instance(&mut self, index: i32) -> Option<&mut StateMachineInstance> {
        if !self.virtualization_enabled() {
            if index >= 0 && (index as usize) < self.state_machines_by_index.len() {
                return unsafe { self.state_machines_by_index[index as usize].as_mut() };
            }
            return None;
        }
        if index >= 0 && (index as usize) < self.list_items.len() {
            let item = self.list_items[index as usize].clone();
            return self.state_machines_map.get_mut(&item).map(Box::as_mut);
        }
        None
    }

    #[cfg(feature = "rive_layout")]
    pub fn layout_node(&mut self, index: i32) -> *mut c_void {
        self.artboard_instance(index)
            .map_or(std::ptr::null_mut(), |artboard| {
                &mut artboard.take_layout_data().node as *mut _ as *mut c_void
            })
    }

    pub fn mark_layout_node_dirty(&mut self, _should_force_update_layout_bounds: bool) {
        let parent_is_row = self.main_axis_is_row();
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.parent_is_row(parent_is_row);
            }
        }
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        #[cfg(feature = "rive_layout")]
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.update_layout_bounds(animate);
                let bounds = artboard.layout_bounds();
                self.set_item_size(Vec2D::new(bounds.width(), bounds.height()), index);
            }
        }
        self.compute_layout_bounds();
    }

    #[cfg(feature = "rive_layout")]
    pub fn cascade_layout_style(
        &mut self,
        inherited_interpolation: LayoutStyleInterpolation,
        inherited_interpolator: Option<&mut KeyFrameInterpolator>,
        inherited_interpolation_time: f32,
        direction: LayoutDirection,
    ) -> bool {
        let interpolator = inherited_interpolator.map(|value| value as *mut _);
        for index in 0..self.artboard_count() as i32 {
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.cascade_layout_style(
                    inherited_interpolation,
                    interpolator.map(|value| unsafe { &mut *value }),
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

    pub fn find_artboard(&mut self, list_item: &Rcp<ViewModelInstanceListItem>) -> *mut Artboard {
        let Some(view_model_instance) = list_item.view_model_instance() else {
            return std::ptr::null_mut();
        };
        let view_model_id = view_model_instance.view_model_id();
        if let Some(artboard) = self.artboards_map.get(&view_model_id) {
            return *artboard;
        }
        let artboards = unsafe { &mut *self.file }.artboards();
        if let Some(artboard_index) = self.artboard_map_rules.get(&(view_model_id as i32)) {
            if *artboard_index >= 0 && (*artboard_index as usize) < artboards.len() {
                let artboard = artboards[*artboard_index as usize];
                self.artboards_map.insert(view_model_id, artboard);
                return artboard;
            }
        }
        for artboard in artboards {
            if unsafe { &*artboard }.view_model_id() == view_model_id {
                self.artboards_map.insert(view_model_id, artboard);
                return artboard;
            }
        }
        std::ptr::null_mut()
    }

    fn dispose_list_item(&mut self, list_item: &Rcp<ViewModelInstanceListItem>) {
        self.remove_artboard(list_item.clone());
    }

    fn create_artboard(
        &mut self,
        _target: *mut Component,
        list_item: Rcp<ViewModelInstanceListItem>,
    ) -> Option<Box<ArtboardInstance>> {
        let artboard = self.find_artboard(&list_item);
        if !artboard.is_null() {
            return unsafe { &*artboard }.instance();
        }
        None
    }

    fn create_state_machine_instance(
        &mut self,
        _target: *mut Component,
        artboard: *mut ArtboardInstance,
    ) -> Option<Box<StateMachineInstance>> {
        let artboard = unsafe { artboard.as_mut()? };
        let default_index = artboard.base.default_state_machine_index();
        let state_machine_index = if default_index >= 0 {
            default_index as usize
        } else {
            0
        };
        let mut instance = artboard.state_machine_at(state_machine_index);
        self.link_state_machine_to_artboard(
            instance
                .as_deref_mut()
                .map_or(std::ptr::null_mut(), |value| value),
            artboard,
        );
        instance
    }

    pub fn ensure_list_scope_focus_node(
        &mut self,
        focus_manager: *mut FocusManager,
        host_parent: Option<FocusNodeRef>,
    ) {
        let Some(focus_manager) = (unsafe { focus_manager.as_mut() }) else {
            return;
        };
        if self.list_scope_focus_node.is_none() {
            let node = FocusNode::make_structural_scope();
            node.borrow_mut().name = "ArtboardComponentListScope".to_owned();
            self.list_scope_focus_node = Some(node);
        }
        focus_manager.add_child(
            host_parent,
            self.list_scope_focus_node.clone().unwrap(),
            None,
        );
        self.sync_list_row_nodes_with_list(focus_manager);
    }

    pub fn list_scope_focus_node(&self) -> Option<FocusNodeRef> {
        self.list_scope_focus_node.clone()
    }

    pub fn remove_list_scope_focus_node(&mut self) {
        let focus_manager = self
            .component_mut()
            .artboard_mut()
            .and_then(Artboard::focus_manager_mut)
            .map_or(std::ptr::null_mut(), |value| value);
        for row in &mut self.list_row_focus_nodes {
            let Some(node) = row.take() else {
                continue;
            };
            if let Some(parent) = node.borrow().parent() {
                FocusNode::remove_child(&parent, &node);
            } else if !focus_manager.is_null() {
                unsafe { &mut *focus_manager }.remove_child(&node);
            }
        }
        self.list_row_focus_nodes.clear();
        let Some(node) = self.list_scope_focus_node.take() else {
            return;
        };
        if let Some(parent) = node.borrow().parent() {
            FocusNode::remove_child(&parent, &node);
        } else if !focus_manager.is_null() {
            unsafe { &mut *focus_manager }.remove_child(&node);
        }
    }

    fn make_list_row_focus_node(&self) -> FocusNodeRef {
        let node = FocusNode::make_structural_scope();
        node.borrow_mut().name = "ArtboardComponentListRow".to_owned();
        node
    }

    fn reparent_list_rows_in_scope(&mut self, focus_manager: &mut FocusManager) {
        let Some(scope) = self.list_scope_focus_node.clone() else {
            return;
        };
        for row in self.list_row_focus_nodes.iter().flatten() {
            if let Some(parent) = row.borrow().parent() {
                FocusNode::remove_child(&parent, row);
            }
        }
        for (index, row) in self.list_row_focus_nodes.iter().enumerate() {
            if let Some(row) = row {
                focus_manager.add_child(Some(scope.clone()), row.clone(), Some(index));
            }
        }
    }
}

fn artboard_has_focus_content(artboard: *mut Artboard) -> bool {
    let Some(artboard) = (unsafe { artboard.as_mut() }) else {
        return false;
    };
    if artboard.root_focus_data_count() > 0 {
        return true;
    }
    for host in artboard.nested_artboards() {
        let Some(host) = (unsafe { host.as_mut() }) else {
            continue;
        };
        if host.is_artboard_data_bound() {
            return true;
        }
        if artboard_has_focus_content(
            host.artboard_instance(0)
                .map_or(std::ptr::null_mut(), |value| value),
        ) {
            return true;
        }
    }
    for list in artboard.artboard_component_lists() {
        if !list.is_null() {
            return true;
        }
    }
    false
}

impl ArtboardComponentList {
    fn list_item_needs_build_under_row(
        &self,
        parent_focus_manager: *mut FocusManager,
        instance: *mut ArtboardInstance,
        row: Option<FocusNodeRef>,
    ) -> bool {
        let Some(instance) = (unsafe { instance.as_ref() }) else {
            return false;
        };
        let Some(row) = row else {
            return false;
        };
        if parent_focus_manager.is_null() {
            return false;
        }
        if instance
            .focus_manager()
            .map_or(std::ptr::null(), |value| value)
            != parent_focus_manager
        {
            return true;
        }
        if row.borrow().children().is_empty()
            && artboard_has_focus_content(instance as *const _ as *mut Artboard)
        {
            return true;
        }
        false
    }

    fn sync_list_row_nodes_with_list(&mut self, focus_manager: &mut FocusManager) {
        if self.list_items.is_empty() {
            while let Some(row) = self.list_row_focus_nodes.pop() {
                if let Some(row) = row {
                    focus_manager.remove_child(&row);
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
        focus_manager: &mut FocusManager,
        previous_list_items: &[Rcp<ViewModelInstanceListItem>],
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
                focus_manager.remove_child(&unmapped);
            }
        }
        self.list_row_focus_nodes = new_rows;
        for index in 0..count {
            if self.list_row_focus_nodes[index].is_none() {
                self.list_row_focus_nodes[index] = Some(self.make_list_row_focus_node());
            }
        }
        self.reparent_list_rows_in_scope(focus_manager);
        for index in 0..count {
            let instance = self
                .artboard_instance(index as i32)
                .map_or(std::ptr::null_mut(), |value| value);
            if instance.is_null() {
                continue;
            }
            let row = self.list_row_focus_nodes[index].clone();
            if row.is_none() {
                continue;
            }
            let state_machine = self
                .state_machine_instance(index as i32)
                .map_or(std::ptr::null_mut(), |value| value);
            if !state_machine.is_null()
                && unsafe { &*state_machine }
                    .focus_manager()
                    .map_or(std::ptr::null(), |value| value)
                    != focus_manager
            {
                unsafe { &mut *state_machine }.set_external_focus_manager(focus_manager);
            }
            if self.list_item_needs_build_under_row(focus_manager, instance, row.clone()) {
                let instance = unsafe { &mut *instance };
                if instance.focus_manager().is_some() {
                    instance.cleanup_focus_tree();
                }
                instance.build_focus_tree(focus_manager, row);
            }
        }
    }

    fn link_state_machine_to_artboard(
        &mut self,
        state_machine_instance: *mut StateMachineInstance,
        artboard_instance: *mut ArtboardInstance,
    ) {
        let (Some(state_machine_instance), Some(artboard_instance)) =
            (unsafe { (state_machine_instance.as_mut(), artboard_instance.as_mut()) })
        else {
            return;
        };
        if let Some(data_context) = artboard_instance.base.data_context() {
            state_machine_instance.data_context(data_context);
            state_machine_instance.update_data_binds(false);
        }
        let parent_artboard = self.component_mut().artboard_mut();
        if let Some(parent_artboard) = parent_artboard {
            if let Some(parent_focus_manager) = parent_artboard.focus_manager_mut() {
                state_machine_instance.set_external_focus_manager(parent_focus_manager);
            }
            if let Some(parent_semantic_manager) = parent_artboard.semantic_manager_mut() {
                let parent_node = SemanticData::find_closest_semantic_node(self.component_mut());
                state_machine_instance
                    .set_external_semantic_manager(parent_semantic_manager, parent_node);
            }
        }
    }

    fn lists_are_equal(
        list: Option<&[Rcp<ViewModelInstanceListItem>]>,
        compared: Option<&[Rcp<ViewModelInstanceListItem>]>,
    ) -> bool {
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

    pub fn update_list(&mut self, list: &[Rcp<ViewModelInstanceListItem>]) {
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
                .resize(self.list_items.len(), std::ptr::null_mut());
            self.state_machines_by_index
                .resize(self.list_items.len(), std::ptr::null_mut());
        }
        if let Some(parent) = self.layout_parent() {
            #[cfg(feature = "rive_layout")]
            parent.clear_layout_children();
        }
        for item in self.old_items.clone() {
            if !self.list_items.contains(&item) {
                self.dispose_list_item(&item);
            }
        }
        for index in 0..self.list_items.len() {
            let item = self.list_items[index].clone();
            if let Some(mut view_model_instance) = item.view_model_instance() {
                if let Some(symbol) =
                    view_model_instance.property_value_for_symbol(SymbolType::ItemIndex)
                {
                    unsafe {
                        symbol
                            .as_ptr()
                            .cast::<ViewModelInstanceSymbolListIndex>()
                            .as_mut()
                    }
                    .unwrap()
                    .set_property_value(index as u32);
                }
            }
            let artboard = self.find_artboard(&item);
            if !artboard.is_null() {
                let artboard = unsafe { &*artboard };
                self.artboard_sizes
                    .push(Vec2D::new(artboard.width(), artboard.height()));
            }
            if !self.virtualization_enabled() {
                if !self.artboard_instances_map.contains_key(&item) {
                    self.create_artboard_at(index as i32, false);
                } else {
                    self.artboard_instances_by_index[index] = self
                        .artboard_instances_map
                        .get_mut(&item)
                        .map_or(std::ptr::null_mut(), |value| value.as_mut());
                    if let Some(state_machine) = self.state_machines_map.get_mut(&item) {
                        self.state_machines_by_index[index] = state_machine.as_mut();
                    }
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
            .component_mut()
            .artboard_mut()
            .and_then(Artboard::focus_manager_mut)
            .map_or(std::ptr::null_mut(), |value| value);
        if !focus_manager.is_null() && self.list_scope_focus_node.is_some() {
            self.sync_list_row_nodes_with_previous(
                unsafe { &mut *focus_manager },
                &previous_list_items,
                &previous_row_nodes,
            );
        }
    }

    pub fn sync_layout_children(&mut self) {
        if let Some(parent) = self.layout_parent() {
            #[cfg(feature = "rive_layout")]
            parent.sync_layout_children();
        }
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
                    if !new_frame {
                        if state_machine.try_change_state()
                            && state_machine.advance(elapsed_seconds, new_frame)
                        {
                            keep_going = true;
                        }
                    } else if state_machine.advance(elapsed_seconds, new_frame) {
                        keep_going = true;
                    }
                }
            }
            if let Some(artboard) = self.artboard_instance(index) {
                if artboard.advance_internal(elapsed_seconds, advancing_flags) {
                    keep_going = true;
                }
                if artboard.base.has_dirt(ComponentDirt::COMPONENTS) {
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
                if let Some(mut view_model_instance) = item.view_model_instance() {
                    view_model_instance.advanced();
                    if let Some(artboard) = self.artboard_instances_map.get_mut(&item) {
                        if let Some(data_context) = artboard.base.data_context() {
                            if let Some(bound_instance) = data_context.main_view_model_instance() {
                                if bound_instance != view_model_instance {
                                    bound_instance.advanced();
                                }
                            }
                        }
                    }
                }
            }
            if let Some(artboard) = self.artboard_instances_map.get_mut(&item) {
                artboard.reset();
            }
        }
    }

    pub fn layout_bounds(&self) -> Aabb {
        Aabb::new(0.0, 0.0, self.layout_size.x, self.layout_size.y)
    }

    pub fn layout_bounds_for_node(&mut self, index: usize) -> Aabb {
        if self.virtualization_enabled() {
            let real_index = index % self.list_items.len();
            let gap = self.gap();
            let mut running_size = 0.0;
            let is_horizontal = self.main_axis_is_row();
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
                return artboard.layout_bounds();
            }
        }
        Aabb::default()
    }

    pub fn mark_hosting_layout_dirty(&mut self, artboard_instance: *mut ArtboardInstance) {
        for index in 0..self.artboard_count() as i32 {
            let artboard = self
                .artboard_instance(index)
                .map_or(std::ptr::null_mut(), |value| value);
            if artboard == artboard_instance {
                if let Some(parent) = self.component_mut().artboard_mut() {
                    parent.mark_layout_dirty(unsafe { &mut *artboard_instance });
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
            let Some(mut instance) = item.view_model_instance() else {
                continue;
            };
            if instance.view_model_ref().is_some_and(|view_model| {
                view_model
                    .property_for_symbol(SymbolType::DrawIndex)
                    .is_some()
            }) {
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
        let Some(mut instance) = self.list_items[index as usize].view_model_instance() else {
            return 0.0;
        };
        if instance.view_model_ref().is_none()
            || instance
                .view_model_ref()
                .unwrap()
                .property_for_symbol(SymbolType::DrawIndex)
                .is_none()
        {
            return 0.0;
        }
        if let Some(value) = instance.property_value_for_symbol(SymbolType::DrawIndex) {
            let number = unsafe { value.as_ptr().cast::<ViewModelInstanceNumber>().as_ref() };
            if let Some(number) = number {
                let value = number.property_value();
                if value.is_finite() {
                    return value;
                }
            }
        }
        0.0
    }

    fn clear_draw_index_listeners(&mut self) {
        self.draw_index_dependents.clear();
    }

    fn remove_draw_index_listener_for_item(&mut self, item: &Rcp<ViewModelInstanceListItem>) {
        self.draw_index_dependents.remove(item);
    }

    fn sync_draw_index_listeners(&mut self) {
        self.clear_draw_index_listeners();
        if !self.list_uses_draw_index_sort {
            return;
        }
        for item in self.list_items.clone() {
            let Some(mut instance) = item.view_model_instance() else {
                continue;
            };
            let Some(value) = instance.property_value_for_symbol(SymbolType::DrawIndex) else {
                continue;
            };
            self.draw_index_dependents.insert(
                item,
                ArtboardListDrawIndexDependent::new(self, value.as_ptr()),
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
            let this = self as *const Self;
            self.cached_ordered_list_indices.sort_by(|left, right| {
                let left_value = unsafe { &*this }.list_item_draw_index(*left);
                let right_value = unsafe { &*this }.list_item_draw_index(*right);
                left_value
                    .partial_cmp(&right_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| left.cmp(right))
            });
        }
        self.ordered_list_indices_cache_valid = true;
    }

    pub fn ordered_list_indices(&mut self) -> &[i32] {
        self.ensure_ordered_list_indices();
        &self.cached_ordered_list_indices
    }
}

impl ArtboardComponentList {
    pub fn draw(&mut self, renderer: &mut dyn Renderer) {
        if self.drawable().needs_save_operation() {
            renderer.save();
        }
        if self.virtualization_enabled() {
            if let Some(parent) = self.component().parent() {
                if let Some(parent) = parent.as_ref::<LayoutComponent>() {
                    renderer.transform(parent.world_transform());
                }
            }
            if self.visible_start_index != -1 && self.visible_end_index != -1 {
                let indices = self.ordered_list_indices().to_vec();
                for index in indices {
                    let artboard = self
                        .artboard_instance(index)
                        .map_or(std::ptr::null_mut(), |value| value);
                    if !artboard.is_null() {
                        renderer.save();
                        let transform = self.artboard_transforms[&artboard];
                        renderer.transform(&transform);
                        unsafe { &mut *artboard }.draw_internal(renderer);
                        renderer.restore();
                    }
                }
            }
        } else {
            let transform = *self.transform().world_transform();
            renderer.transform(&transform);
            let indices = self.ordered_list_indices().to_vec();
            for index in indices {
                let artboard = self
                    .artboard_instance(index)
                    .map_or(std::ptr::null_mut(), |value| value);
                if !artboard.is_null() {
                    renderer.save();
                    let transform = self.artboard_transforms[&artboard];
                    renderer.transform(&transform);
                    unsafe { &mut *artboard }.draw_internal(renderer);
                    renderer.restore();
                }
            }
        }
        if self.drawable().needs_save_operation() {
            renderer.restore();
        }
    }

    pub fn hit_test(&mut self, _hit_info: &mut HitInfo, _transform: &Mat2D) -> Option<&mut Core> {
        None
    }

    pub fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: *mut ArtboardInstance,
    ) -> bool {
        if artboard.is_null() {
            return false;
        }
        let bounds = self.artboard_position(artboard);
        let offset = Vec2D::new(bounds.x + position.x, bounds.y + position.y);
        let transform = if self.virtualization_enabled() {
            self.layout_parent()
                .map_or(Mat2D::identity(), |parent| *parent.world_transform())
        } else {
            *self.transform().world_transform()
        };
        self.component_mut().parent_mut().is_some_and(|parent| {
            parent.hit_test_point(&(transform * offset), skip_on_unclipped, false)
        })
    }

    pub fn host_transform_point(
        &self,
        vector: &Vec2D,
        artboard_instance: *mut ArtboardInstance,
    ) -> Vec2D {
        let bounds = self.artboard_transforms[&artboard_instance];
        let offset = Vec2D::new(bounds[4] + vector.x, bounds[5] + vector.y);
        let transform = if self.virtualization_enabled_ref() {
            self.component()
                .parent()
                .and_then(|parent| parent.as_ref::<LayoutComponent>())
                .map_or(Mat2D::identity(), |parent| *parent.world_transform())
        } else {
            *self.transform().world_transform()
        };
        let local = transform * offset;
        self.component()
            .artboard()
            .map_or(local, |artboard| artboard.root_transform(local))
    }

    pub fn world_transform_for_artboard(&self, artboard_instance: *mut ArtboardInstance) -> Mat2D {
        let offset = self.artboard_transforms[&artboard_instance];
        let position = Vec2D::new(offset[4], offset[5]);
        let parent_layout = self
            .component()
            .parent()
            .and_then(|parent| parent.as_ref::<LayoutComponent>());
        if let Some(parent_layout) = parent_layout {
            let bounds = self.layout_bounds();
            let transform = *parent_layout.world_transform()
                * Mat2D::from_translate(bounds.left(), bounds.top());
            return transform * Mat2D::from_translate(position.x, position.y);
        }
        let transform = if self.virtualization_enabled_ref() {
            self.component()
                .parent()
                .and_then(|parent| parent.as_ref::<LayoutComponent>())
                .map_or(Mat2D::identity(), |parent| *parent.world_transform())
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
                    artboard.mark_semantic_boundary_transform_dirty();
                }
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::RENDER_OPACITY) {
            let opacity = self.transform().render_opacity();
            for index in 0..self.artboard_count() as i32 {
                if let Some(artboard) = self.artboard_instance(index) {
                    artboard.opacity(opacity);
                }
            }
        }
        if Component::has_dirt_in(value, ComponentDirt::COMPONENTS) {
            for index in 0..self.artboard_count() as i32 {
                if let Some(artboard) = self.artboard_instance(index) {
                    artboard.update_pass(false);
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
            let use_layout = self.layout_parent().is_some();
            for index in 0..count {
                let artboard = self
                    .artboard_instance(index as i32)
                    .map_or(std::ptr::null_mut(), |value| value);
                if !artboard.is_null() {
                    let artboard_ref = unsafe { &mut *artboard };
                    let bounds = if use_layout {
                        artboard_ref.layout_bounds()
                    } else {
                        artboard_ref.world_bounds()
                    };
                    let origin = if use_layout {
                        artboard_ref.origin()
                    } else {
                        Vec2D::default()
                    };
                    self.artboard_transforms.insert(
                        artboard,
                        Mat2D::from_translate(bounds.left() - origin.x, bounds.top() - origin.y),
                    );
                }
            }
        }
    }

    pub fn update_constraints(&mut self) {
        let layout_constraints = self.provider_state.layout_constraints().to_vec();
        for parent_constraint in layout_constraints {
            unsafe { &mut *parent_constraint }.constrain_child(self);
        }
        if !self.constrainable_list_state.list_constraints.is_empty()
            && !self.virtualization_enabled()
        {
            let list_constraints = self.constrainable_list_state.list_constraints.clone();
            for list_constraint in list_constraints {
                unsafe { &mut *list_constraint }.constrain_list(self);
            }
        }
        let constraints = self.transform().constraints().to_vec();
        for constraint in constraints {
            if ListConstraint::from(unsafe { &mut *constraint }).is_some() {
                continue;
            }
            unsafe { &mut *constraint }.constrain(self.component_mut());
        }
    }

    pub fn internal_data_context(&mut self, value: Rc<DataContext>) {
        for artboard in self.artboard_instances_map.values_mut() {
            if let Some(data_context) = artboard.base.data_context() {
                data_context.set_parent(Some(value.clone()));
                artboard.internal_data_context(data_context);
            }
        }
        for state_machine in self.state_machines_map.values_mut() {
            if let Some(data_context) = state_machine.data_context() {
                data_context.set_parent(Some(value.clone()));
                state_machine.internal_data_context(data_context);
            }
        }
    }

    pub fn bind_view_model_instance(
        &mut self,
        _view_model_instance: Rc<ViewModelInstance>,
        _parent: Rc<DataContext>,
    ) {
    }

    pub fn clear_data_context(&mut self) {}

    pub fn unbind(&mut self) {
        self.clear();
    }

    pub fn update_data_binds(&mut self) {
        for index in 0..self.artboard_count() as i32 {
            if let Some(state_machine) = self.state_machine_instance(index) {
                state_machine.update_data_binds(false);
            }
            if let Some(artboard) = self.artboard_instance(index) {
                artboard.update_data_binds_default();
            }
        }
    }

    fn artboard_position(&self, artboard: *mut ArtboardInstance) -> Vec2D {
        let matrix = self.artboard_transforms[&artboard];
        Vec2D::new(matrix[4], matrix[5])
    }

    pub fn world_to_local(&mut self, world: Vec2D, local: &mut Vec2D, index: i32) -> bool {
        let artboard = self
            .artboard_instance(index)
            .map_or(std::ptr::null_mut(), |value| value);
        if artboard.is_null() {
            return false;
        }
        let offset = self.artboard_position(artboard);
        let transform = if self.virtualization_enabled() {
            self.layout_parent()
                .map_or(Mat2D::identity(), |parent| *parent.world_transform())
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

    pub fn set_file(&mut self, value: *mut File) {
        self.file = value;
    }

    pub fn file(&self) -> *mut File {
        self.file
    }

    pub fn clone_core(&self) -> Box<dyn Core> {
        let mut cloned = self.base.clone_into(&mut ArtboardComponentList::default());
        cloned.set_file(self.file());
        Box::new(cloned)
    }

    pub fn create_artboard_at(&mut self, index: i32, force_layout_sync: bool) {
        if let Some(item) = self.list_item(index) {
            let target = self.component_mut() as *mut Component;
            if let Some(mut artboard) = self.create_artboard(target, item.clone()) {
                self.attach_artboard_override(artboard.as_mut(), item);
                self.add_artboard_at(artboard, index, force_layout_sync);
            }
        }
    }

    pub fn add_artboard_at(
        &mut self,
        artboard: Box<ArtboardInstance>,
        index: i32,
        force_layout_sync: bool,
    ) {
        let Some(item) = self.list_item(index) else {
            return;
        };
        let artboard_instance = Box::into_raw(artboard);
        self.artboard_instances_map
            .insert(item.clone(), unsafe { Box::from_raw(artboard_instance) });
        self.bind_artboard(artboard_instance, item.clone());
        unsafe {
            (*artboard_instance).host(self);
            (*artboard_instance).frame_origin(false);
            (*artboard_instance).parent_is_row(self.main_axis_is_row());
        }
        if force_layout_sync {
            self.sync_layout_children();
        }
        let mut state_machine_instance = std::ptr::null_mut();
        let artboard = self.find_artboard(&item);
        if !artboard.is_null() {
            let pool = self.state_machines_pool.entry(artboard).or_default();
            if let Some(mut state_machine) = pool.pop() {
                state_machine.reset_state();
                self.apply_recorders_to_state_machine(state_machine.as_mut(), unsafe {
                    &*artboard
                });
                state_machine_instance = state_machine.as_mut();
                self.state_machines_map.insert(item.clone(), state_machine);
                self.link_state_machine_to_artboard(state_machine_instance, artboard_instance);
            }
        }
        if state_machine_instance.is_null() {
            let target = self.component_mut() as *mut Component;
            let mut state_machine = self.create_state_machine_instance(target, artboard_instance);
            state_machine_instance = state_machine
                .as_deref_mut()
                .map_or(std::ptr::null_mut(), |value| value);
            if let Some(state_machine) = state_machine {
                self.state_machines_map.insert(item.clone(), state_machine);
            }
        }
        if !self.virtualization_enabled() {
            if index as usize >= self.artboard_instances_by_index.len() {
                self.artboard_instances_by_index
                    .resize(index as usize + 1, std::ptr::null_mut());
                self.state_machines_by_index
                    .resize(index as usize + 1, std::ptr::null_mut());
            }
            self.artboard_instances_by_index[index as usize] = artboard_instance;
            self.state_machines_by_index[index as usize] = state_machine_instance;
        }
    }

    fn bind_artboard(
        &mut self,
        artboard_instance: *mut ArtboardInstance,
        list_item: Rcp<ViewModelInstanceListItem>,
    ) {
        let Some(artboard_instance) = (unsafe { artboard_instance.as_mut() }) else {
            return;
        };
        let data_context = self.component().artboard().and_then(Artboard::data_context);
        if let Some(view_model_instance) = list_item.view_model_instance() {
            artboard_instance
                .bind_view_model_instance_with_parent(view_model_instance, data_context);
            artboard_instance.update_data_binds_default();
            self.invalidate_ordered_list_indices_cache();
        }
    }

    pub fn remove_artboard_at(&mut self, index: i32) {
        if !self.virtualization_enabled()
            && index >= 0
            && (index as usize) < self.artboard_instances_by_index.len()
        {
            self.artboard_instances_by_index[index as usize] = std::ptr::null_mut();
            self.state_machines_by_index[index as usize] = std::ptr::null_mut();
        }
        if let Some(item) = self.list_item(index) {
            self.remove_artboard(item);
        }
    }

    pub fn remove_artboard(&mut self, item: Rcp<ViewModelInstanceListItem>) {
        self.invalidate_ordered_list_indices_cache();
        self.remove_draw_index_listener_for_item(&item);
        let artboard = self
            .artboard_instances_map
            .get_mut(&item)
            .map_or(std::ptr::null_mut(), |value| value.as_mut());
        if !artboard.is_null() {
            unsafe {
                (*artboard).cleanup_semantic_tree();
                (*artboard).cleanup_focus_tree();
            }
            self.clear_artboard_override(artboard);
        }
        self.state_machines_map.remove(&item);
        self.artboard_instances_map.remove(&item);
    }

    fn create_artboard_recorders(&mut self, artboard: *const Artboard) {
        let Some(artboard_ref) = (unsafe { artboard.as_ref() }) else {
            return;
        };
        if !self.property_recorders_map.contains_key(&artboard) {
            let mut recorder = Box::new(PropertyRecorder::default());
            recorder.record_artboard(artboard_ref);
            self.property_recorders_map.insert(artboard, recorder);
            for nested_artboard in artboard_ref.nested_artboards() {
                let Some(nested_artboard) = (unsafe { nested_artboard.as_ref() }) else {
                    continue;
                };
                self.create_artboard_recorders(nested_artboard.source_artboard());
            }
        }
    }

    fn apply_recorders_to_artboard(
        &self,
        artboard: *mut Artboard,
        source_artboard: *const Artboard,
    ) {
        let Some(artboard) = (unsafe { artboard.as_mut() }) else {
            return;
        };
        if let Some(recorder) = self.property_recorders_map.get(&source_artboard) {
            recorder.apply_artboard(artboard);
        }
        for nested_artboard in artboard.nested_artboards() {
            let Some(nested_artboard) = (unsafe { nested_artboard.as_mut() }) else {
                continue;
            };
            let nested_instance = nested_artboard.source_artboard();
            if nested_instance.is_null() {
                continue;
            }
            unsafe {
                self.apply_recorders_to_artboard(
                    nested_instance,
                    (*nested_instance).artboard_source() as *const Artboard,
                );
            }
        }
    }

    fn apply_recorders_to_state_machine(
        &self,
        state_machine_instance: &mut StateMachineInstance,
        source_artboard: &Artboard,
    ) {
        let source = source_artboard as *const Artboard;
        if let Some(recorder) = self.property_recorders_map.get(&source) {
            recorder.apply_state_machine(state_machine_instance);
        }
    }
}

impl ArtboardComponentList {
    pub fn add_virtualizable(&mut self, index: i32) {
        let Some(list_item) = self.list_item(index) else {
            return;
        };
        let artboard = self.find_artboard(&list_item);
        if artboard.is_null() {
            return;
        }
        self.create_artboard_recorders(artboard);
        let pooled = self.resource_pool.entry(artboard).or_default().pop();
        if let Some(mut pooled_artboard) = pooled {
            self.apply_recorders_to_artboard(
                pooled_artboard.as_mut() as *mut ArtboardInstance as *mut Artboard,
                artboard,
            );
            self.add_artboard_at(pooled_artboard, index, true);
        } else {
            self.create_artboard_at(index, true);
        }
        self.component_mut()
            .add_dirt(ComponentDirt::COMPONENTS, true);
        if let Some(parent) = self.layout_parent() {
            parent.mark_layout_style_dirty();
        }
    }

    pub fn virtualizable_changed(&mut self) {
        let focus_manager = self
            .component_mut()
            .artboard_mut()
            .and_then(Artboard::focus_manager_mut)
            .map_or(std::ptr::null_mut(), |value| value);
        if !focus_manager.is_null() && self.list_scope_focus_node.is_some() {
            self.sync_list_row_nodes_with_list(unsafe { &mut *focus_manager });
        }
    }

    pub fn remove_virtualizable(&mut self, index: i32) {
        if let Some(list_item) = self.list_item(index) {
            let artboard = self.find_artboard(&list_item);
            let artboard_instance = self.artboard_instances_map.remove(&list_item);
            if !artboard.is_null() {
                if let Some(artboard_instance) = artboard_instance {
                    self.resource_pool
                        .entry(artboard)
                        .or_default()
                        .push(artboard_instance);
                }
            }
            if let Some(state_machine) = self.state_machines_map.remove(&list_item) {
                self.state_machines_pool
                    .entry(artboard)
                    .or_default()
                    .push(state_machine);
            }
        }
        self.remove_artboard_at(index);
    }

    pub fn set_virtualizable_position(&mut self, index: i32, position: Vec2D) {
        let use_layout = self.layout_parent().is_some();
        if let Some(artboard) = self.artboard_instance(index) {
            let artboard_pointer = artboard as *mut ArtboardInstance;
            let origin = if use_layout {
                artboard.origin()
            } else {
                Vec2D::default()
            };
            self.artboard_transforms.insert(
                artboard_pointer,
                Mat2D::from_translate(position.x - origin.x, position.y - origin.y),
            );
        }
    }

    fn virtualization_enabled_ref(&self) -> bool {
        self.scroll_constraint_ref()
            .is_some_and(|virtualizer| virtualizer.virtualize())
    }

    pub fn virtualization_enabled(&mut self) -> bool {
        self.scroll_constraint()
            .is_some_and(|virtualizer| virtualizer.virtualize())
    }

    fn scroll_constraint_ref(&self) -> Option<&ScrollConstraint> {
        for parent_constraint in self.provider_state.layout_constraints() {
            let constraint = unsafe { &**parent_constraint }.constraint();
            if let Some(scroll) = constraint.as_scroll_constraint() {
                return Some(scroll);
            }
        }
        None
    }

    pub fn scroll_constraint(&mut self) -> Option<&mut ScrollConstraint> {
        for parent_constraint in self.provider_state.layout_constraints() {
            let constraint = unsafe { &mut **parent_constraint }.constraint_mut();
            if let Some(scroll) = constraint.as_scroll_constraint_mut() {
                return Some(scroll);
            }
        }
        None
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
            if let Some(scroll) = self.scroll_constraint() {
                scroll.constrain_virtualized(true);
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
        let is_row = self.main_axis_is_row();
        self.layout_parent().map_or(0.0, |parent| {
            if is_row {
                parent.gap_horizontal()
            } else {
                parent.gap_vertical()
            }
        })
    }

    fn attach_artboard_override(
        &mut self,
        instance: &mut ArtboardInstance,
        list_item: Rcp<ViewModelInstanceListItem>,
    ) {
        let Some(view_model_instance) = list_item.view_model_instance() else {
            return;
        };
        let artboards = unsafe { &mut *self.file }.artboards();
        let mut artboard_index = -1;
        for artboard in &artboards {
            artboard_index += 1;
            if unsafe { &**artboard }.view_model_id() == view_model_instance.view_model_id() {
                break;
            }
        }
        if artboard_index < 0 && artboard_index as usize >= artboards.len() {
            return;
        }
        let mut artboard_override = std::ptr::null_mut();
        let children = self.container_mut().children().to_vec();
        for child in children {
            let Some(child) = (unsafe { child.as_mut() }) else {
                continue;
            };
            if let Some(candidate) = child.as_mut::<ArtboardComponentListOverride>() {
                if candidate.base.artboard_id() == -1 {
                    artboard_override = candidate;
                } else if candidate.base.artboard_id() == artboard_index {
                    artboard_override = candidate;
                    break;
                }
            }
        }
        if let Some(artboard_override) = unsafe { artboard_override.as_mut() } {
            artboard_override.add_artboard(instance);
        }
    }

    fn clear_artboard_override(&mut self, artboard_instance: *mut ArtboardInstance) {
        let children = self.container_mut().children().to_vec();
        for child in children {
            let Some(child) = (unsafe { child.as_mut() }) else {
                continue;
            };
            if let Some(artboard_override) = child.as_mut::<ArtboardComponentListOverride>() {
                if let Some(artboard_instance) = unsafe { artboard_instance.as_mut() } {
                    artboard_override.remove_artboard(artboard_instance);
                }
            }
        }
    }

    pub fn main_axis_is_row(&mut self) -> bool {
        self.layout_parent()
            .map_or(true, |parent| parent.main_axis_is_row())
    }

    pub fn layout_parent(&mut self) -> Option<&mut LayoutComponent> {
        self.component_mut()
            .parent_mut()
            .and_then(|parent| parent.as_mut::<LayoutComponent>())
    }

    pub fn list_transform(&self) -> &Mat2D {
        self.transform().world_transform()
    }

    pub fn list_item_transforms<'a>(&'a mut self, transforms: &mut Vec<&'a mut Mat2D>) {
        let count = self.list_items.len();
        for index in 0..count {
            let artboard = self
                .artboard_instance(index as i32)
                .map_or(std::ptr::null_mut(), |value| value);
            if !artboard.is_null() {
                let transform = self.artboard_transforms.get_mut(&artboard).unwrap() as *mut Mat2D;
                transforms.push(unsafe { &mut *transform });
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

    pub fn item(&mut self, index: i32) -> Option<&mut dyn Virtualizable> {
        self.artboard_instance(index)
            .map(|value| value as &mut dyn Virtualizable)
    }

    pub fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        Some(self.transform_mut())
    }

    pub fn transform_component(&self) -> Option<&TransformComponent> {
        Some(self.transform())
    }

    pub fn parent_artboard(&mut self) -> &mut Artboard {
        self.component_mut().artboard_mut().unwrap()
    }

    pub fn mark_host_transform_dirty(&mut self) {
        self.transform_mut().mark_transform_dirty();
    }

    pub fn host_component(&mut self) -> Option<&mut Component> {
        Some(self.component_mut())
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

    #[cfg(feature = "rive_layout")]
    fn layout_node(&mut self, index: i32) -> *mut c_void {
        ArtboardComponentList::layout_node(self, index)
    }

    fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        ArtboardComponentList::transform_component_mut(self)
    }

    fn transform_component(&self) -> Option<&TransformComponent> {
        ArtboardComponentList::transform_component(self)
    }

    fn layout_bounds(&self) -> Aabb {
        ArtboardComponentList::layout_bounds(self)
    }

    fn layout_bounds_for_node(&self, index: usize) -> Aabb {
        let this = self as *const Self as *mut Self;
        unsafe { &mut *this }.layout_bounds_for_node(index)
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

    #[cfg(feature = "rive_layout")]
    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<&mut KeyFrameInterpolator>,
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

    fn list_item_transforms<'a>(&'a mut self, transforms: &mut Vec<&'a mut Mat2D>) {
        ArtboardComponentList::list_item_transforms(self, transforms);
    }
}

impl VirtualizingComponent for ArtboardComponentList {
    fn virtualization_enabled(&self) -> bool {
        self.virtualization_enabled_ref()
    }

    fn item_count(&self) -> i32 {
        ArtboardComponentList::item_count(self)
    }

    fn item(&mut self, index: i32) -> Option<&mut dyn Virtualizable> {
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

    fn artboard_instance(&mut self, index: i32) -> Option<&mut ArtboardInstance> {
        ArtboardComponentList::artboard_instance(self, index)
    }

    fn internal_data_context(&mut self, data_context: Rc<DataContext>) {
        ArtboardComponentList::internal_data_context(self, data_context);
    }

    fn bind_view_model_instance(
        &mut self,
        view_model_instance: Rc<ViewModelInstance>,
        parent: Rc<DataContext>,
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

    fn mark_hosting_layout_dirty(&mut self, artboard_instance: *mut ArtboardInstance) {
        ArtboardComponentList::mark_hosting_layout_dirty(self, artboard_instance);
    }

    fn parent_artboard(&mut self) -> &mut Artboard {
        ArtboardComponentList::parent_artboard(self)
    }

    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: *mut ArtboardInstance,
    ) -> bool {
        ArtboardComponentList::hit_test_host(self, position, skip_on_unclipped, artboard)
    }

    fn host_transform_point(&self, position: &Vec2D, artboard: *mut ArtboardInstance) -> Vec2D {
        ArtboardComponentList::host_transform_point(self, position, artboard)
    }

    fn world_transform_for_artboard(&self, artboard: *mut ArtboardInstance) -> Mat2D {
        ArtboardComponentList::world_transform_for_artboard(self, artboard)
    }

    fn mark_host_transform_dirty(&mut self) {
        ArtboardComponentList::mark_host_transform_dirty(self);
    }

    fn is_layout_provider(&self) -> bool {
        true
    }

    fn set_file(&mut self, value: *mut File) {
        ArtboardComponentList::set_file(self, value);
    }

    fn file(&self) -> *mut File {
        ArtboardComponentList::file(self)
    }

    fn host_component(&mut self) -> Option<&mut Component> {
        ArtboardComponentList::host_component(self)
    }

    fn type_(&self) -> i32 {
        ArtboardComponentList::type_(self)
    }
}

impl DataBindListItemConsumer for ArtboardComponentList {
    fn update_list(
        &mut self,
        list: &Vec<Rc<dyn crate::mechanical_port::source::data_bind::data_values::data_value_list::ViewModelInstanceListItem>>,
    ) {
        let translated = list
            .iter()
            .map(|item| unsafe { ref_rcp(Rc::as_ptr(item) as *mut ViewModelInstanceListItem) })
            .collect::<Vec<_>>();
        ArtboardComponentList::update_list(self, &translated);
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
