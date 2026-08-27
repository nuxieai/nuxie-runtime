use crate::mechanical_port::source::{
    artboard::Artboard, artboard_host::ArtboardHost, component::Component,
    container_component::ContainerComponent,
};

pub struct ParentTraversal {
    current: Option<*mut ContainerComponent>,
    current_artboard: Option<*mut Artboard>,
    did_cross_boundary: bool,
    crossing_host: Option<*mut dyn ArtboardHost>,
    source_artboard: Option<*mut Artboard>,
}

impl ParentTraversal {
    pub fn new(mut start: Option<&mut Component>) -> Self {
        let current = start
            .as_deref_mut()
            .and_then(Component::parent_mut)
            .map(|parent| parent as *mut ContainerComponent);
        let current_artboard = start
            .and_then(Component::artboard_mut)
            .map(|artboard| artboard as *mut Artboard);
        Self {
            current,
            current_artboard,
            did_cross_boundary: false,
            crossing_host: None,
            source_artboard: None,
        }
    }

    pub fn next(&mut self) -> Option<&mut ContainerComponent> {
        self.did_cross_boundary = false;
        self.crossing_host = None;
        self.source_artboard = None;

        let current = self.current?;
        let result = unsafe { &mut *current };

        if let Some(parent) = result.base.base.parent_mut() {
            self.current = Some(parent as *mut ContainerComponent);
        } else if let Some(current_artboard) = self.current_artboard {
            let artboard = unsafe { &mut *current_artboard };
            if let Some(host) = artboard.host_mut() {
                let host = host as *mut dyn ArtboardHost;
                let host_ref = unsafe { &mut *host };
                if let Some(host_component) = host_ref.host_component()
                    && let Some(parent) = host_component.parent_mut()
                {
                    self.did_cross_boundary = true;
                    self.crossing_host = Some(host);
                    self.source_artboard = Some(current_artboard);
                    self.current = Some(parent as *mut ContainerComponent);
                    self.current_artboard = Some(host_ref.parent_artboard() as *mut Artboard);
                    return Some(result);
                }
            }
            self.current = None;
        } else {
            self.current = None;
        }

        Some(result)
    }

    pub fn current_artboard(&self) -> Option<&Artboard> {
        self.current_artboard.map(|artboard| unsafe { &*artboard })
    }

    pub fn did_cross_boundary(&self) -> bool {
        self.did_cross_boundary
    }

    pub fn crossing_host(&self) -> Option<&dyn ArtboardHost> {
        self.crossing_host.map(|host| unsafe { &*host })
    }

    pub fn source_artboard(&self) -> Option<&Artboard> {
        self.source_artboard.map(|artboard| unsafe { &*artboard })
    }
}
