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

pub fn from(component: &mut Component) -> Option<&mut dyn IntrinsicallySizeable> {
    if component.is::<TransformComponent>() {
        component
            .as_transform_component_mut()
            .map(|component| component as &mut dyn IntrinsicallySizeable)
    } else if component.is::<Joystick>() {
        component
            .as_joystick_mut()
            .map(|component| component as &mut dyn IntrinsicallySizeable)
    } else {
        None
    }
}
