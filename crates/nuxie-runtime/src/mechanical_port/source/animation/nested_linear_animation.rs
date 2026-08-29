use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        nested_animation::NestedAnimationBehavior,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    generated::animation::nested_linear_animation_base::NestedLinearAnimationBase,
};

pub struct NestedLinearAnimation {
    pub base: NestedLinearAnimationBase,
    animation_instance: Option<LinearAnimationInstance>,
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

    pub fn initialize_animation(&mut self, artboard: RuntimeArtboardInstanceWeakHandle) {
        let animation_id = self.base.animation_id() as usize;
        let animation = artboard
            .with_artboard(|artboard| artboard.animation_handle_at(animation_id))
            .flatten();
        self.animation_instance =
            animation.map(|animation| LinearAnimationInstance::new(animation, artboard, 1.0));
    }

    /// Pinned `NestedLinearAnimation::releaseDependencies` is intentionally a
    /// no-op. The occurrence retains its animation instance until it is
    /// initialized again or destroyed.
    pub fn release_dependencies(&mut self) {}

    pub fn animation_instance(&self) -> Option<&LinearAnimationInstance> {
        self.animation_instance.as_ref()
    }

    pub fn animation_instance_mut(&mut self) -> Option<&mut LinearAnimationInstance> {
        self.animation_instance.as_mut()
    }
}

impl NestedAnimationBehavior for NestedLinearAnimation {
    /// This is the pinned abstract base for the simple and remap variants.
    /// Their concrete generated owners override advancing; invoking the
    /// embedded base directly therefore performs no advance or apply.
    fn advance(&mut self, _elapsed_seconds: f32, _new_frame: bool) -> bool {
        false
    }

    fn animation_initializer(
        &self,
    ) -> crate::mechanical_port::source::animation::nested_animation::NestedAnimationInitializer
    {
        |owner, artboard| {
            owner
                .with_mut(|owner| {
                    owner
                        .as_nested_linear_animation_mut()
                        .expect("NestedLinearAnimation owner")
                        .initialize_animation(artboard)
                })
                .expect("live nested animation");
        }
    }

    fn release_dependencies(&mut self) {
        Self::release_dependencies(self);
    }
}

impl std::ops::Deref for NestedLinearAnimation {
    type Target = NestedLinearAnimationBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for NestedLinearAnimation
{
    fn notify_property_changed(&mut self, key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(&mut self.base, key);
    }
}

impl std::ops::DerefMut for NestedLinearAnimation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl crate::mechanical_port::source::generated::nested_animation_base::NestedAnimationBaseCallbacks
    for NestedLinearAnimation
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}

impl crate::mechanical_port::source::generated::animation::nested_linear_animation_base::NestedLinearAnimationBaseCallbacks
    for NestedLinearAnimation
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
