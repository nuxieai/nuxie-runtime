#[cfg(test)]
use nuxie_binary::RuntimeObject;
use nuxie_binary::{FieldValue, RuntimeFile, StringValue};
use nuxie_schema::{
    FieldKind, bitmask_passthrough_by_key_in_hierarchy,
    core_registry_setter_field_kind_by_property_key, definition_by_name, definition_by_type_key,
    property_by_key_in_hierarchy,
};
use std::collections::BTreeSet;

use crate::bones::weight::RuntimeWeightState;
use crate::components::{ComponentHandle, DataBindHandle, GraphOrder, RuntimeComponent};

mod generated_objects {
    include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"));
}

use generated_objects::InstanceObjectStorage;

#[derive(Debug, Clone)]
pub struct InstanceSlot {
    pub local_id: usize,
    pub source_global_id: u32,
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstanceString {
    value: Option<String>,
    raw: Vec<u8>,
}

impl InstanceString {
    pub(crate) fn from_static(value: &'static str) -> Self {
        Self {
            value: Some(value.to_owned()),
            raw: value.as_bytes().to_vec(),
        }
    }

    pub(crate) fn from_string_value(value: &StringValue) -> Self {
        Self {
            value: value.value.clone(),
            raw: value.raw.clone(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.raw.as_slice()
    }
}

pub(crate) fn set_optional_field<T: PartialEq>(field: &mut Option<T>, value: T) -> bool {
    if field.as_ref().is_some_and(|current| current == &value) {
        return false;
    }
    *field = Some(value);
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectHandle(usize);

impl ObjectHandle {
    pub(crate) const fn local_id(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComponentAddress {
    Authored(ObjectHandle),
    PathComposer(ObjectHandle),
    TextVariationHelper {
        style: ObjectHandle,
        text: ComponentHandle,
    },
}

impl ComponentAddress {
    pub(crate) const fn object(self) -> ObjectHandle {
        match self {
            Self::Authored(object) | Self::PathComposer(object) => object,
            Self::TextVariationHelper { style, .. } => style,
        }
    }
}

#[derive(Debug)]
pub(crate) struct InstanceObjectArena {
    objects: Vec<Option<RuntimeObjectOccurrence>>,
    component_addresses: Vec<ComponentAddress>,
    authored_component_handles: Vec<ComponentHandle>,
    dependency_order: Vec<ComponentHandle>,
    root: Option<ComponentHandle>,
    clone_backend_initialization_pending: bool,
}

#[derive(Debug)]
struct RuntimeObjectOccurrence {
    generated: InstanceObjectStorage,
    component: Option<RuntimeComponent>,
    component_handle: Option<ComponentHandle>,
    path_composer: Option<RuntimeComponent>,
    path_composer_handle: Option<ComponentHandle>,
    text_variation_helper: Option<RuntimeComponent>,
    text_variation_helper_handle: Option<ComponentHandle>,
}

impl RuntimeObjectOccurrence {
    fn clone_without_runtime_links(&self) -> Self {
        Self {
            generated: self.generated.clone(),
            component: self
                .component
                .as_ref()
                .map(RuntimeComponent::clone_for_occurrence),
            component_handle: self.component_handle,
            path_composer: self
                .path_composer
                .as_ref()
                .map(RuntimeComponent::clone_for_occurrence),
            path_composer_handle: self.path_composer_handle,
            text_variation_helper: self
                .text_variation_helper
                .as_ref()
                .map(RuntimeComponent::clone_for_occurrence),
            text_variation_helper_handle: self.text_variation_helper_handle,
        }
    }
}

impl Clone for InstanceObjectArena {
    fn clone(&self) -> Self {
        Self {
            objects: self
                .objects
                .iter()
                .map(|occurrence| {
                    occurrence
                        .as_ref()
                        .map(RuntimeObjectOccurrence::clone_without_runtime_links)
                })
                .collect(),
            component_addresses: self.component_addresses.clone(),
            authored_component_handles: self.authored_component_handles.clone(),
            dependency_order: Vec::new(),
            root: self.root,
            clone_backend_initialization_pending: true,
        }
    }
}

impl InstanceObjectArena {
    pub(crate) fn object_handle(&self, local_id: usize) -> Option<ObjectHandle> {
        self.contains_object(local_id)
            .then_some(ObjectHandle(local_id))
    }

    pub(crate) fn from_slots(file: &RuntimeFile, slots: &[InstanceSlot]) -> Self {
        let mut objects = Vec::with_capacity(slots.len());
        objects.resize_with(slots.len(), || None);
        for slot in slots {
            if slot.local_id >= objects.len() {
                objects.resize_with(slot.local_id + 1, || None);
            }
            objects[slot.local_id] = file
                .object(slot.source_global_id as usize)
                .and_then(InstanceObjectStorage::from_runtime_object)
                .map(|generated| RuntimeObjectOccurrence {
                    generated,
                    component: None,
                    component_handle: None,
                    path_composer: None,
                    path_composer_handle: None,
                    text_variation_helper: None,
                    text_variation_helper_handle: None,
                });
        }
        Self {
            objects,
            component_addresses: Vec::new(),
            authored_component_handles: Vec::new(),
            dependency_order: Vec::new(),
            root: None,
            clone_backend_initialization_pending: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_runtime_objects(objects: Vec<Option<RuntimeObject>>) -> Self {
        Self {
            objects: objects
                .iter()
                .map(|object| {
                    object
                        .as_ref()
                        .and_then(InstanceObjectStorage::from_runtime_object)
                        .map(|generated| RuntimeObjectOccurrence {
                            generated,
                            component: None,
                            component_handle: None,
                            path_composer: None,
                            path_composer_handle: None,
                            text_variation_helper: None,
                            text_variation_helper_handle: None,
                        })
                })
                .collect(),
            component_addresses: Vec::new(),
            authored_component_handles: Vec::new(),
            dependency_order: Vec::new(),
            root: None,
            clone_backend_initialization_pending: false,
        }
    }

    fn object(&self, local_id: usize) -> Option<&InstanceObjectStorage> {
        Some(&self.objects.get(local_id)?.as_ref()?.generated)
    }

    fn object_mut(&mut self, local_id: usize) -> Option<&mut InstanceObjectStorage> {
        Some(&mut self.objects.get_mut(local_id)?.as_mut()?.generated)
    }

    pub(crate) fn contains_object(&self, local_id: usize) -> bool {
        self.objects.get(local_id).is_some_and(Option::is_some)
    }

    pub(crate) fn attach_component(
        &mut self,
        local_id: usize,
        component: RuntimeComponent,
    ) -> Option<ComponentHandle> {
        let occurrence = self.objects.get_mut(local_id)?.as_mut()?;
        if occurrence.component.is_some() {
            return None;
        }
        let handle = ComponentHandle::from_index(self.component_addresses.len());
        let is_artboard = component.type_name == "Artboard";
        occurrence.component = Some(component);
        occurrence.component_handle = Some(handle);
        self.component_addresses
            .push(ComponentAddress::Authored(ObjectHandle(local_id)));
        self.authored_component_handles.push(handle);
        if self.root.is_none() || is_artboard {
            self.root = Some(handle);
        }
        Some(handle)
    }

    pub(crate) fn attach_path_composer(
        &mut self,
        shape_local: usize,
        component: RuntimeComponent,
    ) -> Option<ComponentHandle> {
        let occurrence = self.objects.get_mut(shape_local)?.as_mut()?;
        if occurrence.path_composer.is_some() {
            return None;
        }
        let handle = ComponentHandle::from_index(self.component_addresses.len());
        occurrence.path_composer = Some(component);
        occurrence.path_composer_handle = Some(handle);
        self.component_addresses
            .push(ComponentAddress::PathComposer(ObjectHandle(shape_local)));
        Some(handle)
    }

    pub(crate) fn attach_text_variation_helper(
        &mut self,
        text_style_local: usize,
        component: RuntimeComponent,
    ) -> Option<ComponentHandle> {
        let style = self.component_handle(text_style_local)?;
        let text = self.component(style)?.parent?;
        let occurrence = self.objects.get_mut(text_style_local)?.as_mut()?;
        if occurrence.text_variation_helper.is_some() {
            return None;
        }
        let handle = ComponentHandle::from_index(self.component_addresses.len());
        occurrence.text_variation_helper = Some(component);
        occurrence.text_variation_helper_handle = Some(handle);
        self.component_addresses
            .push(ComponentAddress::TextVariationHelper {
                style: ObjectHandle(text_style_local),
                text,
            });
        Some(handle)
    }

    pub(crate) fn component(&self, handle: ComponentHandle) -> Option<&RuntimeComponent> {
        let address = *self.component_addresses.get(handle.index())?;
        let occurrence = self.objects.get(address.object().local_id())?.as_ref()?;
        match address {
            ComponentAddress::Authored(_) => occurrence.component.as_ref(),
            ComponentAddress::PathComposer(_) => occurrence.path_composer.as_ref(),
            ComponentAddress::TextVariationHelper { .. } => {
                occurrence.text_variation_helper.as_ref()
            }
        }
    }

    pub(crate) fn component_mut(
        &mut self,
        handle: ComponentHandle,
    ) -> Option<&mut RuntimeComponent> {
        let address = *self.component_addresses.get(handle.index())?;
        let occurrence = self
            .objects
            .get_mut(address.object().local_id())?
            .as_mut()?;
        match address {
            ComponentAddress::Authored(_) => occurrence.component.as_mut(),
            ComponentAddress::PathComposer(_) => occurrence.path_composer.as_mut(),
            ComponentAddress::TextVariationHelper { .. } => {
                occurrence.text_variation_helper.as_mut()
            }
        }
    }

    pub(crate) fn component_for_local(&self, local_id: usize) -> Option<&RuntimeComponent> {
        let occurrence = self.objects.get(local_id)?.as_ref()?;
        self.component(occurrence.component_handle?)
    }

    pub(crate) fn component_for_local_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut RuntimeComponent> {
        let handle = self.objects.get(local_id)?.as_ref()?.component_handle?;
        self.component_mut(handle)
    }

    pub(crate) fn component_handle(&self, local_id: usize) -> Option<ComponentHandle> {
        self.objects.get(local_id)?.as_ref()?.component_handle
    }

    pub(crate) fn component_handles(&self) -> &[ComponentHandle] {
        &self.authored_component_handles
    }

    pub(crate) fn component_local_id(&self, handle: ComponentHandle) -> Option<usize> {
        Some(self.address(handle)?.object().local_id())
    }

    pub(crate) fn address(&self, handle: ComponentHandle) -> Option<ComponentAddress> {
        self.component_addresses.get(handle.index()).copied()
    }

    pub(crate) fn path_composer_handle(&self, shape_local: usize) -> Option<ComponentHandle> {
        self.objects
            .get(shape_local)?
            .as_ref()?
            .path_composer_handle
    }

    pub(crate) fn text_variation_helper_handle(
        &self,
        text_style_local: usize,
    ) -> Option<ComponentHandle> {
        self.objects
            .get(text_style_local)?
            .as_ref()?
            .text_variation_helper_handle
    }

    pub(crate) fn relink_text_variation_helper_owner(
        &mut self,
        text_style_local: usize,
    ) -> Option<ComponentHandle> {
        let style = self.component_handle(text_style_local)?;
        let text = self.component(style)?.parent?;
        let helper = self.text_variation_helper_handle(text_style_local)?;
        *self.component_addresses.get_mut(helper.index())? =
            ComponentAddress::TextVariationHelper {
                style: ObjectHandle(text_style_local),
                text,
            };
        Some(helper)
    }

    pub(crate) fn text_variation_helper_text_handle(
        &self,
        text_style_local: usize,
    ) -> Option<ComponentHandle> {
        let helper = self.text_variation_helper_handle(text_style_local)?;
        match self.address(helper)? {
            ComponentAddress::TextVariationHelper { text, .. } => Some(text),
            _ => None,
        }
    }

    pub(crate) fn text_variation_helper_text(
        &self,
        helper: ComponentHandle,
    ) -> Option<ComponentHandle> {
        match self.address(helper)? {
            ComponentAddress::TextVariationHelper { text, .. } => Some(text),
            _ => None,
        }
    }

    pub(crate) fn root(&self) -> Option<ComponentHandle> {
        self.root
    }

    pub(crate) fn take_clone_backend_initialization_pending(&mut self) -> bool {
        std::mem::take(&mut self.clone_backend_initialization_pending)
    }

    pub(crate) fn link_parent(&mut self, child: ComponentHandle, parent: ComponentHandle) -> bool {
        if self.component(child).is_none() || self.component(parent).is_none() {
            return false;
        }
        self.component_mut(child)
            .expect("child handle was validated")
            .parent = Some(parent);
        let children = &mut self
            .component_mut(parent)
            .expect("parent handle was validated")
            .children;
        if !children.contains(&child) {
            children.push(child);
        }
        true
    }

    pub(crate) fn add_constraint(
        &mut self,
        owner: ComponentHandle,
        constraint: ComponentHandle,
    ) -> bool {
        let Some(component) = self.component_mut(owner) else {
            return false;
        };
        // C++ TransformComponent::addConstraint is an unconditional
        // insertion-order push. Construction invokes it once per occurrence;
        // retaining that literal behavior also makes duplicate lifecycle
        // invocation observable instead of silently normalizing it.
        component.constraints.push(constraint);
        true
    }

    pub(crate) fn add_dependent(
        &mut self,
        source: ComponentHandle,
        dependent: ComponentHandle,
    ) -> bool {
        if self.component(dependent).is_none() {
            return false;
        }
        let Some(component) = self.component_mut(source) else {
            return false;
        };
        if component.dependents.contains(&dependent) {
            return false;
        }
        component.dependents.push(dependent);
        true
    }

    /// C++ `Component::addCollapsable`: stable-unique insertion. The caller
    /// owns the first-insert collapse synchronization because it also owns
    /// the retained DataBind state.
    pub(crate) fn add_collapsable(
        &mut self,
        owner: ComponentHandle,
        data_bind: DataBindHandle,
    ) -> bool {
        let Some(component) = self.component_mut(owner) else {
            return false;
        };
        if component.collapsables.contains(&data_bind) {
            return false;
        }
        component.collapsables.push(data_bind);
        true
    }

    pub(crate) fn reset_component_relations(&mut self) {
        self.dependency_order.clear();
        let handles = self.component_addresses.len();
        for index in 0..handles {
            let handle = ComponentHandle::from_index(index);
            if let Some(component) = self.component_mut(handle) {
                component.parent = None;
                component.parent_transform = None;
                component.children.clear();
                component.constraints.clear();
                component.dependents.clear();
                component.collapsables.clear();
                component.layout_ancestors.clear();
                component.constrained_layout_ancestor = None;
                component.graph_order = None;
                if let Some(constraint) = component.concrete.constraint.as_mut() {
                    constraint.target = None;
                    constraint.scratch =
                        crate::components::RuntimeConstraintScratch::for_kind(constraint.kind);
                }
                if let Some(follow_path) = component.concrete.follow_path.as_mut() {
                    *follow_path = Default::default();
                }
                if let Some(ik) = component.concrete.ik.as_mut() {
                    *ik = Default::default();
                }
                if let Some(list) = component.concrete.constrainable_list.as_mut() {
                    *list = Default::default();
                }
                if let Some(path) = component.concrete.path.as_mut() {
                    *path = Default::default();
                }
                if let Some(shape) = component.concrete.shape.as_mut() {
                    *shape = Default::default();
                }
                if let Some(layout) = component.concrete.layout.as_mut() {
                    layout.style = None;
                }
                if let Some(solo) = component.concrete.solo.as_mut() {
                    solo.cpp_local_ids.clear();
                }
                if let Some(bone) = component.concrete.bone.as_mut() {
                    bone.child_bones.clear();
                    bone.peer_constraints.clear();
                }
                if let Some(skin) = component.concrete.skin.as_mut() {
                    *skin = Default::default();
                }
                if let Some(tendon) = component.concrete.tendon.as_mut() {
                    *tendon = Default::default();
                }
                if let Some(skinnable) = component.concrete.skinnable.as_mut() {
                    skinnable.skin = None;
                    skinnable.vertices.clear();
                }
                if let Some(weight) = component.concrete.weight.as_mut() {
                    let is_cubic = weight.cubic.is_some();
                    *weight = RuntimeWeightState::new(is_cubic);
                }
                if let Some(vertex) = component.concrete.vertex.as_mut() {
                    *vertex = Default::default();
                }
            }
        }
    }

    pub(crate) fn collapsable_len(&self, handle: ComponentHandle) -> usize {
        self.component(handle)
            .map_or(0, |component| component.collapsables.len())
    }

    pub(crate) fn collapsable_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<DataBindHandle> {
        self.component(handle)?.collapsables.get(index).copied()
    }

    pub(crate) fn dependent_len(&self, handle: ComponentHandle) -> usize {
        self.component(handle)
            .map_or(0, |component| component.dependents.len())
    }

    pub(crate) fn child_len(&self, handle: ComponentHandle) -> usize {
        self.component(handle)
            .map_or(0, |component| component.children.len())
    }

    pub(crate) fn child_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<ComponentHandle> {
        self.component(handle)?.children.get(index).copied()
    }

    pub(crate) fn constraint_len(&self, handle: ComponentHandle) -> usize {
        self.component(handle)
            .map_or(0, |component| component.constraints.len())
    }

    pub(crate) fn constraint_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<ComponentHandle> {
        self.component(handle)?.constraints.get(index).copied()
    }

    pub(crate) fn dependent_at(
        &self,
        handle: ComponentHandle,
        index: usize,
    ) -> Option<ComponentHandle> {
        self.component(handle)?.dependents.get(index).copied()
    }

    pub(crate) fn dependency_order(&self) -> &[ComponentHandle] {
        &self.dependency_order
    }

    pub(crate) fn scheduled_at(&self, index: usize) -> Option<ComponentHandle> {
        self.dependency_order.get(index).copied()
    }

    pub(crate) fn graph_order(&self, handle: ComponentHandle) -> Option<GraphOrder> {
        self.component(handle)?.graph_order
    }

    pub(crate) fn is_container_component(&self, handle: ComponentHandle) -> bool {
        self.component(handle)
            .and_then(|component| definition_by_name(component.type_name))
            .is_some_and(|definition| definition.is_a("ContainerComponent"))
    }

    pub(crate) fn set_dependency_order(&mut self, order: Vec<ComponentHandle>) {
        for index in 0..self.component_addresses.len() {
            if let Some(component) = self.component_mut(ComponentHandle::from_index(index)) {
                component.graph_order = None;
            }
        }
        self.dependency_order = order;
        let handles = self.dependency_order.clone();
        for (index, handle) in handles.into_iter().enumerate() {
            if let Some(component) = self.component_mut(handle) {
                component.graph_order = Some(GraphOrder::new(index));
            }
        }
    }

    pub(crate) fn sort_dependencies_from_root(&mut self) -> bool {
        let Some(root) = self.root else {
            self.set_dependency_order(Vec::new());
            return true;
        };
        let mut permanent = BTreeSet::new();
        let mut temporary = BTreeSet::new();
        let mut order = Vec::new();
        let complete = self.visit_dependency(root, &mut permanent, &mut temporary, &mut order);
        // Pinned `DependencySorter::sort` ignores `visit`'s cycle result and
        // publishes whatever order completed before the cycle
        // (`src/dependency_sorter.cpp:6-10`). Artboard initialization
        // continues with that partial schedule.
        self.set_dependency_order(order);
        complete
    }

    fn visit_dependency(
        &self,
        handle: ComponentHandle,
        permanent: &mut BTreeSet<ComponentHandle>,
        temporary: &mut BTreeSet<ComponentHandle>,
        order: &mut Vec<ComponentHandle>,
    ) -> bool {
        if permanent.contains(&handle) {
            return true;
        }
        if !temporary.insert(handle) {
            return false;
        }
        let dependent_count = self.dependent_len(handle);
        for index in 0..dependent_count {
            let Some(dependent) = self.dependent_at(handle, index) else {
                continue;
            };
            if !self.visit_dependency(dependent, permanent, temporary, order) {
                return false;
            }
        }
        temporary.remove(&handle);
        permanent.insert(handle);
        order.insert(0, handle);
        true
    }

    pub(crate) fn property_kind(&self, local_id: usize, property_key: u16) -> Option<FieldKind> {
        let object = self.object(local_id)?;
        runtime_property_metadata_by_key(object.type_key(), property_key)
            .map(|(_, property)| property.runtime_type)
    }

    pub(crate) fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.object(local_id)
            .and_then(|object| object.color_property(property_key))
    }

    pub(crate) fn solid_color_value(&self, local_id: usize) -> Option<u32> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::solid_color_value)
    }

    pub(crate) fn replace_solid_color_value(&mut self, local_id: usize, value: u32) -> Option<u32> {
        self.object_mut(local_id)?.replace_solid_color_value(value)
    }

    pub(crate) fn set_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Color(value))
    }

    pub(crate) fn set_generated_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        self.object_mut(local_id)
            .is_some_and(|object| object.set_color_property(property_key, value))
    }

    pub(crate) fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        self.object(local_id)
            .and_then(|object| object.bool_property(property_key))
    }

    pub(crate) fn shape_paint_is_visible(&self, local_id: usize) -> Option<bool> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::shape_paint_is_visible)
    }

    pub(crate) fn shape_paint_blend_mode_value(&self, local_id: usize) -> Option<u64> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::shape_paint_blend_mode_value)
    }

    pub(crate) fn fill_rule(&self, local_id: usize) -> Option<u64> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::fill_rule)
    }

    pub(crate) fn stroke_transform_affects_stroke(&self, local_id: usize) -> Option<bool> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::stroke_transform_affects_stroke)
    }

    pub(crate) fn stroke_thickness(&self, local_id: usize) -> Option<f32> {
        self.object(local_id)
            .and_then(InstanceObjectStorage::stroke_thickness)
    }

    pub(crate) fn set_bool_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Bool(value))
    }

    pub(crate) fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        let object = self.object(local_id)?;
        if let Some(bitmask) =
            bitmask_passthrough_by_key_in_hierarchy(object.type_key(), property_key)
        {
            let (_owner, target) =
                runtime_property_metadata_by_name(object.type_key(), bitmask.target)?;
            let packed = object.uint_property(target.key.int).unwrap_or(0);
            return Some((packed & bitmask_field_mask(bitmask.bit, bitmask.width)) >> bitmask.bit);
        }
        object.uint_property(property_key)
    }

    /// Read generated storage through the retained authored Component owner.
    /// This corresponds to invoking a generated C++ base getter on the same
    /// concrete object; no serialized local-id owner lookup is repeated.
    pub(crate) fn component_uint_property(
        &self,
        handle: ComponentHandle,
        property_key: u16,
    ) -> Option<u64> {
        let ComponentAddress::Authored(object) = self.address(handle)? else {
            return None;
        };
        self.uint_property(object.local_id(), property_key)
    }

    /// Read a generated bool through the retained authored Component owner.
    /// See [`Self::component_uint_property`].
    pub(crate) fn component_bool_property(
        &self,
        handle: ComponentHandle,
        property_key: u16,
    ) -> Option<bool> {
        let ComponentAddress::Authored(object) = self.address(handle)? else {
            return None;
        };
        self.bool_property(object.local_id(), property_key)
    }

    pub(crate) fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.object(local_id)
            .and_then(|object| object.double_property(property_key))
    }

    pub(crate) fn double_property_by_name(
        &self,
        local_id: usize,
        property_name: &str,
    ) -> Option<f32> {
        let object = self.object(local_id)?;
        let (_, property) = runtime_property_metadata_by_name(object.type_key(), property_name)?;
        object.double_property(property.key.int)
    }

    #[cfg(test)]
    pub(crate) fn set_double_property_by_name(
        &mut self,
        local_id: usize,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(type_key) = self.object(local_id).map(InstanceObjectStorage::type_key) else {
            return false;
        };
        let Some((_, property)) = runtime_property_metadata_by_name(type_key, property_name) else {
            return false;
        };
        self.set_double_property(local_id, property.key.int, value)
    }

    pub(crate) fn set_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Double(value))
    }

    pub(crate) fn set_generated_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.object_mut(local_id)
            .is_some_and(|object| object.set_double_property(property_key, value))
    }

    pub(crate) fn set_uint_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u64,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Uint(value))
    }

    pub(crate) fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        let object = self.object(local_id)?;
        match self.property_kind(local_id, property_key)? {
            FieldKind::String => object.string_property(property_key),
            FieldKind::Bytes => object.bytes_property(property_key),
            _ => None,
        }
    }

    pub(crate) fn set_string_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: Vec<u8>,
    ) -> bool {
        let Some(kind) = self.property_kind(local_id, property_key) else {
            return false;
        };
        let value = match kind {
            FieldKind::String => FieldValue::String(StringValue {
                value: String::from_utf8(value.clone()).ok(),
                raw: value,
            }),
            FieldKind::Bytes => return false,
            _ => return false,
        };
        self.set_property_value(local_id, property_key, value)
    }

    fn set_property_value(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: FieldValue,
    ) -> bool {
        let Some(type_key) = self.object(local_id).map(InstanceObjectStorage::type_key) else {
            return false;
        };
        let Some((_owner, property)) = runtime_property_metadata_by_key(type_key, property_key)
        else {
            return false;
        };
        let Some(setter_kind) = core_registry_setter_field_kind_by_property_key(property_key)
        else {
            return false;
        };
        if !field_value_matches_kind(&value, setter_kind) {
            return false;
        }
        if !field_value_matches_kind(&value, property.runtime_type) {
            return false;
        }

        if let (Some(bitmask), FieldValue::Uint(value)) = (property.bitmask_passthrough, &value) {
            let Some((_owner, target)) =
                runtime_property_metadata_by_name(type_key, bitmask.target)
            else {
                return false;
            };
            let Some(object) = self.object_mut(local_id) else {
                return false;
            };
            let mask = bitmask_field_mask(bitmask.bit, bitmask.width);
            let current = object.uint_property(target.key.int).unwrap_or(0);
            let shifted = value.checked_shl(bitmask.bit.into()).unwrap_or(0);
            let next = (current & !mask) | (shifted & mask);
            return object.set_uint_property(target.key.int, next);
        }

        let Some(object) = self.object_mut(local_id) else {
            return false;
        };
        match value {
            FieldValue::Bool(value) => object.set_bool_property(property_key, value),
            FieldValue::Bytes(_) | FieldValue::Callback => false,
            FieldValue::Color(value) => object.set_color_property(property_key, value),
            FieldValue::Double(value) => object.set_double_property(property_key, value),
            FieldValue::String(value) => {
                object.set_string_property(property_key, InstanceString::from_string_value(&value))
            }
            FieldValue::Uint(value) => object.set_uint_property(property_key, value),
        }
    }
}

fn bitmask_field_mask(bit: u8, width: u8) -> u64 {
    if bit >= 64 {
        return 0;
    }
    let width = width.min(64 - bit);
    let width_mask = if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    width_mask << bit
}

fn runtime_property_metadata_by_key(
    type_key: u16,
    property_key: u16,
) -> Option<(&'static str, &'static nuxie_schema::Property)> {
    property_by_key_in_hierarchy(type_key, property_key)
}

fn runtime_property_metadata_by_name(
    type_key: u16,
    property_name: &str,
) -> Option<(&'static str, &'static nuxie_schema::Property)> {
    let definition = definition_by_type_key(type_key)?;
    definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
        .map(|property| (definition.name, property))
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let definition = definition_by_name(ancestor)?;
                definition
                    .properties
                    .iter()
                    .find(|property| property.name == property_name)
                    .map(|property| (*ancestor, property))
            })
        })
}

fn field_value_matches_kind(value: &FieldValue, kind: FieldKind) -> bool {
    matches!(
        (value, kind),
        (FieldValue::Bool(_), FieldKind::Bool)
            | (FieldValue::Bytes(_), FieldKind::Bytes)
            | (FieldValue::Callback, FieldKind::Callback)
            | (FieldValue::Color(_), FieldKind::Color)
            | (FieldValue::Double(_), FieldKind::Double)
            | (FieldValue::String(_), FieldKind::String)
            | (FieldValue::Uint(_), FieldKind::Uint)
    )
}
