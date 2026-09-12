//! Mechanical extraction for a future numerical model, never an admission
//! certificate. Authored exact numbers and ancestor errors are not recovered
//! from already-rounded computed floats.
use super::{Direction, Style, Size};
use crate::wire::{Record, Value};
use serde::{Serialize, Serializer};
use super::computed_provenance::{NumericSize, Scalar};

pub(super) struct Parent {
    direction:Direction,width:Size,height:Size,numeric_width:NumericSize,numeric_height:NumericSize,padding_zero:bool,distributed:bool,
}
impl Parent {pub fn extract(style:&Style)->Self {Self{direction:style.direction,width:style.width,height:style.height,numeric_width:style.numeric.width.clone(),numeric_height:style.numeric.height.clone(),padding_zero:style.padding.is_zero() && style.gap.is_zero(),distributed:style.spacing.distributes()}}}
pub(super) struct Pending {
    pub parent:Parent,pub parent_id:u32,pub path:String,pub record_start:usize,pub record_end:usize,pub items:Vec<Item>,
}
impl Pending {pub fn finish(self,records:&[Record],scene:&SceneIndex)->Group {Group::extract(&self.parent,self.parent_id,&self.path,records,self.record_start,self.record_end,self.items,scene)}}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all="snake_case")]
pub(crate) enum Length { Auto, Pixels(f32), Percent(f32) }
impl From<Size> for Length {
    fn from(value:Size)->Self {match value {Size::Auto=>Self::Auto,Size::Pixels(v)=>Self::Pixels(v),Size::Percent(v)=>Self::Percent(v)}}
}
// Retain the actual carriers (including equality identity) for the analyzer.
// JSON is diagnostic output only and must never be deserialized as a proof.
fn scalar_json(value: &Scalar) -> serde_json::Value {
    match value {
        Ok(v) => serde_json::json!({"native_bits":v.native().to_bits(),
            "ideal_lower":v.ideal_bounds().lower(),"ideal_upper":v.ideal_bounds().upper(),
            "absolute_error_upper":v.absolute_error_upper(),"exact_zero":v.is_exact_zero(),
            "nonnegative":v.is_nonnegative()}),
        Err(reason) => serde_json::json!({"unresolved":format!("{reason:?}")}),
    }
}
fn serialize_scalar<S:Serializer>(value:&Scalar,serializer:S)->Result<S::Ok,S::Error> {
    scalar_json(value).serialize(serializer)
}
fn serialize_size<S:Serializer>(value:&NumericSize,serializer:S)->Result<S::Ok,S::Error> {
    let json=match value {
        NumericSize::Auto=>serde_json::json!({"kind":"auto"}),
        NumericSize::Pixels(v)=>serde_json::json!({"kind":"pixels","scalar":scalar_json(v)}),
        NumericSize::Percent(v)=>serde_json::json!({"kind":"percent_coefficient","scalar":scalar_json(v)}),
    };
    json.serialize(serializer)
}
#[derive(Debug, Serialize)]
pub(crate) struct NumericFacts {
    #[serde(serialize_with="serialize_size")]
    pub main:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub cross:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub basis:NumericSize,
    #[serde(serialize_with="serialize_scalar")]
    pub grow:Scalar,
    #[serde(serialize_with="serialize_scalar")]
    pub shrink:Scalar,
    #[serde(serialize_with="serialize_size")]
    pub minimum:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub maximum:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub cross_minimum:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub cross_maximum:NumericSize,
}
fn size_known(value:&NumericSize)->bool {match value {NumericSize::Auto=>true,NumericSize::Pixels(v)|NumericSize::Percent(v)=>v.is_ok()}}
impl NumericFacts {
    fn available(&self)->bool {self.grow.is_ok() && self.shrink.is_ok() && [&self.main,&self.cross,&self.basis,&self.minimum,&self.maximum,&self.cross_minimum,&self.cross_maximum].into_iter().all(size_known)}
    fn extract(style:&Style,direction:Direction)->Self {
        let n=&style.numeric;
        let main=if direction.is_row(){0}else{1};let cross=1-main;
        let size=[&n.width,&n.height];let min=[&n.min_width,&n.min_height];let max=[&n.max_width,&n.max_height];
        Self{main:size[main].clone(),cross:size[cross].clone(),basis:n.flex.basis.clone(),
            grow:n.flex.grow.clone(),shrink:n.flex.shrink.clone(),minimum:min[main].clone(),maximum:max[main].clone(),
            cross_minimum:min[cross].clone(),cross_maximum:max[cross].clone()}
    }
}
#[derive(Debug, Serialize)]
pub(crate) struct Computed {
    pub numeric: NumericFacts,
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
        Self {numeric:NumericFacts::extract(style,direction),main:sizes[main].into(),cross:sizes[cross].into(),basis:style.flex.basis.into(),grow:style.flex.grow,shrink:style.flex.shrink,
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
    pub cross_minimum:RecordLength,pub cross_maximum:RecordLength,
}
#[derive(Debug, Serialize)]
pub(crate) struct Item {
    pub source_id:String,pub source_path:String,pub authored_id:u32,
    pub dom_index:usize,pub css_order:i32,
    pub computed:Computed,pub native:Native,
    pub descendant_target_preservation:Option<bool>,
    pub local_facts:Option<super::flex_structure::LocalFacts>,
}
#[derive(Debug, Serialize)]
pub(crate) struct Group {
    pub parent_id:u32,pub parent_path:String,pub row:bool,pub logical_reverse:bool,
    pub native_flow:Option<u32>,pub native_alignment:Option<u32>,
    pub parent_local_facts:super::flex_structure::LocalFacts,
    pub parent_main:Length,pub parent_cross:Length,
    #[serde(serialize_with="serialize_size")]
    pub parent_numeric_main:NumericSize,
    #[serde(serialize_with="serialize_size")]
    pub parent_numeric_cross:NumericSize,
    pub native_participant_file_order:Vec<u32>,pub items:Vec<Item>,
    pub helpers:Vec<u32>,pub wrappers:Vec<u32>,
    pub structural_issues:Vec<String>,
    pub no_local_constraint_or_origin:bool,
    // Null means unresolved, never zero error or an asserted identity ancestor.
    pub parent_main_error:Option<f64>,pub parent_world_error:Option<f64>,
    pub ideal_literal_metadata:Option<String>,
    pub numerical_admission:bool,
    pub unresolved_premises:Vec<&'static str>,
}
#[derive(Default)]
pub(super) struct SceneIndex {
    constraints:std::collections::BTreeSet<u32>,
    layout_children:std::collections::BTreeMap<u32,Vec<u32>>,
}
pub(super) fn scene_index(records:&[Record])->SceneIndex {
    let mut index=SceneIndex::default();
    for (position,record) in records.iter().enumerate().skip(1) {
        if let Some(parent)=uint(record,"parentId") {
            if record.kind=="LayoutComponent" {index.layout_children.entry(parent).or_default().push(position as u32-1);}
            if record.kind.ends_with("Constraint") || record.kind=="ComponentOrigin" {index.constraints.insert(parent);}
        }
    }
    index
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
        minimum:read(if direction.is_row(){"minWidth"}else{"minHeight"}),maximum:read(if direction.is_row(){"maxWidth"}else{"maxHeight"}),
        cross_minimum:read(if direction.is_row(){"minHeight"}else{"minWidth"}),cross_maximum:read(if direction.is_row(){"maxHeight"}else{"maxWidth"})}
}
/// Failure to bind a typed value to the actual emitted participant. This is
/// separate from structural/numerical qualification of the whole group.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InputIssue { Wrapper, MissingScalar(&'static str), NonPoint(&'static str), NativeMismatch(&'static str) }
fn scalar(value:&Scalar,name:&'static str)->Result<super::scalar_provenance::ScalarProvenance,InputIssue> {
    value.clone().map_err(|_|InputIssue::MissingScalar(name))
}
fn point(value:&NumericSize,name:&'static str)->Result<super::scalar_provenance::ScalarProvenance,InputIssue> {
    match value {NumericSize::Pixels(v)=>scalar(v,name),_=>Err(InputIssue::NonPoint(name))}
}
fn same_bits(actual:Option<f32>,expected:f32)->bool {actual.is_some_and(|v|v.to_bits()==expected.to_bits())}
fn point_binding(record:&RecordLength,value:f32)->bool {record.units==Some(1) && same_bits(record.value,value)}
impl Item {
    #[allow(dead_code)] // Used by the next record-indexed group qualification stage.
    pub(crate) fn numeric_input(&self)->Result<super::flex_numeric::Item,InputIssue> {
        if self.authored_id!=self.native.participant_id {return Err(InputIssue::Wrapper);}
        let c=&self.computed;let n=&c.numeric;let native=&self.native;
        let grow=scalar(&n.grow,"grow")?;let shrink=scalar(&n.shrink,"shrink")?;
        if !same_bits(native.linked_grow,grow.native()) || !same_bits(native.linked_shrink,grow.native()) {
            return Err(InputIssue::NativeMismatch("linked factors"));
        }
        // Keep authored shrink even where the runtime links it to grow. The
        // analyzer must prove equality or whole-group zero-basis irrelevance.
        let legacy=c.grow==0. && c.shrink==0. && matches!(c.basis,Length::Auto);
        let basis=if legacy {
            let value=point(&n.main,"fixed main size")?;
            if native.main_scale!=Some(0) || !point_binding(&native.main,value.native()) {return Err(InputIssue::NativeMismatch("fixed main size"));}
            value
        } else {
            let value=point(&n.basis,"basis")?;
            if native.main_scale!=Some(1) || !matches!(c.main,Length::Auto) || !point_binding(&native.stored_basis,value.native()) {return Err(InputIssue::NativeMismatch("basis"));}
            value
        };
        let minimum=point(&n.minimum,"minimum")?;
        // Emitter omits zero minima and absent maxima. These bindings describe
        // that compiler encoding; importer default qualification remains a
        // separate group premise, not something this conversion establishes.
        let omitted_min=native.minimum.units.is_none() && native.minimum.value.is_none() && minimum.native()==0.;
        if !omitted_min && !point_binding(&native.minimum,minimum.native()) {return Err(InputIssue::NativeMismatch("minimum"));}
        let maximum=match &n.maximum {
            NumericSize::Auto if native.maximum.units.is_none() && native.maximum.value.is_none()=>None,
            NumericSize::Pixels(v)=>{let value=scalar(v,"maximum")?;if !point_binding(&native.maximum,value.native()){return Err(InputIssue::NativeMismatch("maximum"));}Some(value)},
            _=>return Err(InputIssue::NonPoint("maximum")),
        };
        Ok(super::flex_numeric::Item{basis,grow,shrink,min:minimum,max:maximum})
    }
    pub fn extract(style:&Style,direction:Direction,records:&[Record],source_id:&str,source_path:&str,authored_id:u32,participant_id:u32,dom_index:usize,css_order:i32)->Self {
        Self{source_id:source_id.into(),source_path:source_path.into(),authored_id,dom_index,css_order,
            computed:Computed::extract(style,direction),native:native(records,participant_id,direction),descendant_target_preservation:None,local_facts:None}
    }
}
impl Group {
    fn extract(parent:&Parent,parent_id:u32,path:&str,records:&[Record],_record_start:usize,_record_end:usize,mut items:Vec<Item>,scene:&SceneIndex)->Self {
        let direction=parent.direction;
        // Capture happens before descendant/ancestor helpers finish mutating the
        // scene. Bind inputs to final records, never an earlier native snapshot.
        for item in &mut items {
            item.native=native(records,item.native.participant_id,direction);
            let record=&records[item.native.participant_id as usize+1];
            let style=item.native.style_id.and_then(|id|records.get(id as usize+1));
            let leaf=!scene.layout_children.contains_key(&item.native.participant_id);
            item.local_facts=Some(super::flex_structure::LocalFacts::inspect(record,style,leaf));
        }
        let parent_record=&records[parent_id as usize+1];
        let parent_style=uint(parent_record,"styleId").and_then(|id|records.get(id as usize+1));
        let flow=parent_style.and_then(|s|uint(s,"flexDirectionValue"));
        let alignment=parent_style.and_then(|s|uint(s,"layoutAlignmentType"));
        let native_order=scene.layout_children.get(&parent_id).cloned().unwrap_or_default();
        let expected:Vec<_>=items.iter().map(|i|i.native.participant_id).collect();
        let expected_set:std::collections::BTreeSet<_>=expected.iter().copied().collect();
        let helpers=native_order.iter().copied().filter(|id|!expected_set.contains(id)).collect::<Vec<_>>();
        let wrappers=items.iter().filter(|i|i.authored_id!=i.native.participant_id).map(|i|i.native.participant_id).collect::<Vec<_>>();
        let parent_local_facts=super::flex_structure::LocalFacts::inspect(parent_record,parent_style,native_order.is_empty());
        let mut issues=Vec::new();
        if !parent_local_facts.direct_box_defaults(){issues.push("parent native local defaults/overrides require separate proof".into());}
        if native_order!=expected {issues.push("actual native participants differ from authored participant emission order".into());}
        if !helpers.is_empty(){issues.push("generated helper participants require a different arithmetic model".into());}
        if !wrappers.is_empty(){issues.push("alignment wrapper topology is outside the direct participant model".into());}
        if !parent.padding_zero || parent.distributed{issues.push("parent padding, gaps or distributed spacing is outside the model".into());}
        if flow!=Some(direction.wire()) || alignment!=Some(direction.alignment()){issues.push("native flow/alignment differs from physical flex-start lowering".into());}
        for item in &items {
            if !item.local_facts.as_ref().is_some_and(|f|f.direct_box_defaults()) {issues.push(format!("{}: native local defaults/overrides require separate proof",item.source_path));}
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
        let no_local_constraint_or_origin=!ids.iter().any(|id|scene.constraints.contains(id));
        if !no_local_constraint_or_origin {issues.push("participant or parent has a constraint/origin helper".into());}
        let main=if direction.is_row(){parent.width}else{parent.height};let cross=if direction.is_row(){parent.height}else{parent.width};
        let metadata_available=size_known(&parent.numeric_width) && size_known(&parent.numeric_height) && items.iter().all(|item|item.computed.numeric.available());
        let mut unresolved_premises=vec!["parent_size_error","ancestor_world_error","native_default_transform_provenance","descendant_target_preservation"];
        if !metadata_available {unresolved_premises.push("missing_exact_literal_provenance");}
        Self{parent_id,parent_path:path.into(),row:direction.is_row(),logical_reverse:!direction.reverses_emission(),native_flow:flow,native_alignment:alignment,
            parent_local_facts,parent_main:main.into(),parent_cross:cross.into(),
            parent_numeric_main:if direction.is_row(){parent.numeric_width.clone()}else{parent.numeric_height.clone()},
            parent_numeric_cross:if direction.is_row(){parent.numeric_height.clone()}else{parent.numeric_width.clone()},native_participant_file_order:native_order,items,helpers,wrappers,structural_issues:issues,no_local_constraint_or_origin,
            parent_main_error:None,parent_world_error:None,ideal_literal_metadata:metadata_available.then(||"computed_scalar_provenance".into()),numerical_admission:false,unresolved_premises}
    }
}

#[cfg(test)]
mod tests {
    use super::{NumericSize,InputIssue};
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
            for item in &group.items { assert!(item.numeric_input().is_ok(),"{:?}",item.numeric_input()); }
            let a=&group.items[1];assert_eq!(a.computed.shrink,7.);assert_eq!(a.native.linked_grow,Some(0.25));assert_eq!(a.native.linked_shrink,Some(0.25));
            assert_eq!(a.native.stored_basis.value,Some(0.));assert_eq!(a.native.stored_basis.units,Some(1));
            assert_eq!(group.items[0].native.main.value,Some(40.));assert_eq!(group.items[0].native.cross.value,Some(20.));
            let logical:Vec<_>=group.items.iter().map(|i|i.native.participant_id).collect();
            assert_eq!(group.native_participant_file_order,if direction.ends_with("reverse"){logical}else{logical.into_iter().rev().collect::<Vec<_>>()});
            assert!(group.structural_issues.is_empty(),"{:?}",group.structural_issues);
            assert!(group.parent_local_facts.direct_box_defaults());
            assert!(group.items.iter().all(|i|i.local_facts.as_ref().is_some_and(|f|f.no_layout_children)));
            assert!(!group.numerical_admission);assert!(group.parent_main_error.is_none());assert!(group.parent_world_error.is_none());assert!(group.ideal_literal_metadata.is_some());
            let json=serde_json::to_value(group).unwrap();assert!(json["parent_main_error"].is_null());
        }
    }
    #[test]
    fn descriptors_retain_original_ideals_and_deferred_percentages() {
        let input=request("#p{width:33.333333333%}#a{--f:.25000000000000001;flex:.25 var(--f) 0px}#b{width:100.71428680419922px}");
        let (output,groups)=compile_profile_with_descriptors(&input,FlexPolicy::Candidate).unwrap();
        assert_eq!(output,compile_profile(&input,FlexPolicy::Candidate).unwrap());
        let group=groups.iter().find(|g|g.items.len()==3).unwrap();
        let a=&group.items.iter().find(|i|i.source_id=="a").unwrap().computed.numeric;
        let grow=a.grow.as_ref().unwrap();let shrink=a.shrink.as_ref().unwrap();
        assert_eq!(grow.native().to_bits(),shrink.native().to_bits());
        assert!(!grow.proves_equal(shrink));
        let b=&group.items.iter().find(|i|i.source_id=="b").unwrap().computed.numeric;
        let NumericSize::Pixels(Ok(width))=&b.main else {panic!("missing width metadata")};
        assert!(width.absolute_error_upper()>0.0002);
        let NumericSize::Percent(Ok(percent))=&group.parent_numeric_main else {panic!("parent coefficient was lost")};
        assert!(percent.ideal_bounds().lower()<=33.333333333 && percent.ideal_bounds().upper()>=33.333333333);
        let json=serde_json::to_value(group).unwrap();
        assert_eq!(json["parent_numeric_main"]["kind"],"percent_coefficient");
        assert!(!group.numerical_admission);
        assert!(group.unresolved_premises.contains(&"parent_size_error"));
        assert!(!group.unresolved_premises.contains(&"missing_exact_literal_provenance"));
    }
    #[test]
    fn numeric_inputs_reject_tampered_bindings_and_keep_authored_shrink() {
        let (_,mut groups)=compile_profile_with_descriptors(&request("#a{flex:.25 7 0px}#b,#c{width:20px}"),FlexPolicy::Candidate).unwrap();
        let group=groups.iter_mut().find(|g|g.items.len()==3).unwrap();
        let item=group.items.iter_mut().find(|i|i.source_id=="a").unwrap();
        let input=item.numeric_input().unwrap();assert_eq!(input.grow.native(),0.25);assert_eq!(input.shrink.native(),7.);
        item.native.linked_shrink=Some(7.);
        assert_eq!(item.numeric_input().unwrap_err(),InputIssue::NativeMismatch("linked factors"));
        item.native.linked_shrink=Some(0.25);item.native.stored_basis.value=Some(1.);
        assert_eq!(item.numeric_input().unwrap_err(),InputIssue::NativeMismatch("basis"));
        item.native.stored_basis.value=Some(0.);item.native.minimum.units=Some(2);
        assert_eq!(item.numeric_input().unwrap_err(),InputIssue::NativeMismatch("minimum"));
    }
    #[test]
    fn finalization_refreshes_native_values_after_capture() {
        use super::super::{Style,Direction,Size};
        use super::{Record,Value,Item,Parent,Group};
        let mut records=vec![Record::new("Backboard"),Record::new("Artboard"),Record::new("LayoutComponent"),Record::new("LayoutComponentStyle")];
        records[2].set("parentId",Value::Uint(0)).unwrap();
        records[2].set("styleId",Value::Uint(2)).unwrap();
        records[2].set("width",Value::Float(20.)).unwrap();
        records[3].set("widthUnitsValue",Value::Uint(1)).unwrap();
        records[3].set("layoutWidthScaleType",Value::Uint(0)).unwrap();
        let mut style=Style{width:Size::Pixels(20.),..Style::default()};
        style.numeric.width=NumericSize::Pixels(Ok(super::super::scalar_provenance::ScalarProvenance::exact_constant(20.).unwrap()));
        let item=Item::extract(&style,Direction::Row,&records,"a","a",1,1,0,0);
        assert!(item.numeric_input().is_ok());
        records[2].set("width",Value::Float(25.)).unwrap();
        let parent=Parent::extract(&Style{direction:Direction::Row,..Style::default()});
        // A late helper lies outside the original captured range.
        let mut late=Record::new("LayoutComponent");late.set("parentId",Value::Uint(1)).unwrap();records.push(late);
        let group=Group::extract(&parent,0,"",&records,2,4,vec![item],&super::scene_index(&records));
        assert!(!group.items[0].local_facts.as_ref().unwrap().no_layout_children);
        assert_eq!(group.items[0].native.main.value,Some(25.));
        assert_eq!(group.items[0].numeric_input().unwrap_err(),InputIssue::NativeMismatch("fixed main size"));
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
