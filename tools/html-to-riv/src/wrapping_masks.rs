//! Geometric observation-domain prerequisite for the existing rectangle masks.
//! This is conditional on exact 0/D signals and the pinned renderer's AABB
//! intersection route. It is not a raster, gate or renderer-matrix certificate.
use super::{wrapping_domains::Domains, wrapping_sizes::MachineInterval};
use crate::wire::Value;

const SIDE:f32=32768.;
const SENTINEL:f32=65536.;
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Viewport { axis:usize }, InitialViewport { axis:usize } }
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rectangle { pub min:[f32;2], pub max:[f32;2] }
#[derive(Debug)]
pub(super) struct Proof {
    viewport:[MachineInterval;2],
    active:Rectangle,
    inactive:Rectangle,
}
impl Proof {
    pub(super) fn viewport(&self)->[MachineInterval;2] {self.viewport}
    pub(super) fn active(&self)->Rectangle {self.active}
    pub(super) fn inactive(&self)->Rectangle {self.inactive}
}
fn bounds(viewport:[MachineInterval;2],row:bool)->Result<Proof,Unresolved> {
    for (axis,v) in viewport.iter().enumerate() {
        if v.upper()>SIDE {return Err(Unresolved::Viewport{axis});}
    }
    let active=Rectangle{min:[0.;2],max:[SIDE;2]};
    let mut inactive=active;
    let cross=usize::from(row);
    inactive.min[cross]+=SENTINEL;inactive.max[cross]+=SENTINEL;
    Ok(Proof{viewport,active,inactive})
}
/// Domains owns a checked base: artboard clip is the native true default,
/// origin/transform are zero, radii are absent, and layout is zero inset. The
/// emitted paint grammar must separately bind exact mask dimensions/origins,
/// transforms, clipping ownership and finalized 0/D gate values.
///
/// The active rectangle contains the complete artboard clip for every declared
/// viewport. The inactive rectangle is disjoint on the physical cross axis.
/// Matching renderer matrices and intersection-before-AA remain separate
/// prerequisites. No arbitrary artboard-unit antialias margin is assumed.
/// Both initial stored artboard dimensions must also lie in the declared domain;
/// otherwise importing before a host resize would bypass this observation bound.
pub(super) fn prove(domains:&Domains<'_>)->Result<Proof,Unresolved> {
    let proof=bounds(domains.viewport(),domains.base().row)?;
    let artboard=&domains.base().records()[1];
    for (axis,name) in ["width","height"].iter().enumerate() {
        let value=match artboard.get(name) {
            None=>0.,Some(Value::Float(v))=>*v,_=>return Err(Unresolved::InitialViewport{axis}),
        };
        let v=proof.viewport[axis];
        if !value.is_finite() || value<v.lower() || value>v.upper() {
            return Err(Unresolved::InitialViewport{axis});
        }
    }
    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn interval(upper:f32)->MachineInterval {MachineInterval::new(0.,upper).unwrap()}
    #[test]
    fn active_intersection_preserves_every_domain_endpoint_and_inactive_is_empty() {
        for row in [false,true] {
            for limit in [0.,f32::from_bits(1),0.5,16384.,SIDE] {
                let proof=bounds([interval(limit);2],row).unwrap();
                for width in [0.,limit/2.,limit] {for height in [0.,limit/2.,limit] {
                    let viewport=Rectangle{min:[0.;2],max:[width,height]};
                    let intersection=|mask:Rectangle| Rectangle {
                        min:std::array::from_fn(|i|viewport.min[i].max(mask.min[i])),
                        max:std::array::from_fn(|i|viewport.max[i].min(mask.max[i])),
                    };
                    assert_eq!(intersection(proof.active()),viewport);
                    let inactive=intersection(proof.inactive());
                    assert!(inactive.max[usize::from(row)]<inactive.min[usize::from(row)]);
                }}
            }
        }
    }
    #[test]
    fn wider_viewports_expose_active_truncation_and_reject_instead_of_expanding_masks() {
        for axis in 0..2 {
            let mut viewport=[interval(16384.);2];viewport[axis]=interval(SIDE.next_up());
            assert_eq!(bounds(viewport,true).unwrap_err(),Unresolved::Viewport{axis});
            // This is an actual geometric failure of the current mask, not a
            // declaration that another ordinary-file composition is impossible.
            assert!(SIDE.min(viewport[axis].upper())<viewport[axis].upper());
        }
    }
}
