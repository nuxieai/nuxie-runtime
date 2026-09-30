//! Complete tests/gm/mesh_instanced.cpp scene at upstream 4921ab81.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_render_factory::DeferredFactory,
    render_replay::{replay_render_commands, ReplayHooks, ResourceTable},
};
use nuxie_runtime::source::math::mat2d::Mat2D as SourceMat2D;

const QUAD_SIZE: i32 = 120;
const COLUMNS: usize = 4;
const ROWS: usize = 4;
const PAD: i32 = 10;
const WIDTH: u32 = COLUMNS as u32 * (QUAD_SIZE + PAD) as u32 + PAD as u32;
const HEIGHT: u32 = ROWS as u32 * (QUAD_SIZE + PAD) as u32 + PAD as u32;

struct QuadMesh {
    points: Box<dyn RenderBuffer>,
    uvs: Box<dyn RenderBuffer>,
    indices: Box<dyn RenderBuffer>,
    vertex_count: u32,
    index_count: u32,
}
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
fn make_quad(factory: &mut dyn Factory) -> QuadMesh {
    let size = QUAD_SIZE as f32;
    let points: Vec<u8> = [0.0f32, 0.0, size, 0.0, size, size, 0.0, size]
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
    QuadMesh {
        points: buffer(factory, RenderBufferType::Vertex, &points),
        uvs: buffer(factory, RenderBufferType::Vertex, &uvs),
        indices: buffer(factory, RenderBufferType::Index, &indices),
        vertex_count: 4,
        index_count: 6,
    }
}
fn cell_transform(column: usize, row: usize) -> SourceMat2D {
    SourceMat2D::from_translate(
        (PAD + column as i32 * (QUAD_SIZE + PAD)) as f32,
        (PAD + row as i32 * (QUAD_SIZE + PAD)) as f32,
    )
}
fn draw_scene(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    let Ok(image) = factory.decode_image(super::additive_blend::NOMOON) else {
        return;
    };
    let mesh = make_quad(factory);
    let instances = factory.make_image_mesh_instances(COLUMNS * ROWS);
    {
        let mut instances = instances.borrow_mut();
        let data = instances.edit(None);
        for column in 0..COLUMNS {
            let t = column as f32 / (COLUMNS - 1) as f32;
            let spin = SourceMat2D::from_translate(QUAD_SIZE as f32 * 0.5, QUAD_SIZE as f32 * 0.5)
                * SourceMat2D::from_rotation(t * std::f32::consts::PI * 0.5)
                * SourceMat2D::from_translate(QUAD_SIZE as f32 * -0.5, QUAD_SIZE as f32 * -0.5);
            data[column].transform = Mat2D(*(cell_transform(column, 0) * spin).values());
            data[COLUMNS + column].transform = Mat2D(*cell_transform(column, 1).values());
            data[COLUMNS + column].opacity = 1.0 - 0.25 * column as f32;
            data[COLUMNS * 2 + column].transform = Mat2D(*cell_transform(column, 2).values());
            data[COLUMNS * 2 + column].uv_scale = [0.5, 0.5];
            data[COLUMNS * 2 + column].uv_translate = [
                if column & 1 != 0 { 0.5 } else { 0.0 },
                if column & 2 != 0 { 0.5 } else { 0.0 },
            ];
            data[COLUMNS * 3 + column].transform = Mat2D(*cell_transform(column, 3).values());
            data[COLUMNS * 3 + column].additiveness = t;
        }
        instances.end_edit();
    }
    renderer.draw_image_mesh_instanced(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(mesh.points.as_ref()),
        Some(mesh.uvs.as_ref()),
        Some(mesh.indices.as_ref()),
        mesh.vertex_count,
        mesh.index_count,
        Some(&instances),
    );
}
fn scene(deferred: bool) -> Vec<u8> {
    let mut host = GmHost::with_size(0xff808080, true, WIDTH, HEIGHT);
    let screen = host.screen();
    if deferred {
        let mut factory = DeferredFactory::new();
        let mut renderer = factory.make_renderer(None);
        draw_scene(&mut factory, &mut renderer);
        let buffer = factory.buffer.lock().unwrap();
        replay_render_commands(
            &mut host.factory,
            Some(screen.borrow_mut().as_mut()),
            buffer.command_bytes(),
            buffer.blob_bytes(),
            &mut ResourceTable::default(),
            &mut ReplayHooks::default(),
        );
    } else {
        draw_scene(&mut host.factory, screen.borrow_mut().as_mut());
    }
    host.finish()
}
#[test]
fn mesh_instanced() {
    let immediate = scene(false);
    let deferred = scene(true);
    assert_eq!(immediate.len(), (WIDTH * HEIGHT * 4) as usize);
    assert_eq!(deferred.len(), immediate.len());
    assert!(
        immediate == deferred,
        "mesh_instanced immediate/deferred pixels differ"
    );
    assert_cpp_gm_pixels_with_size("mesh_instanced", WIDTH, HEIGHT, immediate);
}
