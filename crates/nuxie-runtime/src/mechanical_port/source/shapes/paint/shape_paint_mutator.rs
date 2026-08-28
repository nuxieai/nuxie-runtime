use crate::mechanical_port::source::{
    component::Component, core::CoreHandle, core_context::StatusCode,
};
use core::ops::{BitAnd, BitOr, BitOrAssign};
use nuxie_render_api::RenderPaint;
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
    shape_paint: Option<CoreHandle>,
    component: Option<CoreHandle>,
}
impl Default for ShapePaintMutatorState {
    fn default() -> Self {
        Self {
            flags: MutatorFlags::NONE,
            render_opacity: 1.0,
            shape_paint: None,
            component: None,
        }
    }
}
pub trait ShapePaintMutator {
    fn mutator_state(&self) -> &ShapePaintMutatorState;
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState;
    fn render_opacity_changed(&mut self);
    fn apply_to(&self, paint: &mut dyn RenderPaint, opacity: f32);
    fn init_paint_mutator(&mut self, component: &mut Component) -> StatusCode {
        self.mutator_state_mut().flags = MutatorFlags::TRANSLUCENT | MutatorFlags::VISIBLE;
        let Some(this) = component.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(parent) = component.parent_handle() else {
            return StatusCode::MissingObject;
        };
        let initialized = parent
            .with_mut(|parent| {
                parent
                    .as_shape_paint_behavior_mut()
                    .is_some_and(|paint| paint.initialize_render_paint(this.clone()))
            })
            .unwrap_or(false);
        if !initialized {
            return StatusCode::InvalidObject;
        }
        self.mutator_state_mut().component = Some(this);
        self.mutator_state_mut().shape_paint = Some(parent);
        StatusCode::Ok
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
    fn component_handle(&self) -> Option<CoreHandle> {
        self.mutator_state().component.clone()
    }
    fn is_translucent(&self) -> bool {
        self.mutator_state().flags & MutatorFlags::TRANSLUCENT == MutatorFlags::TRANSLUCENT
    }
    fn is_visible(&self) -> bool {
        self.mutator_state().flags & MutatorFlags::VISIBLE == MutatorFlags::VISIBLE
    }
    fn with_render_paint_mut<R>(
        &self,
        use_paint: impl FnOnce(&mut dyn RenderPaint) -> R,
    ) -> Option<R>
    where
        Self: Sized,
    {
        self.mutator_state().shape_paint.as_ref().and_then(|paint| {
            paint
                .with_mut(|paint| {
                    paint
                        .as_shape_paint_mut()
                        .and_then(|paint| paint.with_render_paint_mut(use_paint))
                })
                .flatten()
        })
    }
}
