use crate::mechanical_port::source::{
    artboard::{ArtboardInstance, RuntimeArtboardInstanceHandle},
    artboard_host::ArtboardHost,
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

    pub fn update_height_override(provider: &mut T, artboard: &RuntimeArtboardInstanceHandle) {
        let is_row = provider.is_row();
        LayoutComponent::set_parent_is_stack_with_host_occurrence(
            &artboard.core_handle(),
            provider.is_stack(),
            provider.borrowed_artboard_host(),
        );
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
