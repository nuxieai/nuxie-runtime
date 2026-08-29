use crate::mechanical_port::source::generated::animation::animation_base::AnimationBase;

#[derive(Default)]
pub struct Animation {
    pub base: AnimationBase,
}
impl std::ops::Deref for Animation {
    type Target = AnimationBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for Animation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::animation_base::AnimationBaseCallbacks
    for Animation
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
