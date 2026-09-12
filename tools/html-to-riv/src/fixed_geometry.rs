//! Exact initial layout seeds for a closed, source-bound fixed wrapping base.
//! Does not evaluate sizing constraints or supply final painted corners.
use super::{fixed_layout::{LayoutLength, LayoutStyle}, wrapping_domains::{Domains, LayoutAuthored}, wrapping_sizes::MachineInterval};
use crate::wire::{self, Record};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved { Source, FixedRequired, NonSingleton, Bounds, Alignment, Encoding }
#[derive(Clone, Copy, Debug)]
pub(super) struct Seed {
    pub(super) local: [f32; 2],
    pub(super) size: [f32; 2],
    pub(super) world: [f32; 6],
}
#[derive(Clone, Debug)]
pub(super) struct Binding {
    base: Vec<u8>,
    base_record_count: usize,
    source: LayoutAuthored,
    seeds: BTreeMap<u32, Seed>,
}
impl Binding {
    pub(super) fn base_record_count(&self) -> usize { self.base_record_count }
    pub(super) fn seeds(&self) -> &BTreeMap<u32, Seed> { &self.seeds }
    pub(super) fn source(&self) -> &LayoutAuthored { &self.source }
    pub(super) fn matches_source(&self, source: &LayoutAuthored) -> bool {
        same_sources(&self.source, source)
    }
    /// Rebind actual records and compare every source field's ideal identity,
    /// computed bits and derived encoding, including inactive bounds.
    pub(super) fn matches_domains(&self, domains: &Domains<'_>) -> bool {
        let Ok(other)=bind(domains) else{return false;};
        self.same_binding(&other)
    }
    pub(super) fn same_binding(&self, other: &Self) -> bool {
        self.base_record_count==other.base_record_count && self.base==other.base && same_sources(&self.source,&other.source)
            && self.seeds.len()==other.seeds.len() && self.seeds.iter().all(|(id,a)|other.seeds.get(id).is_some_and(|b|
                a.local.map(f32::to_bits)==b.local.map(f32::to_bits)
                && a.size.map(f32::to_bits)==b.size.map(f32::to_bits)
                && a.world.map(f32::to_bits)==b.world.map(f32::to_bits)))
    }
    pub(super) fn matches_base(&self, records: &[Record]) -> bool {
        wire::encode(records).is_ok_and(|bytes| bytes == self.base)
    }
}
fn same_style(a:&LayoutStyle,b:&LayoutStyle)->bool{
    [&a.width,&a.height,&a.min_width,&a.min_height,&a.max_width,&a.max_height].into_iter()
        .zip([&b.width,&b.height,&b.min_width,&b.min_height,&b.max_width,&b.max_height])
        .all(|(a,b)|match(a,b){
            (LayoutLength::Auto,LayoutLength::Auto)=>true,
            (LayoutLength::Fixed(a),LayoutLength::Fixed(b))=>a.validate().is_ok()&&b.validate().is_ok()
                &&a.computed_bits()==b.computed_bits()&&a.raw_units()==b.raw_units()
                &&a.emitted().to_bits()==b.emitted().to_bits()&&a.authored().proves_equal(b.authored()),
            _=>false,
        })
}
fn same_sources(a:&LayoutAuthored,b:&LayoutAuthored)->bool{
    a.parent.0==b.parent.0&&same_style(&a.parent.1,&b.parent.1)&&a.roles.len()==b.roles.len()
        &&a.roles.iter().all(|(id,s)|b.roles.get(id).is_some_and(|t|same_style(s,t)))
}
// All admitted used dimensions are positive LayoutUnit values. With this bound
// their packing sums and positional half-unit offsets remain exactly f32.
const LIMIT: f32 = 65536.;
fn fixed_style(s: &LayoutStyle) -> Result<(), Unresolved> {
    for (v, auto) in [(&s.width,false),(&s.height,false),(&s.min_width,false),
        (&s.min_height,false),(&s.max_width,true),(&s.max_height,true)] {
        match v {
            LayoutLength::Auto if auto => (),
            LayoutLength::Fixed(f) => {
                f.validate().map_err(|_| Unresolved::Source)?;
                if f.authored().native() < 0. || !(0. ..=LIMIT).contains(&f.emitted()) { return Err(Unresolved::Bounds); }
            }
            _ => return Err(Unresolved::FixedRequired),
        }
    }
    Ok(())
}
fn singleton(v: MachineInterval) -> Result<f32, Unresolved> {
    if v.lower().to_bits() != v.upper().to_bits() { return Err(Unresolved::NonSingleton); }
    let x=v.lower();
    if !(x > 0. && x <= LIMIT && (x*64.).fract()==0.) { return Err(Unresolved::Bounds); }
    Ok(x)
}
fn size(v: [MachineInterval; 2]) -> Result<[f32;2],Unresolved> { Ok([singleton(v[0])?,singleton(v[1])?]) }
fn add(a:f32,b:f32)->Result<f32,Unresolved>{
    let result=a+b;
    if !result.is_finite() || result.abs()>LIMIT || f64::from(result)!=f64::from(a)+f64::from(b) { Err(Unresolved::Bounds) } else { Ok(result) }
}
fn fraction(f:f32, reverse:bool)->Result<f32,Unresolved>{
    if ![0.,0.5,1.].contains(&f) {return Err(Unresolved::Alignment);}
    Ok(if reverse {1.-f}else{f})
}
fn matrix(x:f32,y:f32)->[f32;6]{[1.,0.,0.,1.,x,y]}
// Same explicit multiply/FMA order as immutable Mat2D; even identity-linear
// matrices retain the native zero products when forming world translation.
fn multiply(a:[f32;6],b:[f32;6])->[f32;6]{[
    a[0].mul_add(b[0],a[2]*b[1]),a[1].mul_add(b[0],a[3]*b[1]),
    a[0].mul_add(b[2],a[2]*b[3]),a[1].mul_add(b[2],a[3]*b[3]),
    a[0].mul_add(b[4],a[2]*b[5])+a[4],a[1].mul_add(b[4],a[3]*b[5])+a[5],
]}
fn seed(local:[f32;2],size:[f32;2],parent:[f32;6])->Seed{
    Seed{local,size,world:multiply(multiply(parent,matrix(local[0],local[1])),matrix(0.,0.))}
}
struct Line { indices:Vec<usize>, main:f32, cross:f32 }
fn positions(parent:[f32;2],sizes:&[[f32;2]],row:bool,reverse_main:bool,reverse_cross:bool,main_fraction:f32,line_fraction:f32)->Result<Vec<[f32;2]>,Unresolved>{
    let main=usize::from(!row);let cross=1-main;
    let mf=fraction(main_fraction,reverse_main)?;let cf=fraction(line_fraction,reverse_cross)?;
    let mut lines=Vec::<Line>::new();
    for (i,s) in sizes.iter().enumerate(){
        let next=if let Some(line)=lines.last(){add(line.main,s[main])?}else{s[main]};
        if lines.last().is_none_or(|line| !line.indices.is_empty() && next>parent[main]){
            lines.push(Line{indices:vec![i],main:s[main],cross:s[cross]});
        }else{let line=lines.last_mut().unwrap();line.indices.push(i);line.main=next;line.cross=line.cross.max(s[cross]);}
    }
    let total=lines.iter().try_fold(0.,|sum,l|add(sum,l.cross))?;
    let free=(parent[cross]-total)-0.;
    let mut traversal=(0..lines.len()).collect::<Vec<_>>();if reverse_cross{traversal.reverse();}
    let mut result=vec![[0.;2];sizes.len()];let mut t=0.;
    for (physical_line,j) in traversal.into_iter().enumerate(){
        let line=&lines[j];let line_offset=if physical_line==0{free*cf}else{0.};
        let main_free=parent[main]-(0.+line.main);
        let mut indices=line.indices.clone();if reverse_main{indices.reverse();}
        let mut u=0.;
        for (physical_item,i) in indices.into_iter().enumerate(){
            let offset=if physical_item==0{main_free*mf}else{0.};
            let cross_offset=(line.cross-sizes[i][cross])*cf;
            result[i][main]=add(add(add(u,offset)?,0.)?,0.)?;
            result[i][cross]=add(add(add(add(t,cross_offset)?,line_offset)?,0.)?,0.)?;
            u=add(u,add(add(offset,0.)?,sizes[i][main])?)?;
        }
        t=add(t,add(line_offset,line.cross)?)?;
    }
    Ok(result)
}

pub(super) fn bind(domains:&Domains<'_>)->Result<Binding,Unresolved>{
    let source=domains.layout_authored().ok_or(Unresolved::Source)?;
    fixed_style(&source.parent.1)?;
    for s in source.roles.values(){fixed_style(s)?;}
    let base=domains.base();let parent=size(domains.parent_axes())?;
    let sizes=domains.slots().iter().map(|s|size(s.axes)).collect::<Result<Vec<_>,_>>()?;
    let visible=domains.slots().iter().map(|s|{
        // Bound first experiment: default top-left local child placement. No
        // hidden parent-fraction reconstruction is delegated to the caller.
        if s.visible_fraction != [0.,0.] {return Err(Unresolved::Alignment);}
        size(s.visible_axes)
    }).collect::<Result<Vec<_>,_>>()?;
    let locations=positions(parent,&sizes,base.row,base.reverse_main,base.reverse_cross,base.main_fraction,base.line_fraction)?;
    let parent_seed=seed([0.,0.],parent,matrix(0.,0.));
    let mut seeds=BTreeMap::from([(base.parent,parent_seed)]);
    for (i,slot) in domains.slots().iter().enumerate(){
        let slot_seed=seed(locations[i],sizes[i],parent_seed.world);
        let visible_seed=seed([0.,0.],visible[i],slot_seed.world);
        seeds.insert(slot.object,slot_seed);seeds.insert(slot.visible,visible_seed);
    }
    Ok(Binding{base:wire::encode(base.records()).map_err(|_|Unresolved::Encoding)?,base_record_count:base.records().len(),source:source.clone(),seeds})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<Record>,LayoutStyle,LayoutStyle){
        use super::super::{fixed_layout::FixedLayoutLength,scalar_provenance::ScalarProvenance};
        use crate::wire::Value;
        let length=|v|LayoutLength::Fixed(FixedLayoutLength::new(ScalarProvenance::exact_constant(v).unwrap()).unwrap());
        let style=|w,h|LayoutStyle{width:length(w),height:length(h),min_width:length(0.),min_height:length(0.),max_width:LayoutLength::Auto,max_height:LayoutLength::Auto};
        let p=style(40.,100.);let c=style(20.,20.);
        let mut r=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponentStyle")];
        r[1].set("styleId",Value::Uint(1)).unwrap();r[1].set("width",Value::Float(320.)).unwrap();r[1].set("height",Value::Float(240.)).unwrap();
        for (id,parent,style) in [(2,0,&p),(4,2,&c),(6,4,&c)]{
            let mut n=Record::new("LayoutComponent");n.set("parentId",Value::Uint(parent)).unwrap();n.set("styleId",Value::Uint(id+1)).unwrap();
            n.set("width",Value::Float(style.width.value().unwrap())).unwrap();n.set("height",Value::Float(style.height.value().unwrap())).unwrap();
            let mut t=Record::new("LayoutComponentStyle");
            for key in ["widthUnitsValue","heightUnitsValue","minWidthUnitsValue","minHeightUnitsValue"]{t.set(key,Value::Uint(1)).unwrap();}
            for key in ["minWidth","minHeight"]{t.set(key,Value::Float(0.)).unwrap();}
            if id==2{t.set("flexWrapValue",Value::Uint(1)).unwrap();}
            r.extend([n,t]);
        }
        let mut fill=Record::new("Fill");fill.set("parentId",Value::Uint(6)).unwrap();
        let mut color=Record::new("SolidColor");color.set("parentId",Value::Uint(8)).unwrap();color.set("colorValue",Value::Color(0xffff0000)).unwrap();r.extend([fill,color]);(r,p,c)
    }
    #[test] fn source_and_actual_base_binding(){
        use super::super::wrapping_domains;
        use crate::wire::Value;
        let (records,p,c)=fixture();let viewport=[MachineInterval::new(0.,16384.).unwrap();2];
        let domains=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&c),(6,&c)],viewport).unwrap();
        let b=bind(&domains).unwrap();assert!(b.matches_domains(&domains));assert_eq!(b.seeds().len(),3);assert!(b.matches_base(&records));assert_eq!(b.source().parent.0,2);
        assert_eq!(b.seeds()[&6].world,matrix(0.,0.));
        let mut changed=records.clone();changed[10].set("colorValue",Value::Color(0)).unwrap();assert!(!b.matches_base(&changed));
        let mut changed=records.clone();changed[5].set("width",Value::Float(21.)).unwrap();
        assert!(wrapping_domains::resolve_layout(&changed,2,&[(4,6)],&p,&[(4,&c),(6,&c)],viewport).is_err());
        let mut percent=c.clone();percent.width=LayoutLength::Percent(super::super::scalar_provenance::ScalarProvenance::exact_constant(20.).unwrap());assert_eq!(fixed_style(&percent),Err(Unresolved::FixedRequired));
        let mut aligned=records.clone();aligned[6].set("layoutAlignmentType",Value::Uint(1)).unwrap();
        let d=wrapping_domains::resolve_layout(&aligned,2,&[(4,6)],&p,&[(4,&c),(6,&c)],viewport).unwrap();assert!(matches!(bind(&d),Err(Unresolved::Alignment)));
    }
    #[test] fn oversized_first_and_exact_fit_adjacent_boundary(){
        assert_eq!(positions([20.,100.],&[[30.,10.],[20.,20.]],true,false,false,0.,0.).unwrap(),vec![[0.,0.],[0.,10.]]);
        let items=[[20.,10.],[20.,20.]];
        assert_eq!(positions([40.,100.],&items,true,false,false,0.,0.).unwrap(),vec![[0.,0.],[20.,0.]]);
        assert_eq!(positions([40.-1./64.,100.],&items,true,false,false,0.,0.).unwrap(),vec![[0.,0.],[0.,10.]]);
    }
    #[test] fn same_encoded_base_different_ideal_source_does_not_rebind(){
        use super::super::{wrapping_domains,fixed_layout::FixedLayoutLength,scalar_provenance::ScalarProvenance};
        let (records,p,c)=fixture();let viewport=[MachineInterval::new(0.,16384.).unwrap();2];
        let d=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&c),(6,&c)],viewport).unwrap();let b=bind(&d).unwrap();
        let mut changed=c.clone();changed.width=LayoutLength::Fixed(FixedLayoutLength::new(ScalarProvenance::from_decimal("20.0000001",20.).unwrap()).unwrap());
        let other=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&changed),(6,&c)],viewport).unwrap();
        assert!(b.matches_base(&records));assert!(bind(&other).is_ok());assert!(!b.matches_domains(&other));
        let opaque=||LayoutLength::Fixed(FixedLayoutLength::new(ScalarProvenance::exact_constant(20.).unwrap().multiply(&ScalarProvenance::exact_constant(1.).unwrap()).unwrap()).unwrap());
        let mut a=c.clone();a.width=opaque();let mut z=c.clone();z.width=opaque();
        let da=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&a),(6,&c)],viewport).unwrap();
        let dz=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&z),(6,&c)],viewport).unwrap();
        let ba=bind(&da).unwrap();assert!(ba.matches_domains(&da));assert!(!ba.matches_domains(&dz));
    }
    #[test] fn parent_child_min_wins_and_distinct_visible_extents(){
        use super::super::{wrapping_domains,fixed_layout::FixedLayoutLength,scalar_provenance::ScalarProvenance};
        use crate::wire::Value;
        let (mut records,mut p,mut c)=fixture();
        let length=|v|LayoutLength::Fixed(FixedLayoutLength::new(ScalarProvenance::exact_constant(v).unwrap()).unwrap());
        p.min_width=length(60.);p.max_width=length(50.);
        c.min_width=length(30.);c.max_width=length(25.);
        for (position,min,max) in [(4,60.,50.),(6,30.,25.)]{
            records[position].set("minWidth",Value::Float(min)).unwrap();records[position].set("maxWidth",Value::Float(max)).unwrap();records[position].set("maxWidthUnitsValue",Value::Uint(1)).unwrap();
        }
        let (_,_,visible)=fixture();let viewport=[MachineInterval::new(0.,16384.).unwrap();2];
        let d=wrapping_domains::resolve_layout(&records,2,&[(4,6)],&p,&[(4,&c),(6,&visible)],viewport).unwrap();let b=bind(&d).unwrap();
        assert_eq!(b.seeds()[&2].size,[60.,100.]);assert_eq!(b.seeds()[&4].size,[30.,20.]);assert_eq!(b.seeds()[&6].size,[20.,20.]);
        assert_eq!(b.seeds()[&6].world,matrix(0.,0.));
    }
    #[test] fn physical_traversal_and_item_content_coupling(){
        let s=[[20.,10.],[20.,30.],[20.,20.]];
        assert_eq!(positions([40.,100.],&s,true,false,false,0.,0.).unwrap(),vec![[0.,0.],[20.,0.],[0.,30.]]);
        assert_eq!(positions([40.,100.],&s,true,true,true,0.,0.).unwrap(),vec![[20.,90.],[0.,70.],[20.,50.]]);
        assert_eq!(positions([40.,100.],&s,true,false,false,0.,0.5).unwrap(),vec![[0.,35.],[20.,25.],[0.,55.]]);
    }
    #[test] fn column_overflow_and_fractional_offset(){
        let p=positions([100.,40.],&[[10.,20.],[30.,20.],[20.,20.]],false,true,true,0.,0.).unwrap();
        assert_eq!(p,vec![[90.,20.],[70.,0.],[50.,20.]]);
        assert_eq!(positions([20.,10.],&[[20.,30.]],true,false,true,0.,0.).unwrap(),vec![[0.,-20.]]);
        assert_eq!(positions([20.,1.],&[[20.,1./64.]],true,false,false,0.,0.5).unwrap(),vec![[0.,63./128.]]);
    }
    #[test] fn bounds_and_world_composition(){
        assert!(positions([20.,20.],&[[40000.,20.],[40000.,20.]],true,false,false,0.,0.).is_err());
        assert!(positions([20.,20.],&[[20.,20.]],true,false,false,0.,0.25).is_err());
        let s=seed([1./128.,-20.],[20.,30.],matrix(40.,10.));
        assert_eq!(s.world,matrix(40.+1./128.,-10.));assert_eq!(s.size,[20.,30.]);
        assert_eq!(s.local,[1./128.,-20.]);
    }
}
