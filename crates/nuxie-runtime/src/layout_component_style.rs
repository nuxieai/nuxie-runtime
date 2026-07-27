use super::ArtboardInstance;
use crate::animation::RuntimeInterpolator;
use crate::components::{ComponentDirt, ComponentHandle};
use crate::properties::property_key_for_name;

impl ArtboardInstance {
    /// C++ `LayoutComponentStyle::displayValueChanged -> displayChanged`
    /// forwards to the concrete retained LayoutComponent parent, whose
    /// `displayChanged` propagates collapse and dirties layout
    /// (`src/layout/layout_component_style.cpp:232-237,302`;
    /// `src/layout_component.cpp:1484-1492`).
    pub(super) fn propagate_layout_component_display_changed(
        &mut self,
        style_local_id: usize,
    ) -> bool {
        let Some(style) = self.component_handle(style_local_id) else {
            return false;
        };
        let Some(layout) = self.objects.component(style).and_then(|component| {
            (component.type_name == "LayoutComponentStyle")
                .then_some(component.parent)
                .flatten()
        }) else {
            return false;
        };
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };

        self.propagate_layout_component_display_collapse(layout_local)
            | self.add_dirt(layout_local, ComponentDirt::LAYOUT_STYLE, false)
    }

    pub(super) fn refresh_layout_component_animation_style(
        &mut self,
        style_local_id: usize,
    ) -> bool {
        let Some(style) = self.component_handle(style_local_id) else {
            return false;
        };
        let Some(layout) = self
            .objects
            .component(style)
            .and_then(|component| component.parent)
            .filter(|layout| {
                self.objects
                    .component(*layout)
                    .is_some_and(|component| component.concrete.layout.is_some())
            })
        else {
            return false;
        };
        let animation_style = property_key_for_name("LayoutComponentStyle", "animationStyleType")
            .and_then(|key| self.uint_property(style_local_id, key))
            .unwrap_or(0) as u8;
        let interpolation = property_key_for_name("LayoutComponentStyle", "interpolationType")
            .and_then(|key| self.uint_property(style_local_id, key))
            .unwrap_or(0) as u8;
        let interpolation_time = property_key_for_name("LayoutComponentStyle", "interpolationTime")
            .and_then(|key| self.double_property(style_local_id, key))
            .unwrap_or(0.0);
        let interpolator = property_key_for_name("LayoutComponentStyle", "interpolatorId")
            .and_then(|key| self.uint_property(style_local_id, key))
            .and_then(|local_id| usize::try_from(local_id).ok())
            .and_then(|local_id| self.slot(local_id))
            .and_then(|slot| self.runtime_file()?.object(slot.source_global_id as usize))
            .and_then(RuntimeInterpolator::from_object);
        let Some(layout_state) = self
            .objects
            .component(layout)
            .and_then(|component| component.concrete.layout.as_ref())
        else {
            return false;
        };
        layout_state.set_animation_style(
            animation_style,
            interpolation,
            interpolation_time,
            interpolator,
        );
        self.cascade_layout_component_animation_style(layout);
        let Some(layout_local) = self.component_local_id(layout) else {
            return false;
        };
        self.add_dirt(layout_local, ComponentDirt::LAYOUT_STYLE, false)
    }

    fn cascade_layout_component_animation_style(&self, parent: ComponentHandle) {
        let inherited = self
            .objects
            .component(parent)
            .and_then(|component| component.concrete.layout.as_ref())
            .map(|layout| {
                (
                    layout.effective_interpolation(),
                    layout.effective_interpolation_time(),
                    layout.effective_interpolator(),
                )
            })
            .unwrap_or((0, 0.0, None));
        let child_len = self.component_child_len(parent);
        for index in 0..child_len {
            let Some(child) = self.component_child_at(parent, index) else {
                continue;
            };
            let Some(layout) = self
                .objects
                .component(child)
                .and_then(|component| component.concrete.layout.as_ref())
            else {
                continue;
            };
            layout.set_inherited_animation_style(inherited.0, inherited.1, inherited.2);
            self.cascade_layout_component_animation_style(child);
        }
    }
}
