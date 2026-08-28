use crate::mechanical_port::source::{
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    core::CoreHandle,
};

/// Walks authored container parents and crosses nested-artboard host
/// boundaries without retaining borrows or pointers into either arena.
pub struct ParentTraversal {
    current: Option<CoreHandle>,
    current_artboard: Option<CoreHandle>,
    did_cross_boundary: bool,
    crossing_host: Option<CoreHandle>,
    source_artboard: Option<RuntimeArtboardInstanceWeakHandle>,
}

impl ParentTraversal {
    pub fn new(start: Option<CoreHandle>) -> Self {
        let current = start
            .as_ref()
            .and_then(|start| start.with(|start| start.as_component()?.parent_handle()))
            .flatten();
        let current_artboard = start
            .as_ref()
            .and_then(|start| start.with(|start| start.as_component()?.artboard_handle()))
            .flatten();
        Self {
            current,
            current_artboard,
            did_cross_boundary: false,
            crossing_host: None,
            source_artboard: None,
        }
    }

    pub fn next(&mut self) -> Option<CoreHandle> {
        self.did_cross_boundary = false;
        self.crossing_host = None;
        self.source_artboard = None;

        let result = self.current.clone()?;
        let parent = result
            .with(|result| result.as_component()?.parent_handle())
            .flatten();
        if parent.is_some() {
            self.current = parent;
            return Some(result);
        }

        let Some(current_artboard) = self.current_artboard.clone() else {
            self.current = None;
            return Some(result);
        };
        let Some((host, source_artboard)) = current_artboard
            .with_downcast::<Artboard, _>(|artboard| {
                (artboard.host(), artboard.runtime_weak_handle())
            })
            .and_then(|(host, source)| host.map(|host| (host, source)))
        else {
            self.current = None;
            return Some(result);
        };
        let host_state = host
            .with(|host| {
                let host = host.as_artboard_host()?;
                Some((host.host_component(), host.parent_artboard()))
            })
            .flatten();
        let Some((Some(host_component), parent_artboard)) = host_state else {
            self.current = None;
            return Some(result);
        };
        let parent = host_component
            .with(|component| component.as_component()?.parent_handle())
            .flatten();
        let Some(parent) = parent else {
            self.current = None;
            return Some(result);
        };

        self.did_cross_boundary = true;
        self.crossing_host = Some(host);
        self.source_artboard = Some(source_artboard);
        self.current = Some(parent);
        self.current_artboard = parent_artboard;
        Some(result)
    }

    pub fn current_artboard(&self) -> Option<CoreHandle> {
        self.current_artboard.clone()
    }

    pub fn did_cross_boundary(&self) -> bool {
        self.did_cross_boundary
    }

    pub fn crossing_host(&self) -> Option<CoreHandle> {
        self.crossing_host.clone()
    }

    pub fn source_artboard(&self) -> Option<RuntimeArtboardInstanceWeakHandle> {
        self.source_artboard.clone()
    }
}
