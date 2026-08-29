use super::mat2d::Mat2D;
use super::raw_path::RawPath;
use super::vec2d::Vec2D;

pub trait Axis {
    fn normalized(&self) -> bool;
    fn offset(&self) -> f32;
}

pub trait NSlicedNode {
    fn map_world_point(&self, point: &mut Vec2D);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleInfo {
    pub use_scale: bool,
    pub scale_factor: f32,
    pub fallback_size: f32,
}

pub struct NSlicerHelpers;
impl NSlicerHelpers {
    pub fn uv_stops(axes: &[Option<&dyn Axis>], size: f32) -> Vec<f32> {
        let mut stops = vec![0.0];
        for axis in axes.iter().flatten() {
            stops.push(if axis.normalized() {
                clamp(axis.offset(), 0.0, 1.0)
            } else {
                clamp(axis.offset() / size, 0.0, 1.0)
            });
        }
        stops.push(1.0);
        stops.sort_by(|a, b| a.total_cmp(b));
        stops
    }
    pub fn px_stops(axes: &[Option<&dyn Axis>], size: f32) -> Vec<f32> {
        let mut stops = vec![0.0];
        for axis in axes.iter().flatten() {
            stops.push(if axis.normalized() {
                clamp(axis.offset(), 0.0, 1.0) * size
            } else {
                clamp(axis.offset(), 0.0, size)
            });
        }
        stops.push(size);
        stops.sort_by(|a, b| a.total_cmp(b));
        stops
    }
    pub fn analyze_uv_stops(uvs: &[f32], size: f32, scale: f32) -> ScaleInfo {
        assert!(size >= 0.0 && scale >= 0.0);
        let mut fixed_percent = 0.0;
        let mut empty_patches = 0;
        for index in 0..uvs.len().saturating_sub(1) {
            let range = uvs[index + 1] - uvs[index];
            if Self::is_fixed_segment(index as i32) {
                fixed_percent += range;
            } else {
                empty_patches += i32::from(range == 0.0);
            }
        }
        let fixed_size = fixed_percent * size;
        let scalable_size = size - fixed_size;
        let use_scale = scalable_size != 0.0;
        let scale_factor = if use_scale {
            (size * scale - fixed_size) / scalable_size
        } else {
            0.0
        };
        let mut fallback_size = 0.0;
        if !use_scale && empty_patches != 0 {
            fallback_size = (size - fixed_size / scale) / empty_patches as f32;
        }
        ScaleInfo {
            use_scale,
            scale_factor,
            fallback_size,
        }
    }
    pub fn map_value(stops: &[f32], scale: ScaleInfo, size: f32, value: f32) -> f32 {
        if value < stops[0] - 0.01 {
            return value;
        }
        if value > stops[stops.len() - 1] + 0.01 {
            return value - stops[stops.len() - 1] + size;
        }
        let mut result = 0.0;
        for index in 0..stops.len() - 1 {
            let found = value <= stops[index + 1];
            let span = if found {
                value - stops[index]
            } else {
                stops[index + 1] - stops[index]
            };
            if Self::is_fixed_segment(index as i32) {
                result += span;
            } else {
                result += if scale.use_scale {
                    scale.scale_factor * span
                } else {
                    0.0
                };
            }
            if found {
                break;
            }
        }
        result
    }
    pub fn is_fixed_segment(index: i32) -> bool {
        index % 2 == 0
    }
    pub fn deform_local_render_path_with_n_slicer(
        node: &dyn NSlicedNode,
        local_path: &mut RawPath,
        world: &Mat2D,
        inverse_world: &Mat2D,
    ) {
        let mut temporary_world = local_path.transform(*world);
        Self::deform_world_render_path_with_n_slicer(node, &mut temporary_world);
        local_path.rewind();
        local_path.add_path(&temporary_world, Some(inverse_world));
    }
    pub fn deform_world_render_path_with_n_slicer(
        node: &dyn NSlicedNode,
        world_path: &mut RawPath,
    ) {
        for point in world_path.points_mut() {
            node.map_world_point(point);
        }
    }
}

fn clamp(value: f32, low: f32, high: f32) -> f32 {
    low.max(value).min(high)
}
