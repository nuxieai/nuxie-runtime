// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer_library.cpp
use luaur_rt::{AnyUserData, Lua, Result};
use nuxie_render_api::Factory as RenderFactory;
use nuxie_render_api::{DeferredCanvasHostHandle, OreContextHandle, PersistentFactoryContext};
use std::{cell::RefCell, rc::Rc};

use super::command_server::PersistentRenderContext;
use super::view_model::ScriptViewModelFrameContext;

#[derive(Clone, Default)]
pub(crate) struct RendererBindings {
    render_context: PersistentRenderContext,
    routing: Rc<RefCell<ScriptingRouting>>,
    open_canvas_frames: Rc<RefCell<Vec<AnyUserData>>>,
    pub(super) view_model_frame_context: ScriptViewModelFrameContext,
}

#[derive(Default)]
struct ScriptingRouting {
    render_context: Option<PersistentFactoryContext>,
}

impl RendererBindings {
    pub(crate) fn new(view_model_frame_context: ScriptViewModelFrameContext) -> Self {
        Self {
            render_context: PersistentRenderContext::default(),
            routing: Rc::default(),
            open_canvas_frames: Rc::default(),
            view_model_frame_context,
        }
    }

    pub(crate) fn bootstrap_render_context(&self, factory: &mut dyn RenderFactory) -> Result<()> {
        self.render_context.install(factory)
    }

    pub(crate) fn register_open_canvas_frame(&self, canvas: AnyUserData) {
        self.open_canvas_frames.borrow_mut().push(canvas);
    }

    pub(crate) fn unregister_open_canvas_frame(&self, canvas: &AnyUserData) {
        self.open_canvas_frames
            .borrow_mut()
            .retain(|open| open != canvas);
    }

    pub(crate) fn take_open_canvas_frames(&self) -> Vec<AnyUserData> {
        std::mem::take(&mut *self.open_canvas_frames.borrow_mut())
    }

    pub(crate) fn verify_render_context(&self, factory: &mut dyn RenderFactory) -> Result<()> {
        self.render_context.verify(factory)
    }

    pub(crate) fn install(&self, lua: &Lua) -> Result<()> {
        lua.set_app_data(self.clone());
        super::lua_color::install_color_global(lua)?;
        super::lua_mat2d::install_mat2d_global(lua)?;
        super::lua_path::install_path_global(lua)?;
        super::lua_image::install_image_globals(lua)?;
        super::lua_mesh::install_mesh_globals(lua)?;
        self.install_gradient_global(lua)?;
        self.install_paint_global(lua)?;
        Ok(())
    }

    pub(crate) fn for_lua(lua: &Lua) -> Option<Self> {
        lua.app_data_ref::<Self>().map(|bindings| bindings.clone())
    }

    pub(crate) fn with_factory<R>(
        &self,
        f: impl FnOnce(&mut dyn RenderFactory) -> Result<R>,
    ) -> Result<R> {
        self.render_context.with_factory(f)
    }

    pub(crate) fn set_render_context(&self, context: Option<PersistentFactoryContext>) {
        self.routing.borrow_mut().render_context = context;
    }
    pub(crate) fn render_context_is_late_bound(&self) -> bool {
        self.routing.borrow().render_context.is_none()
    }
    pub(crate) fn render_context(&self) -> Option<PersistentFactoryContext> {
        if let Some(context) = self.routing.borrow().render_context.clone() {
            return Some(context);
        }
        self.with_factory(|factory| Ok(factory.render_context()))
            .ok()
            .flatten()
    }
    pub(crate) fn ore_context(&self) -> Option<OreContextHandle> {
        if let Some(context) = self
            .with_factory(|factory| Ok(factory.ore()))
            .ok()
            .flatten()
        {
            return Some(context);
        }
        // Upstream's final fallback uses only the explicitly supplied device.
        self.routing
            .borrow()
            .render_context
            .clone()
            .and_then(|mut context| context.ore())
    }
    pub(crate) fn deferred_canvas_host(&self) -> Option<DeferredCanvasHostHandle> {
        self.with_factory(|factory| Ok(factory.deferred_canvas_host()))
            .ok()
            .flatten()
    }
    pub(crate) fn route_to_import_factory(&self, factory: &mut dyn RenderFactory) {
        self.render_context.adopt(factory);
        if self.render_context_is_late_bound() {
            if let Some(context) = factory.render_context() {
                self.set_render_context(Some(context));
            }
        }
    }

    pub(crate) fn gpu_features(&self, lua: &Lua) -> Result<luaur_rt::Table> {
        let table = lua.create_table();
        if let Some(context) = self.ore_context() {
            let context = context.borrow();
            if !context.featuresKnown() {
                return Err(luaur_rt::Error::runtime(
                    "context.features is not available yet: this script is recording for a GPU device that has not been attached, so no capability can be reported without guessing at it. Read features from a method that runs after the first frame instead of at module scope",
                ));
            }
            let features = context.features();
            macro_rules! field {
                ($($name:ident),*) => { $(table.set(stringify!($name), features.$name)?;)* };
            }
            field!(
                bc,
                etc2,
                astc,
                maxTextureSize2D,
                maxTextureSizeCube,
                maxTextureSize3D,
                anisotropicFiltering,
                texture3D,
                textureArrays,
                colorBufferFloat,
                colorBufferHalfFloat,
                perTargetBlend,
                perTargetWriteMask,
                drawBaseInstance,
                depthBiasClamp,
                maxColorAttachments,
                maxUniformBufferSize,
                maxSamplers,
                maxSamples
            );
        } else {
            for name in [
                "bc",
                "etc2",
                "astc",
                "anisotropicFiltering",
                "texture3D",
                "textureArrays",
                "colorBufferFloat",
                "colorBufferHalfFloat",
                "perTargetBlend",
                "perTargetWriteMask",
                "drawBaseInstance",
                "depthBiasClamp",
            ] {
                table.set(name, false)?;
            }
            for (name, value) in [
                ("maxTextureSize2D", 4096u32),
                ("maxTextureSizeCube", 4096),
                ("maxTextureSize3D", 256),
                ("maxColorAttachments", 4),
                ("maxUniformBufferSize", 16384),
                ("maxSamplers", 16),
                ("maxSamples", 4),
            ] {
                table.set(name, value)?;
            }
        }
        table.set_readonly(true);
        Ok(table)
    }
}
