use super::ArtboardInstance;
use crate::components::ComponentHandle;

impl ArtboardInstance {
    /// Direct C++ `Drawable::hitTestPoint` owner. Ordinary
    /// `Drawable::hittableComponent` returns `this`; proxy callers have
    /// already supplied the proxy target at the public dispatch boundary
    /// (`src/drawable.cpp:62-77`).
    pub(super) fn drawable_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        let hidden = owner.is_collapsed()
            || owner
                .concrete
                .drawable
                .as_ref()
                .and_then(|drawable| drawable.drawable_flags_property_key)
                .and_then(|key| self.objects.component_uint_property(component, key))
                .is_some_and(|flags| flags & 1 != 0);
        if hidden {
            return false;
        }

        self.base_component_hit_test_point(component, position, skip_on_unclipped, is_primary_hit)
    }
}
