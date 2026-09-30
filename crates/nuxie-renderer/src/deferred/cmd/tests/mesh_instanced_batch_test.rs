//! All mesh_instanced_batch_test.cpp cases at upstream 4921ab81.
//! Count actual source DrawBatch entries and instance elements on a null device.
use super::render_context_null::*;
use super::*;

const INSTANCE_COUNT: u32 = 5;

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
            let mut buffer = factory.make_render_buffer(
                kind,
                RenderBufferFlags::MappedOnceAtInitialization,
                bytes.len(),
            );
            buffer.map_mut().copy_from_slice(bytes);
            buffer.unmap();
            buffer
        }
        let points: Vec<u8> = [0.0f32, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0, 50.0]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        let uvs: Vec<u8> = [0.0f32, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        let indices: Vec<u8> = [0u16, 1, 2, 0, 2, 3]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();
        Self {
            image: factory
                .decode_image(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../fixtures/renderer/image_paint/nomoon.png"
                )))
                .expect("source image decodes"),
            points: buffer(factory, RenderBufferType::Vertex, &points),
            uvs: buffer(factory, RenderBufferType::Vertex, &uvs),
            indices: buffer(factory, RenderBufferType::Index, &indices),
        }
    }
    fn instanced(&self, renderer: &mut dyn Renderer, instances: &ImageMeshInstancesHandle) {
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
    fn plain(&self, renderer: &mut dyn Renderer) {
        renderer.draw_image_mesh(
            Some(self.image.as_ref()),
            ImageSampler::default(),
            Some(self.points.as_ref()),
            Some(self.uvs.as_ref()),
            Some(self.indices.as_ref()),
            4,
            6,
            BlendMode::SrcOver,
            1.0,
        );
    }
}

fn harness(draw: impl FnOnce(&mut dyn Renderer, &mut ObservingFactory, &Mesh)) -> FlushStats {
    let (mut factory, stats, _) = observing_factory(100, 100);
    let mut renderer = factory
        .borrow()
        .begin_frame(0, crate::RenderMode::RasterOrdering)
        .unwrap();
    let mesh = Mesh::make(&mut factory);
    draw(&mut renderer, &mut factory, &mesh);
    renderer.finish_without_readback().unwrap();
    let result = *stats.borrow();
    result
}

#[test]
fn an_instanced_mesh_draw_is_one_batch() {
    let stats = harness(|renderer, factory, mesh| {
        let instances = factory.make_image_mesh_instances(INSTANCE_COUNT as usize);
        {
            let mut instances = instances.borrow_mut();
            let data = instances.edit(None);
            for i in 0..INSTANCE_COUNT {
                data[i as usize].transform = Mat2D([1.0, 0.0, 0.0, 1.0, i as f32 * 10.0, 0.0]);
                data[i as usize].opacity = 1.0 - 0.1 * i as f32;
            }
            instances.end_edit();
        }
        mesh.instanced(renderer, &instances);
    });
    assert_eq!(stats.mesh_batches, 1);
    assert_eq!(stats.mesh_elements, INSTANCE_COUNT);
}

#[test]
fn an_empty_instance_array_draws_nothing() {
    let stats = harness(|renderer, factory, mesh| {
        let instances = factory.make_image_mesh_instances(0);
        instances.borrow_mut().edit(None);
        instances.borrow_mut().end_edit();
        mesh.instanced(renderer, &instances);
    });
    assert_eq!(stats.mesh_batches, 0);
    assert_eq!(stats.mesh_elements, 0);
}

#[test]
fn separate_mesh_draws_are_a_batch_each() {
    let stats = harness(|renderer, _, mesh| {
        for i in 0..INSTANCE_COUNT {
            renderer.save();
            renderer.transform(Mat2D([1.0, 0.0, 0.0, 1.0, i as f32 * 10.0, 0.0]));
            mesh.plain(renderer);
            renderer.restore();
        }
    });
    assert_eq!(stats.mesh_batches, INSTANCE_COUNT as usize);
    assert_eq!(stats.mesh_elements, INSTANCE_COUNT);
}
