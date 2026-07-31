//! PointsPath owns authored vertices and optional Skin deformation. Its
//! generated property callbacks are inherited by Path/ParametricPath; vertex
//! callbacks route here through `path_vertex`.

use crate::components::Mat2D;

pub(crate) fn path_transform(has_skin: bool, world_transform: Mat2D) -> Mat2D {
    if has_skin {
        Mat2D::IDENTITY
    } else {
        world_transform
    }
}
