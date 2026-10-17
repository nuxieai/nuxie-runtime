use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::shapes::parametric_path_base::ParametricPathBase,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{aabb::Aabb, vec2d::Vec2D},
};
impl std::ops::Deref for ParametricPath {
    type Target = ParametricPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ParametricPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ParametricPath {
    pub const TYPE_KEY: u16 = ParametricPathBase::TYPE_KEY;
}

#[derive(Default)]
pub struct ParametricPath {
    pub base: ParametricPathBase,
    layout_natural_size: crate::source::intrinsically_sizeable::LayoutNaturalSize,
}
impl ParametricPath {
    pub(crate) fn natural_size_for_layout(&self) -> Vec2D {
        self.layout_natural_size
            .get(Vec2D::new(self.base.width(), self.base.height()))
    }
    pub fn copy(
        &mut self,
        source: &Self,
        callbacks: &mut impl crate::source::generated::shapes::parametric_path_base::ParametricPathBaseCallbacks,
    ) {
        self.base.copy(&source.base, callbacks);
        self.layout_natural_size
            .copy_from(&source.layout_natural_size);
    }
    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::new(
            if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            }
            .min(self.base.width()),
            if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            }
            .min(self.base.height()),
        )
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        _width: LayoutScaleType,
        _height: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.layout_natural_size
            .begin_control(Vec2D::new(self.base.width(), self.base.height()));
        self.set_width(size.x);
        self.set_height(size.y);
        self.layout_natural_size.end_control();
        self.base.mark_world_transform_dirty();
        self.mark_path_dirty(false);
    }
    pub fn mark_path_dirty(&mut self, send_to_layout: bool) {
        self.base.base.mark_path_dirty(true);
        if send_to_layout {
            Self::send_layout_dirty(self.base.shape_handle(), self.base.parent_handle());
        }
    }
    pub(crate) fn send_layout_dirty_occurrence(owner: &CoreHandle) {
        let (shape, parent) = owner
            .with(|object| {
                let path = object
                    .as_parametric_path()
                    .expect("ParametricPath occurrence");
                (path.base.shape_handle(), path.base.parent_handle())
            })
            .expect("live ParametricPath");
        Self::send_layout_dirty(shape, parent);
    }
    /// Upstream stops at an intervening Node. A leaf LayoutComponent also
    /// measures drawing groups for their natural size (UNIV-4844), so a path
    /// below a group still invalidates the nearest such leaf.
    fn send_layout_dirty(shape: Option<CoreHandle>, mut parent: Option<CoreHandle>) {
        let mut through_group = false;
        while let Some(current) = parent {
            if current.is_type_of(
                    crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
                ) {
                    let measures_group = || {
                        current
                            .with(|object| {
                                object
                                    .as_layout_component()
                                    .is_some_and(|layout| !layout.has_layout_children())
                            })
                            .unwrap_or(false)
                    };
                    if !through_group || measures_group() {
                        crate::mechanical_port::source::layout_component::LayoutComponent::mark_layout_node_dirty_occurrence(&current, false);
                    }
                    break;
                }
            let is_node = current.is_type_of(
                crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY,
            );
            if is_node
                && !(current
                    .is_type_of(crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY)
                    && shape.as_ref() == Some(&current))
            {
                through_group = true;
            }
            parent = current
                .with(|current| current.component_parent_handle())
                .flatten();
        }
    }
    pub fn try_property_bounds(&self, result: &mut Aabb) -> bool {
        *result = Aabb::from_ltwh(
            -self.base.origin_x() * self.base.width(),
            -self.base.origin_y() * self.base.height(),
            self.base.width(),
            self.base.height(),
        );
        true
    }
    pub fn width_changed(&mut self) {
        self.layout_natural_size.width_changed(self.base.width());
        self.mark_path_dirty(true);
    }
    pub fn set_width(&mut self, value: f32) {
        if self.base.set_width_value(value) {
            self.width_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                ParametricPathBase::WIDTH_PROPERTY_KEY,
            );
        }
    }
    pub fn set_height(&mut self, value: f32) {
        if self.base.set_height_value(value) {
            self.height_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                ParametricPathBase::HEIGHT_PROPERTY_KEY,
            );
        }
    }
    pub fn height_changed(&mut self) {
        self.layout_natural_size.height_changed(self.base.height());
        self.mark_path_dirty(true);
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_path_dirty(true);
    }
}
