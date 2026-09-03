//! renderer/cmd/deferred_replayer.hpp at e949498e.
use super::{
    canvas_schedule::schedule_canvases,
    deferred_session::{DeferredSegment, DeferredSession, SegmentTarget},
    gpu_census::{take_gpu_census, GpuCensus},
    render_handle::INVALID_RENDER_HANDLE,
    render_replay::*,
};
use crate::deferred::ore::{ore_make_replay::OreResident, ore_replay::replayOreStream};
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_render_api::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait DeferredFrameSink {
    // A retained factory projection makes the source's factory and frame
    // operations independently borrowable while a canvas hook is executing.
    fn factory(&mut self) -> PersistentFactoryContext;
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        self.factory().with_factory(|factory| factory.ore())
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner>;
    fn default_screen_target(&mut self) -> u64 {
        0
    }
    fn begin_ore_frame(&mut self) {}
    fn end_ore_frame(&mut self) {}
    fn after_ore_frame(&mut self) {
        self.factory()
            .with_factory(|factory| factory.scrub_state_after_ore());
    }
    fn begin_canvas_content(
        &mut self,
        _canvas: RenderCanvasHandle,
        _clear_color: u32,
    ) -> Option<RendererOwner> {
        None
    }
    fn end_canvas_content(&mut self) {}
}
#[derive(Default)]
pub struct DeferredFrame {
    pub commands: Vec<u8>,
    pub blobs: Vec<u8>,
    pub ore_commands: Vec<u8>,
    pub ore_blobs: Vec<u8>,
    pub canvas_images: Vec<Rc<dyn RenderImage>>,
    pub content_canvases: HashMap<u32, RenderCanvasHandle>,
    pub ore_reals: Vec<AnyResourceHandle>,
    pub segments: Vec<DeferredSegment>,
}
pub fn snapshot_frame(session: &mut DeferredSession) -> DeferredFrame {
    session.close_open_range();
    let segments = session.scheduler_segments();
    let buffer = session.command_buffer();
    let buffer = buffer.lock().unwrap();
    let ore = session.ore_context.borrow();
    let stream = ore.stream();
    let stream = stream.borrow();
    let ore_reals = ore.realResources().to_vec();
    DeferredFrame {
        commands: buffer.command_bytes().to_vec(),
        blobs: buffer.blob_bytes().to_vec(),
        ore_commands: stream.command_bytes().to_vec(),
        ore_blobs: stream.blob_bytes().to_vec(),
        canvas_images: session.canvases().borrow().images().to_vec(),
        content_canvases: session.content_canvases(),
        ore_reals,
        segments,
    }
}
pub fn take_frame(session: &mut DeferredSession) -> DeferredFrame {
    let frame = snapshot_frame(session);
    session.reset_frame();
    frame
}

#[derive(Default)]
pub struct DeferredReplayer {
    pub table: ResourceTable,
    pub ore: OreResident,
    stats: ReplayStats,
}
impl DeferredReplayer {
    pub fn reset(&mut self) {
        self.table = ResourceTable::default();
        self.ore = OreResident::default();
    }
    pub fn gpu_census(&self) -> GpuCensus {
        take_gpu_census(&self.table, &self.ore)
    }
    pub fn dropped_draws(&self) -> u32 {
        self.stats.dropped_draws
    }
    pub fn replay_session(
        &mut self,
        session: &mut DeferredSession,
        sink: &mut dyn DeferredFrameSink,
    ) {
        session.close_open_range();
        let segments = session.scheduler_segments();
        let buffer = session.command_buffer();
        let buffer = buffer.lock().unwrap();
        let ore = session.ore_context.borrow();
        let stream = ore.stream();
        let stream = stream.borrow();
        let reals = ore.realResources();
        self.replay(
            buffer.command_bytes(),
            buffer.blob_bytes(),
            stream.command_bytes(),
            stream.blob_bytes(),
            &mut |id| session.canvas_image_at(id),
            &mut |id| session.content_canvas_at(id),
            &reals,
            sink,
            &segments,
        );
    }
    pub fn replay_frame(&mut self, frame: &DeferredFrame, sink: &mut dyn DeferredFrameSink) {
        self.replay(
            &frame.commands,
            &frame.blobs,
            &frame.ore_commands,
            &frame.ore_blobs,
            &mut |id| frame.canvas_images.get(id as usize).cloned(),
            &mut |id| frame.content_canvases.get(&id).cloned(),
            &frame.ore_reals,
            sink,
            &frame.segments,
        );
    }
    fn replay(
        &mut self,
        commands: &[u8],
        blobs: &[u8],
        ore_commands: &[u8],
        ore_blobs: &[u8],
        canvas_image: &mut dyn FnMut(u32) -> Option<Rc<dyn RenderImage>>,
        content_canvas: &mut dyn FnMut(u32) -> Option<RenderCanvasHandle>,
        ore_reals: &[AnyResourceHandle],
        sink: &mut dyn DeferredFrameSink,
        segments: &[DeferredSegment],
    ) {
        self.stats = ReplayStats::default();
        self.table.clear_version_aliases();
        // The proxy borrows only for individual Factory operations, leaving
        // sink callbacks free to access the same retained concrete factory.
        let mut factory = sink.factory();
        let sink = RefCell::new(sink);
        let content_canvas = RefCell::new(content_canvas);
        let open_content = RefCell::new((INVALID_RENDER_HANDLE, None::<RendererOwner>));
        let mut hooks = ReplayHooks {
            filter: ReplayFilter::Resources,
            canvas_image: Some(Box::new(canvas_image)),
            stats: Some(&mut self.stats),
            begin_canvas_content: Some(Box::new(|id, clear_color| {
                let mut open = open_content.borrow_mut();
                if id == open.0 {
                    return open.1.clone();
                }
                open.1 = content_canvas.borrow_mut()(id).and_then(|canvas| {
                    // A deferred canvas has no backing until the replaying
                    // context gives it one, and this is the first operation
                    // that renders into it.
                    canvas.borrow_mut().ensure_backing();
                    sink.borrow_mut().begin_canvas_content(canvas, clear_color)
                });
                open.0 = id;
                open.1.clone()
            })),
        };
        replay_render_commands(
            &mut factory,
            None,
            commands,
            blobs,
            &mut self.table,
            &mut hooks,
        );
        hooks.filter = ReplayFilter::Draws;
        let mut screens = Vec::new();
        let mut canvas_ranges: HashMap<u64, Vec<&DeferredSegment>> = HashMap::new();
        for segment in segments {
            if segment.target == SegmentTarget::Screen {
                screens.push(segment);
            } else {
                canvas_ranges
                    .entry(segment.target_id)
                    .or_default()
                    .push(segment);
            }
        }
        let schedule = schedule_canvases(commands, segments);
        if schedule.had_cycle {
            static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            if COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 120 == 0 {
                eprintln!(
                    "rive deferred: canvas sample cycle, the back edge samples the previous frame"
                );
            }
        }
        for canvas_id in schedule.order {
            for segment in &canvas_ranges[&canvas_id] {
                replay_render_commands(
                    &mut factory,
                    None,
                    &commands[segment.begin as usize..segment.end as usize],
                    blobs,
                    &mut self.table,
                    &mut hooks,
                );
            }
            if open_content.borrow().1.is_some() {
                sink.borrow_mut().end_canvas_content();
            }
            *open_content.borrow_mut() = (INVALID_RENDER_HANDLE, None);
        }
        let mut ore_replayed = false;
        let mut open_screens: HashMap<u64, Option<RendererOwner>> = HashMap::new();
        for segment in screens {
            let screen = open_screen_and_ore(
                segment.target_id,
                &sink,
                &mut open_screens,
                &mut ore_replayed,
                ore_commands,
                ore_blobs,
                &mut self.ore,
                &self.table,
                ore_reals,
                &content_canvas,
            );
            if let Some(screen) = screen {
                replay_render_commands(
                    &mut factory,
                    Some(screen.borrow_mut().as_mut()),
                    &commands[segment.begin as usize..segment.end as usize],
                    blobs,
                    &mut self.table,
                    &mut hooks,
                );
            } else {
                replay_render_commands(
                    &mut factory,
                    None,
                    &commands[segment.begin as usize..segment.end as usize],
                    blobs,
                    &mut self.table,
                    &mut hooks,
                );
            }
        }
        if open_screens.is_empty() && (!ore_commands.is_empty() || !canvas_ranges.is_empty()) {
            let target = sink.borrow_mut().default_screen_target();
            open_screen_and_ore(
                target,
                &sink,
                &mut open_screens,
                &mut ore_replayed,
                ore_commands,
                ore_blobs,
                &mut self.ore,
                &self.table,
                ore_reals,
                &content_canvas,
            );
        }
        hooks.filter = ReplayFilter::Destroys;
        replay_render_commands(
            &mut factory,
            None,
            commands,
            blobs,
            &mut self.table,
            &mut hooks,
        );
    }
}
fn open_screen_and_ore(
    target: u64,
    sink: &RefCell<&mut dyn DeferredFrameSink>,
    screens: &mut HashMap<u64, Option<RendererOwner>>,
    ore_replayed: &mut bool,
    commands: &[u8],
    blobs: &[u8],
    ore: &mut OreResident,
    table: &ResourceTable,
    reals: &[AnyResourceHandle],
    content_canvas: &RefCell<&mut dyn FnMut(u32) -> Option<RenderCanvasHandle>>,
) -> Option<RendererOwner> {
    let screen = screens
        .entry(target)
        .or_insert_with(|| sink.borrow_mut().begin_screen_frame(target))
        .clone();
    if !*ore_replayed {
        if !commands.is_empty() {
            let real = sink.borrow_mut().ore_context();
            if let Some(real) = real {
                sink.borrow_mut().begin_ore_frame();
                replayOreStream(
                    &mut *real.borrow_mut(),
                    commands,
                    blobs,
                    ore,
                    &mut |id| reals.get((id & 0x7fffffff) as usize).cloned(),
                    &mut |id| {
                        content_canvas.borrow_mut()(id).map(|canvas| canvas_texture_info(&canvas))
                    },
                    &mut |id| {
                        table
                            .images
                            .get(id)
                            .and_then(|image| image.ore_texture_info())
                    },
                );
                sink.borrow_mut().end_ore_frame();
                sink.borrow_mut().after_ore_frame();
            } else {
                static COUNT: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(0);
                if COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 120 == 0 {
                    eprintln!(
                        "rive deferred: no ore context, dropping {} ore command bytes (canvas content will be lost)",
                        commands.len()
                    );
                }
            }
        }
        *ore_replayed = true;
    }
    screen
}
