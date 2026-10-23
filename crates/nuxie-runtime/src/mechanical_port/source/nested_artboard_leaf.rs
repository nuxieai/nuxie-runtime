use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::nested_artboard_leaf_base::{
        NestedArtboardLeafBase, NestedArtboardLeafBaseCallbacks,
    },
    layout::{Alignment, Fit},
    layout_component::content_sizing_layout,
    math::aabb::Aabb,
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
        self.base
            .base
            .clone_references_into(&mut nested_artboard.base.base);
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
                        parent.as_layout_component().map(|layout| {
                            if layout.is_css_mode() {
                                let content = layout.css_content_box();
                                if layout.is_leaf() {
                                    // CSS has already placed this drawing content.
                                    Aabb::from_ltwh(0.0, 0.0, content.width(), content.height())
                                } else {
                                    content
                                }
                            } else {
                                layout.local_bounds()
                            }
                        })
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
            // Layout fit is explicit host sizing even when the supplied
            // dimensions equal the stored values of an authored Hug axis.
            let resized = artboard.set_size_axes(Some(bounds.width()), Some(bounds.height()), true);
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
                owner
                    .base
                    .base
                    .with_world_transform_mut(|world| *world *= view_transform);
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
