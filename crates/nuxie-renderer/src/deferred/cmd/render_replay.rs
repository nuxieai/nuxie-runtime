//! renderer/cmd/render_replay.hpp at e949498e.
pub use super::deferred_cmd::replay_render_commands;
use super::render_commands::ResourceKind;
use nuxie_render_api::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type PathOwner = Rc<RefCell<Box<dyn RenderPath>>>;
pub type PaintOwner = Rc<RefCell<Box<dyn RenderPaint>>>;
pub type BufferOwner = Rc<RefCell<Box<dyn RenderBuffer>>>;
pub type RendererOwner = Rc<RefCell<Box<dyn Renderer>>>;
pub type CanvasImageResolver<'a> = Box<dyn FnMut(u32) -> Option<Rc<dyn RenderImage>> + 'a>;
#[derive(Default)]
pub struct ReplayStats {
    pub dropped_draws: u32,
}
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFilter {
    #[default]
    All,
    Resources,
    Draws,
    Destroys,
}
#[derive(Default)]
pub struct ReplayHooks<'a> {
    pub filter: ReplayFilter,
    pub canvas_image: Option<CanvasImageResolver<'a>>,
    pub begin_canvas_content: Option<Box<dyn FnMut(u32, u32) -> Option<RendererOwner> + 'a>>,
    pub stats: Option<&'a mut ReplayStats>,
}

pub struct Resident<T: Clone> {
    pub objects: Vec<Option<T>>,
    pub generations: Vec<u32>,
    pub versions: Vec<u32>,
    pub version_aliases: HashMap<u64, Option<T>>,
}
impl<T: Clone> Default for Resident<T> {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            generations: Vec::new(),
            versions: Vec::new(),
            version_aliases: HashMap::new(),
        }
    }
}
impl<T: Clone> Resident<T> {
    pub fn set(&mut self, id: u32, object: Option<T>, generation: u32) {
        let slot = id as usize;
        if slot > self.objects.len() {
            debug_assert!(false, "nonsequential producer id");
            return;
        }
        if slot == self.objects.len() {
            self.objects.push(object);
            self.generations.push(generation);
            self.versions.push(0);
            return;
        }
        self.drop_version_aliases(id);
        self.objects[slot] = object;
        self.generations[slot] = generation;
        self.versions[slot] = 0;
    }
    pub fn drop_version_aliases(&mut self, id: u32) {
        self.version_aliases
            .retain(|&key, _| (key >> 32) as u32 != id);
    }
    pub fn new_version(&mut self, id: u32, version: u32, object: Option<T>) {
        let slot = id as usize;
        if slot >= self.objects.len() {
            return;
        }
        self.version_aliases.insert(
            (u64::from(id) << 32) | u64::from(self.versions[slot]),
            self.objects[slot].take(),
        );
        self.objects[slot] = object;
        self.versions[slot] = version;
    }
    pub fn destroy(&mut self, id: u32, generation: u32) {
        let slot = id as usize;
        if slot < self.objects.len() && self.generations[slot] == generation {
            self.objects[slot] = None;
            self.drop_version_aliases(id);
        }
    }
    pub fn get(&self, id: u32) -> Option<T> {
        self.objects.get(id as usize).cloned().flatten()
    }
    pub fn get_version(&self, id: u32, version: u32) -> Option<T> {
        let slot = id as usize;
        if slot >= self.objects.len() {
            return None;
        }
        if version == self.versions[slot] {
            return self.objects[slot].clone();
        }
        self.version_aliases
            .get(&((u64::from(id) << 32) | u64::from(version)))
            .cloned()
            .flatten()
    }
}
#[derive(Clone)]
pub struct PaintShadow {
    pub style: u8,
    pub color: u32,
    pub thickness: f32,
    pub join: u8,
    pub cap: u8,
    pub feather: f32,
    pub blend_mode: u8,
    pub shader: u32,
}
impl Default for PaintShadow {
    fn default() -> Self {
        Self {
            style: 1,
            color: 0xff000000,
            thickness: 1.0,
            join: 0,
            cap: 0,
            feather: 0.0,
            blend_mode: 3,
            shader: u32::MAX,
        }
    }
}
#[derive(Clone, Default)]
pub struct BufferShadow {
    pub buffer_type: u8,
    pub flags: u16,
    pub size: u32,
}
#[derive(Default)]
pub struct ResourceTable {
    pub paths: Resident<PathOwner>,
    pub paints: Resident<PaintOwner>,
    pub shaders: Resident<Rc<dyn RenderShader>>,
    pub images: Resident<Rc<dyn RenderImage>>,
    pub buffers: Resident<BufferOwner>,
    pub paint_shadows: Vec<PaintShadow>,
    pub path_fill_rules: Vec<u8>,
    pub buffer_shadows: Vec<BufferShadow>,
}
impl ResourceTable {
    pub fn destroy(&mut self, kind: ResourceKind, id: u32, generation: u32) {
        match kind {
            ResourceKind::Path => self.paths.destroy(id, generation),
            ResourceKind::Paint => self.paints.destroy(id, generation),
            ResourceKind::Shader => self.shaders.destroy(id, generation),
            ResourceKind::Image => self.images.destroy(id, generation),
            ResourceKind::Buffer => self.buffers.destroy(id, generation),
        }
    }
    pub fn clear_version_aliases(&mut self) {
        self.paths.version_aliases.clear();
        self.paints.version_aliases.clear();
        self.shaders.version_aliases.clear();
        self.images.version_aliases.clear();
        self.buffers.version_aliases.clear();
    }
}

pub fn rebuild_raw_path(verbs: &[u8], points: &[u8]) -> RawPath {
    let points: Vec<Vec2D> = points
        .chunks_exact(8)
        .map(|p| {
            Vec2D::new(
                f32::from_ne_bytes(p[..4].try_into().unwrap()),
                f32::from_ne_bytes(p[4..8].try_into().unwrap()),
            )
        })
        .collect();
    let verbs = verbs
        .iter()
        .map(|&verb| match verb {
            0 => PathVerb::Move,
            1 => PathVerb::Line,
            2 => PathVerb::Quad,
            4 => PathVerb::Cubic,
            5 => PathVerb::Close,
            _ => panic!("invalid recorded path verb"),
        })
        .collect();
    RawPath::from_verbs_and_points(verbs, points)
}
