use crate::mechanical_port::source::{core::CoreHandle, core_context::StatusCode};
use core::ops::{BitAnd, BitOr, BitOrAssign};
use nuxie_render_api::RenderPaint;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
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
    render_paint: Weak<RefCell<Box<dyn RenderPaint>>>,
    component: Option<CoreHandle>,
}
impl Default for ShapePaintMutatorState {
    fn default() -> Self {
        Self {
            flags: MutatorFlags::NONE,
            render_opacity: 1.0,
            render_paint: Weak::new(),
            component: None,
        }
    }
}
pub trait ShapePaintMutator {
    fn mutator_state(&self) -> &ShapePaintMutatorState;
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState;
    fn render_opacity_changed(&mut self);
    fn apply_to(
        &mut self,
        paint: &mut dyn RenderPaint,
        opacity: f32,
        path_flags: crate::mechanical_port::source::shapes::path_flags::PathFlags,
    );
    fn init_paint_mutator(
        &mut self,
        component: CoreHandle,
        parent: Option<CoreHandle>,
    ) -> StatusCode {
        self.mutator_state_mut().flags = MutatorFlags::TRANSLUCENT | MutatorFlags::VISIBLE;
        self.mutator_state_mut().component = Some(component.clone());
        let Some(parent) = parent else {
            return StatusCode::MissingObject;
        };
        parent
            .with_mut(|parent| {
                let Some(paint) = parent.as_shape_paint_behavior_mut() else {
                    return StatusCode::MissingObject;
                };
                if !paint.initialize_render_paint(component) {
                    return StatusCode::InvalidObject;
                }
                let render_paint = paint
                    .shape_paint()
                    .render_paint_handle()
                    .expect("initialized ShapePaint owns its render paint");
                self.mutator_state_mut().render_paint = Rc::downgrade(&render_paint);
                StatusCode::Ok
            })
            .unwrap_or(StatusCode::MissingObject)
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
        let paint = self.mutator_state().render_paint.upgrade()?;
        Some(use_paint(paint.borrow_mut().as_mut()))
    }
}

impl ShapePaintMutatorState {
    pub fn render_paint_handle(&self) -> Option<Rc<RefCell<Box<dyn RenderPaint>>>> {
        self.render_paint.upgrade()
    }
}
