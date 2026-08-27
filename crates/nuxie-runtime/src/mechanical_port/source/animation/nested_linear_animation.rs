use crate::mechanical_port::source::generated::animation::nested_linear_animation_base::NestedLinearAnimationBase;

pub trait NestedLinearAnimationInstance {
    fn advance_and_report_to_self(&mut self, elapsed_seconds: f32) -> bool;
    fn apply(&mut self, mix: f32);
    fn duration_seconds(&self) -> f32;
    fn global_to_local_seconds(&self, seconds: f32) -> f32;
    fn set_time(&mut self, value: f32);
}

pub trait NestedLinearAnimationArtboard {
    fn make_linear_animation_instance(
        &mut self,
        animation_id: u32,
    ) -> Box<dyn NestedLinearAnimationInstance>;
}

pub struct NestedLinearAnimation {
    pub base: NestedLinearAnimationBase,
    animation_instance: Option<Box<dyn NestedLinearAnimationInstance>>,
}

impl Default for NestedLinearAnimation {
    fn default() -> Self {
        Self {
            base: NestedLinearAnimationBase::default(),
            animation_instance: None,
        }
    }
}

impl NestedLinearAnimation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize_animation(&mut self, artboard: &mut dyn NestedLinearAnimationArtboard) {
        self.animation_instance =
            Some(artboard.make_linear_animation_instance(self.base.base.base.animation_id()));
    }

    pub fn release_dependencies(&mut self) {}

    pub fn animation_instance(&self) -> Option<&dyn NestedLinearAnimationInstance> {
        self.animation_instance.as_deref()
    }

    pub fn animation_instance_mut(
        &mut self,
    ) -> Option<&mut (dyn NestedLinearAnimationInstance + 'static)> {
        self.animation_instance.as_deref_mut()
    }
}
