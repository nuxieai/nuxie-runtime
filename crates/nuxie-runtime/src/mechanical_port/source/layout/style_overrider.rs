use crate::mechanical_port::source::artboard::ArtboardInstance;

pub trait StyleOverrideProvider {
    fn is_row(&self) -> bool;
    fn instance_height_scale_type(&self) -> u32;
    fn instance_width_scale_type(&self) -> u32;
    fn instance_height_units_value(&self) -> u32;
    fn instance_width_units_value(&self) -> u32;
    fn instance_height(&self) -> f32;
    fn instance_width(&self) -> f32;
    fn mark_hosting_layout_dirty(&mut self, artboard: &mut ArtboardInstance);
}

pub struct StyleOverrider<T: StyleOverrideProvider> {
    style_provider: Option<*mut T>,
}
impl<T: StyleOverrideProvider> StyleOverrider<T> {
    pub fn new(provider: &mut T) -> Self {
        Self {
            style_provider: Some(provider),
        }
    }
    pub fn detached() -> Self {
        Self {
            style_provider: None,
        }
    }
    pub fn attach(&mut self, provider: &mut T) {
        self.style_provider = Some(provider);
    }
    fn provider(&mut self) -> &mut T {
        unsafe { &mut *self.style_provider.unwrap() }
    }

    pub fn update_height_override(&mut self, artboard: &mut ArtboardInstance) {
        let provider = self.provider();
        let is_row = provider.is_row();
        if provider.instance_height_scale_type() == 0 {
            artboard.set_height_intrinsically_size_override(false);
            artboard.height_override(
                Self::actual_instance_height_for(provider, artboard),
                provider.instance_height_units_value(),
                is_row,
            );
        } else if provider.instance_height_scale_type() == 1 {
            artboard.set_height_intrinsically_size_override(false);
            artboard.height_override(
                Self::actual_instance_height_for(provider, artboard),
                3,
                is_row,
            );
        } else if provider.instance_width_scale_type() == 2 {
            // Preserve the pinned width-scale check in the height branch.
            artboard.set_height_intrinsically_size_override(true);
        }
        provider.mark_hosting_layout_dirty(artboard);
    }
    pub fn update_width_override(&mut self, artboard: &mut ArtboardInstance) {
        let provider = self.provider();
        let is_row = provider.is_row();
        if provider.instance_width_scale_type() == 0 {
            artboard.set_width_intrinsically_size_override(false);
            artboard.width_override(
                Self::actual_instance_width_for(provider, artboard),
                provider.instance_width_units_value(),
                is_row,
            );
        } else if provider.instance_width_scale_type() == 1 {
            artboard.set_width_intrinsically_size_override(false);
            artboard.width_override(
                Self::actual_instance_width_for(provider, artboard),
                3,
                is_row,
            );
        } else if provider.instance_width_scale_type() == 2 {
            artboard.set_width_intrinsically_size_override(true);
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
    pub fn actual_instance_width(&mut self, artboard: &ArtboardInstance) -> f32 {
        Self::actual_instance_width_for(self.provider(), artboard)
    }
    pub fn actual_instance_height(&mut self, artboard: &ArtboardInstance) -> f32 {
        Self::actual_instance_height_for(self.provider(), artboard)
    }
}
