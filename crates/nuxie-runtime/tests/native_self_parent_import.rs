//! Import regression for the pinned C++ self-parent lifecycle behavior.
//!
//! This minimized file contains one ContourMeshVertex whose local parent id is
//! its own id. The pinned C++ runtime accepts it, adds the vertex to its own
//! ContainerComponent children, then returns MissingObject from
//! MeshVertex::onAddedDirty because that parent is not a Mesh.

use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::artboard::Artboard;
use nuxie_runtime::{File, RuntimeFactoryHandle};

const SELF_PARENT_CONTOUR_MESH_VERTEX: &[u8] =
    b"RIVE\x07\x00\x00\x02\x00\x00\x00\x00\x00\x17\x00\x01\x95\x01\x00\x00ound\x05\x00\x00\x12\x05\x01ctana\x95\x01\x00\x00";

#[test]
fn self_parented_contour_vertex_matches_pinned_cpp_initialization() {
    let mut factory = PersistentFactory::new(NullFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(SELF_PARENT_CONTOUR_MESH_VERTEX, factory, None, None, None)
        .expect("pinned C++ accepts the minimized file");

    let source = file
        .with_file(|file| file.artboard_at_source(0))
        .expect("source Artboard");
    let vertex = source
        .with_downcast::<Artboard, _>(|artboard| artboard.resolve_handle(1))
        .flatten()
        .expect("ContourMeshVertex local 1");

    vertex
        .with(|object| {
            let component = object.as_component().expect("vertex Component");
            assert_eq!(component.parent_handle(), Some(vertex.clone()));
            assert_eq!(
                object
                    .as_container_component()
                    .expect("vertex ContainerComponent")
                    .children(),
                &[vertex.clone()],
            );
        })
        .expect("live vertex occurrence");

    // The fuzz target continues through source-to-instance cloning. Preserve
    // that part of the reproducer as well: a self child must not make the
    // accepted source graph uninstantiable.
    file.with_file(|file| file.artboard_default())
        .expect("default Artboard instance");
}
