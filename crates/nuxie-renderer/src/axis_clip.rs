//! A local-axis strip intersected with the renderer's actual device clip bounds.
use nuxie_render_api::{IntegerAabb, Mat2D};

fn polygon(horizontal: bool, min: f32, max: f32, matrix: Mat2D, bounds: IntegerAabb) -> Option<Vec<[f64; 2]>> {
    if matrix.0.iter().chain([min, max].iter()).any(|v| !v.is_finite()) { return None; }
    if bounds.empty() || min >= max { return Some(Vec::new()); }
    let [a,b,c,d,tx,ty] = matrix.0.map(f64::from);
    let determinant = a*d-b*c;
    // All local geometry has zero device area under a singular transform.
    if determinant == 0. { return Some(Vec::new()); }
    let (u,v,w) = if horizontal {
        (d/determinant, -c/determinant, (c*ty-d*tx)/determinant)
    } else {
        (-b/determinant, a/determinant, (b*tx-a*ty)/determinant)
    };
    let mut points = vec![
        [bounds.left as f64,bounds.top as f64],
        [bounds.right as f64,bounds.top as f64],
        [bounds.right as f64,bounds.bottom as f64],
        [bounds.left as f64,bounds.bottom as f64],
    ];
    // Sutherland-Hodgman clipping against both transformed half planes. Work
    // depends on polygon vertices, never coordinate magnitude or target area.
    for (sign,edge) in [(1.,max as f64),(-1.,min as f64)] {
        let distance = |p: [f64;2]| sign*(u*p[0]+v*p[1]+w-edge);
        let mut result = Vec::new();
        if points.is_empty() { break; }
        let mut previous = *points.last().unwrap();
        let mut before = distance(previous);
        for current in points {
            let after = distance(current);
            if (before <= 0.) != (after <= 0.) {
                let t = before/(before-after);
                result.push([previous[0]+t*(current[0]-previous[0]),previous[1]+t*(current[1]-previous[1])]);
            }
            if after <= 0. { result.push(current); }
            previous = current;
            before = after;
        }
        points = result;
    }
    if points.iter().flatten().any(|v| !v.is_finite()) { return None; }
    Some(points)
}

#[cfg(feature = "renderer-metal")]
pub(crate) fn apply(
    renderer: &mut crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer,
    horizontal: bool, min: f32, max: f32,
) -> bool {
    use crate::mechanical_port::source::include::rive::renderer_hpp::RendererContract;
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer;
    use nuxie_render_api::{FillRule,RawPath};
    let state = renderer.current_state().clone();
    // An unclipped renderer starts with sentinel i32 bounds, not the frame.
    // Intersect before constructing the polygon: converting sentinel-scale
    // intersections to f32 destroys the precision of ordinary rotated edges.
    let frame = unsafe { &*renderer.m_context }.frameDescriptor();
    let (Ok(width), Ok(height)) = (i32::try_from(frame.renderTargetWidth), i32::try_from(frame.renderTargetHeight)) else { return false; };
    let bounds = state.overallClipPixelBounds.intersect(IntegerAabb::new(0,0,width,height));
    let Some(points) = polygon(horizontal,min,max,state.matrix,bounds) else { return false; };
    let mut mask = RawPath::default();
    if let Some(first) = points.first() {
        mask.move_to(first[0] as f32,first[1] as f32);
        for p in &points[1..] { mask.line_to(p[0] as f32,p[1] as f32); }
        mask.close();
    }
    let path = unsafe { &mut *renderer.m_context }.riveRenderFactoryMut().makeRenderPathHandle(&mut mask,FillRule::NonZero);
    let Some(path) = path else { return false; };
    renderer.current_state_mut().matrix = Mat2D::IDENTITY;
    unsafe { <RiveRenderer as RendererContract>::clipPath(renderer,path.source_base() as *const _ as *mut _); }
    renderer.current_state_mut().matrix = state.matrix;
    true
}

/// Preserve the exact current matrix, avoiding inverse-transform roundoff.
#[cfg(feature = "renderer-metal")]
pub(crate) fn apply_transformed(
    renderer: &mut crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer,
    horizontal: bool, min: f32, max: f32, local: Mat2D,
) -> bool {
    use crate::mechanical_port::source::include::rive::renderer_hpp::RendererContract;
    if local.0.iter().any(|v| !v.is_finite()) { return false; }
    let original = renderer.current_state().matrix;
    RendererContract::transform(renderer, &local);
    let accepted = apply(renderer, horizontal, min, max);
    renderer.current_state_mut().matrix = original;
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bounds() -> IntegerAabb { IntegerAabb::new(0,0,100,80) }
    fn area(points: &[[f64;2]]) -> f64 {
        if points.is_empty() { return 0.; }
        points.iter().zip(points.iter().cycle().skip(1)).take(points.len())
            .map(|(a,b)|a[0]*b[1]-a[1]*b[0]).sum::<f64>().abs()/2.
    }
    #[test]
    fn fractional_strip_keeps_the_other_axis_unrestricted() {
        let x=polygon(true,10.25,30.75,Mat2D::IDENTITY,bounds()).unwrap();
        assert_eq!(area(&x),20.5*80.);
        let y=polygon(false,10.25,30.75,Mat2D::IDENTITY,bounds()).unwrap();
        assert_eq!(area(&y),20.5*100.);
        assert_eq!(area(&polygon(true,-1e20,1e20,Mat2D::IDENTITY,bounds()).unwrap()),8000.);
    }
    #[test]
    fn rotation_reflection_and_shear_use_device_half_planes() {
        assert_eq!(area(&polygon(true,10.,30.,Mat2D([0.,1.,-1.,0.,100.,0.]),bounds()).unwrap()),2000.);
        assert_eq!(area(&polygon(true,10.,30.,Mat2D([-1.,0.,0.,1.,100.,0.]),bounds()).unwrap()),1600.);
        let shear=polygon(true,0.,20.,Mat2D([1.,0.,1.,1.,0.,0.]),bounds()).unwrap();
        assert_eq!(area(&shear),1600.);
    }
    #[test]
    fn empty_and_invalid_inputs_do_not_invent_coverage() {
        for (min,max) in [(5.,5.),(6.,5.),(101.,120.)] {
            assert_eq!(area(&polygon(true,min,max,Mat2D::IDENTITY,bounds()).unwrap()),0.);
        }
        assert!(polygon(true,f32::NAN,1.,Mat2D::IDENTITY,bounds()).is_none());
        assert_eq!(area(&polygon(true,0.,20.,Mat2D([0.;6]),bounds()).unwrap()),0.);
    }
}
