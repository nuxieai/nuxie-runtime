//! Conditional arithmetic bounds for the final wrapping positioning graph.
//! Bounds anchored visible-box positioning, conditional on the checked graph.
use super::{wrapping_coordinates, wrapping_carry::Bound, wrapping_domains::Slot};

const U: f64 = 1. / 16777216.;
const TINY: f64 = f32::from_bits(1) as f64;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Count, Fraction, InvalidError, Arithmetic }

#[derive(Clone, Copy, Debug)]
pub(super) struct Range { pub(super) lower: f32, pub(super) upper: f32 }
impl Range {
    fn new(lower:f32,upper:f32)->Result<Self,Unresolved> {
        if !lower.is_finite() || !upper.is_finite() || lower>upper {
            Err(Unresolved::Arithmetic)
        } else {Ok(Self{lower,upper})}
    }
    fn magnitude(self)->f64 {self.lower.abs().max(self.upper.abs()) as f64}
    fn add(self,b:Self)->Result<Self,Unresolved> {Self::new(self.lower+b.lower,self.upper+b.upper)}
    fn sub(self,b:Self)->Result<Self,Unresolved> {Self::new(self.lower-b.upper,self.upper-b.lower)}
    fn scale(self,f:f32)->Result<Self,Unresolved> {
        if f>=0. {Self::new(self.lower*f,self.upper*f)}
        else {Self::new(self.upper*f,self.lower*f)}
    }
}

#[derive(Debug)]
pub(super) struct Item {
    pub(super) anchor: Range,
    pub(super) offset: Range,
    pub(super) target: Range,
    pub(super) origin: Range,
    pub(super) landed: Range,
    pub(super) anchor_error: f64,
    pub(super) offset_error: f64,
    pub(super) target_error: f64,
    pub(super) origin_error: f64,
    /// Against p + h*f + L*(a-f) - v*a for actual native slot position p,
    /// slot extent h, line maximum L, visible extent v and physical a/f.
    /// CSS-versus-native layout error is separate.
    pub(super) landed_error: f64,
}
#[derive(Debug)]
pub(super) struct Proof { pub(super) items: Vec<Item> }

fn add_up(a:f64,b:f64)->f64 {(a+b).next_up()}
fn mul_up(a:f64,b:f64)->f64 {(a*b).next_up()}
fn error(magnitude:f64)->f64 {add_up(mul_up(U,magnitude),TINY)}
fn checked_error(value:f64)->Result<f64,Unresolved> {
    if value.is_finite() && value>=0. {Ok(value)}else{Err(Unresolved::InvalidError)}
}
fn upper32(v:f64)->Result<f32,Unresolved> {
    let f=v as f32;let f=if (f as f64)<v {f.next_up()}else{f};
    if f.is_finite(){Ok(f)}else{Err(Unresolved::Arithmetic)}
}
fn lower32(v:f64)->Result<f32,Unresolved> {
    let f=v as f32;let f=if (f as f64)>v {f.next_down()}else{f};
    if f.is_finite(){Ok(f)}else{Err(Unresolved::Arithmetic)}
}

fn item(anchor:Range,anchor_error:f64,visible:Range,maximum:Bound,a:f32,f:f32)->Result<Item,Unresolved> {
    if ![0.,0.5,1.].contains(&a) || ![0.,0.5,1.].contains(&f) {
        return Err(Unresolved::Fraction);
    }
    checked_error(anchor_error)?;checked_error(maximum.error)?;
    if !maximum.upper.is_finite() || maximum.upper<0. || visible.lower<0. {
        return Err(Unresolved::Arithmetic);
    }
    let factor=a-f; // These dyadic differences are exact.
    let max_range=Range::new(0.,upper32(maximum.upper)?)?;
    let offset=max_range.scale(factor)?;
    let offset_error=checked_error(add_up(mul_up(factor.abs() as f64,maximum.error),
        error(mul_up(max_range.magnitude(),factor.abs() as f64))))?;
    let target=anchor.add(offset)?;
    let target_error=checked_error(add_up(add_up(anchor_error,offset_error),
        error(add_up(anchor.magnitude(),offset.magnitude()))))?;
    // local_anchor: origin fraction times actual visible extent. Halving
    // subnormals may round, so this product is not assumed exact.
    let origin=visible.scale(a)?;
    let origin_error=checked_error(error(mul_up(visible.magnitude(),a as f64)))?;
    // With identity linear entries and strength one, land_anchor's matrix
    // products and sum copy this origin exactly, then subtract once.
    let landed=target.sub(origin)?;
    let landed_error=checked_error(add_up(add_up(target_error,origin_error),
        error(add_up(target.magnitude(),origin.magnitude()))))?;
    Ok(Item{anchor,offset,target,origin,landed,anchor_error,offset_error,target_error,
        origin_error,landed_error})
}

/// All arguments must belong to the same closed construction. The graph
/// certificate must bind slot anchors at physical f, maximum scaling by a-f,
/// the visible ComponentOrigin at a (orthogonal zero), and a strength-one
/// world copy. This function alone does not prove dependency evaluation.
///
/// coordinates establishes finite initial visible translations on BOTH axes.
/// Identity scale/rotation means build_own_transform_with skips the pivot
/// branch even with ComponentOrigin attached. Thus initial translations stay
/// unchanged. The copied active target and native origin product are finite
/// here; land_anchor's inactive origin is zero and leaves the other coordinate
/// finite. Repeated constraints and clones obey the same conditional bounds.
pub(super) fn prove(coordinates:&wrapping_coordinates::Proof,slots:&[Slot],alignments:&[f32],
    row:bool,line_fraction:f32,reverse_cross:bool)->Result<Proof,Unresolved> {
    let n=coordinates.measured().len();
    if n==0 || slots.len()!=n || alignments.len()!=n || coordinates.carry().line_maxima.len()!=n
        || coordinates.visible_world().len()!=n {return Err(Unresolved::Count);}
    if ![0.,0.5,1.].contains(&line_fraction)
        || alignments.iter().any(|a|![0.,0.5,1.].contains(a)) {return Err(Unresolved::Fraction);}
    let cross=usize::from(row);
    let location=coordinates.axes()[cross].locations;
    // The full conditional 31-eta ledger conservatively covers the measured
    // p+h*f landmark against the actual machine slot p and native extent h.
    let anchor_error=checked_error(coordinates.same_line_error())?;
    let physical=|f:f32|if reverse_cross {1.-f}else{f};
    let f=physical(line_fraction);
    let items=alignments.iter().zip(slots).zip(&coordinates.carry().line_maxima)
        .map(|((&a,slot),&maximum)| {
            let h=slot.axes[cross];let v=slot.visible_axes[cross];
            // Enclose the ideal p+h*f first in f64, then expand by the
            // measured-anchor error. No uncharged f32 sum enters this bound.
            let lower=(location.lower() as f64 + h.lower() as f64*f as f64).next_down();
            let upper=(location.upper() as f64 + h.upper() as f64*f as f64).next_up();
            let anchor=Range::new(lower32((lower-anchor_error).next_down())?,
                upper32(add_up(upper,anchor_error))?)?;
            item(anchor,anchor_error,Range::new(v.lower(),v.upper())?,maximum,physical(a),f)
        }).collect::<Result<Vec<_>,_>>()?;
    Ok(Proof{items})
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{wrapping_carry::{self,Measured},wrapping_sizes::MachineInterval};
    fn range(v:f32)->Range {Range::new(v,v).unwrap()}
    fn contains(r:Range,v:f32) {assert!(v>=r.lower && v<=r.upper,"{v} outside {r:?}");}
    fn native_max(a:f32,b:f32)->f32 {(b-a).max(0.)+a}

    #[test]
    fn anchored_positions_cover_all_partitions_and_unequal_visible_extents() {
        let values=[0.125,50.,1.+3.*2_f32.powi(-23),3.*2_f32.powi(-24),30.];
        let measurements=values.map(|v|Measured{machine:MachineInterval::new(v,v).unwrap(),error:0.});
        let carry=wrapping_carry::prove(&measurements).unwrap();
        for partition in 0..16 {
            let mut forward=values;let mut backward=values;
            let mut lo=[0;5];let mut hi=[4;5];
            for i in 1..5 {
                let reset=partition&(1<<(i-1))!=0;
                forward[i]=native_max((forward[i-1]-if reset{65536.}else{0.}).max(0.),values[i]);
                lo[i]=if reset{i}else{lo[i-1]};
            }
            for i in (0..4).rev() {
                let reset=partition&(1<<i)!=0;
                backward[i]=native_max((backward[i+1]-if reset{65536.}else{0.}).max(0.),values[i]);
                hi[i]=if reset{i}else{hi[i+1]};
            }
            for i in 0..5 {for p in [-8192.,-0.125,0.,0.1,8192.] {
                for reverse in [false,true] {for logical_f in [0.,0.5,1.] {for logical_a in [0.,0.5,1.] {
                    let f=if reverse{1.-logical_f}else{logical_f};
                    let a=if reverse{1.-logical_a}else{logical_a};
                    for v in [0.,values[i]*0.5,values[i],values[i]*2.] {
                        let anchor=p+values[i]*f;
                        let ideal_anchor=p as f64+values[i] as f64*f as f64;
                        let proof=item(range(anchor),(anchor as f64-ideal_anchor).abs(),range(v),carry.line_maxima[i],a,f).unwrap();
                        let maximum=native_max(forward[i],backward[i]);
                        let offset=maximum*(a-f);let target=anchor+offset;
                        let origin=v*a;let landed=target-origin;
                        contains(proof.offset,offset);contains(proof.target,target);
                        contains(proof.origin,origin);contains(proof.landed,landed);
                        let ideal_max=values[lo[i]..=hi[i]].iter().copied().fold(0.,f32::max) as f64;
                        let ideal=ideal_anchor+ideal_max*(a-f) as f64-v as f64*a as f64;
                        assert!((landed as f64-ideal).abs()<=proof.landed_error);
                    }
                }}}
            }}
        }
    }
    #[test]
    fn cancellation_preserves_anchor_and_maximum_error_budgets() {
        let proof=item(Range::new(-20.01,-19.99).unwrap(),0.01,range(30.),
            Bound{upper:50.1,error:0.1},1.,0.).unwrap();
        let actual=(-20_f32+50.05)-30.;
        contains(proof.landed,actual);
        assert!((actual as f64).abs()<=proof.landed_error);
        assert!(proof.landed_error>=0.11);
    }
    #[test]
    fn subnormal_offset_and_native_origin_halving_are_not_assumed_exact() {
        let tiny=f32::from_bits(1);
        let proof=item(range(0.),0.,range(tiny),Bound{upper:tiny as f64,error:0.},0.5,0.).unwrap();
        assert_eq!(tiny*0.5,0.);
        assert!(proof.offset_error>=tiny as f64*0.5);
        assert!(proof.origin_error>=tiny as f64*0.5);
        contains(proof.landed,0.);
    }
    #[test]
    fn unequal_visible_extent_needs_an_additional_semantic_correction() {
        // Algebraic counterexample, not a native or Chrome observation. Slot
        // start is native line_origin + f*(line_height-slot_height).
        let line_origin=7_f32;let line_height=80_f32;
        let slot_height=50_f32;let visible_height=20_f32;
        for f in [0.,0.5,1.] {for a in [0.,0.5,1.] {
            let slot_top=line_origin+f*(line_height-slot_height);
            let emitted=slot_top+(line_height-slot_height)*(a-f);
            let desired=line_origin+a*(line_height-visible_height);
            let corrected=emitted+a*(slot_height-visible_height);
            assert_eq!(corrected,desired);
            if a==0. {assert_eq!(emitted,desired);}
            else {assert_ne!(emitted,desired);}
        }}
        let start_top=0_f32;let f=0_f32;let a=1_f32;
        assert_eq!(start_top+(line_height-slot_height)*(a-f),30.);
        assert_eq!(a*(line_height-visible_height),60.);
    }
    #[test]
    fn nonfinite_target_or_landing_and_invalid_premises_reject() {
        let max=Bound{upper:f32::MAX as f64,error:0.};
        assert!(matches!(item(range(f32::MAX),0.,range(0.),max,1.,0.),Err(Unresolved::Arithmetic)));
        assert!(matches!(item(range(-f32::MAX),0.,range(f32::MAX),Bound{upper:0.,error:0.},1.,1.),Err(Unresolved::Arithmetic)));
        for e in [-1.,f64::NAN,f64::INFINITY] {
            assert!(matches!(item(range(0.),e,range(0.),max,0.,0.),Err(Unresolved::InvalidError)));
        }
        assert!(matches!(item(range(0.),0.,range(0.),max,0.25,0.),Err(Unresolved::Fraction)));
    }
}
