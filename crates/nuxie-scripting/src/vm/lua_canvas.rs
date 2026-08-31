//! Direct owner for the 2D `ScriptedCanvas` portion of pinned
//! `src/lua/renderer/lua_gpu.cpp`.

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{AnyUserData, Lua, Result, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_render_api::{
    BlendMode, ColorInt, DeferredCanvasHostHandle, Factory, ImageSampler, Mat2D,
    PersistentFactoryContext, RenderBuffer, RenderCanvas, RenderCanvasError, RenderCanvasFrame,
    RenderCanvasHandle, RenderImage, RenderPaint, RenderPath, Renderer,
};

use super::lua_image::ScriptedImage;
use super::lua_renderer::ScriptedRenderer;
use super::lua_renderer_library::RendererBindings;

pub(super) struct ScriptedCanvas {
    bindings: RendererBindings,
    canvas: Option<RenderCanvasHandle>,
    render_context: Option<PersistentFactoryContext>,
    pending_width: u32,
    pending_height: u32,
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
        let render_context = bindings.render_context();
        let mut canvas = Self {
            bindings,
            canvas: None,
            render_context,
            pending_width: 0,
            pending_height: 0,
            image: None,
            frame: None,
            renderer: None,
        };
        if width != 0 && height != 0 {
            if canvas.render_context.is_none() {
                if canvas.bindings.deferred_canvas_host().is_none() {
                    return Err(luaur_rt::Error::runtime(
                        "context:canvas() requires a RenderContext — call setRenderContext() first",
                    ));
                }
                canvas.pending_width = width;
                canvas.pending_height = height;
            } else {
                canvas.replace_backing(lua, width, height, "context:canvas()")?;
            }
        }
        lua.create_userdata(canvas)
    }

    fn replace_backing(&mut self, lua: &Lua, width: u32, height: u32, caller: &str) -> Result<()> {
        let context = self
            .render_context
            .as_mut()
            .expect("present allocation device");
        let canvas = allocate_script_render_canvas(&self.bindings, context, width, height)
            .map_err(|_| {
                luaur_rt::Error::runtime(format!("{caller} failed to create RenderCanvas"))
            })?;
        let image =
            lua.create_userdata(ScriptedImage::from_render_image_rc(canvas.render_image()))?;
        self.image = Some(image);
        self.canvas = Some(Rc::new(RefCell::new(canvas)));
        self.pending_width = 0;
        self.pending_height = 0;
        Ok(())
    }

    fn satisfy_pending(&mut self, lua: &Lua) -> Result<()> {
        if self.pending_width == 0 || self.pending_height == 0 {
            return Ok(());
        }
        let Some(context) = self.bindings.render_context() else {
            return Ok(());
        };
        self.render_context = Some(context);
        self.replace_backing(
            lua,
            self.pending_width,
            self.pending_height,
            "Canvas:resize()",
        )
    }

    fn end_frame(&mut self, userdata: &AnyUserData) -> Result<()> {
        self.bindings.unregister_open_canvas_frame(userdata);
        self.end_renderer();
        let frame = self.frame.take().expect("active Canvas frame");
        Rc::try_unwrap(frame)
            .map_err(|_| {
                luaur_rt::Error::runtime("Canvas frame retained after renderer invalidation")
            })?
            .into_inner()
            .finish()
            .map_err(|error| luaur_rt::Error::runtime(error.to_string()))
    }

    fn end_renderer(&mut self) {
        if let Some(renderer) = self.renderer.take() {
            if let Ok(renderer) = renderer.borrow::<ScriptedRenderer>() {
                renderer.end();
            }
        }
    }
}

/// `allocScriptRenderCanvas`: only GL overrides deferred backing allocation.
pub(crate) fn allocate_script_render_canvas(
    bindings: &RendererBindings,
    context: &mut PersistentFactoryContext,
    width: u32,
    height: u32,
) -> std::result::Result<Box<dyn RenderCanvas>, RenderCanvasError> {
    if bindings.deferred_canvas_host().is_some() || bindings.render_context_is_late_bound() {
        context.make_deferred_render_canvas(width, height)
    } else {
        context.make_render_canvas(width, height)
    }
}

/// Called after every protected call, including a script error.
pub(crate) fn close_orphan_canvas_frames(bindings: &RendererBindings) -> Result<bool> {
    let frames = bindings.take_open_canvas_frames();
    let had_orphans = !frames.is_empty();
    for userdata in frames {
        let mut canvas = userdata.borrow_mut::<ScriptedCanvas>()?;
        if canvas.frame.is_some() {
            canvas.end_frame(&userdata)?;
        }
    }
    Ok(had_orphans)
}

struct DeferredCanvasFrame {
    host: DeferredCanvasHostHandle,
    canvas: RenderCanvasHandle,
    renderer: Option<Box<dyn Renderer>>,
}

impl RenderCanvasFrame for DeferredCanvasFrame {
    fn renderer(&mut self) -> &mut dyn Renderer {
        self
    }
    fn finish(self: Box<Self>) -> std::result::Result<(), RenderCanvasError> {
        self.host.borrow_mut().end_canvas_content(&self.canvas);
        Ok(())
    }
}

impl DeferredCanvasFrame {
    fn target(&mut self) -> &mut dyn Renderer {
        self.renderer
            .as_deref_mut()
            .expect("deferred host returned a null renderer")
    }
}

impl Renderer for DeferredCanvasFrame {
    fn save(&mut self) {
        self.target().save();
    }
    fn restore(&mut self) {
        self.target().restore();
    }
    fn transform(&mut self, transform: Mat2D) {
        self.target().transform(transform);
    }
    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.target().draw_path(path, paint);
    }
    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.target().clip_path(path);
    }
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend: BlendMode,
        opacity: f32,
    ) {
        self.target().draw_image(image, sampler, blend, opacity);
    }
    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend: BlendMode,
        opacity: f32,
    ) {
        self.target().draw_image_mesh(
            image,
            sampler,
            vertices,
            uv,
            indices,
            vertex_count,
            index_count,
            blend,
            opacity,
        );
    }
    fn modulate_opacity(&mut self, opacity: f32) {
        self.target().modulate_opacity(opacity);
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
        fields.add_field_function_get("image", |lua, userdata| {
            let mut this = userdata.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this
                .image
                .clone()
                .map(Value::UserData)
                .unwrap_or(Value::Nil))
        });
        fields.add_field_function_get("width", |lua, userdata| {
            let mut this = userdata.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this
                .canvas
                .as_ref()
                .map_or(this.pending_width, |canvas| canvas.borrow().width()))
        });
        fields.add_field_function_get("height", |lua, userdata| {
            let mut this = userdata.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this
                .canvas
                .as_ref()
                .map_or(this.pending_height, |canvas| canvas.borrow().height()))
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
                this.pending_width = 0;
                this.pending_height = 0;
                return Ok(());
            }
            if this.canvas.as_ref().is_some_and(|canvas| {
                let canvas = canvas.borrow();
                canvas.width() == width && canvas.height() == height
            }) {
                return Ok(());
            }
            this.pending_width = width;
            this.pending_height = height;
            this.satisfy_pending(lua)
        });
        methods.add_function(
            "beginFrame",
            |lua, (userdata, descriptor): (AnyUserData, Option<Value>)| {
                let mut this = userdata.borrow_mut::<Self>()?;
                this.satisfy_pending(lua)?;
                if this.render_context.is_none() {
                    return Err(luaur_rt::Error::runtime("Canvas: renderCtx not initialized"));
                }
                if !this.bindings.ore_context().is_some_and(|context| context.borrow().isRecording()) {
                    return Err(luaur_rt::Error::runtime(
                        "Canvas:beginFrame() requires the deferred recorder",
                    ));
                }
                if this.frame.is_some() {
                    return Err(luaur_rt::Error::runtime(
                        "Canvas:beginFrame() called during an active frame",
                    ));
                }
                let canvas = this.canvas.clone().ok_or_else(|| {
                    luaur_rt::Error::runtime(
                        "Canvas:beginFrame() called on a zero-sized canvas; call canvas:resize(w, h) first",
                    )
                })?;
                let clear_color = match descriptor {
                    Some(Value::Table(descriptor)) => {
                        lua.coerce_number(descriptor.get::<Value>("clearColor")?)?
                            .unwrap_or(0.0) as u32
                    }
                    _ => 0,
                };
                let frame: Box<dyn RenderCanvasFrame> = if let Some(host) = this.bindings.deferred_canvas_host() {
                    let renderer = host.borrow_mut().begin_canvas_content(Rc::clone(&canvas), clear_color);
                    Box::new(DeferredCanvasFrame { host, canvas, renderer })
                } else {
                    canvas.borrow_mut().begin_frame(clear_color)
                        .map_err(|error| luaur_rt::Error::runtime(error.to_string()))?
                };
                let frame = Rc::new(RefCell::new(frame));
                this.frame = Some(Rc::clone(&frame));
                this.bindings.register_open_canvas_frame(userdata.clone());
                let renderer = ScriptedRenderer::create_canvas_userdata(
                    lua,
                    Rc::clone(&frame),
                    this.bindings.clone(),
                )?;
                this.renderer = Some(renderer.clone());
                Ok(renderer)
            },
        );
        methods.add_function("endFrame", |_, userdata: AnyUserData| {
            let mut this = userdata.borrow_mut::<Self>()?;
            if this.frame.is_none() {
                return Err(luaur_rt::Error::runtime(
                    "Canvas:endFrame() called without beginFrame()",
                ));
            }
            this.end_frame(&userdata)
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
        identity: Rc<()>,
        width: u32,
        height: u32,
    }

    impl RenderImage for TestCanvasImage {
        fn retain_image(&self) -> Rc<dyn RenderImage> {
            Rc::new(self.clone())
        }
        fn image_identity(&self) -> usize {
            Rc::as_ptr(&self.identity) as usize
        }
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
        fn is_render_context(&self) -> bool {
            true
        }
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
                image: Rc::new(TestCanvasImage {
                    identity: Rc::new(()),
                    width,
                    height,
                }),
                events: Rc::clone(&self.events),
            }))
        }
    }

    #[test]
    fn path_effect_update_closes_orphan_gpu_pass_and_canvas_on_success_and_error() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(TestCanvasFactory::new(events.clone()));
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        vm.set_ore_context(Some(Rc::new(RefCell::new(
            nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::new(None),
        ))));
        let canvas = ScriptedCanvas::create(vm.lua(), vm.renderer_bindings.clone(), 4, 3).unwrap();
        vm.lua().globals().set("canvas", canvas).unwrap();
        let (_gpu_instance, gpu_context) = crate::gpu_canvas::ImportedGpuCanvasInstance::new(
            Default::default(),
            vm.renderer_bindings.clone(),
        );
        let gpu = gpu_context.canvas_userdata(vm.lua()).unwrap();
        vm.lua().globals().set("gpu", gpu).unwrap();
        let effect: luaur_rt::Table = vm
            .lua()
            .load(
                r#"
            local texture = GPUTexture.new {width = 4, height = 3}
            return { update = function(self, path, node)
                retainedRenderer = canvas:beginFrame {clearColor = false}
                retainedRenderer:save()
                retainedPass = gpu:beginRenderPass {
                    color = {{view = texture:view(), storeOp = 'store'}},
                }
                if failUpdate then error('injected update failure') end
                return path
            end }
        "#,
            )
            .eval()
            .unwrap();
        for fail in [false, true] {
            vm.lua().globals().set("failUpdate", fail).unwrap();
            let result = super::super::lua_path::call_path_effect_update(
                &effect,
                RawPath::new(),
                nuxie_runtime::ScriptNode::snapshot(None, None),
            );
            if fail {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("injected update failure")
                );
            } else {
                assert!(result.is_ok());
            }
            vm.lua().load(r#"
                local rendererOk, rendererError = pcall(function() retainedRenderer:save() end)
                assert(not rendererOk and string.find(tostring(rendererError), 'Renderer is no longer valid', 1, true))
                local passOk, passError = pcall(function() retainedPass:finish() end)
                assert(not passOk and string.find(tostring(passError), 'render pass expired', 1, true))
            "#).exec().unwrap();
            assert_eq!(events.borrow().last().map(String::as_str), Some("finish"));
        }
        assert_eq!(
            events
                .borrow()
                .iter()
                .filter(|event| *event == "finish")
                .count(),
            2
        );
        assert_eq!(
            events
                .borrow()
                .iter()
                .filter(|event| *event == "begin:0x00000000")
                .count(),
            2
        );
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
            .expect_err("recorder gate");
        assert!(
            outside
                .to_string()
                .contains("requires the deferred recorder")
        );

        vm.set_ore_context(Some(Rc::new(RefCell::new(
            nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext::new(None),
        ))));

        {
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
        // A failed allocation leaves the prior backing alive, but the pending
        // size is retried on field access. Inspect the retained owner itself.
        let canvas: AnyUserData = vm.lua().globals().get("canvas").unwrap();
        let preserved = canvas.borrow::<ScriptedCanvas>().unwrap();
        assert_eq!(preserved.canvas.as_ref().unwrap().borrow().width(), 4);
        let first_image: AnyUserData = vm.lua().globals().get("firstImage").unwrap();
        assert_eq!(preserved.image.as_ref(), Some(&first_image));
        drop(preserved);

        vm.lua().load("canvas:resize(0, 8)").exec().unwrap();
        let deferred: bool = vm
            .lua()
            .load("return canvas.width == 0 and canvas.height == 0 and canvas.image == nil")
            .eval()
            .unwrap();
        assert!(deferred);

        vm.lua().load("canvas:resize(4, 3)").exec().unwrap();
        {
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
