use crate::mechanical_port::source::{
    animation::{
        nested_animation::NestedAnimationBehavior,
        nested_bool::NestedBool,
        nested_number::NestedNumber,
        nested_trigger::NestedTrigger,
        state_machine_instance::{
            RuntimeStateMachineInstanceHandle, RuntimeStateMachineInstanceWeakHandle,
            StateMachineInstance,
        },
    },
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    core::CoreHandle,
    core_context::CoreContext,
    data_bind::data_context::RuntimeDataContextHandle,
    focus_data::FocusData,
    generated::animation::nested_state_machine_base::NestedStateMachineBase,
    hit_result::HitResult,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
};
#[derive(Default)]
pub struct NestedStateMachine {
    pub base: NestedStateMachineBase,
    instance: Option<RuntimeStateMachineInstanceHandle>,
    nested_inputs: Vec<CoreHandle>,
}

impl NestedStateMachine {
    pub fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool {
        self.instance.as_ref().is_some_and(|instance| {
            instance.with_instance_mut(|instance| instance.advance(elapsed_seconds, new_frame))
        })
    }

    pub fn initialize_animation(&mut self, artboard: RuntimeArtboardInstanceWeakHandle) {
        let animation_id = self.base.animation_id() as usize;
        self.instance = artboard
            .with_artboard_mut(|artboard| artboard.state_machine_instance_handle(animation_id))
            .flatten();

        // Pinned order is significant: create the machine, share the parent
        // focus manager and rebuild the nested focus tree, then apply authored
        // bool/number input values in file order.
        self.share_parent_focus_manager();
        for input in self.nested_inputs.clone() {
            if input
                .with_downcast_mut::<NestedBool, _>(NestedBool::apply_value)
                .is_none()
            {
                input.with_downcast_mut::<NestedNumber, _>(NestedNumber::apply_value);
            }
        }
    }

    fn share_parent_focus_manager(&mut self) {
        let (Some(instance), Some(parent)) = (self.instance.as_ref(), self.base.parent_handle())
        else {
            return;
        };
        let focus_manager = parent
            .with_downcast::<NestedArtboard, _>(NestedArtboard::parent_artboard_handle)
            .flatten()
            .and_then(|artboard| {
                artboard
                    .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
                    .flatten()
            });
        let Some(focus_manager) = focus_manager else {
            return;
        };
        instance.with_instance_mut(|instance| {
            instance.set_external_focus_manager_handle(focus_manager);
        });
        let fallback = FocusData::find_closest_focus_node_handle(parent.clone());
        parent.with_downcast_mut::<NestedArtboard, _>(|nested_artboard| {
            nested_artboard.sync_nested_focus_tree(fallback, false, true);
        });
    }

    pub fn state_machine_instance(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.instance.clone()
    }

    pub fn state_machine_instance_weak(&self) -> RuntimeStateMachineInstanceWeakHandle {
        self.instance
            .as_ref()
            .map(RuntimeStateMachineInstanceHandle::downgrade)
            .unwrap_or_default()
    }

    pub fn hit_test(&self, position: Vec2D) -> bool {
        self.instance
            .as_ref()
            .is_some_and(|instance| instance.with_instance(|instance| instance.hit_test(position)))
    }

    pub fn pointer_move(&mut self, position: Vec2D, timestamp: f32, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance.with_instance_mut(|instance| {
                instance.pointer_move(position, timestamp, pointer_id)
            })
        })
    }

    pub fn pointer_down(&mut self, position: Vec2D, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance.with_instance_mut(|instance| instance.pointer_down(position, pointer_id))
        })
    }

    pub fn pointer_up(&mut self, position: Vec2D, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance.with_instance_mut(|instance| instance.pointer_up(position, pointer_id))
        })
    }

    pub fn pointer_exit(&mut self, position: Vec2D, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance.with_instance_mut(|instance| instance.pointer_exit(position, pointer_id))
        })
    }

    pub fn drag_start(&mut self, position: Vec2D, timestamp: f32, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance.with_instance_mut(|instance| {
                instance.drag_start(position, timestamp, true, pointer_id)
            })
        })
    }

    pub fn drag_end(&mut self, position: Vec2D, timestamp: f32, pointer_id: i32) -> HitResult {
        self.instance.as_ref().map_or(HitResult::None, |instance| {
            instance
                .with_instance_mut(|instance| instance.drag_end(position, timestamp, pointer_id))
        })
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
        self.instance.as_ref()?.with_instance(|instance| {
            instance
                .input(index as usize)
                .map(|input| input.name().to_owned())
        })
    }

    pub fn bool_input_value(&self, index: u32) -> Option<bool> {
        self.instance.as_ref()?.with_instance(|instance| {
            let name = instance.input(index as usize)?.name();
            instance.get_bool(name).map(|input| input.value())
        })
    }

    pub fn number_input_value(&self, index: u32) -> Option<f32> {
        self.instance
            .as_ref()?
            .with_instance(|instance| instance.number_input_value(index))
    }

    pub fn set_bool_input(&mut self, index: u32, value: bool) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(|instance| {
                let Some(name) = instance
                    .input(index as usize)
                    .map(|input| input.name().to_owned())
                else {
                    return;
                };
                if let Some(input) = instance.get_bool_mut(&name) {
                    input.set_value(value);
                }
            });
        }
    }

    pub fn set_number_input(&mut self, index: u32, value: f32) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(|instance| {
                let Some(name) = instance
                    .input(index as usize)
                    .map(|input| input.name().to_owned())
                else {
                    return;
                };
                if let Some(input) = instance.get_number_mut(&name) {
                    input.set_value(value);
                }
            });
        }
    }

    pub fn fire_trigger_input(&mut self, index: u32) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(|instance| {
                let Some(name) = instance
                    .input(index as usize)
                    .map(|input| input.name().to_owned())
                else {
                    return;
                };
                if let Some(input) = instance.get_trigger_mut(&name) {
                    input.fire();
                }
            });
        }
    }

    pub fn bind_view_model_instance(&mut self, view_model_instance: CoreHandle) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(|instance| {
                instance.bind_view_model_instance_handle(view_model_instance)
            });
        }
    }

    pub fn data_context(&mut self, data_context: RuntimeDataContextHandle) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(|instance| instance.set_data_context_handle(data_context));
        }
    }

    pub fn clear_data_context(&mut self) {
        if let Some(instance) = &self.instance {
            instance.with_instance_mut(StateMachineInstance::clear_data_context);
        }
    }

    pub fn try_change_state(&mut self) -> bool {
        self.instance.as_ref().is_some_and(|instance| {
            instance.with_instance_mut(StateMachineInstance::try_change_state)
        })
    }

    pub fn release_dependencies(&mut self) {
        self.instance = None;
    }
}

impl NestedAnimationBehavior for NestedStateMachine {
    fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool {
        Self::advance(self, elapsed_seconds, new_frame)
    }

    fn initialize_animation(&mut self, artboard: RuntimeArtboardInstanceWeakHandle) {
        Self::initialize_animation(self, artboard);
    }

    fn release_dependencies(&mut self) {
        Self::release_dependencies(self);
    }
}
