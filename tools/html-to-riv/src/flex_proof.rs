//! Conditional analyzer bridge for actual direct leaf groups. This is not public
//! admission. Inputs retain in-memory scalar identity and finalized record facts.
use super::{
    computed_provenance::NumericSize,
    flex_descriptor::Group,
    flex_numeric::{self as numeric, ErrorEnvelope, Premise},
    flex_sizes::{self, SizeDomains},
    flex_world::{self, WorldDomains},
};
use std::collections::BTreeMap;
#[derive(Debug)]
pub(crate) enum Unresolved {
    Scene(super::flex_scene::Unresolved),
    ParentSize,
    ParentWorld,
    Premise(Premise),
    NativeInput(String),
    CrossTarget,
    Arithmetic,
    CornerBudget,
    Analyzer(numeric::Unresolved),
}
#[derive(Debug)]
pub(crate) struct GroupProof {
    pub main: numeric::Analysis,
    pub cross_sizes: Vec<ErrorEnvelope>,
    pub cross_origins: Vec<ErrorEnvelope>,
    pub far_main: Vec<ErrorEnvelope>,
    pub far_cross: Vec<ErrorEnvelope>,
    pub max_geometry_error: f64,
}
fn require(
    premises: &mut Vec<Premise>,
    condition: bool,
    premise: Premise,
) -> Result<(), Unresolved> {
    if !condition {
        return Err(Unresolved::Premise(premise));
    }
    premises.push(premise);
    Ok(())
}
fn group_proof(
    group: &Group,
    parent: &SizeDomains,
    world: &WorldDomains,
    budget: f64,
) -> Result<GroupProof, Unresolved> {
    group
        .scene_certificate
        .as_ref()
        .map_err(|e| Unresolved::Scene(e.clone()))?;
    let axis = usize::from(!group.row);
    let cross = 1 - axis;
    let parent_main = *parent.axes[axis]
        .as_ref()
        .map_err(|_| Unresolved::ParentSize)?;
    let parent_world = *world.origins[axis]
        .as_ref()
        .map_err(|_| Unresolved::ParentWorld)?;
    let cross_world = *world.origins[cross]
        .as_ref()
        .map_err(|_| Unresolved::ParentWorld)?;
    let facts = group
        .items
        .iter()
        .map(|i| i.local_facts.as_ref())
        .collect::<Option<Vec<_>>>()
        .ok_or(Unresolved::CrossTarget)?;
    let pf = &group.parent_local_facts;
    let mut premises = Vec::new();
    require(
        &mut premises,
        pf.zero_box_insets_and_gaps && facts.iter().all(|f| f.zero_box_insets_and_gaps),
        Premise::ZeroInsetsAndGaps,
    )?;
    require(
        &mut premises,
        pf.no_wrap && facts.iter().all(|f| f.no_wrap),
        Premise::NoWrap,
    )?;
    require(
        &mut premises,
        pf.zero_margins && facts.iter().all(|f| f.zero_margins),
        Premise::NoAutoMargins,
    )?;
    require(
        &mut premises,
        pf.no_aspect_ratio && facts.iter().all(|f| f.no_aspect_ratio),
        Premise::NoAspectRatio,
    )?;
    require(
        &mut premises,
        group.no_local_constraint_or_origin
            && pf.no_style_interpolation
            && facts.iter().all(|f| f.no_style_interpolation),
        Premise::NoConstraintsOrAnimation,
    )?;
    require(
        &mut premises,
        pf.own_transform_defaults && facts.iter().all(|f| f.own_transform_defaults),
        Premise::IdentityOwnTransform,
    )?;
    require(
        &mut premises,
        group.no_local_constraint_or_origin && pf.own_transform_defaults,
        Premise::ZeroOrigins,
    )?;
    require(
        &mut premises,
        pf.no_direction_override && facts.iter().all(|f| f.no_direction_override),
        Premise::LeftToRight,
    )?;
    require(
        &mut premises,
        pf.unset_position_insets && facts.iter().all(|f| f.unset_position_insets),
        Premise::PureTranslation,
    )?;
    let direct = group.helpers.is_empty()
        && group.wrappers.is_empty()
        && group.items.iter().all(|i| {
            i.authored_id == i.native.participant_id && i.native.parent_id == Some(group.parent_id)
        });
    let leaves = facts.iter().all(|f| f.no_layout_children);
    // The final-scene certificate establishes the closed paint-only vocabulary
    // and ancestor defaults; complete child indexing establishes actual leaves.
    require(
        &mut premises,
        direct && leaves && pf.ordinary_flex_layout && facts.iter().all(|f| f.ordinary_flex_layout),
        Premise::KnownTargetPreserved,
    )?;
    require(&mut premises, leaves, Premise::DescendantGeometryAccounted)?;
    let inputs = group
        .items
        .iter()
        .map(|i| {
            i.numeric_input()
                .map_err(|e| Unresolved::NativeInput(format!("{e:?}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    require(
        &mut premises,
        direct && leaves && pf.no_intrinsic_sizing && facts.iter().all(|f| f.no_intrinsic_sizing),
        Premise::NoIntrinsicSizing,
    )?;
    let mut cross_sizes = Vec::new();
    for item in &group.items {
        let n = &item.computed.numeric;
        let raw = &item.native;
        let NumericSize::Pixels(Ok(value)) = &n.cross else {
            return Err(Unresolved::CrossTarget);
        };
        let min_zero = matches!(&n.cross_minimum,NumericSize::Pixels(Ok(v)) if v.is_exact_zero() && v.native()==0.);
        let min_bound = (raw.cross_minimum.units.is_none() && raw.cross_minimum.value.is_none())
            || (raw.cross_minimum.units == Some(1) && raw.cross_minimum.value == Some(0.));
        if !value.is_nonnegative()
            || !item.computed.start_aligned_cross
            || raw.cross_scale != Some(0)
            || raw.cross.units != Some(1)
            || !raw
                .cross
                .value
                .is_some_and(|v| v.to_bits() == value.native().to_bits())
            || !min_zero
            || !min_bound
            || !matches!(n.cross_maximum, NumericSize::Auto)
            || raw.cross_maximum.units.is_some()
            || raw.cross_maximum.value.is_some()
        {
            return Err(Unresolved::CrossTarget);
        }
        cross_sizes.push(
            ErrorEnvelope::new(
                value.ideal_bounds().lower(),
                value.ideal_bounds().upper(),
                value.absolute_error_upper(),
            )
            .map_err(|_| Unresolved::Arithmetic)?,
        );
    }
    premises.push(Premise::CrossStartDefinite); // all final cross fields checked above
    require(
        &mut premises,
        group.native_flow == Some(if group.row { 3 } else { 1 }),
        Premise::NativeReverseFlow,
    )?;
    let alignment = if group.logical_reverse {
        0
    } else if group.row {
        2
    } else {
        6
    };
    require(
        &mut premises,
        group.native_alignment == Some(alignment),
        Premise::PhysicalFlexStart,
    )?;
    let mut ids: Vec<_> = group
        .items
        .iter()
        .map(|i| i.native.participant_id)
        .collect();
    if !group.logical_reverse {
        ids.reverse();
    }
    require(
        &mut premises,
        ids == group.native_participant_file_order,
        Premise::ActualParticipantOrder,
    )?;
    let additional = cross_sizes
        .iter()
        .fold(cross_world.error_upper(), |e, v| e.max(v.error_upper()));
    let main = numeric::analyze(&numeric::GroupDescriptor {
        premises: &premises,
        parent: parent_main,
        parent_world,
        reverse: group.logical_reverse,
        items: &inputs,
        geometry_budget: budget,
        additional_geometry_error: additional,
    })
    .map_err(Unresolved::Analyzer)?;
    let far_main = main
        .positions
        .iter()
        .zip(&main.sizes)
        .map(|(o, s)| o.add(*s).map_err(|_| Unresolved::Arithmetic))
        .collect::<Result<Vec<_>, _>>()?;
    let far_cross = cross_sizes
        .iter()
        .map(|s| cross_world.add(*s).map_err(|_| Unresolved::Arithmetic))
        .collect::<Result<Vec<_>, _>>()?;
    let max_geometry_error = far_main
        .iter()
        .chain(&far_cross)
        .fold(main.max_error_upper, |e, v| e.max(v.error_upper()));
    if max_geometry_error > budget {
        return Err(Unresolved::CornerBudget);
    }
    let cross_origins = vec![cross_world; group.items.len()];
    Ok(GroupProof {
        main,
        cross_sizes,
        cross_origins,
        far_main,
        far_cross,
        max_geometry_error,
    })
}
pub(crate) fn analyze(
    groups: &[Group],
    viewport: [ErrorEnvelope; 2],
    origins: [ErrorEnvelope; 2],
    budget: f64,
) -> BTreeMap<u32, Result<GroupProof, Unresolved>> {
    let sizes = flex_sizes::propagate(groups, viewport);
    let worlds = flex_world::propagate(groups, &sizes, origins);
    groups
        .iter()
        .map(|g| {
            let result = sizes
                .get(&g.parent_id)
                .ok_or(Unresolved::ParentSize)
                .and_then(|s| {
                    worlds
                        .get(&g.parent_id)
                        .ok_or(Unresolved::ParentWorld)
                        .and_then(|w| group_proof(g, s, w, budget))
                });
            (g.parent_id, result)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::super::{FlexPolicy, compile_profile_with_descriptors};
    use super::*;
    use crate::CompileInput;
    #[test]
    fn actual_leaf_group_uses_ancestor_domains_and_explicit_guards() {
        let input = CompileInput {
            html: "<div id=p><div id=a></div><div id=b></div></div>".into(),
            css: "#p{width:50%;height:80px;flex-direction:row}#a,#b{height:20px;flex:.25 .25 20px}"
                .into(),
            width: 400.,
            height: 200.,
        };
        let (output, mut groups) =
            compile_profile_with_descriptors(&input, FlexPolicy::Candidate).unwrap();
        let id = output.source_map[0].object_id;
        let viewport = [ErrorEnvelope::new(0., 16384., 0.001).unwrap(); 2];
        let origins = [ErrorEnvelope::new(0., 0., 0.).unwrap(); 2];
        let result = analyze(&groups, viewport, origins, 0.125);
        assert!(result[&id].is_ok(), "{:?}", result[&id]);
        let huge = [ErrorEnvelope::new(1_000_000., 1_000_000., 1.).unwrap(); 2];
        assert!(analyze(&groups, viewport, huge, 0.125)[&id].is_err());
        groups.iter_mut().find(|g| g.parent_id == id).unwrap().items[0]
            .local_facts
            .as_mut()
            .unwrap()
            .no_layout_children = false;
        assert!(matches!(
            analyze(&groups, viewport, origins, 0.125)[&id],
            Err(Unresolved::Premise(Premise::KnownTargetPreserved))
        ));
    }
    #[test]
    fn actual_four_direction_groups_require_scene_certificate_and_cross_binding() {
        for direction in ["row", "row-reverse", "column", "column-reverse"] {
            let row = direction.starts_with("row");
            let cross = if row { "height" } else { "width" };
            let input = CompileInput {
                html: "<div id=p><div id=a></div><div id=b></div></div>".into(),
                css: format!(
                    "#p{{width:50%;height:50%;flex-direction:{direction}}}#a,#b{{{cross}:20px;flex:.25 .25 20px;background:#ff000080}}#a{{order:1}}"
                ),
                width: 400.,
                height: 200.,
            };
            let (output, mut groups) =
                compile_profile_with_descriptors(&input, FlexPolicy::Candidate).unwrap();
            let id = output.source_map[0].object_id;
            let viewport = [ErrorEnvelope::new(0., 16384., 0.001).unwrap(); 2];
            let origins = [ErrorEnvelope::new(0., 0., 0.).unwrap(); 2];
            assert!(
                analyze(&groups, viewport, origins, 0.125)[&id].is_ok(),
                "{direction}"
            );
            let group = groups.iter_mut().find(|g| g.parent_id == id).unwrap();
            group.items[0].native.cross_scale = Some(1);
            assert!(matches!(
                analyze(&groups, viewport, origins, 0.125)[&id],
                Err(Unresolved::CrossTarget)
            ));
            let group = groups.iter_mut().find(|g| g.parent_id == id).unwrap();
            group.items[0].native.cross_scale = Some(0);
            group.items[0].native.cross_minimum.units = Some(3);
            assert!(matches!(
                analyze(&groups, viewport, origins, 0.125)[&id],
                Err(Unresolved::CrossTarget)
            ));
            let group = groups.iter_mut().find(|g| g.parent_id == id).unwrap();
            group.items[0].native.cross_minimum.units = None;
            group.scene_certificate = Err(super::super::flex_scene::Unresolved::StyleOwnership);
            assert!(matches!(
                analyze(&groups, viewport, origins, 0.125)[&id],
                Err(Unresolved::Scene(_))
            ));
        }
    }
}
