//! Factory and deferred-canvas routing from upstream e949498e.
//!
//! The source forward-declared pointers retain the same context/canvas identity
//! through closure-scoped Rust owners, without importing a concrete backend.

use std::{cell::RefCell, rc::Rc};

use crate::{ColorInt, RenderCanvas, RenderImage, Renderer};

pub type OreContextHandle = Rc<RefCell<dyn nuxie_ore_metal::context::ContextApi>>;
pub type RenderCanvasHandle = Rc<RefCell<Box<dyn RenderCanvas>>>;
pub type DeferredCanvasHostHandle = Rc<RefCell<dyn DeferredCanvasHost>>;

/// Keep the typed canvas owner behind the lower ORE forward-declaration seam.
pub fn canvas_texture_info(
    canvas: &RenderCanvasHandle,
) -> nuxie_ore_metal::context::CanvasTextureInfo {
    let value = canvas.borrow();
    let mut info =
        value
            .ore_texture_info()
            .unwrap_or(nuxie_ore_metal::context::CanvasTextureInfo {
                canvas: std::ptr::null_mut(),
                texture: std::ptr::null_mut(),
                width: value.width(),
                height: value.height(),
                owner: None,
            });
    // A backend may project through a retained typed bridge instead of the
    // source allocation itself. Keep that projection alongside the canvas.
    info.owner = Some(Rc::new((Rc::clone(canvas), info.owner.take())));
    info
}

/// Recover the typed source canvas while leaving its backend projection alive.
pub fn canvas_texture_owner(
    info: &nuxie_ore_metal::context::CanvasTextureInfo,
) -> Option<RenderCanvasHandle> {
    info.owner
        .as_ref()?
        .downcast_ref::<(RenderCanvasHandle, Option<Rc<dyn std::any::Any>>)>()
        .map(|(canvas, _)| Rc::clone(canvas))
}

/// `renderer/cmd/deferred_canvas_host.hpp`.
///
/// The returned renderer is a scoped proxy into the host's recording state.
/// The scripting owner invalidates its Lua renderer before ending the content
/// bracket, exactly as it does for an immediate canvas frame.
pub trait DeferredCanvasHost {
    /// Allocates the canvas whose content this host is about to take. A
    /// recording host can hand back an unbacked canvas and leave the pixels to
    /// whoever replays, while a host that draws immediately must return one
    /// with a real texture. `None` means this host cannot provide one, and the
    /// caller draws normally instead.
    fn make_content_canvas(&mut self, width: u32, height: u32) -> Option<RenderCanvasHandle>;

    /// The image a composite should sample to get this canvas's pixels.
    /// Usually the canvas's own render image, but a backend that stores canvas
    /// textures bottom-up can hand back a Y-flipped companion instead.
    fn content_canvas_image(&mut self, canvas: &RenderCanvasHandle) -> Option<Rc<dyn RenderImage>>;

    /// A renderer to issue the composite through, or `None` to use the one
    /// that was drawing. The caller re-applies the current transform, since a
    /// fresh renderer shares no state with the original.
    fn composite_renderer(&mut self) -> Option<Box<dyn Renderer>> {
        None
    }

    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear_color: ColorInt,
    ) -> Option<Box<dyn Renderer>>;
    fn end_canvas_content(&mut self, canvas: &RenderCanvasHandle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Factory, NullFactory, RenderCanvasError, RenderCanvasFrame, RenderImage};
    use std::cell::Cell;

    struct ProjectionDrop(Rc<Cell<bool>>);
    impl Drop for ProjectionDrop {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    struct ProjectingCanvas(Rc<Cell<bool>>);
    impl RenderCanvas for ProjectingCanvas {
        fn is_backed(&self) -> bool {
            true
        }
        fn width(&self) -> u32 {
            3
        }
        fn height(&self) -> u32 {
            4
        }
        fn render_image(&self) -> Rc<dyn RenderImage> {
            Rc::from(NullFactory::new().decode_image(&[]).unwrap())
        }
        fn ore_texture_info(&self) -> Option<nuxie_ore_metal::context::CanvasTextureInfo> {
            Some(nuxie_ore_metal::context::CanvasTextureInfo {
                canvas: std::ptr::null_mut(),
                texture: std::ptr::null_mut(),
                width: self.width(),
                height: self.height(),
                owner: Some(Rc::new(ProjectionDrop(self.0.clone()))),
            })
        }
        fn begin_frame(
            &mut self,
            _: ColorInt,
        ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
            Err(RenderCanvasError::unsupported())
        }
    }

    #[test]
    fn canvas_packet_retains_both_typed_canvas_and_backend_projection() {
        let dropped = Rc::new(Cell::new(false));
        let canvas: RenderCanvasHandle =
            Rc::new(RefCell::new(Box::new(ProjectingCanvas(dropped.clone()))));
        let info = canvas_texture_info(&canvas);
        assert!(
            !dropped.get(),
            "forming the packet must not drop the bridge"
        );
        let recovered = canvas_texture_owner(&info).expect("typed canvas owner");
        assert!(Rc::ptr_eq(&canvas, &recovered));
        drop(info);
        assert!(
            dropped.get(),
            "the packet owns the independent bridge retain"
        );
    }
}
