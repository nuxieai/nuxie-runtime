use crate::ArtboardInstance;
use crate::components::ComponentHandle;

/// One mounted-artboard frame in a root-to-leaf occurrence path.
///
/// Rust's parent-owned mounted tree cannot safely store a raw child-to-parent
/// pointer. The caller that already owns the root-to-leaf path supplies the
/// exact host Component in the preceding frame instead.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParentTraversalFrame<'a> {
    pub(crate) artboard: &'a ArtboardInstance,
    pub(crate) host_component_in_parent: Option<ComponentHandle>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TraversedParent<'a> {
    pub(crate) artboard: &'a ArtboardInstance,
    pub(crate) component: ComponentHandle,
}

/// Stateful port of pinned C++ `ParentTraversal`.
///
/// The first `next` returns `start.parent`. Each call resets its crossing
/// metadata, returns the current parent, then advances. When the returned
/// Component has no direct parent, traversal crosses through the mounted
/// ArtboardHost only if that host Component itself has a parent
/// (`src/parent_traversal.cpp:9-61`).
pub(crate) struct ParentTraversal<'a> {
    current: Option<ComponentHandle>,
    current_artboard_index: Option<usize>,
    did_cross_boundary: bool,
    crossing_host: Option<ComponentHandle>,
    source_artboard_index: Option<usize>,
    frames: &'a [ParentTraversalFrame<'a>],
}

impl<'a> ParentTraversal<'a> {
    pub(crate) fn new(
        frames: &'a [ParentTraversalFrame<'a>],
        start: Option<ComponentHandle>,
    ) -> Self {
        let current = start.and_then(|start| {
            frames
                .last()
                .and_then(|frame| frame.artboard.component_parent_handle(start))
        });
        let current_artboard_index = start.and_then(|_| frames.len().checked_sub(1));
        Self {
            current,
            current_artboard_index,
            did_cross_boundary: false,
            crossing_host: None,
            source_artboard_index: None,
            frames,
        }
    }

    pub(crate) fn next(&mut self) -> Option<TraversedParent<'a>> {
        self.did_cross_boundary = false;
        self.crossing_host = None;
        self.source_artboard_index = None;

        let current = self.current?;
        let result_frame_index = self.current_artboard_index?;
        let result_artboard = self.frames.get(result_frame_index)?.artboard;

        if let Some(parent) = result_artboard.component_parent_handle(current) {
            self.current = Some(parent);
        } else if result_frame_index > 0 {
            let host = self.frames[result_frame_index].host_component_in_parent;
            let parent_frame_index = result_frame_index - 1;
            let host_parent = host.and_then(|host| {
                self.frames[parent_frame_index]
                    .artboard
                    .component_parent_handle(host)
            });
            if let (Some(host), Some(host_parent)) = (host, host_parent) {
                self.did_cross_boundary = true;
                self.crossing_host = Some(host);
                self.source_artboard_index = Some(result_frame_index);
                self.current = Some(host_parent);
                self.current_artboard_index = Some(parent_frame_index);
            } else {
                self.current = None;
            }
        } else {
            self.current = None;
        }

        Some(TraversedParent {
            artboard: result_artboard,
            component: current,
        })
    }

    pub(crate) fn current_artboard(&self) -> Option<&'a ArtboardInstance> {
        self.current_artboard_index
            .and_then(|index| self.frames.get(index))
            .map(|frame| frame.artboard)
    }

    pub(crate) fn did_cross_boundary(&self) -> bool {
        self.did_cross_boundary
    }

    pub(crate) fn crossing_host(&self) -> Option<ComponentHandle> {
        self.crossing_host
    }

    pub(crate) fn source_artboard(&self) -> Option<&'a ArtboardInstance> {
        self.source_artboard_index
            .and_then(|index| self.frames.get(index))
            .map(|frame| frame.artboard)
    }
}
