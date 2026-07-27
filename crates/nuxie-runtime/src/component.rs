use super::ArtboardInstance;
use crate::components::ComponentHandle;

impl ArtboardInstance {
    /// C++ `Component::hitTestPoint` walks to the concrete parent while
    /// preserving `skipOnUnclipped` and clearing the primary-hit marker
    /// (`src/component.cpp:97-105`).
    pub(super) fn base_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        let Some(parent) = self
            .objects
            .component(component)
            .and_then(|component| component.parent)
        else {
            return true;
        };
        self.component_hit_test_point(parent, position, skip_on_unclipped, false)
    }
}
