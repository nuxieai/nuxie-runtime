//! Direct port of pinned `src/lua/renderer/lua_mesh.cpp`.

use crate::luau::{
    AnyUserData, Error, FromLua, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value,
};
use std::cell::Cell;
use std::rc::Rc;

use super::host_memory_budget::{HostMemoryCharge, grown_capacity};

use nuxie_render_api::{
    Factory as RenderFactory, ImageMeshInstanceData, ImageMeshInstancesHandle, RenderBuffer,
    RenderBufferFlags, RenderBufferType, Vec2D,
};

/// Host bytes one vertex slot holds: the staged `Vec2D` and its mirror in
/// the render buffer `update` makes.
const VERTEX_SLOT_BYTES: usize = 2 * std::mem::size_of::<Vec2D>();
/// Host bytes one triangle-index slot holds: the staged `u16` and its mirror.
const INDEX_SLOT_BYTES: usize = 2 * std::mem::size_of::<u16>();

/// Makes room for `additional` more values, charging every slot of the
/// capacity it reserves (`slot_bytes` each) before reserving it.
fn reserve_values<T>(
    lua: &Lua,
    values: &mut Vec<T>,
    charge: &mut HostMemoryCharge,
    additional: usize,
    slot_bytes: usize,
) -> Result<()> {
    let capacity = grown_capacity(values.capacity(), values.len(), additional);
    if capacity == values.capacity() {
        return Ok(());
    }
    charge.attach(lua)?;
    charge.reserve_to(capacity.saturating_mul(slot_bytes))?;
    values
        .try_reserve_exact(capacity.saturating_sub(values.len()))
        .map_err(|_| Error::runtime("not enough memory"))
}

pub(super) struct ScriptedVertexBuffer {
    values: Vec<Vec2D>,
    render_buffer: Option<Box<dyn RenderBuffer>>,
    charge: HostMemoryCharge,
}

impl ScriptedVertexBuffer {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            render_buffer: None,
            charge: HostMemoryCharge::default(),
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
            let [x0, x1, x2, x3] = value.x.to_ne_bytes();
            let [y0, y1, y2, y3] = value.y.to_ne_bytes();
            for (byte, value) in chunk.iter_mut().zip([x0, x1, x2, x3, y0, y1, y2, y3]) {
                *byte = value;
            }
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
        methods.add_method_mut("add", |lua, this, values: MultiValue| {
            reserve_values(
                lua,
                &mut this.values,
                &mut this.charge,
                values.len(),
                VERTEX_SLOT_BYTES,
            )?;
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
    charge: HostMemoryCharge,
}

impl ScriptedTriangleBuffer {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            max: 0,
            render_buffer: None,
            charge: HostMemoryCharge::default(),
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
        methods.add_method_mut("add", |lua, this, (a, b, c): (u64, u64, u64)| {
            reserve_values(lua, &mut this.values, &mut this.charge, 3, INDEX_SLOT_BYTES)?;
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

/// Instances one script VM's `MeshInstances` may hold together, a fixed
/// count beside the VM's host memory budget (which their bytes are charged
/// to): 16 MiB of 48-byte instances, 349,525.
pub(super) const MAX_IMAGE_MESH_INSTANCE_BYTES: usize = 16 * 1024 * 1024;
/// 349,525 instances of 48 bytes.
pub(super) const MAX_IMAGE_MESH_INSTANCES: usize =
    MAX_IMAGE_MESH_INSTANCE_BYTES / std::mem::size_of::<ImageMeshInstanceData>();
/// Host bytes one charged instance may hold: its staging slot here and the
/// renderer's mirror (`ImageMeshInstancesStorage::edit`, a `Vec::resize` that
/// grows by doubling, so up to two slots per instance of the largest count).
const IMAGE_MESH_INSTANCE_HOST_BYTES: usize = 3 * std::mem::size_of::<ImageMeshInstanceData>();

/// Instances counted by every live `MeshInstances` of one VM, each counted
/// for the largest staging capacity it has asked for (which also bounds the
/// renderer's mirror) before allocating it, and released when it is dropped.
#[derive(Clone, Default)]
struct MeshInstanceCount(Rc<Cell<usize>>);

impl MeshInstanceCount {
    /// The VM's count, installed with the first `MeshInstances` global.
    fn for_lua(lua: &Lua) -> Result<Self> {
        if let Some(count) = lua.try_app_data_ref::<Self>()? {
            return Ok(count.clone());
        }
        let count = Self::default();
        lua.try_set_app_data(count.clone())?;
        Ok(count)
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

/// What one `MeshInstances` holds against its VM: the instance count and
/// the host bytes of the largest staging capacity it has asked for. Both
/// are released together when it is dropped.
struct MeshInstanceCharge {
    count: MeshInstanceCount,
    bytes: HostMemoryCharge,
    /// Instances charged: the largest staging capacity asked for.
    instances: usize,
}

impl MeshInstanceCharge {
    fn new(lua: &Lua, count: MeshInstanceCount) -> Result<Self> {
        let mut bytes = HostMemoryCharge::default();
        bytes.attach(lua)?;
        Ok(Self {
            count,
            bytes,
            instances: 0,
        })
    }

    /// Whether `target` instances in all would fit both limits.
    fn fits(&self, target: usize) -> bool {
        let growth = target.saturating_sub(self.instances);
        self.count.fits(growth)
            && self
                .bytes
                .fits(target.saturating_mul(IMAGE_MESH_INSTANCE_HOST_BYTES))
    }

    /// Charges the growth to `target` instances; the count is checked first,
    /// so its error is the one a script sees when both would refuse.
    fn charge_to(&mut self, target: usize) -> Result<()> {
        let growth = target.saturating_sub(self.instances);
        self.count.charge(growth)?;
        if let Err(error) = self
            .bytes
            .reserve_to(target.saturating_mul(IMAGE_MESH_INSTANCE_HOST_BYTES))
        {
            self.count.release(growth);
            return Err(error);
        }
        self.instances = self.instances.max(target);
        Ok(())
    }
}

impl Drop for MeshInstanceCharge {
    fn drop(&mut self) {
        self.count.release(self.instances);
    }
}

pub(super) struct ScriptedImageMeshInstances {
    pub(super) instances: ImageMeshInstancesHandle,
    staged: Vec<ImageMeshInstanceData>,
    dirty: bool,
    charge: MeshInstanceCharge,
}

impl ScriptedImageMeshInstances {
    fn new(
        factory: &mut dyn RenderFactory,
        mut charge: MeshInstanceCharge,
        count: usize,
    ) -> Result<Self> {
        let mut staged = Vec::new();
        reserve_staging(&mut charge, &mut staged, count)?;
        staged.resize(count, ImageMeshInstanceData::default());
        Ok(Self {
            instances: factory.make_image_mesh_instances(count),
            staged,
            dirty: false,
            charge,
        })
    }

    fn resize(&mut self, count: usize) -> Result<()> {
        if self.staged.len() == count {
            return Ok(());
        }
        let charged = self.charge.instances;
        if count > charged {
            // Grow by doubling while the limits allow it, so growing one
            // instance at a time is not quadratic; otherwise exactly.
            let doubled = charged.saturating_mul(2).max(count);
            let target = if self.charge.fits(doubled) {
                doubled
            } else {
                count
            };
            reserve_staging(&mut self.charge, &mut self.staged, target)?;
        }
        // A shrink refunds nothing: the renderer's mirror keeps the capacity
        // of the largest count it was given, so the charge stays at that high
        // water mark until the object is dropped.
        self.staged.resize(count, ImageMeshInstanceData::default());
        self.dirty = true;
        Ok(())
    }

    /// The staged instance at `index`, or the script error for an index past
    /// the end.
    fn stage(&mut self, index: usize) -> Result<&mut ImageMeshInstanceData> {
        let data = self.staged.get_mut(index).ok_or_else(|| {
            Error::runtime(format!("index {index} is past the end of MeshInstances"))
        })?;
        self.dirty = true;
        Ok(data)
    }

    pub(super) fn commit(&mut self) {
        if !self.dirty {
            return;
        }
        let mut instances = self.instances.borrow_mut();
        let mirror = instances.edit(Some(self.staged.len()));
        for (slot, value) in mirror.iter_mut().zip(&self.staged) {
            *slot = *value;
        }
        instances.end_edit();
        self.dirty = false;
    }
}

/// Charges the growth to `target` instances, then reserves staging for
/// `target`; a refused reservation is refunded.
fn reserve_staging(
    charge: &mut MeshInstanceCharge,
    staged: &mut Vec<ImageMeshInstanceData>,
    target: usize,
) -> Result<()> {
    let instances = charge.instances;
    let bytes = charge.bytes.bytes();
    charge.charge_to(target)?;
    if staged
        .try_reserve_exact(target.saturating_sub(staged.len()))
        .is_err()
    {
        charge.count.release(charge.instances.saturating_sub(instances));
        charge.instances = instances;
        // Lowering a charge always succeeds.
        charge.bytes.set(bytes)?;
        return Err(Error::runtime("cannot allocate MeshInstances"));
    }
    Ok(())
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
            let data = this.stage(index)?;
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
    let instance_count = MeshInstanceCount::for_lua(lua)?;
    metatable.set(
        "__call",
        lua.create_function(move |lua, (_table, count): (Table, Option<u32>)| {
            let bindings = super::lua_renderer_library::RendererBindings::for_lua(lua)
                .ok_or_else(|| Error::runtime("renderer bindings are not installed"))?;
            let charge = MeshInstanceCharge::new(lua, instance_count.clone())?;
            let instances = bindings.with_factory(|factory| {
                ScriptedImageMeshInstances::new(factory, charge, count.unwrap_or(0) as usize)
            })?;
            lua.create_userdata(instances)
        })?,
    )?;
    metatable.set_readonly(true);
    table.set_metatable(Some(metatable))?;
    table.set_readonly(true);
    lua.globals().set("MeshInstances", table)
}
