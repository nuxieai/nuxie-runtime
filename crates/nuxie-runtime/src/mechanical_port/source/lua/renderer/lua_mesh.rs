use crate::mechanical_port::source::{
    factory::Factory,
    lua::rive_lua_libs::*,
    math::vec2d::Vec2D,
    renderer::{RenderBufferFlags, RenderBufferType},
};
fn vertex_construct(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedVertexBuffer::default());
    1
}
fn vertex_reset(s: &mut LuaState) -> i32 {
    let v = s.to_rive_mut::<ScriptedVertexBuffer>(1);
    v.values.clear();
    v.vertex_buffer = None;
    0
}
fn vertex_add(s: &mut LuaState) -> i32 {
    let count = s.top() - 1;
    for i in 0..count {
        let value = *s.check_vec2d(2 + i);
        s.to_rive_mut::<ScriptedVertexBuffer>(1).values.push(value);
    }
    s.to_rive_mut::<ScriptedVertexBuffer>(1).vertex_buffer = None;
    0
}
fn vertex_namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::Reset => vertex_reset(s),
        LuaAtoms::Add => vertex_add(s),
        _ => s.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedVertexBuffer::LUA_NAME
        )),
    }
}
fn index_construct(s: &mut LuaState) -> i32 {
    s.new_rive(ScriptedTriangleBuffer::default());
    1
}
fn index_reset(s: &mut LuaState) -> i32 {
    let v = s.to_rive_mut::<ScriptedTriangleBuffer>(1);
    v.values.clear();
    v.max = 0;
    v.index_buffer = None;
    0
}
fn index_add(s: &mut LuaState) -> i32 {
    for i in 0..3 {
        let index = s.check_unsigned(2 + i);
        if index > u16::MAX as u32 {
            return s.error(format!("index {index} exceeds {}", u16::MAX));
        }
        let buffer = s.to_rive_mut::<ScriptedTriangleBuffer>(1);
        buffer.max = buffer.max.max(index);
        buffer.values.push(index as u16);
    }
    s.to_rive_mut::<ScriptedTriangleBuffer>(1).index_buffer = None;
    0
}
fn index_namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::Reset => index_reset(s),
        LuaAtoms::Add => index_add(s),
        _ => s.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedTriangleBuffer::LUA_NAME
        )),
    }
}
fn register_callable<T: LuaRive>(s: &mut LuaState, construct: LuaFunction, namecall: LuaFunction) {
    s.register(name_of::<T>(), &[LuaReg::END]);
    s.register_rive::<T>();
    s.push_function(namecall);
    s.set_field(-2, "__namecall");
    s.create_table(0, 1);
    s.push_function(construct);
    s.set_field(-2, "__call");
    s.set_metatable(-3);
    s.set_readonly(-1, true);
    s.pop(1);
}
impl ScriptedVertexBuffer {
    pub fn update(&mut self, factory: &mut Factory) {
        if self.vertex_buffer.is_some() {
            return;
        }
        let mut buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MAPPED_ONCE_AT_INITIALIZATION,
            self.values.len() * std::mem::size_of::<Vec2D>(),
        );
        if let Some(buffer) = buffer.as_mut() {
            if let Some(mapped) = buffer.map_as_mut::<Vec2D>() {
                mapped.copy_from_slice(&self.values);
                buffer.unmap();
            }
        }
        self.vertex_buffer = buffer;
    }
}
impl ScriptedTriangleBuffer {
    pub fn update(&mut self, factory: &mut Factory) {
        if self.index_buffer.is_some() {
            return;
        }
        let mut buffer = factory.make_render_buffer(
            RenderBufferType::Index,
            RenderBufferFlags::MAPPED_ONCE_AT_INITIALIZATION,
            self.values.len() * 2,
        );
        if let Some(buffer) = buffer.as_mut() {
            if let Some(mapped) = buffer.map_as_mut::<u16>() {
                mapped.copy_from_slice(&self.values);
                buffer.unmap();
            }
        }
        self.index_buffer = buffer;
    }
}
pub fn luaopen_rive_mesh(s: &mut LuaState) -> i32 {
    register_callable::<ScriptedVertexBuffer>(s, vertex_construct, vertex_namecall);
    register_callable::<ScriptedTriangleBuffer>(s, index_construct, index_namecall);
    2
}
