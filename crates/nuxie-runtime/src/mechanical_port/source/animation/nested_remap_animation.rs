use crate::mechanical_port::source::{
    artboard::RuntimeArtboardInstanceWeakHandle,
    generated::animation::nested_remap_animation_base::NestedRemapAnimationBase,
};

#[derive(Default)]
pub struct NestedRemapAnimation {
    pub base: NestedRemapAnimationBase,
}

impl NestedRemapAnimation {
    pub fn time_changed(&mut self) {
        let time = self.base.time();
        if let Some(animation_instance) = self.base.base.animation_instance_mut() {
            let local_time = animation_instance
                .global_to_local_seconds(animation_instance.duration_seconds() * time);
            animation_instance.set_time(local_time);
        }
    }

    pub fn initialize_animation(&mut self, artboard: RuntimeArtboardInstanceWeakHandle) {
        self.base.base.initialize_animation(artboard);
        self.time_changed();
    }

    pub fn advance(&mut self, _elapsed_seconds: f32, _new_frame: bool) -> bool {
        let mix = self.base.base.base.mix();
        if mix != 0.0 {
            if let Some(animation_instance) = self.base.base.animation_instance_mut() {
                animation_instance.apply(mix);
            }
        }
        false
    }
}
