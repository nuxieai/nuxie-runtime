//! Complete tests/gm/ore_vertex_buffer_before_pipeline.cpp at upstream 5705446d.
use super::ore_gm_helper::*;

#[test]
fn ore_vertex_buffer_before_pipeline() {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    let vertices: Vec<u8> = [
        [0.0f32, 0.5, 1.0, 0.0, 0.0, 1.0],
        [-0.5, -0.5, 0.0, 1.0, 0.0, 1.0],
        [0.5, -0.5, 0.0, 0.0, 1.0, 1.0],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_ne_bytes)
    .collect();
    let vbo = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: vertices.len() as u32,
            data: Some(&vertices),
            label: Some("ore_vb_before_pipeline_vbo"),
            immutable: false,
        })
        .expect("vertex buffer");
    let unused = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: 16,
            data: Some(&[0; 16]),
            label: Some("ore_vb_before_pipeline_unused_slot"),
            immutable: false,
        })
        .expect("unused vertex buffer");
    let a = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_vb_before_pipeline_a",
    );
    let b = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_vb_before_pipeline_b",
    );
    host.begin_ore();
    let desc = pass_desc(
        &target,
        Some("ore_vertex_buffer_before_pipeline_pass"),
        [0.1, 0.1, 0.1, 1.0],
    );
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("ORE pass");
    pass.setVertexBuffer(0, Some(&vbo), 0);
    pass.setVertexBuffer(1, Some(&unused), 0);
    pass.setPipeline(Some(&a));
    pass.setViewport(0.0, 64.0, 128.0, 128.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.setPipeline(Some(&b));
    pass.setViewport(128.0, 64.0, 128.0, 128.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    assert_cpp_gm_pixels("ore_vertex_buffer_before_pipeline", host.finish());
}
