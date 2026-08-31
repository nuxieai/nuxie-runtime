//! Upstream tests/unit_tests/renderer/ore_deferred_reuse_test.cpp at e949498e.
use nuxie_ore_metal::{
    cmd::id_allocator::IdAllocator,
    ore_cmd::{
        ore_command_buffer::*, ore_commands::*, ore_make_recording::*, ore_resource_commands::*,
    },
    types::*,
};
#[test]
fn id_allocator_recycles_ids_with_a_bumped_generation() {
    let mut ids = IdAllocator::default();
    let a = ids.alloc();
    let b = ids.alloc();
    assert_eq!(a.id, 0);
    assert_eq!(b.id, 1);
    ids.release(a.id, a.generation);
    let c = ids.alloc();
    assert_eq!(c.id, 0);
    assert_eq!(c.generation, 1);
    let d = ids.alloc();
    assert_eq!(d.id, 2);
    assert_eq!(d.generation, 0);
    ids.release(c.id, c.generation);
    let e = ids.alloc();
    assert_eq!(e.id, 0);
    assert_eq!(e.generation, 2);
    let mut ids = IdAllocator::default();
    let a = ids.alloc();
    ids.release(a.id, u32::MAX);
    let b = ids.alloc();
    assert_eq!(b.id, 1);
    assert_eq!(b.generation, 0);
}
#[test]
fn ordered_stream_records_a_create_write_destroy_recreate_lifecycle() {
    let mut ids = IdAllocator::default();
    let mut cb = OreCommandBuffer::default();
    let a = ids.alloc();
    recordMakeBuffer(
        &mut cb,
        a.id,
        a.generation,
        &BufferDesc {
            size: 16,
            usage: BufferUsage::vertex,
            data: None,
            immutable: false,
            label: None,
        },
    );
    let data = encodePods(&[10u32, 20, 30, 40]);
    recordBufferUpdate(&mut cb, a.id, Some(&data), data.len() as u32, 0);
    recordDestroyResource(&mut cb, a.id, a.generation);
    ids.release(a.id, a.generation);
    let b = ids.alloc();
    assert_eq!(b.id, a.id);
    assert_eq!(b.generation, 1);
    recordMakeBuffer(
        &mut cb,
        b.id,
        b.generation,
        &BufferDesc {
            size: 32,
            usage: BufferUsage::index,
            data: None,
            immutable: false,
            label: None,
        },
    );
    let mut r = OreCommandReader::new(cb.command_bytes(), cb.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::makeBuffer));
    let m0: MakeResourcePOD = r.read();
    let d0: BufferDescPOD = r.read();
    assert_eq!(m0.id, a.id);
    assert_eq!(m0.generation, 0);
    assert_eq!(d0.size, 16);
    assert_eq!(d0.usage, BufferUsage::vertex);
    assert_eq!(r.next(), Some(CommandType::bufferUpdate));
    let up: BufferUpdatePOD = r.read();
    assert_eq!(up.handle, a.id);
    assert_eq!(up.offset, 0);
    let bytes = r.blob_at(up.bytes.offset, up.bytes.size);
    assert_eq!(bytes.len(), data.len());
    assert_eq!(bytes, data);
    assert_eq!(r.next(), Some(CommandType::destroyResource));
    let ds: DestroyResourcePOD = r.read();
    assert_eq!(ds.handle, a.id);
    assert_eq!(ds.generation, 0);
    assert_eq!(r.next(), Some(CommandType::makeBuffer));
    let m1: MakeResourcePOD = r.read();
    let d1: BufferDescPOD = r.read();
    assert_eq!(m1.id, a.id);
    assert_eq!(m1.generation, 1);
    assert_eq!(d1.size, 32);
    assert_eq!(d1.usage, BufferUsage::index);
    assert!(r.next::<CommandType>().is_none());
}
