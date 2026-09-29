//! Reusable textures for GPU-to-GPU uploads of external images, such as a
//! browser video's decoded frames or an Android decoder's hardware buffers.

use std::rc::Rc;

/// Textures that receive one stream of external images, such as one video's
/// decoded frames, through `NativeWebGpuFactory::copy_external_image` or
/// `NativeVulkanFactory::import_hardware_buffer`. The stream never rewrites an
/// image someone still holds, so an image handed out earlier keeps its pixels.
/// In steady state it alternates between two textures and allocates again only
/// when the image size changes.
#[cfg(any(
    all(
        feature = "renderer-webgpu",
        target_arch = "wasm32",
        target_os = "unknown"
    ),
    all(feature = "native-vulkan-experimental", target_os = "android")
))]
#[derive(Default)]
pub struct ExternalImageTextures(
    pub(crate)  TextureRing<
        crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle,
    >,
);

/// One image stream's textures, oldest first. The next image is written only
/// into a texture nobody outside the ring holds, so an image handed out
/// earlier keeps its pixels for as long as anyone holds it.
pub(crate) struct TextureRing<T> {
    textures: Vec<Rc<T>>,
}

impl<T> Default for TextureRing<T> {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
        }
    }
}

impl<T> TextureRing<T> {
    /// Two textures let a stream write its next image while the current one
    /// is still on screen.
    const CAPACITY: usize = 2;

    /// The texture to write the stream's next image into; it becomes the
    /// newest. `fits` drops textures of another size or device. A texture is
    /// reused only when the ring holds its only `Rc` and `unshared` finds no
    /// deeper owner. Otherwise `allocate` makes one, and when the ring is full
    /// its oldest texture leaves it, staying alive for whoever still holds it.
    pub(crate) fn next<E>(
        &mut self,
        fits: impl Fn(&T) -> bool,
        unshared: impl Fn(&T) -> bool,
        allocate: impl FnOnce() -> Result<T, E>,
    ) -> Result<Rc<T>, E> {
        self.textures.retain(|texture| fits(texture));
        let reusable = self
            .textures
            .iter()
            .position(|texture| Rc::strong_count(texture) == 1 && unshared(texture));
        let texture = match reusable {
            Some(index) => self.textures.remove(index),
            None => {
                let texture = Rc::new(allocate()?);
                if self.textures.len() >= Self::CAPACITY {
                    self.textures.remove(0);
                }
                texture
            }
        };
        self.textures.push(Rc::clone(&texture));
        Ok(texture)
    }
}

#[cfg(test)]
mod tests {
    use super::TextureRing;
    use std::cell::Cell;
    use std::convert::Infallible;
    use std::rc::Rc;

    struct Texture {
        id: usize,
        size: u32,
        // Stands in for an owner the ring cannot count through its Rc, such
        // as a draw or canvas import retaining the texture itself.
        retained_inside: Cell<bool>,
    }

    struct Stream {
        ring: TextureRing<Texture>,
        allocated: usize,
    }

    impl Stream {
        fn new() -> Self {
            Self {
                ring: TextureRing::default(),
                allocated: 0,
            }
        }

        fn next(&mut self, size: u32) -> Rc<Texture> {
            let allocated = &mut self.allocated;
            self.ring
                .next(
                    |texture| texture.size == size,
                    |texture| !texture.retained_inside.get(),
                    || {
                        *allocated += 1;
                        Ok::<_, Infallible>(Texture {
                            id: *allocated,
                            size,
                            retained_inside: Cell::new(false),
                        })
                    },
                )
                .unwrap()
        }
    }

    #[test]
    fn a_shown_image_alternates_between_two_textures() {
        let mut stream = Stream::new();
        // The scene holds each presented image until the next one replaces it.
        let mut shown = stream.next(64);
        let mut ids = vec![shown.id];
        for _ in 0..6 {
            let next = stream.next(64);
            assert!(!Rc::ptr_eq(&next, &shown), "wrote into the shown image");
            shown = next;
            ids.push(shown.id);
        }
        assert_eq!(stream.allocated, 2);
        assert_eq!(ids, [1, 2, 1, 2, 1, 2, 1]);
    }

    #[test]
    fn a_retained_image_is_never_rewritten() {
        let mut stream = Stream::new();
        let snapshot = stream.next(64);
        let shown = stream.next(64);
        // Both textures are held: the next image needs a third, and the ring
        // lets go of the oldest instead of growing.
        let next = stream.next(64);
        assert_eq!(stream.allocated, 3);
        assert!(!Rc::ptr_eq(&next, &snapshot) && !Rc::ptr_eq(&next, &shown));
        drop(shown);
        let after = stream.next(64);
        assert_eq!(after.id, 2, "reuses the released texture");
        drop(next);
        drop(after);
        // The snapshot left the ring; releasing it later frees it rather than
        // bringing it back.
        drop(snapshot);
        stream.next(64);
        stream.next(64);
        assert_eq!(stream.allocated, 3);
    }

    #[test]
    fn an_owner_inside_the_texture_blocks_reuse() {
        let mut stream = Stream::new();
        let first = stream.next(64);
        first.retained_inside.set(true);
        drop(first);
        let second = stream.next(64);
        assert_eq!(second.id, 2);
        drop(second);
        let third = stream.next(64);
        assert_eq!(
            third.id, 2,
            "the free texture is reused, the held one is not"
        );
        assert_eq!(stream.allocated, 2);
    }

    #[test]
    fn a_new_size_replaces_the_textures() {
        let mut stream = Stream::new();
        let shown = stream.next(64);
        drop(stream.next(64));
        drop(shown);
        assert_eq!(stream.allocated, 2);
        let resized = stream.next(128);
        assert_eq!((resized.id, resized.size), (3, 128));
        drop(resized);
        assert_eq!(stream.next(128).id, 3);
        assert_eq!(stream.allocated, 3);
    }
}
