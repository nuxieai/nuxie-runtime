//! Complete tests/gm/ore_pipeline_switch.cpp at upstream 5705446d.
use super::ore_gm_helper::*;

#[test]
fn ore_pipeline_switch() {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    // The last zero vertex preserves the full doubled stride, as upstream.
    let vertices: Vec<u8> = [
        [-0.5f32, 0.5, 1.0, 0.0, 0.0, 1.0],
        [-0.9, -0.5, 0.0, 1.0, 0.0, 1.0],
        [-0.1, -0.5, 0.0, 0.0, 1.0, 1.0],
        [0.0; 6],
        [0.0; 6],
        [0.0; 6],
        [0.5, 0.5, 1.0, 1.0, 0.0, 1.0],
        [0.0; 6],
        [0.1, -0.5, 0.0, 1.0, 1.0, 1.0],
        [0.0; 6],
        [0.9, -0.5, 1.0, 0.0, 1.0, 1.0],
        [0.0; 6],
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
            label: Some("ore_pipeline_switch_vbo"),
            immutable: false,
        })
        .expect("vertex buffer");
    let single = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_pipeline_switch_1x",
    );
    let twice = triangle_pipeline_with_stride(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_pipeline_switch_2x",
        2 * 24,
    );
    host.begin_ore();
    let desc = pass_desc(
        &target,
        Some("ore_pipeline_switch_pass"),
        [0.1, 0.1, 0.1, 1.0],
    );
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("ORE pass");
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    pass.setPipeline(Some(&single));
    pass.setVertexBuffer(0, Some(&vbo), 0);
    pass.draw(3, 1, 0, 0);
    pass.setPipeline(Some(&twice));
    pass.draw(3, 1, 3, 0);
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    assert_cpp_gm_pixels("ore_pipeline_switch", host.finish());
}
