//! Direct port of pinned `src/lua/renderer/lua_mesh.cpp`.

use luaur_rt::{Error, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value};
use nuxie_render_api::{
    Factory as RenderFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, Vec2D,
};

pub(super) struct ScriptedVertexBuffer {
    values: Vec<Vec2D>,
    render_buffer: Option<Box<dyn RenderBuffer>>,
}

impl ScriptedVertexBuffer {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            render_buffer: None,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    pub(super) fn update(&mut self, factory: &mut dyn RenderFactory) {
        if self.render_buffer.is_some() {
            return;
        }
        let Some(size_in_bytes) = self.values.len().checked_mul(std::mem::size_of::<Vec2D>())
        else {
            return;
        };
        let mut buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            size_in_bytes,
        );
        let mapped = buffer.map_mut();
        for (chunk, value) in mapped.chunks_exact_mut(8).zip(&self.values) {
            chunk[..4].copy_from_slice(&value.x.to_ne_bytes());
            chunk[4..].copy_from_slice(&value.y.to_ne_bytes());
        }
        buffer.unmap();
        self.render_buffer = Some(buffer);
    }

    pub(super) fn render_buffer(&self) -> Option<&dyn RenderBuffer> {
        self.render_buffer.as_deref()
    }
}

impl UserData for ScriptedVertexBuffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("reset", |_, this, ()| {
            this.values.clear();
            this.render_buffer = None;
            Ok(())
        });
        methods.add_method_mut("add", |_, this, values: MultiValue| {
            for value in values {
                let Value::Vector(value) = value else {
                    return Err(Error::runtime("expected vector"));
                };
                this.values.push(Vec2D::new(value.x(), value.y()));
            }
            this.render_buffer = None;
            Ok(())
        });
    }
}

pub(super) struct ScriptedTriangleBuffer {
    values: Vec<u16>,
    max: u16,
    render_buffer: Option<Box<dyn RenderBuffer>>,
}

impl ScriptedTriangleBuffer {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            max: 0,
            render_buffer: None,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    pub(super) fn validate_for_vertices(&self, vertex_count: usize, uv_count: usize) -> Result<()> {
        if vertex_count != uv_count {
            return Err(Error::runtime(format!(
                "vertex and UV buffers differ in length ({vertex_count} != {uv_count})"
            )));
        }
        if !self.values.is_empty() && usize::from(self.max) >= vertex_count {
            return Err(Error::runtime(format!(
                "triangle index {} exceeds vertex buffer bounds {}",
                self.max, vertex_count
            )));
        }
        Ok(())
    }

    pub(super) fn update(&mut self, factory: &mut dyn RenderFactory) {
        if self.render_buffer.is_some() {
            return;
        }
        let Some(size_in_bytes) = self.values.len().checked_mul(std::mem::size_of::<u16>()) else {
            return;
        };
        let mut buffer = factory.make_render_buffer(
            RenderBufferType::Index,
            RenderBufferFlags::MappedOnceAtInitialization,
            size_in_bytes,
        );
        let mapped = buffer.map_mut();
        for (chunk, value) in mapped.chunks_exact_mut(2).zip(&self.values) {
            chunk.copy_from_slice(&value.to_ne_bytes());
        }
        buffer.unmap();
        self.render_buffer = Some(buffer);
    }

    pub(super) fn render_buffer(&self) -> Option<&dyn RenderBuffer> {
        self.render_buffer.as_deref()
    }
}

impl UserData for ScriptedTriangleBuffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("reset", |_, this, ()| {
            this.values.clear();
            this.max = 0;
            this.render_buffer = None;
            Ok(())
        });
        methods.add_method_mut("add", |_, this, (a, b, c): (u64, u64, u64)| {
            for index in [a, b, c] {
                let index = u16::try_from(index)
                    .map_err(|_| Error::runtime(format!("index {index} exceeds {}", u16::MAX)))?;
                this.max = this.max.max(index);
                this.values.push(index);
            }
            this.render_buffer = None;
            Ok(())
        });
    }
}

fn install_callable_constructor<T: UserData + 'static>(
    lua: &Lua,
    name: &str,
    constructor: impl Fn(&Lua) -> Result<T> + 'static,
) -> Result<()> {
    let table = lua.create_table();
    let metatable = lua.create_table();
    metatable.set(
        "__call",
        lua.create_function(move |lua, (_table,): (Table,)| {
            lua.create_userdata(constructor(lua)?)
        })?,
    )?;
    metatable.set_readonly(true);
    table.set_metatable(Some(metatable))?;
    table.set_readonly(true);
    lua.globals().set(name, table)
}

pub(super) fn install_mesh_globals(lua: &Lua) -> Result<()> {
    install_callable_constructor(lua, "VertexBuffer", |_| Ok(ScriptedVertexBuffer::new()))?;
    install_callable_constructor(lua, "TriangleBuffer", |_| Ok(ScriptedTriangleBuffer::new()))
}
