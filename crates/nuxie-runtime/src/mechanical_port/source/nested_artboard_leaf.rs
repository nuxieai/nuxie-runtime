use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    generated::nested_artboard_leaf_base::{
        NestedArtboardLeafBase, NestedArtboardLeafBaseCallbacks,
    },
    layout::{Alignment, Fit},
    renderer::compute_alignment,
};

#[derive(Default)]
pub struct NestedArtboardLeaf {
    pub base: NestedArtboardLeafBase,
}

struct CloneCallbacks;

impl NestedArtboardLeafBaseCallbacks for CloneCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

impl NestedArtboardLeaf {
    pub fn clone_leaf(&self) -> Self {
        let mut nested_artboard = self.base.clone_into(&mut CloneCallbacks);
        nested_artboard.base.base.set_file(self.base.base.file());
        if let Some(referenced) = self.base.base.referenced_artboard() {
            let instance = referenced.instance();
            nested_artboard.base.base.set_referenced_artboard(instance);
        }
        nested_artboard
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        if !value.contains(ComponentDirt::WORLD_TRANSFORM) {
            return;
        }
        let Some(artboard) = self.base.base.artboard_instance_mut() else {
            return;
        };

        let bounds = self
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
            .map(|layout| layout.local_bounds())
            .unwrap_or_else(|| artboard.bounds());

        let fit = match self.base.fit() {
            0 => Fit::Fill,
            1 => Fit::Contain,
            2 => Fit::Cover,
            3 => Fit::FitWidth,
            4 => Fit::FitHeight,
            5 => Fit::None,
            6 => Fit::ScaleDown,
            7 => Fit::Layout,
            value => panic!("invalid fit {value}"),
        };
        if fit == Fit::Layout {
            let mut resized = false;
            if artboard.width() != bounds.width() {
                artboard.set_width(bounds.width());
                resized = true;
            }
            if artboard.height() != bounds.height() {
                artboard.set_height(bounds.height());
                resized = true;
            }
            if resized {
                artboard.update_pass(false);
            }
        }

        let view_transform = compute_alignment(
            fit,
            Alignment::new(self.base.alignment_x(), self.base.alignment_y()),
            &bounds,
            &artboard.bounds(),
            1.0,
        );
        *self.base.base.mutable_world_transform() *= view_transform;
    }

    pub fn fit_changed(&mut self) {
        self.base.base.mark_world_transform_dirty();
    }
}

impl NestedArtboardLeafBaseCallbacks for NestedArtboardLeaf {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
    fn fit_changed(&mut self) {
        NestedArtboardLeaf::fit_changed(self);
    }
}

impl std::ops::Deref for NestedArtboardLeaf {
    type Target = NestedArtboardLeafBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedArtboardLeaf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
