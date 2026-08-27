use crate::mechanical_port::source::semantic::{
    semantic_inference_registry::{
        InferenceComponent, resolve_inferred_semantics, supports_inferred_semantics,
    },
    semantic_snapshot::Bounds,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedSemanticData {
    pub has_semantics: bool,
    pub role: u32,
    pub label: String,
}

impl Bounds {
    pub fn for_expansion() -> Self {
        Self {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: -f32::MAX,
            max_y: -f32::MAX,
        }
    }
    pub fn is_empty_or_nan(self) -> bool {
        !(self.max_x - self.min_x > 0.0 && self.max_y - self.min_y > 0.0)
    }
    pub fn expand(&mut self, p: (f32, f32)) {
        self.min_x = if p.0 < self.min_x { p.0 } else { self.min_x };
        self.min_y = if p.1 < self.min_y { p.1 } else { self.min_y };
        self.max_x = if self.max_x < p.0 { p.0 } else { self.max_x };
        self.max_y = if self.max_y < p.1 { p.1 } else { self.max_y };
    }
}
pub trait SemanticComponent: InferenceComponent {
    fn explicit_semantics(&self) -> Option<(u32, String)>;
    fn local_bounds(&self) -> Bounds;
    fn world_transform(&self) -> [f32; 6];
    fn root_transform(&self, p: (f32, f32)) -> (f32, f32);
    fn descendants(&self) -> Vec<&dyn SemanticComponent>;
}
fn map_bounds(transform: [f32; 6], b: Bounds) -> Bounds {
    let mut out = Bounds::for_expansion();
    for (x, y) in [
        (b.min_x, b.min_y),
        (b.max_x, b.min_y),
        (b.max_x, b.max_y),
        (b.min_x, b.max_y),
    ] {
        out.expand((
            transform[0] * x + transform[2] * y + transform[4],
            transform[1] * x + transform[3] * y + transform[5],
        ));
    }
    out
}
pub fn root_transform_aabb(component: &dyn SemanticComponent, b: Bounds) -> Bounds {
    let mut out = Bounds::for_expansion();
    for p in [
        (b.min_x, b.min_y),
        (b.max_x, b.min_y),
        (b.max_x, b.max_y),
        (b.min_x, b.max_y),
    ] {
        out.expand(component.root_transform(p));
    }
    out
}
pub fn can_infer_semantics(component: Option<&dyn SemanticComponent>) -> bool {
    supports_inferred_semantics(component.map(|v| v as &dyn InferenceComponent))
}
pub fn resolve_semantic_data(component: Option<&dyn SemanticComponent>) -> ResolvedSemanticData {
    let mut out = ResolvedSemanticData::default();
    let Some(c) = component else { return out };
    if let Some((role, label)) = c.explicit_semantics() {
        out.has_semantics = true;
        out.role = role;
        out.label = label;
        return out;
    }
    resolve_inferred_semantics(Some(c), &mut out);
    out
}
pub fn semantic_bounds(component: Option<&dyn SemanticComponent>) -> Bounds {
    let Some(c) = component else {
        return Bounds::default();
    };
    let local = c.local_bounds();
    if !local.is_empty_or_nan() {
        return root_transform_aabb(c, map_bounds(c.world_transform(), local));
    }
    let mut merged = Bounds::for_expansion();
    let mut any = false;
    for child in c.descendants() {
        let local = child.local_bounds();
        if !local.is_empty_or_nan() {
            let b = map_bounds(child.world_transform(), local);
            merged.expand((b.min_x, b.min_y));
            merged.expand((b.max_x, b.max_y));
            any = true;
        }
    }
    if any {
        return root_transform_aabb(c, merged);
    }
    let t = c.world_transform();
    let p = c.root_transform((t[4], t[5]));
    Bounds {
        min_x: p.0,
        min_y: p.1,
        max_x: p.0,
        max_y: p.1,
    }
}
