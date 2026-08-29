use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    core_context::CoreContext,
    generated::{bones::bone_base::BoneBase, core_registry::CoreCapabilities},
    math::vec2d::Vec2D,
    status_code::StatusCode,
    transform_component::TransformComponent,
};

pub struct Bone {
    pub base: BoneBase,
    child_bones: Vec<CoreHandle>,
    peer_constraints: Vec<CoreHandle>,
}

impl std::ops::Deref for Bone {
    type Target = BoneBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Bone {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            base: BoneBase::default(),
            child_bones: Vec::new(),
            peer_constraints: Vec::new(),
        }
    }
}

impl Bone {
    fn component(&self) -> &Component {
        &self.base.base.base.base.base.base.base.base.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.base.base.base.base.base.base.base.base.base.base
    }

    pub(crate) fn core_mut(&mut self) -> &mut crate::mechanical_port::source::core::Core {
        &mut self.component_mut().base.base
    }

    fn transform_component(&self) -> &TransformComponent {
        &self.base.base.base.base
    }

    fn transform_component_mut(&mut self) -> &mut TransformComponent {
        &mut self.base.base.base.base
    }

    pub fn child_bones(&self) -> Vec<CoreHandle> {
        // The C++ signature returns its vector by value.
        self.child_bones.clone()
    }

    pub fn add_child_bone(&mut self, bone: CoreHandle) {
        self.child_bones.push(bone);
    }

    pub fn on_added_clean(
        &mut self,
        this: CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        // The pinned owner deliberately ignores the superclass status here.
        let _ = self.transform_component_mut().on_added_clean(context);
        let Some(parent) = context.resolve(self.component().base.parent_id()) else {
            return StatusCode::MissingObject;
        };
        if !parent.is_type_of(BoneBase::TYPE_KEY) {
            return StatusCode::MissingObject;
        }
        parent
            .with_mut(|parent| {
                parent
                    .as_bone_mut()
                    .map(|parent| parent.add_child_bone(this))
            })
            .flatten()
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }

    pub fn length(&self) -> f32 {
        self.base.length()
    }

    pub fn set_length(&mut self, value: f32) {
        if !self.base.set_length_value(value) {
            return;
        }
        self.length_changed();
        self.core_mut()
            .notify_property_changed(BoneBase::LENGTH_PROPERTY_KEY);
    }

    pub fn length_changed(&mut self) {
        for child in self.child_bones.iter().cloned() {
            child
                .with_downcast_mut::<Bone, _>(|bone| {
                    bone.transform_component_mut().mark_transform_dirty()
                })
                .expect("a retained child Bone must remain a Bone");
        }
    }

    pub fn x(&self, context: &dyn CoreContext) -> f32 {
        let parent = context
            .resolve(self.component().base.parent_id())
            .expect("onAddedClean requires a Bone parent");
        parent
            .with_downcast::<Bone, _>(|bone| bone.base.length())
            .expect("onAddedClean requires a Bone parent")
    }

    pub fn y(&self) -> f32 {
        0.0
    }

    pub fn tip_world_translation(&self) -> Vec2D {
        *self.transform_component().base.base.world_transform()
            * Vec2D::new(self.base.length(), 0.0)
    }

    pub fn add_peer_constraint(&mut self, peer: CoreHandle) {
        assert!(!self.peer_constraints.contains(&peer));
        self.peer_constraints.push(peer);
    }

    pub fn peer_constraints(&self) -> &[CoreHandle] {
        &self.peer_constraints
    }
}
impl crate::mechanical_port::source::generated::bones::bone_base::BoneBaseCallbacks for Bone {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
    fn length_changed(&mut self) {
        Bone::length_changed(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{
        bones::root_bone::RootBone, component_dirt::ComponentDirt, core::CoreArena,
        generated::core_registry::CoreRegistry,
    };

    struct TestContext {
        arena: CoreArena,
        handles: Vec<CoreHandle>,
    }

    impl CoreContext for TestContext {
        fn core_arena(&self) -> &CoreArena {
            &self.arena
        }

        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            self.handles.get(id as usize).cloned()
        }
    }

    #[test]
    fn root_bone_child_registration_propagates_length_dirt() {
        let arena = CoreArena::default();
        let root = arena.insert(RootBone::default());
        let child = arena.insert(Bone::default());
        let mut context = TestContext {
            arena: arena.clone(),
            handles: vec![root.clone(), child.clone()],
        };

        let status = child
            .with_mut(|child| {
                child
                    .lifecycle_on_added_clean(&mut context)
                    .expect("Bone supplies its lifecycle owner")
            })
            .expect("child Bone remains live");
        assert_eq!(status, StatusCode::Ok);
        assert!(
            root.with(|root| {
                root.as_bone()
                    .expect("RootBone has a Bone base view")
                    .child_bones()
                    .contains(&child)
            })
            .expect("RootBone remains live")
        );

        child
            .with_mut(|child| {
                child
                    .as_component_mut()
                    .expect("Bone is a Component")
                    .set_dirt(ComponentDirt::NONE)
            })
            .expect("child Bone remains live");
        assert!(CoreRegistry::set_double_handle(
            &root,
            BoneBase::LENGTH_PROPERTY_KEY.into(),
            16.43033,
        ));
        child
            .with(|child| {
                let child = child.as_component().expect("Bone is a Component");
                assert!(child.has_dirt(ComponentDirt::TRANSFORM));
                assert!(child.has_dirt(ComponentDirt::WORLD_TRANSFORM));
            })
            .expect("child Bone remains live");
    }
}
