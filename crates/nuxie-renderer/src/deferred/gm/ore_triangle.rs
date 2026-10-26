//! tests/gm/ore_triangle.cpp through upstream 4252d730: immutable initial upload.
use super::ore_gm_helper::*;

#[test]
fn ore_triangle_immutable_upload() {
    let mut host = GmHost::new(0xff000000);
    let canvas = host.canvas(256, 256);
    let target = wrap_canvas(&mut *host.ore.borrow_mut(), &canvas);
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
            immutable: true,
            label: Some("ore_triangle_vbo"),
        })
        .expect("immutable vertex buffer");
    let module = shader(&mut *host.ore.borrow_mut(), 0);
    let pipeline = triangle_pipeline(
        &mut *host.ore.borrow_mut(),
        &module,
        TextureFormat::rgba8unorm,
        "ore_triangle_pipeline",
    );
    host.begin_ore();
    let desc = pass_desc(&target, Some("ore_triangle_pass"), [0.1, 0.1, 0.1, 1.0]);
    let mut pass = host
        .ore
        .borrow_mut()
        .beginRenderPass(&desc, None)
        .expect("ORE pass");
    pass.setPipeline(Some(&pipeline));
    pass.setVertexBuffer(0, Some(&vbo), 0);
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.finish();
    host.end_ore();
    draw_canvas_at_origin(host.screen().borrow_mut().as_mut(), &canvas);
    // Both upstream GMs draw precisely the same vertices, colors, background,
    // shader and viewport. Reuse the existing C++ offset-witness capture as
    // the exact expected image; this does not claim a new C++ capture.
    assert_cpp_gm_pixels("ore_buffer_offsets", host.finish());
}
