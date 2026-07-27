use super::ArtboardInstance;
use crate::components::ComponentHandle;

impl ArtboardInstance {
    /// Direct C++ `LayoutComponent::hitTestPoint` owner. Its local-bounds
    /// check delegates to `Drawable::hitTestPoint` with
    /// `skipOnUnclipped=true` (`src/layout_component.cpp:49-80`).
    pub(super) fn layout_component_hit_test_point(
        &self,
        component: ComponentHandle,
        position: (f32, f32),
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        let Some(owner) = self.objects.component(component) else {
            return false;
        };
        let Some(layout) = owner.concrete.layout.as_ref() else {
            return false;
        };
        let world = owner.transform.world_transform;
        if world.determinant() == 0.0 {
            return false;
        }

        let clip = layout
            .clip_property_key
            .and_then(|key| self.objects.component_bool_property(component, key))
            .unwrap_or(owner.type_name == "Artboard" && self.clip);
        if !(skip_on_unclipped && !clip) {
            let mut local = world
                .invert_or_identity()
                .transform_point(position.0, position.1);
            let (_, _, width, height) = layout.constraint_bounds();
            if owner.type_name == "Artboard" && (self.origin_x != 0.0 || self.origin_y != 0.0) {
                local.0 += self.origin_x * width;
                local.1 += self.origin_y * height;
            }
            if local.0 < 0.0 || local.0 > width || local.1 < 0.0 || local.1 > height {
                return false;
            }
        }

        self.drawable_component_hit_test_point(component, position, true, is_primary_hit)
    }
}
