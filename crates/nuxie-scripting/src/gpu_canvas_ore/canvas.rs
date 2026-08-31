//! ScriptedGPUCanvas backing and render-pass creation from lua_gpu.cpp.
use super::pass::Pass;
use super::*;
use crate::vm::lua_canvas::allocate_script_render_canvas;
use crate::vm::lua_image::ScriptedImage;
use nuxie_render_api::{PersistentFactoryContext, RenderCanvasHandle};

pub(crate) struct Canvas {
    bindings: RendererBindings,
    canvas: Option<RenderCanvasHandle>,
    render_context: Option<PersistentFactoryContext>,
    color_view: Option<AnyResourceHandle>,
    image: Option<AnyUserData>,
    pending_width: u32,
    pending_height: u32,
}
impl Canvas {
    pub fn create(
        lua: &Lua,
        bindings: RendererBindings,
        width: u32,
        height: u32,
    ) -> Result<AnyUserData> {
        let mut value = Self {
            render_context: bindings.render_context(),
            bindings,
            canvas: None,
            color_view: None,
            image: None,
            pending_width: 0,
            pending_height: 0,
        };
        if width != 0 && height != 0 {
            if value.render_context.is_none() {
                if value.bindings.deferred_canvas_host().is_none() {
                    return Err(Error::runtime(
                        "context:gpuCanvas() requires a RenderContext — call setRenderContext() first",
                    ));
                }
                value.pending_width = width;
                value.pending_height = height;
            } else {
                if value.bindings.ore_context().is_none() {
                    return Err(Error::runtime(
                        "context:gpuCanvas() requires a GPU context — call scriptingWorkspaceSetOreContext() before requestVM()",
                    ));
                }
                value.replace_backing(lua, width, height, "context:gpuCanvas()")?;
            }
        }
        lua.create_userdata(value)
    }
    fn replace_backing(&mut self, lua: &Lua, width: u32, height: u32, caller: &str) -> Result<()> {
        let context = self.render_context.as_mut().expect("allocation context");
        let canvas = allocate_script_render_canvas(&self.bindings, context, width, height)
            .map_err(|_| Error::runtime(format!("{caller} failed to create RenderCanvas")))?;
        let canvas: RenderCanvasHandle = Rc::new(RefCell::new(canvas));
        let info = nuxie_render_api::canvas_texture_info(&canvas);
        let ore = self
            .bindings
            .ore_context()
            .ok_or_else(|| Error::runtime("GPU context not initialized"))?;
        let color_view = unsafe { ore.borrow_mut().wrapCanvasTextureInfo(info) }
            .ok_or_else(|| Error::runtime(format!("{caller} failed to wrap canvas texture")))?;
        let image = lua.create_userdata(ScriptedImage::from_render_image_rc(
            canvas.borrow().render_image(),
        ))?;
        self.canvas = Some(canvas);
        self.color_view = Some(color_view);
        self.image = Some(image);
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
        if self.bindings.ore_context().is_none() {
            return Ok(());
        }
        self.render_context = Some(context);
        self.replace_backing(
            lua,
            self.pending_width,
            self.pending_height,
            "GPUCanvas:resize()",
        )
    }
    fn width(&self) -> u32 {
        self.canvas
            .as_ref()
            .map_or(self.pending_width, |canvas| canvas.borrow().width())
    }
    fn height(&self) -> u32 {
        self.canvas
            .as_ref()
            .map_or(self.pending_height, |canvas| canvas.borrow().height())
    }
}
impl UserData for Canvas {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_function_get("width", |lua, data| {
            let mut this = data.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this.width())
        });
        fields.add_field_function_get("height", |lua, data| {
            let mut this = data.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this.height())
        });
        fields.add_field_function_get("image", |lua, data| {
            let mut this = data.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this.image.clone())
        });
        fields.add_field_function_get("format", |lua, data| {
            let mut this = data.borrow_mut::<Self>()?;
            this.satisfy_pending(lua)?;
            Ok(this
                .color_view
                .as_ref()
                .map(|resource| {
                    format_string(
                        TextureView {
                            resource: resource.clone(),
                            retained_image: None,
                        }
                        .format(),
                    )
                })
                .unwrap_or("rgba8unorm"))
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |lua, this, (width, height): (u32, u32)| {
            if width == 0 || height == 0 {
                this.canvas = None;
                this.color_view = None;
                this.image = None;
                this.pending_width = 0;
                this.pending_height = 0;
                return Ok(());
            }
            if this.canvas.as_ref().is_some_and(|canvas| {
                canvas.borrow().width() == width && canvas.borrow().height() == height
            }) {
                return Ok(());
            }
            this.pending_width = width;
            this.pending_height = height;
            this.satisfy_pending(lua)
        });
        methods.add_method_mut("colorView",|lua,this,()| {
            this.satisfy_pending(lua)?;let resource=this.color_view.clone().ok_or_else(||Error::runtime("GPUCanvas:colorView() has no backing texture. Call resize(w, h) with non-zero dimensions first."))?;
            lua.create_userdata(TextureView {resource, retained_image:None})
        });
        methods.add_method("beginRenderPass", |lua, this, table: Table| {
            begin_pass(lua, this, &table)
        });
    }
}
fn load_op(value: Option<String>) -> LoadOp {
    if value.as_deref() == Some("load") {
        LoadOp::load
    } else {
        LoadOp::clear
    }
}
fn store_op(value: &str) -> StoreOp {
    if value == "discard" {
        StoreOp::discard
    } else {
        StoreOp::store
    }
}
fn begin_pass(lua: &Lua, canvas: &Canvas, table: &Table) -> Result<AnyUserData> {
    let context = canvas
        .bindings
        .ore_context()
        .ok_or_else(|| Error::runtime("GPUCanvas:beginRenderPass() requires a GPU context"))?;
    if !context.borrow().isRecording() {
        return Err(Error::runtime(
            "GPUCanvas:beginRenderPass() requires the deferred recorder",
        ));
    }
    let mut sample_count = None;
    let mut record_sample = |samples: u32| -> Result<()> {
        if sample_count.is_some_and(|current| current != samples) {
            return Err(Error::runtime(
                "beginRenderPass: all render-pass attachments must share one sampleCount",
            ));
        }
        sample_count = Some(samples);
        Ok(())
    };
    let mut colors = Vec::new();
    if let Some(entries) = optional_table(table, "color")? {
        for index in 1..=4 {
            let Value::Table(entry) = entries.raw_get::<Value>(index)? else {
                break;
            };
            let view=match entry.get::<Option<AnyUserData>>("view")? {Some(value)=>value.borrow::<TextureView>()?.clone(),None=>TextureView {resource:canvas.color_view.clone().ok_or_else(||Error::runtime(format!("beginRenderPass: color[{index}].view omitted but the receiving canvas has no backing texture (zero-sized). Call canvas:resize(w, h) before drawing, or pass an explicit view.")))?, retained_image:None}};
            record_sample(view.sample_count())?;
            let resolve = entry
                .get::<Option<AnyUserData>>("resolveTarget")?
                .map(|value| value.borrow::<TextureView>().map(|view| view.clone()))
                .transpose()?;
            if let Some(resolve) = &resolve {
                if view.sample_count() == 1 {
                    return Err(Error::runtime(
                        "beginRenderPass: resolveTarget is meaningless when source view is single-sampled",
                    ));
                }
                if resolve.format() != view.format() {
                    return Err(Error::runtime(
                        "beginRenderPass: resolveTarget format does not match MSAA attachment format",
                    ));
                }
                if resolve.sample_count() != 1 {
                    return Err(Error::runtime(
                        "beginRenderPass: resolveTarget must have sampleCount=1",
                    ));
                }
            }
            let load_operation = load_op(checked_string(&entry, "loadOp")?);
            let store=checked_string(&entry,"storeOp")?.ok_or_else(||Error::runtime(format!("beginRenderPass: color[{index}].storeOp is required — use 'discard' for MSAA color (after resolve) or 'store' to keep the rendered output")))?;
            let mut clear = ClearColor::default();
            if let Some(values) = optional_table(&entry, "clearColor")? {
                let component = |index| -> Result<f32> {
                    Ok(number_value(lua, values.raw_get::<Value>(index)?, 0.0)? as f32)
                };
                clear = ClearColor {
                    r: component(1)?,
                    g: component(2)?,
                    b: component(3)?,
                    a: component(4)?,
                };
            }
            colors.push((view, resolve, load_operation, store_op(&store), clear));
        }
    }
    let mut depth_desc = DepthStencilAttachment::default();
    let mut depth = None;
    if let Some(entry) = optional_table(table, "depthStencil")? {
        let data: AnyUserData = entry.get("view")?;
        let view = data.borrow::<TextureView>()?.clone();
        record_sample(view.sample_count())?;
        depth_desc.depthLoadOp = load_op(checked_string(&entry, "depthLoadOp")?);
        depth_desc.depthStoreOp=store_op(&checked_string(&entry,"depthStoreOp")?.ok_or_else(||Error::runtime("beginRenderPass: depthStencil.depthStoreOp is required — use 'discard' for transient/MSAA depth or 'store' if you need to read it later"))?);
        depth_desc.depthClearValue = match entry.get::<Value>("depthClearValue")? {
            Value::Nil => 1.0,
            value => number_value(lua, value, 0.0)? as f32,
        };
        depth = Some(view);
    }
    if colors.is_empty() && depth.is_none() {
        return Err(Error::runtime(
            "beginRenderPass: descriptor must include at least one color attachment or a depthStencil attachment",
        ));
    }
    depth_desc.view = depth.as_ref().map(|view| &view.resource);
    let mut desc = RenderPassDesc {
        colorCount: colors.len() as u32,
        depthStencil: depth_desc,
        ..RenderPassDesc::default()
    };
    for (index, (view, resolve, load, store, clear)) in colors.iter().enumerate() {
        desc.colorAttachments[index] = ColorAttachment {
            view: Some(&view.resource),
            resolveTarget: resolve.as_ref().map(|view| &view.resource),
            loadOp: *load,
            storeOp: *store,
            clearColor: *clear,
        };
    }
    let active = {
        context
            .borrow()
            .activeRenderPass()
            .and_then(|active| active.upgrade())
    };
    if let Some(active) = active {
        if !active.isFinished() {
            active.finish();
            context.borrow().setActiveRenderPass(None);
        }
    }
    let pass =
        nuxie_ore_metal::ore_cmd::ore_deferred_render_pass::beginRenderPassRecordingOrImmediate(
            context.clone(),
            &desc,
            None,
        );
    context.borrow().setActiveRenderPass(pass.as_deref());
    lua.create_userdata(Pass {
        pass,
        finished: false,
        sample_count: sample_count.unwrap_or(1).max(1),
        pipeline_set: false,
        draw_call_count: 0,
    })
}
