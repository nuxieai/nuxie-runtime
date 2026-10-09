//! Weight parent contracts from pinned C++ 160085c6.

use super::Weight;
use crate::mechanical_port::source::{
    artboard::Artboard,
    component::Component,
    container_component::ContainerComponent,
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    core_context::CoreContext,
    generated::{
        component_base::ComponentBase,
        container_component_base::ContainerComponentBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
        shapes::vertex_base::VertexBase,
    },
    shapes::{
        cubic_detached_vertex::CubicDetachedVertex,
        vertex::{Vertex, VertexBehavior},
    },
    status_code::StatusCode,
};
use std::any::Any;

// Both variants are valid Component parents and expose the same Vertex
// capability. Only the source ancestry predicate differs.
#[derive(Default)]
struct VertexParent<const SOURCE_VERTEX: bool> {
    vertex: CubicDetachedVertex,
}

impl<const SOURCE_VERTEX: bool> VertexParent<SOURCE_VERTEX> {
    fn source_type(key: u16) -> bool {
        key == 65_000
            || if SOURCE_VERTEX {
                VertexBase::is_type_of(key)
            } else {
                ContainerComponentBase::is_type_of(key)
            }
    }
}

impl<const SOURCE_VERTEX: bool> VertexBehavior for VertexParent<SOURCE_VERTEX> {
    fn vertex(&self) -> &Vertex {
        self.vertex.vertex()
    }
    fn vertex_mut(&mut self) -> &mut Vertex {
        self.vertex.vertex_mut()
    }
    fn mark_geometry_dirty(&mut self) {}
    fn set_weight(&mut self, _weight: CoreHandle) {
        panic!("Weight::onAddedDirty must call the nonvirtual Vertex setter");
    }
}

impl<const SOURCE_VERTEX: bool> CoreCapabilities for VertexParent<SOURCE_VERTEX> {
    fn as_component(&self) -> Option<&Component> {
        self.vertex.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        self.vertex.as_component_mut()
    }
    fn as_container_component(&self) -> Option<&ContainerComponent> {
        self.vertex.as_container_component()
    }
    fn as_container_component_mut(&mut self) -> Option<&mut ContainerComponent> {
        self.vertex.as_container_component_mut()
    }
    fn as_vertex_behavior(&self) -> Option<&dyn VertexBehavior> {
        Some(self)
    }
    fn as_vertex_behavior_mut(&mut self) -> Option<&mut dyn VertexBehavior> {
        Some(self)
    }
}

impl<const SOURCE_VERTEX: bool> CoreObject for VertexParent<SOURCE_VERTEX> {
    fn core(&self) -> &Core {
        self.vertex.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.vertex.core_mut()
    }
    fn core_type(&self) -> u16 {
        65_000
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::source_type(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::source_type
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.vertex.deserialize(key, reader)
    }
}

macro_rules! delegate_setter {
    ($name:ident, $value:ty) => {
        fn $name(
            &mut self,
            field: CoreField,
            value: $value,
            completion: &mut PropertySetterCompletion,
        ) {
            self.vertex.$name(field, value, completion);
        }
    };
}

macro_rules! delegate_getter {
    ($name:ident, $value:ty) => {
        fn $name(&mut self, field: CoreField) -> $value {
            self.vertex.$name(field)
        }
    };
}

impl<const SOURCE_VERTEX: bool> CoreRegistryObject for VertexParent<SOURCE_VERTEX> {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::source_type(key)
    }
    delegate_setter!(set_uint_with_completion, u32);
    delegate_setter!(set_string_with_completion, String);
    delegate_setter!(set_color_with_completion, i32);
    delegate_setter!(set_bool_with_completion, bool);
    delegate_setter!(set_double_with_completion, f32);
    delegate_setter!(set_int_with_completion, i32);
    delegate_setter!(set_callback_with_completion, CallbackData<'_>);
    delegate_getter!(get_uint, u32);
    delegate_getter!(get_string, String);
    delegate_getter!(get_color, i32);
    delegate_getter!(get_bool, bool);
    delegate_getter!(get_double, f32);
    delegate_getter!(get_int, i32);
}

struct ParentContext {
    arena: CoreArena,
    artboard: CoreHandle,
    parent: CoreHandle,
}

impl CoreContext for ParentContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        match id {
            0 => Some(self.artboard.clone()),
            1 => Some(self.parent.clone()),
            _ => None,
        }
    }
}

fn import_weight<const SOURCE_VERTEX: bool>() -> (StatusCode, Option<CoreHandle>, CoreHandle) {
    let arena = CoreArena::default();
    let artboard = arena.insert(Artboard::default());
    let parent = arena.insert(VertexParent::<SOURCE_VERTEX>::default());
    let weight = arena.insert(Weight::default());
    assert!(CoreRegistry::set_uint_handle(
        &weight,
        ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
        1,
    ));
    let mut context = ParentContext {
        arena: arena.weak_handle(),
        artboard,
        parent: parent.clone(),
    };
    let status = weight
        .with_downcast_mut::<Weight, _>(|owner| owner.on_added_dirty(weight.clone(), &mut context))
        .expect("live Weight");
    assert_eq!(
        weight
            .with(|owner| owner.as_component().unwrap().parent_handle())
            .flatten(),
        Some(parent.clone()),
        "Component must have resolved a valid ContainerComponent parent",
    );
    let installed = parent
        .with(|owner| {
            let owner = owner
                .as_registry_any()
                .downcast_ref::<VertexParent<SOURCE_VERTEX>>()
                .unwrap();
            assert!(
                owner
                    .vertex
                    .as_container_component()
                    .unwrap()
                    .children()
                    .contains(&weight)
            );
            owner.vertex.vertex().weight_handle()
        })
        .flatten();
    (status, installed, weight)
}

#[test]
fn weight_rejects_a_vertex_capability_without_source_vertex_ancestry() {
    let (status, installed, _) = import_weight::<false>();
    assert_eq!(status, StatusCode::MissingObject);
    assert_eq!(installed, None);
}

#[test]
fn weight_installs_the_inherited_link_without_calling_a_set_weight_override() {
    let (status, installed, weight) = import_weight::<true>();
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(installed, Some(weight));
}
