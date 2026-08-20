//! Asynchronous Metal submission and buffer-ring completion ownership.
//!
//! Pinned `commitCommandBuffer` commits without waiting, while `postFlush`
//! transfers the current ring lock into a command-buffer completion handler.
//! This module keeps that asynchronous ownership boundary deep: callers may
//! choose to wait for readback or presentation, but neither the context nor a
//! caller-owned frame must stay alive for the submitted ring slot to release.

use std::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use block2::RcBlock;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCommandBufferStatus};

use super::buffer_ring_coordinator::BufferRingCompletion;
use crate::RendererError;

#[derive(Debug)]
struct CompletionState {
    result: Mutex<Option<Result<(), String>>>,
    completed: Condvar,
}

impl CompletionState {
    fn lock_result(&self) -> MutexGuard<'_, Option<Result<(), String>>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn publish(&self, result: Result<(), String>) {
        let mut slot = self.lock_result();
        debug_assert!(slot.is_none(), "Metal completion published more than once");
        if slot.is_none() {
            *slot = Some(result);
            self.completed.notify_all();
        }
    }
}

/// Caller wait policy separated from asynchronous Metal submission ownership.
pub(crate) struct NativeMetalSubmissionCompletion {
    state: Arc<CompletionState>,
}

impl NativeMetalSubmissionCompletion {
    pub(crate) fn commit(
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        upload_completion: Option<BufferRingCompletion>,
    ) -> Self {
        let state = Arc::new(CompletionState {
            result: Mutex::new(None),
            completed: Condvar::new(),
        });
        let state_for_handler = Arc::clone(&state);
        let upload_completion = Mutex::new(upload_completion);
        let completed_handler = RcBlock::new(
            move |buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
                // SAFETY: Metal invokes the copied completion block with the
                // completed command buffer retained for the callback.
                let buffer = unsafe { buffer.as_ref() };
                let command_result = command_buffer_result(
                    buffer.status(),
                    buffer.error().map(|error| format!("{error:?}")),
                );
                let release_result = upload_completion
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .as_mut()
                    .map(BufferRingCompletion::complete)
                    .transpose()
                    .map(|_| ())
                    .map_err(|error| format!("release upload-ring ownership: {error:?}"));
                state_for_handler.publish(combine_results(command_result, release_result));
            },
        );
        // SAFETY: Metal copies this heap block before returning and supplies a
        // non-null retained command buffer to it. Captured state and ring
        // ownership are Arc/Mutex-backed and outlive the callback.
        unsafe {
            command_buffer.addCompletedHandler(RcBlock::as_ptr(&completed_handler));
        }
        command_buffer.commit();
        Self { state }
    }

    pub(crate) fn wait(&self) -> Result<(), RendererError> {
        let mut result = self.state.lock_result();
        while result.is_none() {
            result = self
                .state
                .completed
                .wait(result)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        result
            .as_ref()
            .expect("completion result checked above")
            .clone()
            .map_err(|message| RendererError::NativeMetal(message))
    }
}

fn command_buffer_result(
    status: MTLCommandBufferStatus,
    error: Option<String>,
) -> Result<(), String> {
    if status == MTLCommandBufferStatus::Completed {
        return Ok(());
    }
    let detail = error.unwrap_or_else(|| format!("status {status:?}"));
    Err(format!("command buffer failed: {detail}"))
}

fn combine_results(command: Result<(), String>, release: Result<(), String>) -> Result<(), String> {
    match (command, release) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(command), Ok(())) => Err(command),
        (Ok(()), Err(release)) => Err(release),
        (Err(command), Err(release)) => Err(format!("{command}; {release}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_metal::buffer_ring_coordinator::BufferRingCoordinator;

    #[test]
    fn failed_command_still_releases_the_submitted_ring_slot() {
        let coordinator = BufferRingCoordinator::new();
        let mut first = coordinator.prepare_to_flush();
        let second = coordinator.prepare_to_flush();
        let third = coordinator.prepare_to_flush();
        let mut completion = first.transfer_to_completion().unwrap();

        let command = command_buffer_result(
            MTLCommandBufferStatus::Error,
            Some("synthetic failure".to_owned()),
        );
        let release = completion
            .complete()
            .map_err(|error| format!("release upload-ring ownership: {error:?}"));
        assert_eq!(
            combine_results(command, release),
            Err("command buffer failed: synthetic failure".to_owned())
        );

        drop(first);
        drop(second);
        drop(third);
        let next = coordinator.prepare_to_flush();
        assert_eq!(next.slot(), 1);
    }
}
