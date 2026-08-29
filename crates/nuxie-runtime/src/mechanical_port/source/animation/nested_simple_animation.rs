use crate::mechanical_port::source::{
    animation::nested_animation::NestedAnimationBehavior,
    artboard::RuntimeArtboardInstanceWeakHandle,
    generated::animation::nested_simple_animation_base::NestedSimpleAnimationBase,
};

#[derive(Default)]
pub struct NestedSimpleAnimation {
    pub base: NestedSimpleAnimationBase,
}

impl NestedAnimationBehavior for NestedSimpleAnimation {
    fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool {
        Self::advance(self, elapsed_seconds, new_frame)
    }

    fn animation_initializer(
        &self,
    ) -> crate::mechanical_port::source::animation::nested_animation::NestedAnimationInitializer
    {
        |owner, artboard| {
            owner
                .with_downcast_mut::<Self, _>(|owner| {
                    owner.base.base.initialize_animation(artboard)
                })
                .expect("live NestedSimpleAnimation");
        }
    }

    fn release_dependencies(&mut self) {
        self.base.base.release_dependencies();
    }
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
