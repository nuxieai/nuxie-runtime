// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer_library.cpp
use luaur_rt::{Lua, Result};
use nuxie_render_api::Factory as RenderFactory;

use super::CanvasDrawingPhase;
use super::command_server::PersistentRenderContext;
use super::view_model::ScriptViewModelFrameContext;

#[derive(Clone, Default)]
pub(crate) struct RendererBindings {
    render_context: PersistentRenderContext,
    canvas_drawing_phase: CanvasDrawingPhase,
    pub(super) view_model_frame_context: ScriptViewModelFrameContext,
}

impl RendererBindings {
    pub(crate) fn new(view_model_frame_context: ScriptViewModelFrameContext) -> Self {
        Self {
            render_context: PersistentRenderContext::default(),
            canvas_drawing_phase: CanvasDrawingPhase::default(),
            view_model_frame_context,
        }
    }

    pub(crate) fn bootstrap_render_context(&self, factory: &mut dyn RenderFactory) -> Result<()> {
        self.render_context.install(factory)
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

    pub(crate) fn canvas_drawing_phase(&self) -> &CanvasDrawingPhase {
        &self.canvas_drawing_phase
    }
}
