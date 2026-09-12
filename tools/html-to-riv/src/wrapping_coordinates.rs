//! Conditional source-derived coordinate, line-gate and carry bounds.
//! The native base/domain premises are checked, but scalar record parameters,
//! positioning error and raster mask coverage still require final qualification.
use super::{wrapping_carry, wrapping_domains::Domains, wrapping_normalizer,
    wrapping_sizes::MachineInterval};

const U:f64=1./16777216.;
const TINY:f64=f32::from_bits(1) as f64;
#[derive(Debug)]
pub(super) enum Unresolved {
    Arithmetic,
    Count,
    Fraction,
    Separation,
    Normalizer(wrapping_normalizer::Unresolved),
    Carry(wrapping_carry::Unresolved),
}

/// Signed machine endpoints. Each operation below is evaluated in the native
/// binary32 order; endpoint monotonicity encloses all representable inputs.
#[derive(Clone,Copy,Debug)]
pub(super) struct Range { lower:f32, upper:f32 }
impl Range {
    fn new(lower:f32,upper:f32)->Result<Self,Unresolved> {
        if !lower.is_finite() || !upper.is_finite() || lower>upper {Err(Unresolved::Arithmetic)}
        else {Ok(Self{lower,upper})}
    }
    fn zero()->Self {Self{lower:0.,upper:0.}}
    fn add(self,b:Self)->Result<Self,Unresolved> {Self::new(self.lower+b.lower,self.upper+b.upper)}
    fn sub(self,b:Self)->Result<Self,Unresolved> {Self::new(self.lower-b.upper,self.upper-b.lower)}
    fn scale(self,f:f32)->Result<Self,Unresolved> {
        if ![0.,0.5,1.].contains(&f) {return Err(Unresolved::Fraction);}
        // Multiplication by0/.5/1 matches positional zero/divide-by-two/copy.
        Self::new(self.lower*f,self.upper*f)
    }
    fn hull(self,b:Self)->Self {Self{lower:self.lower.min(b.lower),upper:self.upper.max(b.upper)}}
    fn magnitude(self)->f32 {self.lower.abs().max(self.upper.abs())}
    pub(super) fn lower(self)->f32 {self.lower}
    pub(super) fn upper(self)->f32 {self.upper}
}
impl From<MachineInterval> for Range {
    fn from(v:MachineInterval)->Self {Self{lower:v.lower(),upper:v.upper()}}
}

#[derive(Clone,Debug)]
pub(super) struct Axis {
    pub sum:Range,
    pub first_offset:Range,
    pub accumulator:Range,
    pub locations:Range,
    pub peak:f32,
}
fn sum_upper(count:usize,upper:f32)->Result<f32,Unresolved> {
    let mut sum=0.;
    for _ in 0..count {
        sum+=upper;
        if !sum.is_finite() {return Err(Unresolved::Arithmetic);}
    }
    Ok(sum)
}
fn extents(values:&[MachineInterval])->Result<MachineInterval,Unresolved> {
    let first=*values.first().ok_or(Unresolved::Count)?;
    MachineInterval::new(values.iter().fold(first.lower(),|a,b|a.min(b.lower())),
        values.iter().fold(first.upper(),|a,b|a.max(b.upper()))).map_err(|_|Unresolved::Arithmetic)
}

/// Cross: unknown1..N nonempty lines, each max extent in the slot hull. Main:
/// unknown1..N items in a line. Positional first offset is shared machine input.
/// LTR is supplied by the checked base. Reverse traversal changes the first
/// offset fraction, but not the recurrence; RTL remains outside the base shape.
fn axis(parent:MachineInterval,values:&[MachineInterval],fraction:f32,cross:bool)->Result<Axis,Unresolved> {
    let sizes=extents(values)?;
    let sum=Range::new(if cross {sizes.lower()}else{0.},sum_upper(values.len(),sizes.upper())?)?;
    let free=Range::from(parent).sub(sum)?;
    let first_offset=free.scale(fraction)?;
    let item_offset=if cross {
        // Within a line, native max size >= this slot size, so the actual
        // subtraction is nonnegative. Preserve that correlation explicitly.
        Range::new(0.,sizes.upper()-sizes.lower())?.scale(fraction)?
    }else{Range::zero()};
    let mut peak=sum.magnitude().max(free.magnitude()).max(first_offset.magnitude())
        .max(item_offset.magnitude()).max(parent.upper()).max(sizes.upper());
    let mut total=Range::zero();let mut accumulator=total;let mut locations=total;
    for i in 0..values.len() {
        let offset=if i==0 {first_offset}else{Range::zero()};
        // Native cross location is ((T + item.offset_cross) + line.offset).
        // Native main location is T + item.offset_main. Zero insets add exactly.
        let local=total.add(item_offset)?;
        let location=local.add(offset)?;
        // Cross update T += line.offset + line.size; main update T +=
        // item.offset + zero margins + item.size, with the same order.
        let step=offset.add(sizes.into())?;
        total=total.add(step)?;
        peak=peak.max(local.magnitude()).max(location.magnitude()).max(step.magnitude()).max(total.magnitude());
        accumulator=accumulator.hull(total);locations=locations.hull(location);
    }
    Ok(Axis{sum,first_offset,accumulator,locations,peak})
}
fn up(a:f64,b:f64)->f64 {(a+b).next_up()}
fn mul_up(a:f64,b:f64)->f64 {(a*b).next_up()}
fn up32(v:f64)->Result<f32,Unresolved> {
    let r=v as f32;let r=if (r as f64)<v {r.next_up()}else{r};
    if r.is_finite(){Ok(r)}else{Err(Unresolved::Arithmetic)}
}
fn down32(v:f64)->Result<f32,Unresolved> {
    let r=v as f32;let r=if (r as f64)>v {r.next_down()}else{r};
    if r.is_finite(){Ok(r)}else{Err(Unresolved::Arithmetic)}
}

pub(super) struct Proof {
    axes:[Axis;2],
    visible_world:Vec<[Range;2]>,
    radius:f32,
    epsilon:f32,
    same_line_error:f64,
    different_line_error:f64,
    normalizer:wrapping_normalizer::Proof,
    measured:Vec<wrapping_carry::Measured>,
    carry:wrapping_carry::Proof,
}
impl Proof {
    pub(super) fn axes(&self)->&[Axis;2] {&self.axes}
    pub(super) fn visible_world(&self)->&[[Range;2]] {&self.visible_world}
    pub(super) fn radius(&self)->f32 {self.radius}
    pub(super) fn epsilon(&self)->f32 {self.epsilon}
    pub(super) fn same_line_error(&self)->f64 {self.same_line_error}
    pub(super) fn different_line_error(&self)->f64 {self.different_line_error}
    pub(super) fn normalizer(&self)->&wrapping_normalizer::Proof {&self.normalizer}
    pub(super) fn measured(&self)->&[wrapping_carry::Measured] {&self.measured}
    pub(super) fn carry(&self)->&wrapping_carry::Proof {&self.carry}
}

/// Derive from the complete checked domain, not a captured browser layout.
/// The31-operation anchor/pair ledger and31+4N different-line ledger remain
/// conditional on the audited scalar graph. This does not certify masks or
/// claim Chrome fidelity. Failure is an unresolved sufficient certificate.
pub(super) fn prove(domains:&Domains<'_>)->Result<Proof,Unresolved> {
    let n=domains.slots().len();
    let operations=n.checked_mul(4).and_then(|v|v.checked_add(31))
        .filter(|v|*v<=u32::MAX as usize).ok_or(Unresolved::Count)?;
    let base=domains.base();let cross=usize::from(base.row);let main=1-cross;
    let physical=|f:f32,reverse:bool|if reverse {1.-f}else{f};
    let main_sizes=domains.slots().iter().map(|s|s.axes[main]).collect::<Vec<_>>();
    let cross_sizes=domains.slots().iter().map(|s|s.axes[cross]).collect::<Vec<_>>();
    let main_bounds=axis(domains.parent_axes()[main],&main_sizes,physical(base.main_fraction,base.reverse_main),false)?;
    let cross_bounds=axis(domains.parent_axes()[cross],&cross_sizes,physical(base.line_fraction,base.reverse_cross),true)?;
    let axes=if base.row {[main_bounds,cross_bounds]}else{[cross_bounds,main_bounds]};
    // TransformConstraint copies both coordinates before scalar projection.
    // Also check the unmodified initial visible world transforms: strength-one
    // TranslationConstraint still computes old*0, which cannot repair infinity.
    let mut visible_world=Vec::with_capacity(n);
    for slot in domains.slots() {
        let mut world=[Range::zero();2];
        for physical_axis in 0..2 {
            let free=Range::from(slot.axes[physical_axis]).sub(slot.visible_axes[physical_axis].into())?;
            let local=free.scale(slot.visible_fraction[physical_axis])?;
            world[physical_axis]=axes[physical_axis].locations.add(local)?;
            axes[physical_axis].locations.add(Range::new(0.,slot.axes[physical_axis].upper())?)?;
        }
        visible_world.push(world);
    }
    let h=domains.cross_extents().upper();
    // B is now a checked machine envelope for actual native intermediates,
    // rather than an exact-real sum assumed to cover its own rounding error.
    let radius=up32(mul_up(4.,up(up(axes[cross].peak as f64,h as f64),1.)))?;
    let eta=up(mul_up(U,radius as f64),TINY);
    let same_line_error=mul_up(31.,eta);
    let different_line_error=mul_up(operations as f64,eta);
    let epsilon=up32(same_line_error)?;
    if !(0. ..65536.).contains(&epsilon) {return Err(Unresolved::Separation);}
    let lower=down32((domains.cross_extents().lower() as f64-different_line_error).next_down())?;
    if lower<=epsilon {return Err(Unresolved::Separation);}
    let anchor=axes[cross].locations.add(Range::new(0.,h)?)?;
    // The independent range encloses any anchor pair. Add the full pair
    // budget conservatively to cover the template's local reconstruction.
    let absolute_upper=up32(up((anchor.upper as f64-anchor.lower as f64).next_up(),same_line_error))?;
    if !(absolute_upper*2.).is_finite() {return Err(Unresolved::Arithmetic);}
    let dead=MachineInterval::new(lower-epsilon,absolute_upper-epsilon).map_err(|_|Unresolved::Separation)?;
    let normalizer=wrapping_normalizer::prove(dead).map_err(Unresolved::Normalizer)?;
    let measured=cross_sizes.iter().map(|size| {
        let lower=down32((size.lower() as f64-same_line_error).next_down())?;
        let upper=up32(up(size.upper() as f64,same_line_error))?;
        let machine=MachineInterval::new(lower,upper).map_err(|_|Unresolved::Separation)?;
        Ok(wrapping_carry::Measured{machine,error:same_line_error})
    }).collect::<Result<Vec<_>,Unresolved>>()?;
    let carry=wrapping_carry::prove(&measured).map_err(Unresolved::Carry)?;
    Ok(Proof{axes,visible_world,radius,epsilon,same_line_error,different_line_error,normalizer,measured,carry})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(v:f32)->MachineInterval {MachineInterval::new(v,v).unwrap()}
    fn contains(range:Range,value:f32) {assert!(range.lower<=value && value<=range.upper,"{range:?} excludes {value}");}
    #[test]
    fn recurrence_encloses_every_partition_and_positional_fraction() {
        let sizes=[1.,30.,50.,0.125,80.];
        let intervals=sizes.map(point);
        for parent in [0.,1.,40.,120.,8192.] {for fraction in [0.,0.5,1.] {
            let bounds=axis(point(parent),&intervals,fraction,true).unwrap();
            for partition in 0..(1<<(sizes.len()-1)) {
                let mut lines=vec![vec![sizes[0]]];
                for i in 1..sizes.len() {
                    if partition&(1<<(i-1))!=0 {lines.push(vec![]);}
                    lines.last_mut().unwrap().push(sizes[i]);
                }
                let extents=lines.iter().map(|l|l.iter().copied().fold(0.,f32::max)).collect::<Vec<_>>();
                let sum=extents.iter().sum::<f32>();contains(bounds.sum,sum);
                let first=(parent-sum)*fraction;contains(bounds.first_offset,first);
                let mut t=0.;
                for (i,line) in lines.iter().enumerate() {
                    let offset=if i==0 {first}else{0.};
                    for &size in line {
                        let loc=(t+(extents[i]-size)*fraction)+offset;
                        contains(bounds.locations,loc);
                    }
                    t+=offset+extents[i];contains(bounds.accumulator,t);
                }
            }
        }}
    }
    #[test]
    fn main_alignment_overflow_and_native_sum_overflow_are_checked() {
        let sizes=[point(50.),point(80.),point(30.)];
        for fraction in [0.,0.5,1.] {
            let bounds=axis(point(20.),&sizes,fraction,false).unwrap();
            let offset=(20.-160.)*fraction;let mut t=0.;
            for (i,size) in sizes.iter().enumerate() {
                let o=if i==0 {offset}else{0.};contains(bounds.locations,t+o);
                t+=o+size.upper();contains(bounds.accumulator,t);
            }
        }
        assert!(matches!(axis(point(1.),&[point(f32::MAX);2],0.,true),Err(Unresolved::Arithmetic)));
        assert!(matches!(axis(point(1.),&sizes,0.25,false),Err(Unresolved::Fraction)));
    }
}
