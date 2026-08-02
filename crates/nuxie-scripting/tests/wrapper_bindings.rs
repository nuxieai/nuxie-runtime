#![cfg(feature = "luau")]

use nuxie_scripting::vm::ScriptVm;

#[test]
fn upstream_mesh_construction_case_is_available() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let result: String = vm
        .eval(
            r#"
            local vertices = VertexBuffer()
            vertices:add(Vector.xy(0, 0), Vector.xy(1, 0), Vector.xy(1, 1))
            local uvs = VertexBuffer()
            uvs:add(Vector.xy(0, 0))
            uvs:add(Vector.xy(1, 0))
            uvs:add(Vector.xy(1, 1))
            local indices = TriangleBuffer()
            indices:add(0, 1, 2)
            return type(indices)
            "#,
        )
        .unwrap();

    assert_eq!(result, "userdata");
}

#[test]
fn upstream_image_sampler_construction_case_is_available() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let result: String = vm
        .eval("return type(ImageSampler('clamp', 'mirror', 'nearest'))")
        .unwrap();
    assert_eq!(result, "userdata");

    let linear = vm.eval::<()>("ImageSampler('clamp', 'clamp', 'linear')");
    assert!(
        linear
            .unwrap_err()
            .to_string()
            .contains("not a valid ImageFilter")
    );
    vm.eval::<()>("ImageSampler('repeat', 'mirror', 'bilinear')")
        .unwrap();
}

#[test]
fn triangle_indices_are_bounded_to_the_pinned_u16_surface() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let error = vm
        .eval::<()>("local indices = TriangleBuffer(); indices:add(0, 1, 65536)")
        .unwrap_err()
        .to_string();
    assert!(error.contains("index 65536 exceeds 65535"), "{error}");
}
