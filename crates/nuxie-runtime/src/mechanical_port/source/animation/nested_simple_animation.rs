use crate::mechanical_port::source::generated::animation::nested_simple_animation_base::NestedSimpleAnimationBase;

#[derive(Default)]
pub struct NestedSimpleAnimation {
    pub base: NestedSimpleAnimationBase,
}

impl NestedSimpleAnimation {
    pub fn advance(&mut self, elapsed_seconds: f32, _new_frame: bool) -> bool {
        let is_playing = self.base.is_playing();
        let speed = self.base.speed();
        let mix = self.base.base.mix();
        let Some(animation_instance) = self.base.base.animation_instance_mut() else {
            return false;
        };

        let mut keep_going = false;
        if is_playing {
            keep_going = animation_instance.advance_and_report_to_self(elapsed_seconds * speed);
        }
        if mix != 0.0 {
            animation_instance.apply(mix);
        }
        keep_going
    }
}
