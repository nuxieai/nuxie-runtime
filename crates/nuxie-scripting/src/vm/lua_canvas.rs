//! Direct owner for the 2D `ScriptedCanvas` portion of pinned
//! `src/lua/renderer/lua_gpu.cpp`.

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{AnyUserData, Lua, Result, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_render_api::{RenderCanvas, RenderCanvasFrame};

use super::lua_image::ScriptedImage;
use super::lua_renderer::ScriptedRenderer;
use super::lua_renderer_library::RendererBindings;

pub(super) struct ScriptedCanvas {
    bindings: RendererBindings,
    canvas: Option<Box<dyn RenderCanvas>>,
    image: Option<AnyUserData>,
    frame: Option<Rc<RefCell<Box<dyn RenderCanvasFrame>>>>,
    renderer: Option<AnyUserData>,
}

impl ScriptedCanvas {
    pub(super) fn create(
        lua: &Lua,
        bindings: RendererBindings,
        width: u32,
        height: u32,
    ) -> Result<AnyUserData> {
        bindings.with_factory(|_| Ok(())).map_err(|_| {
            luaur_rt::Error::runtime(
                "context:canvas() requires a RenderContext — call setRenderContext() first",
            )
        })?;
        let mut canvas = Self {
            bindings,
            canvas: None,
            image: None,
            frame: None,
            renderer: None,
        };
        if width != 0 && height != 0 {
            canvas.replace_backing(lua, width, height, "context:canvas()")?;
        }
        lua.create_userdata(canvas)
    }

    fn replace_backing(&mut self, lua: &Lua, width: u32, height: u32, caller: &str) -> Result<()> {
        // Pinned Canvas::resize allocates the replacement before releasing its
        // old image/canvas pair, so an allocation error preserves that pair.
        let canvas = self.bindings.with_factory(|factory| {
            factory.make_render_canvas(width, height).map_err(|_| {
                luaur_rt::Error::runtime(format!("{caller} failed to create RenderCanvas"))
            })
        })?;
        let image =
            lua.create_userdata(ScriptedImage::from_render_image_rc(canvas.render_image()))?;
        self.image = Some(image);
        self.canvas = Some(canvas);
        Ok(())
    }

    fn end_renderer(&mut self) {
        if let Some(renderer) = self.renderer.take() {
            if let Ok(renderer) = renderer.borrow::<ScriptedRenderer>() {
                renderer.end();
            }
        }
    }
}

impl Drop for ScriptedCanvas {
    fn drop(&mut self) {
        // Pinned destructor invalidates/deletes an un-ended renderer but does
        // not submit the frame.
        self.end_renderer();
        self.frame.take();
    }
}

impl UserData for ScriptedCanvas {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("image", |_, this| {
            Ok(this
                .image
                .clone()
                .map(Value::UserData)
                .unwrap_or(Value::Nil))
        });
        fields.add_field_method_get("width", |_, this| {
            Ok(this.canvas.as_ref().map_or(0, |canvas| canvas.width()))
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this.canvas.as_ref().map_or(0, |canvas| canvas.height()))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |lua, this, (width, height): (u32, u32)| {
            if this.frame.is_some() {
                return Err(luaur_rt::Error::runtime(
                    "Canvas:resize() called during an active frame",
                ));
            }
            if width == 0 || height == 0 {
                this.image = None;
                this.canvas = None;
                return Ok(());
            }
            this.replace_backing(lua, width, height, "Canvas:resize()")
        });
        methods.add_method_mut(
            "beginFrame",
            |lua, this, descriptor: Option<Value>| {
                if !this.bindings.canvas_drawing_phase().is_active() {
                    return Err(luaur_rt::Error::runtime(
                        "Canvas:beginFrame() called outside drawing phase",
                    ));
                }
                if this.frame.is_some() {
                    return Err(luaur_rt::Error::runtime(
                        "Canvas:beginFrame() called during an active frame",
                    ));
                }
                let canvas = this.canvas.as_mut().ok_or_else(|| {
                    luaur_rt::Error::runtime(
                        "Canvas:beginFrame() called on a zero-sized canvas; call canvas:resize(w, h) first",
                    )
                })?;
                let clear_color = match descriptor {
                    Some(Value::Table(descriptor)) => {
                        descriptor.get::<Option<u32>>("clearColor")?.unwrap_or(0)
                    }
                    _ => 0,
                };
                let frame = canvas
                    .begin_frame(clear_color)
                    .map_err(|error| luaur_rt::Error::runtime(error.to_string()))?;
                let frame = Rc::new(RefCell::new(frame));
                let renderer = ScriptedRenderer::create_canvas_userdata(
                    lua,
                    Rc::clone(&frame),
                    this.bindings.clone(),
                )?;
                this.renderer = Some(renderer.clone());
                this.frame = Some(frame);
                Ok(renderer)
            },
        );
        methods.add_method_mut("endFrame", |_, this, ()| {
            if this.frame.is_none() {
                return Err(luaur_rt::Error::runtime(
                    "Canvas:endFrame() called without beginFrame()",
                ));
            }
            this.end_renderer();
            let frame = this.frame.take().expect("checked active Canvas frame");
            Rc::try_unwrap(frame)
                .map_err(|_| {
                    luaur_rt::Error::runtime(
                        "Canvas frame still has a live renderer owner after endFrame",
                    )
                })?
                .into_inner()
                .finish()
                .map_err(|error| luaur_rt::Error::runtime(error.to_string()))
        });
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use std::any::Any;
    use std::cell::RefCell;
    use std::rc::Rc;

    use nuxie_render_api::{
        BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D,
        PersistentFactory, RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags,
        RenderBufferType, RenderCanvasError, RenderImage, RenderPaint, RenderPath, RenderShader,
        Renderer,
    };

    use crate::vm::ScriptVm;

    #[derive(Clone)]
    struct TestCanvasImage {
        width: u32,
        height: u32,
    }

    impl RenderImage for TestCanvasImage {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }

    struct TestCanvas {
        width: u32,
        height: u32,
        image: Rc<TestCanvasImage>,
        events: Rc<RefCell<Vec<String>>>,
    }

    impl RenderCanvas for TestCanvas {
        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }

        fn render_image(&self) -> Rc<dyn RenderImage> {
            self.image.clone()
        }

        fn begin_frame(
            &mut self,
            clear_color: ColorInt,
        ) -> std::result::Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
            self.events
                .borrow_mut()
                .push(format!("begin:{clear_color:#010x}"));
            Ok(Box::new(TestCanvasFrame {
                events: Rc::clone(&self.events),
            }))
        }
    }

    struct TestCanvasFrame {
        events: Rc<RefCell<Vec<String>>>,
    }

    impl RenderCanvasFrame for TestCanvasFrame {
        fn renderer(&mut self) -> &mut dyn Renderer {
            self
        }

        fn finish(self: Box<Self>) -> std::result::Result<(), RenderCanvasError> {
            self.events.borrow_mut().push("finish".into());
            Ok(())
        }
    }

    impl Renderer for TestCanvasFrame {
        fn save(&mut self) {
            self.events.borrow_mut().push("save".into());
        }

        fn restore(&mut self) {
            self.events.borrow_mut().push("restore".into());
        }

        fn transform(&mut self, _: Mat2D) {}
        fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
        fn clip_path(&mut self, _: &dyn RenderPath) {}
        fn draw_image(
            &mut self,
            _: Option<&dyn RenderImage>,
            _: ImageSampler,
            _: BlendMode,
            _: f32,
        ) {
        }
        fn draw_image_mesh(
            &mut self,
            _: Option<&dyn RenderImage>,
            _: ImageSampler,
            _: Option<&dyn RenderBuffer>,
            _: Option<&dyn RenderBuffer>,
            _: Option<&dyn RenderBuffer>,
            _: u32,
            _: u32,
            _: BlendMode,
            _: f32,
        ) {
        }
        fn modulate_opacity(&mut self, _: f32) {}
    }

    struct TestCanvasFactory {
        inner: RecordingFactory,
        events: Rc<RefCell<Vec<String>>>,
    }

    impl TestCanvasFactory {
        fn new(events: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                inner: RecordingFactory::new(),
                events,
            }
        }
    }

    impl Factory for TestCanvasFactory {
        fn make_render_buffer(
            &mut self,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size_in_bytes: usize,
        ) -> Box<dyn RenderBuffer> {
            self.inner
                .make_render_buffer(buffer_type, flags, size_in_bytes)
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
            self.inner
                .make_linear_gradient(sx, sy, ex, ey, colors, stops)
        }

        fn make_radial_gradient(
            &mut self,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.inner
                .make_radial_gradient(cx, cy, radius, colors, stops)
        }

        fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
            self.inner.make_render_path(path, fill_rule)
        }

        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.inner.make_empty_render_path()
        }

        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            self.inner.make_render_paint()
        }

        fn decode_image(
            &mut self,
            data: &[u8],
        ) -> std::result::Result<Box<dyn RenderImage>, ImageDecodeError> {
            self.inner.decode_image(data)
        }

        fn make_render_canvas(
            &mut self,
            width: u32,
            height: u32,
        ) -> std::result::Result<Box<dyn RenderCanvas>, RenderCanvasError> {
            if width == 99 {
                return Err(RenderCanvasError::new("injected allocation failure"));
            }
            self.events
                .borrow_mut()
                .push(format!("allocate:{width}x{height}"));
            Ok(Box::new(TestCanvas {
                width,
                height,
                image: Rc::new(TestCanvasImage { width, height }),
                events: Rc::clone(&self.events),
            }))
        }
    }

    #[test]
    fn canvas_ports_deferred_resize_frame_and_renderer_lifecycle() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(TestCanvasFactory::new(Rc::clone(&events)));
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let canvas = ScriptedCanvas::create(vm.lua(), vm.renderer_bindings.clone(), 4, 3).unwrap();
        vm.lua().globals().set("canvas", canvas).unwrap();

        let outside = vm
            .lua()
            .load("canvas:beginFrame()")
            .exec()
            .expect_err("drawing phase gate");
        assert!(outside.to_string().contains("outside drawing phase"));

        {
            let _drawing = vm.canvas_drawing_phase().scoped();
            vm.lua()
                .load(
                    "firstImage = canvas.image\n\
                     retained = canvas:beginFrame({ clearColor = 0xff102030 })\n\
                     retained:save()\n\
                     canvas:endFrame()",
                )
                .exec()
                .unwrap();
        }

        let (valid, error): (bool, String) = vm
            .lua()
            .load(
                "local ok, err = pcall(function() retained:save() end)\n\
                 return ok, tostring(err)",
            )
            .eval()
            .unwrap();
        assert!(!valid);
        assert!(error.contains("Renderer is no longer valid"), "{error}");
        assert_eq!(
            events.borrow().as_slice(),
            [
                "allocate:4x3",
                "begin:0xff102030",
                "save",
                "restore",
                "finish"
            ]
        );

        let failed = vm
            .lua()
            .load("canvas:resize(99, 7)")
            .exec()
            .expect_err("injected replacement failure");
        assert!(failed.to_string().contains("failed to create RenderCanvas"));
        let preserved: bool = vm
            .lua()
            .load("return canvas.width == 4 and canvas.image == firstImage")
            .eval()
            .unwrap();
        assert!(preserved);

        vm.lua().load("canvas:resize(0, 8)").exec().unwrap();
        let deferred: bool = vm
            .lua()
            .load("return canvas.width == 0 and canvas.height == 0 and canvas.image == nil")
            .eval()
            .unwrap();
        assert!(deferred);

        vm.lua().load("canvas:resize(4, 3)").exec().unwrap();
        {
            let _drawing = vm.canvas_drawing_phase().scoped();
            vm.lua()
                .load("canvas:beginFrame('non-table descriptors are ignored'); canvas:endFrame()")
                .exec()
                .unwrap();
        }
        assert_eq!(events.borrow().last().map(String::as_str), Some("finish"));

        let creation = ScriptedCanvas::create(vm.lua(), vm.renderer_bindings.clone(), 99, 7)
            .expect_err("injected initial allocation failure");
        assert!(
            creation
                .to_string()
                .contains("context:canvas() failed to create RenderCanvas")
        );
    }
}
