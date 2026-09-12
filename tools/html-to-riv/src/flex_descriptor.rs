//! Mechanical extraction for a future numerical model, never an admission
//! certificate. Authored exact numbers and ancestor errors are not recovered
//! from already-rounded computed floats.
use super::{Direction, Style, Size};
use crate::wire::{Record, Value};
use serde::Serialize;

pub(super) struct Parent {
    direction:Direction,width:Size,height:Size,padding_zero:bool,distributed:bool,
}
impl Parent {pub fn extract(style:&Style)->Self {Self{direction:style.direction,width:style.width,height:style.height,padding_zero:style.padding.is_zero(),distributed:style.spacing.distributes()}}}
pub(super) struct Pending {
    pub parent:Parent,pub parent_id:u32,pub path:String,pub record_start:usize,pub record_end:usize,pub items:Vec<Item>,
}
impl Pending {pub fn finish(self,records:&[Record],constraint_owners:&std::collections::BTreeSet<u32>)->Group {Group::extract(&self.parent,self.parent_id,&self.path,records,self.record_start,self.record_end,self.items,constraint_owners)}}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all="snake_case")]
pub(crate) enum Length { Auto, Pixels(f32), Percent(f32) }
impl From<Size> for Length {
    fn from(value:Size)->Self {match value {Size::Auto=>Self::Auto,Size::Pixels(v)=>Self::Pixels(v),Size::Percent(v)=>Self::Percent(v)}}
}
#[derive(Debug, Serialize)]
pub(crate) struct Computed {
    pub main: Length, pub cross: Length, pub basis: Length,
    pub grow: f32, pub shrink: f32,
    pub minimum: Length, pub maximum: Length,
    pub cross_minimum: Length, pub cross_maximum: Length,
    pub padding_zero: bool, pub margins_zero: bool, pub start_aligned_cross: bool,
}
impl Computed {
    fn extract(style:&Style,direction:Direction)->Self {
        let main=if direction.is_row(){0}else{1};let cross=1-main;
        let sizes=[style.width,style.height];let min=[style.min_width,style.min_height];let max=[style.max_width,style.max_height];
        Self {main:sizes[main].into(),cross:sizes[cross].into(),basis:style.flex.basis.into(),grow:style.flex.grow,shrink:style.flex.shrink,
            minimum:min[main].into(),maximum:max[main].into(),cross_minimum:min[cross].into(),cross_maximum:max[cross].into(),
            padding_zero:style.padding.is_zero(),margins_zero:!style.margins.any(),
            start_aligned_cross:!style.self_alignment.is_baseline() && !style.self_alignment.is_center() && !style.self_alignment.is_end()}
    }
}
#[derive(Debug, Serialize)]
pub(crate) struct RecordLength { pub value:Option<f32>, pub units:Option<u32> }
#[derive(Debug, Serialize)]
pub(crate) struct Native {
    pub participant_id:u32, pub style_id:Option<u32>, pub parent_id:Option<u32>,
    pub main_scale:Option<u32>, pub cross_scale:Option<u32>,
    pub linked_grow:Option<f32>, pub linked_shrink:Option<f32>,
    pub stored_basis:RecordLength, pub main:RecordLength, pub cross:RecordLength,
    pub minimum:RecordLength,pub maximum:RecordLength,
}
#[derive(Debug, Serialize)]
pub(crate) struct Item {
    pub source_id:String,pub source_path:String,pub authored_id:u32,
    pub dom_index:usize,pub css_order:i32,
    pub computed:Computed,pub native:Native,
    pub descendant_target_preservation:Option<bool>,
}
#[derive(Debug, Serialize)]
pub(crate) struct Group {
    pub parent_id:u32,pub parent_path:String,pub row:bool,pub logical_reverse:bool,
    pub native_flow:Option<u32>,pub native_alignment:Option<u32>,
    pub parent_main:Length,pub parent_cross:Length,
    pub native_participant_file_order:Vec<u32>,pub items:Vec<Item>,
    pub helpers:Vec<u32>,pub wrappers:Vec<u32>,
    pub structural_issues:Vec<String>,
    // Null means unresolved, never zero error or an asserted identity ancestor.
    pub parent_main_error:Option<f64>,pub parent_world_error:Option<f64>,
    pub ideal_literal_metadata:Option<String>,
    pub numerical_admission:bool,
    pub unresolved_premises:Vec<&'static str>,
}
pub fn constraint_owners(records:&[Record])->std::collections::BTreeSet<u32> {
    records.iter().filter(|record|record.kind.ends_with("Constraint") || record.kind=="ComponentOrigin").filter_map(|record|uint(record,"parentId")).collect()
}
fn uint(record:&Record,name:&str)->Option<u32>{match record.get(name){Some(Value::Uint(v))=>Some(*v),_=>None}}
fn float(record:&Record,name:&str)->Option<f32>{match record.get(name){Some(Value::Float(v))=>Some(*v),_=>None}}
fn length(record:&Record,name:&str)->RecordLength {RecordLength {value:float(record,name),units:uint(record,&format!("{name}UnitsValue"))}}
fn native(records:&[Record],id:u32,direction:Direction)->Native {
    let record=&records[id as usize+1];let sid=uint(record,"styleId");
    let style=sid.and_then(|id|records.get(id as usize+1)).filter(|record|record.kind=="LayoutComponentStyle");
    let main=if direction.is_row(){"width"}else{"height"};let cross=if direction.is_row(){"height"}else{"width"};
    let main_scale=style.and_then(|s|uint(s,if direction.is_row(){"layoutWidthScaleType"}else{"layoutHeightScaleType"}));
    let fraction=if main_scale==Some(1){float(record,if direction.is_row(){"fractionalWidth"}else{"fractionalHeight"})}else{Some(0.)};
    let read=|name|style.map(|s|length(s,name)).unwrap_or(RecordLength{value:None,units:None});
    Native{participant_id:id,style_id:sid,parent_id:uint(record,"parentId"),main_scale,
        cross_scale:style.and_then(|s|uint(s,if direction.is_row(){"layoutHeightScaleType"}else{"layoutWidthScaleType"})),
        linked_grow:fraction,linked_shrink:fraction,stored_basis:read("flexBasis"),
        main:RecordLength{value:float(record,main),units:style.and_then(|s|uint(s,&format!("{main}UnitsValue")))},
        cross:RecordLength{value:float(record,cross),units:style.and_then(|s|uint(s,&format!("{cross}UnitsValue")))},
        minimum:read(if direction.is_row(){"minWidth"}else{"minHeight"}),maximum:read(if direction.is_row(){"maxWidth"}else{"maxHeight"})}
}
impl Item {
    pub fn extract(style:&Style,direction:Direction,records:&[Record],source_id:&str,source_path:&str,authored_id:u32,participant_id:u32,dom_index:usize,css_order:i32)->Self {
        Self{source_id:source_id.into(),source_path:source_path.into(),authored_id,dom_index,css_order,
            computed:Computed::extract(style,direction),native:native(records,participant_id,direction),descendant_target_preservation:None}
    }
}
impl Group {
    fn extract(parent:&Parent,parent_id:u32,path:&str,records:&[Record],record_start:usize,record_end:usize,mut items:Vec<Item>,constraint_owners:&std::collections::BTreeSet<u32>)->Self {
        let direction=parent.direction;
        let parent_record=&records[parent_id as usize+1];
        let parent_style=uint(parent_record,"styleId").and_then(|id|records.get(id as usize+1));
        let flow=parent_style.and_then(|s|uint(s,"flexDirectionValue"));
        let alignment=parent_style.and_then(|s|uint(s,"layoutAlignmentType"));
        let native_order:Vec<u32>=records.iter().enumerate().take(record_end).skip(record_start).filter(|(_,r)|r.kind=="LayoutComponent" && uint(r,"parentId")==Some(parent_id)).map(|(index,_)|index as u32-1).collect();
        let expected:Vec<_>=items.iter().map(|i|i.native.participant_id).collect();
        let expected_set:std::collections::BTreeSet<_>=expected.iter().copied().collect();
        let helpers=native_order.iter().copied().filter(|id|!expected_set.contains(id)).collect::<Vec<_>>();
        let wrappers=items.iter().filter(|i|i.authored_id!=i.native.participant_id).map(|i|i.native.participant_id).collect::<Vec<_>>();
        let mut issues=Vec::new();
        if native_order!=expected {issues.push("actual native participants differ from authored participant emission order".into());}
        if !helpers.is_empty(){issues.push("generated helper participants require a different arithmetic model".into());}
        if !wrappers.is_empty(){issues.push("alignment wrapper topology is outside the direct participant model".into());}
        if !parent.padding_zero || parent.distributed{issues.push("parent padding or distributed spacing is outside the model".into());}
        if flow!=Some(direction.wire()) || alignment!=Some(direction.alignment()){issues.push("native flow/alignment differs from physical flex-start lowering".into());}
        for item in &items {
            let c=&item.computed;
            if !c.padding_zero || !c.margins_zero {issues.push(format!("{}: padding or automatic margin",item.source_path));}
            if !matches!(c.cross,Length::Pixels(_)) || !c.start_aligned_cross || !matches!(c.cross_minimum,Length::Pixels(0.)) || !matches!(c.cross_maximum,Length::Auto) {issues.push(format!("{}: cross target/origin needs separate proof",item.source_path));}
            if !matches!(c.minimum,Length::Pixels(_)) || !matches!(c.maximum,Length::Auto|Length::Pixels(_)){issues.push(format!("{}: responsive/intrinsic bounds",item.source_path));}
            let legacy=item.native.linked_grow==Some(0.) && matches!(c.basis,Length::Auto) && c.grow==0. && c.shrink==0.;
            if legacy {
                if !matches!(c.main,Length::Pixels(_)){issues.push(format!("{}: intrinsic/percentage fixed basis",item.source_path));}
            }else if !matches!(c.main,Length::Auto) || !matches!(c.basis,Length::Pixels(_)){issues.push(format!("{}: flexible basis/preferred-size profile unresolved",item.source_path));}
            if item.native.linked_grow.is_none(){issues.push(format!("{}: missing explicit native Fill fraction",item.source_path));}
        }
        // The model consumes logical order; retain actual file order separately.
        items.sort_by_key(|item|(item.css_order,item.dom_index));
        // Record mutation/constraint machinery must never silently become a rigid
        // transform assertion. Ancestor/world facts remain unresolved regardless.
        let ids:std::collections::BTreeSet<_>=std::iter::once(parent_id).chain(items.iter().flat_map(|i|[i.authored_id,i.native.participant_id])).collect();
        if ids.iter().any(|id|constraint_owners.contains(id)) {issues.push("participant or parent has a constraint/origin helper".into());}
        let main=if direction.is_row(){parent.width}else{parent.height};let cross=if direction.is_row(){parent.height}else{parent.width};
        Self{parent_id,parent_path:path.into(),row:direction.is_row(),logical_reverse:!direction.reverses_emission(),native_flow:flow,native_alignment:alignment,
            parent_main:main.into(),parent_cross:cross.into(),native_participant_file_order:native_order,items,helpers,wrappers,structural_issues:issues,
            parent_main_error:None,parent_world_error:None,ideal_literal_metadata:None,numerical_admission:false,unresolved_premises:vec!["missing_exact_literal_provenance","parent_size_error","ancestor_world_error","native_default_transform_provenance","descendant_target_preservation"]}
    }
}

#[cfg(test)]
mod tests {
    use super::super::{compile_profile,compile_profile_with_descriptors,FlexPolicy};
    use crate::CompileInput;
    fn request(css:&str)->CompileInput {CompileInput {html:"<div id=p><div id=a></div><div id=b></div><div id=c></div></div>".into(),css:format!("#p{{width:160px;height:120px;flex-direction:row}}#a,#b,#c{{height:20px}}{css}"),width:240.,height:160.}}
    #[test]
    fn actual_pipeline_extracts_ordered_native_facts_without_changing_bytes() {
        for direction in ["row","row-reverse","column","column-reverse"] {
            let row=direction.starts_with("row");
            let mut input=request(&format!("#p{{flex-direction:{direction}}}#a{{flex:.25 7 0px;order:2}}#b{{order:-1}}#c{{flex:.5 0 0px;order:2}}"));
            input.css.push_str(if row {"#b{width:40px}"}else{"#a,#b,#c{width:20px;height:auto}#b{height:40px}"});
            let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Candidate).unwrap();
            assert_eq!(output,compile_profile(&input,FlexPolicy::Candidate).unwrap());
            let parent=output.source_map[0].object_id;
            let group=groups.iter().find(|g|g.parent_id==parent).unwrap();
            assert_eq!(group.items.iter().map(|i|i.source_id.as_str()).collect::<Vec<_>>(),["b","a","c"]);
            let a=&group.items[1];assert_eq!(a.computed.shrink,7.);assert_eq!(a.native.linked_grow,Some(0.25));assert_eq!(a.native.linked_shrink,Some(0.25));
            assert_eq!(a.native.stored_basis.value,Some(0.));assert_eq!(a.native.stored_basis.units,Some(1));
            assert_eq!(group.items[0].native.main.value,Some(40.));assert_eq!(group.items[0].native.cross.value,Some(20.));
            let logical:Vec<_>=group.items.iter().map(|i|i.native.participant_id).collect();
            assert_eq!(group.native_participant_file_order,if direction.ends_with("reverse"){logical}else{logical.into_iter().rev().collect::<Vec<_>>()});
            assert!(group.structural_issues.is_empty(),"{:?}",group.structural_issues);
            assert!(!group.numerical_admission);assert!(group.parent_main_error.is_none());assert!(group.parent_world_error.is_none());assert!(group.ideal_literal_metadata.is_none());
            let json=serde_json::to_value(group).unwrap();assert!(json["parent_main_error"].is_null());
        }
    }
    #[test]
    fn emitted_wrappers_and_spacing_helpers_remain_explicit() {
        let (_,groups)=compile_profile_with_descriptors(&request("#a{flex:1 1 0px;align-self:center}"),FlexPolicy::Candidate).unwrap();
        assert!(groups.iter().any(|g|!g.wrappers.is_empty() && !g.structural_issues.is_empty()));
        let (_,groups)=compile_profile_with_descriptors(&request("#p{justify-content:space-around}#a,#b,#c{width:20px}"),FlexPolicy::Guarded).unwrap();
        let group=groups.iter().find(|g|g.items.len()==3).unwrap();assert_eq!(group.helpers.len(),4);
    }
    #[test]
    fn constraints_added_by_ancestors_are_checked_after_the_full_emission() {
        let mut input=request("#a{width:30px;align-self:baseline}#leaf{height:10px}");
        input.html="<div id=p><div id=a><div id=leaf></div></div></div>".into();
        let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Guarded).unwrap();
        let a=output.source_map.iter().find(|n|n.id=="a").unwrap().object_id;
        assert!(groups.iter().find(|g|g.parent_id==a).unwrap().structural_issues.iter().any(|v|v.contains("constraint")));
    }
}
