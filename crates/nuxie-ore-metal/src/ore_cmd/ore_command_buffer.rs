//! renderer/ore/cmd/ore_command_buffer.hpp at e949498e.
#![allow(non_snake_case)]
use super::{
    ore_commands::{CommandType, DestroyResourcePOD},
    ore_handle::{INVALID_HANDLE, ResourceHandle},
    ore_resource_commands::{BlobRef, NO_BLOB},
};
use crate::cmd::{
    command_stream::{CommandByteStream, CommandReader, WirePod},
    id_allocator::IdAllocator,
    recording_thread::RecordingThread,
};
use crate::gpu_resource::AnyResourceHandle;
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

pub type SharedOreCommandBuffer = Rc<RefCell<OreCommandBuffer>>;
pub type SharedIdAllocator = Arc<Mutex<IdAllocator>>;
pub type DestroyQueue = Arc<Mutex<Vec<PendingDestroy>>>;
pub type OreCommandReader<'a> = CommandReader<'a>;
pub struct PendingDestroy {
    pub handle: ResourceHandle,
    pub generation: u32,
    pub allocator: Option<Weak<Mutex<IdAllocator>>>,
}

#[derive(Default)]
pub struct OreCommandBuffer {
    bytes: CommandByteStream,
    recordingThread: RecordingThread,
    pendingDestroys: DestroyQueue,
    keepAlive: Vec<AnyResourceHandle>,
    resourceIds: HashMap<usize, ResourceHandle>,
    pub realHandleProvider: Option<Box<dyn FnMut(&AnyResourceHandle) -> ResourceHandle>>,
}
impl Deref for OreCommandBuffer {
    type Target = CommandByteStream;
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}
impl DerefMut for OreCommandBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}
impl OreCommandBuffer {
    pub fn bindRecordingThread(&mut self) {
        self.recordingThread.bind();
    }
    pub fn recorderIdentity(&self) -> usize {
        Arc::as_ptr(&self.pendingDestroys) as usize
    }
    pub fn destroyQueue(&self) -> Weak<Mutex<Vec<PendingDestroy>>> {
        Arc::downgrade(&self.pendingDestroys)
    }
    pub fn capture(&mut self, resource: Option<&AnyResourceHandle>) -> ResourceHandle {
        let Some(resource) = resource else {
            return INVALID_HANDLE;
        };
        self.recordingThread.check();
        if let Some(provider) = &mut self.realHandleProvider {
            return provider(resource);
        }
        let identity = resource.allocation_identity();
        if let Some(handle) = self.resourceIds.get(&identity) {
            return *handle;
        }
        let handle = self.keepAlive.len() as ResourceHandle;
        self.keepAlive.push(resource.clone());
        self.resourceIds.insert(identity, handle);
        handle
    }
    pub fn append<P: WirePod>(&mut self, command: CommandType, pod: &P) {
        self.recordingThread.check();
        self.appendUnchecked(command, pod);
    }
    fn appendUnchecked<P: WirePod>(&mut self, command: CommandType, pod: &P) {
        self.bytes.write(&command);
        self.bytes.write(pod);
    }
    pub fn appendOpcode(&mut self, command: CommandType) {
        self.recordingThread.check();
        self.bytes.write(&command);
    }
    pub fn appendPayload<P: WirePod>(&mut self, pod: &P) {
        self.recordingThread.check();
        self.bytes.write(pod);
    }
    pub fn appendBlobRef(&mut self, data: Option<&[u8]>, size: u32, absent: bool) -> BlobRef {
        if absent {
            return NO_BLOB;
        }
        let data = data.unwrap_or(&[]);
        assert!(
            size as usize <= data.len(),
            "blob source is shorter than its recorded size"
        );
        BlobRef {
            offset: self.bytes.append_blob(&data[..size as usize]),
            size,
            pad: 0,
        }
    }
    pub fn appendStringRef(&mut self, value: Option<&str>) -> BlobRef {
        let Some(value) = value else {
            return NO_BLOB;
        };
        // C++ strlen ends at the first NUL and records its terminator.
        let prefix = value.as_bytes().split(|byte| *byte == 0).next().unwrap();
        let mut bytes = Vec::with_capacity(prefix.len() + 1);
        bytes.extend_from_slice(prefix);
        bytes.push(0);
        self.appendBlobRef(Some(&bytes), bytes.len() as u32, false)
    }
    pub fn queueDestroy(&self, pending: PendingDestroy) {
        self.pendingDestroys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(pending);
    }
    pub fn drainDestroys(&mut self) {
        let pending = std::mem::take(
            &mut *self
                .pendingDestroys
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for pending in pending {
            self.appendUnchecked(
                CommandType::destroyResource,
                &DestroyResourcePOD {
                    handle: pending.handle,
                    generation: pending.generation,
                },
            );
            if let Some(allocator) = pending.allocator.and_then(|allocator| allocator.upgrade()) {
                allocator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .release(pending.handle, pending.generation);
            }
        }
    }
    pub fn reset(&mut self) {
        self.recordingThread.check();
        self.bytes.clear_bytes();
        self.keepAlive.clear();
        self.resourceIds.clear();
    }
    pub fn keepAlive(&self) -> &[AnyResourceHandle] {
        &self.keepAlive
    }
}
