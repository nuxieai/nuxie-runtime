//! Source-bound fixed line stretching. This certificate does not admit a public
//! feature or authorize a native graph: callers still bind every emitted role
//! and discharge the existing arithmetic, position, scalar and paint proofs.
use super::{fixed_layout::{LayoutLength, LayoutStyle, FixedLayoutLength},
    scalar_provenance::ScalarProvenance};

// At this bound every nonnegative LayoutUnit sum below is exactly an f32.
const LIMIT: i64 = 65_536 * 64;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Empty, FixedRequired, InvalidSource, Negative,
    ArithmeticBound, Remainder, NativePartition, InvalidBinding }
#[derive(Clone, Debug, PartialEq, Eq)]
struct Calculation { used: Vec<[i64; 2]>, lines: Vec<usize>, heights: Vec<i64>, expanded: Vec<i64> }
#[derive(Clone, Debug)]
pub(super) struct Plan {
    parent: LayoutStyle,
    children: Vec<LayoutStyle>,
    row: bool,
    // Immutable source witness, separate from derived constants. It also makes
    // internal mutation controls distinguish changed inactive authored bounds.
    source: (LayoutStyle, Vec<LayoutStyle>, bool),
    calculation: Calculation,
    slots: Vec<LayoutStyle>,
}
fn fixed(v: &LayoutLength) -> Result<i64, Unresolved> {
    let LayoutLength::Fixed(v) = v else { return Err(Unresolved::FixedRequired); };
    v.validate().map_err(|_| Unresolved::InvalidSource)?;
    // Check computed sign before quantization; a negative tiny source must not
    // pass because conversion has produced zero.
    if v.authored().native() < 0. { return Err(Unresolved::Negative); }
    let raw = i64::from(v.raw_units());
    if !(0..=LIMIT).contains(&raw) { return Err(Unresolved::ArithmeticBound); }
    Ok(raw)
}
fn used(s: &LayoutStyle) -> Result<[i64; 2], Unresolved> {
    let axis = |preferred: &LayoutLength, minimum: &LayoutLength, maximum: &LayoutLength| -> Result<i64, Unresolved> {
        let p = fixed(preferred)?; let min = fixed(minimum)?;
        let max = match maximum { LayoutLength::Auto => LIMIT, _ => fixed(maximum)? };
        Ok(p.min(max).max(min)) // CSS minimum wins over a conflicting maximum.
    };
    Ok([axis(&s.width, &s.min_width, &s.max_width)?, axis(&s.height, &s.min_height, &s.max_height)?])
}
fn add(a: i64, b: i64) -> Result<i64, Unresolved> {
    let result = a.checked_add(b).ok_or(Unresolved::ArithmeticBound)?;
    if result > LIMIT { Err(Unresolved::ArithmeticBound) } else { Ok(result) }
}
fn native(raw: i64) -> Result<f32, Unresolved> {
    let exact = raw as f64 / 64.; let f = exact as f32;
    if f64::from(f) != exact { Err(Unresolved::ArithmeticBound) } else { Ok(f) }
}
fn calculate(parent: &LayoutStyle, children: &[LayoutStyle], row: bool) -> Result<Calculation, Unresolved> {
    if children.is_empty() { return Err(Unresolved::Empty); }
    let p = used(parent)?; let values = children.iter().map(used).collect::<Result<Vec<_>, _>>()?;
    let main = usize::from(!row); let cross = 1 - main;
    let mut lines = Vec::new(); let mut heights = vec![0];
    let mut sum = 0; let mut float_sum = 0_f32; let mut count = 0;
    // Native collect_flex_lines adds the next hypothetical outer main size,
    // then breaks only when > available AND the item is not first. Insets/gaps
    // and flex factors must remain zero in the caller's independent binder.
    for v in &values {
        let next = add(sum, v[main])?;
        let float_next = float_sum + native(v[main])?;
        let integer_break = count != 0 && next > p[main];
        let float_break = count != 0 && float_next > native(p[main])?;
        if integer_break != float_break || f64::from(float_next) != next as f64 / 64. {
            return Err(Unresolved::NativePartition);
        }
        if integer_break { heights.push(0); sum = v[main]; float_sum = native(sum)?; count = 0; }
        else { sum = next; float_sum = float_next; }
        count += 1;
        let line = heights.len() - 1; lines.push(line); heights[line] = heights[line].max(v[cross]);
    }
    let total = heights.iter().try_fold(0, |a, b| add(a, *b))?;
    let free = (p[cross] - total).max(0);
    let k = i64::try_from(heights.len()).map_err(|_| Unresolved::ArithmeticBound)?;
    if free % k != 0 { return Err(Unresolved::Remainder); }
    let delta = free / k;
    let expanded = heights.iter().map(|h| add(*h, delta)).collect::<Result<Vec<_>, _>>()?;
    let mut end = 0;
    for h in &expanded { end = add(end, *h)?; native(end)?; }
    Ok(Calculation { used: values, lines, heights, expanded })
}
fn generated(raw: i64) -> Result<LayoutLength, Unresolved> {
    // These are compiler-generated constants, justified by Calculation. They
    // never replace the parent's or visible child's authored provenance.
    let value = native(raw)?;
    let source = ScalarProvenance::exact_constant(value).map_err(|_| Unresolved::InvalidSource)?;
    let fixed = FixedLayoutLength::new(source).map_err(|_| Unresolved::InvalidSource)?;
    if i64::from(fixed.raw_units()) != raw { return Err(Unresolved::InvalidBinding); }
    Ok(LayoutLength::Fixed(fixed))
}
fn slots(c: &Calculation, row: bool) -> Result<Vec<LayoutStyle>, Unresolved> {
    c.used.iter().enumerate().map(|(i, axes)| {
        let main = usize::from(!row); let mut size = *axes;
        size[1-main] = c.expanded[c.lines[i]];
        Ok(LayoutStyle { width: generated(size[0])?, height: generated(size[1])?,
            min_width: generated(0)?, min_height: generated(0)?,
            max_width: LayoutLength::Auto, max_height: LayoutLength::Auto })
    }).collect()
}
fn same_length(a: &LayoutLength, b: &LayoutLength) -> bool {
    match (a,b) {
        (LayoutLength::Auto, LayoutLength::Auto) => true,
        (LayoutLength::Fixed(a), LayoutLength::Fixed(b)) => a.validate().is_ok() && b.validate().is_ok()
            && a.computed_bits() == b.computed_bits() && a.raw_units() == b.raw_units()
            && a.emitted().to_bits() == b.emitted().to_bits()
            && a.authored().proves_equal(b.authored()),
        _ => false,
    }
}
fn same(a: &LayoutStyle, b: &LayoutStyle) -> bool {
    [&a.width,&a.height,&a.min_width,&a.min_height,&a.max_width,&a.max_height].into_iter()
        .zip([&b.width,&b.height,&b.min_width,&b.min_height,&b.max_width,&b.max_height])
        .all(|(a,b)| same_length(a,b))
}
impl Plan {
    pub(super) fn new(parent: &LayoutStyle, children: &[LayoutStyle], row: bool) -> Result<Self, Unresolved> {
        let calculation = calculate(parent, children, row)?;
        let slots = slots(&calculation, row)?;
        let plan = Self { parent: parent.clone(), children: children.to_vec(), row,
            source: (parent.clone(), children.to_vec(), row), calculation, slots };
        plan.validate()?; Ok(plan)
    }
    pub(super) fn slots(&self) -> &[LayoutStyle] { &self.slots }
    pub(super) fn parent(&self) -> &LayoutStyle { &self.parent }
    pub(super) fn children(&self) -> &[LayoutStyle] { &self.children }
    pub(super) fn matches_sources(&self, parent: &LayoutStyle, children: &[LayoutStyle], row: bool) -> bool {
        self.row == row && self.source.2 == row && same(&self.source.0, parent) && children.len() == self.source.1.len()
            && children.iter().zip(&self.source.1).all(|(a,b)| same(a,b))
    }
    pub(super) fn validate(&self) -> Result<(), Unresolved> {
        if !self.matches_sources(&self.parent, &self.children, self.row) { return Err(Unresolved::InvalidBinding); }
        let expected = calculate(&self.parent, &self.children, self.row)?;
        if expected != self.calculation { return Err(Unresolved::InvalidBinding); }
        let expected_slots = slots(&expected, self.row)?;
        if expected_slots.len() != self.slots.len() || !expected_slots.iter().zip(&self.slots).all(|(a,b)| same(a,b)) {
            return Err(Unresolved::InvalidBinding);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn box_(w: i64, h: i64) -> LayoutStyle {
        LayoutStyle { width: generated(w*64).unwrap(), height: generated(h*64).unwrap(),
            min_width: generated(0).unwrap(), min_height: generated(0).unwrap(),
            max_width: LayoutLength::Auto, max_height: LayoutLength::Auto }
    }
    #[test] fn equal_lines_and_column_axes() {
        let p = Plan::new(&box_(20,120), &vec![box_(20,20);3], true).unwrap();
        assert_eq!(p.calculation.lines, vec![0,1,2]);
        assert_eq!(p.calculation.expanded, vec![40*64;3]);
        assert!(p.slots().iter().all(|s| s.height.value()==Some(40.)));
        let p = Plan::new(&box_(120,20), &vec![box_(20,20);3], false).unwrap();
        assert!(p.slots().iter().all(|s| s.width.value()==Some(40.)));
    }
    #[test] fn unequal_lines_exact_fit_oversize_zero_and_overflow() {
        let children = vec![box_(30,10),box_(0,40),box_(20,20),box_(20,30)];
        let p = Plan::new(&box_(20,50), &children, true).unwrap();
        assert_eq!(p.calculation.lines, vec![0,1,1,2]);
        assert_eq!(p.calculation.expanded, vec![640,2560,1920]);
        let exact = Plan::new(&box_(40,20), &vec![box_(20,20);2], true).unwrap();
        assert_eq!(exact.calculation.lines,vec![0,0]);
        let zero = Plan::new(&box_(0,0), &vec![box_(0,0);2], true).unwrap();
        assert_eq!(zero.calculation.expanded,vec![0]);
    }
    #[test] fn bounds_minimum_wins_and_original_is_retained() {
        let mut child=box_(10,10);child.min_width=generated(30*64).unwrap();child.max_width=generated(20*64).unwrap();
        child.max_height=generated(5*64).unwrap();
        let p=Plan::new(&box_(30,40), &[child.clone()], true).unwrap();
        assert_eq!(p.slots()[0].width.value(),Some(30.));
        assert_eq!(p.slots()[0].height.value(),Some(40.));
        assert!(matches!(p.slots()[0].max_height,LayoutLength::Auto));
        assert!(same(&p.children()[0],&child));
        assert_eq!(p.children()[0].max_height.value(),Some(5.));
    }
    #[test] fn fractional_boundary_and_remainder_rejection() {
        let mut parent=box_(40,40);parent.width=generated(40*64-1).unwrap();
        assert_eq!(Plan::new(&parent,&vec![box_(20,20);2],true).unwrap().calculation.lines,vec![0,1]);
        parent.width=generated(40*64+1).unwrap();
        assert_eq!(Plan::new(&parent,&vec![box_(20,20);2],true).unwrap().calculation.lines,vec![0,0]);
        assert_eq!(Plan::new(&box_(20,100),&vec![box_(20,20);3],true).unwrap_err(),Unresolved::Remainder);
        assert_eq!(Plan::new(&box_(20,20),&[],true).unwrap_err(),Unresolved::Empty);
        assert!(Plan::new(&box_(65_537,20),&[box_(1,1)],true).is_err());
    }
    #[test] fn every_derived_field_and_source_bound_mutation_is_rejected() {
        let p=Plan::new(&box_(20,120),&vec![box_(20,20);3],true).unwrap();
        assert!(p.clone().validate().is_ok());
        let mut changed=p.clone();changed.calculation.lines[1]=0;assert!(changed.validate().is_err());
        let mut changed=p.clone();changed.slots[0].height=generated(41*64).unwrap();assert!(changed.validate().is_err());
        let mut changed=p.clone();changed.slots[0].max_height=generated(20*64).unwrap();assert!(changed.validate().is_err());
        let mut changed=p.clone();changed.children[0].max_height=generated(100*64).unwrap();assert!(changed.validate().is_err());
        let mut other=p.children.clone();other[0].max_width=generated(100*64).unwrap();
        assert!(!p.matches_sources(p.parent(),&other,true));
        assert!(!p.matches_sources(p.parent(),p.children(),false));
    }
}
