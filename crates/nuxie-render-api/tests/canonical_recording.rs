use nuxie_render_api::{
    BlendMode, CanonicalRecording, Factory, ImageSampler, RecordingFactory, RenderBufferFlags,
    RenderBufferType, Renderer,
};

#[derive(Clone, Copy)]
struct AllocatorOffsets {
    paths: usize,
    paints: usize,
    shaders: usize,
    images: usize,
    buffers: usize,
}

fn record_mesh_scene(
    allocator_offsets: AllocatorOffsets,
) -> (String, nuxie_render_api::CanonicalRecording) {
    let mut factory = RecordingFactory::new();

    for _ in 0..allocator_offsets.paths {
        drop(factory.make_empty_render_path());
    }
    for _ in 0..allocator_offsets.paints {
        drop(factory.make_render_paint());
    }
    for _ in 0..allocator_offsets.shaders {
        drop(factory.make_linear_gradient(0.0, 0.0, 1.0, 1.0, &[0xff000000], &[0.0]));
    }
    for _ in 0..allocator_offsets.images {
        drop(factory.decode_image(&[0]));
    }
    for _ in 0..allocator_offsets.buffers {
        drop(factory.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 1));
    }
    factory.clear();

    let shader =
        factory.make_linear_gradient(0.0, 0.0, 10.0, 10.0, &[0xff112233, 0xffaabbcc], &[0.0, 1.0]);
    let mut paint = factory.make_render_paint();
    paint.shader(Some(shader.as_ref()));

    let mut path = factory.make_empty_render_path();
    path.move_to(0.0, 0.0);
    path.line_to(10.0, 0.0);
    path.line_to(10.0, 10.0);
    path.close();

    let image = factory.decode_image(&[1, 2, 3]);
    let mut vertices = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        4,
    );
    vertices.map_mut().copy_from_slice(&[1, 2, 3, 4]);
    vertices.unmap();
    let mut uvs = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        4,
    );
    uvs.map_mut().copy_from_slice(&[5, 6, 7, 8]);
    uvs.unmap();
    let mut indices = factory.make_render_buffer(
        RenderBufferType::Index,
        RenderBufferFlags::MappedOnceAtInitialization,
        2,
    );
    indices.map_mut().copy_from_slice(&[0, 1]);
    indices.unmap();

    let mut renderer = factory.make_renderer();
    renderer.draw_path(path.as_ref(), paint.as_ref());
    renderer.clip_path(path.as_ref());
    renderer.draw_image(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    renderer.draw_image_mesh(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(vertices.as_ref()),
        Some(uvs.as_ref()),
        Some(indices.as_ref()),
        3,
        3,
        BlendMode::SrcOver,
        1.0,
    );

    (factory.stream(), factory.canonical_recording())
}

#[test]
fn canonical_recordings_ignore_allocator_ids_across_all_resource_categories() {
    let (raw_a, canonical_a) = record_mesh_scene(AllocatorOffsets {
        paths: 0,
        paints: 0,
        shaders: 0,
        images: 0,
        buffers: 0,
    });
    let (raw_b, canonical_b) = record_mesh_scene(AllocatorOffsets {
        paths: 7,
        paints: 2,
        shaders: 5,
        images: 11,
        buffers: 3,
    });

    assert_ne!(raw_a, raw_b);
    assert_eq!(canonical_a.stream(), canonical_b.stream());
    assert_eq!(canonical_a.fnv1a64(), canonical_b.fnv1a64());
    assert_eq!(
        canonical_a.fnv1a64_hex(),
        format!("{:016x}", canonical_a.fnv1a64())
    );
}

fn record_path_and_paint_relationship(distinct_second: bool) -> CanonicalRecording {
    let mut factory = RecordingFactory::new();
    let mut path_a = factory.make_empty_render_path();
    let mut path_b = factory.make_empty_render_path();
    for path in [&mut path_a, &mut path_b] {
        path.move_to(0.0, 0.0);
        path.line_to(1.0, 1.0);
    }
    let paint_a = factory.make_render_paint();
    let paint_b = factory.make_render_paint();

    let mut renderer = factory.make_renderer();
    renderer.draw_path(path_a.as_ref(), paint_a.as_ref());
    if distinct_second {
        renderer.draw_path(path_b.as_ref(), paint_b.as_ref());
    } else {
        renderer.draw_path(path_a.as_ref(), paint_a.as_ref());
    }
    factory.canonical_recording()
}

fn record_shader_relationship(distinct_second: bool) -> CanonicalRecording {
    let mut factory = RecordingFactory::new();
    let shader_a = factory.make_linear_gradient(0.0, 0.0, 1.0, 1.0, &[0xff112233], &[0.0]);
    let shader_b = factory.make_linear_gradient(0.0, 0.0, 1.0, 1.0, &[0xff112233], &[0.0]);
    let mut paint_a = factory.make_render_paint();
    let mut paint_b = factory.make_render_paint();
    paint_a.shader(Some(shader_a.as_ref()));
    paint_b.shader(Some(if distinct_second {
        shader_b.as_ref()
    } else {
        shader_a.as_ref()
    }));
    let path = factory.make_empty_render_path();

    let mut renderer = factory.make_renderer();
    renderer.draw_path(path.as_ref(), paint_a.as_ref());
    renderer.draw_path(path.as_ref(), paint_b.as_ref());
    factory.canonical_recording()
}

fn record_image_relationship(distinct_second: bool) -> CanonicalRecording {
    let mut factory = RecordingFactory::new();
    let image_a = factory.decode_image(&[1, 2, 3]);
    let image_b = factory.decode_image(&[1, 2, 3]);

    let mut renderer = factory.make_renderer();
    renderer.draw_image(
        Some(image_a.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    renderer.draw_image(
        Some(if distinct_second {
            image_b.as_ref()
        } else {
            image_a.as_ref()
        }),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    factory.canonical_recording()
}

fn record_buffer_relationship(distinct_uvs: bool) -> CanonicalRecording {
    let mut factory = RecordingFactory::new();
    let mut buffer_a = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        4,
    );
    buffer_a.map_mut().copy_from_slice(&[1, 2, 3, 4]);
    buffer_a.unmap();
    let mut buffer_b = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        4,
    );
    buffer_b.map_mut().copy_from_slice(&[1, 2, 3, 4]);
    buffer_b.unmap();

    let mut renderer = factory.make_renderer();
    renderer.draw_image_mesh(
        None,
        ImageSampler::LINEAR_CLAMP,
        Some(buffer_a.as_ref()),
        Some(if distinct_uvs {
            buffer_b.as_ref()
        } else {
            buffer_a.as_ref()
        }),
        None,
        2,
        0,
        BlendMode::SrcOver,
        1.0,
    );
    factory.canonical_recording()
}

fn assert_relationship_differs(left: CanonicalRecording, right: CanonicalRecording) {
    assert_ne!(left.stream(), right.stream());
    assert_ne!(left.fnv1a64(), right.fnv1a64());
}

#[test]
fn canonical_recordings_preserve_distinct_resource_relationships() {
    assert_relationship_differs(
        record_path_and_paint_relationship(false),
        record_path_and_paint_relationship(true),
    );
    assert_relationship_differs(
        record_shader_relationship(false),
        record_shader_relationship(true),
    );
    assert_relationship_differs(
        record_image_relationship(false),
        record_image_relationship(true),
    );
    assert_relationship_differs(
        record_buffer_relationship(false),
        record_buffer_relationship(true),
    );
}

#[test]
fn canonical_recording_is_identical_before_and_after_render_cache_reuse() {
    const IMAGE_DATA: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0,
        0, 2, 0, 0, 0, 3,
    ];
    let mut factory = RecordingFactory::new();
    let shader =
        factory.make_radial_gradient(5.0, 6.0, 7.0, &[0xff112233, 0xffaabbcc], &[0.25, 0.75]);
    let mut paint = factory.make_render_paint();
    paint.shader(Some(shader.as_ref()));
    let mut path = factory.make_empty_render_path();
    path.move_to(1.0, 2.0);
    path.line_to(3.0, 4.0);
    let image = factory.decode_image(IMAGE_DATA);

    let mut vertices = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        4,
    );
    vertices.map_mut().copy_from_slice(&[1, 2, 3, 4]);
    vertices.unmap();
    let mut uvs = factory.make_render_buffer(RenderBufferType::Vertex, RenderBufferFlags::None, 4);
    uvs.map_mut().copy_from_slice(&[5, 6, 7, 8]);
    uvs.unmap();
    let mut indices = factory.make_render_buffer(
        RenderBufferType::Index,
        RenderBufferFlags::MappedOnceAtInitialization,
        2,
    );
    indices.map_mut().copy_from_slice(&[0, 1]);
    indices.unmap();

    let mut renderer = factory.make_renderer();
    renderer.draw_path(path.as_ref(), paint.as_ref());
    renderer.draw_image(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        0.75,
    );
    renderer.draw_image_mesh(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(vertices.as_ref()),
        Some(uvs.as_ref()),
        Some(indices.as_ref()),
        2,
        2,
        BlendMode::SrcOver,
        1.0,
    );
    let cold_raw = factory.stream();
    let cold = factory.canonical_recording();

    factory.clear();
    renderer.draw_path(path.as_ref(), paint.as_ref());
    renderer.draw_image(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        0.75,
    );
    renderer.draw_image_mesh(
        Some(image.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        Some(vertices.as_ref()),
        Some(uvs.as_ref()),
        Some(indices.as_ref()),
        2,
        2,
        BlendMode::SrcOver,
        1.0,
    );
    let warm_raw = factory.stream();
    let warm = factory.canonical_recording();

    assert_ne!(cold_raw, warm_raw);
    assert!(cold_raw.starts_with("rive-golden-stream-v1\n"));
    assert!(warm_raw.starts_with("rive-golden-stream-v1\n"));
    assert_eq!(cold.stream(), warm.stream());
    assert!(cold.stream().starts_with("nuxie-canonical-recording-v1\n"));
    assert_eq!(cold.fnv1a64(), warm.fnv1a64());
    assert_eq!(cold.fnv1a64_hex(), warm.fnv1a64_hex());
    assert!(
        cold.stream()
            .contains("path={id=1,fillRule=0,path={verbs=[move,line],points=[(1,2),(3,4)]}}")
    );
    assert!(cold.stream().contains(
        "shader={id=1,kind=radial,center=(5,6),radius=7,stops=[{color=0xff112233,stop=0.25},{color=0xffaabbcc,stop=0.75}]}"
    ));
    assert!(cold.stream().contains(
        "image={id=1,width=2,height=3,data=89504e470d0a1a0a0000000d494844520000000200000003}"
    ));
    assert!(
        cold.stream()
            .contains("vertices={id=1,type=1,flags=1,size=4,data=01020304}")
    );
    assert!(
        cold.stream()
            .contains("uvs={id=2,type=1,flags=0,size=4,data=05060708}")
    );
    assert!(
        cold.stream()
            .contains("indices={id=3,type=0,flags=1,size=2,data=0001}")
    );
}
