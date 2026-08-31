//! tests/unit_tests/renderer/gpu_census_test.cpp at e949498e.
use super::super::{
    deferred_replayer::*, deferred_session::DeferredSession, gpu_census::GpuCensus,
};
use super::*;
fn replay(s: &mut DeferredSession, r: &mut DeferredReplayer, sink: &mut TestSink) -> GpuCensus {
    let frame = take_frame(s);
    r.replay_frame(&frame, sink);
    r.gpu_census()
}
#[test]
fn counts_resident_not_running_total() {
    let mut s = DeferredSession::new(None);
    let mut r = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let screen = s.screen_renderer(0);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    let c = replay(&mut s, &mut r, &mut sink);
    assert_eq!(c.paths, 1);
    assert_eq!(c.paints, 1);
    assert_eq!(c.total_bytes(), 0);
    assert_eq!(r.gpu_census().total_bytes(), c.total_bytes());
    assert_eq!(r.gpu_census().live_objects(), c.live_objects());
}
#[test]
fn bytes_scale_with_resident_resources() {
    let mut s = DeferredSession::new(None);
    let mut r = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let screen = s.screen_renderer(0);
    let paint = s.make_render_paint();
    let _buffer = s.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 1024);
    let path = s.make_empty_render_path();
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    let one = replay(&mut s, &mut r, &mut sink);
    assert_eq!(one.buffers, 1);
    assert_eq!(one.buffer_bytes, 1024);
    assert_eq!(one.total_bytes(), 1024);
    let _buffer2 = s.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 1024);
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    let two = replay(&mut s, &mut r, &mut sink);
    assert_eq!(two.buffers, 2);
    assert_eq!(two.buffer_bytes, 2048);
    assert_eq!(two.paths, one.paths);
    assert_eq!(two.paints, one.paints);
}
#[test]
fn destroyed_resource_leaves_census_keeps_slot() {
    let mut s = DeferredSession::new(None);
    let mut r = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let screen = s.screen_renderer(0);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    {
        let _doomed = s.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 4096);
        screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
        let live = replay(&mut s, &mut r, &mut sink);
        assert_eq!(live.buffer_bytes, 4096);
        assert!(live.slots_2d >= live.live_objects());
    }
    s.command_buffer().lock().unwrap().drain_destroys();
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    let after = replay(&mut s, &mut r, &mut sink);
    assert_eq!(after.buffers, 0);
    assert_eq!(after.buffer_bytes, 0);
    assert!(after.slots_2d >= 1);
}
#[test]
fn reset_empties_census() {
    let mut s = DeferredSession::new(None);
    let mut r = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let screen = s.screen_renderer(0);
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    let _buffer = s.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 2048);
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    assert_eq!(replay(&mut s, &mut r, &mut sink).total_bytes(), 2048);
    r.reset();
    let empty = r.gpu_census();
    assert_eq!(empty.total_bytes(), 0);
    assert_eq!(empty.live_objects(), 0);
    assert_eq!(empty.slots_2d, 0);
    assert_eq!(empty.slots_ore, 0);
}
#[test]
fn ore_texture_format_sizes() {
    use nuxie_ore_metal::types::{textureFormatBytesPerTexel, TextureFormat};
    assert_eq!(textureFormatBytesPerTexel(TextureFormat::rgba8unorm), 4);
    assert_eq!(textureFormatBytesPerTexel(TextureFormat::r8unorm), 1);
    assert_eq!(textureFormatBytesPerTexel(TextureFormat::rgba32float), 16);
}
