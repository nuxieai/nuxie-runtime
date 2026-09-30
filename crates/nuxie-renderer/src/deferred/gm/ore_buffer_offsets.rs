//! Complete tests/gm/ore_buffer_offsets.cpp at upstream 5705446d.
use super::ore_gm_helper::*;

#[test]
fn ore_buffer_offsets() {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    let vertices: Vec<u8> = [
        [-2.0f32, -2.0, 1.0, 1.0, 1.0, 1.0],
        [-2.0, -2.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 0.5, 1.0, 0.0, 0.0, 1.0],
        [-0.5, -0.5, 0.0, 1.0, 0.0, 1.0],
        [0.5, -0.5, 0.0, 0.0, 1.0, 1.0],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_ne_bytes)
    .collect();
    let indices: Vec<u8> = [0u16, 0, 0, 0, 0, 1, 2, 0]
        .into_iter()
        .flat_map(u16::to_ne_bytes)
        .collect();
    let vbo = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::vertex,
            size: vertices.len() as u32,
            data: Some(&vertices),
            label: Some("ore_buffer_offsets_vbo"),
            immutable: false,
        })
        .expect("vertex buffer");
    let ibo = host
        .ore
        .borrow_mut()
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::index,
            size: indices.len() as u32,
            data: Some(&indices),
            label: Some("ore_buffer_offsets_ibo"),
            immutable: false,
        })
        .expect("index buffer");
    let pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_buffer_offsets_pipeline",
    );
    host.begin_ore();
    let desc = pass_desc(
        &target,
        Some("ore_buffer_offsets_pass"),
        [0.1, 0.1, 0.1, 1.0],
    );
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("ORE pass");
    pass.setPipeline(Some(&pipeline));
    pass.setVertexBuffer(0, Some(&vbo), 2 * 24);
    pass.setIndexBuffer(Some(&ibo), IndexFormat::uint16, 4 * 2);
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    pass.drawIndexed(3, 1, 0, 0, 0);
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    assert_cpp_gm_pixels("ore_buffer_offsets", host.finish());
}
