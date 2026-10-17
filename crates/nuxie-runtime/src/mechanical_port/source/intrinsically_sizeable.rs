use crate::mechanical_port::source::component::Component;
use crate::mechanical_port::source::joystick::Joystick;
use crate::mechanical_port::source::layout::layout_enums::{LayoutDirection, LayoutScaleType};
use crate::mechanical_port::source::layout::layout_measure_mode::LayoutMeasureMode;
use crate::mechanical_port::source::math::vec2d::Vec2D;
use crate::mechanical_port::source::transform_component::TransformComponent;

pub trait IntrinsicallySizeable {
    fn measure_layout(
        &mut self,
        _width: f32,
        _width_mode: LayoutMeasureMode,
        _height: f32,
        _height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::default()
    }

    fn control_size(
        &mut self,
        _size: Vec2D,
        _width_scale_type: LayoutScaleType,
        _height_scale_type: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
    }

    fn should_propagate_size_to_children(&self) -> bool {
        true
    }
}

pub fn from(
    component: &mut dyn crate::mechanical_port::source::core::CoreObject,
) -> Option<&mut dyn IntrinsicallySizeable> {
    component.as_intrinsically_sizeable_mut()
}

/// Dispatch controlSize without holding a host borrow across a child callback.
pub fn control_size_handle(
    owner: &crate::mechanical_port::source::core::CoreHandle,
    size: Vec2D,
    width: LayoutScaleType,
    height: LayoutScaleType,
    direction: LayoutDirection,
) -> bool {
    if owner.is_type_of(
        crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY,
    ) {
        crate::mechanical_port::source::shapes::shape::Shape::control_size_occurrence(
            owner, size, width, height, direction,
        );
        return true;
    }
    owner
        .with_mut(|owner| {
            let Some(sizeable) = owner.as_intrinsically_sizeable_mut() else {
                return false;
            };
            sizeable.control_size(size, width, height, direction);
            true
        })
        .unwrap_or(false)
}

macro_rules! intrinsic_owner {
    ($owner:path) => {
        impl IntrinsicallySizeable for $owner {
            fn measure_layout(
                &mut self,
                width: f32,
                width_mode: LayoutMeasureMode,
                height: f32,
                height_mode: LayoutMeasureMode,
            ) -> Vec2D {
                <$owner>::measure_layout(self, width, width_mode, height, height_mode)
            }
            fn control_size(
                &mut self,
                size: Vec2D,
                width: LayoutScaleType,
                height: LayoutScaleType,
                direction: LayoutDirection,
            ) {
                <$owner>::control_size(self, size, width, height, direction)
            }
        }
    };
}
intrinsic_owner!(crate::mechanical_port::source::shapes::shape::Shape);
intrinsic_owner!(crate::mechanical_port::source::shapes::image::Image);
intrinsic_owner!(crate::mechanical_port::source::shapes::parametric_path::ParametricPath);
intrinsic_owner!(crate::mechanical_port::source::text::text::Text);
intrinsic_owner!(crate::mechanical_port::source::text::text_input::TextInput);
intrinsic_owner!(crate::mechanical_port::source::nested_artboard::NestedArtboard);
intrinsic_owner!(crate::mechanical_port::source::scripted::scripted_layout::ScriptedLayout);
impl IntrinsicallySizeable for crate::mechanical_port::source::layout_component::LayoutComponent {
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Self::measure_layout(self, width, width_mode, height, height_mode)
    }
}
impl IntrinsicallySizeable for crate::mechanical_port::source::layout::n_sliced_node::NSlicedNode {
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Self::measure_layout(self, width, width_mode, height, height_mode)
    }
    fn control_size(
        &mut self,
        size: Vec2D,
        width: LayoutScaleType,
        height: LayoutScaleType,
        direction: LayoutDirection,
    ) {
        Self::control_size(self, size, width, height, direction);
    }
    fn should_propagate_size_to_children(&self) -> bool {
        false
    }
}
