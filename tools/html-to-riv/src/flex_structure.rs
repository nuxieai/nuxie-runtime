//! Local facts from completed ordinary records. These do not establish ancestor
//! transforms, used sizes, intrinsic minima, descendant errors or CSS admission.
use crate::wire::{Record,Value};
use serde::Serialize;

#[derive(Debug,Serialize)]
pub(crate) struct LocalFacts {
    pub zero_box_insets_and_gaps:bool,
    pub zero_margins:bool,
    pub no_wrap:bool,
    pub no_aspect_ratio:bool,
    pub unset_position_insets:bool,
    pub own_transform_defaults:bool,
    pub no_direction_override:bool,
    pub no_style_interpolation:bool,
    pub ordinary_flex_layout:bool,
    pub no_layout_children:bool,
}
fn number(record:&Record,name:&str,default:f32)->Option<f32> {
    match record.get(name) {None=>Some(default),Some(Value::Float(v))=>Some(*v),_=>None}
}
fn uint(record:&Record,name:&str,default:u32)->Option<u32> {
    match record.get(name) {None=>Some(default),Some(Value::Uint(v))=>Some(*v),_=>None}
}
fn zero_lengths(record:&Record,names:&[&str])->bool {
    names.iter().all(|name|number(record,name,0.)==Some(0.) && matches!(uint(record,&format!("{name}UnitsValue"),0),Some(0..=2)))
}
impl LocalFacts {
    pub(super) fn inspect(record:&Record,style:Option<&Record>,no_layout_children:bool)->Self {
        let valid=style.filter(|s|s.kind=="LayoutComponentStyle");
        let identity=if record.kind=="Artboard" {
            ["originX","originY"].iter().all(|n|number(record,n,0.)==Some(0.))
        }else if record.kind=="LayoutComponent" {
            ["x","y","rotation"].iter().all(|n|number(record,n,0.)==Some(0.)) && ["scaleX","scaleY"].iter().all(|n|number(record,n,1.)==Some(1.))
        }else{false};
        Self {
            zero_box_insets_and_gaps:valid.is_some_and(|s|zero_lengths(s,&["paddingLeft","paddingTop","paddingRight","paddingBottom","borderLeft","borderTop","borderRight","borderBottom","gapHorizontal","gapVertical"])),
            zero_margins:valid.is_some_and(|s|zero_lengths(s,&["marginLeft","marginTop","marginRight","marginBottom"])),
            no_wrap:valid.is_some_and(|s|uint(s,"flexWrapValue",0)==Some(0)),
            no_aspect_ratio:valid.is_some_and(|s|number(s,"aspectRatio",0.)==Some(0.)),
            unset_position_insets:valid.is_some_and(|s|uint(s,"positionTypeValue",1)==Some(1) && ["positionLeft","positionTop","positionRight","positionBottom"].iter().all(|n|uint(s,&format!("{n}UnitsValue"),0)==Some(0))),
            own_transform_defaults:identity,
            // Only absence/default is admitted here; inherited LTR context still
            // needs an ancestor proof before this becomes a direction premise.
            no_direction_override:valid.is_some_and(|s|uint(s,"directionValue",0)==Some(0)),
            no_style_interpolation:valid.is_some_and(|s|uint(s,"animationStyleType",0)==Some(0) && uint(s,"interpolationType",0)==Some(0) && uint(s,"interpolatorId",u32::MAX)==Some(u32::MAX) && number(s,"interpolationTime",0.)==Some(0.)),
            ordinary_flex_layout:valid.is_some_and(|s|uint(s,"layoutTypeValue",0)==Some(0)),
            no_layout_children,
        }
    }
    pub(super) fn direct_box_defaults(&self)->bool {
        self.zero_box_insets_and_gaps && self.zero_margins && self.no_wrap && self.no_aspect_ratio && self.unset_position_insets && self.own_transform_defaults && self.no_direction_override && self.no_style_interpolation && self.ordinary_flex_layout
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_local_facts_and_overrides_are_not_hidden_by_absent_units() {
        let record=Record::new("LayoutComponent");let style=Record::new("LayoutComponentStyle");
        let facts=LocalFacts::inspect(&record,Some(&style),true);assert!(facts.direct_box_defaults());assert!(facts.no_layout_children);
        for (name,value) in [("gapHorizontal",Value::Float(3.)),("paddingLeft",Value::Float(1.)),("borderRight",Value::Float(1.)),("marginLeftUnitsValue",Value::Uint(3)),("flexWrapValue",Value::Uint(1)),("aspectRatio",Value::Float(2.)),("positionLeftUnitsValue",Value::Uint(1)),("directionValue",Value::Uint(2)),("animationStyleType",Value::Uint(1)),("layoutTypeValue",Value::Uint(1))] {
            let mut changed=style.clone();changed.set(name,value).unwrap();assert!(!LocalFacts::inspect(&record,Some(&changed),true).direct_box_defaults(),"{name}");
        }
        for (name,value) in [("x",1.),("rotation",0.1),("scaleX",2.)] {
            let mut changed=record.clone();changed.set(name,Value::Float(value)).unwrap();assert!(!LocalFacts::inspect(&changed,Some(&style),true).own_transform_defaults);
        }
        assert!(!LocalFacts::inspect(&record,None,true).direct_box_defaults());
    }
}
