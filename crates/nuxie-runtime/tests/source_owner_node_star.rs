//! Regressions for complete Node/Star owners matched to upstream 160085c6.

use nuxie_runtime::source::{
    component::{ComponentDirt, ComponentOccurrenceHandle},
    core::CoreArena,
    generated::{core_registry::CoreRegistry, shapes::polygon_base::PolygonBase},
    shapes::{path::PathVertexOccurrence, star::Star},
};
use std::rc::Rc;

#[test]
fn star_vertex_count_multiplies_as_cpp_uint32_before_widening() {
    let mut star = Star::default();
    for (points, expected) in [
        (0, 0),
        (1, 2),
        (5, 10),
        (0x7fff_ffff, 0xffff_fffe),
        (0x8000_0000, 0),
        (0x8000_0001, 2),
        (u32::MAX, 0xffff_fffe),
    ] {
        CoreRegistry::set_uint(&mut star, PolygonBase::POINTS_PROPERTY_KEY.into(), points);
        assert_eq!(star.vertex_count(), expected, "points={points:#x}");
    }
}

#[test]
fn star_update_uses_the_inherited_polygon_storage_and_wrapped_count() {
    let arena = CoreArena::default();
    let owner = arena.insert(Star::default());
    for (points, expected) in [(5, 10), (3, 6), (0x8000_0002, 4), (0, 0), (4, 8)] {
        assert!(CoreRegistry::set_uint_handle(
            &owner,
            PolygonBase::POINTS_PROPERTY_KEY.into(),
            points,
        ));
        // Fail before allocating if the source uint32 wrap regresses.
        assert_eq!(
            owner.with_downcast::<Star, _>(Star::vertex_count),
            Some(expected),
            "points={points:#x}",
        );
        ComponentOccurrenceHandle::Authored(owner.clone()).update(ComponentDirt::PATH);
        owner
            .with_downcast::<Star, _>(|star| {
                let polygon_vertices = &star.base.base.polygon.vertices;
                let path_vertices = star.vertices();
                assert_eq!(polygon_vertices.len(), expected, "points={points:#x}");
                assert_eq!(path_vertices.len(), expected, "points={points:#x}");
                for (polygon_vertex, path_vertex) in polygon_vertices.iter().zip(path_vertices) {
                    let PathVertexOccurrence::RuntimeStraight(path_vertex) = path_vertex else {
                        panic!("a Star's inherited Polygon stores straight vertices");
                    };
                    assert!(Rc::ptr_eq(polygon_vertex, path_vertex));
                }
            })
            .unwrap();
    }
}

#[cfg(feature = "tools")]
mod node {
    use nuxie_runtime::source::{
        artboard::Artboard,
        core::{CoreArena, CoreHandle},
        core_context::CoreContext,
        generated::{component_base::ComponentBase, core_registry::CoreRegistry},
        layout_component::LayoutComponent,
        node::Node,
        status_code::StatusCode,
    };
    use std::cell::RefCell;

    struct ParentContext {
        arena: CoreArena,
        artboard: CoreHandle,
        parent: Option<CoreHandle>,
    }

    impl CoreContext for ParentContext {
        fn core_arena(&self) -> &CoreArena {
            &self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            match id {
                0 => Some(self.artboard.clone()),
                1 => self.parent.clone(),
                _ => None,
            }
        }
    }

    fn bind_component(
        arena: &CoreArena,
        owner: &CoreHandle,
        artboard: &CoreHandle,
        parent: Option<&CoreHandle>,
    ) {
        assert!(CoreRegistry::set_uint_handle(
            owner,
            ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
            1,
        ));
        let mut context = ParentContext {
            arena: arena.weak_handle(),
            artboard: artboard.clone(),
            parent: parent.cloned(),
        };
        let status = owner
            .with_mut(|object| {
                object
                    .as_component_mut()
                    .unwrap()
                    .on_added_dirty(&mut context)
            })
            .unwrap();
        assert_eq!(
            status,
            if parent.is_some() {
                StatusCode::Ok
            } else {
                StatusCode::MissingObject
            }
        );
    }

    struct RelinkState {
        arena: CoreArena,
        current: CoreHandle,
        current_artboard: CoreHandle,
        new_parent: CoreHandle,
        callbacks: Vec<&'static str>,
    }

    thread_local! {
        static RELINK: RefCell<Option<RelinkState>> = const { RefCell::new(None) };
    }

    fn reparent_current(_data: *mut ()) {
        let (arena, current, artboard, new_parent) = RELINK.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().unwrap();
            state.callbacks.push("current");
            (
                state.arena.weak_handle(),
                state.current.clone(),
                state.current_artboard.clone(),
                state.new_parent.clone(),
            )
        });
        bind_component(&arena, &current, &artboard, Some(&new_parent));
    }

    fn visited_old(_data: *mut ()) {
        RELINK.with(|state| state.borrow_mut().as_mut().unwrap().callbacks.push("old"));
    }

    fn visited_new(_data: *mut ()) {
        RELINK.with(|state| state.borrow_mut().as_mut().unwrap().callbacks.push("new"));
    }

    #[test]
    fn node_layout_dirty_walk_reads_parent_after_synchronous_callback() {
        let arena = CoreArena::default();
        let current_artboard = arena.insert(Artboard::default());
        let old_artboard = arena.insert(Artboard::default());
        let new_artboard = arena.insert(Artboard::default());
        let current = arena.insert(LayoutComponent::default());
        let old_parent = arena.insert(LayoutComponent::default());
        let new_parent = arena.insert(LayoutComponent::default());
        let node = arena.insert(Node::default());

        bind_component(&arena, &old_parent, &old_artboard, None);
        bind_component(&arena, &new_parent, &new_artboard, None);
        bind_component(&arena, &current, &current_artboard, Some(&old_parent));
        bind_component(&arena, &node, &current_artboard, Some(&current));

        RELINK.with(|state| {
            *state.borrow_mut() = Some(RelinkState {
                arena: arena.weak_handle(),
                current: current.clone(),
                current_artboard: current_artboard.clone(),
                new_parent,
                callbacks: Vec::new(),
            });
        });
        current_artboard
            .with_downcast_mut::<Artboard, _>(|root| root.on_layout_dirty(Some(reparent_current)))
            .unwrap();
        old_artboard
            .with_downcast_mut::<Artboard, _>(|root| root.on_layout_dirty(Some(visited_old)))
            .unwrap();
        new_artboard
            .with_downcast_mut::<Artboard, _>(|root| root.on_layout_dirty(Some(visited_new)))
            .unwrap();

        node.with_downcast_mut::<Node, _>(Node::mark_layout_node_dirty)
            .unwrap();
        RELINK.with(|state| {
            let state = state.borrow_mut().take().unwrap();
            assert_eq!(state.callbacks, ["current", "new"]);
        });
    }
}
