//! All three deferred_mesh_instanced_test.cpp cases at upstream 4921ab81.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::*;
use nuxie_render_api::serialized_replay::{replay_serialized_commands, SerializedReplayHooks};

const INSTANCE_COUNT: usize = 3;

#[derive(Default)]
struct MeshSink {
    instanced_draws: usize,
    plain_draws: usize,
    frames: Vec<Vec<ImageMeshInstanceData>>,
}
impl Renderer for MeshSink {
    fn modulate_opacity(&mut self, _: f32) {}
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
        self.plain_draws += 1;
    }
    fn draw_image_mesh_instanced(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        instances: Option<&ImageMeshInstancesHandle>,
    ) {
        self.instanced_draws += 1;
        self.frames
            .push(instances.unwrap().borrow().instance_data().to_vec());
    }
}

struct Mesh {
    image: Box<dyn RenderImage>,
    points: Box<dyn RenderBuffer>,
    uvs: Box<dyn RenderBuffer>,
    indices: Box<dyn RenderBuffer>,
}
impl Mesh {
    fn make(factory: &mut dyn Factory) -> Self {
        fn buffer(
            factory: &mut dyn Factory,
            kind: RenderBufferType,
            bytes: &[u8],
        ) -> Box<dyn RenderBuffer> {
            let mut b = factory.make_render_buffer(kind, RenderBufferFlags::None, bytes.len());
            b.map_mut().copy_from_slice(bytes);
            b.unmap();
            b
        }
        let quad: Vec<u8> = [0.0f32, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        let uv: Vec<u8> = [0.0f32, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        let idx: Vec<u8> = [0u16, 1, 2, 0, 2, 3]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();
        Self {
            image: factory
                .decode_image(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../fixtures/renderer/image_paint/nomoon.png"
                )))
                .unwrap(),
            points: buffer(factory, RenderBufferType::Vertex, &quad),
            uvs: buffer(factory, RenderBufferType::Vertex, &uv),
            indices: buffer(factory, RenderBufferType::Index, &idx),
        }
    }
    fn draw(&self, renderer: &mut dyn Renderer, instances: &ImageMeshInstancesHandle) {
        renderer.draw_image_mesh_instanced(
            Some(self.image.as_ref()),
            ImageSampler::default(),
            Some(self.points.as_ref()),
            Some(self.uvs.as_ref()),
            Some(self.indices.as_ref()),
            4,
            6,
            Some(instances),
        );
    }
}
fn replay_frame(
    session: &mut DeferredSession,
    replayer: &mut DeferredReplayer,
    sink: &mut TestSink,
) {
    let frame = take_frame(session);
    replayer.replay_frame(&frame, sink);
}
fn read_back(sink: &TestSink) -> MeshSink {
    let mut out = MeshSink::default();
    let mut discard = SerializingFactory::new();
    assert!(replay_serialized_commands(
        &sink.factory.borrow().bytes(),
        &mut discard,
        &mut out,
        &mut SerializedReplayHooks::default()
    ));
    out
}

#[test]
fn an_instanced_draw_survives_the_deferred_round_trip() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let mesh = Mesh::make(&mut session);
    let instances = session.make_image_mesh_instances(INSTANCE_COUNT);
    {
        let mut instances = instances.borrow_mut();
        let data = instances.edit(None);
        data[0].transform = Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
        data[1].opacity = 0.5;
        data[2].uv_scale = [0.5, 0.25];
        data[2].additiveness = 1.0;
        instances.end_edit();
    }
    mesh.draw(session.screen_renderer(0).borrow_mut().as_mut(), &instances);
    replay_frame(&mut session, &mut replayer, &mut sink);
    assert_eq!(replayer.dropped_draws(), 0);
    let out = read_back(&sink);
    assert_eq!(out.instanced_draws, 1);
    assert_eq!(out.plain_draws, 0);
    assert_eq!(out.frames.len(), 1);
    assert_eq!(out.frames[0].len(), INSTANCE_COUNT);
    assert_eq!(out.frames[0][0].transform.0[4], 10.0);
    assert_eq!(out.frames[0][0].transform.0[5], 20.0);
    assert_eq!(out.frames[0][1].opacity, 0.5);
    assert_eq!(out.frames[0][2].uv_scale[0], 0.5);
    assert_eq!(out.frames[0][2].uv_scale[1], 0.25);
    assert_eq!(out.frames[0][2].additiveness, 1.0);
}

#[test]
fn an_in_place_instance_update_reaches_the_next_frame() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let mesh = Mesh::make(&mut session);
    let instances = session.make_image_mesh_instances(INSTANCE_COUNT);
    for opacity in [0.25, 0.75] {
        instances.borrow_mut().edit(None)[0].opacity = opacity;
        instances.borrow_mut().end_edit();
        mesh.draw(session.screen_renderer(0).borrow_mut().as_mut(), &instances);
        replay_frame(&mut session, &mut replayer, &mut sink);
    }
    let out = read_back(&sink);
    assert_eq!(out.frames.len(), 2);
    assert_eq!(out.frames[0][0].opacity, 0.25);
    assert_eq!(out.frames[1][0].opacity, 0.75);
}

#[test]
fn dropping_instances_frees_the_replayers_copy() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let mesh = Mesh::make(&mut session);
    let instances = session.make_image_mesh_instances(INSTANCE_COUNT);
    instances.borrow_mut().edit(None);
    instances.borrow_mut().end_edit();
    mesh.draw(session.screen_renderer(0).borrow_mut().as_mut(), &instances);
    replay_frame(&mut session, &mut replayer, &mut sink);
    assert_eq!(replayer.gpu_census().image_mesh_instances, 1);
    drop(instances);
    let pending = session.take_pending_destroys();
    assert!(!pending.empty());
    replayer.replay_destroys(&pending.commands, &pending.ore_commands);
    assert_eq!(replayer.gpu_census().image_mesh_instances, 0);
}
