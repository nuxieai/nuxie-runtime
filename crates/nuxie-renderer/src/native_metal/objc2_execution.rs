//! Production Objective-C execution owner for the source-shaped Metal port.

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject, Sel};
use objc2::{msg_send, ClassType, Message};
use objc2_foundation::{NSError, NSString};
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLDevice, MTLLibrary,
    MTLPixelFormat, MTLRenderPipelineDescriptor, MTLResourceOptions, MTLSamplerDescriptor, MTLTexture,
    MTLTextureDescriptor, MTLTextureUsage, MTLGPUFamily,
};
use std::ffi::{c_void, CString};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// Match the pinned simulator branch's NXGetLocalArchInfo() query.  In
// particular, `target_arch` describes the simulator binary, not the host
// running it, so it cannot replace this source query.
#[cfg(target_vendor = "apple")]
#[repr(C)]
struct NxArchInfo {
    _name: *const std::ffi::c_char,
    cputype: i32,
    _cpusubtype: i32,
    _byteorder: i32,
    _description: *const std::ffi::c_char,
}

#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    fn NXGetLocalArchInfo() -> *const NxArchInfo;
}

use crate::mechanical_metal_implementation::source_execution::{
    DrawType, Handle, HostExecution, InterlockMode, MetalAliasValidity, MetalExecution,
    MetalObjectKind, ObjectCreation, OwnedMetalHandle, PipelineSemantic, PipelineSemanticKind, Value,
    PixelFormat,
};
#[cfg(test)]
use crate::mechanical_metal_implementation::source_execution::SOURCE_STATIC_FUNCTION_NAMES;

fn source_static_function_name(name: &str) -> Option<&'static NSString> {
    Some(match name {
        "EF" => objc2_foundation::ns_string!("EF"),
        "FF" => objc2_foundation::ns_string!("FF"),
        "WF" => objc2_foundation::ns_string!("WF"),
        "XF" => objc2_foundation::ns_string!("XF"),
        "RF" => objc2_foundation::ns_string!("RF"),
        "UE" => objc2_foundation::ns_string!("UE"),
        "VE" => objc2_foundation::ns_string!("VE"),
        "GC" => objc2_foundation::ns_string!("GC"),
        "JB" => objc2_foundation::ns_string!("JB"),
        _ => return None,
    })
}

pub(crate) trait NativeMetalHostCallbacks {
    fn log(&mut self, message: String);
    fn generate_patch_buffer_data(
        &mut self,
        vertex_buffer: &ProtocolObject<dyn MTLBuffer>,
        index_buffer: &ProtocolObject<dyn MTLBuffer>,
    );
    fn make_ore_context(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        queue: Option<&ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Option<Box<dyn std::any::Any>>;
}

enum RetainedMetalObject {
    ObjectiveC(Retained<AnyObject>),
    /// Generation-checked alias into a canonical source owner's +1 retain.
    /// The registry owns no Objective-C retain in this state.
    BorrowedObjectiveC {
        object: NonNull<AnyObject>,
        validity: MetalAliasValidity,
    },
    Dispatch {
        object: NonNull<AnyObject>,
        _bytes: Arc<[u8]>,
    },
    Host(Box<dyn std::any::Any>),
}

struct RegistryEntry {
    kind: MetalObjectKind,
    object: RetainedMetalObject,
    children: Vec<Handle>,
    pipeline_semantic: Option<PipelineSemantic>,
    encoder_pipeline: Option<PipelineSemantic>,
}

struct RegistrySlot {
    generation: u64,
    entry: Option<RegistryEntry>,
}

#[derive(Clone, Copy)]
struct DeferredRetirement {
    handle: Handle,
    kind: MetalObjectKind,
}

/// Cloneable destruction edge for intrusive source owners whose destructor no
/// longer has a mutable reference to the execution table. Retirements remain
/// typed and generation checked when the owning execution next drains them.
#[derive(Clone, Default)]
pub(crate) struct Objc2MetalRetirementQueue {
    pending: Arc<Mutex<Vec<DeferredRetirement>>>,
}

impl Objc2MetalRetirementQueue {
    fn push(&self, handle: Handle, kind: MetalObjectKind) {
        if handle == Handle::NIL || handle.kind != kind {
            return;
        }
        self.pending
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .push(DeferredRetirement { handle, kind });
    }

    pub(crate) fn retire_texture(&self, texture: Handle) {
        self.push(texture, MetalObjectKind::Texture);
    }

    pub(crate) fn retire_buffer(&self, buffer: Handle) {
        self.push(buffer, MetalObjectKind::Buffer);
    }

    pub(crate) fn retire_command_queue(&self, queue: Handle) {
        self.push(queue, MetalObjectKind::CommandQueue);
    }

    pub(crate) fn retire_textures(&self, textures: impl IntoIterator<Item = Handle>) {
        for texture in textures {
            self.retire_texture(texture);
        }
    }

    pub(crate) fn retire_buffers(&self, buffers: impl IntoIterator<Item = Handle>) {
        for buffer in buffers {
            self.retire_buffer(buffer);
        }
    }
}

/// Terminal state reported by the exact Metal command buffer passed to a
/// completion block. Keeping status and error separate lets the product layer
/// preserve Metal's completed-vs-error distinction without retaining an
/// Objective-C error object across the callback boundary.
#[derive(Clone, Debug)]
pub(crate) struct NativeMetalCommandBufferCompletion {
    pub(crate) status: MTLCommandBufferStatus,
    pub(crate) error: Option<String>,
}

/// Frame-local evidence emitted only at the native selector boundary. This is
/// deliberately raw: the product layer maps these successful bindings and
/// submissions into its public inventory after command-buffer completion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActualMetalExecutionInventory {
    pub(crate) pipeline_creations: usize,
    pub(crate) pipeline_binds: usize,
    /// Successful physical Metal draw selector submissions across all passes.
    pub(crate) draw_calls: usize,
    /// Instances submitted by the physical Metal draw selectors above.
    pub(crate) draw_instances: usize,
    /// Logical flushes executed by the pinned source RenderContext.
    pub(crate) logical_flushes: usize,
    /// Physical atomic-content draw selectors, excluding PLS initialize/resolve.
    pub(crate) atomic_draw_calls: usize,
    /// Instances submitted by `atomic_draw_calls`.
    pub(crate) atomic_draw_instances: usize,
    /// Atomic content draws whose shader writes the offscreen color plane.
    /// Metal binds the color buffer at pass setup even when every content
    /// pipeline uses fixed-function color output, so binding counts alone do
    /// not establish that the plane was used.
    pub(crate) atomic_color_plane_draw_calls: usize,
    /// Atomic content groups observed after source-executed group boundaries.
    pub(crate) draw_groups: usize,
    /// Source-executed atomic boundaries, independent of the device policy.
    pub(crate) semantic_atomic_barriers: usize,
    /// Physical Metal memory-barrier selectors successfully submitted.
    pub(crate) memory_barriers: usize,
    /// Successful render-pass replacements used as physical barriers.
    pub(crate) render_pass_breaks: usize,
    /// Validated source ROG boundaries; Metal emits no selector for these.
    pub(crate) raster_order_group_barriers: usize,
    pub(crate) color_ramp_draw_calls: usize,
    pub(crate) tessellation_draw_calls: usize,
    pub(crate) feather_fill_draw_calls: usize,
    pub(crate) feather_stroke_draw_calls: usize,
    pub(crate) midpoint_fan_draw_calls: usize,
    pub(crate) outer_curve_draw_calls: usize,
    pub(crate) interior_triangulation_draw_calls: usize,
    pub(crate) image_rect_draw_calls: usize,
    pub(crate) image_mesh_draw_calls: usize,
    pub(crate) clip_reset_draw_calls: usize,
    pub(crate) render_pass_initialize_draw_calls: usize,
    pub(crate) render_pass_resolve_draw_calls: usize,
    pub(crate) clip_feature_draw_calls: usize,
    pub(crate) clip_rect_feature_draw_calls: usize,
    pub(crate) advanced_blend_draw_calls: usize,
    pub(crate) hsl_blend_draw_calls: usize,
    pub(crate) fixed_function_draw_calls: usize,
    pub(crate) gradient_texture_binds: usize,
    /// Successful source image-texture bindings at IMAGE_TEXTURE_IDX.
    pub(crate) image_texture_binds: usize,
    pub(crate) color_atomic_buffer_binds: usize,
    pub(crate) clip_atomic_buffer_binds: usize,
    pub(crate) coverage_atomic_buffer_binds: usize,
    pub(crate) color_attachment_binds: usize,
    pub(crate) clip_attachment_binds: usize,
    pub(crate) coverage_attachment_binds: usize,
    pub(crate) executed_shader_features: u32,
    pub(crate) executed_shader_misc: u32,
}

#[derive(Default)]
struct ExecutionInventoryState {
    snapshot: ActualMetalExecutionInventory,
    saw_atomic_draw: bool,
    atomic_group_boundary: bool,
}

impl NativeMetalCommandBufferCompletion {
    pub(crate) fn succeeded(&self) -> bool {
        self.status == MTLCommandBufferStatus::Completed
    }

    pub(crate) fn into_result(self) -> Result<(), String> {
        if self.succeeded() {
            return Ok(());
        }
        let detail = self
            .error
            .unwrap_or_else(|| format!("status {:?}", self.status));
        Err(format!("command buffer failed: {detail}"))
    }
}

impl Drop for RetainedMetalObject {
    fn drop(&mut self) {
        if let Self::Dispatch { object, .. } = self {
            unsafe { super::dispatch_release(object.as_ptr()) };
        }
    }
}

/// One retained object table for every handle published to the mechanical
/// renderer. Handles are kind checked before dispatch and never expose an
/// unretained Objective-C pointer.
pub(crate) struct Objc2MetalExecution {
    registry_id: u64,
    // Nonowning selector receiver. Before canonical construction the registry
    // creation entry owns the device; after `take_owned`, RenderContextMetal's
    // m_gpu owns it. The recording-thread/drop ordering prevents use after the
    // canonical owner is destroyed.
    device: NonNull<ProtocolObject<dyn MTLDevice>>,
    device_handle: Handle,
    host: Box<dyn NativeMetalHostCallbacks>,
    objects: Vec<RegistrySlot>,
    free_slots: Vec<u32>,
    retirement_queue: Objc2MetalRetirementQueue,
    execution_inventory: ExecutionInventoryState,
    #[cfg(test)]
    adopted_background_libraries: Vec<(Handle, usize)>,
    // Selector dispatch and canonical owner destruction are confined to one
    // recording thread. This makes a borrowed alias pointer impossible to race
    // with `OwnedMetalHandle::drop`; both types are explicitly !Send/!Sync.
    _recording_thread: core::marker::PhantomData<std::rc::Rc<()>>,
}

static NEXT_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

fn next_registry_id() -> u64 {
    let id = NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed);
    assert_ne!(id, 0, "Metal registry identity exhausted");
    id
}

impl Drop for Objc2MetalExecution {
    fn drop(&mut self) {
        // The executor owns only aliases for transferred source members.
        // Invalidate those aliases before the table disappears; canonical
        // owners retain and release the native objects independently.
        for slot in &self.objects {
            if let Some(RegistryEntry {
                object: RetainedMetalObject::BorrowedObjectiveC { validity, .. },
                ..
            }) = &slot.entry
            {
                validity.invalidate();
            }
        }
    }
}

impl Objc2MetalExecution {
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        host: Box<dyn NativeMetalHostCallbacks>,
    ) -> Self {
        let device_ptr = NonNull::from(&*device);
        let mut this = Self {
            registry_id: next_registry_id(),
            device: device_ptr,
            device_handle: Handle::NIL,
            host,
            objects: Vec::new(),
            free_slots: Vec::new(),
            retirement_queue: Objc2MetalRetirementQueue::default(),
            execution_inventory: ExecutionInventoryState::default(),
            #[cfg(test)]
            adopted_background_libraries: Vec::new(),
            _recording_thread: core::marker::PhantomData,
        };
        this.device_handle = this.insert(device, MetalObjectKind::Device);
        this
    }

    fn device(&self) -> &ProtocolObject<dyn MTLDevice> {
        // SAFETY: documented by `device`: either the pre-transfer registry
        // entry or the canonical m_gpu owner is live whenever selectors run.
        unsafe { self.device.as_ref() }
    }

    fn device_for_handle(&self, handle: Handle) -> Option<&ProtocolObject<dyn MTLDevice>> {
        let object = self.object(handle, MetalObjectKind::Device)?;
        // SAFETY: the registry kind check above is the typed provenance for
        // this protocol projection; the registry entry keeps the object live
        // for the synchronous selector call.
        Some(unsafe {
            &*(object as *const AnyObject as *const ProtocolObject<dyn MTLDevice>)
        })
    }

    fn insert<T: Message + 'static>(
        &mut self,
        object: Retained<T>,
        kind: MetalObjectKind,
    ) -> Handle {
        let object = unsafe { Retained::cast_unchecked::<AnyObject>(object) };
        self.insert_entry(RegistryEntry {
            kind,
            object: RetainedMetalObject::ObjectiveC(object),
            children: Vec::new(),
            pipeline_semantic: None,
            encoder_pipeline: None,
        })
    }

    fn insert_dispatch(&mut self, object: NonNull<AnyObject>, bytes: Arc<[u8]>) -> Handle {
        self.insert_entry(RegistryEntry {
            kind: MetalObjectKind::DispatchData,
            object: RetainedMetalObject::Dispatch {
                object,
                _bytes: bytes,
            },
            children: Vec::new(),
            pipeline_semantic: None,
            encoder_pipeline: None,
        })
    }

    fn insert_borrowed_object(
        &mut self,
        object: NonNull<AnyObject>,
        kind: MetalObjectKind,
        validity: MetalAliasValidity,
    ) -> Handle {
        self.insert_entry(RegistryEntry {
            kind,
            object: RetainedMetalObject::BorrowedObjectiveC { object, validity },
            children: Vec::new(),
            pipeline_semantic: None,
            encoder_pipeline: None,
        })
    }

    /// Publishes a selector-returned child as a +0 alias of its owning
    /// descriptor.  Metal's attachment collection and child accessors are
    /// borrowed Objective-C properties; retaining each result independently
    /// changes the source ARC scope and can keep a child alive after its
    /// descriptor has gone away.  The parent/child edge invalidates and
    /// reclaims this alias when the parent slot is retired.
    fn insert_borrowed_child(
        &mut self,
        parent: Handle,
        object: NonNull<AnyObject>,
        kind: MetalObjectKind,
    ) -> Option<Handle> {
        if self.owner(parent, parent.kind).is_none() {
            return None;
        }
        let validity = MetalAliasValidity::live();
        let child = self.insert_borrowed_object(object, kind, validity);
        if self.link_child(parent, child) {
            Some(child)
        } else {
            self.retire(child, kind);
            None
        }
    }

    fn insert_entry(&mut self, entry: RegistryEntry) -> Handle {
        self.drain_retirements();
        self.reap_invalidated_aliases();
        let kind = entry.kind;
        if let Some(index) = self.free_slots.pop() {
            let slot = &mut self.objects[index as usize];
            debug_assert!(slot.entry.is_none());
            slot.entry = Some(entry);
            return Handle::with_registry(index + 1, kind, slot.generation, self.registry_id);
        }
        self.objects.push(RegistrySlot {
            generation: 1,
            entry: Some(entry),
        });
        Handle::with_registry(self.objects.len() as u32, kind, 1, self.registry_id)
    }

    fn owner(&self, handle: Handle, expected: MetalObjectKind) -> Option<&RetainedMetalObject> {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return None;
        }
        let slot = self.objects.get(handle.slot.checked_sub(1)? as usize)?;
        if slot.generation != handle.generation {
            return None;
        }
        let entry = slot.entry.as_ref()?;
        if entry.kind != expected || !Self::entry_is_live(entry) {
            return None;
        }
        Some(&entry.object)
    }

    fn entry(&self, handle: Handle, expected: MetalObjectKind) -> Option<&RegistryEntry> {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return None;
        }
        let slot = self.objects.get(handle.slot.checked_sub(1)? as usize)?;
        if slot.generation != handle.generation {
            return None;
        }
        let entry = slot.entry.as_ref()?;
        if entry.kind != expected || !Self::entry_is_live(entry) {
            return None;
        }
        Some(entry)
    }

    fn entry_mut(
        &mut self,
        handle: Handle,
        expected: MetalObjectKind,
    ) -> Option<&mut RegistryEntry> {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return None;
        }
        let slot = self.objects.get_mut(handle.slot.checked_sub(1)? as usize)?;
        if slot.generation != handle.generation {
            return None;
        }
        let entry = slot.entry.as_mut()?;
        if entry.kind != expected || !Self::entry_is_live(entry) {
            return None;
        }
        Some(entry)
    }

    fn object(&self, handle: Handle, expected: MetalObjectKind) -> Option<&AnyObject> {
        match self.owner(handle, expected)? {
            RetainedMetalObject::ObjectiveC(object) => Some(object),
            RetainedMetalObject::BorrowedObjectiveC { object, .. } => {
                Some(unsafe { object.as_ref() })
            }
            RetainedMetalObject::Dispatch { .. } | RetainedMetalObject::Host(_) => None,
        }
    }

    fn cloned_object(
        &self,
        handle: Handle,
        expected: MetalObjectKind,
    ) -> Option<Retained<AnyObject>> {
        match self.owner(handle, expected)? {
            RetainedMetalObject::ObjectiveC(object) => Some(object.clone()),
            RetainedMetalObject::BorrowedObjectiveC { object, .. } => {
                unsafe { Retained::retain(object.as_ptr()) }
            }
            RetainedMetalObject::Dispatch { .. } | RetainedMetalObject::Host(_) => None,
        }
    }

    fn dispatch(&self, handle: Handle) -> Option<*mut AnyObject> {
        match self.owner(handle, MetalObjectKind::DispatchData)? {
            RetainedMetalObject::Dispatch { object, .. } => Some(object.as_ptr()),
            RetainedMetalObject::ObjectiveC(_)
            | RetainedMetalObject::BorrowedObjectiveC { .. }
            | RetainedMetalObject::Host(_) => None,
        }
    }

    fn entry_is_live(entry: &RegistryEntry) -> bool {
        match &entry.object {
            RetainedMetalObject::BorrowedObjectiveC { validity, .. } => validity.is_live(),
            RetainedMetalObject::ObjectiveC(_)
            | RetainedMetalObject::Dispatch { .. }
            | RetainedMetalObject::Host(_) => true,
        }
    }

    fn reclaim_slot(&mut self, index: usize) -> Option<RegistryEntry> {
        let slot = self.objects.get_mut(index)?;
        let entry = slot.entry.take()?;
        if slot.generation != u64::MAX {
            slot.generation += 1;
            self.free_slots.push(index as u32);
        }
        Some(entry)
    }

    fn reap_invalidated_aliases(&mut self) {
        let invalidated: Vec<usize> = self
            .objects
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let entry = slot.entry.as_ref()?;
                (!Self::entry_is_live(entry)).then_some(index)
            })
            .collect();
        for index in invalidated {
            let Some(entry) = self.reclaim_slot(index) else {
                continue;
            };
            for child in entry.children {
                self.retire(child, child.kind);
            }
        }
    }

    fn take_owned_object(
        &mut self,
        handle: Handle,
        expected: MetalObjectKind,
    ) -> Option<(Retained<AnyObject>, MetalAliasValidity)> {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return None;
        }
        let index = handle.slot.checked_sub(1)? as usize;
        let slot = self.objects.get_mut(index)?;
        if slot.generation != handle.generation || slot.entry.as_ref()?.kind != expected {
            return None;
        }
        let entry = slot.entry.as_mut()?;
        if !matches!(entry.object, RetainedMetalObject::ObjectiveC(_)) {
            return None;
        }
        let old = core::mem::replace(
            &mut entry.object,
            RetainedMetalObject::Host(Box::new(())),
        );
        let mut old = core::mem::ManuallyDrop::new(old);
        let RetainedMetalObject::ObjectiveC(object) = &mut *old else {
            unreachable!("entry kind checked before ownership transfer")
        };
        // SAFETY: `old` is ManuallyDrop and this field is read exactly once;
        // the enum's Drop implementation will not run for this moved value.
        let object = unsafe { core::ptr::read(object) };
        let validity = MetalAliasValidity::live();
        entry.object = RetainedMetalObject::BorrowedObjectiveC {
            object: NonNull::from(&*object),
            validity: validity.clone(),
        };
        Some((object, validity))
    }

    fn insert_host(&mut self, object: Box<dyn std::any::Any>, kind: MetalObjectKind) -> Handle {
        self.insert_entry(RegistryEntry {
            kind,
            object: RetainedMetalObject::Host(object),
            children: Vec::new(),
            pipeline_semantic: None,
            encoder_pipeline: None,
        })
    }

    fn link_child(&mut self, parent: Handle, child: Handle) -> bool {
        if parent == Handle::NIL || child == Handle::NIL {
            return false;
        }
        if parent.registry != self.registry_id || child.registry != self.registry_id {
            return false;
        }
        if self.owner(child, child.kind).is_none() {
            return false;
        }
        let Some(index) = parent.slot.checked_sub(1) else {
            return false;
        };
        let Some(slot) = self.objects.get_mut(index as usize) else {
            return false;
        };
        if slot.generation != parent.generation {
            return false;
        }
        let Some(entry) = slot.entry.as_mut() else {
            return false;
        };
        if entry.kind != parent.kind {
            return false;
        }
        if !entry.children.contains(&child) {
            entry.children.push(child);
        }
        true
    }

    fn take_owner(
        &mut self,
        handle: Handle,
        expected: MetalObjectKind,
    ) -> Option<RetainedMetalObject> {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return None;
        }
        let index = handle.slot.checked_sub(1)? as usize;
        let slot = self.objects.get_mut(index)?;
        if slot.generation != handle.generation || slot.entry.as_ref()?.kind != expected {
            return None;
        }
        let entry = self.reclaim_slot(index)?;
        for child in entry.children {
            self.take_owner(child, child.kind);
        }
        Some(entry.object)
    }

    fn retire(&mut self, handle: Handle, expected: MetalObjectKind) -> bool {
        if handle == Handle::NIL || handle.kind != expected || handle.registry != self.registry_id {
            return false;
        }
        let Some(index) = handle.slot.checked_sub(1).map(|index| index as usize) else {
            return false;
        };
        let Some(slot) = self.objects.get(index) else {
            return false;
        };
        if slot.generation != handle.generation
            || slot.entry.as_ref().map(|entry| entry.kind) != Some(expected)
        {
            return false;
        }
        let Some(entry) = self.reclaim_slot(index) else {
            return false;
        };
        for child in entry.children {
            self.retire(child, child.kind);
        }
        #[cfg(test)]
        if let Some(index) = self
            .adopted_background_libraries
            .iter()
            .position(|(candidate, _)| *candidate == handle)
        {
            let (_, identity) = self.adopted_background_libraries.remove(index);
            crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::record_compiled_library_context_phase(
                "ReleaseContext",
                identity,
            );
        }
        true
    }

    fn drain_retirements(&mut self) {
        let pending = {
            let mut pending = self
                .retirement_queue
                .pending
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            core::mem::take(&mut *pending)
        };
        for retirement in pending {
            self.retire(retirement.handle, retirement.kind);
        }
    }

    pub(crate) fn retirement_queue(&self) -> Objc2MetalRetirementQueue {
        self.retirement_queue.clone()
    }

    /// Starts one frame-local native execution evidence window. Persistent
    /// pipeline tags remain in the generation-checked registry.
    pub(crate) fn reset_execution_inventory(&mut self) {
        self.execution_inventory = ExecutionInventoryState::default();
    }

    /// Snapshots only events whose Objective-C selector was actually submitted
    /// against a live, typed registry owner since the last reset.
    pub(crate) fn snapshot_execution_inventory(&self) -> ActualMetalExecutionInventory {
        self.execution_inventory.snapshot
    }

    pub(crate) fn record_logical_flushes(&mut self, logical_flushes: usize) {
        self.execution_inventory.snapshot.logical_flushes = logical_flushes;
    }

    fn pipeline_semantic(&self, pipeline: Handle) -> Option<PipelineSemantic> {
        self.entry(pipeline, MetalObjectKind::RenderPipelineState)?
            .pipeline_semantic
    }

    fn bind_pipeline_semantic(&mut self, encoder: Handle, semantic: Option<PipelineSemantic>) {
        if let Some(entry) = self.entry_mut(encoder, MetalObjectKind::RenderCommandEncoder) {
            entry.encoder_pipeline = semantic;
            if semantic.is_some() {
                self.execution_inventory.snapshot.pipeline_binds += 1;
            }
        }
    }

    fn record_draw_submission(&mut self, encoder: Handle, instances: usize) {
        let semantic = self
            .entry(encoder, MetalObjectKind::RenderCommandEncoder)
            .and_then(|entry| entry.encoder_pipeline);
        let inventory = &mut self.execution_inventory;
        inventory.snapshot.draw_calls += 1;
        inventory.snapshot.draw_instances += instances;
        let Some(semantic) = semantic else {
            return;
        };

        inventory.snapshot.executed_shader_features |= semantic.features.0;
        inventory.snapshot.executed_shader_misc |= semantic.misc.0;
        match semantic.kind {
            PipelineSemanticKind::ColorRamp => inventory.snapshot.color_ramp_draw_calls += 1,
            PipelineSemanticKind::Tessellate => inventory.snapshot.tessellation_draw_calls += 1,
            PipelineSemanticKind::FeatherFill => inventory.snapshot.feather_fill_draw_calls += 1,
            PipelineSemanticKind::FeatherStroke => {
                inventory.snapshot.feather_stroke_draw_calls += 1
            }
            PipelineSemanticKind::Draw => {
                let Some(draw_type) = semantic.draw_type else {
                    return;
                };
                match draw_type {
                    DrawType::MidpointFanPatches | DrawType::MidpointFanCenterAAPatches => {
                        inventory.snapshot.midpoint_fan_draw_calls += 1
                    }
                    DrawType::OuterCurvePatches | DrawType::MsaaOuterCubics => {
                        inventory.snapshot.outer_curve_draw_calls += 1
                    }
                    DrawType::InteriorTriangulation => {
                        inventory.snapshot.interior_triangulation_draw_calls += 1
                    }
                    DrawType::ImageRect => inventory.snapshot.image_rect_draw_calls += 1,
                    DrawType::ImageMesh => inventory.snapshot.image_mesh_draw_calls += 1,
                    DrawType::ClipReset | DrawType::MsaaMidpointFanStencilReset => {
                        inventory.snapshot.clip_reset_draw_calls += 1
                    }
                    DrawType::RenderPassInitialize => {
                        inventory.snapshot.render_pass_initialize_draw_calls += 1
                    }
                    DrawType::RenderPassResolve => {
                        inventory.snapshot.render_pass_resolve_draw_calls += 1
                    }
                    DrawType::FeatherAtlasBlit
                    | DrawType::MsaaStrokes
                    | DrawType::MsaaMidpointFanBorrowedCoverage
                    | DrawType::MsaaMidpointFans
                    | DrawType::MsaaDynamicMidpointFans
                    | DrawType::MsaaMidpointFanPathsStencil
                    | DrawType::MsaaMidpointFanPathsCover => {}
                }

                const ENABLE_CLIPPING: u32 = 1 << 0;
                const ENABLE_CLIP_RECT: u32 = 1 << 1;
                const ENABLE_ADVANCED_BLEND: u32 = 1 << 2;
                const ENABLE_HSL_BLEND_MODES: u32 = 1 << 6;
                const FIXED_FUNCTION_COLOR_OUTPUT: u32 = 1 << 0;
                if semantic.features.0 & ENABLE_CLIPPING != 0 {
                    inventory.snapshot.clip_feature_draw_calls += 1;
                }
                if semantic.features.0 & ENABLE_CLIP_RECT != 0 {
                    inventory.snapshot.clip_rect_feature_draw_calls += 1;
                }
                if semantic.features.0 & ENABLE_ADVANCED_BLEND != 0 {
                    inventory.snapshot.advanced_blend_draw_calls += 1;
                }
                if semantic.features.0 & ENABLE_HSL_BLEND_MODES != 0 {
                    inventory.snapshot.hsl_blend_draw_calls += 1;
                }
                if semantic.misc.0 & FIXED_FUNCTION_COLOR_OUTPUT != 0 {
                    inventory.snapshot.fixed_function_draw_calls += 1;
                }

                let is_atomic_content_submission = semantic.interlock
                    == Some(InterlockMode::Atomics)
                    && !matches!(
                        draw_type,
                        DrawType::RenderPassInitialize | DrawType::RenderPassResolve
                    );
                if is_atomic_content_submission {
                    inventory.snapshot.atomic_draw_calls += 1;
                    inventory.snapshot.atomic_draw_instances += instances;
                    if semantic.misc.0 & FIXED_FUNCTION_COLOR_OUTPUT == 0 {
                        inventory.snapshot.atomic_color_plane_draw_calls += 1;
                    }
                    if !inventory.saw_atomic_draw || inventory.atomic_group_boundary {
                        inventory.snapshot.draw_groups += 1;
                    }
                    inventory.saw_atomic_draw = true;
                    inventory.atomic_group_boundary = false;
                }
            }
        }
    }

    pub(crate) fn insert_command_queue(
        &mut self,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Handle {
        self.insert(queue, MetalObjectKind::CommandQueue)
    }

    pub(crate) fn insert_buffer(
        &mut self,
        buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    ) -> Handle {
        self.insert(buffer, MetalObjectKind::Buffer)
    }

    pub(crate) fn insert_texture(
        &mut self,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
    ) -> Handle {
        self.insert(texture, MetalObjectKind::Texture)
    }

    fn duplicate(&mut self, handle: Handle, expected: MetalObjectKind) -> Option<Handle> {
        let (pipeline_semantic, encoder_pipeline) = {
            let source = self.entry(handle, expected)?;
            (source.pipeline_semantic, source.encoder_pipeline)
        };
        let object = self.cloned_object(handle, expected)?;
        let duplicate = self.insert(object, expected);
        let entry = self
            .entry_mut(duplicate, expected)
            .expect("the just-inserted duplicate must remain live");
        // These tags describe the native object's authored pipeline identity,
        // not the generation-safe registry alias. A source strong local keeps
        // that exact identity when ARC creates its independent +1.
        entry.pipeline_semantic = pipeline_semantic;
        entry.encoder_pipeline = encoder_pipeline;
        Some(duplicate)
    }

    /// Creates a second independently retired registry retain for one native
    /// texture. Shared source owners must each receive their own duplicate
    /// rather than copying a Handle value.
    pub(crate) fn duplicate_texture(&mut self, texture: Handle) -> Option<Handle> {
        self.duplicate(texture, MetalObjectKind::Texture)
    }

    pub(crate) fn duplicate_buffer(&mut self, buffer: Handle) -> Option<Handle> {
        self.duplicate(buffer, MetalObjectKind::Buffer)
    }

    pub(crate) fn duplicate_command_queue(&mut self, queue: Handle) -> Option<Handle> {
        self.duplicate(queue, MetalObjectKind::CommandQueue)
    }

    /// Releases the registry's +1 texture retain. NIL, stale-generation,
    /// wrong-kind, and already-retired handles are exact no-ops.
    pub(crate) fn retire_texture(&mut self, texture: Handle) -> bool {
        self.drain_retirements();
        self.retire(texture, MetalObjectKind::Texture)
    }

    /// Releases the registry's +1 buffer retain. NIL, stale-generation,
    /// wrong-kind, and already-retired handles are exact no-ops.
    pub(crate) fn retire_buffer(&mut self, buffer: Handle) -> bool {
        self.drain_retirements();
        self.retire(buffer, MetalObjectKind::Buffer)
    }

    /// Releases the registry's +1 queue retain. The device's separate queue
    /// owner remains governed by the product context.
    pub(crate) fn retire_command_queue(&mut self, queue: Handle) -> bool {
        self.drain_retirements();
        self.retire(queue, MetalObjectKind::CommandQueue)
    }

    /// Publishes a replacement before retiring the old texture generation, so
    /// callers never observe a gap even when both handles retain the same Metal
    /// object.
    pub(crate) fn replace_texture(
        &mut self,
        current: Handle,
        replacement: Retained<ProtocolObject<dyn MTLTexture>>,
    ) -> Handle {
        let replacement = self.insert_texture(replacement);
        self.retire_texture(current);
        replacement
    }

    /// Applies the exact nullable queue transition and retires the prior
    /// registry generation. A stale prior handle fails closed and cannot retire
    /// a newer queue that reused its slot.
    pub(crate) fn replace_command_queue(
        &mut self,
        current: Handle,
        replacement: Option<Retained<ProtocolObject<dyn MTLCommandQueue>>>,
    ) -> Handle {
        let replacement = replacement
            .map(|queue| self.insert_command_queue(queue))
            .unwrap_or(Handle::NIL);
        self.retire_command_queue(current);
        replacement
    }

    /// Installs the native completion block while preserving the terminal Metal
    /// status and error. Returns false for NIL/stale/wrong-kind handles, in
    /// which case no callback is installed.
    pub(crate) fn add_command_buffer_completion_handler(
        &mut self,
        command_buffer: Handle,
        handler: Box<dyn FnOnce(NativeMetalCommandBufferCompletion) + Send + 'static>,
    ) -> bool {
        self.drain_retirements();
        let Some(command) = self.object(command_buffer, MetalObjectKind::CommandBuffer) else {
            return false;
        };
        let handler = std::sync::Mutex::new(Some(handler));
        let block = RcBlock::new(
            move |buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
                let buffer = unsafe { buffer.as_ref() };
                let completion = NativeMetalCommandBufferCompletion {
                    status: buffer.status(),
                    error: buffer.error().map(|error| format!("{error:?}")),
                };
                if let Some(handler) = handler
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .take()
                {
                    handler(completion);
                }
            },
        );
        unsafe {
            let command =
                &*(command as *const AnyObject as *const ProtocolObject<dyn MTLCommandBuffer>);
            command.addCompletedHandler(RcBlock::as_ptr(&block));
        }
        true
    }

    /// Retains the exact command object for an adapter-side terminal wait.
    /// This does not replace the source's opaque `__bridge_retained` owner;
    /// it is the Rust unwind guard that keeps `waitUntilCompleted` callable
    /// after the source commit consumes that opaque owner.
    pub(crate) fn retained_command_buffer(
        &self,
        command_buffer: Handle,
    ) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        let command = self.object(command_buffer, MetalObjectKind::CommandBuffer)?;
        unsafe {
            Retained::retain(
                (command as *const AnyObject as *mut ProtocolObject<dyn MTLCommandBuffer>)
                    .cast(),
            )
        }
    }

    fn handle_arg(args: &[Value], index: usize) -> Option<Handle> {
        match args.get(index) {
            Some(Value::Handle(value)) => Some(*value),
            Some(Value::Nil) | None => None,
            _ => None,
        }
    }

    fn u64_arg(args: &[Value], index: usize) -> Option<u64> {
        match args.get(index)? {
            Value::U64(value) => Some(*value),
            Value::I64(value) => Some(*value as u64),
            Value::Text(value) => source_integer(value),
            _ => None,
        }
    }

    fn f64_arg(args: &[Value], index: usize) -> Option<f64> {
        match args.get(index)? {
            Value::F64(value) => Some(*value),
            Value::U64(value) => Some(*value as f64),
            Value::I64(value) => Some(*value as f64),
            _ => None,
        }
    }

    fn bool_arg(args: &[Value], index: usize) -> Option<bool> {
        match args.get(index)? {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn bytes_arg(args: &[Value], index: usize) -> Option<&[u8]> {
        match args.get(index)? {
            Value::Bytes(value) => Some(value),
            _ => None,
        }
    }

    fn origin_arg(
        args: &[Value],
        index: usize,
    ) -> Option<crate::mechanical_metal_implementation::source_execution::Origin> {
        match args.get(index)? {
            Value::Origin(value) => Some(*value),
            _ => None,
        }
    }

    fn size_arg(
        args: &[Value],
        index: usize,
    ) -> Option<crate::mechanical_metal_implementation::source_execution::Size> {
        match args.get(index)? {
            Value::Size(value) => Some(*value),
            _ => None,
        }
    }

    fn viewport_arg(
        args: &[Value],
        index: usize,
    ) -> Option<crate::mechanical_metal_implementation::source_execution::Viewport> {
        match args.get(index)? {
            Value::Viewport(value) => Some(*value),
            _ => None,
        }
    }

    fn scissor_arg(
        args: &[Value],
        index: usize,
    ) -> Option<crate::mechanical_metal_implementation::source_execution::Scissor> {
        match args.get(index)? {
            Value::Scissor(value) => Some(*value),
            _ => None,
        }
    }

    fn clear_color_arg(
        args: &[Value],
        index: usize,
    ) -> Option<crate::mechanical_metal_implementation::source_execution::ClearColor> {
        match args.get(index)? {
            Value::ClearColor(value) => Some(*value),
            _ => None,
        }
    }

    fn text_arg(args: &[Value], index: usize) -> Option<&str> {
        match args.get(index)? {
            Value::Text(value) => Some(value),
            _ => None,
        }
    }

    fn new_buffer(&mut self, args: &[Value]) -> Option<Handle> {
        let device = self.device_for_handle(Self::handle_arg(args, 0)?)?;
        let length = Self::u64_arg(args, 1)? as usize;
        let options = resource_options(Self::u64_arg(args, 2)?);
        device
            .newBufferWithLength_options(length, options)
            .map(|buffer| self.insert(buffer, MetalObjectKind::Buffer))
    }

    fn new_static_buffer(&mut self, args: &[Value]) -> Option<Handle> {
        let device = self.device_for_handle(Self::handle_arg(args, 0)?)?;
        let Value::Bytes(bytes) = args.get(1)? else {
            return None;
        };
        let length = Self::u64_arg(args, 2)? as usize;
        if length != bytes.len() {
            return None;
        }
        let pointer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>())?;
        let options = resource_options(Self::u64_arg(args, 3)?);
        unsafe {
            device
                .newBufferWithBytes_length_options(pointer, length, options)
        }
        .map(|buffer| self.insert(buffer, MetalObjectKind::Buffer))
    }

    fn new_texture(&mut self, args: &[Value]) -> Option<Handle> {
        let device = self.device_for_handle(Self::handle_arg(args, 0)?)?;
        let descriptor_handle = Self::handle_arg(args, 1)?;
        let texture: Option<Retained<AnyObject>> = {
            let descriptor = self.object(descriptor_handle, MetalObjectKind::TextureDescriptor)?;
            unsafe { msg_send![device, newTextureWithDescriptor: descriptor] }
        };
        texture.map(|texture| self.insert(texture, MetalObjectKind::Texture))
    }
}

impl MetalExecution for Objc2MetalExecution {
    fn device_handle(&self) -> Handle {
        self.device_handle
    }

    fn device_supports_family(&mut self, device: Handle, family: u64) -> bool {
        self.device_for_handle(device)
            .is_some_and(|device| device.supportsFamily(MTLGPUFamily(family as _)))
    }

    fn device_raster_order_groups_supported(&mut self, device: Handle) -> bool {
        self.device_for_handle(device)
            .is_some_and(MTLDevice::areRasterOrderGroupsSupported)
    }

    fn device_is_apple_silicon(&mut self, device: Handle) -> bool {
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "visionos"))]
        {
            let Some(device) = self.device_for_handle(device) else { return false; };
            if objc2::available!(ios = 13.0, tvos = 13.0, visionos = 1.0) {
                return device.supportsFamily(MTLGPUFamily::Apple4);
            }
        }
        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "visionos")))]
        let _ = device;
        false
    }

    fn host_architecture_is_arm64(&mut self) -> bool {
        #[cfg(all(target_vendor = "apple", target_os = "ios", target_abi = "sim"))]
        {
            // The source compares the host architecture name prefix, so
            // arm64e follows the same path as arm64.
            let info = unsafe { NXGetLocalArchInfo() };
            return !info.is_null()
                && unsafe {
                    (*info)
                        ._name
                        .as_ref()
                        .is_some_and(|_| {
                            std::ffi::CStr::from_ptr((*info)._name)
                                .to_bytes()
                                .starts_with(b"arm64")
                        })
                };
        }
        #[cfg(not(all(target_vendor = "apple", target_os = "ios", target_abi = "sim")))]
        {
            let _ = self;
            false
        }
    }

    fn memory_barrier_available(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        {
            objc2::available!(macos = 10.14)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn texture_compatible(
        &mut self,
        texture: Handle,
        width: u32,
        height: u32,
        format: PixelFormat,
    ) -> bool {
        let Some(texture) = self.retained_texture(texture) else {
            return false;
        };
        let expected_format = match format {
            PixelFormat::RGBA8Unorm => MTLPixelFormat::RGBA8Unorm,
            PixelFormat::RGBA8UnormSrgb => MTLPixelFormat::RGBA8Unorm_sRGB,
            PixelFormat::BGRA8Unorm => MTLPixelFormat::BGRA8Unorm,
            PixelFormat::BGRA8UnormSrgb => MTLPixelFormat::BGRA8Unorm_sRGB,
            PixelFormat::R32Uint => MTLPixelFormat::R32Uint,
            PixelFormat::RGBA16Float => MTLPixelFormat::RGBA16Float,
            _ => return false,
        };
        // This is the literal source predicate: usage is an authored
        // debug-only assertion, while release compares only the inherited
        // dimensions and pixel format. Stronger admission belongs to the
        // product target adapter before it enters this source seam.
        debug_assert!(texture.usage().contains(MTLTextureUsage::RenderTarget));
        texture.width() == width as usize
            && texture.height() == height as usize
            && texture.pixelFormat() == expected_format
    }

    fn take_owned(
        &mut self,
        handle: Handle,
        kind: MetalObjectKind,
    ) -> Option<OwnedMetalHandle> {
        self.drain_retirements();
        let (object, validity) = self.take_owned_object(handle, kind)?;
        Some(unsafe { OwnedMetalHandle::native(handle, object, validity) })
    }

    fn clone_owned(
        &mut self,
        handle: Handle,
        kind: MetalObjectKind,
    ) -> Option<OwnedMetalHandle> {
        let duplicate = self.duplicate(handle, kind)?;
        self.take_owned(duplicate, kind)
    }

    fn make_function_name(&mut self, name: &str) -> Option<OwnedMetalHandle> {
        // GetPrecompiledFunctionName returns the native source NSString before
        // DrawPipeline construction. Preserve that identity by using the
        // Objective-C stringWithUTF8String boundary, rather than rebuilding a
        // Rust String through NSString::from_str inside the callee.
        let name = CString::new(name).ok()?;
        let name = NonNull::new(name.as_ptr().cast_mut())?;
        let native = unsafe { NSString::stringWithUTF8String(name) }?;
        let handle = self.insert(native, MetalObjectKind::NSString);
        self.take_owned(handle, MetalObjectKind::NSString)
    }

    fn make_precompiled_function_name(
        &mut self,
        prefix: u8,
        namespace_id: &str,
        function_base_name: &str,
    ) -> Option<Handle> {
        // Keep the pinned producer boundary: GetPrecompiledFunctionName uses
        // NSString stringWithFormat:@"%c%s::%s" and passes that same object
        // through both newFunctionWithName: calls. objc2 intentionally does
        // not expose variadic methods in msg_send!, so call the known
        // Objective-C ABI directly with the three authored arguments.
        let namespace_id = CString::new(namespace_id).ok()?;
        let function_base_name = CString::new(function_base_name).ok()?;
        let selector = Sel::register(c"stringWithFormat:");
        let class = core::ptr::from_ref(NSString::class()).cast::<AnyObject>();
        type StringWithFormat = unsafe extern "C" fn(
            *const AnyObject,
            Sel,
            *const NSString,
            ...,
        ) -> *mut NSString;
        let string_with_format: StringWithFormat = unsafe {
            core::mem::transmute(objc2::ffi::objc_msgSend as *const ())
        };
        let native = unsafe {
            string_with_format(
                class,
                selector,
                core::ptr::from_ref(objc2_foundation::ns_string!("%c%s::%s")),
                prefix as std::ffi::c_int,
                namespace_id.as_ptr(),
                function_base_name.as_ptr(),
            )
        };
        let native = NonNull::new(native)?;
        // stringWithFormat: is an autoreleased +0 source local. Publish only
        // a generation-checked nonowning alias for the synchronous
        // DrawPipeline selector scope; do not synthesize an extra retain.
        Some(self.insert_borrowed_object(
            native.cast(),
            MetalObjectKind::NSString,
            MetalAliasValidity::live(),
        ))
    }

    fn publish_owned(&mut self, owner: &mut OwnedMetalHandle) -> Option<Handle> {
        if let Some(validity) = owner.alias_validity() {
            let handle = owner.handle();
            let already_published_here =
                self.entry(handle, owner.kind())
                    .is_some_and(|entry| match &entry.object {
                        RetainedMetalObject::BorrowedObjectiveC {
                            validity: registered,
                            ..
                        } => registered.ptr_eq(validity),
                        _ => false,
                    });
            if already_published_here {
                return Some(handle);
            }

            // A canonical owner can outlive (or be handed away from) its
            // previous executor. Invalidate that foreign alias before
            // publishing the same +1 into this execution domain. The former
            // registry then fails closed and reaps its nonowning slot.
            owner.clear_alias_for_republication();
        }

        let object = NonNull::from(owner.native_object()?);
        let validity = MetalAliasValidity::live();
        let handle = self.insert_borrowed_object(object, owner.kind(), validity.clone());
        if owner.install_alias(handle, validity) {
            Some(handle)
        } else {
            self.retire(handle, owner.kind());
            None
        }
    }

    unsafe fn adopt_compiled_library(
        &mut self,
        library: *mut crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLLibrary,
    ) -> Option<Handle> {
        // `BackgroundCompileJob::take_compiled_library_raw` transfers the
        // compiler's existing +1. Reconstitute that exact retain and publish
        // it as the registry owner; `from_raw` neither adds nor removes a
        // retain and returns `None` for nil.
        #[cfg(test)]
        let identity = library as usize;
        let library: Retained<ProtocolObject<dyn MTLLibrary>> =
            unsafe { Retained::from_raw(library.cast()) }?;
        let handle = self.insert(library, MetalObjectKind::Library);
        #[cfg(test)]
        {
            self.adopted_background_libraries.push((handle, identity));
            crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::record_compiled_library_context_phase(
                "AdoptContext",
                identity,
            );
        }
        Some(handle)
    }

    fn buffer_contents(&mut self, buffer: Handle) -> *mut u8 {
        self.drain_retirements();
        let Some(buffer) = self.object(buffer, MetalObjectKind::Buffer) else {
            return core::ptr::null_mut();
        };
        let contents: *mut c_void = unsafe { msg_send![buffer, contents] };
        contents.cast()
    }

    fn retire_handle(&mut self, handle: Handle) {
        self.drain_retirements();
        self.retire(handle, handle.kind);
    }

    fn tag_pipeline(&mut self, pipeline: Handle, semantic: PipelineSemantic) {
        self.drain_retirements();
        let newly_tagged =
            if let Some(entry) = self.entry_mut(pipeline, MetalObjectKind::RenderPipelineState) {
                let newly_tagged = entry.pipeline_semantic.is_none();
                entry.pipeline_semantic = Some(semantic);
                newly_tagged
            } else {
                false
            };
        if newly_tagged {
            self.execution_inventory.snapshot.pipeline_creations += 1;
        }
    }

    fn record_draw_semantic(&mut self, encoder: Handle, semantic: PipelineSemantic) {
        self.drain_retirements();
        if let Some(entry) = self.entry_mut(encoder, MetalObjectKind::RenderCommandEncoder) {
            entry.encoder_pipeline = Some(semantic);
        }
    }

    fn record_render_pass_break(&mut self) {
        self.execution_inventory.snapshot.render_pass_breaks += 1;
        self.execution_inventory.snapshot.semantic_atomic_barriers += 1;
        self.execution_inventory.atomic_group_boundary = true;
    }

    fn record_raster_order_group_barrier(&mut self, encoder: Handle) {
        self.drain_retirements();
        if self
            .entry(encoder, MetalObjectKind::RenderCommandEncoder)
            .is_none()
        {
            return;
        }
        self.execution_inventory
            .snapshot
            .raster_order_group_barriers += 1;
        self.execution_inventory.snapshot.semantic_atomic_barriers += 1;
        self.execution_inventory.atomic_group_boundary = true;
    }

    fn call(
        &mut self,
        receiver: &'static str,
        selector: &'static str,
        args: Vec<Value>,
    ) -> Option<Handle> {
        self.drain_retirements();
        match (receiver, selector) {
            ("dispatch", "dispatch_data_create") => {
                let Value::Bytes(bytes) = args.first()? else {
                    return None;
                };
                if bytes.is_empty() {
                    return None;
                }
                let bytes = Arc::clone(bytes);
                let pointer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>())?;
                let data = unsafe {
                    super::dispatch_data_create(pointer, bytes.len(), None, core::ptr::null_mut())
                };
                NonNull::new(data).map(|data| self.insert_dispatch(data, bytes))
            }
            ("MTLRenderPipelineDescriptor", "alloc/init") => Some(self.insert(
                MTLRenderPipelineDescriptor::new(),
                MetalObjectKind::RenderPipelineDescriptor,
            )),
            ("MTLTextureDescriptor", "alloc/init") => Some(self.insert(
                MTLTextureDescriptor::new(),
                MetalObjectKind::TextureDescriptor,
            )),
            ("MTLSamplerDescriptor", "new") => Some(self.insert(
                MTLSamplerDescriptor::new(),
                MetalObjectKind::SamplerDescriptor,
            )),
            ("MTLRenderPassDescriptor", "renderPassDescriptor") => {
                let descriptor = objc2_metal::MTLRenderPassDescriptor::renderPassDescriptor();
                Some(self.insert(descriptor, MetalObjectKind::RenderPassDescriptor))
            }
            ("gpu", "newBufferWithLength:options:") => self.new_buffer(&args),
            ("gpu", "newBufferWithBytes:length:options:") => self.new_static_buffer(&args),
            ("gpu", "newTextureWithDescriptor:") => self.new_texture(&args),
            ("gpu", "newSamplerStateWithDescriptor:") => {
                let device = self.device_for_handle(Self::handle_arg(&args, 0)?)?;
                let descriptor_handle = Self::handle_arg(&args, 1)?;
                let sampler: Option<Retained<AnyObject>> = {
                    let descriptor =
                        self.object(descriptor_handle, MetalObjectKind::SamplerDescriptor)?;
                    unsafe { msg_send![device, newSamplerStateWithDescriptor: descriptor] }
                };
                // The source selector only borrows the descriptor.  Its
                // caller owns the lexical descriptor local and retires it at
                // that scope boundary; keeping this adapter borrow-only is
                // important for source ARC ordering and error paths.
                sampler.map(|sampler| self.insert(sampler, MetalObjectKind::SamplerState))
            }
            ("library", "newFunctionWithName:") => {
                let library = self.object(Self::handle_arg(&args, 0)?, MetalObjectKind::Library)?;
                let function: Option<Retained<AnyObject>> = match args.get(1) {
                    Some(Value::Handle(name_handle)) => {
                        let name = self.object(*name_handle, MetalObjectKind::NSString)?;
                        unsafe { msg_send![library, newFunctionWithName: name] }
                    }
                    Some(Value::Text(name)) => {
                        let name = NSString::from_str(name);
                        unsafe { msg_send![library, newFunctionWithName: &*name] }
                    }
                    Some(Value::StaticText(name)) => {
                        let name = source_static_function_name(name)?;
                        unsafe { msg_send![library, newFunctionWithName: name] }
                    }
                    _ => None,
                };
                function.map(|function| self.insert(function, MetalObjectKind::Function))
            }
            ("descriptor", "colorAttachmentAtIndex:") => {
                let descriptor_handle = Self::handle_arg(&args, 0)?;
                self.object(
                    descriptor_handle,
                    MetalObjectKind::RenderPipelineDescriptor,
                )?;
                let attachments = self.object(
                    Self::handle_arg(&args, 1)?,
                    MetalObjectKind::RenderPipelineColorAttachmentDescriptorArray,
                )?;
                let index = Self::u64_arg(&args, 2)? as usize;
                let attachment = unsafe {
                    let attachment: *mut AnyObject =
                        msg_send![attachments, objectAtIndexedSubscript: index];
                    NonNull::new(attachment)
                }?;
                self.insert_borrowed_child(
                    descriptor_handle,
                    attachment,
                    MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
                )
            }
            ("descriptor", "colorAttachments") => {
                let descriptor_handle = Self::handle_arg(&args, 0)?;
                let descriptor = self.object(
                    descriptor_handle,
                    MetalObjectKind::RenderPipelineDescriptor,
                )?;
                let attachments = unsafe {
                    let attachments: *mut AnyObject = msg_send![descriptor, colorAttachments];
                    NonNull::new(attachments)
                }?;
                self.insert_borrowed_child(
                    descriptor_handle,
                    attachments,
                    MetalObjectKind::RenderPipelineColorAttachmentDescriptorArray,
                )
            }
            ("pass", "colorAttachmentAtIndex:") => {
                let descriptor_handle = Self::handle_arg(&args, 0)?;
                self.object(descriptor_handle, MetalObjectKind::RenderPassDescriptor)?;
                let attachments = self.object(
                    Self::handle_arg(&args, 1)?,
                    MetalObjectKind::RenderPassColorAttachmentDescriptorArray,
                )?;
                let index = Self::u64_arg(&args, 2)? as usize;
                let attachment = unsafe {
                    let attachment: *mut AnyObject =
                        msg_send![attachments, objectAtIndexedSubscript: index];
                    NonNull::new(attachment)
                }?;
                self.insert_borrowed_child(
                    descriptor_handle,
                    attachment,
                    MetalObjectKind::RenderPassColorAttachmentDescriptor,
                )
            }
            ("pass", "colorAttachments") => {
                let descriptor_handle = Self::handle_arg(&args, 0)?;
                let descriptor =
                    self.object(descriptor_handle, MetalObjectKind::RenderPassDescriptor)?;
                let attachments = unsafe {
                    let attachments: *mut AnyObject = msg_send![descriptor, colorAttachments];
                    NonNull::new(attachments)
                }?;
                self.insert_borrowed_child(
                    descriptor_handle,
                    attachments,
                    MetalObjectKind::RenderPassColorAttachmentDescriptorArray,
                )
            }
            ("commandQueue", "commandBuffer (__bridge_retained)") => {
                let queue =
                    self.object(Self::handle_arg(&args, 0)?, MetalObjectKind::CommandQueue)?;
                let command: Option<Retained<AnyObject>> =
                    unsafe { msg_send![queue, commandBuffer] };
                command.map(|command| self.insert(command, MetalObjectKind::CommandBuffer))
            }
            ("commandBuffer", "renderCommandEncoderWithDescriptor:") => {
                let command =
                    self.object(Self::handle_arg(&args, 0)?, MetalObjectKind::CommandBuffer)?;
                let pass = self.object(
                    Self::handle_arg(&args, 1)?,
                    MetalObjectKind::RenderPassDescriptor,
                )?;
                let encoder: Option<Retained<AnyObject>> =
                    unsafe { msg_send![command, renderCommandEncoderWithDescriptor: pass] };
                encoder.map(|encoder| {
                    let encoder = self.insert(encoder, MetalObjectKind::RenderCommandEncoder);
                    encoder
                })
            }
            ("commandBuffer", "blitCommandEncoder") => {
                let command =
                    self.object(Self::handle_arg(&args, 0)?, MetalObjectKind::CommandBuffer)?;
                let encoder: Option<Retained<AnyObject>> =
                    unsafe { msg_send![command, blitCommandEncoder] };
                encoder.map(|encoder| {
                    let encoder = self.insert(encoder, MetalObjectKind::BlitCommandEncoder);
                    encoder
                })
            }
            _ => {
                self.dispatch_void(receiver, selector, &args);
                None
            }
        }
    }

    fn call_with_error(
        &mut self,
        receiver: &'static str,
        selector: &'static str,
        args: Vec<Value>,
    ) -> ObjectCreation {
        self.drain_retirements();
        let mut error: Option<Retained<NSError>> = None;
        let object: Option<Retained<AnyObject>> = match (receiver, selector) {
            ("gpu", "newLibraryWithData:error:") => {
                let Some(device) = self.device_for_handle(
                    Self::handle_arg(&args, 0).unwrap_or(Handle::NIL),
                ) else { return ObjectCreation::default(); };
                let Some(handle) = Self::handle_arg(&args, 1) else {
                    return ObjectCreation::default();
                };
                // `newLibraryWithData:` borrows dispatchData for the
                // selector call.  The source caller keeps its dispatchData
                // local alive through this expression and releases it at the
                // authored block boundary; do not consume it here.
                let Some(data) = self.dispatch(handle) else {
                    return ObjectCreation::default();
                };
                let library = unsafe {
                    msg_send![device,
                        newLibraryWithData: data,
                        error: &mut error
                    ]
                };
                library
            }
            ("gpu", "newRenderPipelineStateWithDescriptor:error:") => {
                let Some(device) = self.device_for_handle(
                    Self::handle_arg(&args, 0).unwrap_or(Handle::NIL),
                ) else { return ObjectCreation::default(); };
                let Some(descriptor_handle) = Self::handle_arg(&args, 1) else {
                    return ObjectCreation::default();
                };
                let descriptor = if descriptor_handle == Handle::NIL {
                    core::ptr::null::<AnyObject>()
                } else {
                    let Some(descriptor) =
                        self.object(descriptor_handle, MetalObjectKind::RenderPipelineDescriptor)
                    else {
                        return ObjectCreation::default();
                    };
                    core::ptr::from_ref(descriptor)
                };
                // The pinned Objective-C++ still sends this selector when
                // descriptor allocation returned nil. Passing the literal
                // nil object argument preserves that call/error boundary;
                // only a stale nonnil registry handle fails closed above.
                let object = unsafe {
                    msg_send![device,
                        newRenderPipelineStateWithDescriptor: descriptor,
                        error: &mut error
                    ]
                };
                object
            }
            _ => return ObjectCreation::default(),
        };
        let kind = if selector == "newLibraryWithData:error:" {
            MetalObjectKind::Library
        } else {
            MetalObjectKind::RenderPipelineState
        };
        let error_present = error.is_some();
        ObjectCreation {
            object: object.map(|object| self.insert(object, kind)),
            error_owner_handle: None,
            // Keep NSError opaque and query localizedDescription only from
            // the source caller's authored logging branch.
            error: None,
            error_present,
            #[cfg(target_vendor = "apple")]
            error_owner: error,
        }
    }

    fn add_completed_handler(
        &mut self,
        command_buffer: Handle,
        handler: Box<dyn FnOnce(Result<(), String>) + Send + 'static>,
    ) -> bool {
        self.add_command_buffer_completion_handler(
            command_buffer,
            Box::new(move |completion| handler(completion.into_result())),
        )
    }
}

impl Objc2MetalExecution {
    pub(crate) fn take_ore_context_owner(
        &mut self,
        handle: Handle,
    ) -> Option<Box<dyn std::any::Any>> {
        let mut object =
            core::mem::ManuallyDrop::new(self.take_owner(handle, MetalObjectKind::OreContext)?);
        if let RetainedMetalObject::Host(owner) = &mut *object {
            // Move the host payload out without running the enum destructor
            // on the moved field. The non-host branch is explicitly dropped
            // below so Objective-C/dispatch ownership is not leaked.
            return Some(unsafe { core::ptr::read(owner) });
        }
        unsafe { core::mem::ManuallyDrop::drop(&mut object) };
        None
    }

    pub(crate) fn retained_texture(
        &self,
        handle: Handle,
    ) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        let object = self.cloned_object(handle, MetalObjectKind::Texture)?;
        // SAFETY: the registry kind check above proves this retained Objective-C
        // object was inserted from an MTLTexture allocation.
        Some(unsafe { Retained::cast_unchecked::<ProtocolObject<dyn MTLTexture>>(object) })
    }
}

impl HostExecution for Objc2MetalExecution {
    fn log(&mut self, message: String) {
        self.host.log(message);
    }

    fn generate_patch_buffer_data(&mut self, vertex_buffer: Handle, index_buffer: Handle) {
        let Some(vertex_buffer) = self.cloned_object(vertex_buffer, MetalObjectKind::Buffer) else {
            return;
        };
        let Some(index_buffer) = self.cloned_object(index_buffer, MetalObjectKind::Buffer) else {
            return;
        };
        unsafe {
            let vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>> =
                Retained::cast_unchecked(vertex_buffer);
            let index_buffer: Retained<ProtocolObject<dyn MTLBuffer>> =
                Retained::cast_unchecked(index_buffer);
            self.host
                .generate_patch_buffer_data(&vertex_buffer, &index_buffer);
        }
    }

    fn make_ore_context(&mut self, device: Handle, queue: Option<Handle>) -> Option<Handle> {
        let device = self.cloned_object(device, MetalObjectKind::Device)?;
        let queue = match queue {
            Some(queue) => Some(self.cloned_object(queue, MetalObjectKind::CommandQueue)?),
            None => None,
        };
        let owner = unsafe {
            let device: Retained<ProtocolObject<dyn MTLDevice>> = Retained::cast_unchecked(device);
            let queue = queue.map(|queue| Retained::cast_unchecked(queue));
            self.host.make_ore_context(&device, queue.as_deref())
        }?;
        Some(self.insert_host(owner, MetalObjectKind::OreContext))
    }

}

impl Objc2MetalExecution {
    fn dispatch_void(&mut self, receiver: &str, selector: &str, args: &[Value]) {
        macro_rules! some {
            ($value:expr) => {
                match $value {
                    Some(value) => value,
                    None => return,
                }
            };
        }
        let handle = Self::handle_arg(args, 0).unwrap_or(Handle::NIL);
        match (receiver, selector) {
            ("commandBuffer", "commit (__bridge_transfer)") => {
                if let Some(object) = self.take_owner(handle, MetalObjectKind::CommandBuffer) {
                    let RetainedMetalObject::ObjectiveC(command) = &object else { return; };
                    unsafe {
                        let _: () = msg_send![&*command, commit];
                    }
                }
            }
            (_, "endEncoding") => {
                let expected = match handle.kind {
                    MetalObjectKind::RenderCommandEncoder => {
                        MetalObjectKind::RenderCommandEncoder
                    }
                    MetalObjectKind::BlitCommandEncoder => MetalObjectKind::BlitCommandEncoder,
                    _ => return,
                };
                let Some(object) = self.object(handle, expected) else {
                    return;
                };
                unsafe { let _: () = msg_send![object, endEncoding]; }
            }
            ("textureDescriptor", selector)
                if matches!(
                    selector,
                    "setPixelFormat:"
                        | "setTextureType:"
                        | "setWidth:"
                        | "setHeight:"
                        | "setMipmapLevelCount:"
                        | "setArrayLength:"
                        | "setUsage:"
                        | "setStorageMode:"
                ) =>
            {
                let descriptor = Self::handle_arg(args, 0)
                    .and_then(|handle| self.object(handle, MetalObjectKind::TextureDescriptor));
                let descriptor = some!(descriptor);
                let value = some!(Self::u64_arg(args, 1));
                unsafe {
                    match selector {
                        "setPixelFormat:" => { let _: () = msg_send![descriptor, setPixelFormat: value]; }
                        "setTextureType:" => { let _: () = msg_send![descriptor, setTextureType: value]; }
                        "setWidth:" => { let _: () = msg_send![descriptor, setWidth: value as usize]; }
                        "setHeight:" => { let _: () = msg_send![descriptor, setHeight: value as usize]; }
                        "setMipmapLevelCount:" => { let _: () = msg_send![descriptor, setMipmapLevelCount: value as usize]; }
                        "setArrayLength:" => { let _: () = msg_send![descriptor, setArrayLength: value as usize]; }
                        "setUsage:" => { let _: () = msg_send![descriptor, setUsage: value]; }
                        "setStorageMode:" => { let _: () = msg_send![descriptor, setStorageMode: value]; }
                        _ => {}
                    }
                }
            }
            ("descriptor", "setVertexFunction:") => {
                let descriptor = self.object(
                    some!(Self::handle_arg(args, 0)),
                    MetalObjectKind::RenderPipelineDescriptor,
                );
                let descriptor = some!(descriptor);
                let function = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Function));
                unsafe { let _: () = msg_send![descriptor, setVertexFunction: function]; }
            }
            ("descriptor", "setFragmentFunction:") => {
                let descriptor = self.object(
                    some!(Self::handle_arg(args, 0)),
                    MetalObjectKind::RenderPipelineDescriptor,
                );
                let descriptor = some!(descriptor);
                let function = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Function));
                unsafe { let _: () = msg_send![descriptor, setFragmentFunction: function]; }
            }
            ("colorAttachments[0]" | "framebuffer", "setPixelFormat:") => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(
                        handle,
                        MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
                    )
                });
                let attachment = some!(attachment);
                let pixel_format = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setPixelFormat: pixel_format]; }
            }
            ("clipAttachment", "setPixelFormat:") => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPipelineColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let pixel_format = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setPixelFormat: pixel_format]; }
            }
            ("scratchAttachment", "setPixelFormat:") => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPipelineColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let pixel_format = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setPixelFormat: pixel_format]; }
            }
            ("coverageAttachment", "setPixelFormat:") => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPipelineColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let pixel_format = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setPixelFormat: pixel_format]; }
            }
            ("colorAttachments[0]" | "framebuffer", "setBlendingEnabled:") => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPipelineColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let enabled = some!(Self::bool_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setBlendingEnabled: enabled]; }
            }
            ("samplerDescriptor", selector)
                if matches!(
                    selector,
                    "setMinFilter:"
                        | "setMagFilter:"
                        | "setMipFilter:"
                        | "setSAddressMode:"
                        | "setTAddressMode:"
                ) =>
            {
                let descriptor = Self::handle_arg(args, 0)
                    .and_then(|handle| self.object(handle, MetalObjectKind::SamplerDescriptor));
                let descriptor = some!(descriptor);
                let value = some!(Self::u64_arg(args, 1));
                unsafe {
                    match selector {
                        "setMinFilter:" => { let _: () = msg_send![descriptor, setMinFilter: value]; }
                        "setMagFilter:" => { let _: () = msg_send![descriptor, setMagFilter: value]; }
                        "setMipFilter:" => { let _: () = msg_send![descriptor, setMipFilter: value]; }
                        "setSAddressMode:" => { let _: () = msg_send![descriptor, setSAddressMode: value]; }
                        "setTAddressMode:" => { let _: () = msg_send![descriptor, setTAddressMode: value]; }
                        _ => {}
                    }
                }
            }
            ("framebuffer", selector)
                if matches!(
                    selector,
                    "setSourceRGBBlendFactor:"
                        | "setDestinationRGBBlendFactor:"
                        | "setRgbBlendOperation:"
                        | "setSourceAlphaBlendFactor:"
                        | "setDestinationAlphaBlendFactor:"
                        | "setAlphaBlendOperation:"
                        | "setWriteMask:"
                ) =>
            {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPipelineColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let value = some!(Self::u64_arg(args, 1));
                unsafe {
                    match selector {
                        "setSourceRGBBlendFactor:" => { let _: () = msg_send![attachment, setSourceRGBBlendFactor: value]; }
                        "setDestinationRGBBlendFactor:" => { let _: () = msg_send![attachment, setDestinationRGBBlendFactor: value]; }
                        "setRgbBlendOperation:" => { let _: () = msg_send![attachment, setRgbBlendOperation: value]; }
                        "setSourceAlphaBlendFactor:" => { let _: () = msg_send![attachment, setSourceAlphaBlendFactor: value]; }
                        "setDestinationAlphaBlendFactor:" => { let _: () = msg_send![attachment, setDestinationAlphaBlendFactor: value]; }
                        "setAlphaBlendOperation:" => { let _: () = msg_send![attachment, setAlphaBlendOperation: value]; }
                        "setWriteMask:" => { let _: () = msg_send![attachment, setWriteMask: value]; }
                        _ => {}
                    }
                }
            }
            (
                "pass" | "gradPass" | "tessPass" | "atlasPass",
                "setRenderTargetWidth:renderTargetHeight:",
            ) => {
                let descriptor = Self::handle_arg(args, 0)
                    .and_then(|handle| self.object(handle, MetalObjectKind::RenderPassDescriptor));
                let descriptor = some!(descriptor);
                let width = some!(Self::u64_arg(args, args.len().saturating_sub(2))) as usize;
                let height = some!(Self::u64_arg(args, args.len().saturating_sub(1))) as usize;
                unsafe {
                    let _: () = msg_send![descriptor, setRenderTargetWidth: width];
                    let _: () = msg_send![descriptor, setRenderTargetHeight: height];
                }
            }
            (
                "pass" | "gradPass" | "tessPass" | "atlasPass",
                "setRenderTargetWidth:",
            ) => {
                let descriptor = Self::handle_arg(args, 0)
                    .and_then(|handle| self.object(handle, MetalObjectKind::RenderPassDescriptor));
                let descriptor = some!(descriptor);
                let width = some!(Self::u64_arg(args, 1)) as usize;
                unsafe { let _: () = msg_send![descriptor, setRenderTargetWidth: width]; }
            }
            (
                "pass" | "gradPass" | "tessPass" | "atlasPass",
                "setRenderTargetHeight:",
            ) => {
                let descriptor = Self::handle_arg(args, 0)
                    .and_then(|handle| self.object(handle, MetalObjectKind::RenderPassDescriptor));
                let descriptor = some!(descriptor);
                let height = some!(Self::u64_arg(args, 1)) as usize;
                unsafe { let _: () = msg_send![descriptor, setRenderTargetHeight: height]; }
            }
            (receiver, "setTexture:") if is_render_pass_attachment_receiver(receiver) => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPassColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let texture = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Texture));
                unsafe { let _: () = msg_send![attachment, setTexture: texture]; }
                if texture.is_some() {
                    match receiver {
                        "colorAttachment" => {
                            self.execution_inventory.snapshot.color_attachment_binds += 1
                        }
                        "clipAttachment" => {
                            self.execution_inventory.snapshot.clip_attachment_binds += 1
                        }
                        "coverageAttachment" => {
                            self.execution_inventory.snapshot.coverage_attachment_binds += 1
                        }
                        _ => {}
                    }
                }
            }
            (receiver, "setLoadAction:") if is_render_pass_attachment_receiver(receiver) => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPassColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let action = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setLoadAction: action]; }
            }
            (receiver, "setStoreAction:") if is_render_pass_attachment_receiver(receiver) => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPassColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let action = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![attachment, setStoreAction: action]; }
            }
            (receiver, "setClearColor:") if is_render_pass_attachment_receiver(receiver) => {
                let attachment = Self::handle_arg(args, 0).and_then(|handle| {
                    self.object(handle, MetalObjectKind::RenderPassColorAttachmentDescriptor)
                });
                let attachment = some!(attachment);
                let clear = some!(Self::clear_color_arg(args, 1));
                let clear = objc2_metal::MTLClearColor {
                    red: clear.red,
                    green: clear.green,
                    blue: clear.blue,
                    alpha: clear.alpha,
                };
                unsafe { let _: () = msg_send![attachment, setClearColor: clear]; }
            }
            (
                "gaussianTexture",
                "replaceRegion:mipmapLevel:slice:withBytes:bytesPerRow:bytesPerImage:",
            ) => {
                let texture = some!(self.object(handle, MetalObjectKind::Texture));
                let origin = some!(Self::origin_arg(args, 1));
                let size = some!(Self::size_arg(args, 2));
                let level = some!(Self::u64_arg(args, 3)) as usize;
                let slice = some!(Self::u64_arg(args, 4)) as usize;
                let bytes = some!(Self::bytes_arg(args, 5));
                let bytes_per_row = some!(Self::u64_arg(args, 6)) as usize;
                let bytes_per_image = some!(Self::u64_arg(args, 7)) as usize;
                let region = objc2_metal::MTLRegion {
                    origin: objc2_metal::MTLOrigin {
                        x: origin.x,
                        y: origin.y,
                        z: origin.z,
                    },
                    size: objc2_metal::MTLSize {
                        width: size.width,
                        height: size.height,
                        depth: size.depth,
                    },
                };
                unsafe {
                    let _: () = msg_send![texture,
                        replaceRegion: region,
                        mipmapLevel: level,
                        slice: slice,
                        withBytes: bytes.as_ptr().cast::<c_void>(),
                        bytesPerRow: bytes_per_row,
                        bytesPerImage: bytes_per_image
                    ];
                }
            }
            ("texture", "replaceRegion:mipmapLevel:withBytes:bytesPerRow:") => {
                let texture = some!(self.object(handle, MetalObjectKind::Texture));
                let origin = some!(Self::origin_arg(args, 1));
                let size = some!(Self::size_arg(args, 2));
                let level = some!(Self::u64_arg(args, 3)) as usize;
                let bytes = some!(Self::bytes_arg(args, 4));
                let bytes_per_row = some!(Self::u64_arg(args, 5)) as usize;
                let region = objc2_metal::MTLRegion {
                    origin: objc2_metal::MTLOrigin {
                        x: origin.x,
                        y: origin.y,
                        z: origin.z,
                    },
                    size: objc2_metal::MTLSize {
                        width: size.width,
                        height: size.height,
                        depth: size.depth,
                    },
                };
                unsafe {
                    let _: () = msg_send![texture,
                        replaceRegion: region,
                        mipmapLevel: level,
                        withBytes: bytes.as_ptr().cast::<c_void>(),
                        bytesPerRow: bytes_per_row
                    ];
                }
            }
            ("blitEncoder", "generateMipmapsForTexture:") => {
                let encoder = some!(self.object(handle, MetalObjectKind::BlitCommandEncoder));
                let texture = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Texture));
                let Some(texture) = texture else { return };
                unsafe { let _: () = msg_send![encoder, generateMipmapsForTexture: texture]; }
            }
            (
                "copyEncoder",
                "copyFromTexture:sourceSlice:sourceLevel:sourceOrigin:sourceSize:toBuffer:destinationOffset:destinationBytesPerRow:destinationBytesPerImage:",
            ) => {
                let encoder = some!(self.object(handle, MetalObjectKind::BlitCommandEncoder));
                let texture = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Texture));
                let Some(texture) = texture else { return };
                let source_slice = some!(Self::u64_arg(args, 2)) as usize;
                let source_level = some!(Self::u64_arg(args, 3)) as usize;
                let origin = some!(Self::origin_arg(args, 4));
                let size = some!(Self::size_arg(args, 5));
                let buffer = Self::handle_arg(args, 6)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Buffer));
                let Some(buffer) = buffer else { return };
                let destination_offset = some!(Self::u64_arg(args, 7)) as usize;
                let destination_bytes_per_row = some!(Self::u64_arg(args, 8)) as usize;
                let destination_bytes_per_image = some!(Self::u64_arg(args, 9)) as usize;
                let origin = objc2_metal::MTLOrigin {
                    x: origin.x,
                    y: origin.y,
                    z: origin.z,
                };
                let size = objc2_metal::MTLSize {
                    width: size.width,
                    height: size.height,
                    depth: size.depth,
                };
                unsafe {
                    let _: () = msg_send![encoder,
                        copyFromTexture: texture,
                        sourceSlice: source_slice,
                        sourceLevel: source_level,
                        sourceOrigin: origin,
                        sourceSize: size,
                        toBuffer: buffer,
                        destinationOffset: destination_offset,
                        destinationBytesPerRow: destination_bytes_per_row,
                        destinationBytesPerImage: destination_bytes_per_image
                    ];
                }
            }
            (receiver, "setViewport:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let viewport = some!(Self::viewport_arg(args, 1));
                let viewport = objc2_metal::MTLViewport {
                    originX: viewport.origin_x,
                    originY: viewport.origin_y,
                    width: viewport.width,
                    height: viewport.height,
                    znear: viewport.znear,
                    zfar: viewport.zfar,
                };
                unsafe { let _: () = msg_send![encoder, setViewport: viewport]; }
            }
            (receiver, "setScissorRect:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let scissor = some!(Self::scissor_arg(args, 1));
                let scissor = objc2_metal::MTLScissorRect {
                    x: scissor.x,
                    y: scissor.y,
                    width: scissor.width,
                    height: scissor.height,
                };
                unsafe { let _: () = msg_send![encoder, setScissorRect: scissor]; }
            }
            (receiver, "setRenderPipelineState:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let pipeline_handle = some!(Self::handle_arg(args, 1));
                let semantic = self.pipeline_semantic(pipeline_handle);
                let pipeline = self.object(pipeline_handle, MetalObjectKind::RenderPipelineState);
                let Some(pipeline) = pipeline else { return };
                unsafe { let _: () = msg_send![encoder, setRenderPipelineState: pipeline]; }
                self.bind_pipeline_semantic(handle, semantic);
            }
            (receiver, "setVertexBuffer:offset:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let buffer = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Buffer));
                let offset = some!(Self::u64_arg(args, 2)) as usize;
                let index = some!(Self::u64_arg(args, 3)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        setVertexBuffer: buffer,
                        offset: offset,
                        atIndex: index
                    ];
                }
            }
            (receiver, "setFragmentBuffer:offset:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let buffer = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Buffer));
                let offset = some!(Self::u64_arg(args, 2)) as usize;
                let index = some!(Self::u64_arg(args, 3)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        setFragmentBuffer: buffer,
                        offset: offset,
                        atIndex: index
                    ];
                }
                if buffer.is_some() {
                    match index {
                        16 => self.execution_inventory.snapshot.color_atomic_buffer_binds += 1,
                        17 => self.execution_inventory.snapshot.clip_atomic_buffer_binds += 1,
                        19 => {
                            self.execution_inventory.snapshot.coverage_atomic_buffer_binds += 1
                        }
                        _ => {}
                    }
                }
            }
            (receiver, "setVertexTexture:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let texture = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Texture));
                let index = some!(Self::u64_arg(args, 2)) as usize;
                unsafe { let _: () = msg_send![encoder, setVertexTexture: texture, atIndex: index]; }
            }
            (receiver, "setFragmentTexture:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let texture = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Texture));
                let index = some!(Self::u64_arg(args, 2)) as usize;
                unsafe { let _: () = msg_send![encoder, setFragmentTexture: texture, atIndex: index]; }
                if texture.is_some() && index == 8 {
                    self.execution_inventory.snapshot.gradient_texture_binds += 1;
                } else if texture.is_some() && index == 11 {
                    self.execution_inventory.snapshot.image_texture_binds += 1;
                }
            }
            (receiver, "setFragmentSamplerState:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let sampler = Self::handle_arg(args, 1)
                    .and_then(|handle| self.object(handle, MetalObjectKind::SamplerState));
                let index = some!(Self::u64_arg(args, 2)) as usize;
                unsafe { let _: () = msg_send![encoder, setFragmentSamplerState: sampler, atIndex: index]; }
            }
            (receiver, "setCullMode:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let mode = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![encoder, setCullMode: mode]; }
            }
            (receiver, "setTriangleFillMode:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let mode = some!(Self::u64_arg(args, 1));
                unsafe { let _: () = msg_send![encoder, setTriangleFillMode: mode]; }
            }
            (receiver, "setVertexBytes:length:atIndex:") if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let bytes = some!(Self::bytes_arg(args, 1));
                let length = some!(Self::u64_arg(args, 2)) as usize;
                let index = some!(Self::u64_arg(args, 3)) as usize;
                if length != bytes.len() {
                    return;
                }
                unsafe {
                    let _: () = msg_send![encoder,
                        setVertexBytes: bytes.as_ptr().cast::<c_void>(),
                        length: length,
                        atIndex: index
                    ];
                }
            }
            (receiver, "memoryBarrierWithScope:afterStages:beforeStages:")
                if is_render_encoder(receiver) =>
            {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let scope = some!(Self::u64_arg(args, 1));
                let after = some!(Self::u64_arg(args, 2));
                let before = some!(Self::u64_arg(args, 3));
                unsafe {
                    let _: () = msg_send![encoder,
                        memoryBarrierWithScope: scope,
                        afterStages: after,
                        beforeStages: before
                    ];
                }
                self.execution_inventory.snapshot.memory_barriers += 1;
                self.execution_inventory
                    .snapshot
                    .semantic_atomic_barriers += 1;
                self.execution_inventory.atomic_group_boundary = true;
            }
            (receiver, "drawPrimitives:vertexStart:vertexCount:")
                if is_render_encoder(receiver) =>
            {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let primitive = some!(Self::u64_arg(args, 1));
                let start = some!(Self::u64_arg(args, 2)) as usize;
                let count = some!(Self::u64_arg(args, 3)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        drawPrimitives: primitive,
                        vertexStart: start,
                        vertexCount: count
                    ];
                }
                self.record_draw_submission(handle, 1);
            }
            (receiver, "drawPrimitives:vertexStart:vertexCount:instanceCount:")
                if is_render_encoder(receiver) =>
            {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let primitive = some!(Self::u64_arg(args, 1));
                let start = some!(Self::u64_arg(args, 2)) as usize;
                let count = some!(Self::u64_arg(args, 3)) as usize;
                let instances = some!(Self::u64_arg(args, 4)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        drawPrimitives: primitive,
                        vertexStart: start,
                        vertexCount: count,
                        instanceCount: instances
                    ];
                }
                self.record_draw_submission(handle, instances);
            }
            (
                receiver,
                "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:",
            ) if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let primitive = some!(Self::u64_arg(args, 1));
                let count = some!(Self::u64_arg(args, 2)) as usize;
                let index_type = some!(Self::u64_arg(args, 3));
                let buffer = Self::handle_arg(args, 4)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Buffer));
                let Some(buffer) = buffer else { return };
                let offset = some!(Self::u64_arg(args, 5)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        drawIndexedPrimitives: primitive,
                        indexCount: count,
                        indexType: index_type,
                        indexBuffer: buffer,
                        indexBufferOffset: offset
                    ];
                }
                self.record_draw_submission(handle, 1);
            }
            (
                receiver,
                "drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:",
            ) if is_render_encoder(receiver) => {
                let encoder = some!(self.object(handle, MetalObjectKind::RenderCommandEncoder));
                let primitive = some!(Self::u64_arg(args, 1));
                let count = some!(Self::u64_arg(args, 2)) as usize;
                let index_type = some!(Self::u64_arg(args, 3));
                let buffer = Self::handle_arg(args, 4)
                    .and_then(|handle| self.object(handle, MetalObjectKind::Buffer));
                let Some(buffer) = buffer else { return };
                let offset = some!(Self::u64_arg(args, 5)) as usize;
                let instances = some!(Self::u64_arg(args, 6)) as usize;
                unsafe {
                    let _: () = msg_send![encoder,
                        drawIndexedPrimitives: primitive,
                        indexCount: count,
                        indexType: index_type,
                        indexBuffer: buffer,
                        indexBufferOffset: offset,
                        instanceCount: instances
                    ];
                }
                self.record_draw_submission(handle, instances);
            }
            _ => debug_assert!(
                false,
                "unsupported production Metal selector {receiver} {selector}"
            ),
        }
    }
}

fn is_render_encoder(receiver: &str) -> bool {
    matches!(
        receiver,
        "encoder" | "gradEncoder" | "tessEncoder" | "atlasEncoder"
    )
}

fn is_render_pass_attachment_receiver(receiver: &str) -> bool {
    matches!(
        receiver,
        "colorAttachment"
            | "clipAttachment"
            | "scratchAttachment"
            | "coverageAttachment"
            | "gradAttachment"
            | "tessAttachment"
            | "atlasAttachment"
    )
}

fn resource_options(storage_mode: u64) -> MTLResourceOptions {
    match storage_mode {
        0 => MTLResourceOptions::StorageModeShared,
        1 => MTLResourceOptions::StorageModeManaged,
        2 => MTLResourceOptions::StorageModePrivate,
        3 => MTLResourceOptions::StorageModeMemoryless,
        _ => MTLResourceOptions::StorageModeShared,
    }
}

fn source_integer(source: &str) -> Option<u64> {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
    match source {
        "GAUSSIAN_TABLE_SIZE" => Some(gpu::GAUSSIAN_TABLE_SIZE as u64),
        "kGradTextureWidth" => Some(gpu::kGradTextureWidth as u64),
        "kTessTextureWidth" => Some(gpu::kTessTextureWidth as u64),
        "GRAD_SPAN_TRI_STRIP_VERTEX_COUNT" => Some(gpu::GRAD_SPAN_TRI_STRIP_VERTEX_COUNT as u64),
        "std::size(kTessSpanIndices)" => Some(gpu::kTessSpanIndices.len() as u64),
        "kMidpointFanCenterAAPatchIndexCount" => {
            Some(gpu::kMidpointFanCenterAAPatchIndexCount as u64)
        }
        "kMidpointFanPatchBorderIndexCount" => Some(gpu::kMidpointFanPatchBorderIndexCount as u64),
        "kMidpointFanCenterAAPatchBaseIndex*sizeof(uint16_t)" => Some(
            gpu::kMidpointFanCenterAAPatchBaseIndex as u64 * core::mem::size_of::<u16>() as u64,
        ),
        "kMidpointFanPatchBaseIndex*sizeof(uint16_t)" => {
            Some(gpu::kMidpointFanPatchBaseIndex as u64 * core::mem::size_of::<u16>() as u64)
        }
        "FLUSH_UNIFORM_BUFFER_IDX" => Some(3),
        "PATH_BASE_INSTANCE_UNIFORM_BUFFER_IDX" => Some(4),
        "PATH_BUFFER_IDX" => Some(5),
        "PAINT_BUFFER_IDX" => Some(6),
        "PAINT_AUX_BUFFER_IDX" => Some(7),
        "CONTOUR_BUFFER_IDX" => Some(8),
        "TESS_VERTEX_TEXTURE_IDX" => Some(7),
        "GRAD_TEXTURE_IDX" => Some(8),
        "GAUSSIAN_INTEGRAL_TEXTURE_IDX" => Some(9),
        "FEATHER_ATLAS_TEXTURE_IDX" => Some(10),
        "IMAGE_TEXTURE_IDX" => Some(11),
        "COLOR_PLANE_IDX+DEFAULT_BINDINGS_SET_SIZE" => Some(16),
        "CLIP_PLANE_IDX+DEFAULT_BINDINGS_SET_SIZE" => Some(17),
        "COVERAGE_PLANE_IDX+DEFAULT_BINDINGS_SET_SIZE" => Some(19),
        "None" | "DontCare" | "UInt16" => Some(0),
        "Lines" | "Store" | "Load" => Some(1),
        "Back" | "Fragment" => Some(2),
        "Triangle" => Some(3),
        "TriangleStrip" => Some(4),
        "Clear" => Some(2),
        "Buffers|RenderTargets" => Some(5),
        "sizeof(uint32_t)" => Some(core::mem::size_of::<u32>() as u64),
        "kPatchVertexBufferCount*sizeof(PatchVertex)" => Some(
            gpu::kPatchVertexBufferCount as u64 * core::mem::size_of::<gpu::PatchVertex>() as u64,
        ),
        "kPatchIndexBufferCount*sizeof(uint16_t)" => {
            Some(gpu::kPatchIndexBufferCount as u64 * core::mem::size_of::<u16>() as u64)
        }
        "sizeof(kTessSpanIndices)" => Some(core::mem::size_of_val(&gpu::kTessSpanIndices) as u64),
        "sizeof(kImageRectVertices)" => {
            Some(core::mem::size_of_val(&gpu::kImageRectVertices) as u64)
        }
        "sizeof(kImageRectIndices)" => Some(core::mem::size_of_val(&gpu::kImageRectIndices) as u64),
        value => value.parse().ok().or_else(|| source_sizeof_product(value)),
    }
}

fn source_sizeof_product(source: &str) -> Option<u64> {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
    let (count, ty) = source.split_once("*sizeof(")?;
    let ty = ty.strip_suffix(')')?;
    let count: u64 = count.parse().ok()?;
    let size = match ty {
        "uint16_t" => core::mem::size_of::<u16>(),
        "uint32_t" => core::mem::size_of::<u32>(),
        "PathData" => core::mem::size_of::<gpu::PathData>(),
        "PaintData" => core::mem::size_of::<gpu::PaintData>(),
        "PaintAuxData" => core::mem::size_of::<gpu::PaintAuxData>(),
        "ContourData" => core::mem::size_of::<gpu::ContourData>(),
        "GradientSpan" => core::mem::size_of::<gpu::GradientSpan>(),
        "TessVertexSpan" => core::mem::size_of::<gpu::TessVertexSpan>(),
        "ImageDrawInstance" => core::mem::size_of::<gpu::ImageDrawInstance>(),
        _ => return None,
    };
    Some(count * size as u64)
}

#[cfg(test)]
mod ownership_transfer_tests {
    use super::*;
    use objc2::runtime::NSObject;
    use objc2_foundation::NSObjectProtocol;

    struct NoopHost;

    impl NativeMetalHostCallbacks for NoopHost {
        fn log(&mut self, _message: String) {}

        fn generate_patch_buffer_data(
            &mut self,
            _vertex_buffer: &ProtocolObject<dyn MTLBuffer>,
            _index_buffer: &ProtocolObject<dyn MTLBuffer>,
        ) {
        }

        fn make_ore_context(
            &mut self,
            _device: &ProtocolObject<dyn MTLDevice>,
            _queue: Option<&ProtocolObject<dyn MTLCommandQueue>>,
        ) -> Option<Box<dyn std::any::Any>> {
            None
        }

    }

    fn execution_without_real_device() -> Objc2MetalExecution {
        let object = NSObject::new();
        let device = unsafe {
            Retained::cast_unchecked::<ProtocolObject<dyn MTLDevice>>(object)
        };
        let device_ptr = NonNull::from(&*device);
        let mut execution = Objc2MetalExecution {
            registry_id: next_registry_id(),
            device: device_ptr,
            device_handle: Handle::NIL,
            host: Box::new(NoopHost),
            objects: Vec::new(),
            free_slots: Vec::new(),
            retirement_queue: Objc2MetalRetirementQueue::default(),
            execution_inventory: ExecutionInventoryState::default(),
            adopted_background_libraries: Vec::new(),
            _recording_thread: core::marker::PhantomData,
        };
        execution.device_handle = execution.insert(device, MetalObjectKind::Device);
        execution
    }

    fn insert_probe(
        execution: &mut Objc2MetalExecution,
        kind: MetalObjectKind,
    ) -> (Handle, Retained<NSObject>) {
        let object = NSObject::new();
        let observer = object.clone();
        let handle = execution.insert(object, kind);
        (handle, observer)
    }

    #[test]
    fn transfer_rejects_wrong_kind_and_second_transfer() {
        let mut execution = execution_without_real_device();
        let (handle, observer) = insert_probe(&mut execution, MetalObjectKind::Texture);

        assert!(execution
            .take_owned(handle, MetalObjectKind::Buffer)
            .is_none());
        let owner = execution
            .take_owned(handle, MetalObjectKind::Texture)
            .expect("first exact typed transfer must succeed");
        assert!(execution
            .take_owned(handle, MetalObjectKind::Texture)
            .is_none());
        assert!(execution.object(handle, MetalObjectKind::Texture).is_some());

        let cloned_owner = execution
            .clone_owned(handle, MetalObjectKind::Texture)
            .expect("a source strong assignment creates an independent +1");
        assert_ne!(cloned_owner.handle(), handle);
        assert_eq!(observer.retainCount(), 3);
        drop(cloned_owner);
        assert_eq!(observer.retainCount(), 2);
        drop(owner);
        assert_eq!(observer.retainCount(), 1);
    }

    #[test]
    fn strong_pipeline_clone_preserves_the_native_semantic_identity() {
        let mut execution = execution_without_real_device();
        let (pipeline, _observer) =
            insert_probe(&mut execution, MetalObjectKind::RenderPipelineState);
        let semantic = PipelineSemantic::simple(PipelineSemanticKind::ColorRamp);
        execution.tag_pipeline(pipeline, semantic);

        let clone = execution
            .clone_owned(pipeline, MetalObjectKind::RenderPipelineState)
            .expect("source pipeline strong local");
        assert_ne!(clone.handle(), pipeline);
        assert_eq!(execution.pipeline_semantic(pipeline), Some(semantic));
        assert_eq!(execution.pipeline_semantic(clone.handle()), Some(semantic));
    }

    #[test]
    fn draw_pipeline_capture_and_framebuffer_are_exact_independent_strong_clones() {
        let mut execution = execution_without_real_device();

        let (device, device_observer) = insert_probe(&mut execution, MetalObjectKind::Device);
        let device_count = device_observer.retainCount();
        let device_capture = execution
            .clone_owned(device, MetalObjectKind::Device)
            .expect("the DrawPipeline block capture adds one device +1");
        assert_ne!(device_capture.handle(), device);
        assert_eq!(device_observer.retainCount(), device_count + 1);
        assert!(execution.object(device, MetalObjectKind::Device).is_some());
        drop(device_capture);
        assert_eq!(device_observer.retainCount(), device_count);
        assert!(execution.object(device, MetalObjectKind::Device).is_some());

        let (descriptor, descriptor_observer) =
            insert_probe(&mut execution, MetalObjectKind::RenderPipelineDescriptor);
        let attachment = NSObject::new();
        let attachment_observer = attachment.clone();
        let attachment_count = attachment_observer.retainCount();
        let attachment_alias = execution
            .insert_borrowed_child(
                descriptor,
                NonNull::from(&*attachment).cast::<AnyObject>(),
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .expect("the descriptor publishes its +0 framebuffer child");
        assert_eq!(attachment_observer.retainCount(), attachment_count);

        let framebuffer = execution
            .clone_owned(
                attachment_alias,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .expect("the named framebuffer local adds exactly one +1");
        assert_ne!(framebuffer.handle(), attachment_alias);
        assert_eq!(attachment_observer.retainCount(), attachment_count + 1);
        execution.retire_handle(attachment_alias);
        assert!(execution
            .object(
                framebuffer.handle(),
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .is_some());
        assert_eq!(attachment_observer.retainCount(), attachment_count + 1);
        drop(framebuffer);
        assert_eq!(attachment_observer.retainCount(), attachment_count);

        execution.retire_handle(descriptor);
        assert_eq!(descriptor_observer.retainCount(), 1);
        drop(attachment);
        assert_eq!(attachment_observer.retainCount(), 1);
    }

    #[test]
    fn descriptor_child_alias_is_retain_neutral_and_stale_with_its_parent() {
        let mut execution = execution_without_real_device();
        let (parent, parent_observer) =
            insert_probe(&mut execution, MetalObjectKind::RenderPipelineDescriptor);
        let child = NSObject::new();
        let child_observer = child.clone();
        let retain_count_before_alias = child_observer.retainCount();
        let child_pointer = NonNull::from(&*child).cast::<AnyObject>();
        let alias = execution
            .insert_borrowed_child(
                parent,
                child_pointer,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .expect("a live descriptor publishes its +0 attachment child");

        assert_eq!(
            child_observer.retainCount(),
            retain_count_before_alias,
            "publishing a descriptor-owned +0 child must not retain it"
        );
        assert!(execution
            .object(
                alias,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .is_some());

        execution.retire_handle(parent);
        assert!(execution
            .object(parent, MetalObjectKind::RenderPipelineDescriptor)
            .is_none());
        assert!(execution
            .object(
                alias,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            )
            .is_none());
        assert_eq!(parent_observer.retainCount(), 1);
        assert_eq!(child_observer.retainCount(), retain_count_before_alias);
        drop(child);
        assert_eq!(child_observer.retainCount(), 1);
    }

    #[test]
    fn attachment_collection_alias_is_retain_neutral_and_ends_before_its_child() {
        let mut execution = execution_without_real_device();
        for (parent_kind, collection_kind, child_kind) in [
            (
                MetalObjectKind::RenderPipelineDescriptor,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptorArray,
                MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            ),
            (
                MetalObjectKind::RenderPassDescriptor,
                MetalObjectKind::RenderPassColorAttachmentDescriptorArray,
                MetalObjectKind::RenderPassColorAttachmentDescriptor,
            ),
        ] {
            let (parent, parent_observer) = insert_probe(&mut execution, parent_kind);
            let collection = NSObject::new();
            let collection_observer = collection.clone();
            let collection_retain_count = collection_observer.retainCount();
            let collection_alias = execution
                .insert_borrowed_child(
                    parent,
                    NonNull::from(&*collection).cast::<AnyObject>(),
                    collection_kind,
                )
                .expect("a live descriptor publishes its +0 attachment collection");
            let child = NSObject::new();
            let child_observer = child.clone();
            let child_retain_count = child_observer.retainCount();
            let child_alias = execution
                .insert_borrowed_child(
                    parent,
                    NonNull::from(&*child).cast::<AnyObject>(),
                    child_kind,
                )
                .expect("the indexed child remains tied to the descriptor");

            assert_eq!(collection_observer.retainCount(), collection_retain_count);
            assert_eq!(child_observer.retainCount(), child_retain_count);
            execution.retire_handle(collection_alias);
            assert!(execution.object(collection_alias, collection_kind).is_none());
            assert!(execution.object(child_alias, child_kind).is_some());
            assert_eq!(collection_observer.retainCount(), collection_retain_count);
            assert_eq!(child_observer.retainCount(), child_retain_count);

            execution.retire_handle(parent);
            assert!(execution.object(child_alias, child_kind).is_none());
            assert_eq!(parent_observer.retainCount(), 1);
            assert_eq!(collection_observer.retainCount(), collection_retain_count);
            assert_eq!(child_observer.retainCount(), child_retain_count);
        }
    }

    #[test]
    fn registry_live_by_kind_reverse_owner_and_failpoint_sweep() {
        // Every selector-created strong owner used by the translated source
        // must be visible in the registry under its exact kind, and every
        // failure/retirement path must leave the slot stale rather than
        // silently retaining a parent or child. Keep this table in source
        // declaration order, then release owners in the authored reverse
        // order used by the native units.
        let kinds = [
            MetalObjectKind::TextureDescriptor,
            MetalObjectKind::SamplerDescriptor,
            MetalObjectKind::RenderPipelineDescriptor,
            MetalObjectKind::RenderPipelineColorAttachmentDescriptor,
            MetalObjectKind::RenderPassDescriptor,
            MetalObjectKind::RenderPassColorAttachmentDescriptor,
            MetalObjectKind::Function,
            MetalObjectKind::RenderPipelineState,
            MetalObjectKind::CommandBuffer,
            MetalObjectKind::RenderCommandEncoder,
        ];
        let mut execution = execution_without_real_device();
        let mut owners = Vec::new();
        let mut observers = Vec::new();
        for kind in kinds {
            let (handle, observer) = insert_probe(&mut execution, kind);
            assert_eq!(execution.entry(handle, kind).map(|entry| entry.kind), Some(kind));
            let owner = execution
                .take_owned(handle, kind)
                .expect("selector creation +1 must transfer for every live kind");
            assert!(execution.object(handle, kind).is_some());
            owners.push((handle, kind, owner));
            observers.push(observer);
        }
        for ((handle, kind, owner), observer) in owners.into_iter().rev().zip(observers.into_iter().rev()) {
            drop(owner);
            assert!(execution.object(handle, kind).is_none());
            assert_eq!(observer.retainCount(), 1, "owner release must be exact");
        }

        // A failed typed transfer is the source failpoint equivalent: it
        // must not consume the creation owner or leave a second alias.
        let (handle, observer) = insert_probe(&mut execution, MetalObjectKind::Texture);
        assert!(execution.take_owned(handle, MetalObjectKind::Buffer).is_none());
        assert!(execution.object(handle, MetalObjectKind::Texture).is_some());
        let owner = execution
            .take_owned(handle, MetalObjectKind::Texture)
            .expect("retry after typed failure must still transfer once");
        drop(owner);
        assert_eq!(observer.retainCount(), 1);
        assert!(execution.object(handle, MetalObjectKind::Texture).is_none());
    }

    #[test]
    fn function_name_is_a_scoped_native_owner_not_text_registry_state() {
        let mut execution = execution_without_real_device();
        let owner = execution
            .make_function_name("p00000000::vertexMain")
            .expect("native NSString owner must be created");
        let handle = owner.handle();
        assert_eq!(handle.kind, MetalObjectKind::NSString);
        assert!(execution.object(handle, MetalObjectKind::NSString).is_some());
        drop(owner);
        assert!(execution.object(handle, MetalObjectKind::NSString).is_none());
    }

    #[test]
    fn precompiled_function_name_runs_the_variadic_source_bridge_without_retain() {
        let mut execution = execution_without_real_device();
        let handle = objc2::rc::autoreleasepool(|_| {
            let handle = execution
                .make_precompiled_function_name(b'p', "0000000000", "GC")
                .expect("source stringWithFormat bridge must produce a name");
            assert_eq!(handle.kind, MetalObjectKind::NSString);
            let native = execution
                .object(handle, MetalObjectKind::NSString)
                .expect("the +0 alias stays valid for its synchronous source scope");
            let utf8: *const std::ffi::c_char = unsafe { msg_send![native, UTF8String] };
            assert!(!utf8.is_null());
            assert_eq!(
                unsafe { std::ffi::CStr::from_ptr(utf8) }.to_bytes(),
                b"p0000000000::GC"
            );
            // The producer returns an autoreleased +0 object. The selector
            // alias ends before the pool drains; it is never turned into a
            // synthetic strong owner by the bridge.
            execution.retire_handle(handle);
            assert!(execution.object(handle, MetalObjectKind::NSString).is_none());
            handle
        });
        assert!(execution.object(handle, MetalObjectKind::NSString).is_none());

        let slots_before_failure = execution.objects.len();
        assert!(execution
            .make_precompiled_function_name(b'p', "bad\0namespace", "GC")
            .is_none());
        assert!(execution
            .make_precompiled_function_name(b'p', "0000000000", "bad\0base")
            .is_none());
        assert_eq!(execution.objects.len(), slots_before_failure);
    }

    #[test]
    fn static_function_literals_are_stable_immortal_borrows_outside_the_registry() {
        let execution = execution_without_real_device();
        let slots_before = execution.objects.len();
        let mut identities = std::collections::BTreeSet::new();
        for expected in SOURCE_STATIC_FUNCTION_NAMES {
            let literal = source_static_function_name(expected)
                .expect("every pinned static function literal has a native mapping");
            let again = source_static_function_name(expected).unwrap();
            assert!(core::ptr::eq(literal, again), "{expected} identity changed");
            assert!(identities.insert(core::ptr::from_ref(literal) as usize));
            let retain_count = literal.retainCount();
            let utf8: *const std::ffi::c_char = unsafe { msg_send![literal, UTF8String] };
            assert_eq!(
                unsafe { std::ffi::CStr::from_ptr(utf8) }.to_bytes(),
                expected.as_bytes()
            );
            assert_eq!(literal.retainCount(), retain_count);
        }
        assert!(source_static_function_name("not-a-source-literal").is_none());
        assert_eq!(execution.objects.len(), slots_before);
        assert!(execution.objects.iter().all(|slot| {
            slot.entry
                .as_ref()
                .is_none_or(|entry| entry.kind != MetalObjectKind::NSString)
        }));
    }

    #[test]
    fn objc_parameter_projection_is_retain_neutral_for_every_object_family() {
        let mut execution = execution_without_real_device();
        for kind in [
            MetalObjectKind::Device,
            MetalObjectKind::Library,
            MetalObjectKind::RenderPipelineDescriptor,
            MetalObjectKind::RenderCommandEncoder,
        ] {
            let (handle, observer) = insert_probe(&mut execution, kind);
            let retain_count = observer.retainCount();
            let slots = execution.objects.len();
            for _ in 0..4 {
                let borrowed = execution
                    .object(handle, kind)
                    .expect("caller-owned parameter remains borrowable");
                assert_eq!(
                    NonNull::from(borrowed).as_ptr(),
                    NonNull::from(&*observer).cast::<AnyObject>().as_ptr()
                );
                assert_eq!(observer.retainCount(), retain_count);
                assert_eq!(execution.objects.len(), slots);
            }
            if kind == MetalObjectKind::Device {
                let borrowed = execution
                    .device_for_handle(handle)
                    .expect("typed device parameter projection");
                assert_eq!(
                    NonNull::from(borrowed).cast::<AnyObject>().as_ptr(),
                    NonNull::from(&*observer).cast::<AnyObject>().as_ptr()
                );
                assert_eq!(observer.retainCount(), retain_count);
            }
            execution.retire_handle(handle);
            assert_eq!(observer.retainCount(), 1);
        }
    }

    #[test]
    fn owner_drop_invalidates_alias_then_reuses_slot_with_new_generation() {
        let mut execution = execution_without_real_device();
        let (handle, observer) = insert_probe(&mut execution, MetalObjectKind::Texture);
        assert_eq!(observer.retainCount(), 2);

        let owner = execution
            .take_owned(handle, MetalObjectKind::Texture)
            .expect("creation retain must transfer");
        let validity = match &execution
            .entry(handle, MetalObjectKind::Texture)
            .expect("borrowed alias remains published")
            .object
        {
            RetainedMetalObject::BorrowedObjectiveC { validity, .. } => validity.clone(),
            _ => panic!("transferred entry must be nonowning"),
        };
        assert!(validity.is_live());
        assert_eq!(observer.retainCount(), 2, "transfer must not add a retain");

        drop(owner);
        assert!(!validity.is_live());
        assert!(execution.object(handle, MetalObjectKind::Texture).is_none());
        assert_eq!(observer.retainCount(), 1, "owner releases exactly one +1");

        let (replacement, _replacement_observer) =
            insert_probe(&mut execution, MetalObjectKind::Texture);
        assert_eq!(replacement.slot, handle.slot);
        assert_eq!(replacement.generation, handle.generation + 1);
        assert!(execution.object(handle, MetalObjectKind::Texture).is_none());
        assert!(execution
            .take_owned(handle, MetalObjectKind::Texture)
            .is_none());
    }

    #[test]
    fn executor_can_drop_before_transferred_owner_without_double_release() {
        let mut execution = execution_without_real_device();
        let object = NSObject::new();
        let observer = object.clone();
        let mut owner = unsafe {
            OwnedMetalHandle::detached_native(
                MetalObjectKind::Buffer,
                Retained::cast_unchecked::<AnyObject>(object),
            )
        };
        assert_eq!(owner.handle(), Handle::NIL);
        let handle = execution
            .publish_owned(&mut owner)
            .expect("detached direct owner must publish a borrowed alias");
        assert_eq!(observer.retainCount(), 2);
        assert_eq!(owner.handle(), handle);

        drop(execution);
        assert_eq!(owner.handle(), Handle::NIL);
        assert_eq!(
            observer.retainCount(),
            2,
            "executor's borrowed alias must not release the canonical +1"
        );

        let mut replacement_execution = execution_without_real_device();
        let replacement = replacement_execution
            .publish_owned(&mut owner)
            .expect("owner must republish after the old executor invalidates its alias");
        assert_ne!(replacement.registry, handle.registry);
        assert!(replacement_execution
            .object(handle, MetalObjectKind::Buffer)
            .is_none());
        assert!(replacement_execution
            .object(replacement, MetalObjectKind::Buffer)
            .is_some());
        drop(owner);
        assert!(replacement_execution
            .object(replacement, MetalObjectKind::Buffer)
            .is_none());
        assert_eq!(observer.retainCount(), 1);
    }

    #[test]
    fn live_foreign_alias_is_invalidated_before_republication() {
        let mut first = execution_without_real_device();
        let mut second = execution_without_real_device();
        let (first_handle, observer) = insert_probe(&mut first, MetalObjectKind::Texture);
        let mut owner = first
            .take_owned(first_handle, MetalObjectKind::Texture)
            .expect("creation retain must transfer");
        assert_eq!(observer.retainCount(), 2);

        let second_handle = second
            .publish_owned(&mut owner)
            .expect("canonical owner must move its alias to the new executor");
        assert_ne!(second_handle.registry, first_handle.registry);
        assert!(first
            .object(first_handle, MetalObjectKind::Texture)
            .is_none());
        assert!(second
            .object(second_handle, MetalObjectKind::Texture)
            .is_some());
        assert_eq!(
            observer.retainCount(),
            2,
            "republishing a borrowed alias must not retain"
        );

        drop(owner);
        assert!(second
            .object(second_handle, MetalObjectKind::Texture)
            .is_none());
        assert_eq!(observer.retainCount(), 1);
    }

    #[test]
    fn compiled_library_adoption_transfers_exact_plus_one_and_publishes_alias() {
        let mut execution = execution_without_real_device();
        assert!(unsafe { execution
            .adopt_compiled_library(core::ptr::null_mut())
            .is_none() });

        let library = NSObject::new();
        let observer = library.clone();
        let raw = Retained::into_raw(library).cast::<
            crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::MTLLibrary,
        >();
        let handle = unsafe { execution
            .adopt_compiled_library(raw) }
            .expect("non-null compiler +1 must become a library handle");
        assert_eq!(handle.kind, MetalObjectKind::Library);
        assert_eq!(observer.retainCount(), 2, "adoption must not add a retain");

        let owner = execution
            .take_owned(handle, MetalObjectKind::Library)
            .expect("published compiler result must transfer to source ownership");
        assert!(execution.object(handle, MetalObjectKind::Library).is_some());
        assert_eq!(observer.retainCount(), 2);
        drop(owner);
        assert!(execution.object(handle, MetalObjectKind::Library).is_none());
        assert_eq!(observer.retainCount(), 1);
    }

    #[test]
    fn live_background_library_keeps_one_identity_through_context_adoption_and_release() {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
        use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{
            MetalFeatures, Retained as SourceRetained,
        };
        use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::{
            self as background, BackgroundCompileJob,
        };

        objc2::rc::autoreleasepool(|_| {
            let _ = background::take_owner_detail_events();
            let device = objc2_metal::MTLCreateSystemDefaultDevice()
                .expect("native Metal device required");
            let compiler_device = unsafe {
                SourceRetained::from_raw_retained(
                    Retained::into_raw(device.clone()).cast(),
                )
            }
            .expect("compiler device transfer");
            let compiler = background::new_for_device_with_sources(
                compiler_device,
                MetalFeatures::default(),
                background::GeneratedShaderSources {
                    metal: "#include <metal_stdlib>\nusing namespace metal;\n",
                    constants: "",
                    flush_uniforms: "",
                    common: "",
                    advanced_blend: "",
                    draw_path_common: "",
                    draw_path_vert: "",
                    draw_raster_order_path_frag: "",
                    draw_image_mesh_vert: "",
                    draw_mesh_frag: "",
                    atomic_draw: "",
                },
            );
            compiler.pushJob(BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            ));
            let mut finished = BackgroundCompileJob::new(
                gpu::DrawType::imageMesh,
                gpu::ShaderFeatures::NONE,
                gpu::InterlockMode::rasterOrdering,
                gpu::ShaderMiscFlags::none,
            );
            assert!(compiler.popFinishedJob(&mut finished, true));
            let raw = finished
                .take_compiled_library_raw()
                .expect("worker compiled library transfer");
            let identity = raw as usize;

            let mut execution = Objc2MetalExecution::new(device, Box::new(NoopHost));
            let handle = unsafe { execution.adopt_compiled_library(raw) }
                .expect("context adopts exact compiler +1");
            assert!(execution.retire(handle, MetalObjectKind::Library));
            drop(finished);
            drop(compiler);

            let phases = background::take_owner_detail_events()
                .into_iter()
                .filter(|event| {
                    event.ledger_id == "BG-LIB-COMPILED" && event.identity == identity
                })
                .map(|event| event.phase)
                .collect::<Vec<_>>();
            assert_eq!(
                phases,
                vec![
                    "Create",
                    "TransferJob",
                    "TransferFinished",
                    "TransferCaller",
                    "AdoptContext",
                    "ReleaseContext",
                ],
                "one compiled library identity crosses worker, finished queue, caller, context and final release"
            );
        });
    }

    #[test]
    fn owner_can_drop_before_executor_and_stale_retirement_fails_closed() {
        let mut execution = execution_without_real_device();
        let (handle, observer) = insert_probe(&mut execution, MetalObjectKind::CommandQueue);
        let owner = execution
            .take_owned(handle, MetalObjectKind::CommandQueue)
            .expect("creation retain must transfer");

        drop(owner);
        assert_eq!(observer.retainCount(), 1);
        assert!(execution
            .object(handle, MetalObjectKind::CommandQueue)
            .is_none());
        assert!(execution.retire_command_queue(handle));
        assert!(!execution.retire_command_queue(handle));

        let (replacement, _replacement_observer) =
            insert_probe(&mut execution, MetalObjectKind::CommandQueue);
        assert_eq!(replacement.slot, handle.slot);
        assert!(replacement.generation > handle.generation);
        assert!(!execution.retire_command_queue(handle));
        assert!(execution
            .object(replacement, MetalObjectKind::CommandQueue)
            .is_some());
    }

    #[test]
    fn direct_device_operation_rejects_wrong_kind_before_protocol_cast() {
        let owner = unsafe {
            OwnedMetalHandle::native(
                Handle::new(7, MetalObjectKind::Texture),
                Retained::cast_unchecked::<AnyObject>(NSObject::new()),
                MetalAliasValidity::live(),
            )
        };
        assert!(owner
            .new_buffer_with_length(16, MTLResourceOptions::StorageModeShared)
            .is_none());
        assert!(owner.buffer_contents().is_none());
    }
}
