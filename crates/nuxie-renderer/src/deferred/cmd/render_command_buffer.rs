//! renderer/cmd/render_command_buffer.hpp at e949498e.
use super::command_stream::{CommandByteStream, CommandReader, WirePod};
use super::id_allocator::IdAllocator;
use super::recording_thread::RecordingThread;
use super::render_commands::{DestroyResourcePod, RenderCmd};
use std::sync::{Arc, Mutex};

pub type SharedIdAllocator = Arc<Mutex<IdAllocator>>;
pub struct PendingDestroy {
    pub kind: u8,
    pub id: u32,
    pub generation: u32,
    pub allocator: Option<SharedIdAllocator>,
}
pub struct RenderCommandBuffer {
    stream: CommandByteStream,
    recording_thread: RecordingThread,
    pending_destroys: Mutex<Vec<PendingDestroy>>,
    frame_id: u32,
    recorder_live: bool,
}
impl Default for RenderCommandBuffer {
    fn default() -> Self {
        Self {
            stream: CommandByteStream::default(),
            recording_thread: RecordingThread::default(),
            pending_destroys: Mutex::new(Vec::new()),
            frame_id: 0,
            recorder_live: true,
        }
    }
}
impl RenderCommandBuffer {
    pub fn unregister_recorder(&mut self) {
        self.recorder_live = false;
    }
    pub fn recorder_live(&self) -> bool {
        self.recorder_live
    }
    pub fn bind_recording_thread(&mut self) {
        self.recording_thread.bind();
    }
    pub fn append<P: WirePod>(&mut self, command: RenderCmd, pod: &P) {
        self.recording_thread.check();
        self.stream.write(&(command as u8));
        self.stream.write(pod);
    }
    pub fn append_type(&mut self, command: RenderCmd) {
        self.recording_thread.check();
        self.stream.write(&(command as u8));
    }
    pub fn queue_destroy(
        &self,
        kind: u8,
        id: u32,
        generation: u32,
        allocator: Option<SharedIdAllocator>,
    ) {
        self.pending_destroys.lock().unwrap().push(PendingDestroy {
            kind,
            id,
            generation,
            allocator,
        });
    }
    pub fn drain_destroys(&mut self) {
        let pending = std::mem::take(&mut *self.pending_destroys.lock().unwrap());
        for destroy in pending {
            self.append(
                RenderCmd::DestroyResource,
                &DestroyResourcePod {
                    kind: destroy.kind,
                    id: destroy.id,
                    generation: destroy.generation,
                },
            );
            if let Some(allocator) = destroy.allocator {
                allocator
                    .lock()
                    .unwrap()
                    .release(destroy.id, destroy.generation);
            }
        }
    }
    pub fn reset(&mut self) {
        self.stream.clear_bytes();
        self.frame_id = self.frame_id.wrapping_add(1);
    }
    pub fn frame_id(&self) -> u32 {
        self.frame_id
    }
}
impl std::ops::Deref for RenderCommandBuffer {
    type Target = CommandByteStream;
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}
impl std::ops::DerefMut for RenderCommandBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
pub type RenderCommandReader<'a> = CommandReader<'a>;
