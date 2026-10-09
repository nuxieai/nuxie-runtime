//! Skin owner contracts from pinned C++ 160085c6.
use nuxie_runtime::source::{
    artboard::Artboard,
    bones::skin::Skin,
    core::{CoreArena, CoreHandle},
    core_context::CoreContext,
    generated::{component_base::ComponentBase, core_registry::CoreRegistry},
    shapes::{mesh::Mesh, points_path::PointsPath},
    status_code::StatusCode,
};
use nuxie_runtime::source::{
    core::CoreObject,
    generated::core_registry::{CoreCapabilities, CoreRegistryObject},
};
use std::cell::Cell;

struct ChangingParentContext {
    arena: CoreArena,
    artboard: CoreHandle,
    first: CoreHandle,
    second: CoreHandle,
    parent_reads: Cell<usize>,
}
impl CoreContext for ChangingParentContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        match id {
            0 => Some(self.artboard.clone()),
            1 => {
                let reads = self.parent_reads.get();
                self.parent_reads.set(reads + 1);
                Some(
                    if reads == 0 {
                        &self.first
                    } else {
                        &self.second
                    }
                    .clone(),
                )
            }
            _ => None,
        }
    }
}

#[test]
fn skin_installs_on_the_parent_already_resolved_by_component() {
    for first_is_mesh in [false, true] {
        let arena = CoreArena::default();
        let artboard = arena.insert(Artboard::default());
        let path = arena.insert(PointsPath::default());
        let mesh = arena.insert(Mesh::default());
        let owner = arena.insert(Skin::default());
        assert!(CoreRegistry::set_uint_handle(
            &owner,
            ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
            1,
        ));
        let (first, second) = if first_is_mesh {
            (mesh, path)
        } else {
            (path, mesh)
        };
        let mut context = ChangingParentContext {
            arena: arena.weak_handle(),
            artboard,
            first: first.clone(),
            second: second.clone(),
            parent_reads: Cell::new(0),
        };
        let status = owner
            .with_downcast_mut::<Skin, _>(|skin| skin.on_added_dirty(owner.clone(), &mut context))
            .unwrap();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(
            context.parent_reads.get(),
            1,
            "Skin must use Component::parent()"
        );
        assert_eq!(
            owner
                .with(|skin| skin.as_component().unwrap().parent_handle())
                .flatten(),
            Some(first.clone())
        );
        assert_eq!(
            first
                .with(|object| object.as_skinnable_behavior().unwrap().skin())
                .flatten(),
            Some(owner)
        );
        assert_eq!(
            second
                .with(|object| object.as_skinnable_behavior().unwrap().skin())
                .flatten(),
            None
        );
    }
}

// Open Rust capabilities do not override Skinnable::from's exact source type
// switch. This proxy advertises the PointsPath ancestry while retaining its
// distinct concrete source key.
struct OtherSkinnable {
    path: PointsPath,
}
impl nuxie_runtime::source::generated::core_registry::CoreCapabilities for OtherSkinnable {
    fn as_component(&self) -> Option<&nuxie_runtime::source::component::Component> {
        self.path.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut nuxie_runtime::source::component::Component> {
        self.path.as_component_mut()
    }
    fn as_container_component_mut(
        &mut self,
    ) -> Option<&mut nuxie_runtime::source::container_component::ContainerComponent> {
        self.path.as_container_component_mut()
    }
    fn as_skinnable_behavior(
        &self,
    ) -> Option<&dyn nuxie_runtime::source::bones::skinnable::SkinnableBehavior> {
        Some(&self.path)
    }
    fn as_skinnable_behavior_mut(
        &mut self,
    ) -> Option<&mut dyn nuxie_runtime::source::bones::skinnable::SkinnableBehavior> {
        Some(&mut self.path)
    }
}
impl nuxie_runtime::source::core::CoreObject for OtherSkinnable {
    fn core(&self) -> &nuxie_runtime::source::core::Core {
        self.path.core()
    }
    fn core_mut(&mut self) -> &mut nuxie_runtime::source::core::Core {
        self.path.core_mut()
    }
    fn core_type(&self) -> u16 {
        65534
    }
    fn is_type_of(&self, key: u16) -> bool {
        key == 65534 || nuxie_runtime::source::generated::shapes::points_path_base::PointsPathBase::is_type_of(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        |key| {
            key == 65534 || nuxie_runtime::source::generated::shapes::points_path_base::PointsPathBase::is_type_of(key)
        }
    }
    fn deserialize(
        &mut self,
        key: u16,
        reader: &mut nuxie_runtime::source::core::binary_reader::BinaryReader<'_>,
    ) -> bool {
        self.path.deserialize(key, reader)
    }
}
impl nuxie_runtime::source::generated::core_registry::CoreRegistryObject for OtherSkinnable {
    fn as_registry_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        nuxie_runtime::source::core::CoreObject::is_type_of(self, key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: u32,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path.set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: String,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: i32,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: bool,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path.set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: f32,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: nuxie_runtime::source::core::field_types::core_callback_type::CallbackData<'_>,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
        value: i32,
        completion: &mut nuxie_runtime::source::core::PropertySetterCompletion,
    ) {
        self.path.set_int_with_completion(field, value, completion);
    }
    fn get_uint(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> u32 {
        self.path.get_uint(field)
    }
    fn get_string(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> String {
        self.path.get_string(field)
    }
    fn get_color(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> i32 {
        self.path.get_color(field)
    }
    fn get_bool(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> bool {
        self.path.get_bool(field)
    }
    fn get_double(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> f32 {
        self.path.get_double(field)
    }
    fn get_int(
        &mut self,
        field: nuxie_runtime::source::generated::core_registry::CoreField,
    ) -> i32 {
        self.path.get_int(field)
    }
}

#[test]
fn skin_rejects_a_capability_only_parent_outside_the_source_type_switch() {
    let arena = CoreArena::default();
    let artboard = arena.insert(Artboard::default());
    let parent = arena.insert(OtherSkinnable {
        path: PointsPath::default(),
    });
    let owner = arena.insert(Skin::default());
    assert!(CoreRegistry::set_uint_handle(
        &owner,
        ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
        1
    ));
    let mut context = ChangingParentContext {
        arena: arena.weak_handle(),
        artboard,
        first: parent.clone(),
        second: parent.clone(),
        parent_reads: Cell::new(0),
    };
    assert_eq!(
        owner.with_downcast_mut::<Skin, _>(|skin| skin.on_added_dirty(owner.clone(), &mut context)),
        Some(StatusCode::MissingObject)
    );
    assert_eq!(
        parent
            .with(|parent| parent.as_skinnable_behavior().unwrap().skin())
            .flatten(),
        None
    );
}
