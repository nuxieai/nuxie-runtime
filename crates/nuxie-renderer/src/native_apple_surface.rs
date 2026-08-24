//! Native Metal adaptation for a caller-owned Core Animation drawable.
//!
//! This seam keeps caller-owned presentation separate from the exact Metal
//! renderer's command and resource ownership.

use objc2::runtime::ProtocolObject;
use objc2_quartz_core::CAMetalDrawable;

use crate::{NativeMetalDrawableFrame, NativeMetalFactory, RendererError};

impl NativeMetalFactory {
    /// Begins a native Metal frame from one caller-acquired drawable. The
    /// texture is derived from that same object so rendering and presentation
    /// cannot accidentally target different native owners.
    pub fn begin_drawable_frame<'a>(
        &self,
        drawable: &'a ProtocolObject<dyn CAMetalDrawable>,
        clear_color: u32,
    ) -> Result<NativeMetalDrawableFrame<'a>, RendererError> {
        self.begin_drawable_frame_parts(drawable.as_ref(), drawable.texture(), clear_color)
    }
}
