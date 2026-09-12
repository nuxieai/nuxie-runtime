//! Conditional arithmetic bounds for the final wrapping positioning graph.
//! This bounds the emitted slot-based correction, not Chrome/CSS alignment.
use super::{wrapping_coordinates, wrapping_carry::{Bound, Measured}};

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
    pub(super) excess: Range,
    pub(super) offset: Range,
    pub(super) target: Range,
    pub(super) excess_error: f64,
    pub(super) offset_error: f64,
    /// Absolute error against p + (L-h)*(a-f), using actual native slot
    /// location p, line maximum L, slot extent h and physical fractions a/f.
    /// This excludes any CSS-vs-native layout error or differing visible size.
    pub(super) target_error: f64,
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

fn item(top:Range,top_error:f64,height:Measured,maximum:Bound,factor:f32)->Result<Item,Unresolved> {
    if ![-1.,-0.5,0.,0.5,1.].contains(&factor) {return Err(Unresolved::Fraction);}
    checked_error(top_error)?;checked_error(height.error)?;checked_error(maximum.error)?;
    if !maximum.upper.is_finite() || maximum.upper<0. {return Err(Unresolved::Arithmetic);}
    let measured=Range::new(height.machine.lower(),height.machine.upper())?;
    let max_range=Range::new(0.,upper32(maximum.upper)?)?;
    // diff(max,height): exact negation, then one destination-local sum.
    let excess=max_range.sub(measured)?;
    let difference_round=error(max_range.magnitude().max(measured.magnitude()));
    let excess_error=checked_error(add_up(add_up(maximum.error,height.error),difference_round))?;
    // The factor is dyadic, but halving a subnormal can round. Retain a
    // general absolute rounding term instead of incorrectly calling it exact.
    let offset=excess.scale(factor)?;
    let offset_error=checked_error(add_up(mul_up(factor.abs() as f64,excess_error),
        error(mul_up(excess.magnitude(),factor.abs() as f64))))?;
    let target=top.add(offset)?;
    let addition_round=error(add_up(top.magnitude(),offset.magnitude()));
    let target_error=checked_error(add_up(add_up(top_error,offset_error),addition_round))?;
    Ok(Item{excess,offset,target,excess_error,offset_error,target_error})
}

/// `coordinates` must belong to the same closed graph whose row, physical
/// fractions and item order are supplied here. Record fields/evaluation must
/// separately certify the audited identity-linear, zero-anchor, strength-one
/// templates. No arithmetic function can establish that association by itself.
///
/// Under those premises, both coordinates of every initial visible transform
/// are finite (coordinates::prove). Target endpoints are checked here. The
/// final world copy evaluates old*0 + target*1 exactly for finite values; the
/// untouched coordinate is also copied exactly. Repeating this argument keeps
/// original and clone arithmetic finite, conditional on correct evaluation.
pub(super) fn prove(coordinates:&wrapping_coordinates::Proof,alignments:&[f32],
    row:bool,line_fraction:f32,reverse_cross:bool)->Result<Proof,Unresolved> {
    let n=coordinates.measured().len();
    if n==0 || alignments.len()!=n || coordinates.carry().line_maxima.len()!=n
        || coordinates.visible_world().len()!=n {return Err(Unresolved::Count);}
    if ![0.,0.5,1.].contains(&line_fraction)
        || alignments.iter().any(|a|![0.,0.5,1.].contains(a)) {return Err(Unresolved::Fraction);}
    let location=coordinates.axes()[usize::from(row)].locations;
    // The f=0 landmark's operations are a subset of the conditional 31 eta
    // ledger. Using its full bound is conservative against machine slot p,
    // even though the 31 ledger additionally charges native placement/pairs.
    let top_error=checked_error(coordinates.same_line_error())?;
    let top=Range::new(lower32((location.lower() as f64-top_error).next_down())?,
        upper32(add_up(location.upper() as f64,top_error))?)?;
    let physical=|f:f32|if reverse_cross {1.-f}else{f};
    let f=physical(line_fraction);
    let items=alignments.iter().zip(coordinates.measured()).zip(&coordinates.carry().line_maxima)
        .map(|((&a,&height),&maximum)|item(top,top_error,height,maximum,physical(a)-f))
        .collect::<Result<Vec<_>,_>>()?;
    Ok(Proof{items})
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{wrapping_carry,wrapping_sizes::MachineInterval};
    fn measured(v:f32,e:f64)->Measured {Measured{machine:MachineInterval::new(v,v).unwrap(),error:e}}
    fn contains(r:Range,v:f32) {assert!(v>=r.lower && v<=r.upper,"{v} outside {r:?}");}
    fn native_max(a:f32,b:f32)->f32 {(b-a).max(0.)+a}

    #[test]
    fn final_positions_cover_all_partitions_and_signed_alignment_factors() {
        let values=[0.125,50.,1.+3.*2_f32.powi(-23),3.*2_f32.powi(-24),30.];
        let measurements=values.map(|v|measured(v,0.));
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
            for i in 0..5 {for p in [-8192.,-0.125,0.,0.1,8192.] {for factor in [-1.,-0.5,0.,0.5,1.] {
                let proof=item(Range::new(p,p).unwrap(),0.,measurements[i],carry.line_maxima[i],factor).unwrap();
                let maximum=native_max(forward[i],backward[i]);
                let excess=maximum-values[i];let offset=excess*factor;let target=p+offset;
                contains(proof.excess,excess);contains(proof.offset,offset);contains(proof.target,target);
                let ideal_max=values[lo[i]..=hi[i]].iter().copied().fold(0.,f32::max) as f64;
                let ideal_excess=ideal_max-values[i] as f64;
                assert!((excess as f64-ideal_excess).abs()<=proof.excess_error);
                assert!((offset as f64-ideal_excess*factor as f64).abs()<=proof.offset_error);
                assert!((target as f64-(p as f64+ideal_excess*factor as f64)).abs()<=proof.target_error);
            }}}
        }
    }
    #[test]
    fn top_and_measurement_errors_are_propagated_even_under_cancellation() {
        let h=measured(30.,0.01);let maximum=Bound{upper:50.1,error:0.1};
        let proof=item(Range::new(-20.01,-19.99).unwrap(),0.01,h,maximum,1.).unwrap();
        let actual=(-20_f32)+((50.05_f32)-30.);
        contains(proof.target,actual);
        assert!((actual as f64-0.).abs()<=proof.target_error);
        assert!(proof.target_error>=0.12);
    }
    #[test]
    fn subnormal_halving_is_not_assumed_exact() {
        let smallest=f32::from_bits(1);
        let proof=item(Range::new(0.,0.).unwrap(),0.,measured(0.,0.),
            Bound{upper:smallest as f64,error:0.},0.5).unwrap();
        let actual=smallest*0.5;
        assert_eq!(actual,0.);
        assert!((actual as f64-smallest as f64*0.5).abs()<=proof.offset_error);
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
    fn nonfinite_sum_and_invalid_arithmetic_premises_reject() {
        let top=Range::new(f32::MAX,f32::MAX).unwrap();
        assert!(matches!(item(top,0.,measured(0.,0.),Bound{upper:f32::MAX as f64,error:0.},1.),Err(Unresolved::Arithmetic)));
        for e in [-1.,f64::NAN,f64::INFINITY] {
            assert!(matches!(item(top,e,measured(0.,0.),Bound{upper:0.,error:0.},0.),Err(Unresolved::InvalidError)));
        }
        assert!(matches!(item(top,0.,measured(0.,0.),Bound{upper:0.,error:0.},0.25),Err(Unresolved::Fraction)));
    }
}
