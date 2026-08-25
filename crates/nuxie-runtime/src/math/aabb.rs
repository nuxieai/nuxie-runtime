pub use nuxie_render_api::{
    Aabb as FloatAabb, AabbInteger, AabbScalarBounds, IntegerAabb, TypedAabb,
};

#[derive(Clone, Copy)]
struct RuntimeAabb {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl RuntimeAabb {
    fn from_artboard(instance: &ArtboardInstance) -> Self {
        Self {
            left: -instance.width * instance.origin_x,
            top: -instance.height * instance.origin_y,
            width: instance.width,
            height: instance.height,
        }
    }

    fn from_artboard_with_layout(instance: &ArtboardInstance, graph: &ArtboardGraph) -> Self {
        instance
            .retained_layout_bounds()
            .and_then(|bounds| bounds.get(&0).copied())
            .or_else(|| instance.runtime_root_artboard_layout_bounds(graph))
            .map(|bounds| Self {
                left: 0.0,
                top: 0.0,
                width: bounds.width,
                height: bounds.height,
            })
            .unwrap_or_else(|| Self::from_artboard(instance))
    }

    fn from_local_layout_bounds(bounds: RuntimeLayoutBounds) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: bounds.width,
            height: bounds.height,
        }
    }
}
