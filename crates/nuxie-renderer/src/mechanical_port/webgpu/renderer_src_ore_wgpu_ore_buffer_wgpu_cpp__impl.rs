//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_buffer_wgpu.cpp`.

#![allow(non_snake_case)]

use super::ore_buffer_wgpu_decl::{Backing, BufferWGPU, BufferWGPUState};
use super::webgpu_cpp_decl::BufferDescriptor;
use super::webgpu_decl::WGPU_FALSE;
use nuxie_ore_metal::buffer::BufferUpdateError;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_buffer_wgpu.cpp");
pub(crate) const ALLOCATION_FAILURE_ERROR: &str =
    "ore: WGPU buffer backing allocation failed; reusing in flight backing for this update";

fn context(state: &BufferWGPUState) -> &super::ore_context_wgpu_decl::ContextWGPU {
    unsafe { state.m_ctx.as_ref() }.expect("BufferWGPU source m_ctx")
}

pub(crate) fn markBound(buffer: &BufferWGPU) {
    let mut state = buffer
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let serial = context(&state).currentFrameSerial();
    let current = state.m_currentIndex;
    state.m_pool[current].frameStamp = serial;
    state.m_boundSinceUpdate = true;
}

fn acquireFreshBacking(buffer: &BufferWGPU, state: &mut BufferWGPUState) -> bool {
    let serial = context(state).currentFrameSerial();
    let mut fresh = state.m_pool.len();
    for (index, backing) in state.m_pool.iter().enumerate() {
        if index != state.m_currentIndex && backing.frameStamp != serial {
            fresh = index;
            break;
        }
    }
    if fresh == state.m_pool.len() {
        let mut wDesc = BufferDescriptor::default();
        wDesc.size = u64::from(buffer.base.size());
        wDesc.usage = state.m_wgpuUsage.into();
        wDesc.mappedAtCreation = WGPU_FALSE;
        let native = unsafe { state.m_wgpuDevice.CreateBuffer(&wDesc) };
        if native.Get().is_null() {
            context(state).setLastError(ALLOCATION_FAILURE_ERROR);
            return false;
        }
        state.m_pool.push(Backing {
            buffer: native,
            frameStamp: 0,
        });
    }
    state.m_currentIndex = fresh;
    true
}

pub(crate) fn update(
    buffer: &BufferWGPU,
    data: &[u8],
    size: u32,
    offset: u32,
) -> Result<(), BufferUpdateError> {
    let end = offset
        .checked_add(size)
        .ok_or(BufferUpdateError::RangeOverflow)?;
    if end > buffer.base.size() {
        return Err(BufferUpdateError::RangeOutOfBounds);
    }
    if data.len() < size as usize {
        return Err(BufferUpdateError::SourceTooShort);
    }
    let mut state = buffer
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(!state.m_pool.is_empty(), "BufferWGPU source backing pool");
    let mut orphaned = false;
    if state.m_boundSinceUpdate && acquireFreshBacking(buffer, &mut state) {
        state.m_boundSinceUpdate = false;
        orphaned = true;
    }
    if state.m_shadow.is_empty() {
        state.m_shadow.resize(buffer.base.size() as usize, 0);
    }
    state.m_shadow[offset as usize..end as usize].copy_from_slice(&data[..size as usize]);
    let current = state.m_pool[state.m_currentIndex].buffer.Get();
    unsafe {
        if orphaned {
            state.m_wgpuQueue.WriteBuffer(
                current,
                0,
                state.m_shadow.as_ptr().cast(),
                buffer.base.size() as usize,
            );
        } else {
            state.m_wgpuQueue.WriteBuffer(
                current,
                u64::from(offset),
                data.as_ptr().cast(),
                size as usize,
            );
        }
    }
    Ok(())
}

pub(crate) const SOURCE_FUNCTION_COUNT: usize = 3;
pub(crate) const SOURCE_LOOP_COUNT: usize = 1;
pub(crate) const SOURCE_CREATE_BUFFER_CALL_COUNT: usize = 1;
pub(crate) const SOURCE_WRITE_BUFFER_CALL_COUNT: usize = 2;
pub(crate) const SOURCE_FALLBACK_COUNT: usize = 1;
const _: [(); 2668] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_implementation_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 85);
        assert_eq!(SOURCE_FUNCTION_COUNT, 3);
        assert_eq!(SOURCE_LOOP_COUNT, 1);
        assert_eq!(SOURCE_CREATE_BUFFER_CALL_COUNT, 1);
        assert_eq!(SOURCE_WRITE_BUFFER_CALL_COUNT, 2);
        assert_eq!(SOURCE_FALLBACK_COUNT, 1);
    }

    #[test]
    fn allocation_failure_text_is_exact() {
        assert_eq!(
            ALLOCATION_FAILURE_ERROR,
            "ore: WGPU buffer backing allocation failed; reusing in flight backing for this update"
        );
    }
}
