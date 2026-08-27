use crate::mechanical_port::source::{
    component::Component, core_context::StatusCode, renderer::RenderPaint,
    shapes::paint::shape_paint::ShapePaint,
};
use core::ops::{BitAnd, BitOr, BitOrAssign};
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MutatorFlags(pub u8);
impl MutatorFlags {
    pub const NONE: Self = Self(0);
    pub const VISIBLE: Self = Self(1);
    pub const TRANSLUCENT: Self = Self(2);
}
impl BitOr for MutatorFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for MutatorFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitAnd for MutatorFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

pub struct ShapePaintMutatorState {
    pub flags: MutatorFlags,
    render_opacity: f32,
    render_paint: Option<*mut RenderPaint>,
    component: Option<*mut Component>,
}
impl Default for ShapePaintMutatorState {
    fn default() -> Self {
        Self {
            flags: MutatorFlags::NONE,
            render_opacity: 1.0,
            render_paint: None,
            component: None,
        }
    }
}
pub trait ShapePaintMutator {
    fn mutator_state(&self) -> &ShapePaintMutatorState;
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState;
    fn render_opacity_changed(&mut self);
    fn apply_to(&self, paint: &mut RenderPaint, opacity: f32);
    fn init_paint_mutator(&mut self, component: &mut Component) -> StatusCode {
        self.mutator_state_mut().flags = MutatorFlags::TRANSLUCENT | MutatorFlags::VISIBLE;
        self.mutator_state_mut().component = Some(component);
        let parent = component.parent_mut();
        if let Some(paint) = parent.as_mut::<ShapePaint>() {
            if paint.render_paint().is_some() {
                return StatusCode::InvalidObject;
            }
            self.mutator_state_mut().render_paint = Some(paint.init_render_paint(self));
            StatusCode::Ok
        } else {
            StatusCode::MissingObject
        }
    }
    fn render_opacity(&self) -> f32 {
        self.mutator_state().render_opacity
    }
    fn set_render_opacity(&mut self, value: f32) {
        if self.mutator_state().render_opacity == value {
            return;
        }
        self.mutator_state_mut().render_opacity = value;
        self.render_opacity_changed();
    }
    fn component(&self) -> Option<&Component> {
        self.mutator_state().component.map(|p| unsafe { &*p })
    }
    fn is_translucent(&self) -> bool {
        self.mutator_state().flags & MutatorFlags::TRANSLUCENT == MutatorFlags::TRANSLUCENT
    }
    fn is_visible(&self) -> bool {
        self.mutator_state().flags & MutatorFlags::VISIBLE == MutatorFlags::VISIBLE
    }
    fn render_paint(&self) -> Option<&RenderPaint> {
        self.mutator_state().render_paint.map(|p| unsafe { &*p })
    }
}
