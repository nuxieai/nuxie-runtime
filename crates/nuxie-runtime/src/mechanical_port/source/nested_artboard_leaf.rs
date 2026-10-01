use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::nested_artboard_leaf_base::{
        NestedArtboardLeafBase, NestedArtboardLeafBaseCallbacks,
    },
    layout::{Alignment, Fit},
    layout_component::content_sizing_layout,
    renderer::compute_alignment,
};

#[derive(Default)]
pub struct NestedArtboardLeaf {
    pub base: NestedArtboardLeafBase,
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    NestedArtboardLeaf,
    |owner, dirt| {
        crate::mechanical_port::source::transform_component::update_transform_super::<Self>(
            owner, dirt,
        );
        if owner.is_alive() {
            crate::mechanical_port::source::nested_artboard::NestedArtboard::update_after_transform_occurrence(owner, dirt);
            if owner.is_alive() {
                Self::update_after_nested_artboard_super_occurrence(owner, dirt);
            }
        }
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

impl NestedArtboardLeaf {
    pub fn clone_leaf(&self) -> Self {
        let mut nested_artboard = NestedArtboardLeafBase::clone_into(self);
        nested_artboard.base.base.set_file(self.base.base.file());
        // Upstream instances the currently referenced Artboard, which is the
        // mounted instance after mounting and the authored source before it.
        let referenced = match self.base.base.artboard_instance_handle(0) {
            Some(instance) => Some(instance.core_handle()),
            None => self.base.base.source_artboard(),
        };
        if let Some(referenced) = referenced {
            if let Some(instance) =
                crate::mechanical_port::source::artboard::Artboard::nested_instance_from_handle(
                    &referenced,
                )
            {
                nested_artboard
                    .base
                    .base
                    .referenced_artboard_instance(instance);
            }
        }
        nested_artboard
    }

    pub(crate) fn update_after_nested_artboard_super_occurrence(
        owner: &CoreHandle,
        value: ComponentDirt,
    ) {
        if !value.contains(ComponentDirt::WORLD_TRANSFORM) {
            return;
        }
        let Some(artboard) = owner
            .with_downcast::<Self, _>(|owner| owner.base.base.artboard_instance_handle(0))
            .expect("live NestedArtboardLeaf")
        else {
            return;
        };

        let parent = owner
            .with_downcast::<Self, _>(|owner| owner.base.base.parent_handle())
            .expect("live NestedArtboardLeaf");
        let fit_to_layout_parent = owner
            .with_downcast::<Self, _>(|owner| owner.base.fit_to_layout_parent())
            .expect("live NestedArtboardLeaf");
        let sizing_layout = if fit_to_layout_parent {
            content_sizing_layout(parent)
        } else {
            parent.filter(|parent| {
                parent
                    .with(|parent| parent.as_layout_component().is_some())
                    .unwrap_or(false)
            })
        };
        let bounds = sizing_layout
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_layout_component()
                            .map(|layout| layout.local_bounds())
                    })
                    .flatten()
            })
            .unwrap_or_else(|| artboard.with_artboard(|artboard| artboard.bounds()));

        let fit = match owner
            .with_downcast::<Self, _>(|owner| owner.base.fit())
            .expect("live NestedArtboardLeaf")
        {
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
            let resized = artboard.with_artboard_mut(|artboard| {
                let mut resized = false;
                if artboard.width() != bounds.width() {
                    artboard.set_width(bounds.width());
                    resized = true;
                }
                if artboard.height() != bounds.height() {
                    artboard.set_height(bounds.height());
                    resized = true;
                }
                resized
            });
            if resized {
                artboard.update_pass(false);
            }
        }

        let alignment = owner
            .with_downcast::<Self, _>(|owner| {
                Alignment::new(owner.base.alignment_x(), owner.base.alignment_y())
            })
            .expect("live NestedArtboardLeaf");
        let view_transform = compute_alignment(
            fit,
            alignment,
            &bounds,
            &artboard.with_artboard(|artboard| artboard.bounds()),
            1.0,
        );
        owner
            .with_downcast_mut::<Self, _>(|owner| {
                *owner.base.base.mutable_world_transform() *= view_transform;
            })
            .expect("live NestedArtboardLeaf");
    }

    pub fn fit_changed(&mut self) {
        self.base.base.mark_world_transform_dirty();
    }

    pub fn fit_to_layout_parent_changed(&mut self) {
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
    fn fit_to_layout_parent_changed(&mut self) {
        NestedArtboardLeaf::fit_to_layout_parent_changed(self);
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
