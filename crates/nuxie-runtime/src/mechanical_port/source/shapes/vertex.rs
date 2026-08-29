use crate::mechanical_port::source::{
    bones::weight::Weight,
    core::CoreHandle,
    generated::shapes::vertex_base::{VertexBase, VertexBaseCallbacks},
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

#[derive(Default)]
pub struct VertexState {
    weight: Option<CoreHandle>,
}

pub struct Vertex {
    pub base: VertexBase,
    state: VertexState,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            base: VertexBase::default(),
            state: VertexState::default(),
        }
    }
}

impl VertexBaseCallbacks for Vertex {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.notify_property_changed(property_key);
    }

    fn x_changed(&mut self) {
        VertexBehavior::x_changed(self);
    }

    fn y_changed(&mut self) {
        VertexBehavior::y_changed(self);
    }
}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks for Vertex {
    fn notify_property_changed(&mut self, property_key: u16) {
        Vertex::notify_property_changed(self, property_key);
    }
}

impl Vertex {
    pub fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

pub trait VertexBehavior {
    fn vertex(&self) -> &Vertex;
    fn vertex_mut(&mut self) -> &mut Vertex;
    fn mark_geometry_dirty(&mut self);
    fn set_weight(&mut self, weight: CoreHandle) {
        self.vertex_mut().state.weight = Some(weight);
    }
    fn has_weight(&self) -> bool {
        self.vertex().state.weight.is_some()
    }
    fn weight_handle(&self) -> Option<CoreHandle> {
        self.vertex().state.weight.clone()
    }
    fn render_translation(&self) -> Vec2D {
        if let Some(weight) = self.vertex().state.weight.as_ref() {
            return weight
                .with_mut(|object| {
                    *object
                        .as_weight_mut()
                        .expect("a retained vertex weight preserves its Weight base")
                        .translation()
                })
                .expect("a retained vertex weight remains live");
        }
        Vec2D::new(self.vertex().base.x(), self.vertex().base.y())
    }
    fn x_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn y_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn deform(&mut self, world: &Mat2D, bone_transforms: &[f32]) {
        let weight = self
            .vertex()
            .state
            .weight
            .clone()
            .expect("a skin-deformed vertex has a Weight");
        let position = Vec2D::new(self.vertex().base.x(), self.vertex().base.y());
        weight
            .with_mut(|object| {
                let weight = object
                    .as_weight_mut()
                    .expect("a retained vertex weight preserves its Weight base");
                *weight.translation() = Weight::deform(
                    position,
                    weight.indices(),
                    weight.values(),
                    world,
                    bone_transforms,
                );
            })
            .expect("a retained vertex weight remains live");
    }
}

impl VertexBehavior for Vertex {
    fn vertex(&self) -> &Vertex {
        self
    }

    fn vertex_mut(&mut self) -> &mut Vertex {
        self
    }

    fn mark_geometry_dirty(&mut self) {}
}

impl std::ops::Deref for Vertex {
    type Target = VertexBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Vertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
