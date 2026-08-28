use crate::mechanical_port::source::{
    animation::{
        nested_bool::NestedBool, nested_number::NestedNumber, nested_trigger::NestedTrigger,
    },
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::nested_state_machine_base::NestedStateMachineBase,
    math::vec2d::Vec2D,
};
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
    fn bind_view_model(&mut self, value: CoreHandle);
    fn set_data_context(&mut self, value: CoreHandle);
    fn clear_data_context(&mut self);
    fn share_parent_focus_manager(&mut self);
    fn input_name(&self, index: u32) -> Option<&str>;
    fn bool_input_value(&self, index: u32) -> Option<bool>;
    fn number_input_value(&self, index: u32) -> Option<f32>;
    fn set_bool_input(&mut self, index: u32, value: bool);
    fn set_number_input(&mut self, index: u32, value: f32);
    fn fire_trigger_input(&mut self, index: u32);
}
pub trait NestedStateMachineArtboard {
    fn state_machine_at(&mut self, id: u32) -> Option<Box<dyn NestedStateMachineInstance>>;
}
#[derive(Default)]
pub struct NestedStateMachine {
    pub base: NestedStateMachineBase,
    instance: Option<Box<dyn NestedStateMachineInstance>>,
    nested_inputs: Vec<CoreHandle>,
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
        let Some(instance) = self.instance.as_mut() else {
            return;
        };
        for input in self.nested_inputs.iter().cloned() {
            if let Some((input_id, value)) = input.with_downcast::<NestedBool, _>(|input| {
                (input.base.base.base.input_id(), input.base.nested_value())
            }) {
                instance.set_bool_input(input_id, value);
            } else if let Some((input_id, value)) =
                input.with_downcast::<NestedNumber, _>(|input| {
                    (input.base.base.base.input_id(), input.base.nested_value())
                })
            {
                instance.set_number_input(input_id, value);
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
    pub fn add_nested_input(&mut self, input: CoreHandle) {
        self.nested_inputs.push(input);
    }
    pub fn input_count(&self) -> usize {
        self.nested_inputs.len()
    }
    pub fn input(&self, index: usize) -> Option<CoreHandle> {
        self.nested_inputs.get(index).cloned()
    }
    pub fn input_named(&self, name: &str, context: &dyn CoreContext) -> Option<CoreHandle> {
        self.nested_inputs.iter().find_map(|input| {
            let matches = input
                .with_downcast::<NestedBool, _>(|input| input.base.base.name(context) == name)
                .or_else(|| {
                    input.with_downcast::<NestedNumber, _>(|input| {
                        input.base.base.name(context) == name
                    })
                })
                .or_else(|| {
                    input.with_downcast::<NestedTrigger, _>(|input| {
                        input.base.base.name(context) == name
                    })
                })
                .unwrap_or(false);
            matches.then(|| input.clone())
        })
    }
    pub fn input_name(&self, index: u32) -> Option<String> {
        self.instance.as_ref()?.input_name(index).map(str::to_owned)
    }
    pub fn bool_input_value(&self, index: u32) -> Option<bool> {
        self.instance.as_ref()?.bool_input_value(index)
    }
    pub fn number_input_value(&self, index: u32) -> Option<f32> {
        self.instance.as_ref()?.number_input_value(index)
    }
    pub fn set_bool_input(&mut self, index: u32, value: bool) {
        if let Some(instance) = &mut self.instance {
            instance.set_bool_input(index, value);
        }
    }
    pub fn set_number_input(&mut self, index: u32, value: f32) {
        if let Some(instance) = &mut self.instance {
            instance.set_number_input(index, value);
        }
    }
    pub fn fire_trigger_input(&mut self, index: u32) {
        if let Some(instance) = &mut self.instance {
            instance.fire_trigger_input(index);
        }
    }
    pub fn bind_view_model_instance(&mut self, v: CoreHandle) {
        if let Some(i) = &mut self.instance {
            i.bind_view_model(v)
        }
    }
    pub fn data_context(&mut self, v: CoreHandle) {
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
