use crate::mechanical_port::source::{
    animation::nested_input::NestedInput,
    generated::animation::nested_state_machine_base::NestedStateMachineBase, math::vec2d::Vec2D,
};
use std::ptr::NonNull;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HitResult {
    #[default]
    None,
    Hit,
}
pub trait NestedStateMachineInstance {
    fn advance(&mut self, elapsed: f32, new_frame: bool) -> bool;
    fn hit_test(&self, position: Vec2D) -> bool;
    fn pointer(
        &mut self,
        kind: u8,
        position: Vec2D,
        timestamp: f32,
        pointer: i32,
        primary: bool,
    ) -> HitResult;
    fn try_change_state(&mut self) -> bool;
    fn bind_view_model(&mut self, value: *mut ());
    fn set_data_context(&mut self, value: *mut ());
    fn clear_data_context(&mut self);
    fn share_parent_focus_manager(&mut self);
}
pub trait NestedStateMachineArtboard {
    fn state_machine_at(&mut self, id: u32) -> Option<Box<dyn NestedStateMachineInstance>>;
}
#[derive(Default)]
pub struct NestedStateMachine {
    pub base: NestedStateMachineBase,
    instance: Option<Box<dyn NestedStateMachineInstance>>,
    nested_inputs: Vec<NonNull<NestedInput>>,
}
impl NestedStateMachine {
    pub fn advance(&mut self, elapsed: f32, new_frame: bool) -> bool {
        self.instance
            .as_mut()
            .is_some_and(|v| v.advance(elapsed, new_frame))
    }
    pub fn initialize_animation(&mut self, artboard: &mut dyn NestedStateMachineArtboard) {
        self.instance = artboard.state_machine_at(self.base.base.animation_id());
        if let Some(instance) = &mut self.instance {
            instance.share_parent_focus_manager();
        }
        for mut input in self.nested_inputs.iter().copied() {
            unsafe {
                if input.as_ref().is_bool_or_number() {
                    input.as_mut().apply_value();
                }
            }
        }
    }
    pub fn state_machine_instance(
        &mut self,
    ) -> Option<&mut (dyn NestedStateMachineInstance + 'static)> {
        self.instance.as_deref_mut()
    }
    pub fn hit_test(&self, position: Vec2D) -> bool {
        self.instance.as_ref().is_some_and(|v| v.hit_test(position))
    }
    pub fn pointer_move(&mut self, p: Vec2D, t: f32, id: i32) -> HitResult {
        self.pointer(0, p, t, id, false)
    }
    pub fn pointer_down(&mut self, p: Vec2D, id: i32) -> HitResult {
        self.pointer(1, p, 0.0, id, false)
    }
    pub fn pointer_up(&mut self, p: Vec2D, id: i32) -> HitResult {
        self.pointer(2, p, 0.0, id, false)
    }
    pub fn pointer_exit(&mut self, p: Vec2D, id: i32) -> HitResult {
        self.pointer(3, p, 0.0, id, false)
    }
    pub fn drag_start(&mut self, p: Vec2D, t: f32, id: i32) -> HitResult {
        self.pointer(4, p, t, id, true)
    }
    pub fn drag_end(&mut self, p: Vec2D, t: f32, id: i32) -> HitResult {
        self.pointer(5, p, t, id, false)
    }
    fn pointer(&mut self, k: u8, p: Vec2D, t: f32, id: i32, primary: bool) -> HitResult {
        self.instance
            .as_mut()
            .map(|v| v.pointer(k, p, t, id, primary))
            .unwrap_or(HitResult::None)
    }
    pub fn add_nested_input(&mut self, input: &mut NestedInput) {
        self.nested_inputs.push(NonNull::from(input));
    }
    pub fn input(&mut self, index: usize) -> Option<&mut NestedInput> {
        self.nested_inputs
            .get_mut(index)
            .map(|v| unsafe { v.as_mut() })
    }
    pub fn input_named(&mut self, name: &str) -> Option<&mut NestedInput> {
        self.nested_inputs.iter_mut().find_map(|v| {
            let i = unsafe { v.as_mut() };
            (i.name() == name).then_some(i)
        })
    }
    pub fn bind_view_model_instance(&mut self, v: *mut ()) {
        if let Some(i) = &mut self.instance {
            i.bind_view_model(v)
        }
    }
    pub fn data_context(&mut self, v: *mut ()) {
        if let Some(i) = &mut self.instance {
            i.set_data_context(v)
        }
    }
    pub fn clear_data_context(&mut self) {
        if let Some(i) = &mut self.instance {
            i.clear_data_context()
        }
    }
    pub fn try_change_state(&mut self) -> bool {
        self.instance.as_mut().is_some_and(|v| v.try_change_state())
    }
    pub fn release_dependencies(&mut self) {
        self.instance = None;
    }
}
