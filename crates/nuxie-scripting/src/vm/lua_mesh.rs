//! Direct port of pinned `src/lua/renderer/lua_mesh.cpp`.

use crate::luau::{
    AnyUserData, Error, FromLua, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value,
};
use std::cell::Cell;
use std::rc::Rc;

use nuxie_render_api::{
    Factory as RenderFactory, ImageMeshInstanceData, ImageMeshInstancesHandle, RenderBuffer,
    RenderBufferFlags, RenderBufferType, Vec2D,
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

/// Host memory one script VM's mesh instances may hold, staged here and
/// mirrored once by the renderer: as much again as the VM's own 16 MiB
/// ceiling, which the instances are outside of.
pub(super) const MAX_IMAGE_MESH_INSTANCE_BYTES: usize = 16 * 1024 * 1024;
/// 349,525 instances of 48 bytes.
pub(super) const MAX_IMAGE_MESH_INSTANCES: usize =
    MAX_IMAGE_MESH_INSTANCE_BYTES / std::mem::size_of::<ImageMeshInstanceData>();

/// Instances charged by every live `MeshInstances` of one VM, each charged
/// for the staging capacity it holds before allocating it.
#[derive(Clone, Default)]
struct MeshInstanceBudget(Rc<Cell<usize>>);

impl MeshInstanceBudget {
    /// The VM's budget, installed with the first `MeshInstances` global.
    fn for_lua(lua: &Lua) -> Self {
        if let Some(budget) = lua.app_data_ref::<Self>() {
            return budget.clone();
        }
        let budget = Self::default();
        lua.set_app_data(budget.clone());
        budget
    }

    fn fits(&self, count: usize) -> bool {
        count <= MAX_IMAGE_MESH_INSTANCES.saturating_sub(self.0.get())
    }

    fn charge(&self, count: usize) -> Result<()> {
        if !self.fits(count) {
            return Err(Error::runtime(format!(
                "MeshInstances may hold at most {MAX_IMAGE_MESH_INSTANCES} instances per script VM"
            )));
        }
        self.0.set(self.0.get().saturating_add(count));
        Ok(())
    }

    fn release(&self, count: usize) {
        self.0.set(self.0.get().saturating_sub(count));
    }
}

pub(super) struct ScriptedImageMeshInstances {
    pub(super) instances: ImageMeshInstancesHandle,
    staged: Vec<ImageMeshInstanceData>,
    dirty: bool,
    budget: MeshInstanceBudget,
    /// Instances charged to `budget`: the staging capacity asked for.
    charged: usize,
}

impl ScriptedImageMeshInstances {
    fn new(factory: &mut dyn RenderFactory, budget: MeshInstanceBudget, count: usize) -> Result<Self> {
        let mut staged = Vec::new();
        reserve_staging(&budget, &mut staged, 0, count)?;
        staged.resize(count, ImageMeshInstanceData::default());
        Ok(Self {
            instances: factory.make_image_mesh_instances(count),
            staged,
            dirty: false,
            budget,
            charged: count,
        })
    }

    fn resize(&mut self, count: usize) -> Result<()> {
        if self.staged.len() == count {
            return Ok(());
        }
        if count > self.charged {
            // Grow by doubling while the budget allows it, so growing one
            // instance at a time is not quadratic; otherwise exactly.
            let doubled = self.charged.saturating_mul(2).max(count);
            let target = if self.budget.fits(doubled.saturating_sub(self.charged)) {
                doubled
            } else {
                count
            };
            reserve_staging(&self.budget, &mut self.staged, self.charged, target)?;
            self.charged = target;
        }
        self.staged.resize(count, ImageMeshInstanceData::default());
        if count < self.charged / 2 {
            self.staged.shrink_to(count);
            self.budget.release(self.charged.saturating_sub(count));
            self.charged = count;
        }
        self.dirty = true;
        Ok(())
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
        instances
            .edit(Some(self.staged.len()))
            .copy_from_slice(&self.staged);
        instances.end_edit();
        self.dirty = false;
    }
}

/// Charges the growth from `charged` to `target` instances, then reserves
/// staging for `target`; a refused reservation is refunded.
fn reserve_staging(
    budget: &MeshInstanceBudget,
    staged: &mut Vec<ImageMeshInstanceData>,
    charged: usize,
    target: usize,
) -> Result<()> {
    let growth = target.saturating_sub(charged);
    budget.charge(growth)?;
    if staged
        .try_reserve_exact(target.saturating_sub(staged.len()))
        .is_err()
    {
        budget.release(growth);
        return Err(Error::runtime("cannot allocate MeshInstances"));
    }
    Ok(())
}

impl Drop for ScriptedImageMeshInstances {
    fn drop(&mut self) {
        self.budget.release(self.charged);
    }
}

impl UserData for ScriptedImageMeshInstances {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("resize", |_, this, count: u32| {
            this.resize(count as usize)
        });
        methods.add_method_mut("set", |lua, this, values: MultiValue| {
            let value = |index| values.get(index).cloned().unwrap_or(Value::Nil);
            let index = u32::from_lua(value(0), lua)? as usize;
            if index >= this.staged.len() {
                return Err(Error::runtime(format!(
                    "index {index} is past the end of MeshInstances"
                )));
            }
            let matrix = AnyUserData::from_lua(value(1), lua)?;
            let matrix = matrix.borrow::<super::lua_mat2d::ScriptedMat2D>()?.0;
            let data = this.stage(index);
            data.transform = matrix;
            data.opacity = Option::<f32>::from_lua(value(2), lua)?.unwrap_or(1.0);
            data.additiveness = Option::<f32>::from_lua(value(3), lua)?.unwrap_or(0.0);
            let vector = |index, default| -> Result<[f32; 2]> {
                if values.len() <= index {
                    return Ok(default);
                }
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
    let budget = MeshInstanceBudget::for_lua(lua);
    metatable.set(
        "__call",
        lua.create_function(move |lua, (_table, count): (Table, Option<u32>)| {
            let bindings = super::lua_renderer_library::RendererBindings::for_lua(lua)
                .ok_or_else(|| Error::runtime("renderer bindings are not installed"))?;
            let instances = bindings.with_factory(|factory| {
                ScriptedImageMeshInstances::new(
                    factory,
                    budget.clone(),
                    count.unwrap_or(0) as usize,
                )
            })?;
            lua.create_userdata(instances)
        })?,
    )?;
    metatable.set_readonly(true);
    table.set_metatable(Some(metatable))?;
    table.set_readonly(true);
    lua.globals().set("MeshInstances", table)
}
