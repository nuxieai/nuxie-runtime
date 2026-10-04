//! Direct port of pinned `src/lua/renderer/lua_mesh.cpp`.

use luaur_rt::{AnyUserData, Error, FromLua, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value};
use nuxie_render_api::{
    Factory as RenderFactory, RenderBuffer, RenderBufferFlags, RenderBufferType, Vec2D,
    ImageMeshInstancesHandle,
    ImageMeshInstanceData,
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

pub(super) struct ScriptedImageMeshInstances {
    pub(super) instances: ImageMeshInstancesHandle,
    staged: Vec<ImageMeshInstanceData>,
    dirty: bool,
}

impl ScriptedImageMeshInstances {
    fn new(factory: &mut dyn RenderFactory, count: usize) -> Self {
        Self {
            instances: factory.make_image_mesh_instances(count),
            staged: vec![ImageMeshInstanceData::default(); count],
            dirty: false,
        }
    }

    fn resize(&mut self, count: usize) {
        if self.staged.len() != count {
            self.staged.resize(count, ImageMeshInstanceData::default());
            self.dirty = true;
        }
    }

    fn stage(&mut self, index: usize) -> &mut ImageMeshInstanceData {
        self.dirty = true;
        &mut self.staged[index]
    }

    pub(super) fn commit(&mut self) {
        if !self.dirty {
            return;
        }
        let mut instances = self.instances.borrow_mut();
        instances.edit(Some(self.staged.len())).copy_from_slice(&self.staged);
        instances.end_edit();
        self.dirty = false;
    }
}

impl UserData for ScriptedImageMeshInstances {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |_, this, count: u32| {
            this.resize(count as usize);
            Ok(())
        });
        methods.add_method_mut("set", |lua, this, values: MultiValue| {
            let value = |index| values.get(index).cloned().unwrap_or(Value::Nil);
            let index = u32::from_lua(value(0), lua)? as usize;
            if index >= this.staged.len() {
                return Err(Error::runtime(format!("index {index} is past the end of MeshInstances")));
            }
            let matrix = AnyUserData::from_lua(value(1), lua)?;
            let matrix = matrix.borrow::<super::lua_mat2d::ScriptedMat2D>()?.0;
            let data = this.stage(index);
            data.transform = matrix;
            data.opacity = Option::<f32>::from_lua(value(2), lua)?.unwrap_or(1.0);
            data.additiveness = Option::<f32>::from_lua(value(3), lua)?.unwrap_or(0.0);
            let vector = |index, default| -> Result<[f32; 2]> {
                if values.len() <= index { return Ok(default); }
                match value(index) {
                    Value::Vector(v) => Ok([v.x(), v.y()]),
                    _ => Err(Error::runtime("expected vector")),
                }
            };
            data.uv_translate = vector(4, [0.0, 0.0])?;
            data.uv_scale = vector(5, [1.0, 1.0])?;
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
    install_callable_constructor(lua, "TriangleBuffer", |_| Ok(ScriptedTriangleBuffer::new()))?;
    let table = lua.create_table();
    let metatable = lua.create_table();
    metatable.set("__call", lua.create_function(|lua, (_table, count): (Table, Option<u32>)| {
        let bindings = super::lua_renderer_library::RendererBindings::for_lua(lua)
            .ok_or_else(|| Error::runtime("renderer bindings are not installed"))?;
        let instances = bindings.with_factory(|factory| {
            Ok(ScriptedImageMeshInstances::new(factory, count.unwrap_or(0) as usize))
        })?;
        lua.create_userdata(instances)
    })?)?;
    metatable.set_readonly(true);
    table.set_metatable(Some(metatable))?;
    table.set_readonly(true);
    lua.globals().set("MeshInstances", table)
}
