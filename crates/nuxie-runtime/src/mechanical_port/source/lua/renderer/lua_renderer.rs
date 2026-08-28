use crate::mechanical_port::source::{
    lua::rive_lua_libs::*, math::mat2d::Mat2D, renderer::Renderer,
};
impl ScriptedRenderer {
    pub fn end(&mut self) -> bool {
        let renderer = self.renderer.as_mut().unwrap();
        for _ in 0..self.save_count {
            renderer.restore();
        }
        self.renderer = None;
        let success = self.save_count == 0;
        self.save_count = 0;
        success
    }
    pub fn save(&mut self, state: &mut LuaState) {
        self.validate(state).save();
        self.save_count += 1;
    }
    pub fn restore(&mut self, state: &mut LuaState) {
        self.validate(state);
        if self.save_count == 0 {
            state.error::<()>(format!(
                "{} save/restore stack was unbalanced by trying to restore more times than saved.",
                Self::LUA_NAME
            ));
        }
        self.renderer.as_mut().unwrap().restore();
        self.save_count -= 1;
    }
    pub fn transform(&mut self, state: &mut LuaState, matrix: Mat2D) {
        self.validate(state).transform(matrix);
    }
    pub fn clip_path(&mut self, state: &mut LuaState, path: &mut ScriptedPathData) {
        let render = path.render_path(state);
        self.validate(state).clip_path(render);
    }
    pub fn validate(&mut self, state: &mut LuaState) -> &mut Renderer {
        if self.renderer.is_none() {
            state.error::<()>(format!("{} is no longer valid.", Self::LUA_NAME));
        }
        self.renderer.as_mut().unwrap()
    }
}
fn namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::DrawPath => {
            let (r, p, paint) = s.rive3_mut::<ScriptedRenderer, ScriptedPath, ScriptedPaint>();
            let path = p.render_path(s);
            r.validate(s)
                .draw_path(path, paint.render_paint.as_mut().unwrap());
            0
        }
        LuaAtoms::Save => {
            s.to_rive_mut::<ScriptedRenderer>(1).save(s);
            0
        }
        LuaAtoms::Restore => {
            s.to_rive_mut::<ScriptedRenderer>(1).restore(s);
            0
        }
        LuaAtoms::ClipPath => {
            let (r, p) = s.rive2_mut::<ScriptedRenderer, ScriptedPath>();
            r.clip_path(s, p);
            0
        }
        LuaAtoms::Transform => {
            let (r, m) = s.rive2_mut::<ScriptedRenderer, ScriptedMat2D>();
            r.transform(s, m.value);
            0
        }
        LuaAtoms::DrawImage => {
            let (r, image, sampler) =
                s.rive3_mut::<ScriptedRenderer, ScriptedImage, ScriptedImageSampler>();
            let blend = s.to_blend_mode(4);
            let opacity = s.check_number(5) as f32;
            r.validate(s).draw_image(
                image.image.as_ref().unwrap(),
                sampler.sampler,
                blend,
                opacity,
            );
            0
        }
        LuaAtoms::DrawImageMesh => {
            let(r,image,sampler,v,uv,index)=s.rive6_mut::<ScriptedRenderer,ScriptedImage,ScriptedImageSampler,ScriptedVertexBuffer,ScriptedVertexBuffer,ScriptedTriangleBuffer>();
            let blend = s.to_blend_mode(7);
            let opacity = s.check_number(8) as f32;
            let factory = s.thread_data::<dyn ScriptingContext>().factory();
            v.update(factory);
            uv.update(factory);
            index.update(factory);
            r.validate(s).draw_image_mesh(
                image.image.as_ref().unwrap(),
                sampler.sampler,
                v.vertex_buffer.as_ref(),
                uv.vertex_buffer.as_ref(),
                index.index_buffer.as_ref(),
                v.values.len() as u32,
                index.values.len() as u32,
                blend,
                opacity,
            );
            0
        }
        _ => s.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedRenderer::LUA_NAME
        )),
    }
}
pub fn luaopen_rive_renderer(s: &mut LuaState) -> i32 {
    s.register_rive::<ScriptedRenderer>();
    s.push_function(namecall);
    s.set_field(-2, "__namecall");
    s.set_readonly(-1, true);
    s.pop(1);
    0
}
