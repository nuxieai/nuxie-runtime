//! Authored Star owner from pinned rive-runtime 160085c6 star.hpp/star.cpp.

use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    component::ComponentDirt, generated::shapes::star_base::StarBase, math::math_types,
    shapes::straight_vertex::StraightVertex,
};

pub struct Star {
    pub base: StarBase,
}

impl Default for Star {
    fn default() -> Self {
        Self::new(StarBase::default())
    }
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    Star,
    |owner, dirt| {
        if dirt.contains(ComponentDirt::PATH) {
            owner.with_downcast_mut::<Self, _>(|object| object.update_before_path_super(dirt));
        }
        crate::mechanical_port::source::shapes::path::Path::update_occurrence::<Self>(owner, dirt);
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

impl Star {
    pub const TYPE_KEY: u16 = StarBase::TYPE_KEY;

    pub fn new(base: StarBase) -> Self {
        Self { base }
    }

    pub fn inner_radius_changed(&mut self) {
        self.base.mark_path_dirty(true);
    }

    pub fn vertex_count(&self) -> usize {
        // points() returns uint32_t; C++ multiplies in that type before
        // converting the result to size_t.
        self.base.points().wrapping_mul(2) as usize
    }

    pub fn build_polygon(&mut self) {
        let half_width = self.base.width() / 2.0;
        let half_height = self.base.height() / 2.0;
        let inner_half_width = self.base.width() * self.base.inner_radius() / 2.0;
        let inner_half_height = self.base.height() * self.base.inner_radius() / 2.0;
        let ox = -self.base.origin_x() * self.base.width() + half_width;
        let oy = -self.base.origin_y() * self.base.height() + half_height;

        let length = self.vertex_count();
        let mut angle = -math_types::PI / 2.0;
        let increment = 2.0 * math_types::PI / length as f32;
        for index in (0..length).step_by(2) {
            {
                let mut vertex = self.base.base.polygon.vertices[index].borrow_mut();
                vertex.set_x(ox + angle.cos() * half_width);
                vertex.set_y(oy + angle.sin() * half_height);
                vertex.set_radius(self.base.corner_radius());
                angle += increment;
            }
            {
                let mut vertex = self.base.base.polygon.vertices[index + 1].borrow_mut();
                vertex.set_x(ox + angle.cos() * inner_half_width);
                vertex.set_y(oy + angle.sin() * inner_half_height);
                vertex.set_radius(self.base.corner_radius());
                angle += increment;
            }
        }
    }

    /// Star::update delegates to Polygon::update, whose two virtual calls
    /// select Star::vertexCount and Star::buildPolygon on this same owner.
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if value.contains(ComponentDirt::PATH) {
            if self.base.base.polygon.vertices.len() != self.vertex_count() {
                let count = self.vertex_count();
                self.base
                    .base
                    .polygon
                    .vertices
                    .resize_with(count, || Rc::new(RefCell::new(StraightVertex::default())));
                let polygon = &mut self.base.base;
                polygon.base.clear_vertices();
                for vertex in &polygon.polygon.vertices {
                    polygon.base.add_runtime_straight_vertex(vertex.clone());
                }
            }
            self.build_polygon();
        }
    }
}

impl std::ops::Deref for Star {
    type Target = StarBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Star {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
