//! Direct owner for pinned `src/lua/renderer/lua_image.cpp`.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{AnyUserData, Error, Lua, Result, UserData, UserDataFields, UserDataMethods};
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap, RenderCanvasHandle, RenderImage};
use nuxie_runtime::{RuntimeImageAssetOwners, ScriptImage, ScriptImageAssets};

use super::logging_scripting_context::LoggingScriptingContext;

#[derive(Clone, Default)]
struct ScriptedImageAssetOwners(Option<Arc<RuntimeImageAssetOwners>>);

#[derive(Clone, Default)]
struct ScriptedImageAssets(ScriptImageAssets);

impl ScriptedImageAssetOwners {
    fn install(lua: &Lua) {
        if lua.app_data_ref::<Self>().is_none() {
            lua.set_app_data(Self::default());
        }
    }

    fn set(lua: &Lua, owners: Arc<RuntimeImageAssetOwners>) {
        lua.set_app_data(Self(Some(owners)));
    }

    fn for_lua(lua: &Lua) -> Option<Arc<RuntimeImageAssetOwners>> {
        lua.app_data_ref::<Self>()
            .and_then(|owners| owners.0.clone())
    }
}

/// One Lua Image wrapper, whether sourced from a File ImageAsset identity or
/// an already-created render-factory image.
///
pub(crate) struct ScriptedImage {
    image: Rc<dyn RenderImage>,
    // Set when this image is a canvas's backing, so Image:view() imports
    // through the backend's canvas sampling wrap rather than the raw texture.
    source_canvas: Option<RenderCanvasHandle>,
    cached_gpu_view: RefCell<Option<nuxie_ore_metal::gpu_resource::AnyResourceHandle>>,
}

impl ScriptedImage {
    pub(super) fn from_asset(lua: &Lua, identity: ScriptImage) -> Option<Self> {
        let asset_owners = ScriptedImageAssetOwners::for_lua(lua)?;
        let image = asset_owners.get(identity.asset_global_id())?;
        Some(Self {
            image,
            source_canvas: None,
            cached_gpu_view: RefCell::new(None),
        })
    }

    /// Construction seam used by factory-backed image producers. It does not
    /// decode or schedule work; the caller transfers an already-made image.
    pub(super) fn from_render_image(image: Box<dyn RenderImage>) -> Self {
        Self::from_render_image_rc(Rc::from(image))
    }

    pub(crate) fn from_render_image_rc(image: Rc<dyn RenderImage>) -> Self {
        Self {
            image,
            source_canvas: None,
            cached_gpu_view: RefCell::new(None),
        }
    }

    pub(crate) fn from_render_canvas(canvas: RenderCanvasHandle) -> Self {
        let image = canvas.borrow().render_image();
        Self {
            image,
            source_canvas: Some(canvas),
            cached_gpu_view: RefCell::new(None),
        }
    }

    pub(super) fn render_image(&self) -> Result<Rc<dyn RenderImage>> {
        Ok(Rc::clone(&self.image))
    }

    pub(super) fn with_render_image<R>(
        &self,
        callback: impl FnOnce(&dyn RenderImage) -> R,
    ) -> Result<R> {
        let image = self.render_image()?;
        Ok(callback(image.as_ref()))
    }
}

impl UserData for ScriptedImage {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            this.with_render_image(RenderImage::width)
        });
        fields.add_field_method_get("height", |_, this| {
            this.with_render_image(RenderImage::height)
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Upstream `image_index` returns `riveImageViewImpl` as a function, so
        // scripts call `image:view()`.
        methods.add_method("view", |lua, this, ()| {
            crate::gpu_canvas::ore::image_view(
                lua,
                this.render_image()?,
                this.source_canvas.as_ref(),
                &this.cached_gpu_view,
            )
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScriptedImageSampler(pub(crate) ImageSampler);

impl UserData for ScriptedImageSampler {}

pub(crate) fn install_image_globals(lua: &Lua) -> Result<()> {
    ScriptedImageAssetOwners::install(lua);
    if lua.app_data_ref::<ScriptedImageAssets>().is_none() {
        lua.set_app_data(ScriptedImageAssets::default());
    }
    lua.globals().set(
        "ImageSampler",
        lua.create_function(|lua, (wrap_x, wrap_y, filter): (String, String, String)| {
            let parse_wrap = |value: &str| match value {
                "clamp" => Ok(ImageWrap::Clamp),
                "repeat" => Ok(ImageWrap::Repeat),
                "mirror" => Ok(ImageWrap::Mirror),
                other => Err(Error::runtime(format!(
                    "'{other}' is not a valid ImageWrap"
                ))),
            };
            let filter = match filter.as_str() {
                "bilinear" => ImageFilter::Bilinear,
                "nearest" => ImageFilter::Nearest,
                other => {
                    return Err(Error::runtime(format!(
                        "'{other}' is not a valid ImageFilter"
                    )));
                }
            };
            lua.create_userdata(ScriptedImageSampler(ImageSampler {
                wrap_x: parse_wrap(&wrap_x)?,
                wrap_y: parse_wrap(&wrap_y)?,
                filter,
            }))
        })?,
    )?;
    Ok(())
}

pub(super) fn set_image_asset_owners(lua: &Lua, owners: Arc<RuntimeImageAssetOwners>) {
    ScriptedImageAssetOwners::set(lua, owners);
}

pub(super) fn set_script_image_assets(lua: &Lua, assets: ScriptImageAssets) {
    lua.set_app_data(ScriptedImageAssets(assets));
}

pub(super) fn script_image_asset_named(lua: &Lua, name: &str) -> Option<ScriptImage> {
    lua.app_data_ref::<ScriptedImageAssets>()
        .and_then(|assets| assets.0.named(name))
}

#[derive(Clone, Default)]
struct ImageLookupReports(Rc<RefCell<BTreeSet<(String, bool)>>>);

/// Report a nil `context:image(name)` once per name through the VM log, so a
/// host can show why a drawable drew nothing. An undecoded image is logged at
/// info level because it usually resolves within a frame or two; a name with
/// no asset is a warning that lists the names the file does carry. The
/// undecoded wording matches the Rive editor's own message.
pub(super) fn report_image_lookup_miss(lua: &Lua, name: &str, undecoded: bool) {
    let reports = match lua.app_data_ref::<ImageLookupReports>() {
        Some(reports) => reports.clone(),
        None => {
            let reports = ImageLookupReports::default();
            lua.set_app_data(reports.clone());
            reports
        }
    };
    if !reports.0.borrow_mut().insert((name.to_owned(), undecoded)) {
        return;
    }
    let Some(logging) = lua
        .app_data_ref::<LoggingScriptingContext>()
        .map(|logging| logging.clone())
    else {
        return;
    };
    if undecoded {
        logging.begin_line();
        logging.append(
            format!(
                "context:image(\"{name}\") has no decoded image (GPU-compressed format or still loading)"
            )
            .as_bytes(),
        );
        logging.end_line();
        return;
    }
    let names = lua
        .app_data_ref::<ScriptedImageAssets>()
        .map(|assets| {
            assets
                .0
                .names()
                .map(|name| format!("\"{name}\""))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let message = if names.is_empty() {
        format!("context:image(\"{name}\") found no image asset; this file has no image assets")
    } else {
        format!(
            "context:image(\"{name}\") found no image asset with that name; available: {}",
            names.join(", ")
        )
    };
    logging.print_warning(message.as_bytes());
}

pub(super) fn create_asset_image(lua: &Lua, image: ScriptImage) -> Result<Option<AnyUserData>> {
    ScriptedImage::from_asset(lua, image)
        .map(|image| lua.create_userdata(image))
        .transpose()
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use nuxie_render_api::{
        ColorInt, Factory, FillRule, GpuCanvasError, Mat2D, PersistentFactory, RawPath,
        RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderPaint,
        RenderPath, RenderShader,
    };

    use crate::vm::{RoutedTestFactory, ScriptVm};

    #[derive(Clone, Default)]
    struct TestImage(Rc<()>);

    impl RenderImage for TestImage {
        fn retain_image(&self) -> Rc<dyn RenderImage> {
            Rc::new(self.clone())
        }
        fn image_identity(&self) -> usize {
            Rc::as_ptr(&self.0) as usize
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn width(&self) -> u32 {
            7
        }

        fn height(&self) -> u32 {
            11
        }

        fn uv_transform(&self) -> Mat2D {
            Mat2D::IDENTITY
        }
    }

    struct ImageViewFactory(RecordingFactory);

    impl Factory for ImageViewFactory {
        fn make_render_buffer(
            &mut self,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size_in_bytes: usize,
        ) -> Box<dyn RenderBuffer> {
            self.0.make_render_buffer(buffer_type, flags, size_in_bytes)
        }

        fn make_linear_gradient(
            &mut self,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
        }

        fn make_radial_gradient(
            &mut self,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.0.make_radial_gradient(cx, cy, radius, colors, stops)
        }

        fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
            self.0.make_render_path(path, fill_rule)
        }

        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.0.make_empty_render_path()
        }

        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            self.0.make_render_paint()
        }

        fn decode_image(
            &mut self,
            data: &[u8],
        ) -> std::result::Result<Box<dyn RenderImage>, nuxie_render_api::ImageDecodeError> {
            self.0.decode_image(data)
        }

        fn make_gpu_canvas_image_view(
            &mut self,
            image: Rc<dyn RenderImage>,
        ) -> std::result::Result<Rc<dyn RenderImage>, GpuCanvasError> {
            Ok(image)
        }
    }

    #[test]
    fn video_snapshot_uses_the_existing_gpu_image_view() {
        use nuxie_runtime::video::Video;
        let vm = ScriptVm::new();
        let mut recorder =
            nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::fromReal(None);
        recorder.setCanvasRegistry(Some(Rc::new(RefCell::new(
            nuxie_renderer::deferred::cmd::foreign_image_registry::ForeignImageRegistry::default(),
        ))));
        let mut factory = PersistentFactory::new(RoutedTestFactory {
            inner: ImageViewFactory(RecordingFactory::new()),
            ore: Some(Rc::new(RefCell::new(recorder))),
            canvas_host: None,
        });
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let arena = nuxie_runtime::source::core::CoreArena::default();
        let video = arena.insert(Video::default());
        let frame: Rc<dyn RenderImage> = Rc::new(TestImage::default());
        video
            .with_downcast_mut::<Video, _>(|video| {
                video.playback.opened(0, 1.0);
                assert!(video.present(0, frame.clone(), 0.0));
            })
            .unwrap();
        let lua = vm.lua();
        lua.globals()
            .set(
                "video",
                lua.create_userdata(super::super::lua_video::ScriptVideo::new(
                    video,
                    Rc::new(std::cell::Cell::new(true)),
                    Rc::new(std::cell::Cell::new(false)),
                ))
                .unwrap(),
            )
            .unwrap();
        let format: String = lua
            .load("image = video:image(); return image:view().format")
            .eval()
            .unwrap();
        assert_eq!(format, "rgba8unorm");
        let image = lua.globals().get::<AnyUserData>("image").unwrap();
        let image = image.borrow::<ScriptedImage>().unwrap();
        assert!(Rc::ptr_eq(&frame, &image.render_image().unwrap()));
        let cached = image.cached_gpu_view.borrow().as_ref().unwrap().clone();
        let view = cached.textureViewBase().unwrap();
        assert_eq!(view.texture().width(), Some(7));
        assert_eq!(view.texture().height(), Some(11));
    }

    #[test]
    fn image_members_include_a_cached_renderer_backed_gpu_view() {
        let vm = ScriptVm::new();
        let mut recorder =
            nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::fromReal(None);
        recorder.setCanvasRegistry(Some(Rc::new(RefCell::new(
            nuxie_renderer::deferred::cmd::foreign_image_registry::ForeignImageRegistry::default(),
        ))));
        let mut factory = PersistentFactory::new(RoutedTestFactory {
            inner: ImageViewFactory(RecordingFactory::new()),
            ore: Some(Rc::new(RefCell::new(recorder))),
            canvas_host: None,
        });
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let lua = vm.lua();
        let image = lua
            .create_userdata(ScriptedImage::from_render_image(Box::new(
                TestImage::default(),
            )))
            .unwrap();
        lua.globals().set("image", image).unwrap();

        assert_eq!(lua.load("return image.width").eval::<u32>().unwrap(), 7);
        assert_eq!(lua.load("return image.height").eval::<u32>().unwrap(), 11);
        assert!(lua.load("image.width = 3").exec().is_err());
        assert!(lua.load("image.height = 5").exec().is_err());
        let format: String = lua
            .load(
                "local first = image:view()\n\
                 local second = image:view()\n\
                 return first.format",
            )
            .eval()
            .unwrap();
        assert_eq!(format, "rgba8unorm");
        assert_eq!(
            lua.load("return type(image.view)")
                .eval::<String>()
                .unwrap(),
            "function",
            "view is a method, as upstream: scripts call image:view()"
        );

        let image = lua.globals().get::<AnyUserData>("image").unwrap();
        let image = image.borrow::<ScriptedImage>().unwrap();
        let cached = image.cached_gpu_view.borrow().as_ref().unwrap().clone();
        let view = cached.textureViewBase().unwrap();
        assert_eq!(view.texture().width(), Some(7));
        assert_eq!(view.texture().height(), Some(11));

        let lua = Lua::new();
        lua.globals()
            .set(
                "image",
                lua.create_userdata(ScriptedImage::from_render_image(Box::new(
                    TestImage::default(),
                )))
                .unwrap(),
            )
            .unwrap();
        let error = lua
            .load("return image:view()")
            .eval::<AnyUserData>()
            .expect_err("Image:view requires the active renderer context");
        assert!(
            error
                .to_string()
                .contains("GPU context not available for Image:view()")
        );
    }
}
