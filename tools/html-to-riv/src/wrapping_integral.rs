//! Private selection of visible owners whose unsnapped native paint is integral.
//!
//! In the bound zero-origin/inset tree, forward start/start wrapping partitions
//! positive integral slots without introducing fractional offsets. Every line
//! main offset and cross offset is a sum of integral slot extents/maxima. Bounding
//! the sum on each axis by65536 keeps these additions exactly representable in
//! f32. A start-aligned visible owner of positive integral size consequently has
//! integral corners. Other visible children cannot affect the independent slots.
//!
//! This certificate selects an optimization only. It does not establish the
//! sizing/gate/position constraints, source field binding, renderer behavior or
//! public CSS admission; the caller must retain those independent proofs. A
//! failed premise returns an empty or mixed selection, never narrows the viewport.
use super::{wrapping_domains::Domains, wrapping_sizes::MachineInterval};
use std::collections::BTreeSet;
const EXACT_BOUND: f64 = 65_536.0;

#[derive(Clone, Debug, Default)]
pub(super) struct Proof { geometries: BTreeSet<u32> }
impl Proof {
    pub(super) fn geometries(&self) -> &BTreeSet<u32> { &self.geometries }
}
fn positive_integer(axis: MachineInterval) -> Option<f64> {
    let low=axis.lower();let high=axis.upper();
    if low != high || !low.is_finite() || low <= 0. || low.fract() != 0. {
        None
    } else { Some(f64::from(low)) }
}

pub(super) fn select(domains: &Domains<'_>, alignments: &[f32]) -> Proof {
    let empty=Proof::default();let base=domains.base();
    if domains.layout_authored().is_none() || alignments.len()!=domains.slots().len()
        || base.reverse_main || base.reverse_cross
        || base.main_fraction != 0. || base.line_fraction != 0. {
        return empty;
    }
    let mut sums=[0_f64;2];
    for slot in domains.slots() {
        for axis in 0..2 {
            let Some(value)=positive_integer(slot.axes[axis]) else { return empty; };
            // All operands are nonnegative integers. Checking each prefix
            // before continuing makes the f64 sum exact within the bound.
            if value > EXACT_BOUND || sums[axis]+value > EXACT_BOUND { return empty; }
            sums[axis]+=value;
        }
    }
    let mut geometries=BTreeSet::new();
    for (slot,&alignment) in domains.slots().iter().zip(alignments) {
        if alignment != 0. || slot.visible_fraction != [0.,0.] { continue; }
        if (0..2).all(|axis|positive_integer(slot.visible_axes[axis])
            .is_some_and(|value|value<=EXACT_BOUND && sums[axis]+value<=EXACT_BOUND)) {
            geometries.insert(slot.visible);
        }
    }
    Proof { geometries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{computed_provenance::{NumericSize,NumericStyle},fixed_layout::LayoutStyle,
        scalar_provenance::ScalarProvenance,wrapping_domains};
    use crate::wire::{Record,Value};
    fn numeric(w:f32,h:f32,percent:bool)->NumericStyle {
        let value=|x|{let s=Ok(ScalarProvenance::exact_constant(x).unwrap());
            if percent{NumericSize::Percent(s)}else{NumericSize::Pixels(s)}};
        NumericStyle{width:value(w),height:value(h),..NumericStyle::default()}
    }
    fn set(r:&mut Record,k:&str,v:Value){r.set(k,v).unwrap();}
    struct Fixture { records:Vec<Record>,roles:Vec<(u32,u32)>,parent:LayoutStyle,styles:Vec<(u32,LayoutStyle)> }
    fn fixture(slots:&[[f32;2]],visible:&[[f32;2]],row:bool)->Fixture {
        assert_eq!(slots.len(),visible.len());
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];
        set(&mut r[1],"styleId",Value::Uint(1));
        fn node(r:&mut Vec<Record>,parent:u32,dims:[f32;2],percent:bool)->(u32,LayoutStyle){
            let id=r.len()as u32-1;let mut n=Record::new("LayoutComponent");let mut s=Record::new("LayoutComponentStyle");
            set(&mut n,"parentId",Value::Uint(parent));set(&mut n,"styleId",Value::Uint(id+1));
            for (axis,v) in ["width","height"].into_iter().zip(dims){set(&mut n,axis,Value::Float(v));set(&mut s,&format!("{axis}UnitsValue"),Value::Uint(if percent{2}else{1}));}
            for axis in ["Width","Height"]{set(&mut s,&format!("min{axis}"),Value::Float(0.));set(&mut s,&format!("min{axis}UnitsValue"),Value::Uint(1));}
            r.extend([n,s]);(id,LayoutStyle::from_numeric(&numeric(dims[0],dims[1],percent)).unwrap())
        }
        let (p,parent)=node(&mut r,0,[50.,50.],true);
        set(&mut r[p as usize+2],"flexDirectionValue",Value::Uint(if row{2}else{0}));set(&mut r[p as usize+2],"flexWrapValue",Value::Uint(1));
        let mut roles=Vec::new();let mut styles=Vec::new();
        for (a,b) in slots.iter().zip(visible){
            let (slot,style)=node(&mut r,p,*a,false);let (visible,vstyle)=node(&mut r,slot,*b,false);
            let fill=r.len()as u32-1;let mut f=Record::new("Fill");set(&mut f,"parentId",Value::Uint(visible));let mut c=Record::new("SolidColor");set(&mut c,"parentId",Value::Uint(fill));set(&mut c,"colorValue",Value::Color(0x80abcdef));r.extend([f,c]);roles.push((slot,visible));styles.extend([(slot,style),(visible,vstyle)]);
        }
        Fixture{records:r,roles,parent,styles}
    }
    fn domains(f:&Fixture)->Domains<'_>{
        wrapping_domains::resolve_layout(&f.records,2,&f.roles,&f.parent,
            &f.styles.iter().map(|(id,s)|(*id,s)).collect::<Vec<_>>(),[MachineInterval::new(0.,16384.).unwrap();2]).unwrap()
    }
    #[test]
    fn both_axes_select_integral_owners_over_full_responsive_parent_domain(){
        for row in [false,true]{let f=fixture(&[[60.,30.],[40.,50.]],&[[80.,30.],[40.,70.]],row);
            assert_eq!(select(&domains(&f),&[0.,0.]).geometries(),&f.roles.iter().map(|r|r.1).collect::<BTreeSet<_>>());
            assert!(select(&domains(&f),&[0.]).geometries().is_empty());}
    }
    #[test]
    fn fractional_or_moving_visible_owners_leave_other_independent_owners_eligible(){
        let mut f=fixture(&[[60.,30.],[40.,50.]],&[[80.,30.],[40.5,70.]],true);
        assert_eq!(select(&domains(&f),&[0.,0.]).geometries(),&BTreeSet::from([f.roles[0].1]));
        assert!(select(&domains(&f),&[0.5,0.]).geometries().is_empty());
        set(&mut f.records[f.roles[0].0 as usize+2],"layoutAlignmentType",Value::Uint(4));
        assert!(select(&domains(&f),&[0.,0.]).geometries().is_empty());
        let f=fixture(&[[60.,30.],[40.,50.]],&[[0.,30.],[40.,70.]],true);
        assert_eq!(select(&domains(&f),&[0.,1.]).geometries(),&BTreeSet::new());
        assert_eq!(select(&domains(&f),&[0.,0.]).geometries(),&BTreeSet::from([f.roles[1].1]));
    }
    #[test]
    fn fractional_slots_reverse_and_nonstart_parent_layouts_fall_back(){
        let f=fixture(&[[60.5,30.],[40.,50.]],&[[80.,30.],[40.,70.]],true);
        assert!(select(&domains(&f),&[0.,0.]).geometries().is_empty());
        for (key,value) in [("flexDirectionValue",3),("flexWrapValue",2),("layoutAlignmentType",1),("layoutAlignmentType",3)]{
            let mut f=fixture(&[[60.,30.]],&[[80.,30.]],true);set(&mut f.records[4],key,Value::Uint(value));
            assert!(select(&domains(&f),&[0.]).geometries().is_empty(),"{key}");
        }
    }
    #[test]
    fn zero_and_responsive_slot_extents_fall_back_without_sampling(){
        let f=fixture(&[[0.,30.]],&[[1.,1.]],true);
        assert!(select(&domains(&f),&[0.]).geometries().is_empty());
        let mut f=fixture(&[[60.,30.]],&[[1.,1.]],true);
        let id=f.roles[0].0;set(&mut f.records[id as usize+2],"widthUnitsValue",Value::Uint(2));
        let mut n=numeric(60.,30.,false);n.width=numeric(60.,30.,true).width;
        f.styles.iter_mut().find(|(owner,_)|*owner==id).unwrap().1=LayoutStyle::from_numeric(&n).unwrap();
        assert!(select(&domains(&f),&[0.]).geometries().is_empty());
    }
    #[test]
    fn integral_sum_capacity_has_exact_boundary_and_is_per_axis(){
        for row in [false,true]{
            let f=fixture(&[[32768.,32768.]],&[[32768.,32768.]],row);
            assert_eq!(select(&domains(&f),&[0.]).geometries(),&BTreeSet::from([f.roles[0].1]));
            for axis in 0..2{let mut v=[32768.,32768.];v[axis]+=1.;let f=fixture(&[[32768.,32768.]],&[v],row);assert!(select(&domains(&f),&[0.]).geometries().is_empty());}
        }
        let f=fixture(&[[40000.,30.],[30000.,30.]],&[[1.,1.],[1.,1.]],true);
        assert!(select(&domains(&f),&[0.,0.]).geometries().is_empty());
    }
    #[test]
    fn legacy_dimensions_without_visible_authored_binding_do_not_certify(){
        let f=fixture(&[[60.,30.]],&[[80.,30.]],true);let p=numeric(50.,50.,true);let s=numeric(60.,30.,false);
        let legacy=wrapping_domains::resolve(&f.records,2,&f.roles,&p,&[(f.roles[0].0,&s)],
            [MachineInterval::new(0.,16384.).unwrap();2]).unwrap();
        assert!(select(&legacy,&[0.]).geometries().is_empty());
    }
}
