use crate::mechanical_port::source::{
    artboard::{ArtboardInstance, RuntimeArtboardInstanceHandle},
    artboard_host::ArtboardHost,
    core::CoreHandle,
    layout_component::LayoutComponent,
};

pub trait StyleOverrideProvider {
    fn is_row(&self) -> bool;
    fn is_stack(&self) -> bool;
    fn instance_height_scale_type(&self) -> u32;
    fn instance_width_scale_type(&self) -> u32;
    fn instance_height_units_value(&self) -> u32;
    fn instance_width_units_value(&self) -> u32;
    fn instance_height(&self) -> f32;
    fn instance_width(&self) -> f32;
    /// List overrides use the row's record. Nested copies supply their own.
    fn css_item_record(&self, artboard: &RuntimeArtboardInstanceHandle) -> Option<CoreHandle> {
        artboard.with_artboard(|artboard| artboard.css_record())
    }
    fn mark_hosting_layout_dirty(&mut self, artboard: &RuntimeArtboardInstanceHandle);
    /// The provider can be the currently borrowed host of the sized Artboard.
    fn borrowed_artboard_host(&mut self) -> Option<&mut dyn ArtboardHost>;
}

pub struct StyleOverrider<T: StyleOverrideProvider>(std::marker::PhantomData<fn() -> T>);
impl<T: StyleOverrideProvider> StyleOverrider<T> {
    pub fn new(_provider: &mut T) -> Self {
        Self::detached()
    }
    pub fn detached() -> Self {
        Self(std::marker::PhantomData)
    }
    pub fn attach(&mut self, _provider: &mut T) {}

    fn update_css_override(provider: &mut T, artboard: &RuntimeArtboardInstanceHandle) -> bool {
        let record = artboard
            .with_artboard(|artboard| artboard.css_record())
            .and_then(|_| provider.css_item_record(artboard));
        let css = record.is_some();
        if css {
            // Parent context applies even when CSS owns instance sizing.
            LayoutComponent::set_parent_is_row_with_host_occurrence(
                &artboard.core_handle(),
                provider.is_row(),
                provider.borrowed_artboard_host(),
            );
        }
        LayoutComponent::set_hosted_css_item_occurrence(
            &artboard.core_handle(),
            record,
            provider.borrowed_artboard_host(),
        );
        if css {
            provider.mark_hosting_layout_dirty(artboard);
        }
        css
    }

    pub fn update_height_override(provider: &mut T, artboard: &RuntimeArtboardInstanceHandle) {
        let is_row = provider.is_row();
        LayoutComponent::set_parent_is_stack_with_host_occurrence(
            &artboard.core_handle(),
            provider.is_stack(),
            provider.borrowed_artboard_host(),
        );
        if Self::update_css_override(provider, artboard) {
            return;
        }
        if provider.instance_height_scale_type() == 0 {
            LayoutComponent::set_height_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                false,
                provider.borrowed_artboard_host(),
            );
            let height = artboard
                .with_artboard(|artboard| Self::actual_instance_height_for(provider, artboard));
            let units = provider.instance_height_units_value() as i32;
            LayoutComponent::height_override_occurrence(
                &artboard.core_handle(),
                height,
                units,
                is_row,
                provider.borrowed_artboard_host(),
            );
        } else if provider.instance_height_scale_type() == 1 {
            LayoutComponent::set_height_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                false,
                provider.borrowed_artboard_host(),
            );
            let height = artboard
                .with_artboard(|artboard| Self::actual_instance_height_for(provider, artboard));
            LayoutComponent::height_override_occurrence(
                &artboard.core_handle(),
                height,
                3,
                is_row,
                provider.borrowed_artboard_host(),
            );
        } else if provider.instance_width_scale_type() == 2 {
            // Preserve the pinned width-scale check in the height branch.
            LayoutComponent::set_height_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                true,
                provider.borrowed_artboard_host(),
            );
        }
        provider.mark_hosting_layout_dirty(artboard);
    }
    pub fn update_width_override(provider: &mut T, artboard: &RuntimeArtboardInstanceHandle) {
        let is_row = provider.is_row();
        LayoutComponent::set_parent_is_stack_with_host_occurrence(
            &artboard.core_handle(),
            provider.is_stack(),
            provider.borrowed_artboard_host(),
        );
        if Self::update_css_override(provider, artboard) {
            return;
        }
        if provider.instance_width_scale_type() == 0 {
            LayoutComponent::set_width_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                false,
                provider.borrowed_artboard_host(),
            );
            let width = artboard
                .with_artboard(|artboard| Self::actual_instance_width_for(provider, artboard));
            let units = provider.instance_width_units_value() as i32;
            LayoutComponent::width_override_occurrence(
                &artboard.core_handle(),
                width,
                units,
                is_row,
                provider.borrowed_artboard_host(),
            );
        } else if provider.instance_width_scale_type() == 1 {
            LayoutComponent::set_width_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                false,
                provider.borrowed_artboard_host(),
            );
            let width = artboard
                .with_artboard(|artboard| Self::actual_instance_width_for(provider, artboard));
            LayoutComponent::width_override_occurrence(
                &artboard.core_handle(),
                width,
                3,
                is_row,
                provider.borrowed_artboard_host(),
            );
        } else if provider.instance_width_scale_type() == 2 {
            LayoutComponent::set_width_intrinsically_size_override_occurrence(
                &artboard.core_handle(),
                true,
                provider.borrowed_artboard_host(),
            );
        }
        provider.mark_hosting_layout_dirty(artboard);
    }
    fn actual_instance_width_for(provider: &T, artboard: &ArtboardInstance) -> f32 {
        if provider.instance_width() == -1.0 {
            artboard.original_width()
        } else {
            provider.instance_width()
        }
    }
    fn actual_instance_height_for(provider: &T, artboard: &ArtboardInstance) -> f32 {
        if provider.instance_height() == -1.0 {
            artboard.original_height()
        } else {
            provider.instance_height()
        }
    }
    pub fn actual_instance_width(provider: &T, artboard: &ArtboardInstance) -> f32 {
        Self::actual_instance_width_for(provider, artboard)
    }
    pub fn actual_instance_height(provider: &T, artboard: &ArtboardInstance) -> f32 {
        Self::actual_instance_height_for(provider, artboard)
    }
}
